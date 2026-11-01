//! Role: Three-lane Task Board app with context, keyed lists and a saving queue.
//!
//! Three-lane Task Board on the Kivact public API: context, keyed virtual
//! lists, a resource for loading and a coalescing queue for saving.
#![warn(redundant_imports)]

mod components;
mod context;
mod model;
mod platform;
mod storage;

pub use components::app;
pub use model::{Board, Lane, Task};
pub use storage::{default_path, load, load_initial, save};

#[cfg(test)]
mod tests {
    //! Role: Tests for the Task Board with a large number of rows.

    use super::*;

    #[test]
    fn hundred_thousand_rows_materialize_only_the_window_and_one_edit_clones_one_task() {
        let path = std::env::temp_dir().join(format!(
            "kivact-task-board-large-{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let file = path.clone();
        let screen = kivact::test::render(move || app(file, Some(100_000)));
        screen.find_by_text("予定 (100000件)");
        let stats = screen.stats();
        println!(
            "tree_nodes={} live_reactive={} core_ms={:.1}",
            stats.tree_nodes,
            stats.live_reactive,
            stats.core_ns as f64 / 1e6
        );
        assert!(
            stats.tree_nodes < 1_000,
            "only the visible window is mounted: {}",
            stats.tree_nodes
        );
        model::reset_task_clones();
        let before = screen.stats();
        screen.get_by_test_id("right-1").click();
        let after = screen.stats();
        println!(
            "move: task_clones={} ops={} core_ms={:.2}",
            model::task_clone_count(),
            after.ops - before.ops,
            (after.core_ns - before.core_ns) as f64 / 1e6
        );
        assert_eq!(model::task_clone_count(), 1, "one task is copied on edit");
        screen.find_by_text("進行中 (1件)");
        screen.find_by_text("保存しました");
        assert_eq!(load(&path).unwrap().task(1).unwrap().lane, Lane::InProgress);
        let _ = std::fs::remove_file(path);
    }
}
