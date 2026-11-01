//! Role: Task Board binary.
//! `KIVACT_TASK_BOARD_GENERATED=N` opens an unsaved
//! board of N generated tasks for the large-list check.
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact_example_task_board::{app, default_path};

fn main() -> kivact::Result<()> {
    // Large-list check only: an unsaved board of N generated tasks.
    let generated = std::env::var("KIVACT_TASK_BOARD_GENERATED")
        .ok()
        .and_then(|n| n.parse().ok());
    App::new("Kivact · Task Board").run(move || app(default_path(), generated))
}
