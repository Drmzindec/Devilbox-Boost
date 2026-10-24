//! Devilbox Boost companion: a small Tauri app that watches the Docker stack,
//! lists the projects in data/www, launches the bundled admin tools, imports
//! database dumps and warns when a container or PHP-FPM worker dies.
//!
//! Everything that talks to Docker shells out to `docker compose` inside the
//! Devilbox root, exactly like the MCP server does, so both stay in sync.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;

mod files;
mod ops;
mod platform;
mod sites;
mod ssh;
mod term;

pub const QUICK_OPEN_SHORTCUT: &str = "CommandOrControl+Shift+D";

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    /// Absolute path of the Devilbox Boost checkout (contains docker-compose.yml).
    pub root: String,
    /// macOS application name handed to `open -a` for mysql:// URLs.
    pub db_client: String,
    /// macOS application name used for "Open in editor".
    pub editor: String,
    /// macOS application name used for "Open terminal".
    pub terminal: String,
    /// Desktop notifications when a container stops or PHP-FPM crashes.
    pub notifications: bool,
    /// Scheduled site updates (only while the app is running).
    pub schedule_enabled: bool,
    /// "HH:MM" local time.
    pub schedule_time: String,
    /// "wordpress", "php" or "all".
    pub schedule_scope: String,
    /// "YYYY-MM-DD" of the last scheduled run.
    pub schedule_last_run: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            root: String::new(),
            db_client: "Sequel Ace".into(),
            editor: "Visual Studio Code".into(),
            terminal: "Terminal".into(),
            notifications: true,
            schedule_enabled: false,
            schedule_time: "03:00".into(),
            schedule_scope: "wordpress".into(),
            schedule_last_run: String::new(),
        }
    }
}

#[derive(Default)]
struct Watch {
    /// service -> state as of the previous poll; empty until the first poll ran.
    previous: HashMap<String, String>,
    primed: bool,
    /// whether the last log check already saw crashes (edge-triggered alerts).
    php_crashing: bool,
    /// poll counter: the PHP crash-log scan only runs every other poll.
    tick: u64,
    /// fingerprint of the last tray menu, so it is only rebuilt on change.
    menu_sig: String,
}

pub struct AppState {
    pub settings: Mutex<Settings>,
    tray: Mutex<Option<TrayIcon>>,
    watch: Mutex<Watch>,
}

fn settings_path(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| std::env::temp_dir());
    let _ = fs::create_dir_all(&dir);
    dir.join("settings.json")
}

fn is_devilbox_root(p: &Path) -> bool {
    p.join("docker-compose.yml").is_file() && p.join("data").join("www").is_dir()
}

/// Where the stack lives. Order: saved setting, DEVILBOX_ROOT env, the
/// checkout this app was built from (dev convenience), then well-known folders.
fn discover_root(saved: &str) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if !saved.is_empty() {
        candidates.push(PathBuf::from(saved));
    }
    if let Ok(env) = std::env::var("DEVILBOX_ROOT") {
        candidates.push(PathBuf::from(env));
    }
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join(".."));
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        for sub in [
            "Development/Devilbox-Boost",
            "Development/devilbox-boost",
            "Projects/Devilbox-Boost",
            "devilbox-boost",
            "Devilbox-Boost",
            "devilbox",
        ] {
            candidates.push(home.join(sub));
        }
    }
    candidates
        .into_iter()
        .find(|c| is_devilbox_root(c))
        .and_then(|c| c.canonicalize().ok())
}

fn load_settings(app: &AppHandle) -> Settings {
    let mut s: Settings = fs::read_to_string(settings_path(app))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    if let Some(root) = discover_root(&s.root) {
        s.root = root.to_string_lossy().to_string();
    }
    s
}

pub(crate) fn save_settings(app: &AppHandle, s: &Settings) -> Result<(), String> {
    let json = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    fs::write(settings_path(app), json).map_err(|e| e.to_string())
}

fn current_settings(state: &State<AppState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

pub(crate) fn root_or_err(state: &State<AppState>) -> Result<PathBuf, String> {
    let s = current_settings(state);
    if s.root.is_empty() || !is_devilbox_root(Path::new(&s.root)) {
        return Err("Devilbox folder not set. Pick it in Settings.".into());
    }
    Ok(PathBuf::from(s.root))
}

// ---------------------------------------------------------------------------
// .env parsing
// ---------------------------------------------------------------------------

fn read_env(root: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(text) = fs::read_to_string(root.join(".env")) {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                map.insert(k.trim().to_string(), v.trim().trim_matches('"').to_string());
            }
        }
    }
    map
}

pub(crate) fn read_env_value(root: &Path, key: &str) -> Option<String> {
    read_env(root).get(key).cloned()
}

#[derive(Serialize, Clone, Debug)]
pub struct Environment {
    pub root: String,
    pub tld: String,
    pub scheme: String,
    pub vhost_port: String,
    pub ui_url: String,
    pub php: String,
    pub httpd: String,
    pub mysql: String,
    pub pgsql: String,
    pub redis: String,
    pub mongo: String,
    pub mysql_root_password: String,
    pub mysql_host_port: String,
    pub shortcut: String,
    /// Dashboard tool paths discovered in .devilbox/www/htdocs/vendor, so a
    /// tool version bump does not break every link in the app.
    pub tool_pma: String,
    pub tool_adminer: String,
    pub tool_pca: String,
    pub tool_opcache: String,
    pub tool_ppga: String,
}

