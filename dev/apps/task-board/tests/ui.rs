//! Role: Task Board through `kivact::test` (a real Arvane window, virtual clock).
//! Every interaction is real input; the JSON file is the real storage.
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact::test::{Element, Screen, render};
use kivact_example_task_board::{Lane, app, load};
use std::path::{Path, PathBuf};

fn temp_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "kivact-task-board-{name}-{}.json",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    path
}

fn open(path: &Path) -> Screen {
    let path = path.to_path_buf();
    let screen = render(move || app(path, None));
    screen.find_by_test_id("task-title");
    screen
}

/// Replace the value of a text input (caret to the end, erase, type).
fn fill(input: Element<'_>, text: &str) {
    let old = input.value().unwrap_or_default().chars().count();
    input.press_key(Key::End);
    for _ in 0..old {
        input.press_key(Key::Backspace);
    }
    if !text.is_empty() {
        input.type_text(text);
    }
}

fn lane_of(path: &Path, id: u64) -> Option<Lane> {
    load(path).ok()?.task(id).map(|task| task.lane)
}

#[test]
fn crud_search_select_and_reopen() {
    let path = temp_path("crud");
    let screen = open(&path);
    assert_eq!(screen.get_by_test_id("board-status").name(), "準備完了");
    assert!(!path.exists(), "loading alone does not write the file");

    // Add
    fill(screen.get_by_role(Role::TextBox, "題名"), "From UI");
    fill(screen.get_by_role(Role::TextBox, "メモ"), "Notes");
    screen.get_by_role(Role::Button, "追加").click();
    screen.find_by_text("保存しました");
    assert_eq!(load(&path).unwrap().task(4).unwrap().title, "From UI");
    assert_eq!(
        screen.get_by_role(Role::TextBox, "題名").value().as_deref(),
        Some(""),
        "the form is cleared"
    );
    let card = screen.get_by_test_id("card-4");
    assert!(card.is_selected(), "a new card is selected");
    assert_eq!(screen.get_by_test_id("lane-planned").text(), "予定 (2件)");

    // Move with the editor, then check local card state kept on the data side
    screen.get_by_test_id("select-4").click();
    screen.get_by_test_id("move-right").click();
    screen.find_by_text("進行中 (2件)");
    screen.find_by_text("保存しました");
    assert_eq!(lane_of(&path, 4), Some(Lane::InProgress));
    screen.get_by_test_id("select-4").click();
    assert_eq!(
        screen.get_by_test_id("card-meta-4").text(),
        "進行中 · 閲覧 2 · 選択中",
        "visits survive the lane move"
    );

    // Search hides and restores cards
    fill(screen.get_by_role(Role::TextBox, "検索"), "absent");
    assert!(screen.query_by_test_id("card-4").is_none());
    assert_eq!(
        screen.get_by_test_id("lane-progress").text(),
        "進行中 (0件)"
    );
    fill(screen.get_by_role(Role::TextBox, "検索"), "from ui");
    assert!(
        screen.query_by_test_id("card-4").is_some(),
        "case-insensitive"
    );
    fill(screen.get_by_role(Role::TextBox, "検索"), "");

    // Edit title, empty title is rejected with an alert
    fill(screen.get_by_role(Role::TextBox, "題名"), "  ");
    screen.get_by_test_id("edit-task").click();
    assert_eq!(
        screen
            .get_by_role(Role::Alert, "題名を入力してください")
            .text(),
        "題名を入力してください"
    );
    fill(screen.get_by_role(Role::TextBox, "題名"), "Edited");
    screen.get_by_test_id("edit-task").click();
    assert!(screen.query_by_test_id("notice").is_none());
    screen.find_by_text("保存しました");
    assert_eq!(load(&path).unwrap().task(4).unwrap().title, "Edited");

    // Select moves the lane; the checkbox completes and reopens
    screen
        .get_by_role(Role::ComboBox, "列")
        .select_option("完了");
    screen.find_by_text("保存しました");
    assert_eq!(lane_of(&path, 4), Some(Lane::Done));
    let done = screen.get_by_test_id("done-4");
    assert_eq!(done.is_checked(), Some(true));
    done.toggle();
    screen.find_by_text("保存しました");
    assert_eq!(lane_of(&path, 4), Some(Lane::InProgress));
    screen.get_by_test_id("done-4").toggle();
    screen.find_by_text("保存しました");
    assert_eq!(lane_of(&path, 4), Some(Lane::Done));
    assert!(screen.request_close(), "the app exits after draining");
    drop(screen);

    // Reopen in a fresh App: the saved board is restored
    let reopened = open(&path);
    let card = reopened.get_by_test_id("card-4");
    assert_eq!(card.name(), "Edited");
    assert_eq!(
        reopened.within(card).get_by_test_id("card-meta-4").text(),
        "完了 · 閲覧 0"
    );
    reopened.get_by_test_id("select-4").click();
    reopened.get_by_role(Role::Button, "削除").click();
    reopened.find_by_text("保存しました");
    assert!(load(&path).unwrap().task(4).is_none());
    assert!(
        reopened.query_by_test_id("task-editor").is_none(),
        "the conditional editor unmounts after deletion"
    );
    assert!(reopened.errors().is_empty(), "{:?}", reopened.errors());
    let _ = std::fs::remove_file(path);
}

