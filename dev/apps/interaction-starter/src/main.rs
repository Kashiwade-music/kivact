//! Role: Entry point of the Interaction Starter app, which opens a notes file.
#![warn(redundant_imports)]

use std::path::PathBuf;

fn main() -> kivact::Result<()> {
    let path = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("interaction-notes.txt"), PathBuf::from);
    kivact::App::new("Kivact Interaction Starter")
        .run(move || kivact_interaction_starter::app(path))
}
