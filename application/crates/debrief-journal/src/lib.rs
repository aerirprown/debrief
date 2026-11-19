//! Debrief gaming journal — append-only folder of sessions and freeform note revisions.
//!
//! Layout (see Design tab):
//! ```text
//! journal/
//!   sessions/{sessionId}_{device}.json
//!   notes/{noteId}/{revision}_{device}.json
//!   games/{gameId}/{revision}_{device}.json
//!   frames/  media/  audio/
//! ```
//!
//! Each save is a new file. An edit names its parent revision. Two edits of the
//! same parent are a fork. No database file; nothing is rewritten in place.

mod audio;
mod device;
mod error;
mod game;
mod hotkey;
mod listing;
mod memoir;
mod note;
mod notify;
mod paths;
mod remind;
mod return_card;
mod session;
mod shelf;
mod steam;
mod vdf;

pub use audio::{resolve_audio_path, write_audio};
pub use device::{create_device_id, load_or_create_device_id};
pub use error::{Error, Result};
pub use game::{read_games, write_game_revision, GameRevision, NewGame};
pub use hotkey::append_hotkey_line;
pub use listing::{
    is_local_only_name, list_journal_files, suggest_device_id_beside_settings, SYNCED_TOP,
};
pub use memoir::{read_memoir, Fork, Memoir, NoteLineage};
pub use note::{
    write_note_revision, Container, NewNoteRevision, NoteRevision, Reminder, FREEFORM_TYPE,
};
pub use notify::{due_notifications, notification_plan, NotifyKind, PendingNotification};
pub use paths::{audio_ref, media_ref, AUDIO, FRAMES, GAMES, MEDIA, NOTES, SESSIONS};
pub use remind::{add_reminder, clear_reminder};
pub use return_card::{days_since_stopped, newest_session, return_card_for, ReturnCard};
pub use session::{write_session, NewSession, Session};
pub use shelf::{build_shelf, LaunchTarget, ShelfRow, ShelfSource};
pub use steam::{
    default_steam_roots, read_installed_games, steam_game_id, steam_launch_uri, SteamGame,
};

use std::path::{Path, PathBuf};

/// Open a journal root with a fixed device id for writes from this install.
#[derive(Debug, Clone)]
pub struct Journal {
    root: PathBuf,
    device_id: String,
}

impl Journal {
    pub fn open(root: impl Into<PathBuf>, device_id: impl Into<String>) -> Result<Self> {
        let root = root.into();
        paths::ensure_layout(&root)?;
        Ok(Self {
            root,
            device_id: device_id.into(),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub fn write_session(&self, input: NewSession) -> Result<Session> {
        write_session(&self.root, &self.device_id, input)
    }

    pub fn write_note_revision(&self, input: NewNoteRevision) -> Result<NoteRevision> {
        write_note_revision(&self.root, &self.device_id, input)
    }

    pub fn write_game_revision(&self, input: NewGame) -> Result<GameRevision> {
        write_game_revision(&self.root, &self.device_id, input)
    }

    pub fn read_memoir(&self) -> Result<Memoir> {
        read_memoir(&self.root)
    }

    pub fn read_games(&self) -> Result<Vec<GameRevision>> {
        read_games(&self.root)
    }

    pub fn return_card(&self, game_id: &str, as_of_day: &str) -> Result<Option<ReturnCard>> {
        let games = read_games(&self.root)?;
        let Some(game) = games.into_iter().find(|g| g.id == game_id) else {
            return Ok(None);
        };
        let memoir = read_memoir(&self.root)?;
        Ok(Some(return_card_for(&game, &memoir.sessions, as_of_day)))
    }

    pub fn shelf(&self, steam_root: Option<&Path>, as_of_day: &str) -> Result<Vec<ShelfRow>> {
        build_shelf(&self.root, steam_root, as_of_day)
    }

    pub fn write_audio(&self, bytes: &[u8], extension: &str) -> Result<String> {
        write_audio(&self.root, bytes, extension)
    }

    pub fn append_hotkey_line(&self, game_id: &str, line: &str) -> Result<NoteRevision> {
        append_hotkey_line(&self.root, &self.device_id, game_id, line)
    }

    pub fn add_reminder(
        &self,
        note_id: &str,
        at: impl Into<String>,
        label: Option<String>,
        container_id: Option<String>,
    ) -> Result<NoteRevision> {
        add_reminder(
            &self.root,
            &self.device_id,
            note_id,
            at,
            label,
            container_id,
        )
    }

    pub fn clear_reminder(&self, note_id: &str, reminder_id: &str) -> Result<NoteRevision> {
        clear_reminder(&self.root, &self.device_id, note_id, reminder_id)
    }

    pub fn notification_plan(&self, now_rfc3339: &str) -> Result<Vec<PendingNotification>> {
        notification_plan(&self.root, now_rfc3339)
    }

    pub fn due_notifications(&self, now_rfc3339: &str) -> Result<Vec<PendingNotification>> {
        due_notifications(&self.root, now_rfc3339)
    }

    pub fn list_files(&self) -> Result<Vec<String>> {
        list_journal_files(&self.root)
    }
}
