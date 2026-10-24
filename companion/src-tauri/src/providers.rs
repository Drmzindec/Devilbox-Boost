//! Repository browsing for the clone panel. Four sources, no OAuth in the app:
//! GitHub and GitLab speak through their official CLIs (gh, glab), Gitea
//! through its official CLI (tea) — those CLIs hold the auth, the app never
//! sees a token. Bitbucket has no official CLI, so workspaces are configured
//! in the app with an app password kept in the macOS Keychain.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Clone)]
pub struct RepoSource {
    /// e.g. "gh:github.com", "glab:gitlab.com", "tea:myserver", "bb:workspace"
    pub id: String,
    pub provider: String,
    pub label: String,
}

#[derive(Serialize, Clone, Default)]
pub struct Repo {
    pub full_name: String,
    pub ssh_url: String,
    pub description: String,
}

/// The app does not inherit a shell PATH, so look for a CLI in the usual
/// places as well.
fn find_cli(name: &str) -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("PATH") {
        if let Some(f) = std::env::split_paths(&p).map(|d| d.join(name)).find(|f| f.is_file()) {
            return Some(f);
        }
    }
    let mut dirs = vec![PathBuf::from("/opt/homebrew/bin"), PathBuf::from("/usr/local/bin"), PathBuf::from("/usr/bin")];
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        dirs.push(home.join("bin"));
        dirs.push(home.join(".local/bin"));
    }
    dirs.into_iter().map(|d| d.join(name)).find(|f| f.is_file())
}

fn run(cli: &PathBuf, args: &[&str], envs: &[(&str, &str)]) -> Result<String, String> {
    let mut c = Command::new(cli);
    c.args(args);
    for (k, v) in envs {
        c.env(k, v);
    }
    let out = c.output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
    }
}

/// Hosts from the glab config (~/.config/glab-cli/config.yml): the keys
/// indented two spaces under "hosts:".
fn glab_hosts() -> Vec<String> {
    let Some(home) = std::env::var_os("HOME") else { return Vec::new() };
    let Ok(text) = fs::read_to_string(PathBuf::from(home).join(".config/glab-cli/config.yml")) else { return Vec::new() };
    let mut hosts = Vec::new();
    let mut in_hosts = false;
    for line in text.lines() {
        if line.trim_end() == "hosts:" {
            in_hosts = true;
            continue;
        }
        if in_hosts {
            if line.starts_with("    ") || line.trim().is_empty() {
                continue; // values under a host
            }
            if let Some(rest) = line.strip_prefix("  ") {
                if !rest.starts_with(' ') && rest.trim_end().ends_with(':') {
                    hosts.push(rest.trim_end().trim_end_matches(':').to_string());
                    continue;
                }
            }
            break; // left the hosts: block
        }
    }
    hosts
}

// ----- Bitbucket workspaces (app passwords in the Keychain) -----------------

const BB_KC_SERVICE: &str = "devilbox-boost-bitbucket";

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct BbWorkspace {
    pub workspace: String,
    pub username: String,
}

fn bb_path(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_config_dir().unwrap_or_else(|_| std::env::temp_dir());
    let _ = fs::create_dir_all(&dir);
    dir.join("bitbucket.json")
}

