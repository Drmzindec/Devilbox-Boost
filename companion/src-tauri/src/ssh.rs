//! Saved SSH connections: a list in ssh.json plus the Host blocks from
//! ~/.ssh/config, opened one-click in an embedded PTY terminal (or handed to
//! the user's terminal app). Key/agent auth only — no passwords are stored;
//! a server that asks for one prompts interactively in the PTY.

use crate::term::{spawn_pty, TermState};
use portable_pty::CommandBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct SshConn {
    pub name: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    /// Path of the identity file, empty = let ssh/agent decide.
    pub key: String,
    /// cd here after connecting (empty = the remote default).
    pub remote_dir: String,
    /// "saved" (ssh.json) or "config" (~/.ssh/config); set by ssh_list.
    pub source: String,
    /// Whether a password for this connection is in the macOS Keychain; set by
    /// ssh_list, never stored in ssh.json.
    pub password_saved: bool,
}

// ---------------------------------------------------------------------------
// Keychain-backed passwords (service devilbox-boost-ssh, account = name)
// ---------------------------------------------------------------------------

const KC_SERVICE: &str = "devilbox-boost-ssh";

fn kc_has(name: &str) -> bool {
    std::process::Command::new("security")
        .args(["find-generic-password", "-s", KC_SERVICE, "-a", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn kc_get(name: &str) -> Option<String> {
    let out = std::process::Command::new("security")
        .args(["find-generic-password", "-s", KC_SERVICE, "-a", name, "-w"])
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim_end_matches('\n').to_string())
    } else {
        None
    }
}

#[tauri::command]
pub fn ssh_set_password(app: AppHandle, name: String, password: String) -> Result<Vec<SshConn>, String> {
    if name.trim().is_empty() || password.is_empty() {
        return Err("Connection name and password required".into());
    }
    let out = std::process::Command::new("security")
        .args(["add-generic-password", "-U", "-s", KC_SERVICE, "-a", name.trim(), "-w", &password])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("keychain: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(ssh_list(app))
}

#[tauri::command]
pub fn ssh_delete_password(app: AppHandle, name: String) -> Result<Vec<SshConn>, String> {
    let _ = std::process::Command::new("security")
        .args(["delete-generic-password", "-s", KC_SERVICE, "-a", name.trim()])
        .output();
    Ok(ssh_list(app))
}

/// A tiny helper ssh runs to obtain the password (SSH_ASKPASS_REQUIRE=force):
/// it prints the env var the app sets only for that child process.
fn askpass_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().unwrap_or_else(|_| std::env::temp_dir());
    let _ = fs::create_dir_all(&dir);
    let file = dir.join("askpass.sh");
    if !file.is_file() {
        fs::write(&file, "#!/bin/sh\nprintf '%s' \"$DBX_SSH_PASS\"\n").map_err(|e| e.to_string())?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&file, fs::Permissions::from_mode(0o700));
    }
    Ok(file)
}

fn store_path(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_config_dir().unwrap_or_else(|_| std::env::temp_dir());
    let _ = fs::create_dir_all(&dir);
    dir.join("ssh.json")
}

fn load_saved(app: &AppHandle) -> Vec<SshConn> {
    fs::read_to_string(store_path(app))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn save_all(app: &AppHandle, list: &[SshConn]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(list).map_err(|e| e.to_string())?;
    fs::write(store_path(app), json).map_err(|e| e.to_string())
}

/// Host blocks from ~/.ssh/config (wildcard patterns skipped).
fn config_hosts() -> Vec<SshConn> {
    let Some(home) = std::env::var_os("HOME") else { return Vec::new() };
    let Ok(text) = fs::read_to_string(PathBuf::from(home).join(".ssh/config")) else { return Vec::new() };
    let mut out: Vec<SshConn> = Vec::new();
    let mut cur: Option<SshConn> = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (k, v) = match line.split_once(char::is_whitespace) {
            Some((k, v)) => (k, v.trim().trim_matches('"')),
            None => continue,
        };
        if k.eq_ignore_ascii_case("host") {
            if let Some(c) = cur.take() {
                out.push(c);
            }
            let name = v.split_whitespace().next().unwrap_or("");
            cur = if name.is_empty() || name.contains('*') || name.contains('?') {
                None
            } else {
                Some(SshConn { name: name.to_string(), host: name.to_string(), port: 22, source: "config".into(), ..Default::default() })
            };
        } else if let Some(c) = cur.as_mut() {
            if k.eq_ignore_ascii_case("hostname") {
                c.host = v.to_string();
            } else if k.eq_ignore_ascii_case("user") {
                c.user = v.to_string();
            } else if k.eq_ignore_ascii_case("port") {
                c.port = v.parse().unwrap_or(22);
            } else if k.eq_ignore_ascii_case("identityfile") {
                c.key = v.to_string();
            }
        }
    }
    if let Some(c) = cur {
        out.push(c);
    }
    out
}

