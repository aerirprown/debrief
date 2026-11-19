//! Session files: one play period, plain one-line fields (not freeform).

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{io, json, Error, Result};
use crate::paths;

/// A finished (or dismissed) play session written to the journal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub device_id: String,
    pub game_id: String,
    /// RFC 3339 UTC start.
    pub started_at: String,
    /// RFC 3339 UTC stop.
    pub stopped_at: String,
    /// One-line "what happened". Empty when the prompt was dismissed.
    #[serde(default)]
    pub text: String,
    /// One-line "what next".
    #[serde(default)]
    pub what_next: String,
    /// Optional `frames/{id}` basename (no path prefix in the field).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<String>,
}

/// Fields the caller supplies when ending a session.
#[derive(Debug, Clone)]
pub struct NewSession {
    pub game_id: String,
    pub started_at: String,
    pub stopped_at: String,
    pub text: String,
    pub what_next: String,
    pub frame_id: Option<String>,
}

/// Write a new session file. Never overwrites an existing path.
pub fn write_session(root: &Path, device_id: &str, input: NewSession) -> Result<Session> {
    paths::ensure_layout(root)?;
    let session = Session {
        id: Uuid::new_v4().to_string(),
        device_id: device_id.to_string(),
        game_id: input.game_id,
        started_at: input.started_at,
        stopped_at: input.stopped_at,
        text: input.text,
        what_next: input.what_next,
        frame_id: input.frame_id,
    };
    let path = paths::session_path(root, &session.id, device_id);
    write_new_json(&path, &session)?;
    Ok(session)
}

pub(crate) fn write_new_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if path.exists() {
        return Err(Error::AlreadyExists {
            path: path.to_path_buf(),
        });
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| io(parent, e))?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| json(path, e))?;
    fs::write(path, bytes).map_err(|e| io(path, e))?;
    Ok(())
}

pub(crate) fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).map_err(|e| io(path, e))?;
    serde_json::from_slice(&bytes).map_err(|e| json(path, e))
}
