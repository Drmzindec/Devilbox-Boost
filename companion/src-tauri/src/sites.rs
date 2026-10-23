//! Site operations: update checks and updates (per site, in bulk, on a
//! schedule) and new-project creation. The work is done by the scripts in
//! <root>/tools so the `dbox` CLI and the app share one implementation; this
//! module runs them, streams their output to the UI and parses their
//! machine-readable lines.

use crate::{root_or_err, save_settings, AppState};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter, Manager, State};

// ---------------------------------------------------------------------------
// Streaming command runner
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
pub struct JobLine {
    pub job: String,
    pub line: String,
}

fn pump(reader: Box<dyn Read + Send>, app: AppHandle, job: String) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut all = String::new();
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            let _ = app.emit("job-output", JobLine { job: job.clone(), line: line.clone() });
            all.push_str(&line);
            all.push('\n');
        }
        all
    })
}

/// Run a command, stream every output line to the UI as a `job-output` event
/// and return the collected output (Err when the exit status is non-zero).
pub fn run_streaming(app: &AppHandle, job: &str, mut cmd: Command) -> Result<String, String> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null());
    let mut child = cmd.spawn().map_err(|e| format!("cannot start command: {e}"))?;
    let out = pump(Box::new(child.stdout.take().unwrap()), app.clone(), job.to_string());
    let err = pump(Box::new(child.stderr.take().unwrap()), app.clone(), job.to_string());
    let status = child.wait().map_err(|e| e.to_string())?;
    let text = format!("{}{}", out.join().unwrap_or_default(), err.join().unwrap_or_default());
    if status.success() {
        Ok(text)
    } else {
        Err(text)
    }
}

/// `bash <root>/tools/<script> args…` with DEVILBOX_ROOT set and no TTY (so the
/// scripts print their machine-readable format).
pub fn tool(root: &Path, script: &str, args: &[&str]) -> Command {
    let mut c = Command::new("bash");
    c.arg(root.join("tools").join(script))
        .args(args)
        .current_dir(root)
        .env("DEVILBOX_ROOT", root)
        .env("TERM", "dumb");
    c
}

pub fn tool_output(root: &Path, script: &str, args: &[&str]) -> Result<String, String> {
    let out = tool(root, script, args).output().map_err(|e| e.to_string())?;
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    if out.status.success() {
        Ok(text)
    } else {
        Err(text)
    }
}

// ---------------------------------------------------------------------------
// Types, presets
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
pub struct ProjectType {
    pub id: String,
    pub description: String,
}

#[derive(Serialize, Clone)]
pub struct Preset {
    pub kind: String,
    pub name: String,
    pub description: String,
}

#[tauri::command]
pub fn project_types(state: State<AppState>) -> Result<Vec<ProjectType>, String> {
    let root = root_or_err(&state)?;
    let out = tool_output(&root, "new-project.sh", &["--list"])?;
    Ok(out
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            let (id, rest) = l.split_once(char::is_whitespace)?;
            Some(ProjectType { id: id.to_string(), description: rest.trim().to_string() })
        })
        .collect())
}

#[tauri::command]
pub fn presets(state: State<AppState>) -> Result<Vec<Preset>, String> {
    let root = root_or_err(&state)?;
    let out = tool_output(&root, "new-project.sh", &["--list-presets"])?;
    Ok(out
        .lines()
        .filter_map(|l| {
            let mut p = l.splitn(3, '|');
            Some(Preset { kind: p.next()?.to_string(), name: p.next()?.to_string(), description: p.next().unwrap_or("").to_string() })
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Update checks and updates
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone, Default)]
pub struct UpdateInfo {
    pub project: String,
    pub plugins: Vec<String>,
    pub themes: Vec<String>,
    pub core: Option<String>,
    pub packages: Vec<String>,
    pub error: Option<String>,
}

fn parse_check(project: &str, text: &str) -> UpdateInfo {
    let mut info = UpdateInfo { project: project.to_string(), ..Default::default() };
    for line in text.lines() {
        let f: Vec<&str> = line.split('|').collect();
        match f.as_slice() {
            ["PLUGIN", _, name, cur, new] => info.plugins.push(format!("{name} {cur} → {new}")),
            ["THEME", _, name, cur, new] => info.themes.push(format!("{name} {cur} → {new}")),
            ["CORE", _, cur, new] => info.core = Some(format!("{cur} → {new}")),
            ["PACKAGE", _, name, cur, new] => info.packages.push(format!("{name} {cur} → {new}")),
            ["ERROR", _, msg] => info.error = Some(msg.to_string()),
            _ => {}
        }
    }
    info
}

#[derive(Serialize, Clone)]
pub struct BulkResult {
    pub project: String,
    pub ok: bool,
    pub message: String,
}

fn parse_results(text: &str) -> Vec<BulkResult> {
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('|').collect();
            match f.as_slice() {
                ["RESULT", p, status, msg] => Some(BulkResult { project: p.to_string(), ok: *status == "ok", message: msg.to_string() }),
                _ => None,
            }
        })
        .collect()
}