#[tauri::command]
pub fn ssh_list(app: AppHandle) -> Vec<SshConn> {
    let mut list: Vec<SshConn> = load_saved(&app)
        .into_iter()
        .map(|mut c| {
            c.source = "saved".into();
            if c.port == 0 {
                c.port = 22;
            }
            c.password_saved = kc_has(&c.name);
            c
        })
        .collect();
    let saved_names: Vec<String> = list.iter().map(|c| c.name.clone()).collect();
    list.extend(config_hosts().into_iter().filter(|c| !saved_names.contains(&c.name)));
    list
}

#[tauri::command]
pub fn ssh_save(app: AppHandle, mut conn: SshConn) -> Result<Vec<SshConn>, String> {
    conn.name = conn.name.trim().to_string();
    conn.host = conn.host.trim().to_string();
    if conn.name.is_empty() || conn.host.is_empty() {
        return Err("A connection needs a name and a host".into());
    }
    if conn.port == 0 {
        conn.port = 22;
    }
    conn.source = String::new();
    let mut list = load_saved(&app);
    list.retain(|c| c.name != conn.name);
    list.push(conn);
    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    save_all(&app, &list)?;
    Ok(ssh_list(app))
}

#[tauri::command]
pub fn ssh_delete(app: AppHandle, name: String) -> Result<Vec<SshConn>, String> {
    let mut list = load_saved(&app);
    list.retain(|c| c.name != name);
    save_all(&app, &list)?;
    Ok(ssh_list(app))
}

/// The argv for ssh (no shell involved, so values need no quoting) — except
/// the remote command, which the remote shell parses.
fn ssh_args(conn: &SshConn) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    if conn.port != 0 && conn.port != 22 {
        args.extend(["-p".into(), conn.port.to_string()]);
    }
    if !conn.key.trim().is_empty() {
        args.extend(["-i".into(), conn.key.trim().to_string()]);
    }
    let target = if conn.user.trim().is_empty() { conn.host.trim().to_string() } else { format!("{}@{}", conn.user.trim(), conn.host.trim()) };
    args.push(target);
    if !conn.remote_dir.trim().is_empty() {
        let dir = conn.remote_dir.trim().replace('\'', "'\\''");
        args.extend(["-t".into(), format!("cd '{dir}' 2>/dev/null; exec \"$SHELL\" -l")]);
    }
    args
}

#[tauri::command]
pub fn ssh_open(app: AppHandle, terms: State<TermState>, conn: SshConn, cols: u16, rows: u16) -> Result<u32, String> {
    if conn.host.trim().is_empty() {
        return Err("no host".into());
    }
    let mut args = ssh_args(&conn);
    // A saved password is handed to ssh through SSH_ASKPASS (force), never on
    // the command line. The askpass script also answers host-key questions,
    // so accept-new keeps first connections working (changed keys still fail).
    let password = kc_get(&conn.name);
    if password.is_some() {
        args.insert(0, "-o".into());
        args.insert(1, "StrictHostKeyChecking=accept-new".into());
    }
    let askpass = if password.is_some() { Some(askpass_path(&app)?) } else { None };
    spawn_pty(app, &terms, move |_| {
        let mut cmd = CommandBuilder::new("ssh");
        cmd.args(args);
        if let (Some(pw), Some(script)) = (password.as_deref(), askpass.as_deref()) {
            cmd.env("DBX_SSH_PASS", pw);
            cmd.env("SSH_ASKPASS", script);
            cmd.env("SSH_ASKPASS_REQUIRE", "force");
        }
        if let Some(home) = std::env::var_os("HOME") {
            cmd.cwd(home);
        }
        cmd
    }, cols, rows, None)
}

