//! Process list sources for the watcher.

use std::path::{Path, PathBuf};

use sysinfo::{ProcessesToUpdate, System};

/// A running process image the watcher can match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningProcess {
    pub pid: u32,
    pub exe: PathBuf,
}

pub trait ProcessSource {
    fn list(&mut self) -> Vec<RunningProcess>;
}

/// Live OS process list (Windows / Linux desktop).
pub struct SystemProcessSource {
    system: System,
}

impl SystemProcessSource {
    pub fn new() -> Self {
        Self {
            system: System::new(),
        }
    }
}

impl Default for SystemProcessSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessSource for SystemProcessSource {
    fn list(&mut self) -> Vec<RunningProcess> {
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        let mut out = Vec::new();
        for (pid, proc_) in self.system.processes() {
            let Some(exe) = proc_.exe() else {
                continue;
            };
            out.push(RunningProcess {
                pid: pid.as_u32(),
                exe: exe.to_path_buf(),
            });
        }
        out
    }
}

/// Test double: caller sets which paths are "running".
#[derive(Debug, Default, Clone)]
pub struct MockProcessSource {
    pub running: Vec<PathBuf>,
}

impl MockProcessSource {
    pub fn with(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> Self {
        Self {
            running: paths
                .into_iter()
                .map(|p| p.as_ref().to_path_buf())
                .collect(),
        }
    }
}

impl ProcessSource for MockProcessSource {
    fn list(&mut self) -> Vec<RunningProcess> {
        self.running
            .iter()
            .enumerate()
            .map(|(i, exe)| RunningProcess {
                pid: (i as u32) + 1,
                exe: exe.clone(),
            })
            .collect()
    }
}
