//! On-disk layout for the gaming journal folder.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{io, Result};

pub const SESSIONS: &str = "sessions";
pub const NOTES: &str = "notes";
pub const GAMES: &str = "games";
pub const FRAMES: &str = "frames";
pub const MEDIA: &str = "media";
pub const AUDIO: &str = "audio";

/// Ensure the top-level journal directories exist.
pub fn ensure_layout(root: &Path) -> Result<()> {
    for name in [SESSIONS, NOTES, GAMES, FRAMES, MEDIA, AUDIO] {
        let dir = root.join(name);
        fs::create_dir_all(&dir).map_err(|e| io(&dir, e))?;
    }
    Ok(())
}

pub fn session_path(root: &Path, session_id: &str, device_id: &str) -> PathBuf {
    root.join(SESSIONS)
        .join(format!("{session_id}_{device_id}.json"))
}

pub fn note_dir(root: &Path, note_id: &str) -> PathBuf {
    root.join(NOTES).join(note_id)
}

pub fn note_revision_path(root: &Path, note_id: &str, revision_id: &str, device_id: &str) -> PathBuf {
    note_dir(root, note_id).join(format!("{revision_id}_{device_id}.json"))
}

pub fn media_ref(media_id: &str) -> String {
    format!("media/{media_id}")
}

pub fn audio_ref(audio_id: &str) -> String {
    format!("audio/{audio_id}")
}

/// Absolute path for an audio file under `audio/`. `file_name` is the basename
/// (id + extension), e.g. `memo-1.ogg`.
pub fn audio_path(root: &Path, file_name: &str) -> PathBuf {
    root.join(AUDIO).join(file_name)
}
