//! Role: The editing machinery only the native host uses: editor, journal, damage record.

pub use crate::behavior::{EditLayout, Lines};
pub use crate::change::Damage;
pub use crate::edit_state::Taken;
pub use crate::editor::Editor;
pub use crate::journal::{Incoming, Journal, Limits};
