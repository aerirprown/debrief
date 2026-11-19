//! Read the journal folder into a memoir: sessions in time order, note lineages, forks.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::error::{io, Result};
use crate::note::NoteRevision;
use crate::paths;
use crate::session::{read_json_file, Session};

/// A fork: two or more revisions that name the same parent, neither descended from the other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fork {
    pub note_id: String,
    pub parent_revision_id: Option<String>,
    pub child_revision_ids: Vec<String>,
}

/// All revisions for one logical note, plus detected forks.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteLineage {
    pub note_id: String,
    pub game_id: String,
    pub revisions: Vec<NoteRevision>,
    pub forks: Vec<Fork>,
    /// Tip of the unique linear chain when there is no open fork; otherwise `None`.
    pub newest_unforked: Option<NoteRevision>,
}

/// Derived read of the whole journal folder.
#[derive(Debug, Clone, PartialEq)]
pub struct Memoir {
    pub sessions: Vec<Session>,
    pub notes: Vec<NoteLineage>,
}

/// Load every session and note revision under `root`.
pub fn read_memoir(root: &Path) -> Result<Memoir> {
    let sessions = read_sessions(root)?;
    let notes = read_notes(root)?;
    Ok(Memoir { sessions, notes })
}

fn read_sessions(root: &Path) -> Result<Vec<Session>> {
    let dir = root.join(paths::SESSIONS);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut sessions = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| io(&dir, e))? {
        let entry = entry.map_err(|e| io(&dir, e))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        sessions.push(read_json_file::<Session>(&path)?);
    }
    sessions.sort_by(|a, b| {
        a.stopped_at
            .cmp(&b.stopped_at)
            .then_with(|| a.started_at.cmp(&b.started_at))
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(sessions)
}

fn read_notes(root: &Path) -> Result<Vec<NoteLineage>> {
    let notes_root = root.join(paths::NOTES);
    if !notes_root.exists() {
        return Ok(Vec::new());
    }
    let mut lineages = Vec::new();
    for entry in fs::read_dir(&notes_root).map_err(|e| io(&notes_root, e))? {
        let entry = entry.map_err(|e| io(&notes_root, e))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let mut revisions = Vec::new();
        for file in fs::read_dir(&path).map_err(|e| io(&path, e))? {
            let file = file.map_err(|e| io(&path, e))?;
            let file_path = file.path();
            if file_path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            revisions.push(read_json_file::<NoteRevision>(&file_path)?);
        }
        if revisions.is_empty() {
            continue;
        }
        lineages.push(build_lineage(revisions));
    }
    lineages.sort_by(|a, b| a.note_id.cmp(&b.note_id));
    Ok(lineages)
}

fn build_lineage(mut revisions: Vec<NoteRevision>) -> NoteLineage {
    revisions.sort_by(|a, b| a.revision_id.cmp(&b.revision_id));
    let note_id = revisions[0].note_id.clone();
    let game_id = revisions[0].game_id.clone();

    let mut children: HashMap<Option<String>, Vec<String>> = HashMap::new();
    for rev in &revisions {
        children
            .entry(rev.parent_revision_id.clone())
            .or_default()
            .push(rev.revision_id.clone());
    }

    let mut forks = Vec::new();
    for (parent, kids) in &children {
        if kids.len() < 2 {
            continue;
        }
        let mut child_revision_ids = kids.clone();
        child_revision_ids.sort();
        forks.push(Fork {
            note_id: note_id.clone(),
            parent_revision_id: parent.clone(),
            child_revision_ids,
        });
    }
    forks.sort_by(|a, b| {
        a.parent_revision_id
            .cmp(&b.parent_revision_id)
            .then_with(|| a.child_revision_ids.cmp(&b.child_revision_ids))
    });

    // A tip is a revision that no other revision names as its parent.
    let mut is_named_as_parent: HashSet<&str> = HashSet::new();
    for rev in &revisions {
        if let Some(parent) = &rev.parent_revision_id {
            is_named_as_parent.insert(parent.as_str());
        }
    }
    let tips: Vec<&NoteRevision> = revisions
        .iter()
        .filter(|r| !is_named_as_parent.contains(r.revision_id.as_str()))
        .collect();

    let newest_unforked = if forks.is_empty() && tips.len() == 1 {
        Some(tips[0].clone())
    } else {
        None
    };

    NoteLineage {
        note_id,
        game_id,
        revisions,
        forks,
        newest_unforked,
    }
}
