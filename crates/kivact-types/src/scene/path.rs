//! Role: Owned portable path command recordings.
use crate::scene::PathCmd;

/// An owned path recording, independent of a geometry library or renderer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PathData {
    /// Contours in declaration order.
    pub cmds: Vec<PathCmd>,
}
