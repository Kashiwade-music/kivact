//! Role: Expose palette behavior and keep its transition implementation private.
mod handle;
mod machine;
pub use handle::{Command, CommandGroup, CommandPaletteApi, filter_commands, use_command_palette};
