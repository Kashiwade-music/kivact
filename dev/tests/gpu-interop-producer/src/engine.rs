//! Role: What the backends' engines do, and the pool's constants.

use std::time::Duration;

/// Textures in the pool.
pub const POOL: usize = 3;

/// The longest the engine waits on the CPU for its earlier work.
pub const WAIT: Duration = Duration::from_secs(5);

/// A backend's engine: its own device drawing into a shared pool.
pub(crate) trait Engine {
    /// Copy `rgba` (rows `pitch` apart) into the pool's next texture and
    /// signal `ready`: the texture and the value.
    fn draw(&mut self, rgba: &[u8]) -> Result<kivact_gpu_share::FrameReady, String>;
    /// The row pitch [`Engine::draw`] takes.
    fn pitch(&self) -> usize;
    /// The release fence's value now.
    fn release_value(&self) -> u64;
    /// Wait at most `timeout` for the release fence to reach `value`.
    fn wait_release(&self, value: u64, timeout: Duration) -> bool;
    /// Texture `slot`'s last frame, read back as straight RGBA8.
    fn read_back(&self, slot: u32) -> Result<Vec<u8>, String>;
}