/// Find the bundled admin tools in the vendor folder. Falls back to the
/// versions bundled when this code was written.
fn vendor_tools(root: &Path) -> (String, String, String, String, String) {
    let mut pma = "/vendor/phpmyadmin-5.2.3/index.php".to_string();
    let mut adminer = "/vendor/adminer-6.1.2-devilbox.php".to_string();
    let mut pca = "/vendor/phpcacheadmin-2.7.2/index.php".to_string();
    let mut opc = "/vendor/opcache-gui-3.6.1.php".to_string();
    let mut ppga = "/vendor/phppgadmin-7.13.0/".to_string();
    if let Ok(rd) = fs::read_dir(root.join(".devilbox/www/htdocs/vendor")) {
        let mut names: Vec<String> = rd.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).collect();
        names.sort(); // ascending: the highest version overwrites last
        for n in names {
            if n.starts_with("phpmyadmin-") {
                pma = format!("/vendor/{n}/index.php");
            } else if n.starts_with("adminer-") && n.ends_with("-devilbox.php") {
                adminer = format!("/vendor/{n}");
            } else if n.starts_with("phpcacheadmin-") {
                pca = format!("/vendor/{n}/index.php");
            } else if n.starts_with("opcache-gui-") && n.ends_with(".php") {
                opc = format!("/vendor/{n}");
            } else if n.starts_with("phppgadmin-") {
                ppga = format!("/vendor/{n}/");
            }
        }
    }
    (pma, adminer, pca, opc, ppga)
}

pub(crate) fn environment_for(root: &Path) -> Environment {
    let env = read_env(root);
    let get = |k: &str, d: &str| env.get(k).cloned().unwrap_or_else(|| d.to_string());
    let ssl_type = get("HTTPD_VHOST_SSL_TYPE", "plain").to_lowercase();
    let https = matches!(ssl_type.as_str(), "ssl" | "both" | "redir");
    let http_port = get("HOST_PORT_HTTPD", "80");
    let https_port = get("HOST_PORT_HTTPD_SSL", "443");
    let (scheme, vhost_port) = if https {
        ("https".to_string(), if https_port == "443" { String::new() } else { format!(":{https_port}") })
    } else {
        ("http".to_string(), if http_port == "80" { String::new() } else { format!(":{http_port}") })
    };
    let (tool_pma, tool_adminer, tool_pca, tool_opcache, tool_ppga) = vendor_tools(root);
    Environment {
        root: root.to_string_lossy().to_string(),
        tld: get("TLD_SUFFIX", "loc"),
        ui_url: format!("{scheme}://localhost{vhost_port}"),
        tool_pma,
        tool_adminer,
        tool_pca,
        tool_opcache,
        tool_ppga,
        scheme,
        vhost_port,
        php: get("PHP_SERVER", ""),
        httpd: get("HTTPD_SERVER", ""),
        mysql: get("MYSQL_SERVER", ""),
        pgsql: get("PGSQL_SERVER", ""),
        redis: get("REDIS_SERVER", ""),
        mongo: get("MONGO_SERVER", ""),
        mysql_root_password: get("MYSQL_ROOT_PASSWORD", "root"),
        mysql_host_port: get("HOST_PORT_MYSQL", "3306"),
        shortcut: QUICK_OPEN_SHORTCUT.replace("CommandOrControl", if cfg!(target_os = "macos") { "Cmd" } else { "Ctrl" }),
    }
}

// ---------------------------------------------------------------------------
// docker compose
// ---------------------------------------------------------------------------

pub(crate) fn docker_path() -> String {
    for p in platform::docker_candidates() {
        if Path::new(p).is_file() {
            return p.to_string();
        }
    }
    "docker".to_string()
}

