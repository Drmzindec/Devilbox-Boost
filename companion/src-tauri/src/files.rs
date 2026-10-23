//! Project file explorer and uploader. Every browsed or written path must stay
//! inside <root>/data/www; sources for uploads can come from anywhere the user
//! picked or dropped.

use crate::{root_or_err, AppState};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::State;

#[derive(Serialize, Clone)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub modified: u64,
}

#[derive(Serialize, Clone)]
pub struct Listing {
    pub path: String,
    pub relative: String,
    pub parent: Option<String>,
    pub entries: Vec<Entry>,
}

fn www(root: &Path) -> PathBuf {
    root.join("data").join("www")
}

/// Resolve `requested` and make sure it stays under data/www.
fn safe_path(root: &Path, requested: &str) -> Result<PathBuf, String> {
    let base = www(root).canonicalize().map_err(|e| e.to_string())?;
    let candidate = if requested.trim().is_empty() { base.clone() } else { PathBuf::from(requested) };
    let candidate = if candidate.is_absolute() { candidate } else { base.join(candidate) };
    // The final component may not exist yet (new folder), so canonicalize the parent.
    let resolved = if candidate.exists() {
        candidate.canonicalize().map_err(|e| e.to_string())?
    } else {
        let parent = candidate.parent().ok_or("invalid path")?.canonicalize().map_err(|e| e.to_string())?;
        parent.join(candidate.file_name().ok_or("invalid path")?)
    };
    if !resolved.starts_with(&base) {
        return Err("path is outside data/www".into());
    }
    Ok(resolved)
}

fn copy_recursive(src: &Path, dst: &Path) -> Result<u64, String> {
    let meta = fs::symlink_metadata(src).map_err(|e| e.to_string())?;
    if meta.is_dir() {
        fs::create_dir_all(dst).map_err(|e| e.to_string())?;
        let mut n = 0;
        for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            n += copy_recursive(&entry.path(), &dst.join(entry.file_name()))?;
        }
        Ok(n)
    } else {
        if let Some(p) = dst.parent() {
            fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        fs::copy(src, dst).map_err(|e| format!("{}: {e}", src.display()))?;
        Ok(1)
    }
}

fn is_archive(name: &str) -> Option<&'static str> {
    let n = name.to_lowercase();
    if n.ends_with(".zip") {
        Some("zip")
    } else if n.ends_with(".tar.gz") || n.ends_with(".tgz") {
        Some("tgz")
    } else if n.ends_with(".tar") {
        Some("tar")
    } else {
        None
    }
}

fn extract(src: &Path, dest: &Path, kind: &str) -> Result<(), String> {
    let out = match kind {
        "zip" => Command::new("unzip").args(["-o", "-q"]).arg(src).arg("-d").arg(dest).output(),
        "tgz" => Command::new("tar").args(["-xzf"]).arg(src).arg("-C").arg(dest).output(),
        _ => Command::new("tar").args(["-xf"]).arg(src).arg("-C").arg(dest).output(),
    }
    .map_err(|e| e.to_string())?;
    if out.status.success() {
        // macOS zips often carry a __MACOSX folder nobody wants.
        let _ = fs::remove_dir_all(dest.join("__MACOSX"));
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

#[tauri::command]
pub fn list_dir(state: State<AppState>, path: String) -> Result<Listing, String> {
    let root = root_or_err(&state)?;
    let base = www(&root).canonicalize().map_err(|e| e.to_string())?;
    let dir = safe_path(&root, &path)?;
    if !dir.is_dir() {
        return Err(format!("{} is not a folder", dir.display()));
    }
    let mut entries: Vec<Entry> = fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            let link = fs::symlink_metadata(e.path()).map(|m| m.file_type().is_symlink()).unwrap_or(false);
            Some(Entry {
                name: e.file_name().to_string_lossy().to_string(),
                path: e.path().to_string_lossy().to_string(),
                is_dir: meta.is_dir(),
                is_symlink: link,
                size: if meta.is_dir() { 0 } else { meta.len() },
                modified: meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0),
            })
        })
        .collect();
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    let relative = dir.strip_prefix(&base).map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    let parent = if dir == base { None } else { dir.parent().map(|p| p.to_string_lossy().to_string()) };
    Ok(Listing { path: dir.to_string_lossy().to_string(), relative, parent, entries })
}

