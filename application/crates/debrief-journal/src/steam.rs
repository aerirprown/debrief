//! Read installed Steam games from local library manifests (no account).

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{io, Result};
use crate::vdf::{parse_vdf, VdfValue};

/// One installed Steam app from an `appmanifest_*.acf`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamGame {
    pub app_id: u32,
    pub name: String,
    /// Absolute install directory (`steamapps/common/{installdir}`).
    pub install_dir: PathBuf,
    pub manifest_path: PathBuf,
}

fn vdf_err(path: &Path, msg: String) -> crate::error::Error {
    crate::error::Error::Io {
        path: path.to_path_buf(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, msg),
    }
}

/// Discover library roots from `steamapps/libraryfolders.vdf` under `steam_root`,
/// or treat `steam_root` itself as a single library if that file is missing.
pub fn library_roots(steam_root: &Path) -> Result<Vec<PathBuf>> {
    let vdf_path = steam_root.join("steamapps").join("libraryfolders.vdf");
    if !vdf_path.exists() {
        return Ok(vec![steam_root.to_path_buf()]);
    }
    let text = fs::read_to_string(&vdf_path).map_err(|e| io(&vdf_path, e))?;
    let root = parse_vdf(&text).map_err(|e| vdf_err(&vdf_path, e))?;

    let mut roots = Vec::new();
    if let Some(folders) = root.get("libraryfolders").and_then(VdfValue::as_table) {
        for (_k, v) in folders {
            if let Some(path) = v.get_str("path") {
                let path = rewrite_fixture_path(path, steam_root);
                roots.push(PathBuf::from(path));
            }
        }
    }
    if roots.is_empty() {
        roots.push(steam_root.to_path_buf());
    }
    Ok(roots)
}

fn rewrite_fixture_path(path: &str, steam_root: &Path) -> String {
    if path == "FIXTURE_STEAM_ROOT" {
        steam_root.to_string_lossy().into_owned()
    } else {
        path.to_string()
    }
}

/// Read every installed app under a Steam install (or fixture) root.
pub fn read_installed_games(steam_root: &Path) -> Result<Vec<SteamGame>> {
    let roots = library_roots(steam_root)?;
    let mut games = Vec::new();
    for lib in roots {
        let apps = lib.join("steamapps");
        if !apps.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&apps).map_err(|e| io(&apps, e))? {
            let entry = entry.map_err(|e| io(&apps, e))?;
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with("appmanifest_") || !name.ends_with(".acf") {
                continue;
            }
            if let Some(game) = read_manifest(&path, &apps)? {
                games.push(game);
            }
        }
    }
    games.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.app_id.cmp(&b.app_id))
    });
    Ok(games)
}

fn read_manifest(path: &Path, steamapps: &Path) -> Result<Option<SteamGame>> {
    let text = fs::read_to_string(path).map_err(|e| io(path, e))?;
    let root = parse_vdf(&text).map_err(|e| vdf_err(path, e))?;
    let Some(app) = root.get("AppState") else {
        return Ok(None);
    };
    let Some(app_id) = app.get_str("appid").and_then(|s| s.parse().ok()) else {
        return Ok(None);
    };
    let name = app.get_str("name").unwrap_or("Unknown").to_string();
    let installdir = app.get_str("installdir").unwrap_or("").to_string();
    if installdir.is_empty() {
        return Ok(None);
    }
    let install_dir = steamapps.join("common").join(&installdir);
    Ok(Some(SteamGame {
        app_id,
        name,
        install_dir,
        manifest_path: path.to_path_buf(),
    }))
}

/// Default Steam install guesses for this OS (may not exist).
pub fn default_steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    #[cfg(windows)]
    {
        if let Ok(pf) = std::env::var("ProgramFiles(x86)") {
            roots.push(PathBuf::from(pf).join("Steam"));
        }
        if let Ok(pf) = std::env::var("ProgramFiles") {
            roots.push(PathBuf::from(pf).join("Steam"));
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let home = PathBuf::from(home);
            roots.push(home.join(".steam/steam"));
            roots.push(home.join(".local/share/Steam"));
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            roots.push(PathBuf::from(home).join("Library/Application Support/Steam"));
        }
    }
    roots
}

pub fn steam_launch_uri(app_id: u32) -> String {
    format!("steam://rungameid/{app_id}")
}

pub fn steam_game_id(app_id: u32) -> String {
    format!("steam-{app_id}")
}
