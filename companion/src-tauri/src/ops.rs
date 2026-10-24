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
