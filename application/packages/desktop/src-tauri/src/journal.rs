//! Gaming journal bridge: read games and return cards from a journal folder.
//! Manual session writes are used on phone (no process watcher).

use debrief_journal::{
    read_games, read_memoir, return_card_for, write_session, NewSession, ReturnCard, Session,
};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use std::path::PathBuf;

use crate::device_id::install_device_id;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDto {
    pub id: String,
    pub title: String,
    pub cover_path: Option<String>,
    pub come_back_on: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReturnCardDto {
    pub game_id: String,
    pub title: String,
    pub line: String,
    pub what_next: String,
    pub days_since: u32,
    pub frame_id: Option<String>,
    pub frame_path: Option<String>,
    pub session_id: Option<String>,
    pub stopped_at: Option<String>,
}

impl From<ReturnCard> for ReturnCardDto {
    fn from(c: ReturnCard) -> Self {
        let frame_path = c
            .frame_id
            .as_ref()
            .map(|id| format!("frames/{id}.png"));
        Self {
            game_id: c.game_id,
            title: c.title,
            line: c.line,
            what_next: c.what_next,
            days_since: c.days_since,
            frame_id: c.frame_id,
            frame_path,
            session_id: c.session_id,
            stopped_at: c.stopped_at,
        }
    }
}

fn root_path(journal_root: String) -> Result<PathBuf, String> {
    let path = PathBuf::from(journal_root);
    if !path.is_dir() {
        return Err(format!("journal root is not a directory: {}", path.display()));
    }
    Ok(path)
}

#[tauri::command]
pub fn journal_list_games(journal_root: String) -> Result<Vec<GameDto>, String> {
    let root = root_path(journal_root)?;
    let games = read_games(&root).map_err(|e| e.to_string())?;
    Ok(games
        .into_iter()
        .map(|g| GameDto {
            id: g.id,
            title: g.title,
            cover_path: g.cover_path,
            come_back_on: g.come_back_on,
        })
        .collect())
}

#[tauri::command]
pub fn journal_return_card(
    journal_root: String,
    game_id: String,
    as_of_day: String,
) -> Result<Option<ReturnCardDto>, String> {
    let root = root_path(journal_root)?;
    let games = read_games(&root).map_err(|e| e.to_string())?;
    let Some(game) = games.into_iter().find(|g| g.id == game_id) else {
        return Ok(None);
    };
    let memoir = read_memoir(&root).map_err(|e| e.to_string())?;
    Ok(Some(ReturnCardDto::from(return_card_for(
        &game,
        &memoir.sessions,
        &as_of_day,
    ))))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSessionInput {
    pub game_id: String,
    pub text: String,
    pub what_next: Option<String>,
    /// RFC 3339; defaults to now when omitted.
    pub started_at: Option<String>,
    /// RFC 3339; defaults to now when omitted.
    pub stopped_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDto {
    pub id: String,
    pub device_id: String,
    pub game_id: String,
    pub started_at: String,
    pub stopped_at: String,
    pub text: String,
    pub what_next: String,
    pub frame_id: Option<String>,
}

impl From<Session> for SessionDto {
    fn from(s: Session) -> Self {
        Self {
            id: s.id,
            device_id: s.device_id,
            game_id: s.game_id,
            started_at: s.started_at,
            stopped_at: s.stopped_at,
            text: s.text,
            what_next: s.what_next,
            frame_id: s.frame_id,
        }
    }
}

/// Manual session entry (phones, consoles, cloud streams — no watcher).
/// Callers should pass RFC 3339 `started_at` / `stopped_at` (web uses `Date.toISOString()`).
#[tauri::command]
pub fn journal_add_session(
    app: AppHandle,
    journal_root: String,
    input: AddSessionInput,
) -> Result<SessionDto, String> {
    let root = root_path(journal_root)?;
    let device = install_device_id(&app)?;
    let started = input
        .started_at
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "started_at is required".to_string())?;
    let stopped = input
        .stopped_at
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "stopped_at is required".to_string())?;
    let session = write_session(
        &root,
        &device,
        NewSession {
            game_id: input.game_id,
            started_at: started,
            stopped_at: stopped,
            text: input.text,
            what_next: input.what_next.unwrap_or_default(),
            frame_id: None,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(SessionDto::from(session))
}
