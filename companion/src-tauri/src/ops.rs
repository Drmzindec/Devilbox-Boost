//! Per-project operations backed by the tools/ scripts: doctor, database
//! snapshots, background workers, production media fallback, logs, host-side
//! Vite dev servers and SQL client detection.

use crate::sites::{run_streaming, tool, tool_output};
use crate::{root_or_err, AppState};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

// ---------------------------------------------------------------------------
// Doctor
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
pub struct Finding {
    pub level: String,
    pub scope: String,
    pub message: String,
}

fn parse_findings(text: &str) -> Vec<Finding> {
    text.lines()
        .filter_map(|l| {
            let mut p = l.splitn(3, '|');
            let level = p.next()?;
            if !matches!(level, "OK" | "WARN" | "FAIL") {
                return None;
            }
            Some(Finding { level: level.to_string(), scope: p.next()?.to_string(), message: p.next().unwrap_or("").to_string() })
        })
        .collect()
}

/// Run the doctor. `project` = None checks the stack and every site.
#[tauri::command]
pub async fn doctor(app: AppHandle, state: State<'_, AppState>, project: Option<String>) -> Result<Vec<Finding>, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let args: Vec<&str> = match project.as_deref() {
            Some(p) => vec![p],
            None => vec![],
        };
        let text = run_streaming(&app, "doctor", tool(&root, "doctor.sh", &args)).unwrap_or_else(|e| e);
        Ok(parse_findings(&text))
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Doctor fixes
// ---------------------------------------------------------------------------

fn valid_project(root: &Path, p: &str) -> Result<(), String> {
    if p.is_empty() || !p.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')) {
        return Err(format!("invalid project name {p}"));
    }
    if !root.join("data/www").join(p).is_dir() {
        return Err(format!("project {p} not found"));
    }
    Ok(())
}

/// `bash -lc <cmd>` inside the PHP container, in the project folder, as devilbox.
fn in_project(root: &Path, project: &str, cmd: &str) -> Command {
    let mut c = Command::new(crate::docker_path());
    c.args(["compose", "exec", "-T", "--user", "devilbox", "-w", &format!("/shared/httpd/{project}"), "php", "bash", "-lc", cmd])
        .current_dir(root);
    c
}

/// Deactivate every active plugin whose files are missing on disk — the repo
/// is the source of truth, so a stale active_plugins entry gets removed
/// rather than the plugin downloaded.
const WP_CLEAN_PLUGINS: &str = r#"wp eval '$a = get_option("active_plugins"); $k = array_values(array_filter($a, function ($p) { return file_exists(WP_PLUGIN_DIR . "/" . $p); })); $r = array_diff($a, $k); update_option("active_plugins", $k); echo count($r) ? "deactivated: " . implode(", ", $r) . "\n" : "nothing to deactivate\n";'"#;

