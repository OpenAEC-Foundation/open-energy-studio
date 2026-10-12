//! Project files on disk: an atomic save and a recent list kept on the Rust
//! side.
//!
//! The dialog plugin adds a picked file to the fs scope for the running
//! session only, so a path from an earlier session is out of scope after a
//! restart. The recent list here is that memory: a file that was opened or
//! saved through a dialog may be read and written again later, and no other
//! path is. The fs scope itself is not widened.
//!
//! A save writes a temporary file next to the target, flushes it, keeps the
//! previous version as `<name>.bak` and renames the temporary file over the
//! target, so a crash or a full disk leaves either the old or the new file
//! and never a truncated one.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Name of the recent list in the app data directory.
pub const RECENT_FILE: &str = "recent-project-files.json";
/// Files kept on the recent list.
pub const RECENT_LIMIT: usize = 30;

/// Error code and message, as the other commands return them.
pub fn error(code: &str, message: impl std::fmt::Display) -> String {
    format!("{code}: {message}")
}

/// A project is saved only as a `.json` file (`.oes.json` included).
pub fn check_project_path(path: &Path) -> Result<(), String> {
    let json = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"));
    if !json {
        return Err(error(
            "project_path_not_json",
            format!("{} is not a .json file", path.display()),
        ));
    }
    if path.file_name().is_none() || path.parent().is_none() {
        return Err(error("project_path_invalid", path.display()));
    }
    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    path.with_file_name(format!(".{name}.tmp-{}-{stamp}", std::process::id()))
}

/// The backup of the previous version: `<file name>.bak` next to it.
pub fn backup_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    path.with_file_name(format!("{name}.bak"))
}

/// Writes `text` to `path` atomically and keeps the previous version as a
/// backup. On any error the target is untouched and the temporary file is
/// removed.
pub fn atomic_write(path: &Path, text: &str) -> io::Result<()> {
    let temporary = temporary_path(path);
    let written = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()
    })();
    if let Err(failure) = written {
        let _ = fs::remove_file(&temporary);
        return Err(failure);
    }
    if path.exists() {
        // The previous version stays available; a failed backup does not
        // stop the save, the rename below is what must succeed.
        let _ = fs::copy(path, backup_path(path));
    }
    if let Err(failure) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(failure);
    }
    sync_directory(path);
    Ok(())
}

#[cfg(unix)]
fn sync_directory(path: &Path) {
    if let Some(directory) = path.parent() {
        if let Ok(handle) = File::open(directory) {
            let _ = handle.sync_all();
        }
    }
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) {}