fn bb_load(app: &AppHandle) -> Vec<BbWorkspace> {
    fs::read_to_string(bb_path(app)).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

#[tauri::command]
pub fn bb_add(app: AppHandle, workspace: String, username: String, app_password: String) -> Result<(), String> {
    let (ws, user) = (workspace.trim().to_string(), username.trim().to_string());
    if ws.is_empty() || user.is_empty() || app_password.is_empty() {
        return Err("Workspace, username and app password are all required".into());
    }
    let out = Command::new("security")
        .args(["add-generic-password", "-U", "-s", BB_KC_SERVICE, "-a", &format!("{user}@{ws}"), "-w", &app_password])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("keychain: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    let mut list = bb_load(&app);
    list.retain(|w| w.workspace != ws);
    list.push(BbWorkspace { workspace: ws, username: user });
    fs::write(bb_path(&app), serde_json::to_string_pretty(&list).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bb_remove(app: AppHandle, workspace: String) -> Result<(), String> {
    let mut list = bb_load(&app);
    if let Some(w) = list.iter().find(|w| w.workspace == workspace) {
        let _ = Command::new("security")
            .args(["delete-generic-password", "-s", BB_KC_SERVICE, "-a", &format!("{}@{}", w.username, w.workspace)])
            .output();
    }
    list.retain(|w| w.workspace != workspace);
    fs::write(bb_path(&app), serde_json::to_string_pretty(&list).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

// ----- CLI install and login --------------------------------------------------

#[derive(Serialize, Clone)]
pub struct CliStatus {
    pub name: String,
    pub installed: bool,
    pub authenticated: bool,
}

#[tauri::command]
pub fn cli_status() -> Vec<CliStatus> {
    ["glab", "gh", "tea"]
        .iter()
        .map(|n| {
            let cli = find_cli(n);
            let authenticated = cli
                .as_ref()
                .map(|c| match *n {
                    "gh" => run(c, &["auth", "status"], &[]).is_ok(),
                    "glab" => run(c, &["auth", "status"], &[]).is_ok(),
                    _ => run(c, &["logins", "list", "--output", "json"], &[]).map(|o| o.trim() != "[]" && !o.trim().is_empty()).unwrap_or(false),
                })
                .unwrap_or(false);
            CliStatus { name: n.to_string(), installed: cli.is_some(), authenticated }
        })
        .collect()
}

fn writable_bin() -> Result<PathBuf, String> {
    let usr_local = PathBuf::from("/usr/local/bin");
    let probe = usr_local.join(".dbx-write-test");
    if fs::write(&probe, "").is_ok() {
        let _ = fs::remove_file(&probe);
        return Ok(usr_local);
    }
    let home = std::env::var_os("HOME").ok_or("no HOME")?;
    let dir = PathBuf::from(home).join(".local/bin");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn curl_json(url: &str) -> Result<serde_json::Value, String> {
    let out = Command::new("curl")
        .args(["-fsSL", "-m", "30", "-H", "User-Agent: devilbox-boost-companion"])
        .arg(url)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("could not reach {url}"));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())
}

fn curl_download(url: &str, dest: &PathBuf) -> Result<(), String> {
    let out = Command::new("curl")
        .args(["-fsSL", "-m", "300", "-H", "User-Agent: devilbox-boost-companion", "-o"])
        .arg(dest)
        .arg(url)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() { Ok(()) } else { Err(format!("download failed: {url}")) }
}

/// Download the official release of a provider CLI and install the binary.
#[tauri::command]
pub async fn cli_install(name: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let arch = if std::env::consts::ARCH == "aarch64" { "arm64" } else { "amd64" };
        let bin_dir = writable_bin()?;
        let tmp = std::env::temp_dir().join(format!("dbx-cli-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
        let installed: PathBuf = match name.as_str() {
            "glab" => {
                let rel = curl_json("https://gitlab.com/api/v4/projects/gitlab-org%2Fcli/releases/permalink/latest")?;
                let want = format!("darwin_{arch}.tar.gz");
                let url = rel.pointer("/assets/links").and_then(|l| l.as_array()).into_iter().flatten()
                    .find_map(|l| {
                        let u = l.get("direct_asset_url").and_then(|x| x.as_str())?;
                        u.ends_with(&want).then(|| u.to_string())
                    })
                    .ok_or("no macOS build in the latest glab release")?;
                let tgz = tmp.join("glab.tar.gz");
                curl_download(&url, &tgz)?;
                let out = Command::new("tar").args(["-xzf"]).arg(&tgz).arg("-C").arg(&tmp).output().map_err(|e| e.to_string())?;
                if !out.status.success() {
                    return Err("could not extract glab".into());
                }
                // the tarball contains bin/glab
                let found = walk_find(&tmp, "glab").ok_or("glab binary not found in the archive")?;
                let dest = bin_dir.join("glab");
                fs::copy(&found, &dest).map_err(|e| e.to_string())?;
                dest
            }
            "gh" => {
                let rel = curl_json("https://api.github.com/repos/cli/cli/releases/latest")?;
                let want = format!("macOS_{arch}.zip");
                let url = rel.get("assets").and_then(|a| a.as_array()).into_iter().flatten()
                    .find_map(|a| {
                        let n = a.get("name").and_then(|x| x.as_str())?;
                        let u = a.get("browser_download_url").and_then(|x| x.as_str())?;
                        n.ends_with(&want).then(|| u.to_string())
                    })
                    .ok_or("no macOS build in the latest gh release")?;
                let zip = tmp.join("gh.zip");
                curl_download(&url, &zip)?;
                let out = Command::new("unzip").args(["-oq"]).arg(&zip).arg("-d").arg(&tmp).output().map_err(|e| e.to_string())?;
                if !out.status.success() {
                    return Err("could not extract gh".into());
                }
                let found = walk_find(&tmp, "gh").ok_or("gh binary not found in the archive")?;
                let dest = bin_dir.join("gh");
                fs::copy(&found, &dest).map_err(|e| e.to_string())?;
                dest
            }
            "tea" => {
                let rel = curl_json("https://gitea.com/api/v1/repos/gitea/tea/releases/latest")?;
                let want = format!("darwin-{arch}");
                let url = rel.get("assets").and_then(|a| a.as_array()).into_iter().flatten()
                    .find_map(|a| {
                        let n = a.get("name").and_then(|x| x.as_str())?;
                        let u = a.get("browser_download_url").and_then(|x| x.as_str())?;
                        n.ends_with(&want).then(|| u.to_string())
                    })
                    .ok_or("no macOS build in the latest tea release")?;
                let dest = bin_dir.join("tea");
                curl_download(&url, &dest)?;
                dest
            }
            other => return Err(format!("unknown cli {other}")),
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&installed, fs::Permissions::from_mode(0o755));
        }
        // macOS quarantines downloaded binaries; clear it so Gatekeeper lets a
        // plain CLI run.
        let _ = Command::new("xattr").args(["-d", "com.apple.quarantine"]).arg(&installed).output();
        let _ = fs::remove_dir_all(&tmp);
        Ok(format!("Installed {}", installed.display()))
    })
    .await
    .map_err(|e| e.to_string())?
}

fn walk_find(dir: &PathBuf, name: &str) -> Option<PathBuf> {
    let rd = fs::read_dir(dir).ok()?;
    for e in rd.filter_map(|e| e.ok()) {
        let p = e.path();
        if p.is_dir() {
            if let Some(f) = walk_find(&p, name) {
                return Some(f);
            }
        } else if p.file_name().map(|n| n == name).unwrap_or(false) {
            return Some(p);
        }
    }
    None
}

/// The auth flows are interactive, so hand them to the user's terminal app.
#[tauri::command]
pub fn cli_login(app: AppHandle, state: tauri::State<crate::AppState>, name: String) -> Result<(), String> {
    let login: &[&str] = match name.as_str() {
        "gh" => &["auth", "login"],
        "glab" => &["auth", "login"],
        "tea" => &["login", "add"],
        _ => return Err("unknown cli".into()),
    };
    let cli = find_cli(&name).ok_or(format!("{name} is not installed"))?;
    let terminal = {
        let s = state.settings.lock().unwrap();
        if s.terminal.trim().is_empty() { "Terminal".to_string() } else { s.terminal.clone() }
    };
    let line = std::iter::once(cli.to_string_lossy().to_string())
        .chain(login.iter().map(|a| a.to_string()))
        .map(|a| crate::shell_quote(&a))
        .collect::<Vec<_>>()
        .join(" ");
    let dir = app.path().app_cache_dir().unwrap_or_else(|_| std::env::temp_dir());
    let _ = fs::create_dir_all(&dir);
    let file = dir.join(format!("login-{name}.command"));
    fs::write(&file, format!("#!/bin/bash\nclear\n{line}\necho\necho 'Done — you can close this window and reopen the clone panel in Devilbox Boost.'\n")).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&file, fs::Permissions::from_mode(0o755));
    }
    crate::platform::open_with(&terminal, &file.to_string_lossy())
}

// ----- Sources and repo listing ---------------------------------------------

#[tauri::command]
pub fn repo_sources(app: AppHandle) -> Vec<RepoSource> {
    let mut v = Vec::new();
    if let Some(gh) = find_cli("gh") {
        if run(&gh, &["auth", "status"], &[]).is_ok() {
            v.push(RepoSource { id: "gh:".into(), provider: "github".into(), label: "GitHub (gh)".into() });
        }
    }
    if find_cli("glab").is_some() {
        for host in glab_hosts() {
            v.push(RepoSource { id: format!("glab:{host}"), provider: "gitlab".into(), label: format!("GitLab · {host}") });
        }
    }
    if let Some(tea) = find_cli("tea") {
        if let Ok(out) = run(&tea, &["logins", "list", "--output", "json"], &[]) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&out) {
                for login in val.as_array().into_iter().flatten() {
                    let name = login.get("name").or_else(|| login.get("Name")).and_then(|x| x.as_str()).unwrap_or("");
                    let url = login.get("url").or_else(|| login.get("URL")).and_then(|x| x.as_str()).unwrap_or("");
                    if !name.is_empty() {
                        v.push(RepoSource { id: format!("tea:{name}"), provider: "gitea".into(), label: format!("Gitea · {}", if url.is_empty() { name } else { url }) });
                    }
                }
            }
        }
    }
    for w in bb_load(&app) {
        v.push(RepoSource { id: format!("bb:{}", w.workspace), provider: "bitbucket".into(), label: format!("Bitbucket · {}", w.workspace) });
    }
    v
}

