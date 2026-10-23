//! The few places where the desktop OS matters: launching GUI apps, finding
//! out which ones are installed, and where the docker binary lives.
//! Everything else in the app is already cross-platform (Tauri, portable-pty,
//! the tools/ scripts under bash).

use std::path::PathBuf;
use std::process::Command;

/// Known apps by display name with their Linux command name and Windows
/// install folder name (when they exist there). macOS uses the display name
/// as the `.app` bundle name.
#[allow(dead_code)]
const APPS: &[(&str, &str, &str)] = &[
    // editors
    ("PhpStorm", "phpstorm", "JetBrains"),
    ("WebStorm", "webstorm", "JetBrains"),
    ("IntelliJ IDEA", "idea", "JetBrains"),
    ("Visual Studio Code", "code", "Microsoft VS Code"),
    ("Cursor", "cursor", "cursor"),
    ("Windsurf", "windsurf", "Windsurf"),
    ("Zed", "zed", "Zed"),
    ("Sublime Text", "subl", "Sublime Text"),
    ("Nova", "", ""),
    ("BBEdit", "", ""),
    ("Fleet", "fleet", "JetBrains"),
    ("TextMate", "", ""),
    ("Xcode", "", ""),
    // terminals
    ("Terminal", "", ""),
    ("iTerm", "", ""),
    ("Warp", "warp-terminal", "Warp"),
    ("Ghostty", "ghostty", ""),
    ("kitty", "kitty", ""),
    ("Alacritty", "alacritty", "Alacritty"),
    ("WezTerm", "wezterm", "WezTerm"),
    ("Hyper", "hyper", "Hyper"),
    ("Tabby", "tabby", "Tabby"),
    ("GNOME Terminal", "gnome-terminal", ""),
    ("Konsole", "konsole", ""),
    ("Windows Terminal", "", "WindowsApps"),
    // git clients
    ("Fork", "", "Fork"),
    ("Tower", "", "Tower"),
    ("GitHub Desktop", "github-desktop", "GitHubDesktop"),
    ("Sourcetree", "", "Sourcetree"),
    ("GitKraken", "gitkraken", "gitkraken"),
    ("Sublime Merge", "smerge", "Sublime Merge"),
    ("GitUp", "", ""),
    // sql clients
    ("Sequel Ace", "", ""),
    ("TablePlus", "tableplus", "TablePlus"),
    ("DBeaver", "dbeaver", "DBeaver"),
    ("Beekeeper Studio", "beekeeper-studio", "beekeeper-studio"),
    ("DbGate", "dbgate", "DbGate"),
    ("Querious", "", ""),
    ("HeidiSQL", "", "HeidiSQL"),
];

#[allow(dead_code)]
fn entry(name: &str) -> Option<&'static (&'static str, &'static str, &'static str)> {
    APPS.iter().find(|(n, _, _)| n.eq_ignore_ascii_case(name))
}

#[allow(dead_code)]
fn which(cmd: &str) -> bool {
    if cmd.is_empty() {
        return false;
    }
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(cmd).is_file()))
        .unwrap_or(false)
        || ["/usr/bin", "/usr/local/bin", "/snap/bin", "/var/lib/flatpak/exports/bin"].iter().any(|d| PathBuf::from(d).join(cmd).is_file())
}

/// Which of `known` display names are installed here.
pub fn installed(known: &[&str]) -> Vec<String> {
    known.iter().filter(|name| is_installed(name)).map(|s| s.to_string()).collect()
}

pub fn is_installed(name: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        let mut dirs = vec![PathBuf::from("/Applications"), PathBuf::from("/Applications/Utilities"), PathBuf::from("/System/Applications/Utilities"), PathBuf::from("/System/Applications")];
        if let Some(home) = std::env::var_os("HOME") {
            dirs.push(PathBuf::from(home).join("Applications"));
        }
        return dirs.iter().any(|d| d.join(format!("{name}.app")).exists());
    }
    #[cfg(target_os = "linux")]
    {
        return entry(name).map(|(_, cmd, _)| which(cmd)).unwrap_or(false);
    }
    #[cfg(target_os = "windows")]
    {
        let Some((_, cmd, folder)) = entry(name) else { return false };
        if which(&format!("{cmd}.exe")) || which(&format!("{cmd}.cmd")) {
            return true;
        }
        if folder.is_empty() {
            return false;
        }
        let mut roots: Vec<PathBuf> = ["C:\\Program Files", "C:\\Program Files (x86)"].iter().map(PathBuf::from).collect();
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            roots.push(PathBuf::from(&local));
            roots.push(PathBuf::from(local).join("Programs"));
        }
        return roots.iter().any(|r| r.join(folder).exists());
    }
    #[allow(unreachable_code)]
    false
}

/// Open `target` (a path or URL) with the GUI app `app`. "Finder" means the
/// platform file manager.
pub fn open_with(app: &str, target: &str) -> Result<(), String> {
    let app = app.trim();
    #[cfg(target_os = "macos")]
    {
        let out = Command::new("open").arg("-a").arg(app).arg(target).output().map_err(|e| e.to_string())?;
        return if out.status.success() { Ok(()) } else { Err(format!("could not open with {app}: {}", String::from_utf8_lossy(&out.stderr).trim())) };
    }
    #[cfg(target_os = "linux")]
    {
        let cmd = if app.eq_ignore_ascii_case("finder") || app.is_empty() { "xdg-open".to_string() } else { entry(app).map(|(_, c, _)| c.to_string()).filter(|c| !c.is_empty()).unwrap_or_else(|| app.to_lowercase().replace(' ', "-")) };
        return Command::new(&cmd).arg(target).spawn().map(|_| ()).map_err(|e| format!("could not run {cmd}: {e}"));
    }
    #[cfg(target_os = "windows")]
    {
        if app.eq_ignore_ascii_case("finder") || app.is_empty() {
            return Command::new("cmd").args(["/C", "start", "", target]).spawn().map(|_| ()).map_err(|e| e.to_string());
        }
        let cmd = entry(app).map(|(_, c, _)| c.to_string()).filter(|c| !c.is_empty()).unwrap_or_else(|| app.to_string());
        return Command::new("cmd").args(["/C", "start", "", &cmd, target]).spawn().map(|_| ()).map_err(|e| e.to_string());
    }
    #[allow(unreachable_code)]
    Err("unsupported platform".into())
}

/// Candidate docker binaries, most specific first; the bare name is the fallback.
pub fn docker_candidates() -> &'static [&'static str] {
    #[cfg(target_os = "macos")]
    {
        return &["/usr/local/bin/docker", "/opt/homebrew/bin/docker", "/Applications/Docker.app/Contents/Resources/bin/docker"];
    }
    #[cfg(target_os = "linux")]
    {
        return &["/usr/bin/docker", "/usr/local/bin/docker", "/snap/bin/docker"];
    }
    #[cfg(target_os = "windows")]
    {
        return &["C:\\Program Files\\Docker\\Docker\\resources\\bin\\docker.exe"];
    }
    #[allow(unreachable_code)]
    &[]
}

/// The file manager's display name for buttons.
pub fn file_manager_name() -> &'static str {
    if cfg!(target_os = "macos") { "Finder" } else if cfg!(target_os = "windows") { "Explorer" } else { "Files" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_apps_resolve_case_insensitively() {
        assert!(entry("phpstorm").is_some());
        assert!(entry("nothing-here").is_none());
        assert!(!docker_candidates().is_empty());
        let _ = installed(&["PhpStorm", "Visual Studio Code"]);
    }
}