/// Apply one of the known doctor fixes. Streams output as job `fix-<scope>`.
#[tauri::command]
pub async fn apply_fix(app: AppHandle, state: State<'_, AppState>, fix: String, project: Option<String>, arg: Option<String>) -> Result<String, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let job = format!("fix-{}", project.as_deref().unwrap_or("stack"));
        let run = |cmd: Command| run_streaming(&app, &job, cmd);
        match fix.as_str() {
            "composer-install" | "npm-build" | "migrate" | "wp-clean-plugins" => {
                let p = project.as_deref().ok_or("this fix needs a project")?;
                valid_project(&root, p)?;
                let cmd = match fix.as_str() {
                    "composer-install" => "composer install --no-interaction",
                    "npm-build" => "npm install --no-audit --no-fund && npm run build",
                    "migrate" => "php artisan migrate --force",
                    _ => WP_CLEAN_PLUGINS,
                };
                run(in_project(&root, p, cmd))
            }
            "create-db" => {
                let db = arg.as_deref().ok_or("create-db needs a database name")?;
                if db.is_empty() || !db.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                    return Err(format!("invalid database name {db}"));
                }
                let env = crate::environment_for(&root);
                let pw = format!("-p{}", crate::shell_quote(&env.mysql_root_password));
                crate::compose(&root, &["exec", "-T", "mysql", "sh", "-c", &format!(
                    "mariadb -uroot {pw} -e 'CREATE DATABASE IF NOT EXISTS `{db}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci' 2>/dev/null || mysql -uroot {pw} -e 'CREATE DATABASE IF NOT EXISTS `{db}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci'"
                )]).map(|_| format!("Database {db} created (or already present)\n"))
            }
            "trust-ca" => {
                let home = std::env::var("HOME").map_err(|e| e.to_string())?;
                let mut c = Command::new("security");
                c.args(["add-trusted-cert", "-r", "trustRoot", "-k", &format!("{home}/Library/Keychains/login.keychain-db")])
                    .arg(root.join("ca/devilbox-ca.crt"));
                run(c).map(|o| if o.trim().is_empty() { "Devilbox CA trusted in the login keychain\n".into() } else { o })
            }
            "resolver-loc" => {
                // Writing /etc/resolver needs admin rights: macOS prompts for the password.
                let env = crate::environment_for(&root);
                let port = crate::read_env_value(&root, "HOST_PORT_BIND").unwrap_or_else(|| "1053".into());
                let tld = env.tld;
                let mut c = Command::new("osascript");
                c.arg("-e").arg(format!(
                    "do shell script \"mkdir -p /etc/resolver && printf 'nameserver 127.0.0.1\\nport {port}\\n' > /etc/resolver/{tld}\" with administrator privileges"
                ));
                run(c).map(|o| if o.trim().is_empty() { format!("/etc/resolver/{tld} written (nameserver 127.0.0.1, port {port})\n") } else { o })
            }
            "image-prune" => {
                let mut c = Command::new(crate::docker_path());
                c.args(["image", "prune", "-f"]).current_dir(&root);
                run(c)
            }
            other => Err(format!("unknown fix {other}")),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Snapshots
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
pub struct Snapshot {
    pub file: String,
    pub name: String,
    pub size: u64,
    pub modified: u64,
}

fn parse_snapshots(text: &str) -> Vec<Snapshot> {
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('|').collect();
            if f.len() != 3 || !f[0].ends_with(".sql.gz") {
                return None;
            }
            Some(Snapshot {
                file: f[0].to_string(),
                name: Path::new(f[0]).file_name()?.to_string_lossy().to_string(),
                size: f[1].parse().unwrap_or(0),
                modified: f[2].parse().unwrap_or(0),
            })
        })
        .collect()
}

