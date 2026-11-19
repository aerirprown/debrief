//! Add or clear note reminders as new revisions (never rewrite in place).

use std::path::Path;

use uuid::Uuid;

use crate::error::{Error, Result};
use crate::memoir::read_memoir;
use crate::note::{write_note_revision, NewNoteRevision, NoteRevision, Reminder};

/// Add a reminder onto the tip revision for `note_id` (new file, parent-linked).
pub fn add_reminder(
    root: &Path,
    device_id: &str,
    note_id: &str,
    at: impl Into<String>,
    label: Option<String>,
    container_id: Option<String>,
) -> Result<NoteRevision> {
    let tip = tip_for_note(root, note_id)?;
    let mut reminders = tip.reminders.clone();
    reminders.push(Reminder {
        id: Uuid::new_v4().to_string(),
        at: at.into(),
        label,
        container_id,
        cleared: false,
    });
    write_note_revision(
        root,
        device_id,
        NewNoteRevision {
            note_id: tip.note_id.clone(),
            game_id: tip.game_id.clone(),
            parent_revision_id: Some(tip.revision_id.clone()),
            containers: tip.containers.clone(),
            reminders,
        },
    )
}

/// Clear a reminder by writing a new revision with that reminder marked cleared.
pub fn clear_reminder(
    root: &Path,
    device_id: &str,
    note_id: &str,
    reminder_id: &str,
) -> Result<NoteRevision> {
    let tip = tip_for_note(root, note_id)?;
    let mut found = false;
    let reminders: Vec<_> = tip
        .reminders
        .iter()
        .map(|r| {
            if r.id == reminder_id {
                found = true;
                Reminder {
                    cleared: true,
                    ..r.clone()
                }
            } else {
                r.clone()
            }
        })
        .collect();
    if !found {
        return Err(Error::NotFound {
            what: format!("reminder {reminder_id} on note {note_id}"),
        });
    }
    write_note_revision(
        root,
        device_id,
        NewNoteRevision {
            note_id: tip.note_id.clone(),
            game_id: tip.game_id.clone(),
            parent_revision_id: Some(tip.revision_id.clone()),
            containers: tip.containers.clone(),
            reminders,
        },
    )
}

fn tip_for_note(root: &Path, note_id: &str) -> Result<NoteRevision> {
    let memoir = read_memoir(root)?;
    let lineage = memoir
        .notes
        .into_iter()
        .find(|n| n.note_id == note_id)
        .ok_or_else(|| Error::NotFound {
            what: format!("note {note_id}"),
        })?;
    lineage
        .newest_unforked
        .ok_or_else(|| Error::OpenFork {
            note_id: note_id.to_string(),
        })
}
