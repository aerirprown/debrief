//! Game process watcher + quit prompt (Windows / Linux desktop).
//! Polls confirmed executables, then opens a small prompt window to write a session.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use debrief_watcher::{
    finish_session, LocalLog, LogEvent, NoopFrameSampler, PendingSession, SystemProcessSource,
    WatchConfig, WatchEngine, WatchEvent, WatchedGame,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

const LABEL: &str = "quit-prompt";
const WIDTH: f64 = 420.0;
const HEIGHT: f64 = 200.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchGameInput {
    pub game_id: String,
    pub title: String,
    pub match_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptInfo {
    pub game_id: String,
    pub title: String,
    pub started_at: String,
    pub stopped_at: String,
}

struct Inner {
    journal_root: PathBuf,
    device_id: String,
    log: LocalLog,
    pending: Option<(PendingSession, String)>,
    stop: bool,
}

#[derive(Clone, Default)]
pub struct WatcherHandle {
    inner: Arc<Mutex<Option<Inner>>>,
}

impl WatcherHandle {
    pub fn new() -> Self {
        Self::default()
    }
}

#[tauri::command]
pub fn watcher_start(
    app: AppHandle,
    state: tauri::State<'_, WatcherHandle>,
    journal_root: String,
    device_id: String,
    games: Vec<WatchGameInput>,
    debounce_ms: Option<u64>,
    grace_ms: Option<u64>,
) -> Result<(), String> {
    let root = PathBuf::from(&journal_root);
    if !root.is_dir() {
        return Err(format!("journal root is not a directory: {journal_root}"));
    }
    // Watcher log stays on this PC (app data), not in the synced journal folder.
    let log_path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("debrief-watcher.log");
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let log = LocalLog::open(&log_path).map_err(|e| e.to_string())?;

    let watched: Vec<WatchedGame> = games
        .into_iter()
        .map(|g| WatchedGame {
            game_id: g.game_id,
            title: g.title,
            match_paths: g.match_paths.into_iter().map(PathBuf::from).collect(),
        })
        .collect();

    let config = WatchConfig {
        debounce: Duration::from_millis(debounce_ms.unwrap_or(3_000)),
        grace: Duration::from_millis(grace_ms.unwrap_or(15_000)),
    };

    {
        let mut slot = state.inner.lock().map_err(|e| e.to_string())?;
        if let Some(prev) = slot.as_mut() {
            prev.stop = true;
        }
        *slot = Some(Inner {
            journal_root: root,
            device_id,
            log,
            pending: None,
            stop: false,
        });
    }

    let handle = state.inner.clone();
    let app_handle = app.clone();
    thread::Builder::new()
        .name("debrief-watcher".into())
        .spawn(move || {
            run_loop(app_handle, handle, config, watched);
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn watcher_stop(state: tauri::State<'_, WatcherHandle>) -> Result<(), String> {
    let mut slot = state.inner.lock().map_err(|e| e.to_string())?;
    if let Some(inner) = slot.as_mut() {
        inner.stop = true;
    }
    Ok(())
}

#[tauri::command]
pub fn watcher_prompt_info(state: tauri::State<'_, WatcherHandle>) -> Result<Option<PromptInfo>, String> {
    let slot = state.inner.lock().map_err(|e| e.to_string())?;
    Ok(slot.as_ref().and_then(|inner| {
        inner.pending.as_ref().map(|(p, title)| PromptInfo {
            game_id: p.game_id.clone(),
            title: title.clone(),
            started_at: p.started_at.clone(),
            stopped_at: p.stopped_at.clone(),
        })
    }))
}

#[tauri::command]
pub fn watcher_submit(
    state: tauri::State<'_, WatcherHandle>,
    app: AppHandle,
    text: String,
    what_next: String,
) -> Result<(), String> {
    finish_pending(&state, &app, text, what_next, false)
}

#[tauri::command]
pub fn watcher_dismiss(state: tauri::State<'_, WatcherHandle>, app: AppHandle) -> Result<(), String> {
    finish_pending(&state, &app, String::new(), String::new(), true)
}

fn finish_pending(
    state: &WatcherHandle,
    app: &AppHandle,
    text: String,
    what_next: String,
    dismissed: bool,
) -> Result<(), String> {
    let mut slot = state.inner.lock().map_err(|e| e.to_string())?;
    let Some(inner) = slot.as_mut() else {
        return Err("watcher not started".into());
    };
    let Some((pending, _)) = inner.pending.take() else {
        return Err("no pending prompt".into());
    };
    let _ = crate::mid_session::mid_session_set_context(None);
    let game_id = pending.game_id.clone();
    finish_session(
        &inner.journal_root,
        &inner.device_id,
        &pending,
        text,
        what_next,
    )
    .map_err(|e| e.to_string())?;
    let log_event = if dismissed {
        LogEvent::PromptDismissed {
            game_id: game_id.clone(),
            at: pending.stopped_at.clone(),
        }
    } else {
        LogEvent::PromptSubmitted {
            game_id: game_id.clone(),
            at: pending.stopped_at.clone(),
        }
    };
    let _ = inner.log.append(&log_event);
    drop(slot);
    hide_prompt(app);
    let _ = app.emit("watcher://session-saved", game_id);
    Ok(())
}

fn run_loop(
    app: AppHandle,
    handle: Arc<Mutex<Option<Inner>>>,
    config: WatchConfig,
    games: Vec<WatchedGame>,
) {
    let mut engine = WatchEngine::new(config, games);
    let mut processes = SystemProcessSource::new();
    let mut frames = NoopFrameSampler;
    let t0 = Instant::now();
    loop {
        {
            let Ok(slot) = handle.lock() else { break };
            if slot.as_ref().map(|i| i.stop).unwrap_or(true) {
                break;
            }
            if slot.as_ref().and_then(|i| i.pending.as_ref()).is_some() {
                // Wait for the user to answer before watching again.
                drop(slot);
                thread::sleep(Duration::from_millis(200));
                continue;
            }
        }

        let now = Instant::now();
        let wall = SystemTime::now();
        let events = engine.tick(now, wall, &mut processes, &mut frames);
        for ev in events {
            match ev {
                WatchEvent::SessionStarted { game_id, started_at } => {
                    let title = engine
                        .games()
                        .find(|g| g.game_id == game_id)
                        .map(|g| g.title.clone())
                        .unwrap_or_else(|| game_id.clone());
                    if let Ok(mut slot) = handle.lock() {
                        if let Some(inner) = slot.as_mut() {
                            let _ = inner.log.append(&LogEvent::SessionStarted {
                                game_id: game_id.clone(),
                                at: started_at,
                            });
                            let _ = crate::mid_session::mid_session_set_context(Some(
                                crate::mid_session::MidSessionContext {
                                    journal_root: inner.journal_root.to_string_lossy().into_owned(),
                                    game_id: game_id.clone(),
                                    title,
                                },
                            ));
                        }
                    }
                    let _ = app.emit("watcher://session-started", game_id);
                }
                WatchEvent::Prompt {
                    game_id,
                    title,
                    started_at,
                    stopped_at,
                    frame,
                } => {
                    let info = PromptInfo {
                        game_id: game_id.clone(),
                        title: title.clone(),
                        started_at: started_at.clone(),
                        stopped_at: stopped_at.clone(),
                    };
                    if let Ok(mut slot) = handle.lock() {
                        if let Some(inner) = slot.as_mut() {
                            let _ = inner.log.append(&LogEvent::PromptOffered {
                                game_id: game_id.clone(),
                                at: stopped_at.clone(),
                            });
                            inner.pending = Some((
                                PendingSession {
                                    game_id,
                                    started_at,
                                    stopped_at,
                                    frame,
                                },
                                title,
                            ));
                        }
                    }
                    let _ = crate::mid_session::mid_session_set_context(None);
                    let _ = app.emit("watcher://prompt", info);
                    show_prompt(&app);
                }
            }
        }
        let _ = t0;
        thread::sleep(Duration::from_secs(2));
    }
}

fn show_prompt(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let builder = WebviewWindowBuilder::new(
        app,
        LABEL,
        WebviewUrl::App("index.html?quit-prompt=1".into()),
    )
    .title("Debrief")
    .inner_size(WIDTH, HEIGHT)
    .resizable(false)
    .minimizable(false)
    .maximizable(false)
    .always_on_top(true)
    .visible(true)
    .center();
    #[cfg(windows)]
    let builder = match app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == "main")
        .and_then(|w| w.additional_browser_args.clone())
    {
        Some(args) => builder.additional_browser_args(&args),
        None => builder,
    };
    let _ = builder.build();
}

fn hide_prompt(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.hide();
    }
}