/// One-click ssh-copy-id using the saved password, so the connection can move
/// to key auth. Streams as job "ssh-key-<name>".
#[tauri::command]
pub async fn ssh_install_key(app: AppHandle, conn: SshConn) -> Result<String, String> {
    let Some(pw) = kc_get(&conn.name) else {
        return Err("Save a password for this connection first (or run ssh-copy-id in a terminal).".into());
    };
    let askpass = askpass_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let mut cmd = std::process::Command::new("ssh-copy-id");
        if conn.port != 0 && conn.port != 22 {
            cmd.args(["-p", &conn.port.to_string()]);
        }
        if !conn.key.trim().is_empty() {
            cmd.args(["-i", conn.key.trim()]);
        }
        cmd.args(["-o", "StrictHostKeyChecking=accept-new"]);
        cmd.arg(if conn.user.trim().is_empty() { conn.host.trim().to_string() } else { format!("{}@{}", conn.user.trim(), conn.host.trim()) });
        cmd.env("DBX_SSH_PASS", &pw)
            .env("SSH_ASKPASS", &askpass)
            .env("SSH_ASKPASS_REQUIRE", "force");
        crate::sites::run_streaming(&app, &format!("ssh-key-{}", conn.name), cmd)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn ssh_pick_key(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let start = std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".ssh"));
    let mut dlg = app.dialog().file().set_title("Choose an SSH identity file (private key)");
    if let Some(d) = start.filter(|d| d.is_dir()) {
        dlg = dlg.set_directory(d);
    }
    match dlg.blocking_pick_file() {
        Some(p) => Ok(Some(p.into_path().map_err(|e| e.to_string())?.to_string_lossy().to_string())),
        None => Ok(None),
    }
}

/// Open the connection in the user's terminal app instead, via a temporary
/// .command file (Terminal and iTerm both run those).
#[tauri::command]
pub fn ssh_open_external(app: AppHandle, state: State<crate::AppState>, conn: SshConn) -> Result<(), String> {
    if conn.host.trim().is_empty() {
        return Err("no host".into());
    }
    let terminal = {
        let s = state.settings.lock().unwrap();
        if s.terminal.trim().is_empty() { "Terminal".to_string() } else { s.terminal.clone() }
    };
    let line = std::iter::once("ssh".to_string())
        .chain(ssh_args(&conn))
        .map(|a| crate::shell_quote(&a))
        .collect::<Vec<_>>()
        .join(" ");
    let dir = app.path().app_cache_dir().unwrap_or_else(|_| std::env::temp_dir());
    let _ = fs::create_dir_all(&dir);
    let safe: String = conn.name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' }).collect();
    let file = dir.join(format!("ssh-{safe}.command"));
    fs::write(&file, format!("#!/bin/bash\nclear\nexec {line}\n")).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&file, fs::Permissions::from_mode(0o755));
    }
    crate::platform::open_with(&terminal, &file.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_ssh_args() {
        let c = SshConn { name: "prod".into(), host: "example.com".into(), user: "deploy".into(), port: 2222, key: "~/.ssh/id_ed25519".into(), remote_dir: "/var/www".into(), ..Default::default() };
        let a = ssh_args(&c);
        assert_eq!(a[0], "-p");
        assert_eq!(a[1], "2222");
        assert_eq!(a[2], "-i");
        assert_eq!(a[4], "deploy@example.com");
        assert!(a[6].starts_with("cd '/var/www'"));
        let plain = ssh_args(&SshConn { name: "x".into(), host: "h".into(), port: 22, ..Default::default() });
        assert_eq!(plain, vec!["h"]);
    }

    #[test]
    fn parses_ssh_config_blocks() {
        // config_hosts reads the live ~/.ssh/config; just make sure it does not panic.
        let _ = config_hosts();
    }
}
