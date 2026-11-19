//! Window frame sample kept in memory while the game is in front.

/// Bytes of a captured frame (PNG). Optional until platform capture lands.
pub type FrameBytes = Vec<u8>;

pub trait FrameSampler {
    /// Sample the game window when it is in front. `None` if not in front or unavailable.
    fn sample_foreground(&mut self, game_exe: &std::path::Path) -> Option<FrameBytes>;
}

/// Never captures a frame (tests / phones / until capture is wired).
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopFrameSampler;

impl FrameSampler for NoopFrameSampler {
    fn sample_foreground(&mut self, _game_exe: &std::path::Path) -> Option<FrameBytes> {
        None
    }
}

/// Returns a fixed PNG whenever asked (tests).
#[derive(Debug, Clone)]
pub struct FixedFrameSampler {
    pub png: FrameBytes,
}

impl FrameSampler for FixedFrameSampler {
    fn sample_foreground(&mut self, _game_exe: &std::path::Path) -> Option<FrameBytes> {
        Some(self.png.clone())
    }
}
