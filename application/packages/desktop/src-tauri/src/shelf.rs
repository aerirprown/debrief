//! Games shelf: Steam installs + manual journal games, launch, add manual.

use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::process::Command;

use debrief_journal::{
    build_shelf, create_device_id, list_journal_files, steam_launch_uri, write_game_revision,
    Journal, LaunchTarget, NewGame, ShelfRow, ShelfSource,
};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_dialog::DialogExt;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_opener::OpenerExt;

use crate::device_id::install_device_id;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfRowDto {
    pub id: String,
    pub title: String,
    pub source: String,
    pub steam_app_id: Option<u32>,
    pub install_dir: Option<String>,
    pub cover_path: Option<String>,
    pub line: String,
    pub days_since: u32,
    pub can_launch: bool,
    pub launch_uri: Option<String>,
    pub launch_exe: Option<String>,
}

impl From<ShelfRow> for ShelfRowDto {
    fn from(r: ShelfRow) -> Self {
        let (can_launch, launch_uri, launch_exe) = match &r.launch {
            LaunchTarget::Steam { uri, .. } => (true, Some(uri.clone()), None),
            LaunchTarget::Exe { path } => (true, None, Some(path.to_string_lossy().into_owned())),
            LaunchTarget::None => (false, None, None),
        };
        Self {
            id: r.id,
            title: r.title,
            source: match r.source {
                ShelfSource::Steam => "steam".into(),
                ShelfSource::Manual => "manual".into(),
            },
            steam_app_id: r.steam_app_id,
            install_dir: r.install_dir.map(|p| p.to_string_lossy().into_owned()),
            cover_path: r.cover_path,
            line: r.line,
            days_since: r.days_since,
            can_launch,
            launch_uri,
            launch_exe,
        }
    }
}

fn root_path(journal_root: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(journal_root);
    if !path.is_dir() {
        return Err(format!("journal root is not a directory: {journal_root}"));
    }
    Ok(path)
}

/// List shelf rows. When `phone` is true, skip Steam and clear launch targets.
#[tauri::command]
pub fn shelf_list(
    journal_root: String,
    steam_root: Option<String>,
    as_of_day: String,
    phone: Option<bool>,
) -> Result<Vec<ShelfRowDto>, String> {
    let root = root_path(&journal_root)?;
    let phone = phone.unwrap_or(false);
    let steam = if phone {
        None
    } else {
        steam_root.as_deref().map(Path::new)
    };
    let rows = build_shelf(&root, steam, &as_of_day).map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|r| {
            let mut dto = ShelfRowDto::from(r);
            if phone {
                dto.can_launch = false;
                dto.launch_uri = None;
                dto.launch_exe = None;
            }
            dto
        })
        .collect())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddManualGame {
    pub title: String,
    pub executable: Option<String>,
    /// Absolute path to a cover image to copy into the journal `media/` folder.
    pub cover_file: Option<String>,
    pub steam_app_id: Option<u32>,
}

#[tauri::command]
pub fn shelf_add_manual(
    app: AppHandle,
    journal_root: String,
    input: AddManualGame,
) -> Result<ShelfRowDto, String> {
    let root = root_path(&journal_root)?;
    let device_id = install_device_id(&app)?;
    let cover_path = match input.cover_file.as_deref() {
        Some(src) => Some(copy_cover(&root, Path::new(src))?),
        None => None,
    };
    let executables = input
        .executable
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let game = write_game_revision(
        &root,
        &device_id,
        NewGame {
            id: None,
            title: input.title,
            executables,
            cover_path,
            come_back_on: None,
            parent_revision_id: None,
            steam_app_id: input.steam_app_id,
        },
    )
    .map_err(|e| e.to_string())?;

    // Return a shelf-shaped row for the new game (no sessions yet).
    let launch = if let Some(app_id) = game.steam_app_id {
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
    Ok(ShelfRowDto::from(ShelfRow {
        id: game.id,
        title: game.title,
        source: if game.steam_app_id.is_some() {
            ShelfSource::Steam
        } else {
            ShelfSource::Manual
        },
        steam_app_id: game.steam_app_id,
        install_dir: None,
        cover_path: game.cover_path,
        line: String::new(),
        days_since: 0,
        launch,
    }))
}

fn copy_cover(journal_root: &Path, src: &Path) -> Result<String, String> {
    if !src.is_file() {
        return Err(format!("cover file not found: {}", src.display()));
    }
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let id = create_device_id(); // random id reuse
    let id = id.replace('-', "");
    let media = journal_root.join("media");
    fs::create_dir_all(&media).map_err(|e| e.to_string())?;
    let name = format!("cover-{id}.{ext}");
    let dest = media.join(&name);
    fs::copy(src, &dest).map_err(|e| e.to_string())?;
    Ok(format!("media/{name}"))
}

#[tauri::command]
pub fn shelf_launch(app: AppHandle, launch_uri: Option<String>, launch_exe: Option<String>) -> Result<(), String> {
    // Phones never launch PC games from the shelf.
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = (app, launch_uri, launch_exe);
        return Err("launch is desktop-only".into());
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        if let Some(uri) = launch_uri {
            app.opener()
                .open_url(uri, None::<&str>)
                .map_err(|e| e.to_string())?;
            return Ok(());
        }
        if let Some(exe) = launch_exe {
            Command::new(&exe)
                .spawn()
                .map_err(|e| format!("failed to launch {exe}: {e}"))?;
            return Ok(());
        }
        Err("nothing to launch".into())
    }
}

#[tauri::command]
pub fn shelf_default_steam_root() -> Option<String> {
    debrief_journal::default_steam_roots()
        .into_iter()
        .find(|p| p.is_dir())
        .map(|p| p.to_string_lossy().into_owned())
}

/// Ensure a journal folder exists (layout only). Device id stays in app data.
#[tauri::command]
pub fn shelf_ensure_journal(app: AppHandle, journal_root: String) -> Result<String, String> {
    let path = PathBuf::from(&journal_root);
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    let device = install_device_id(&app)?;
    let _ = Journal::open(&path, &device).map_err(|e| e.to_string())?;
    // Never write device id into the journal — two PCs sharing a cloud folder
    // each keep their own id beside app settings.
    let legacy = path.join(".device_id");
    if legacy.is_file() {
        let _ = fs::remove_file(&legacy);
    }
    Ok(device)
}

/// Relative paths under the synced journal trees (sessions, notes, …).
#[tauri::command]
pub fn journal_list_files(journal_root: String) -> Result<Vec<String>, String> {
    let root = root_path(&journal_root)?;
    list_journal_files(&root).map_err(|e| e.to_string())
}

/// Folder picker for pointing the gaming journal at a cloud-synced directory.
/// Desktop: system folder dialog. Phone: app-data `debrief-journal` (std::fs
/// journal; SAF/bookmark sync can come later — see gaming-journal-phone.md).
#[tauri::command]
pub fn journal_pick_folder(
    app: AppHandle,
    window: tauri::WebviewWindow,
) -> Result<Option<String>, String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = &window;
        use tauri::Manager;
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|e| e.to_string())?
            .join("debrief-journal");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        return Ok(Some(dir.to_string_lossy().into_owned()));
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let Some(picked) = app
            .dialog()
            .file()
            .set_parent(&window)
            .set_title("Choose gaming journal folder")
            .blocking_pick_folder()
        else {
            return Ok(None);
        };
        let path = picked.into_path().map_err(|e| e.to_string())?;
        Ok(Some(path.to_string_lossy().into_owned()))
    }
}