fn s(v: &serde_json::Value, keys: &[&str]) -> String {
    keys.iter().find_map(|k| v.get(*k).and_then(|x| x.as_str())).unwrap_or("").to_string()
}

fn parse_repos(json: &str, name_keys: &[&str], ssh_keys: &[&str], desc_keys: &[&str]) -> Vec<Repo> {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(json) else { return Vec::new() };
    let arr = val.as_array().cloned().or_else(|| val.get("values").and_then(|x| x.as_array().cloned())).unwrap_or_default();
    arr.iter()
        .map(|r| Repo { full_name: s(r, name_keys), ssh_url: s(r, ssh_keys), description: s(r, desc_keys) })
        .filter(|r| !r.full_name.is_empty())
        .collect()
}

#[tauri::command]
pub async fn repo_list(app: AppHandle, source: String) -> Result<Vec<Repo>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (kind, rest) = source.split_once(':').ok_or("bad source")?;
        match kind {
            "gh" => {
                let gh = find_cli("gh").ok_or("gh is not installed")?;
                let out = run(&gh, &["repo", "list", "--limit", "100", "--json", "nameWithOwner,sshUrl,description"], &[])?;
                Ok(parse_repos(&out, &["nameWithOwner"], &["sshUrl"], &["description"]))
            }
            "glab" => {
                let glab = find_cli("glab").ok_or("glab is not installed")?;
                let out = run(
                    &glab,
                    &["api", "projects?membership=true&per_page=100&order_by=last_activity_at&simple=false"],
                    &[("GITLAB_HOST", rest), ("NO_COLOR", "1")],
                )?;
                Ok(parse_repos(&out, &["path_with_namespace"], &["ssh_url_to_repo"], &["description"]))
            }
            "tea" => {
                let tea = find_cli("tea").ok_or("tea is not installed")?;
                let out = run(&tea, &["repos", "list", "--login", rest, "--output", "json"], &[])?;
                let mut repos = parse_repos(&out, &["full_name", "FullName", "fullname"], &["ssh_url", "SSHURL", "ssh"], &["description", "Description"]);
                // Older tea builds print name/owner separately.
                if repos.is_empty() {
                    repos = parse_repos(&out, &["name", "Name"], &["ssh_url", "SSHURL", "ssh"], &["description", "Description"]);
                }
                Ok(repos)
            }
            "bb" => {
                let list = bb_load(&app);
                let ws = list.iter().find(|w| w.workspace == rest).ok_or("workspace not configured")?;
                let pw = Command::new("security")
                    .args(["find-generic-password", "-s", BB_KC_SERVICE, "-a", &format!("{}@{}", ws.username, ws.workspace), "-w"])
                    .output()
                    .ok()
                    .filter(|o| o.status.success())
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim_end_matches('\n').to_string())
                    .ok_or("app password not found in the Keychain")?;
                let out = Command::new("curl")
                    .args(["-fsSL", "-m", "20", "-u", &format!("{}:{}", ws.username, pw)])
                    .arg(format!("https://api.bitbucket.org/2.0/repositories/{}?pagelen=100&sort=-updated_on", ws.workspace))
                    .output()
                    .map_err(|e| e.to_string())?;
                if !out.status.success() {
                    return Err("Bitbucket API request failed (check the app password and workspace)".into());
                }
                let text = String::from_utf8_lossy(&out.stdout).to_string();
                let mut repos = parse_repos(&text, &["full_name"], &[], &["description"]);
                // ssh clone link lives in links.clone[name=ssh].href
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                    let arr = val.get("values").and_then(|x| x.as_array().cloned()).unwrap_or_default();
                    for (repo, raw) in repos.iter_mut().zip(arr.iter()) {
                        if let Some(clones) = raw.pointer("/links/clone").and_then(|x| x.as_array()) {
                            if let Some(ssh) = clones.iter().find(|c| c.get("name").and_then(|n| n.as_str()) == Some("ssh")) {
                                repo.ssh_url = ssh.get("href").and_then(|h| h.as_str()).unwrap_or("").to_string();
                            }
                        }
                    }
                }
                Ok(repos)
            }
            other => Err(format!("unknown provider {other}")),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_provider_payloads() {
        let gh = r#"[{"nameWithOwner":"me/app","sshUrl":"git@github.com:me/app.git","description":"x"}]"#;
        let r = parse_repos(gh, &["nameWithOwner"], &["sshUrl"], &["description"]);
        assert_eq!(r[0].full_name, "me/app");
        let gl = r#"[{"path_with_namespace":"grp/site","ssh_url_to_repo":"git@gitlab.com:grp/site.git","description":null}]"#;
        let r = parse_repos(gl, &["path_with_namespace"], &["ssh_url_to_repo"], &["description"]);
        assert_eq!(r[0].ssh_url, "git@gitlab.com:grp/site.git");
        let bb = r#"{"values":[{"full_name":"ws/repo","description":"d"}]}"#;
        let r = parse_repos(bb, &["full_name"], &[], &["description"]);
        assert_eq!(r[0].full_name, "ws/repo");
        assert!(parse_repos("not json", &["x"], &["y"], &["z"]).is_empty());
    }
}