pub(crate) fn compose(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new(docker_path())
        .arg("compose")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("cannot run docker: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    if out.status.success() {
        Ok(stdout)
    } else {
        Err(if stderr.trim().is_empty() { stdout } else { stderr })
    }
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct PortMap {
    pub published: u32,
    pub target: u32,
    pub url: String,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct ServiceStatus {
    pub service: String,
    pub name: String,
    pub state: String,
    pub health: String,
    pub status: String,
    pub image: String,
    pub ports: Vec<PortMap>,
}

#[derive(Deserialize)]
struct PsPublisher {
    #[serde(rename = "URL", default)]
    url: String,
    #[serde(rename = "TargetPort", default)]
    target_port: u32,
    #[serde(rename = "PublishedPort", default)]
    published_port: u32,
    #[serde(rename = "Protocol", default)]
    protocol: String,
}

#[derive(Deserialize)]
struct PsRow {
    #[serde(rename = "Service", default)]
    service: String,
    #[serde(rename = "Name", default)]
    name: String,
    #[serde(rename = "State", default)]
    state: String,
    #[serde(rename = "Health", default)]
    health: String,
    #[serde(rename = "Status", default)]
    status: String,
    #[serde(rename = "Image", default)]
    image: String,
    #[serde(rename = "Publishers", default)]
    publishers: Vec<PsPublisher>,
}

fn parse_ps(json: &str) -> Vec<ServiceStatus> {
    // `docker compose ps --format json` prints one object per line on
    // compose >= 2.21 and a JSON array on older versions. Accept both.
    let rows: Vec<PsRow> = if json.trim_start().starts_with('[') {
        serde_json::from_str(json).unwrap_or_default()
    } else {
        json.lines().filter_map(|l| serde_json::from_str::<PsRow>(l).ok()).collect()
    };
    let mut out: Vec<ServiceStatus> = rows
        .into_iter()
        .map(|r| ServiceStatus {
            service: r.service,
            name: r.name,
            state: r.state,
            health: r.health,
            status: r.status,
            image: r.image,
            ports: r
                .publishers
                .into_iter()
                .filter(|p| p.protocol != "udp" && p.published_port > 0)
                .map(|p| PortMap { published: p.published_port, target: p.target_port, url: p.url })
                .collect(),
        })
        .collect();
    out.sort_by(|a, b| a.service.cmp(&b.service));
    out
}

fn stack_status_sync(root: &Path) -> Result<Vec<ServiceStatus>, String> {
    let json = compose(root, &["ps", "--all", "--format", "json"])?;
    Ok(parse_ps(&json))
}

fn summarize(status: &[ServiceStatus]) -> (usize, usize) {
    let running = status.iter().filter(|s| s.state == "running").count();
    (running, status.len())
}

/// Count PHP-FPM worker crashes logged in the last `window` seconds.
fn php_crashes_since(root: &Path, window: u64) -> usize {
    let since = format!("{window}s");
    compose(root, &["logs", "--no-color", "--since", &since, "php"])
        .map(|log| {
            log.lines()
                .filter(|l| l.contains("exited on signal") || l.contains("SIGSEGV") || l.contains("segfault"))
                .count()
        })
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone, Debug)]
pub struct Project {
    pub name: String,
    pub kind: String,
    pub url: String,
    pub path: String,
    pub db: String,
}

fn grab(text: &str, key: &str) -> Option<String> {
    // Matches define('KEY', 'value') in wp-config.php and KEY=value in .env
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with(&format!("{key}=")) {
            let v = t[key.len() + 1..].trim();
            // A quoted value ends at its closing quote; an unquoted one at an
            // inline comment (DB_DATABASE=shop # main db).
            if let Some(rest) = v.strip_prefix('"') {
                return rest.split('"').next().map(|s| s.to_string());
            }
            if let Some(rest) = v.strip_prefix('\'') {
                return rest.split('\'').next().map(|s| s.to_string());
            }
            let v = v.find(" #").map(|i| v[..i].trim_end()).unwrap_or(v);
            return Some(v.to_string());
        }
        if t.contains(&format!("'{key}'")) && t.contains("define") {
            let after = t.split(&format!("'{key}'")).nth(1)?;
            let mut parts = after.split('\'');
            parts.next();
            return parts.next().map(|s| s.to_string());
        }
    }
    None
}

fn detect_kind(dir: &Path) -> &'static str {
    let has = |rel: &str| dir.join(rel).exists();
    if has("please") {
        "statamic"
    } else if has("artisan") {
        "laravel"
    } else if has("wp-config.php") || has("wp-content") || has("wp-config-sample.php") {
        "wordpress"
    } else if has("bin/console") {
        "symfony"
    } else if has("bin/cake") || has("webroot") {
        "cakephp"
    } else if has("craft") {
        "craft"
    } else if has("yii") {
        "yii"
    } else if has("web/core/lib/Drupal.php") || has("core/lib/Drupal.php") {
        "drupal"
    } else if has("bin/magento") {
        "magento"
    } else if has("next.config.mjs") || has("next.config.js") || has("next.config.ts") {
        "next.js"
    } else if has("index.html") || has("dist/index.html") {
        "static"
    } else if has("index.php") || has("public/index.php") || has("www/index.php") {
        "php"
    } else {
        "other"
    }
}

fn detect_project(dir: &Path, env: &Environment) -> Project {
    let name = dir.file_name().unwrap_or_default().to_string_lossy().to_string();
    let kind = detect_kind(dir);
    let db = if kind == "wordpress" {
        fs::read_to_string(dir.join("wp-config.php")).ok().and_then(|t| grab(&t, "DB_NAME"))
    } else {
        fs::read_to_string(dir.join(".env")).ok().and_then(|t| grab(&t, "DB_DATABASE"))
    }
    .unwrap_or_default();
    Project {
        url: format!("{}://{}.{}{}", env.scheme, name, env.tld, env.vhost_port),
        name,
        kind: kind.to_string(),
        path: dir.to_string_lossy().to_string(),
        db,
    }
}

pub(crate) fn projects_sync(root: &Path) -> Vec<Project> {
    let env = environment_for(root);
    let mut list: Vec<Project> = fs::read_dir(root.join("data").join("www"))
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .filter(|p| !p.file_name().unwrap_or_default().to_string_lossy().starts_with('.'))
                .map(|p| detect_project(&p, &env))
                .collect()
        })
        .unwrap_or_default();
    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    list
}

// ---------------------------------------------------------------------------
// Dump import
// ---------------------------------------------------------------------------

