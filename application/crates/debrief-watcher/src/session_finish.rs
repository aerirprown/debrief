//! Turn a quit-prompt answer into a journal session file.

use std::path::Path;

use debrief_journal::{write_session, NewSession, Session};

use crate::engine::write_frame;
use crate::error::Result;
use crate::frame::FrameBytes;

#[derive(Debug, Clone)]
pub struct PendingSession {
    pub game_id: String,
    pub started_at: String,
    pub stopped_at: String,
    pub frame: Option<FrameBytes>,
}

/// Write the session. Empty `text` / `what_next` is a dismiss.
pub fn finish_session(
    journal_root: &Path,
    device_id: &str,
    pending: &PendingSession,
    text: impl Into<String>,
    what_next: impl Into<String>,
) -> Result<Session> {
    let frame_id = match &pending.frame {
        Some(png) if !png.is_empty() => Some(write_frame(journal_root, png)?),
        _ => None,
    };
    Ok(write_session(
        journal_root,
        device_id,
        NewSession {
            game_id: pending.game_id.clone(),
            started_at: pending.started_at.clone(),
            stopped_at: pending.stopped_at.clone(),
            text: text.into(),
            what_next: what_next.into(),
            frame_id,
        },
    )?)
}
