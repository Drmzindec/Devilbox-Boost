//! Devilbox Boost companion: a small Tauri app that watches the Docker stack,
//! lists the projects in data/www and launches the bundled admin tools.
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
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

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
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            root: String::new(),
            db_client: "Sequel Ace".into(),
            editor: "Visual Studio Code".into(),
            terminal: "Terminal".into(),
        }
    }
}

struct AppState {
    settings: Mutex<Settings>,
    tray: Mutex<Option<TrayIcon>>,
    tray_status: Mutex<Option<MenuItem<tauri::Wry>>>,
    tray_toggle: Mutex<Option<MenuItem<tauri::Wry>>>,
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

fn save_settings(app: &AppHandle, s: &Settings) -> Result<(), String> {
    let json = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    fs::write(settings_path(app), json).map_err(|e| e.to_string())
}

fn current_settings(state: &State<AppState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

fn root_or_err(state: &State<AppState>) -> Result<PathBuf, String> {
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
}

fn environment_for(root: &Path) -> Environment {
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
    Environment {
        root: root.to_string_lossy().to_string(),
        tld: get("TLD_SUFFIX", "loc"),
        ui_url: format!("{scheme}://localhost{vhost_port}"),
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
    }
}

// ---------------------------------------------------------------------------
// docker compose
// ---------------------------------------------------------------------------

fn docker_path() -> String {
    for p in ["/usr/local/bin/docker", "/opt/homebrew/bin/docker", "/Applications/Docker.app/Contents/Resources/bin/docker"] {
        if Path::new(p).is_file() {
            return p.to_string();
        }
    }
    "docker".to_string()
}

fn compose(root: &Path, args: &[&str]) -> Result<String, String> {
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

fn grab<'a>(text: &'a str, key: &str) -> Option<String> {
    // Matches define('KEY', 'value') in wp-config.php and KEY=value in .env
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with(&format!("{key}=")) {
            return Some(t[key.len() + 1..].trim().trim_matches('"').trim_matches('\'').to_string());
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

fn detect_project(dir: &Path, env: &Environment) -> Project {
    let name = dir.file_name().unwrap_or_default().to_string_lossy().to_string();
    let has = |rel: &str| dir.join(rel).exists();
    let kind = if has("please") {
        "statamic"
    } else if has("artisan") {
        "laravel"
    } else if has("wp-config.php") || has("wp-content") || has("wp-config-sample.php") {
        "wordpress"
    } else if has("bin/console") {
        "symfony"
    } else if has("next.config.mjs") || has("next.config.js") {
        "next.js"
    } else if has("index.html") {
        "static"
    } else if has("index.php") || has("public/index.php") {
        "php"
    } else {
        "other"
    };
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

fn projects_sync(root: &Path) -> Vec<Project> {
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
async fn stack_status(state: State<'_, AppState>) -> Result<Vec<ServiceStatus>, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || stack_status_sync(&root))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn stack_action(app: AppHandle, state: State<'_, AppState>, action: String, service: Option<String>) -> Result<String, String> {
    let root = root_or_err(&state)?;
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
        let pw = format!("-p{}", env.mysql_root_password);
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

/// Open (or focus) a dedicated window for a web tool. Used for pages that
/// refuse to be framed (phpMyAdmin, Adminer send X-Frame-Options: DENY).
#[tauri::command]
fn open_tool(app: AppHandle, label: String, title: String, url: String) -> Result<(), String> {
    let safe_label: String = label.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect();
    if let Some(w) = app.get_webview_window(&safe_label) {
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }
    let parsed = url.parse().map_err(|e| format!("bad url: {e}"))?;
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

/// `open -a <App> <target>` on macOS; falls back to the system opener elsewhere.
#[tauri::command]
fn open_with(app: AppHandle, app_name: String, target: String) -> Result<(), String> {
    if cfg!(target_os = "macos") && !app_name.trim().is_empty() {
        let out = Command::new("open").arg("-a").arg(&app_name).arg(&target).output().map_err(|e| e.to_string())?;
        if out.status.success() {
            return Ok(());
        }
        return Err(format!("could not open with {app_name}: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    app.opener().open_path(target, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_db_client(state: State<AppState>, db: String) -> Result<(), String> {
    let root = root_or_err(&state)?;
    let env = environment_for(&root);
    let s = current_settings(&state);
    let url = format!(
        "mysql://root:{}@127.0.0.1:{}/{}?name={}&env=local",
        env.mysql_root_password, env.mysql_host_port, db, db
    );
    let out = Command::new("open").arg("-a").arg(&s.db_client).arg(&url).output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{} could not be opened (is it installed?): {}",
            s.db_client,
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

// ---------------------------------------------------------------------------
// Tray
// ---------------------------------------------------------------------------

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn refresh_tray(app: &AppHandle) {
    let state = app.state::<AppState>();
    let settings = current_settings(&state);
    let (text, toggle, status) = match discover_root(&settings.root) {
        None => ("Devilbox folder not set".to_string(), "Start stack".to_string(), Vec::new()),
        Some(root) => match stack_status_sync(&root) {
            Ok(status) => {
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
    if let Some(item) = state.tray_status.lock().unwrap().as_ref() {
        let _ = item.set_text(&text);
    }
    if let Some(item) = state.tray_toggle.lock().unwrap().as_ref() {
        let _ = item.set_text(&toggle);
    }
    if let Some(tray) = state.tray.lock().unwrap().as_ref() {
        let _ = tray.set_tooltip(Some(format!("Devilbox Boost: {text}")));
        let (running, total) = summarize(&status);
        let glyph = if total == 0 || running == 0 { "○" } else if running == total { "●" } else { "◐" };
        let _ = tray.set_title(Some(glyph));
    }
    let _ = app.emit("stack-status", &status);
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let status_item = MenuItem::with_id(app, "status", "Checking stack…", false, None::<&str>)?;
    let open_item = MenuItem::with_id(app, "open", "Open Devilbox Boost", true, Some("CmdOrCtrl+Shift+D"))?;
    let toggle_item = MenuItem::with_id(app, "toggle", "Start stack", true, None::<&str>)?;
    let restart_item = MenuItem::with_id(app, "restart", "Restart stack", true, None::<&str>)?;
    let intranet_item = MenuItem::with_id(app, "intranet", "Open intranet in browser", true, None::<&str>)?;
    let quit_item = PredefinedMenuItem::quit(app, Some("Quit Devilbox Boost"))?;
    let menu = Menu::with_items(
        app,
        &[
            &status_item,
            &PredefinedMenuItem::separator(app)?,
            &open_item,
            &intranet_item,
            &PredefinedMenuItem::separator(app)?,
            &toggle_item,
            &restart_item,
            &PredefinedMenuItem::separator(app)?,
            &quit_item,
        ],
    )?;

    let tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().cloned().expect("app icon"))
        .icon_as_template(false)
        .tooltip("Devilbox Boost")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            let app = app.clone();
            match event.id().as_ref() {
                "open" => show_main(&app),
                "intranet" => {
                    let state = app.state::<AppState>();
                    if let Ok(root) = root_or_err(&state) {
                        let env = environment_for(&root);
                        let _ = app.opener().open_url(env.ui_url, None::<&str>);
                    }
                }
                "toggle" | "restart" => {
                    let id = event.id().as_ref().to_string();
                    std::thread::spawn(move || {
                        let state = app.state::<AppState>();
                        if let Ok(root) = root_or_err(&state) {
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

    let state = app.state::<AppState>();
    *state.tray.lock().unwrap() = Some(tray);
    *state.tray_status.lock().unwrap() = Some(status_item);
    *state.tray_toggle.lock().unwrap() = Some(toggle_item);
    Ok(())
}

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
        fs::write(dir.join("docker-compose.yml"), "services: {}\n").unwrap();
        fs::write(dir.join(".env"), "TLD_SUFFIX=loc\nHTTPD_VHOST_SSL_TYPE=both\nHOST_PORT_HTTPD_SSL=8443\nPHP_SERVER=8.4\n").unwrap();
        fs::write(dir.join("data/www/demo/artisan"), "").unwrap();
        fs::write(dir.join("data/www/demo/.env"), "DB_DATABASE=demo_db\n").unwrap();
        let env = environment_for(&dir);
        assert_eq!(env.scheme, "https");
        assert_eq!(env.vhost_port, ":8443");
        assert_eq!(env.ui_url, "https://localhost:8443");
        let projects = projects_sync(&dir);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].kind, "laravel");
        assert_eq!(projects[0].url, "https://demo.loc:8443");
        assert_eq!(projects[0].db, "demo_db");
        assert!(is_devilbox_root(&dir));
        let _ = fs::remove_dir_all(&dir);
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
        .manage(AppState {
            settings: Mutex::new(Settings::default()),
            tray: Mutex::new(None),
            tray_status: Mutex::new(None),
            tray_toggle: Mutex::new(None),
        })
        .setup(|app| {
            let handle = app.handle().clone();
            let settings = load_settings(&handle);
            *app.state::<AppState>().settings.lock().unwrap() = settings;
            build_tray(&handle)?;

            // Poll the stack in the background and keep tray + UI current.
            let poller = handle.clone();
            std::thread::spawn(move || loop {
                refresh_tray(&poller);
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
            stack_status,
            stack_action,
            stack_logs,
            projects,
            databases,
            open_tool,
            open_url,
            open_with,
            open_db_client
        ])
        .run(tauri::generate_context!())
        .expect("error while running Devilbox Boost companion");
}
