//! Role: Portable identities shared by text producers and hosts.

/// The stable key of a text run; usable on any thread.
/// Authors keep the key across updates; the engine assigns its platform identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextRunId(pub u64);
