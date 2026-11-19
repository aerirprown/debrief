//! Mid-session hotkey: append a one-line text box onto the game's note page.

use std::path::Path;

use uuid::Uuid;

use crate::error::Result;
use crate::memoir::read_memoir;
use crate::note::{write_note_revision, Container, NewNoteRevision, NoteRevision};

const DEFAULT_X: f64 = 48.0;
const DEFAULT_W: f64 = 280.0;
const DEFAULT_H: f64 = 64.0;
const GAP: f64 = 16.0;

/// Append a plain one-line text box to the tip note for `game_id`.
/// Creates the first revision when the game has no note yet.
/// Names the previous tip as parent so fork rules still apply.
pub fn append_hotkey_line(
    root: &Path,
    device_id: &str,
    game_id: &str,
    line: &str,
) -> Result<NoteRevision> {
    let memoir = read_memoir(root)?;
    let tip = memoir
        .notes
        .iter()
        .filter(|n| n.game_id == game_id)
        .find_map(|n| n.newest_unforked.as_ref());

    let html = escape_line(line);
    let (note_id, parent_revision_id, mut containers, reminders) = match tip {
        Some(rev) => (
            rev.note_id.clone(),
            Some(rev.revision_id.clone()),
            rev.containers.clone(),
            rev.reminders.clone(),
        ),
        None => (
            Uuid::new_v4().to_string(),
            None,
            Vec::new(),
            Vec::new(),
        ),
    };

    let y = next_y(&containers);
    containers.push(Container::text(
        Uuid::new_v4().to_string(),
        DEFAULT_X,
        y,
        DEFAULT_W,
        DEFAULT_H,
        format!("<p>{html}</p>"),
    ));

    write_note_revision(
        root,
        device_id,
        NewNoteRevision {
            note_id,
            game_id: game_id.to_string(),
            parent_revision_id,
            containers,
            reminders,
        },
    )
}

fn next_y(containers: &[Container]) -> f64 {
    containers
        .iter()
        .map(|c| {
            let (_, y, _, h) = c.bounds();
            y + h
        })
        .fold(48.0_f64, f64::max)
        + if containers.is_empty() { 0.0 } else { GAP }
}

fn escape_line(line: &str) -> String {
    line.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\n' | '\r' => ' '.to_string(),
            other => other.to_string(),
        })
        .collect()
}