#[tauri::command]
pub fn make_dir(state: State<AppState>, path: String, name: String) -> Result<String, String> {
    let root = root_or_err(&state)?;
    if name.trim().is_empty() || name.contains('/') || name.starts_with('.') {
        return Err("Folder name must not be empty, contain / or start with a dot".into());
    }
    let dir = safe_path(&root, &path)?.join(name.trim());
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.to_string_lossy().to_string())
}

#[derive(Serialize, Clone)]
pub struct PlanItem {
    pub source: String,
    pub name: String,
    /// "copy" (new entry), "merge" (into an existing folder), "extract" (archive into dest),
    /// "extract-merge" (archive's single top folder matches dest), "replace" (file exists)
    pub action: String,
    pub target: String,
    pub note: String,
}

fn top_level_dir_of_archive(src: &Path, kind: &str) -> Option<String> {
    let out = match kind {
        "zip" => Command::new("unzip").arg("-Z1").arg(src).output().ok()?,
        _ => Command::new("tar").arg("-tf").arg(src).output().ok()?,
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let mut tops: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for line in text.lines() {
        let l = line.trim_start_matches("./");
        if l.is_empty() || l.starts_with("__MACOSX") || l.ends_with(".DS_Store") {
            continue;
        }
        tops.insert(l.split('/').next().unwrap_or("").to_string());
        if tops.len() > 1 {
            return None;
        }
    }
    tops.into_iter().next().filter(|t| !t.is_empty())
}

/// Decide where each dropped item lands. A folder (or archive) whose name equals
/// the destination folder merges into it instead of nesting (dropping
/// `wp-content` onto `wp-content`), and a folder dropped where a same-named
/// folder exists merges too.
fn plan_items(dest: &Path, sources: &[String], extract_archives: bool) -> Vec<PlanItem> {
    let dest_name = dest.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    sources
        .iter()
        .filter_map(|s| {
            let src = PathBuf::from(s);
            let name = src.file_name()?.to_string_lossy().to_string();
            if !src.exists() {
                return Some(PlanItem { source: s.clone(), name: name.clone(), action: "skip".into(), target: String::new(), note: "not found".into() });
            }
            if let (true, Some(kind)) = (extract_archives && src.is_file(), is_archive(&name)) {
                return Some(match top_level_dir_of_archive(&src, kind) {
                    Some(top) if top == dest_name => PlanItem { source: s.clone(), name, action: "extract-merge".into(), target: dest.to_string_lossy().to_string(), note: format!("archive contains a single '{top}' folder: its contents go straight into this folder") },
                    Some(top) => PlanItem { source: s.clone(), name, action: "extract".into(), target: dest.join(&top).to_string_lossy().to_string(), note: format!("extracts as '{top}/'{}", if dest.join(&top).exists() { ", merging with the existing folder" } else { "" }) },
                    None => PlanItem { source: s.clone(), name, action: "extract".into(), target: dest.to_string_lossy().to_string(), note: "extracts its files into this folder".into() },
                });
            }
            if src.is_dir() {
                if name == dest_name {
                    return Some(PlanItem { source: s.clone(), name: name.clone(), action: "merge".into(), target: dest.to_string_lossy().to_string(), note: format!("'{name}' dropped onto '{dest_name}': contents merge into this folder") });
                }
                let target = dest.join(&name);
                let exists = target.exists();
                return Some(PlanItem { source: s.clone(), name, action: if exists { "merge".into() } else { "copy".into() }, target: target.to_string_lossy().to_string(), note: if exists { "merges into the existing folder (existing files are overwritten)".into() } else { String::new() } });
            }
            let target = dest.join(&name);
            let exists = target.exists();
            Some(PlanItem { source: s.clone(), name, action: if exists { "replace".into() } else { "copy".into() }, target: target.to_string_lossy().to_string(), note: if exists { "replaces the existing file".into() } else { String::new() } })
        })
        .collect()
}

#[tauri::command]
pub fn upload_plan(state: State<AppState>, dest: String, sources: Vec<String>, extract_archives: bool) -> Result<Vec<PlanItem>, String> {
    let root = root_or_err(&state)?;
    let dest = safe_path(&root, &dest)?;
    Ok(plan_items(&dest, &sources, extract_archives))
}

fn move_children(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(from).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let target = to.join(entry.file_name());
        if entry.path().is_dir() && target.is_dir() {
            move_children(&entry.path(), &target)?;
            let _ = fs::remove_dir(entry.path());
        } else {
            if target.exists() {
                let _ = if target.is_dir() { fs::remove_dir_all(&target) } else { fs::remove_file(&target) };
            }
            fs::rename(entry.path(), &target).map_err(|e| format!("{}: {e}", entry.path().display()))?;
        }
    }
    Ok(())
}

