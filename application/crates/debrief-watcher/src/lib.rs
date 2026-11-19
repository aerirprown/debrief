//! Desktop game watcher for Debrief.
//!
//! Polls the process list, starts a session after a debounce, ends it after a
//! grace period when every matched process is gone, and signals the quit prompt.
//! Focus loss is not a stop. Phone builds do not use this crate.

mod engine;
mod error;
mod frame;
mod local_log;
mod match_path;
mod process;
mod session_finish;

pub use engine::{rfc3339, write_frame, WatchConfig, WatchEngine, WatchEvent, WatchedGame};
pub use error::{Error, Result};
pub use frame::{FixedFrameSampler, FrameBytes, FrameSampler, NoopFrameSampler};
pub use local_log::{LocalLog, LogEvent};
pub use match_path::path_matches;
pub use process::{MockProcessSource, ProcessSource, RunningProcess, SystemProcessSource};
pub use session_finish::{finish_session, PendingSession};
