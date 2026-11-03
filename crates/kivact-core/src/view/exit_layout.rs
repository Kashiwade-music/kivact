//! Role: Select how a retained exit participates in its parent's layout.

/// How a removed host node participates in layout during its exit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExitLayout {
    /// Keep the node in flow until its exit finishes.
    #[default]
    Keep,
    /// Release its layout slot at removal, keeping its last local box for exit paint.
    Pop,
}