/// The canonical form used on the recent list: absolute, with `..` resolved
/// where the file exists.
fn key(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Files opened or saved through a dialog, newest first.
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RecentFiles {
    pub paths: Vec<PathBuf>,
}

impl RecentFiles {
    pub fn load(directory: &Path) -> Self {
        fs::read_to_string(directory.join(RECENT_FILE))
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn store(&self, directory: &Path) -> io::Result<()> {
        fs::create_dir_all(directory)?;
        let text = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        atomic_write_any(&directory.join(RECENT_FILE), &text)
    }

    pub fn contains(&self, path: &Path) -> bool {
        let wanted = key(path);
        self.paths
            .iter()
            .any(|known| known == &wanted || known == path)
    }

    pub fn remember(&mut self, path: &Path) {
        let wanted = key(path);
        self.paths.retain(|known| known != &wanted);
        self.paths.insert(0, wanted);
        self.paths.truncate(RECENT_LIMIT);
    }
}

/// An atomic write without the `.json` and backup rules, for app data.
fn atomic_write_any(path: &Path, text: &str) -> io::Result<()> {
    let temporary = temporary_path(path);
    let written = (|| {
        let mut file = File::create(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()
    })();
    if let Err(failure) = written {
        let _ = fs::remove_file(&temporary);
        return Err(failure);
    }
    fs::rename(&temporary, path).inspect_err(|_| {
        let _ = fs::remove_file(&temporary);
    })
}

/// Whether a project file may be read or written: it was picked through a
/// dialog in this session (`in_scope`) or is on the recent list.
pub fn allowed(path: &Path, in_scope: bool, recent: &RecentFiles) -> Result<(), String> {
    if in_scope || recent.contains(path) {
        Ok(())
    } else {
        Err(error(
            "project_path_not_allowed",
            format!(
                "{} was not opened or saved through a dialog; open it with Open",
                path.display()
            ),
        ))
    }
}

/// Reads a project file, telling a missing file apart from other errors.
pub fn read_project(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|failure| {
        if failure.kind() == io::ErrorKind::NotFound {
            error("project_file_not_found", path.display())
        } else {
            error("project_file_unreadable", failure)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "oes-project-files-{name}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&directory).unwrap();
        directory
    }

    #[test]
    fn save_replaces_the_file_and_keeps_a_backup() {
        let directory = scratch("save");
        let path = directory.join("woning.oes.json");
        atomic_write(&path, "{\"v\":1}").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "{\"v\":1}");
        assert!(!backup_path(&path).exists(), "no backup for a new file");
        atomic_write(&path, "{\"v\":2}").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "{\"v\":2}");
        assert_eq!(fs::read_to_string(backup_path(&path)).unwrap(), "{\"v\":1}");
        let leftovers: Vec<_> = fs::read_dir(&directory)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp-"))
            .collect();
        assert!(leftovers.is_empty(), "temporary files left: {leftovers:?}");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn a_failed_save_leaves_the_old_file_intact() {
        let directory = scratch("fail");
        let path = directory.join("woning.oes.json");
        atomic_write(&path, "old").unwrap();
        // The temporary file cannot be created in a missing directory, the
        // same way a full disk or a read-only folder fails before the rename.
        let elsewhere = directory.join("missing").join("woning.oes.json");
        assert!(atomic_write(&elsewhere, "new").is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "old");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn only_json_files_are_project_files() {
        assert!(check_project_path(Path::new("/tmp/a.oes.json")).is_ok());
        assert!(check_project_path(Path::new("/tmp/A.JSON")).is_ok());
        let refused = check_project_path(Path::new("/etc/passwd")).unwrap_err();
        assert!(refused.starts_with("project_path_not_json: "), "{refused}");
        assert!(check_project_path(Path::new("/tmp/a.json.exe")).is_err());
    }

    #[test]
    fn a_path_outside_the_scope_and_the_recent_list_is_refused() {
        let directory = scratch("scope");
        let path = directory.join("woning.oes.json");
        let mut recent = RecentFiles::default();
        let refused = allowed(&path, false, &recent).unwrap_err();
        assert!(
            refused.starts_with("project_path_not_allowed: "),
            "{refused}"
        );
        assert!(allowed(&path, true, &recent).is_ok(), "picked in a dialog");
        recent.remember(&path);
        assert!(allowed(&path, false, &recent).is_ok(), "remembered");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn the_recent_list_survives_a_restart() {
        let directory = scratch("recent");
        let file = directory.join("a.oes.json");
        fs::write(&file, "{}").unwrap();
        let mut recent = RecentFiles::default();
        recent.remember(&file);
        recent.remember(&directory.join("b.oes.json"));
        recent.remember(&file);
        recent.store(&directory).unwrap();
        let loaded = RecentFiles::load(&directory);
        assert_eq!(loaded, recent);
        assert_eq!(loaded.paths.len(), 2, "no duplicates");
        assert!(loaded.contains(&file));
        // A path written with `..` names the same file.
        let detour = directory.join("x").join("..").join("a.oes.json");
        fs::create_dir_all(directory.join("x")).unwrap();
        assert!(loaded.contains(&detour));
        for index in 0..(RECENT_LIMIT + 5) {
            recent.remember(&directory.join(format!("{index}.json")));
        }
        assert_eq!(recent.paths.len(), RECENT_LIMIT);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn a_missing_file_is_told_apart() {
        let directory = scratch("missing");
        let missing = read_project(&directory.join("weg.oes.json")).unwrap_err();
        assert!(missing.starts_with("project_file_not_found: "), "{missing}");
        fs::remove_dir_all(directory).unwrap();
    }
}
