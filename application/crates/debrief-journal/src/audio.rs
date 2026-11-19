//! Voice memo bytes under `audio/`. Revisions only store `audio/{id}` refs.

use std::fs;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::error::{io, Result};
use crate::paths;

/// Write audio bytes as a new file. Never overwrites. Returns the basename
/// (id + extension) used in `audio/{basename}` refs.
pub fn write_audio(root: &Path, bytes: &[u8], extension: &str) -> Result<String> {
    paths::ensure_layout(root)?;
    let ext = extension.trim_start_matches('.').trim();
    let ext = if ext.is_empty() { "ogg" } else { ext };
    let id = Uuid::new_v4().to_string();
    let file_name = format!("{id}.{ext}");
    let path = paths::audio_path(root, &file_name);
    if path.exists() {
        return Err(crate::error::Error::AlreadyExists { path });
    }
    fs::write(&path, bytes).map_err(|e| io(&path, e))?;
    Ok(file_name)
}

/// Resolve an `audio/{file}` ref to an absolute path under the journal root.
pub fn resolve_audio_path(root: &Path, src: &str) -> Option<PathBuf> {
    let name = src.strip_prefix("audio/").unwrap_or(src);
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
        return None;
    }
    Some(paths::audio_path(root, name))
}