#[tauri::command]
pub async fn check_updates(state: State<'_, AppState>, project: String) -> Result<UpdateInfo, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let text = tool_output(&root, "update-sites.sh", &["--check", &project]).unwrap_or_else(|e| e);
        Ok(parse_check(&project, &text))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn update_site(app: AppHandle, state: State<'_, AppState>, project: String) -> Result<BulkResult, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let job = format!("update-{project}");
        let text = run_streaming(&app, &job, tool(&root, "update-sites.sh", &[&project])).unwrap_or_else(|e| e);
        parse_results(&text)
            .into_iter()
            .next()
            .ok_or_else(|| format!("no result reported for {project}:\n{text}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

pub fn update_scope(app: &AppHandle, root: &Path, scope: &str, job: &str) -> Vec<BulkResult> {
    let text = run_streaming(app, job, tool(root, "update-sites.sh", &["--scope", scope])).unwrap_or_else(|e| e);
    parse_results(&text)
}

#[tauri::command]
pub async fn update_all(app: AppHandle, state: State<'_, AppState>, scope: String) -> Result<Vec<BulkResult>, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || Ok(update_scope(&app, &root, &scope, "update-all")))
        .await
        .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Schedule (runs while the app is open)
// ---------------------------------------------------------------------------

pub fn maybe_run_schedule(app: &AppHandle) {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap().clone();
    if !settings.schedule_enabled {
        return;
    }
    let now = chrono::Local::now();
    let today = now.format("%Y-%m-%d").to_string();
    if settings.schedule_last_run == today {
        return;
    }
    let (h, m) = settings
        .schedule_time
        .split_once(':')
        .map(|(h, m)| (h.parse::<u32>().unwrap_or(3), m.parse::<u32>().unwrap_or(0)))
        .unwrap_or((3, 0));
    let due = chrono::NaiveTime::from_hms_opt(h, m, 0).unwrap_or_default();
    if now.time() < due {
        return;
    }
    let Ok(root) = root_or_err(&state) else { return };
    {
        let mut s = state.settings.lock().unwrap();
        s.schedule_last_run = today;
        let _ = save_settings(app, &s);
    }
    let scope = settings.schedule_scope.clone();
    let app2 = app.clone();
    std::thread::spawn(move || {
        let results = update_scope(&app2, &root, &scope, "schedule");
        let failed: Vec<&str> = results.iter().filter(|r| !r.ok).map(|r| r.project.as_str()).collect();
        let body = if failed.is_empty() {
            format!("{} project(s) updated.", results.len())
        } else {
            format!("{} updated, failed: {}", results.len() - failed.len(), failed.join(", "))
        };
        crate::notify(&app2, "Scheduled site updates finished", &body);
    });
}

// ---------------------------------------------------------------------------
// New projects
// ---------------------------------------------------------------------------

#[derive(Deserialize, Clone)]
pub struct NewProject {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub admin_user: String,
    #[serde(default)]
    pub admin_password: String,
    #[serde(default)]
    pub admin_email: String,
    #[serde(default)]
    pub preset: String,
}

#[tauri::command]
pub async fn create_site(app: AppHandle, state: State<'_, AppState>, spec: NewProject) -> Result<String, String> {
    let root = root_or_err(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let job = format!("create-{}", spec.name);
        let mut args: Vec<String> = vec![spec.kind.clone(), spec.name.clone()];
        if !spec.title.is_empty() {
            args.extend(["--title".into(), spec.title.clone()]);
        }
        if !spec.admin_user.is_empty() {
            args.extend(["--admin-user".into(), spec.admin_user.clone()]);
        }
        if !spec.admin_password.is_empty() {
            args.extend(["--admin-pass".into(), spec.admin_password.clone()]);
        }
        if !spec.admin_email.is_empty() {
            args.extend(["--admin-email".into(), spec.admin_email.clone()]);
        }
        if !spec.preset.is_empty() {
            args.extend(["--preset".into(), spec.preset.clone()]);
        }
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let text = run_streaming(&app, &job, tool(&root, "new-project.sh", &refs))?;
        text.lines()
            .find_map(|l| l.split("Project ready: ").nth(1).map(|r| r.split_whitespace().next().unwrap_or("").to_string()))
            .ok_or_else(|| "project script finished without reporting a URL".to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_check_lines() {
        let text = "PLUGIN|site|akismet|5.3|5.4\nTHEME|site|twentytwentyfour|1.0|1.1\nCORE|site|6.7|6.8\nPACKAGE|site|laravel/framework|v12.1|v12.2\nDONE|site\n";
        let i = parse_check("site", text);
        assert_eq!(i.plugins, vec!["akismet 5.3 → 5.4"]);
        assert_eq!(i.themes.len(), 1);
        assert_eq!(i.core.as_deref(), Some("6.7 → 6.8"));
        assert_eq!(i.packages, vec!["laravel/framework v12.1 → v12.2"]);
        assert!(i.error.is_none());
        assert_eq!(parse_check("x", "ERROR|x|boom\n").error.as_deref(), Some("boom"));
    }

    #[test]
    fn parses_results() {
        let r = parse_results("junk\nRESULT|a|ok|updated\nRESULT|b|fail|composer update failed\n");
        assert_eq!(r.len(), 2);
        assert!(r[0].ok && !r[1].ok);
        assert_eq!(r[1].message, "composer update failed");
    }
}
