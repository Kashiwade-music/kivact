//! Role: Expose the command palette behavior.
mod handle;
pub use handle::{Command, CommandGroup, CommandPaletteApi, filter_commands, use_command_palette};