pub(crate) fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Percent-encode a URL userinfo component (the password in mysql://root:…@).
fn url_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Generic SQL import: stream a .sql / .sql.gz / .zip dump into a database,
/// dropping CREATE DATABASE/USE statements for the production DB name and
/// repairing the `DEFAULT x AS` binary-default quirk some hosts export.
fn import_generic(root: &Path, env: &Environment, db: &str, file: &Path, fresh: bool) -> Result<String, String> {
    let name = file.to_string_lossy();
    let reader = if name.ends_with(".gz") {
        format!("gzip -dc {}", shell_quote(&name))
    } else if name.ends_with(".zip") {
        format!("unzip -p {} '*.sql'", shell_quote(&name))
    } else {
        format!("cat {}", shell_quote(&name))
    };
    let pw = format!("-p{}", shell_quote(&env.mysql_root_password));
    let mut log = String::new();
    if fresh {
        compose(root, &["exec", "-T", "mysql", "sh", "-c", &format!(
            "mariadb -uroot {pw} -e 'DROP DATABASE IF EXISTS `{db}`; CREATE DATABASE `{db}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci' 2>/dev/null || mysql -uroot {pw} -e 'DROP DATABASE IF EXISTS `{db}`; CREATE DATABASE `{db}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci'"
        )])?;
        log.push_str(&format!("Recreated database {db}\n"));
    } else {
        compose(root, &["exec", "-T", "mysql", "sh", "-c", &format!(
            "mariadb -uroot {pw} -e 'CREATE DATABASE IF NOT EXISTS `{db}`' 2>/dev/null || mysql -uroot {pw} -e 'CREATE DATABASE IF NOT EXISTS `{db}`'"
        )])?;
    }
    let pipeline = format!(
        "set -o pipefail; {reader} | LC_ALL=C sed -E '/^(CREATE DATABASE|USE `)/d; s/DEFAULT x AS `([0-9A-Fa-f]+)`/DEFAULT x'\"'\"'\\1'\"'\"'/g' | {docker} compose exec -T mysql sh -c {inner}",
        docker = docker_path(),
        inner = shell_quote(&format!("command -v mariadb >/dev/null 2>&1 && mariadb -uroot {pw} --max-allowed-packet=256M {db} || mysql -uroot {pw} --max-allowed-packet=256M {db}")),
    );
    let out = Command::new("bash").arg("-c").arg(&pipeline).current_dir(root).output().map_err(|e| e.to_string())?;
    log.push_str(&String::from_utf8_lossy(&out.stdout));
    log.push_str(&String::from_utf8_lossy(&out.stderr));
    if !out.status.success() {
        return Err(format!("import failed:\n{log}"));
    }
    let tables = compose(root, &["exec", "-T", "mysql", "sh", "-c", &format!(
        "mariadb -uroot {pw} -N -e \"SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='{db}'\" 2>/dev/null || mysql -uroot {pw} -N -e \"SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='{db}'\""
    )]).unwrap_or_default();
    log.push_str(&format!("Imported {} into {db}: {} tables\n", file.file_name().unwrap_or_default().to_string_lossy(), tables.trim()));
    Ok(log)
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_settings(state: State<AppState>) -> Settings {
    current_settings(&state)
}

#[tauri::command]
fn get_environment(state: State<AppState>) -> Result<Environment, String> {
    let root = root_or_err(&state)?;
    Ok(environment_for(&root))
}

#[tauri::command]
fn save_settings_cmd(app: AppHandle, state: State<AppState>, settings: Settings) -> Result<Settings, String> {
    if !settings.root.is_empty() && !is_devilbox_root(Path::new(&settings.root)) {
        return Err("That folder has no docker-compose.yml and data/www, so it is not a Devilbox checkout.".into());
    }
    save_settings(&app, &settings)?;
    *state.settings.lock().unwrap() = settings.clone();
    Ok(settings)
}

#[tauri::command]
async fn pick_root(app: AppHandle) -> Result<Option<String>, String> {
    let picked = app.dialog().file().blocking_pick_folder();
    match picked {
        Some(p) => {
            let path = p.into_path().map_err(|e| e.to_string())?;
            if !is_devilbox_root(&path) {
                return Err("That folder has no docker-compose.yml and data/www.".into());
            }
            Ok(Some(path.to_string_lossy().to_string()))
        }
        None => Ok(None),
    }
}

#[tauri::command]
async fn pick_dump(app: AppHandle) -> Result<Option<String>, String> {
    let picked = app
        .dialog()
        .file()
        .add_filter("SQL dumps", &["sql", "gz", "zip"])
        .set_title("Choose a database dump")
        .blocking_pick_file();
    match picked {
        Some(p) => Ok(Some(p.into_path().map_err(|e| e.to_string())?.to_string_lossy().to_string())),
        None => Ok(None),
    }
}

#[tauri::command]
async fn stack_status(state: State<'_, AppState>) -> Result<Vec<ServiceStatus>, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || stack_status_sync(&root))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn stack_action(app: AppHandle, state: State<'_, AppState>, action: String, service: Option<String>) -> Result<String, String> {
    let root = root_or_err(&state)?;
    // Unprime BEFORE acting so a poll that lands mid-stop does not raise a
    // "container stopped" alert for a deliberate stop.
    state.watch.lock().unwrap().primed = false;
    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut args: Vec<&str> = match action.as_str() {
            "start" => vec!["up", "-d"],
            "stop" => vec!["stop"],
            "restart" => vec!["restart"],
            "recreate" => vec!["up", "-d", "--force-recreate"],
            _ => return Err(format!("unknown action {action}")),
        };
        if let Some(s) = service.as_deref() {
            args.push(s);
        }
        compose(&root, &args)
    })
    .await
    .map_err(|e| e.to_string())?;
    app.state::<AppState>().watch.lock().unwrap().primed = false;
    refresh_tray(&app);
    result
}