#[tauri::command]
pub async fn snapshots(state: State<'_, AppState>, project: String) -> Result<Vec<Snapshot>, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || Ok(parse_snapshots(&tool_output(&root, "snapshot.sh", &[&project, "list"])?)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn snapshot_action(state: State<'_, AppState>, project: String, action: String, arg: Option<String>) -> Result<String, String> {
    let root = root_or_err(&state)?;
    if !matches!(action.as_str(), "create" | "restore" | "delete" | "prune") {
        return Err(format!("unknown snapshot action {action}"));
    }
    tauri::async_runtime::spawn_blocking(move || {
        let mut args = vec![project.as_str(), action.as_str()];
        if let Some(a) = arg.as_deref() {
            args.push(a);
        }
        tool_output(&root, "snapshot.sh", &args)
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Workers (inside the PHP container)
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
pub struct Worker {
    pub name: String,
    pub running: bool,
    pub pid: String,
}

fn parse_workers(text: &str) -> Vec<Worker> {
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('|').collect();
            match f.as_slice() {
                ["WORKER", name, state, pid] => Some(Worker { name: name.to_string(), running: *state == "running", pid: pid.to_string() }),
                _ => None,
            }
        })
        .collect()
}

#[tauri::command]
pub async fn workers_status(state: State<'_, AppState>, project: String) -> Result<Vec<Worker>, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || Ok(parse_workers(&tool_output(&root, "workers.sh", &[&project, "status"]).unwrap_or_else(|e| e))))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn worker_action(state: State<'_, AppState>, project: String, action: String, worker: String) -> Result<Vec<Worker>, String> {
    let root = root_or_err(&state)?;
    if !matches!(action.as_str(), "start" | "stop" | "restart") {
        return Err(format!("unknown worker action {action}"));
    }
    tauri::async_runtime::spawn_blocking(move || {
        let text = tool_output(&root, "workers.sh", &[&project, &action, &worker])?;
        Ok(parse_workers(&text))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn worker_logs(state: State<'_, AppState>, project: String, worker: String) -> Result<String, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || tool_output(&root, "workers.sh", &[&project, "logs", &worker, "200"]))
        .await
        .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Media fallback, logs
// ---------------------------------------------------------------------------

fn parse_media(text: &str) -> String {
    text.lines()
        .find_map(|l| l.strip_prefix("MEDIA|").and_then(|r| r.split_once('|')).map(|(_, v)| v.to_string()))
        .unwrap_or_else(|| "off".into())
}

#[tauri::command]
pub async fn media_status(state: State<'_, AppState>, project: String) -> Result<String, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || Ok(parse_media(&tool_output(&root, "media-fallback.sh", &[&project, "--status"]).unwrap_or_default())))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn media_set(app: AppHandle, state: State<'_, AppState>, project: String, url: String, subdir: bool) -> Result<String, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let mut args = vec![project.as_str(), url.as_str()];
        if subdir && url != "--off" {
            args.push("--subdir-multisite");
        }
        let text = run_streaming(&app, &format!("media-{project}"), tool(&root, "media-fallback.sh", &args))?;
        Ok(parse_media(&text))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn project_logs(state: State<'_, AppState>, project: String, lines: Option<u32>, which: Option<String>) -> Result<String, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let n = lines.unwrap_or(150).to_string();
        let w = which.unwrap_or_else(|| "all".into());
        tool_output(&root, "logs.sh", &[&project, &n, &w])
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Host-side dev servers (npm run dev)
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct DevState {
    children: Mutex<HashMap<String, Child>>,
}

/// "v22.18.0" -> (22, 18, 0) for numeric version ordering.
fn semver_key(name: &str) -> (u32, u32, u32) {
    let mut it = name.trim_start_matches('v').split('.').map(|x| x.parse::<u32>().unwrap_or(0));
    (it.next().unwrap_or(0), it.next().unwrap_or(0), it.next().unwrap_or(0))
}

/// Node on the host: the app does not inherit a shell PATH, so look in the
/// usual places (nvm newest first, Homebrew, /usr/local).
fn host_npm() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        let nvm = PathBuf::from(home).join(".nvm/versions/node");
        if let Ok(rd) = std::fs::read_dir(&nvm) {
            let mut versions: Vec<PathBuf> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
            // Numeric semver order: a plain sort puts v9 after v22.
            versions.sort_by_key(|p| semver_key(&p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()));
            if let Some(v) = versions.last() {
                let npm = v.join("bin/npm");
                if npm.is_file() {
                    return Some(npm);
                }
            }
        }
    }
    ["/opt/homebrew/bin/npm", "/usr/local/bin/npm", "/usr/bin/npm"].iter().map(PathBuf::from).find(|p| p.is_file())
}

#[derive(Serialize, Clone)]
pub struct DevStatus {
    pub running: bool,
    pub url: String,
}

fn dev_alive(dev: &DevState, project: &str) -> bool {
    let mut map = dev.children.lock().unwrap();
    if let Some(child) = map.get_mut(project) {
        match child.try_wait() {
            Ok(None) => return true,
            _ => {
                map.remove(project);
            }
        }
    }
    false
}

#[tauri::command]
pub fn dev_server_status(dev: State<DevState>, state: State<AppState>, project: String) -> Result<DevStatus, String> {
    let root = root_or_err(&state)?;
    let hot = root.join("data/www").join(&project).join("public/hot");
    let url = std::fs::read_to_string(&hot).map(|s| s.trim().to_string()).unwrap_or_default();
    Ok(DevStatus { running: dev_alive(&dev, &project), url })
}

#[tauri::command]
pub async fn dev_server_start(app: AppHandle, dev: State<'_, DevState>, state: State<'_, AppState>, project: String) -> Result<DevStatus, String> {
    let root = root_or_err(&state)?;
    if dev_alive(&dev, &project) {
        return Ok(DevStatus { running: true, url: String::new() });
    }
    let dir = root.join("data/www").join(&project);
    if !dir.join("package.json").is_file() {
        return Err("no package.json in this project".into());
    }
    let npm = host_npm().ok_or("npm not found on this Mac (install Node via nvm or Homebrew)")?;
    let mut cmd = Command::new(&npm);
    cmd.args(["run", "dev"]).current_dir(&dir).stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null());
    cmd.env("PATH", format!("{}:{}", npm.parent().unwrap().display(), std::env::var("PATH").unwrap_or_default()));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = cmd.spawn().map_err(|e| format!("cannot start npm run dev: {e}"))?;
    let job = format!("dev-{project}");
    let _ = app.emit("job-output", crate::sites::JobLine { job: job.clone(), line: format!("$ {} run dev   (in data/www/{project})", npm.display()) });
    for reader in [child.stdout.take().map(|s| Box::new(s) as Box<dyn std::io::Read + Send>), child.stderr.take().map(|s| Box::new(s) as Box<dyn std::io::Read + Send>)].into_iter().flatten() {
        let app2 = app.clone();
        let job2 = job.clone();
        std::thread::spawn(move || {
            use std::io::BufRead;
            for line in std::io::BufReader::new(reader).lines().map_while(Result::ok) {
                let _ = app2.emit("job-output", crate::sites::JobLine { job: job2.clone(), line });
            }
        });
    }
    dev.children.lock().unwrap().insert(project.clone(), child);
    Ok(DevStatus { running: true, url: String::new() })
}

#[tauri::command]
pub fn dev_server_stop(dev: State<DevState>, project: String) -> Result<(), String> {
    let mut map = dev.children.lock().unwrap();
    if let Some(mut child) = map.remove(&project) {
        #[cfg(unix)]
        unsafe {
            // Kill the whole process group so vite (spawned by npm) goes too.
            libc::kill(-(child.id() as i32), libc::SIGTERM);
        }
        let _ = child.kill();
        let _ = child.wait();
    }
    Ok(())
}

pub fn stop_all_dev_servers(app: &AppHandle) {
    if let Some(dev) = app.try_state::<DevState>() {
        let mut map = dev.children.lock().unwrap();
        for (_, mut child) in map.drain() {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGTERM);
            }
            let _ = child.kill();
        }
    }
}

// ---------------------------------------------------------------------------
// Xdebug (same drop-in the MCP server manages, so both stay in sync)
// ---------------------------------------------------------------------------

const XDEBUG_INI: &str = "zzz-mcp-xdebug.ini";

#[derive(Serialize, Clone)]
pub struct XdebugStatus {
    pub enabled: bool,
    pub detail: String,
}

fn xdebug_ini_path(root: &Path) -> Result<std::path::PathBuf, String> {
    let php = crate::read_env_value(root, "PHP_SERVER").unwrap_or_default();
    if php.is_empty() {
        return Err("PHP_SERVER not set in .env".into());
    }
    let dir = root.join("cfg").join(format!("php-ini-{php}"));
    if !dir.is_dir() {
        return Err(format!("cfg/php-ini-{php} does not exist"));
    }
    Ok(dir.join(XDEBUG_INI))
}

fn xdebug_status_sync(root: &Path) -> XdebugStatus {
    let enabled = xdebug_ini_path(root).map(|p| p.is_file()).unwrap_or(false);
    let detail = crate::compose(root, &["exec", "-T", "php", "php", "-r",
        "echo extension_loaded('xdebug') ? 'loaded, mode=' . ini_get('xdebug.mode') : 'not loaded';"])
        .unwrap_or_else(|e| e.lines().next().unwrap_or("php container unreachable").to_string());
    XdebugStatus { enabled, detail }
}

#[tauri::command]
pub async fn xdebug_status(state: State<'_, AppState>) -> Result<XdebugStatus, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || Ok(xdebug_status_sync(&root))).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn xdebug_set(state: State<'_, AppState>, on: bool) -> Result<XdebugStatus, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let ini = xdebug_ini_path(&root)?;
        if on {
            std::fs::write(&ini, concat!(
                "; Managed by Devilbox Boost (companion app / MCP devilbox_xdebug).\n",
                "; Delete this file to disable Xdebug.\n",
                "[PHP]\n",
                "xdebug.mode               = debug,develop\n",
                "xdebug.start_with_request = yes\n",
                "xdebug.client_host        = host.docker.internal\n",
                "xdebug.client_port        = 9003\n",
                "xdebug.log_level          = 0\n",
            )).map_err(|e| e.to_string())?;
        } else if ini.is_file() {
            std::fs::remove_file(&ini).map_err(|e| e.to_string())?;
        }
        // The ini dir is bind-mounted; a PHP-FPM reload picks the change up.
        crate::compose(&root, &["exec", "-T", "php", "sh", "-c", "kill -USR2 1"])?;
        Ok(xdebug_status_sync(&root))
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Script runner (composer.json / package.json scripts)
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone, Default)]
pub struct ProjectScripts {
    pub composer: Vec<(String, String)>,
    pub npm: Vec<(String, String)>,
}

