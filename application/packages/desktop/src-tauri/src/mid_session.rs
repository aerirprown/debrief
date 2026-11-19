//! Mid-session hotkey: one-line append onto the active game's note page.
//! Desktop only. Uses the same global-shortcut plugin as Ninja; the Ninja
//! handler forwards presses that match this accelerator.

use std::str::FromStr;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Modifiers, Shortcut};

const LABEL: &str = "mid-session";
const WIDTH: f64 = 420.0;
const HEIGHT: f64 = 140.0;
const DEFAULT_ACCEL: &str = "Alt+Shift+N";

static CURRENT: Mutex<Option<Shortcut>> = Mutex::new(None);
static CONTEXT: Mutex<Option<MidSessionContext>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MidSessionContext {
    pub journal_root: String,
    pub game_id: String,
    pub title: String,
}

/// True when this press belongs to the mid-session hotkey (and was handled).
pub fn handle_if_ours<R: Runtime>(app: &AppHandle<R>, shortcut: &Shortcut) -> bool {
    let ours = CURRENT.lock().ok().and_then(|g| g.clone());
    let Some(ours) = ours else {
        return false;
    };
    if *shortcut != ours {
        return false;
    }
    let ctx = CONTEXT.lock().ok().and_then(|g| g.clone());
    let Some(ctx) = ctx else {
        // No active game session — ignore quietly.
        return true;
    };
    show_prompt(app, &ctx);
    true
}

#[tauri::command]
pub fn mid_session_set_context(ctx: Option<MidSessionContext>) -> Result<(), String> {
    let mut slot = CONTEXT.lock().map_err(|e| e.to_string())?;
    *slot = ctx;
    Ok(())
}

#[tauri::command]
pub fn mid_session_context() -> Result<Option<MidSessionContext>, String> {
    let slot = CONTEXT.lock().map_err(|e| e.to_string())?;
    Ok(slot.clone())
}

/// Register (or clear) the mid-session accelerator. Default Alt+Shift+N.
#[tauri::command]
pub fn mid_session_set_shortcut<R: Runtime>(
    app: AppHandle<R>,
    accelerator: Option<String>,
) -> Result<(), String> {
    let mut current = CURRENT.lock().map_err(|e| e.to_string())?;
    if let Some(old) = current.take() {
        let _ = app.global_shortcut().unregister(old);
    }
    let accel = accelerator.unwrap_or_else(|| DEFAULT_ACCEL.to_string());
    if accel.is_empty() {
        return Ok(());
    }
    let shortcut = Shortcut::from_str(&accel).map_err(|_| "invalid".to_owned())?;
    if !shortcut
        .mods
        .intersects(Modifiers::ALT | Modifiers::CONTROL | Modifiers::SUPER)
    {
        return Err("invalid".to_owned());
    }
    app.global_shortcut()
        .register(shortcut)
        .map_err(|_| "taken".to_owned())?;
    *current = Some(shortcut);
    Ok(())
}

#[tauri::command]
pub fn mid_session_submit(
    app: AppHandle,
    journal_root: String,
    game_id: String,
    line: String,
) -> Result<(), String> {
    crate::note_extras::note_append_hotkey_line(
        app.clone(),
        journal_root,
        game_id.clone(),
        line,
    )?;
    hide_prompt(&app);
    let _ = app.emit("mid-session://saved", game_id);
    Ok(())
}

#[tauri::command]
pub fn mid_session_dismiss(app: AppHandle) -> Result<(), String> {
    hide_prompt(&app);
    Ok(())
}

fn show_prompt<R: Runtime>(app: &AppHandle<R>, ctx: &MidSessionContext) {
    let _ = app.emit_to("main", "mid-session://open", ctx.clone());
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.show();
        let _ = w.set_focus();
        let _ = app.emit_to(LABEL, "mid-session://shown", ctx.clone());
        return;
    }
    let app = app.clone();
    let ctx = ctx.clone();
    std::thread::spawn(move || {
        let builder = WebviewWindowBuilder::new(
            &app,
            LABEL,
            WebviewUrl::App("index.html?mid-session=1".into()),
        )
        .title("Debrief note")
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
        match builder.build() {
            Ok(w) => {
                let _ = w.set_focus();
                let _ = app.emit_to(LABEL, "mid-session://shown", ctx);
            }
            Err(e) => eprintln!("could not open mid-session prompt: {e}"),
        }
    });
}

fn hide_prompt(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.hide();
    }
}

/// Register the default shortcut once the global-shortcut plugin is ready.
pub fn register_default<R: Runtime>(app: &AppHandle<R>) {
    let _ = mid_session_set_shortcut(app.clone(), Some(DEFAULT_ACCEL.to_string()));
}