#[derive(Serialize, Clone)]
pub struct UploadReport {
    pub copied: u64,
    pub extracted: Vec<String>,
    pub errors: Vec<String>,
}

/// Copy the given files/folders into `dest`. Archives are extracted in place
/// when `extract_archives` is set, otherwise copied as files.
#[tauri::command]
pub async fn upload_files(state: State<'_, AppState>, dest: String, sources: Vec<String>, extract_archives: bool) -> Result<UploadReport, String> {
    let root = root_or_err(&state)?;
    let dest = safe_path(&root, &dest)?;
    if !dest.is_dir() {
        return Err(format!("{} is not a folder", dest.display()));
    }
    tauri::async_runtime::spawn_blocking(move || {
        let mut report = UploadReport { copied: 0, extracted: Vec::new(), errors: Vec::new() };
        for item in plan_items(&dest, &sources, extract_archives) {
            let src = PathBuf::from(&item.source);
            match item.action.as_str() {
                "skip" => report.errors.push(format!("{}: {}", item.name, item.note)),
                "extract" | "extract-merge" => {
                    let kind = is_archive(&item.name).unwrap_or("zip");
                    let tmp = dest.join(format!(".dbx-extract-{}", std::process::id()));
                    let _ = fs::remove_dir_all(&tmp);
                    let r = fs::create_dir_all(&tmp).map_err(|e| e.to_string()).and_then(|_| extract(&src, &tmp, kind)).and_then(|_| {
                        if item.action == "extract-merge" {
                            // single top-level folder named like dest: lift its contents
                            let top = fs::read_dir(&tmp).map_err(|e| e.to_string())?.filter_map(|e| e.ok()).map(|e| e.path()).find(|p| p.is_dir());
                            match top {
                                Some(t) => move_children(&t, &dest),
                                None => move_children(&tmp, &dest),
                            }
                        } else {
                            move_children(&tmp, &dest)
                        }
                    });
                    let _ = fs::remove_dir_all(&tmp);
                    match r {
                        Ok(()) => report.extracted.push(item.name.clone()),
                        Err(e) => report.errors.push(format!("{}: {e}", item.name)),
                    }
                }
                "merge" if PathBuf::from(&item.target) == dest => match copy_recursive(&src, &dest) {
                    Ok(n) => report.copied += n,
                    Err(e) => report.errors.push(e),
                },
                _ => match copy_recursive(&src, &PathBuf::from(&item.target)) {
                    Ok(n) => report.copied += n,
                    Err(e) => report.errors.push(e),
                },
            }
        }
        Ok(report)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pick_files(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app.dialog().file().set_title("Choose files to copy into the project").blocking_pick_files();
    Ok(picked
        .unwrap_or_default()
        .into_iter()
        .filter_map(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planner_merges_same_named_folders() {
        let root = std::env::temp_dir().join(format!("dbx-plan-{}", std::process::id()));
        let dest = root.join("site/wp-content");
        fs::create_dir_all(dest.join("plugins")).unwrap();
        let drop = root.join("drop/wp-content/uploads");
        fs::create_dir_all(&drop).unwrap();
        fs::write(root.join("drop/readme.txt"), "x").unwrap();
        let plan = plan_items(&dest, &[root.join("drop/wp-content").to_string_lossy().to_string(), root.join("drop/readme.txt").to_string_lossy().to_string(), root.join("drop/plugins").to_string_lossy().to_string()], true);
        assert_eq!(plan[0].action, "merge");
        assert_eq!(plan[0].target, dest.to_string_lossy());
        assert_eq!(plan[1].action, "copy");
        assert_eq!(plan[2].action, "skip");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn safe_path_rejects_escapes() {
        let root = std::env::temp_dir().join(format!("dbx-files-{}", std::process::id()));
        fs::create_dir_all(root.join("data/www/site/wp-content")).unwrap();
        let ok = safe_path(&root, &root.join("data/www/site").to_string_lossy()).unwrap();
        assert!(ok.ends_with("site"));
        assert!(safe_path(&root, "../..").is_err());
        assert!(safe_path(&root, &root.join("data").to_string_lossy()).is_err());
        assert_eq!(is_archive("Uploads.ZIP"), Some("zip"));
        assert_eq!(is_archive("a.tar.gz"), Some("tgz"));
        assert_eq!(is_archive("a.sql"), None);
        let _ = fs::remove_dir_all(&root);
    }
}