fn json_scripts(path: &Path) -> Vec<(String, String)> {
    let Ok(text) = std::fs::read_to_string(path) else { return Vec::new() };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else { return Vec::new() };
    let Some(map) = v.get("scripts").and_then(|s| s.as_object()) else { return Vec::new() };
    map.iter()
        .map(|(k, v)| {
            let preview = match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Array(a) => a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(" && "),
                _ => String::new(),
            };
            (k.clone(), preview)
        })
        .collect()
}

#[tauri::command]
pub fn project_scripts(state: State<AppState>, project: String) -> Result<ProjectScripts, String> {
    let root = root_or_err(&state)?;
    valid_project(&root, &project)?;
    let dir = root.join("data/www").join(&project);
    Ok(ProjectScripts {
        composer: json_scripts(&dir.join("composer.json")),
        npm: json_scripts(&dir.join("package.json")),
    })
}

#[tauri::command]
pub async fn run_script(app: AppHandle, state: State<'_, AppState>, project: String, kind: String, name: String) -> Result<String, String> {
    let root = root_or_err(&state)?;
    valid_project(&root, &project)?;
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':' | '.')) {
        return Err(format!("invalid script name {name}"));
    }
    let cmd = match kind.as_str() {
        "composer" => format!("composer run-script {} --no-interaction", crate::shell_quote(&name)),
        "npm" => format!("npm run {}", crate::shell_quote(&name)),
        other => return Err(format!("unknown script kind {other}")),
    };
    tauri::async_runtime::spawn_blocking(move || {
        run_streaming(&app, &format!("script-{project}"), in_project(&root, &project, &cmd))
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Live log follow (per project)
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct LogsState {
    children: Mutex<HashMap<String, Child>>,
}

/// Build the host-side follow command for a project's logs.
fn follow_cmd(root: &Path, project: &str, tld: &str, which: &str) -> String {
    let docker = crate::docker_path();
    let d = root.join("data/www").join(project);
    let app_files: Vec<String> = [
        "wp-content/debug.log", "storage/logs/laravel.log", "var/log/dev.log", "logs/error.log", "storage/logs/web.log",
    ]
    .iter()
    .map(|f| d.join(f))
    .filter(|p| p.is_file())
    .map(|p| crate::shell_quote(&p.to_string_lossy()))
    .collect();
    let tail_apps = if app_files.is_empty() { String::new() } else { format!("tail -n 0 -F {} &\n", app_files.join(" ")) };
    let err = format!(
        "{docker} compose logs --no-color --follow --tail 0 httpd php 2>/dev/null | grep --line-buffered -E {} &\n",
        crate::shell_quote(&format!("{project}\\.{tld}|/shared/httpd/{project}/"))
    );
    let access = format!(
        "{docker} compose logs --no-color --follow --tail 0 httpd 2>/dev/null | grep --line-buffered -E '\" [0-9]{{3}} ' &\n"
    );
    let body = match which {
        "app" => tail_apps,
        "error" => err,
        "access" => access,
        _ => format!("{tail_apps}{err}"),
    };
    format!("{body}wait")
}

#[tauri::command]
pub fn logs_follow_start(app: AppHandle, logs: State<LogsState>, state: State<AppState>, project: String, which: String) -> Result<(), String> {
    let root = root_or_err(&state)?;
    valid_project(&root, &project)?;
    logs_follow_stop(logs.clone(), project.clone())?;
    let env = crate::environment_for(&root);
    let script = follow_cmd(&root, &project, &env.tld, &which);
    let mut cmd = Command::new("bash");
    cmd.arg("-c").arg(&script).current_dir(&root).stdout(Stdio::piped()).stderr(Stdio::null()).stdin(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = cmd.spawn().map_err(|e| format!("cannot follow logs: {e}"))?;
    if let Some(stdout) = child.stdout.take() {
        let job = format!("logs-{project}");
        let app2 = app.clone();
        std::thread::spawn(move || {
            use std::io::BufRead;
            for line in std::io::BufReader::new(stdout).lines().map_while(Result::ok) {
                let _ = app2.emit("job-output", crate::sites::JobLine { job: job.clone(), line });
            }
        });
    }
    logs.children.lock().unwrap().insert(project, child);
    Ok(())
}

#[tauri::command]
pub fn logs_follow_stop(logs: State<LogsState>, project: String) -> Result<(), String> {
    if let Some(mut child) = logs.children.lock().unwrap().remove(&project) {
        #[cfg(unix)]
        unsafe {
            libc::kill(-(child.id() as i32), libc::SIGTERM);
        }
        let _ = child.kill();
        let _ = child.wait();
    }
    Ok(())
}

pub fn stop_all_log_follows(app: &AppHandle) {
    if let Some(logs) = app.try_state::<LogsState>() {
        let mut map = logs.children.lock().unwrap();
        for (_, mut child) in map.drain() {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGTERM);
            }
            let _ = child.kill();
        }
    }
}

// ---------------------------------------------------------------------------
// Git status (read-only; committing stays in the user's own git tool)
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone, Default)]
pub struct GitStatus {
    pub is_repo: bool,
    pub branch: String,
    pub dirty: u32,
    pub untracked: u32,
    pub ahead: u32,
    pub behind: u32,
    pub remote: String,
    pub last_commit: String,
}

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).current_dir(dir).output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

