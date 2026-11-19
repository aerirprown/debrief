//! Game note pages: freeform containers + reminders, one revision per file.

use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;
use crate::paths;
use crate::session::write_new_json;

pub const FREEFORM_TYPE: &str = "debrief-freeform";

/// Container kinds on a game note page (mirrors the web freeform model + audio).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Container {
    Text {
        id: String,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        html: String,
    },
    Image {
        id: String,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        /// `media/{id}` reference; bytes live under `media/`.
        src: String,
    },
    Table {
        id: String,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        rows: Vec<Vec<String>>,
    },
    Audio {
        id: String,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        /// `audio/{id}` reference.
        src: String,
    },
}

/// Local reminder on a note revision (date-and-time, not "come back on").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reminder {
    pub id: String,
    /// RFC 3339 UTC.
    pub at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,
    #[serde(default)]
    pub cleared: bool,
}

/// One append-only revision of a game note page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteRevision {
    #[serde(rename = "type")]
    pub kind: String,
    pub version: u32,
    pub note_id: String,
    pub game_id: String,
    pub revision_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_revision_id: Option<String>,
    pub device_id: String,
    pub containers: Vec<Container>,
    #[serde(default)]
    pub reminders: Vec<Reminder>,
}

/// Input for writing a new note revision.
#[derive(Debug, Clone)]
pub struct NewNoteRevision {
    pub note_id: String,
    pub game_id: String,
    /// `None` for the first revision of a note; otherwise the revision this edit replaces.
    pub parent_revision_id: Option<String>,
    pub containers: Vec<Container>,
    pub reminders: Vec<Reminder>,
}

/// Write a new note revision file. Never rewrites an older revision.
pub fn write_note_revision(
    root: &Path,
    device_id: &str,
    input: NewNoteRevision,
) -> Result<NoteRevision> {
    paths::ensure_layout(root)?;
    let revision = NoteRevision {
        kind: FREEFORM_TYPE.to_string(),
        version: 1,
        note_id: input.note_id.clone(),
        game_id: input.game_id,
        revision_id: Uuid::new_v4().to_string(),
        parent_revision_id: input.parent_revision_id,
        device_id: device_id.to_string(),
        containers: input.containers,
        reminders: input.reminders,
    };
    let path = paths::note_revision_path(
        root,
        &revision.note_id,
        &revision.revision_id,
        device_id,
    );
    write_new_json(&path, &revision)?;
    Ok(revision)
}

impl Container {
    pub fn text(id: impl Into<String>, x: f64, y: f64, w: f64, h: f64, html: impl Into<String>) -> Self {
        Self::Text {
            id: id.into(),
            x,
            y,
            w,
            h,
            html: html.into(),
        }
    }

    pub fn image(
        id: impl Into<String>,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        media_id: impl AsRef<str>,
    ) -> Self {
        Self::Image {
            id: id.into(),
            x,
            y,
            w,
            h,
            src: paths::media_ref(media_id.as_ref()),
        }
    }

    pub fn audio(
        id: impl Into<String>,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        audio_file: impl AsRef<str>,
    ) -> Self {
        Self::Audio {
            id: id.into(),
            x,
            y,
            w,
            h,
            src: paths::audio_ref(audio_file.as_ref()),
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Text { id, .. }
            | Self::Image { id, .. }
            | Self::Table { id, .. }
            | Self::Audio { id, .. } => id,
        }
    }

    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        match self {
            Self::Text { x, y, w, h, .. }
            | Self::Image { x, y, w, h, .. }
            | Self::Table { x, y, w, h, .. }
            | Self::Audio { x, y, w, h, .. } => (*x, *y, *w, *h),
        }
    }
}
