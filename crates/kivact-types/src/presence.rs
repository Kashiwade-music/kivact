//! Role: Describe the observable lifetime of a displayed node.

/// The node's entrance, settled display, or departure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presence {
    Entering,
    Present,
    Leaving,
}