#[tauri::command]
pub async fn git_status(state: State<'_, AppState>, project: String) -> Result<GitStatus, String> {
    let root = root_or_err(&state)?;
    let dir = root.join("data/www").join(&project);
    tauri::async_runtime::spawn_blocking(move || {
        let mut st = GitStatus::default();
        if git(&dir, &["rev-parse", "--is-inside-work-tree"]).as_deref() != Some("true") {
            return Ok(st);
        }
        st.is_repo = true;
        st.branch = git(&dir, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default();
        if let Some(porcelain) = git(&dir, &["status", "--porcelain"]) {
            for l in porcelain.lines() {
                if l.starts_with("??") { st.untracked += 1 } else { st.dirty += 1 }
            }
        }
        if let Some(ab) = git(&dir, &["rev-list", "--left-right", "--count", "HEAD...@{upstream}"]) {
            let mut p = ab.split_whitespace();
            st.ahead = p.next().and_then(|v| v.parse().ok()).unwrap_or(0);
            st.behind = p.next().and_then(|v| v.parse().ok()).unwrap_or(0);
        }
        st.remote = git(&dir, &["remote", "get-url", "origin"]).unwrap_or_default();
        st.last_commit = git(&dir, &["log", "-1", "--format=%h %s (%cr)"]).unwrap_or_default();
        Ok(st)
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Update check (newest repo release vs the local checkout)
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
pub struct UpdateCheck {
    pub app_version: String,
    pub local_tag: String,
    pub latest_tag: String,
    pub latest_url: String,
    pub newer: bool,
}

#[tauri::command]
pub async fn app_update_check(state: State<'_, AppState>) -> Result<UpdateCheck, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let remote = git(&root, &["remote", "get-url", "origin"]).unwrap_or_default();
        let repo = remote
            .trim_end_matches(".git")
            .rsplit_once(':').map(|(_, r)| r.to_string())
            .or_else(|| remote.trim_end_matches(".git").split_once("github.com/").map(|(_, r)| r.to_string()))
            .ok_or("origin is not a GitHub remote")?;
        let out = Command::new("curl")
            .args(["-fsSL", "-m", "10", "-H", "User-Agent: devilbox-boost-companion"])
            .arg(format!("https://api.github.com/repos/{repo}/releases/latest"))
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err("could not reach GitHub (offline, or the repo has no releases)".into());
        }
        let v: serde_json::Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
        let latest_tag = v.get("tag_name").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let latest_url = v.get("html_url").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let local_tag = git(&root, &["describe", "--tags", "--abbrev=0"]).unwrap_or_default();
        Ok(UpdateCheck {
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            newer: !latest_tag.is_empty() && latest_tag != local_tag,
            local_tag,
            latest_tag,
            latest_url,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Launch at login
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn autostart_get(app: AppHandle) -> Result<bool, String> {
    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;
        return app.autolaunch().is_enabled().map_err(|e| e.to_string());
    }
    #[allow(unreachable_code)]
    Err("desktop only".into())
}

#[tauri::command]
pub fn autostart_set(app: AppHandle, on: bool) -> Result<bool, String> {
    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;
        let l = app.autolaunch();
        if on { l.enable() } else { l.disable() }.map_err(|e| e.to_string())?;
        return l.is_enabled().map_err(|e| e.to_string());
    }
    #[allow(unreachable_code)]
    {
        let _ = on;
        Err("desktop only".into())
    }
}

// ---------------------------------------------------------------------------
// SQL clients installed on this Mac
// ---------------------------------------------------------------------------

/// Installed GUI apps of a given kind ("editor", "terminal", "git"), by display name.
#[tauri::command]
pub fn detect_apps(kind: String) -> Vec<String> {
    let known: &[&str] = match kind.as_str() {
        "editor" => &["PhpStorm", "WebStorm", "IntelliJ IDEA", "Visual Studio Code", "Cursor", "Windsurf", "Zed", "Sublime Text", "Nova", "BBEdit", "Fleet", "TextMate", "Xcode"],
        "terminal" => &["Terminal", "iTerm", "Warp", "Ghostty", "kitty", "Alacritty", "WezTerm", "Hyper", "Tabby", "GNOME Terminal", "Konsole", "Windows Terminal"],
        "git" => &["Fork", "Tower", "GitHub Desktop", "Sourcetree", "GitKraken", "Sublime Merge", "GitUp"],
        _ => &[],
    };
    crate::platform::installed(known)
}

#[tauri::command]
pub fn db_clients() -> Vec<String> {
    crate::platform::installed(&["Sequel Ace", "TablePlus", "DBeaver", "Beekeeper Studio", "DbGate", "Querious", "HeidiSQL"])
}

#[tauri::command]
pub fn file_manager_name() -> String {
    crate::platform::file_manager_name().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tool_lines() {
        let f = parse_findings("OK|stack|Docker is running\nFAIL|site|db missing\nnoise\n");
        assert_eq!(f.len(), 2);
        assert_eq!(f[1].level, "FAIL");
        let s = parse_snapshots("/x/db/2026-10-23-1.sql.gz|100|1700000000\nSNAPSHOT|a|b\n");
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].name, "2026-10-23-1.sql.gz");
        let w = parse_workers("WORKER|queue|running|123\nWORKER|schedule|stopped|\n");
        assert!(w[0].running && !w[1].running);
        assert_eq!(parse_media("MEDIA|site|https://example.com\n"), "https://example.com");
        assert_eq!(parse_media(""), "off");
    }
}
