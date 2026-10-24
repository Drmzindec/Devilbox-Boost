//! Embedded terminals: a PTY running `docker compose exec … php bash -l` in a
//! project folder, bridged to xterm.js in the window through events.

use crate::{docker_path, root_or_err, AppState};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

struct Session {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    /// Devilbox root for docker-exec sessions (their in-container shell needs
    /// an explicit hang-up on close); None for plain host sessions (ssh).
    docker_root: Option<PathBuf>,
}

/// Killing the host-side `docker compose exec` client does NOT kill the shell
/// it started inside the container (Docker leaves exec'd processes running),
/// so every closed terminal would leak a `bash -l` in the php container. The
/// shell's PID is written to /tmp/.dbx-term-<id> on start; hang it up here.
fn kill_in_container(root: &PathBuf, id: u32) {
    let _ = Command::new(docker_path())
        .args(["compose", "exec", "-T", "php", "sh", "-c",
            &format!("p=$(cat /tmp/.dbx-term-{id} 2>/dev/null); [ -n \"$p\" ] && kill -HUP \"$p\" 2>/dev/null; rm -f /tmp/.dbx-term-{id}")])
        .current_dir(root)
        .spawn();
}

#[derive(Default)]
pub struct TermState {
    sessions: Mutex<HashMap<u32, Session>>,
    next_id: AtomicU32,
}

#[derive(Serialize, Clone)]
struct TermData {
    id: u32,
    data: String,
}

#[derive(Serialize, Clone)]
struct TermExit {
    id: u32,
}

/// Decode PTY bytes as UTF-8, carrying an incomplete trailing sequence over to
/// the next read so multibyte characters never get split.
fn decode(buf: &mut Vec<u8>) -> String {
    match std::str::from_utf8(buf) {
        Ok(s) => {
            let out = s.to_string();
            buf.clear();
            out
        }
        Err(e) => {
            let valid = e.valid_up_to();
            if e.error_len().is_some() {
                // genuinely invalid byte: replace and move on
                let out = String::from_utf8_lossy(buf).to_string();
                buf.clear();
                out
            } else {
                let out = String::from_utf8_lossy(&buf[..valid]).to_string();
                let rest = buf.split_off(valid);
                *buf = rest;
                out
            }
        }
    }
}

/// Spawn `cmd` on a fresh PTY, bridge its output to term-data/term-exit
/// events and register the session. Shared by the container terminal and ssh.
pub(crate) fn spawn_pty(app: AppHandle, terms: &TermState, build: impl FnOnce(u32) -> CommandBuilder, cols: u16, rows: u16, docker_root: Option<PathBuf>) -> Result<u32, String> {
    let pty = native_pty_system();
    let pair = pty
        .openpty(PtySize { rows: rows.max(5), cols: cols.max(20), pixel_width: 0, pixel_height: 0 })
        .map_err(|e| e.to_string())?;
    let id = terms.next_id.fetch_add(1, Ordering::SeqCst) + 1;
    let mut cmd = build(id);
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    cmd.env("LANG", "C.UTF-8");
    let child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    drop(pair.slave);
    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;
    terms.sessions.lock().unwrap().insert(id, Session { master: pair.master, writer, child, docker_root });

    std::thread::spawn(move || {
        let mut chunk = [0u8; 8192];
        let mut pending: Vec<u8> = Vec::new();
        loop {
            match reader.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    pending.extend_from_slice(&chunk[..n]);
                    let text = decode(&mut pending);
                    if !text.is_empty() {
                        let _ = app.emit("term-data", TermData { id, data: text });
                    }
                }
            }
        }
        let _ = app.emit("term-exit", TermExit { id });
    });
    Ok(id)
}

#[tauri::command]
pub fn terminal_open(app: AppHandle, terms: State<TermState>, state: State<AppState>, project: Option<String>, cols: u16, rows: u16) -> Result<u32, String> {
    let root = root_or_err(&state)?;
    let workdir = match project.as_deref() {
        Some(p) if !p.is_empty() => format!("/shared/httpd/{p}"),
        _ => "/shared/httpd".to_string(),
    };
    let root2 = root.clone();
    spawn_pty(app, &terms, move |id| {
        let mut cmd = CommandBuilder::new(docker_path());
        // Record the shell's in-container PID so terminal_close can hang it up
        // (`exec` keeps the PID, so the file holds bash's own PID).
        cmd.args(["compose", "exec", "--user", "devilbox", "-w", &workdir, "php", "sh", "-c",
            &format!("echo $$ > /tmp/.dbx-term-{id}; exec bash -l")]);
        cmd.cwd(&root2);
        cmd
    }, cols, rows, Some(root))
}

#[tauri::command]
pub fn terminal_write(terms: State<TermState>, id: u32, data: String) -> Result<(), String> {
    let mut map = terms.sessions.lock().unwrap();
    let s = map.get_mut(&id).ok_or("terminal closed")?;
    s.writer.write_all(data.as_bytes()).map_err(|e| e.to_string())?;
    s.writer.flush().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn terminal_resize(terms: State<TermState>, id: u32, cols: u16, rows: u16) -> Result<(), String> {
    let map = terms.sessions.lock().unwrap();
    let s = map.get(&id).ok_or("terminal closed")?;
    s.master.resize(PtySize { rows: rows.max(5), cols: cols.max(20), pixel_width: 0, pixel_height: 0 }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn terminal_close(terms: State<TermState>, id: u32) -> Result<(), String> {
    if let Some(mut s) = terms.sessions.lock().unwrap().remove(&id) {
        if let Some(root) = &s.docker_root {
            kill_in_container(root, id);
        }
        let _ = s.child.kill();
    }
    Ok(())
}

pub fn close_all(app: &AppHandle) {
    use tauri::Manager;
    if let Some(terms) = app.try_state::<TermState>() {
        for (id, mut s) in terms.sessions.lock().unwrap().drain() {
            if let Some(root) = &s.docker_root {
                kill_in_container(root, id);
            }
            let _ = s.child.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::decode;

    #[test]
    fn decode_keeps_partial_utf8() {
        let mut buf = "héllo".as_bytes()[..2].to_vec(); // 'h' + first byte of 'é'
        assert_eq!(decode(&mut buf), "h");
        assert_eq!(buf.len(), 1);
        buf.extend_from_slice(&"héllo".as_bytes()[2..]);
        assert_eq!(decode(&mut buf), "éllo");
        assert!(buf.is_empty());
    }
}