#[test]
fn empty_title_is_not_added() {
    let path = temp_path("empty");
    let screen = open(&path);
    screen.get_by_role(Role::Button, "追加").click();
    screen.get_by_role(Role::Alert, "題名を入力してください");
    assert_eq!(screen.get_by_test_id("lane-planned").text(), "予定 (1件)");
    assert!(!path.exists());
}

#[test]
fn card_buttons_move_and_keyboard_activates() {
    let path = temp_path("keys");
    let screen = open(&path);
    let right = screen.get_by_test_id("right-1");
    right.focus();
    right.press_key(Key::Enter);
    screen.find_by_text("進行中 (2件)");
    // The moved card is a new row in the other lane; it takes the focus back.
    assert!(
        screen.get_by_test_id("right-1").is_focused(),
        "focus follows the moved card: {:?}",
        screen.focused().map(|f| f.name())
    );
    screen.press_key(Key::Space);
    screen.find_by_text("完了 (2件)");
    assert!(
        screen.get_by_test_id("right-1").is_disabled(),
        "a done card cannot move right"
    );
    let done = screen.get_by_test_id("done-1");
    assert!(done.is_focused(), "a done card focuses its checkbox");
    // Space on the checkbox reopens it, and the focus stays on the card
    screen.press_key(Key::Space);
    screen.find_by_text("完了 (1件)");
    assert!(screen.get_by_test_id("done-1").is_focused());
    assert_eq!(screen.get_by_test_id("done-1").is_checked(), Some(false));
    screen.find_by_text("保存しました");
    assert_eq!(lane_of(&path, 1), Some(Lane::InProgress));
    let _ = std::fs::remove_file(path);
}

#[test]
fn corrupt_file_reports_and_retries_the_load() {
    let path = temp_path("corrupt");
    std::fs::write(&path, b"{broken board").unwrap();
    let file = path.clone();
    let screen = render(move || app(file, None));
    let error = screen.find_by_test_id("load-error");
    assert!(
        error.text().starts_with("読めません: JSONを読めません"),
        "{}",
        error.text()
    );
    assert!(
        screen.query_by_test_id("task-title").is_none(),
        "no editor on failure"
    );
    // Repair the file and retry
    std::fs::write(
        &path,
        br#"{"next_id":8,"tasks":[{"id":7,"title":"Recovered","notes":"","lane":"Done"}]}"#,
    )
    .unwrap();
    screen.get_by_role(Role::Button, "読込みを再試行").click();
    screen.find_by_text("Recovered");
    assert!(screen.query_by_test_id("load-error").is_none());
    assert_eq!(screen.get_by_test_id("lane-done").text(), "完了 (1件)");
    // The corrupt original was copied aside, never removed
    let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
    for entry in std::fs::read_dir(path.parent().unwrap()).unwrap() {
        let entry = entry.unwrap().path();
        let name = entry.file_name().unwrap().to_string_lossy().into_owned();
        if name.starts_with(&stem) && name.contains("corrupt-") {
            let _ = std::fs::remove_file(entry);
        }
    }
    let _ = std::fs::remove_file(path);
}

#[test]
fn failed_save_is_visible_and_retry_recovers() {
    let directory = std::env::temp_dir().join(format!("kivact-board-save-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    let _ = std::fs::remove_file(&directory);
    // A plain file where the directory should be: every save fails.
    std::fs::write(&directory, "blocks directory creation").unwrap();
    let path = directory.join("board.json");
    let screen = open(&path);
    screen.get_by_test_id("right-1").click();
    let status = screen.find_by_text("進行中 (2件)");
    let _ = status;
    let status = screen.get_by_test_id("board-status");
    assert!(
        status.text().starts_with("保存できません"),
        "{}",
        status.text()
    );
    let retry = screen.get_by_role(Role::Button, "保存を再試行");
    assert!(!retry.is_disabled());
    std::fs::remove_file(&directory).unwrap();
    retry.click();
    screen.find_by_text("保存しました");
    assert!(
        screen
            .get_by_role(Role::Button, "保存を再試行")
            .is_disabled()
    );
    assert_eq!(lane_of(&path, 1), Some(Lane::InProgress));
    let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn rapid_changes_coalesce_and_the_newest_board_is_saved() {
    let path = temp_path("coalesce");
    let screen = open(&path);
    for index in 0..20 {
        fill(
            screen.get_by_role(Role::TextBox, "題名"),
            &format!("task {index}"),
        );
        screen.get_by_role(Role::Button, "追加").click();
    }
    screen.find_by_text("保存しました");
    let saved = load(&path).unwrap();
    assert_eq!(saved.len(), 23);
    assert_eq!(saved.task(23).unwrap().title, "task 19");
    let _ = std::fs::remove_file(path);
}

#[test]
fn large_board_scrolls_a_virtual_window() {
    let path = temp_path("virtual");
    let file = path.clone();
    let screen = render(move || app(file, Some(20_000)));
    screen.find_by_text("予定 (20000件)");
    assert!(screen.query_by_test_id("card-1").is_some());
    assert!(screen.query_by_test_id("card-5000").is_none());
    let list = screen.get_by_test_id("list-planned");
    list.scroll(0.0, 132.0 * 4_999.0);
    assert!(screen.query_by_test_id("card-5000").is_some());
    assert!(
        screen.query_by_test_id("card-1").is_none(),
        "rows outside the window are disposed"
    );
    assert!(screen.stats().tree_nodes < 1_000);
    assert!(
        !path.exists(),
        "the generated board is not saved until edited"
    );
}
