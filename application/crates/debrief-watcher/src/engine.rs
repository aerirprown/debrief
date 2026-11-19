//! Watcher state machine: debounce start, grace-period end, prompt signal.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::frame::{FrameBytes, FrameSampler};
use crate::match_path::path_matches;
use crate::process::{ProcessSource, RunningProcess};

/// A game the user confirmed for watching.
#[derive(Debug, Clone)]
pub struct WatchedGame {
    pub game_id: String,
    pub title: String,
    /// Exact executable paths and/or install folders.
    pub match_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// Match must stay alive this long before a session starts.
    pub debounce: Duration,
    /// All matches must stay gone this long before the session ends.
    pub grace: Duration,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            debounce: Duration::from_secs(3),
            grace: Duration::from_secs(15),
        }
    }
}

/// Events the shell reacts to (quit prompt, UI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchEvent {
    SessionStarted {
        game_id: String,
        started_at: String,
    },
    /// Process stayed gone past the grace period — show the quit prompt.
    Prompt {
        game_id: String,
        title: String,
        started_at: String,
        stopped_at: String,
        frame: Option<FrameBytes>,
    },
}

#[derive(Debug)]
enum Phase {
    Idle,
    Debouncing {
        since: Instant,
    },
    Active {
        started_at: String,
        started_instant: Instant,
        last_frame: Option<FrameBytes>,
        matched_exe: PathBuf,
    },
    Grace {
        started_at: String,
        missing_since: Instant,
        last_frame: Option<FrameBytes>,
    },
}

struct GameState {
    game: WatchedGame,
    phase: Phase,
}

/// One watcher over a set of confirmed games.
pub struct WatchEngine {
    config: WatchConfig,
    games: Vec<GameState>,
}

impl WatchEngine {
    pub fn new(config: WatchConfig, games: Vec<WatchedGame>) -> Self {
        Self {
            config,
            games: games
                .into_iter()
                .map(|game| GameState {
                    game,
                    phase: Phase::Idle,
                })
                .collect(),
        }
    }

    pub fn games(&self) -> impl Iterator<Item = &WatchedGame> {
        self.games.iter().map(|g| &g.game)
    }

    /// Advance one poll tick. Inject `now` so tests can run without sleeping.
    pub fn tick<P: ProcessSource, F: FrameSampler>(
        &mut self,
        now: Instant,
        wall: SystemTime,
        processes: &mut P,
        frames: &mut F,
    ) -> Vec<WatchEvent> {
        let running = processes.list();
        let mut events = Vec::new();
        for state in &mut self.games {
            if let Some(ev) = tick_game(state, &self.config, now, wall, &running, frames) {
                events.push(ev);
            }
        }
        events
    }
}

fn tick_game<F: FrameSampler>(
    state: &mut GameState,
    config: &WatchConfig,
    now: Instant,
    wall: SystemTime,
    running: &[RunningProcess],
    frames: &mut F,
) -> Option<WatchEvent> {
    let matched = find_match(&state.game.match_paths, running);
    let phase = std::mem::replace(&mut state.phase, Phase::Idle);
    match phase {
        Phase::Idle => {
            if matched.is_some() {
                state.phase = Phase::Debouncing { since: now };
            } else {
                state.phase = Phase::Idle;
            }
            None
        }
        Phase::Debouncing { since } => {
            if matched.is_none() {
                state.phase = Phase::Idle;
                return None;
            }
            if now.duration_since(since) >= config.debounce {
                let proc_ = matched.expect("matched");
                let started_at = rfc3339(wall);
                let frame = frames.sample_foreground(&proc_.exe);
                state.phase = Phase::Active {
                    started_at: started_at.clone(),
                    started_instant: now,
                    last_frame: frame,
                    matched_exe: proc_.exe.clone(),
                };
                Some(WatchEvent::SessionStarted {
                    game_id: state.game.game_id.clone(),
                    started_at,
                })
            } else {
                state.phase = Phase::Debouncing { since };
                None
            }
        }
        Phase::Active {
            started_at,
            started_instant,
            mut last_frame,
            matched_exe,
        } => {
            if let Some(proc_) = matched {
                if let Some(frame) = frames.sample_foreground(&proc_.exe) {
                    last_frame = Some(frame);
                }
                state.phase = Phase::Active {
                    started_at,
                    started_instant,
                    last_frame,
                    matched_exe: proc_.exe.clone(),
                };
                None
            } else {
                let _ = matched_exe;
                state.phase = Phase::Grace {
                    started_at,
                    missing_since: now,
                    last_frame,
                };
                None
            }
        }
        Phase::Grace {
            started_at,
            missing_since,
            last_frame,
        } => {
            if let Some(proc_) = matched {
                let frame = frames
                    .sample_foreground(&proc_.exe)
                    .or_else(|| last_frame.clone());
                state.phase = Phase::Active {
                    started_at,
                    started_instant: now,
                    last_frame: frame,
                    matched_exe: proc_.exe.clone(),
                };
                return None;
            }
            if now.duration_since(missing_since) >= config.grace {
                let stopped_at = rfc3339(wall);
                Some(WatchEvent::Prompt {
                    game_id: state.game.game_id.clone(),
                    title: state.game.title.clone(),
                    started_at,
                    stopped_at,
                    frame: last_frame,
                })
            } else {
                state.phase = Phase::Grace {
                    started_at,
                    missing_since,
                    last_frame,
                };
                None
            }
        }
    }
}

fn find_match<'a>(
    confirmed: &[PathBuf],
    running: &'a [RunningProcess],
) -> Option<&'a RunningProcess> {
    running.iter().find(|proc_| {
        confirmed
            .iter()
            .any(|path| path_matches(&proc_.exe, path))
    })
}

pub fn rfc3339(wall: SystemTime) -> String {
    let secs = wall
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let tod = secs % 86400;
    let (y, m, d) = civil_from_days(days as i64);
    let hh = tod / 3600;
    let mm = (tod % 3600) / 60;
    let ss = tod % 60;
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

/// Write a captured frame under `journal/frames/{id}.png`.
pub fn write_frame(journal_root: &Path, png: &[u8]) -> debrief_journal::Result<String> {
    use std::fs;
    use uuid::Uuid;
    let id = Uuid::new_v4().to_string();
    let dir = journal_root.join(debrief_journal::FRAMES);
    fs::create_dir_all(&dir).map_err(|e| debrief_journal::Error::Io {
        path: dir.clone(),
        source: e,
    })?;
    let path = dir.join(format!("{id}.png"));
    fs::write(&path, png).map_err(|e| debrief_journal::Error::Io {
        path: path.clone(),
        source: e,
    })?;
    Ok(id)
}