#[tauri::command]
async fn stack_logs(state: State<'_, AppState>, service: String, lines: Option<u32>) -> Result<String, String> {
    let root = root_or_err(&state)?;
    let n = lines.unwrap_or(200).to_string();
    tauri::async_runtime::spawn_blocking(move || {
        let out = Command::new(docker_path())
            .args(["compose", "logs", "--no-color", "--tail", &n, &service])
            .current_dir(&root)
            .output()
            .map_err(|e| e.to_string())?;
        Ok(format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let root = root_or_err(&state)?;
    Ok(tauri::async_runtime::spawn_blocking(move || projects_sync(&root))
        .await
        .map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn databases(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let root = root_or_err(&state)?;
    let env = environment_for(&root);
    tauri::async_runtime::spawn_blocking(move || {
        let pw = format!("-p{}", shell_quote(&env.mysql_root_password));
        let out = compose(&root, &["exec", "-T", "mysql", "sh", "-c",
            &format!("command -v mariadb >/dev/null 2>&1 && mariadb -uroot {pw} -N -e 'SHOW DATABASES' || mysql -uroot {pw} -N -e 'SHOW DATABASES'")])?;
        let system = ["information_schema", "performance_schema", "mysql", "sys"];
        Ok(out
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !system.contains(&l.as_str()))
            .collect())
    })
    .await
    .map_err(|e| e.to_string())?
}

fn valid_db_name(db: &str) -> Result<(), String> {
    if db.is_empty() || !db.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err("Database name may only contain letters, digits, _ and -".into());
    }
    Ok(())
}

/// Run one SQL statement as root in the mysql container (mariadb client with
/// a mysql fallback), with the password shell-quoted.
pub(crate) fn mysql_exec(root: &Path, sql: &str) -> Result<String, String> {
    let env = environment_for(root);
    let pw = format!("-p{}", shell_quote(&env.mysql_root_password));
    let q = shell_quote(sql);
    compose(root, &["exec", "-T", "mysql", "sh", "-c", &format!(
        "mariadb -uroot {pw} -e {q} 2>/dev/null || mysql -uroot {pw} -e {q}"
    )])
}

#[tauri::command]
async fn db_create(state: State<'_, AppState>, db: String) -> Result<String, String> {
    let root = root_or_err(&state)?;
    valid_db_name(&db)?;
    tauri::async_runtime::spawn_blocking(move || {
        mysql_exec(&root, &format!("CREATE DATABASE IF NOT EXISTS `{db}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci"))
            .map(|_| format!("Database {db} created"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn db_drop(state: State<'_, AppState>, db: String) -> Result<String, String> {
    let root = root_or_err(&state)?;
    valid_db_name(&db)?;
    if ["information_schema", "performance_schema", "mysql", "sys"].contains(&db.as_str()) {
        return Err(format!("{db} is a system database"));
    }
    tauri::async_runtime::spawn_blocking(move || {
        mysql_exec(&root, &format!("DROP DATABASE IF EXISTS `{db}`")).map(|_| format!("Database {db} dropped"))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Import a dump. WordPress projects go through tools/import-wp-dumps.sh, which
/// also detects the table prefix and writes wp-config.php; everything else is a
/// plain SQL import into `db`.
#[tauri::command]
async fn import_dump(state: State<'_, AppState>, project: Option<String>, db: String, file: String, fresh: bool) -> Result<String, String> {
    let root = root_or_err(&state)?;
    if db.trim().is_empty() || !db.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err("Database name may only contain letters, digits, _ and -".into());
    }
    let path = PathBuf::from(&file);
    if !path.is_file() {
        return Err(format!("{file} is not a file"));
    }
    tauri::async_runtime::spawn_blocking(move || {
        let env = environment_for(&root);
        let script = root.join("tools").join("import-wp-dumps.sh");
        let is_wp = project.as_deref().map(|p| root.join("data/www").join(p).join("wp-content").is_dir()).unwrap_or(false);
        if is_wp && script.is_file() {
            let out = Command::new("bash")
                .arg(&script)
                .arg("--file")
                .arg(&path)
                .arg(project.as_deref().unwrap_or_default())
                .current_dir(&root)
                .output()
                .map_err(|e| e.to_string())?;
            let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
            if out.status.success() && !text.contains("FAIL") {
                Ok(text)
            } else {
                Err(format!("WordPress import reported a problem:\n{text}"))
            }
        } else {
            import_generic(&root, &env, &db, &path, fresh)
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Open (or focus) a dedicated window for a web tool. Used for pages that
/// refuse to be framed (phpMyAdmin, Adminer send X-Frame-Options: DENY).
#[tauri::command]
fn open_tool(app: AppHandle, label: String, title: String, url: String) -> Result<(), String> {
    let safe_label: String = label.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect();
    let parsed = url.parse().map_err(|e| format!("bad url: {e}"))?;
    if let Some(w) = app.get_webview_window(&safe_label) {
        // The window may be showing an older URL under the same label
        // (Adminer opened for another database): navigate before focusing.
        let _ = w.navigate(parsed);
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }
    WebviewWindowBuilder::new(&app, &safe_label, WebviewUrl::External(parsed))
        .title(title)
        .inner_size(1200.0, 820.0)
        .min_inner_size(640.0, 480.0)
        .build()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    app.opener().open_url(url, None::<&str>).map_err(|e| e.to_string())
}

/// Open a path or URL with a GUI app ("Finder" = the platform file manager).
#[tauri::command]
fn open_with(app: AppHandle, app_name: String, target: String) -> Result<(), String> {
    if app_name.trim().is_empty() {
        return app.opener().open_path(target, None::<&str>).map_err(|e| e.to_string());
    }
    platform::open_with(&app_name, &target)
}

#[tauri::command]
fn open_db_client(state: State<AppState>, db: String) -> Result<(), String> {
    let root = root_or_err(&state)?;
    let env = environment_for(&root);
    let s = current_settings(&state);
    if s.db_client.trim().is_empty() {
        return Err("No SQL client selected in Settings".into());
    }
    let url = format!(
        "mysql://root:{}@127.0.0.1:{}/{}?name={}&env=local",
        url_encode(&env.mysql_root_password), env.mysql_host_port, db, db
    );
    platform::open_with(&s.db_client, &url).map_err(|e| format!("{} could not be opened (is it installed?): {e}", s.db_client))
}

#[tauri::command]
fn test_notification(app: AppHandle) -> Result<(), String> {
    notify(&app, "Devilbox Boost", "Notifications are working.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Tray, polling, alerts
// ---------------------------------------------------------------------------

pub(crate) fn notify(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// Compare this poll with the previous one and raise desktop alerts for
/// containers that stopped on their own and for PHP-FPM worker crashes.
fn check_alerts(app: &AppHandle, root: &Path, status: &[ServiceStatus]) {
    let state = app.state::<AppState>();
    let settings = current_settings(&state);
    let mut watch = state.watch.lock().unwrap();
    let current: HashMap<String, String> = status.iter().map(|s| (s.service.clone(), s.state.clone())).collect();
    if settings.notifications && watch.primed {
        let mut gone: Vec<String> = Vec::new();
        for (service, prev_state) in &watch.previous {
            let now = current.get(service).cloned().unwrap_or_else(|| "missing".into());
            if prev_state == "running" && now != "running" {
                let detail = status.iter().find(|s| &s.service == service).map(|s| s.status.clone()).unwrap_or(now.clone());
                gone.push(format!("{service} ({detail})"));
            }
        }
        if !gone.is_empty() {
            notify(app, "Devilbox container stopped", &gone.join(", "));
        }
    }
    watch.previous = current;
    watch.primed = true;
    watch.tick = watch.tick.wrapping_add(1);

    // PHP-FPM crash detection: edge-triggered, every other poll (the log scan
    // spawns a docker process, so don't pay for it every 15 seconds).
    if settings.notifications && watch.tick % 2 == 0 && status.iter().any(|s| s.service == "php" && s.state == "running") {
        let crashes = php_crashes_since(root, 35);
        if crashes > 0 && !watch.php_crashing {
            notify(app, "PHP-FPM workers are crashing", &format!(
                "{crashes} crash line(s) in the last 35 seconds. Check Overview › php › Logs. If JIT is on, turn it off in cfg/php-ini-*/custom.ini."
            ));
        }
        watch.php_crashing = crashes > 0;
    }
}

fn refresh_tray(app: &AppHandle) {
    let state = app.state::<AppState>();
    let settings = current_settings(&state);
    let (text, toggle, status) = match discover_root(&settings.root) {
        None => ("Devilbox folder not set".to_string(), "Start stack".to_string(), Vec::new()),
        Some(root) => match stack_status_sync(&root) {
            Ok(status) => {
                check_alerts(app, &root, &status);
                let (running, total) = summarize(&status);
                let text = if total == 0 {
                    "Stack is down".to_string()
                } else if running == total {
                    format!("Running: {running} of {total} services")
                } else if running == 0 {
                    format!("Stopped ({total} services)")
                } else {
                    format!("Degraded: {running} of {total} running")
                };
                let toggle = if running > 0 { "Stop stack" } else { "Start stack" };
                (text, toggle.to_string(), status)
            }
            Err(e) => (format!("Docker unavailable: {}", e.lines().next().unwrap_or("")), "Start stack".to_string(), Vec::new()),
        },
    };
    if let Some(tray) = state.tray.lock().unwrap().as_ref() {
        // Rebuilding the menu replaces it while the user may have it open, so
        // only do it when its contents actually changed.
        let projects = root_or_err(&state).map(|r| projects_sync(&r)).unwrap_or_default();
        let sig = format!(
            "{text}|{toggle}|{}|{}",
            status.iter().map(|s| format!("{}:{}:{}", s.service, s.state, s.health)).collect::<Vec<_>>().join(","),
            projects.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(",")
        );
        let changed = {
            let mut w = state.watch.lock().unwrap();
            if w.menu_sig == sig { false } else { w.menu_sig = sig; true }
        };
        if changed {
            if let Ok(menu) = build_menu(app, &text, &toggle, &status, &projects) {
                let _ = tray.set_menu(Some(menu));
            }
        }
        let _ = tray.set_tooltip(Some(format!("Devilbox Boost: {text}")));
        let (running, total) = summarize(&status);
        let glyph = if total == 0 || running == 0 { "○" } else if running == total { "●" } else { "◐" };
        let _ = tray.set_title(Some(glyph));
    }
    let _ = app.emit("stack-status", &status);
}

/// Tools reachable from the tray: label, title, url. Depends on which services run.
fn tray_tools(env: &Environment, status: &[ServiceStatus]) -> Vec<(String, String, String)> {
    let ui = &env.ui_url;
    let port = |names: &[&str], target: u32| -> Option<u32> {
        names.iter().find_map(|n| status.iter().find(|s| &s.service == n).and_then(|s| s.ports.iter().find(|p| p.target == target).map(|p| p.published)))
    };
    let mut v = vec![
        ("intranet".to_string(), "Intranet".to_string(), ui.clone()),
        ("phpmyadmin".to_string(), "phpMyAdmin".to_string(), format!("{ui}{}", env.tool_pma)),
        ("adminer".to_string(), "Adminer".to_string(), format!("{ui}{}?username=root", env.tool_adminer)),
        ("phpcacheadmin".to_string(), "phpCacheAdmin".to_string(), format!("{ui}{}", env.tool_pca)),
        ("opcache".to_string(), "OPcache GUI".to_string(), format!("{ui}{}", env.tool_opcache)),
    ];
    if let Some(p) = port(&["mailpit"], 8025) { v.push(("mailpit".into(), "Mailpit".into(), format!("http://localhost:{p}"))); }
    if let Some(p) = port(&["rabbitmq", "rabbit"], 15672) { v.push(("rabbitmq".into(), "RabbitMQ".into(), format!("http://localhost:{p}"))); }
    if let Some(p) = port(&["meilisearch"], 7700) { v.push(("meilisearch".into(), "Meilisearch".into(), format!("http://localhost:{p}"))); }
    if let Some(p) = port(&["rustfs", "minio"], 9001) { v.push(("rustfs".into(), "RustFS console".into(), format!("http://localhost:{p}"))); }
    v
}

fn build_menu(app: &AppHandle, status_text: &str, toggle_text: &str, status: &[ServiceStatus], projects: &[Project]) -> tauri::Result<Menu<tauri::Wry>> {
    let status_item = MenuItem::with_id(app, "status", status_text, false, None::<&str>)?;
    let open_item = MenuItem::with_id(app, "open", "Open Devilbox Boost", true, Some(QUICK_OPEN_SHORTCUT))?;
    let toggle_item = MenuItem::with_id(app, "toggle", toggle_text, true, None::<&str>)?;
    let restart_item = MenuItem::with_id(app, "restart", "Restart stack", true, None::<&str>)?;
    let quit_item = PredefinedMenuItem::quit(app, Some("Quit Devilbox Boost"))?;
    let menu = Menu::new(app)?;
    menu.append(&status_item)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&open_item)?;
    let state = app.state::<AppState>();
    if let Ok(root) = root_or_err(&state) {
        let env = environment_for(&root);
        let tools = Submenu::new(app, "Tools", true)?;
        for (label, title, url) in tray_tools(&env, status) {
            tools.append(&MenuItem::with_id(app, format!("tool:{label}|{title}|{url}"), &title, true, None::<&str>)?)?;
        }
        tools.append(&PredefinedMenuItem::separator(app)?)?;
        tools.append(&MenuItem::with_id(app, "intranet", "Open intranet in browser", true, None::<&str>)?)?;
        menu.append(&tools)?;
        if !projects.is_empty() {
            let sub = Submenu::new(app, format!("Projects ({})", projects.len()), true)?;
            for p in projects.iter().take(40) {
                sub.append(&MenuItem::with_id(app, format!("proj:{}", p.url), &p.name, true, None::<&str>)?)?;
            }
            if projects.len() > 40 {
                sub.append(&MenuItem::with_id(app, "open", format!("… {} more in the app", projects.len() - 40), true, None::<&str>)?)?;
            }
            menu.append(&sub)?;
        }
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&toggle_item)?;
    menu.append(&restart_item)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&quit_item)?;
    Ok(menu)
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_menu(app, "Checking stack…", "Start stack", &[], &[])?;
    let tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().cloned().expect("app icon"))
        .icon_as_template(false)
        .tooltip("Devilbox Boost")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            let app = app.clone();
            let id = event.id().as_ref().to_string();
            if let Some(url) = id.strip_prefix("proj:") {
                let _ = app.opener().open_url(url, None::<&str>);
                return;
            }
            if let Some(rest) = id.strip_prefix("tool:") {
                let mut parts = rest.splitn(3, '|');
                let (label, title, url) = (parts.next().unwrap_or("tool"), parts.next().unwrap_or("Tool"), parts.next().unwrap_or(""));
                let _ = open_tool(app.clone(), label.to_string(), title.to_string(), url.to_string());
                return;
            }
            match id.as_str() {
                "open" => show_main(&app),
                "intranet" => {
                    let state = app.state::<AppState>();
                    if let Ok(root) = root_or_err(&state) {
                        let env = environment_for(&root);
                        let _ = app.opener().open_url(env.ui_url, None::<&str>);
                    }
                }
                "toggle" | "restart" => {
                    std::thread::spawn(move || {
                        let state = app.state::<AppState>();
                        if let Ok(root) = root_or_err(&state) {
                            state.watch.lock().unwrap().primed = false;
                            let action = if id == "restart" {
                                "restart"
                            } else {
                                let running = stack_status_sync(&root).map(|s| summarize(&s).0).unwrap_or(0);
                                if running > 0 { "stop" } else { "start" }
                            };
                            let args: &[&str] = match action {
                                "start" => &["up", "-d"],
                                "stop" => &["stop"],
                                _ => &["restart"],
                            };
                            let _ = compose(&root, args);
                            app.state::<AppState>().watch.lock().unwrap().primed = false;
                        }
                        refresh_tray(&app);
                    });
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick { .. } = event {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;
    *app.state::<AppState>().tray.lock().unwrap() = Some(tray);
    Ok(())
}

#[cfg(desktop)]
fn register_quick_open(app: &AppHandle) -> tauri::Result<()> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    show_main(app);
                    let _ = app.emit("quick-open", ());
                }
            })
            .build(),
    )?;
    if let Err(e) = app.global_shortcut().register(QUICK_OPEN_SHORTCUT) {
        eprintln!("quick-open shortcut not registered: {e}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grab_reads_wp_config_and_dotenv() {
        let wp = "define( 'DB_NAME', 'eotb' );\n$table_prefix = 'wp_';";
        assert_eq!(grab(wp, "DB_NAME").as_deref(), Some("eotb"));
        let env = "APP_NAME=x\nDB_DATABASE=\"ipwas\"\n";
        assert_eq!(grab(env, "DB_DATABASE").as_deref(), Some("ipwas"));
        assert_eq!(grab(env, "DB_HOST"), None);
        assert_eq!(grab("DB_DATABASE=shop # main db\n", "DB_DATABASE").as_deref(), Some("shop"));
        assert_eq!(grab("DB_DATABASE='with # inside'\n", "DB_DATABASE").as_deref(), Some("with # inside"));
    }

    #[test]
    fn parse_ps_accepts_ndjson_and_arrays() {
        let line = r#"{"Service":"mysql","Name":"d-mysql-1","State":"running","Health":"healthy","Status":"Up 1 hour","Image":"mariadb:11.8","Publishers":[{"URL":"127.0.0.1","TargetPort":3306,"PublishedPort":3306,"Protocol":"tcp"},{"URL":"","TargetPort":3306,"PublishedPort":0,"Protocol":"tcp"}]}"#;
        let nd = parse_ps(line);
        assert_eq!(nd.len(), 1);
        assert_eq!(nd[0].ports.len(), 1, "unpublished ports are dropped");
        assert_eq!(nd[0].ports[0].published, 3306);
        let arr = parse_ps(&format!("[{line}]"));
        assert_eq!(arr[0].service, "mysql");
        assert_eq!(summarize(&arr), (1, 1));
    }

    #[test]
    fn environment_derives_https_vhost_urls() {
        let dir = std::env::temp_dir().join(format!("dbx-test-{}", std::process::id()));
        fs::create_dir_all(dir.join("data/www/demo")).unwrap();
        fs::create_dir_all(dir.join("data/www/cake/webroot")).unwrap();
        fs::write(dir.join("docker-compose.yml"), "services: {}\n").unwrap();
        fs::write(dir.join(".env"), "TLD_SUFFIX=loc\nHTTPD_VHOST_SSL_TYPE=both\nHOST_PORT_HTTPD_SSL=8443\nPHP_SERVER=8.4\n").unwrap();
        fs::write(dir.join("data/www/demo/artisan"), "").unwrap();
        fs::write(dir.join("data/www/demo/.env"), "DB_DATABASE=demo_db\n").unwrap();
        let env = environment_for(&dir);
        assert_eq!(env.scheme, "https");
        assert_eq!(env.vhost_port, ":8443");
        assert_eq!(env.ui_url, "https://localhost:8443");
        let projects = projects_sync(&dir);
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].kind, "cakephp");
        assert_eq!(projects[1].kind, "laravel");
        assert_eq!(projects[1].url, "https://demo.loc:8443");
        assert_eq!(projects[1].db, "demo_db");
        assert!(is_devilbox_root(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("a'b"), "'a'\\''b'");
        assert_eq!(shell_quote("/tmp/x y.sql"), "'/tmp/x y.sql'");
    }
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(ops::DevState::default())
        .manage(ops::LogsState::default())
        .manage(term::TermState::default())
        .manage(AppState {
            settings: Mutex::new(Settings::default()),
            tray: Mutex::new(None),
            watch: Mutex::new(Watch::default()),
        })
        .setup(|app| {
            let handle = app.handle().clone();
            let settings = load_settings(&handle);
            *app.state::<AppState>().settings.lock().unwrap() = settings;
            build_tray(&handle)?;
            #[cfg(desktop)]
            register_quick_open(&handle)?;
            #[cfg(desktop)]
            handle.plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))?;

            // Poll the stack in the background and keep tray + UI current.
            let poller = handle.clone();
            std::thread::spawn(move || loop {
                refresh_tray(&poller);
                sites::maybe_run_schedule(&poller);
                std::thread::sleep(Duration::from_secs(15));
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the main window keeps the app alive in the menu bar.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            get_environment,
            save_settings_cmd,
            pick_root,
            pick_dump,
            stack_status,
            stack_action,
            stack_logs,
            projects,
            databases,
            db_create,
            db_drop,
            import_dump,
            open_tool,
            open_url,
            open_with,
            open_db_client,
            test_notification,
            sites::check_updates,
            sites::update_site,
            sites::update_all,
            sites::create_site,
            sites::project_types,
            sites::presets,
            ops::doctor,
            ops::snapshots,
            ops::snapshot_action,
            ops::workers_status,
            ops::worker_action,
            ops::worker_logs,
            ops::media_status,
            ops::media_set,
            ops::project_logs,
            ops::dev_server_status,
            ops::dev_server_start,
            ops::dev_server_stop,
            ops::db_clients,
            ops::detect_apps,
            ops::file_manager_name,
            ops::git_status,
            ops::apply_fix,
            ops::xdebug_status,
            ops::xdebug_set,
            ops::project_scripts,
            ops::run_script,
            ops::logs_follow_start,
            ops::logs_follow_stop,
            ops::app_update_check,
            ops::autostart_get,
            ops::autostart_set,
            ops::git_panel,
            ops::git_do,
            ops::git_commit,
            ops::git_diff,
            ops::git_clone,
            ops::git_log,
            ops::git_show,
            ssh::ssh_list,
            ssh::ssh_save,
            ssh::ssh_delete,
            ssh::ssh_open,
            ssh::ssh_open_external,
            ssh::ssh_set_password,
            ssh::ssh_delete_password,
            ssh::ssh_install_key,
            ssh::ssh_pick_key,
            ssh::ssh_export,
            ssh::ssh_import,
            term::terminal_open,
            term::terminal_write,
            term::terminal_resize,
            term::terminal_close,
            files::list_dir,
            files::make_dir,
            files::upload_files,
            files::upload_plan,
            files::pick_files
        ])
        .build(tauri::generate_context!())
        .expect("error while building Devilbox Boost companion")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                ops::stop_all_dev_servers(app);
                ops::stop_all_log_follows(app);
                term::close_all(app);
            }
        });
}
