//! Directory listing of a journal folder (what a sync client would copy).

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{io, Result};
use crate::paths::{AUDIO, FRAMES, GAMES, MEDIA, NOTES, SESSIONS};

/// Top-level trees that belong in a synced journal (not device-local files).
pub const SYNCED_TOP: &[&str] = &[SESSIONS, NOTES, GAMES, FRAMES, MEDIA, AUDIO];

/// Relative paths (forward slashes) of every file under the synced trees,
/// sorted. Empty dirs are omitted. Dotfiles at the journal root (for example
/// a leftover `.device_id`) are not listed — they must not live in a cloud
/// folder shared by two installs.
pub fn list_journal_files(root: &Path) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for top in SYNCED_TOP {
        let dir = root.join(top);
        if !dir.is_dir() {
            continue;
        }
        walk(&dir, root, &mut out)?;
    }
    out.sort();
    Ok(out)
}

fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(|e| io(dir, e))? {
        let entry = entry.map_err(|e| io(dir, e))?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, root, out)?;
        } else if path.is_file() {
            out.push(rel_forward(root, &path));
        }
    }
    Ok(())
}

fn rel_forward(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// True when `name` is a local-only file that must not be relied on inside a
/// synced journal (device id, watcher log).
pub fn is_local_only_name(name: &str) -> bool {
    matches!(
        name,
        ".device_id" | ".debrief-watcher.log" | "device_id"
    )
}

/// Absolute path helpers for tests / callers that place device id outside the journal.
pub fn suggest_device_id_beside_settings(settings_dir: impl AsRef<Path>) -> PathBuf {
    settings_dir.as_ref().join("device_id")
}
