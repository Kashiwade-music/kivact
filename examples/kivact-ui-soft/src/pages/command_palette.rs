//! Role: The catalog page of Command Palette.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "command-palette",
        "Command Palette",
        "Overlay",
        include_str!("overlays.rs"),
    )
    .example(Example::new(
        "command-palette",
        "Command Palette",
        "",
        overlays::example_command_palette,
    ))
}
