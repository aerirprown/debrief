//! Shelf: Steam installs + manual journal games, with last line and days since.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::game::read_games;
use crate::memoir::read_memoir;
use crate::return_card::{days_since_stopped, newest_session};
use crate::steam::{read_installed_games, steam_game_id, steam_launch_uri, SteamGame};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShelfSource {
    Steam,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchTarget {
    /// `steam://rungameid/{id}`
    Steam { app_id: u32, uri: String },
    /// Absolute path to an executable.
    Exe { path: PathBuf },
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShelfRow {
    pub id: String,
    pub title: String,
    pub source: ShelfSource,
    pub steam_app_id: Option<u32>,
    pub install_dir: Option<PathBuf>,
    pub cover_path: Option<String>,
    pub line: String,
    pub days_since: u32,
    pub launch: LaunchTarget,
}

/// Build the shelf from a Steam library root (optional) and a journal folder.
pub fn build_shelf(
    journal_root: &Path,
    steam_root: Option<&Path>,
    as_of_day: &str,
) -> Result<Vec<ShelfRow>> {
    let memoir = read_memoir(journal_root)?;
    let manual = read_games(journal_root)?;
    let steam_games = match steam_root {
        Some(root) if root.is_dir() => read_installed_games(root)?,
        _ => Vec::new(),
    };

    let mut rows = Vec::new();
    let mut seen_steam: HashSet<u32> = HashSet::new();

    // Manual / journal games first (user-added covers, confirmed exes).
    for game in &manual {
        let session = newest_session(&memoir.sessions, &game.id);
        let (line, days_since) = match session {
            Some(s) => (
                s.text.clone(),
                days_since_stopped(&s.stopped_at, as_of_day),
            ),
            None => (String::new(), 0),
        };
        let launch = if let Some(app_id) = game.steam_app_id {
            seen_steam.insert(app_id);
            LaunchTarget::Steam {
                app_id,
                uri: steam_launch_uri(app_id),
            }
        } else if let Some(exe) = game.executables.first() {
            LaunchTarget::Exe {
                path: PathBuf::from(exe),
            }
        } else {
            LaunchTarget::None
        };
        rows.push(ShelfRow {
            id: game.id.clone(),
            title: game.title.clone(),
            source: if game.steam_app_id.is_some() {
                ShelfSource::Steam
            } else {
                ShelfSource::Manual
            },
            steam_app_id: game.steam_app_id,
            install_dir: None,
            cover_path: game.cover_path.clone(),
            line,
            days_since,
            launch,
        });
    }

    // Steam installs not yet represented in the journal.
    for sg in steam_games {
        if seen_steam.contains(&sg.app_id) {
            continue;
        }
        // If a journal game uses the steam-{id} convention, skip duplicate.
        let sid = steam_game_id(sg.app_id);
        if rows.iter().any(|r| r.id == sid) {
            continue;
        }
        rows.push(steam_only_row(&sg, &memoir.sessions, as_of_day));
    }

    rows.sort_by(|a, b| {
        a.title
            .to_lowercase()
            .cmp(&b.title.to_lowercase())
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(rows)
}

fn steam_only_row(
    sg: &SteamGame,
    sessions: &[crate::session::Session],
    as_of_day: &str,
) -> ShelfRow {
    let id = steam_game_id(sg.app_id);
    let session = newest_session(sessions, &id);
    let (line, days_since) = match session {
        Some(s) => (
            s.text.clone(),
            days_since_stopped(&s.stopped_at, as_of_day),
        ),
        None => (String::new(), 0),
    };
    ShelfRow {
        id,
        title: sg.name.clone(),
        source: ShelfSource::Steam,
        steam_app_id: Some(sg.app_id),
        install_dir: Some(sg.install_dir.clone()),
        cover_path: None,
        line,
        days_since,
        launch: LaunchTarget::Steam {
            app_id: sg.app_id,
            uri: steam_launch_uri(sg.app_id),
        },
    }
}
