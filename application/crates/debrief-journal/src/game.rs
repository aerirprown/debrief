//! Game records in `games/{gameId}/{revision}_{device}.json`.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{io, Result};
use crate::paths;
use crate::session::{read_json_file, write_new_json};

/// One revision of a game shelf/page record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameRevision {
    pub id: String,
    pub revision_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_revision_id: Option<String>,
    pub device_id: String,
    pub title: String,
    #[serde(default)]
    pub executables: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_path: Option<String>,
    /// "Come back on this day" — date only (YYYY-MM-DD).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub come_back_on: Option<String>,
    /// When this journal game is linked to a Steam app.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steam_app_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct NewGame {
    pub title: String,
    pub executables: Vec<String>,
    pub cover_path: Option<String>,
    pub come_back_on: Option<String>,
    pub parent_revision_id: Option<String>,
    /// When continuing an existing game, pass its id; otherwise a new id is minted.
    pub id: Option<String>,
    pub steam_app_id: Option<u32>,
}

pub fn write_game_revision(root: &Path, device_id: &str, input: NewGame) -> Result<GameRevision> {
    paths::ensure_layout(root)?;
    let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let revision = GameRevision {
        id: id.clone(),
        revision_id: Uuid::new_v4().to_string(),
        parent_revision_id: input.parent_revision_id,
        device_id: device_id.to_string(),
        title: input.title,
        executables: input.executables,
        cover_path: input.cover_path,
        come_back_on: input.come_back_on,
        steam_app_id: input.steam_app_id,
    };
    let dir = root.join(paths::GAMES).join(&revision.id);
    fs::create_dir_all(&dir).map_err(|e| io(&dir, e))?;
    let path = dir.join(format!("{}_{}.json", revision.revision_id, device_id));
    write_new_json(&path, &revision)?;
    Ok(revision)
}

/// Newest unforked revision per game id (linear tip; forks → no tip, omitted).
pub fn read_games(root: &Path) -> Result<Vec<GameRevision>> {
    let games_root = root.join(paths::GAMES);
    if !games_root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&games_root).map_err(|e| io(&games_root, e))? {
        let entry = entry.map_err(|e| io(&games_root, e))?;
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
            revisions.push(read_json_file::<GameRevision>(&file_path)?);
        }
        if let Some(tip) = newest_unforked_game(revisions) {
            out.push(tip);
        }
    }
    out.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()).then_with(|| a.id.cmp(&b.id)));
    Ok(out)
}

fn newest_unforked_game(revisions: Vec<GameRevision>) -> Option<GameRevision> {
    if revisions.is_empty() {
        return None;
    }
    let mut child_count: std::collections::HashMap<Option<String>, usize> = std::collections::HashMap::new();
    for r in &revisions {
        *child_count.entry(r.parent_revision_id.clone()).or_default() += 1;
    }
    if child_count.values().any(|&n| n > 1) {
        return None;
    }
    let named: std::collections::HashSet<&str> = revisions
        .iter()
        .filter_map(|r| r.parent_revision_id.as_deref())
        .collect();
    let tips: Vec<_> = revisions
        .iter()
        .filter(|r| !named.contains(r.revision_id.as_str()))
        .collect();
    if tips.len() == 1 {
        Some(tips[0].clone())
    } else {
        None
    }
}
