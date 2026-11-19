//! Per-PC watcher log (not part of the synced journal folder).

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{io, Result};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEvent {
    MatchSeen { game_id: String, at: String },
    SessionStarted { game_id: String, at: String },
    SessionEnded { game_id: String, at: String },
    PromptOffered { game_id: String, at: String },
    PromptSubmitted { game_id: String, at: String },
    PromptDismissed { game_id: String, at: String },
}

pub struct LocalLog {
    path: PathBuf,
}

impl LocalLog {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| io(parent, e))?;
        }
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(&self, event: &LogEvent) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| io(&self.path, e))?;
        let line = serde_json::to_string(event).map_err(|e| {
            crate::error::Error::Json {
                path: self.path.clone(),
                source: e,
            }
        })?;
        writeln!(file, "{line}").map_err(|e| io(&self.path, e))?;
        Ok(())
    }
}
