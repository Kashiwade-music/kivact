//! Role: The game's visible source states, exercised through the public consumer driver.
use std::time::Duration;

use kivact::draw::Point;
use kivact::test::{Element, Screen};
use kivact::view::{A11yAction, Key, Modifiers, MouseButton, Role, SortDirection};

use super::{evidence, mount};

fn chapter(screen: &Screen, name: &str) {
    screen.get_by_test_id(&format!("game-nav-{name}")).click();
    screen.advance(Duration::from_millis(400));
    assert!(
        screen
            .get_by_test_id(&format!("game-nav-{name}"))
            .is_selected()
    );
    screen.assert_layout_with(&crate::layout_check());
}

fn clean(screen: &Screen) {
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn fill(input: Element<'_>, value: &str) {
    input.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    input.type_text(value);
}

#[test]
fn game_toast_is_live_pauses_for_the_pointer_and_expires() {
    let screen = mount();
    chapter(&screen, "buttons");
    screen.get_by_test_id("game-primary").click();
    assert!(screen.get_by_role(Role::Status, "承認").exists());
    screen.get_by_text("案件を承認しました。").hover();
    screen.advance(Duration::from_secs(20));
    assert!(
        screen.query_by_role(Role::Status, "承認").is_some(),
        "held while the pointer is over it"
    );
    screen.mouse_leave();
    screen.advance(Duration::from_secs(3));
    assert!(screen.query_by_role(Role::Status, "承認").is_none());
    clean(&screen);
}

#[test]
fn game_dark_compact_and_real_button_states() {
    let screen = mount();
    screen.get_by_test_id("game-theme-toggle").click();
    screen.get_by_test_id("game-density-compact").click();
    assert!(screen.get_by_test_id("game-density-compact").is_selected());
    assert!(screen.get_by_text("DARK").exists());
    chapter(&screen, "buttons");
    let primary = screen.get_by_test_id("game-primary");
    assert_eq!(primary.rect().map(|rect| rect.height), Some(28.));
    evidence(&screen, "game-dark-compact-buttons");
    primary.hover();
    let hover = screen.screenshot().hash();
    evidence(&screen, "game-button-hover");
    screen.mouse_down(MouseButton::Primary);
    assert_ne!(hover, screen.screenshot().hash());
    evidence(&screen, "game-button-pressed");
    screen.mouse_up(MouseButton::Primary);
    primary.focus();
    assert!(primary.is_focused());
    assert!(screen.get_by_test_id("game-disabled").is_disabled());
    evidence(&screen, "game-button-focus-disabled");
    let busy = screen.get_by_test_id("game-busy");
    busy.click();
    assert!(busy.is_disabled());
    evidence(&screen, "game-button-busy");
    screen.advance(Duration::from_millis(1400));
    assert!(!busy.is_disabled());
    let filter = screen.get_by_test_id("game-toolbar-filter");
    assert!(!filter.is_selected());
    filter.click();
    assert!(filter.is_selected());
    filter.click();
    assert!(!filter.is_selected());
    clean(&screen);
}

#[test]
fn game_input_mixed_selection_and_pointer_slider_change_real_values() {
    let screen = mount();
    chapter(&screen, "inputs");
    let input = screen.get_by_test_id("game-title-input");
    fill(input, "第 4 区画 点検");
    assert_eq!(input.value().as_deref(), Some("第 4 区画 点検"));
    let code = screen.get_by_role(Role::TextBox, "管理コード");
    fill(code, "OPS-2041");
    assert!(!code.invalid());
    code.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    code.press_key(Key::Backspace);
    assert!(code.invalid());
    evidence(&screen, "game-input-focus");
    chapter(&screen, "selection");
    let all = screen.get_by_role(Role::CheckBox, "全権限");
    assert!(all.is_indeterminate());
    all.click();
    assert_eq!(all.is_checked(), Some(true));
    assert!(!all.is_indeterminate());
    screen.get_by_role(Role::CheckBox, "承認").click();
    assert!(all.is_indeterminate());
    let track = screen.get_by_test_id("game-cpu-slider");
    let rect = track.rect().expect("visible slider bounds");
    track.drag(
        Point::new(rect.width * 0.35, rect.height / 2.),
        Point::new(rect.width * 0.8, rect.height / 2.),
    );
    let slider = screen.get_by_role(Role::Slider, "CPU 上限");
    let value = slider.value_range().2.expect("slider exposes its value");
    assert!((70. ..=90.).contains(&value), "dragged CPU value {value}");
    slider.a11y_with_value(A11yAction::SetValue, "37.5");
    assert_eq!(slider.value_range().2, Some(37.5));
    evidence(&screen, "game-selection-mixed-slider");
    clean(&screen);
}

#[test]
fn game_table_sort_filter_bulk_and_drawer_preserve_record_keys() {
    let screen = mount();
    chapter(&screen, "table");
    let page_size = screen.get_by_test_id("game-table-page-size");
    page_size.press_key(Key::ArrowDown);
    assert_eq!(page_size.value().as_deref(), Some("50"));
    let all = screen.get_by_role(Role::CheckBox, "表示行をすべて選択");
    screen
        .get_by_role(Role::CheckBox, "OPS-2041 を選択")
        .click();
    assert!(all.is_indeterminate());
    assert!(screen.get_by_role(Role::Row, "OPS-2041").is_selected());
    evidence(&screen, "game-table-partial-bulk");
    let progress = screen.get_by_role(Role::ColumnHeader, "進捗");
    progress.click();
    assert_eq!(progress.sort(), SortDirection::Ascending);
    let zero = screen
        .get_by_role(Role::Row, "OPS-2036")
        .rect()
        .expect("zero row")
        .y;
    let twenty = screen
        .get_by_role(Role::Row, "OPS-2039")
        .rect()
        .expect("twenty row")
        .y;
    let hundred = screen
        .get_by_role(Role::Row, "OPS-2040")
        .rect()
        .expect("hundred row")
        .y;
    assert!(
        zero < twenty && twenty < hundred,
        "numeric progress ordering"
    );
    fill(screen.get_by_test_id("game-table-search"), "OPS-2041");
    assert!(screen.query_by_role(Role::Row, "OPS-2036").is_none());
    assert!(screen.get_by_role(Role::Row, "OPS-2041").is_selected());
    screen.get_by_test_id("game-bulk-approve").click();
    let row = screen.get_by_role(Role::Row, "OPS-2041");
    row.get_by_text("PASSED");
    row.get_by_text("100%");
    evidence(&screen, "game-table-filter-approved");
    screen.mouse_leave();
    let details = screen.get_by_role(Role::Button, "OPS-2041 の詳細");
    let bounds = details.rect().expect("detail action geometry");
    let crop = || {
        screen
            .screenshot()
            .crop(
                bounds.x as u32,
                bounds.y as u32,
                bounds.width as u32,
                bounds.height as u32,
            )
            .hash()
    };
    let hidden = crop();
    row.hover();
    assert_ne!(hidden, crop(), "the row's hover reveals its action");
    screen.mouse_leave();
    screen.press_key(Key::Tab);
    details.focus();
    assert!(details.is_focused());
    assert_ne!(
        hidden,
        crop(),
        "keyboard focus reveals the action and its focus ring"
    );
    details.click();
    assert!(screen.get_by_role(Role::Dialog, "案件詳細").exists());
    evidence(&screen, "game-table-row-drawer");
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Dialog, "案件詳細").is_none());
    clean(&screen);
}

#[test]
fn game_tooltip_dialog_drawer_banner_and_mission_preview() {
    let screen = mount();
    chapter(&screen, "overlays");
    screen.get_by_test_id("game-tooltip-save").focus();
    screen.advance(Duration::from_millis(500));
    assert!(
        screen
            .get_by_role(Role::Tooltip, "上書き保存  Ctrl+S")
            .exists()
    );
    evidence(&screen, "game-tooltip-focus");
    screen.get_by_test_id("game-create-open").click();
    let dialog = screen.get_by_role(Role::Dialog, "新規案件");
    fill(dialog.get_by_role(Role::TextBox, "件名 *"), "点検準備");
    evidence(&screen, "game-dialog-input");
    dialog.get_by_role(Role::Button, "作成").click();
    assert!(screen.query_by_role(Role::Dialog, "新規案件").is_none());
    screen.get_by_text("案件を作成しました");
    screen.get_by_test_id("game-drawer-open").click();
    evidence(&screen, "game-drawer");
    screen.get_by_test_id("game-drawer-footer-edit").click();
    assert!(screen.query_by_role(Role::Dialog, "案件詳細").is_none());
    screen.get_by_test_id("game-drawer-open").click();
    screen.get_by_test_id("game-drawer-footer-close").click();
    assert!(screen.query_by_role(Role::Dialog, "案件詳細").is_none());
    screen.get_by_test_id("game-drawer-open").click();
    screen.press_key(Key::Escape);
    chapter(&screen, "feedback");
    screen.get_by_test_id("game-banner-details").click();
    screen.get_by_text("計画メンテナンス");
    evidence(&screen, "game-banner-action");
    chapter(&screen, "layout");
    screen.get_by_role(Role::TreeItem, "OPS-2041.md").click();
    assert!(
        screen
            .get_by_role(Role::TreeItem, "OPS-2041.md")
            .is_selected()
    );
    assert_eq!(
        screen.get_by_test_id("game-document-title").text(),
        "OPS-2041.md"
    );
    screen.get_by_role(Role::TreeItem, "OPS-2042.md").click();
    assert_eq!(
        screen.get_by_test_id("game-document-title").text(),
        "OPS-2042.md"
    );
    assert_eq!(
        screen
            .get_by_role(Role::Slider, "Inspector 不透明度")
            .value_range()
            .2,
        Some(90.)
    );
    let preview = screen.get_by_role(Role::RadioButton, "プレビュー");
    preview.click();
    assert!(preview.is_selected());
    screen.get_by_role(Role::Button, "基本").click();
    assert_eq!(
        screen.get_by_role(Role::Button, "基本").is_expanded(),
        Some(false)
    );
    evidence(&screen, "game-mission-preview");
    clean(&screen);
}

#[test]
fn game_original_token_copy_search_shortcut_and_filtered_click_identity() {
    let screen = mount();
    screen.get_by_test_id("game-swatch---c-primary").click();
    assert_eq!(screen.clipboard().as_deref(), Some("var(--c-primary)"));
    screen.get_by_test_id("game-theme-toggle").click();
    assert_eq!(
        screen.get_by_test_id("game-swatch---c-primary").name(),
        "#2B8CEB"
    );
    screen.press_key_with(
        Key::Char('k'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    let query = screen.get_by_role(Role::TextBox, "コマンド検索");
    query.type_text("ボタン");
    let command = screen.get_by_role(Role::Option, "ボタン");
    assert!(command.is_selected());
    assert!(screen.query_by_role(Role::Option, "基礎トークン").is_none());
    command.click();
    screen.advance(Duration::from_millis(400));
    assert!(screen.get_by_test_id("game-nav-buttons").is_selected());
    screen.get_by_test_id("game-primary").focus();
    screen.press_key(Key::Char('/'));
    let search = screen.get_by_test_id("game-search-input");
    assert!(search.is_focused());
    screen.press_key(Key::Char('/'));
    assert!(search.is_focused());
    assert_eq!(search.value().as_deref(), Some("/"));
    screen.get_by_role(Role::TextBox, "API キー ID").focus();
    screen.press_key(Key::Char('/'));
    assert!(
        screen
            .get_by_role(Role::TextBox, "API キー ID")
            .is_focused()
    );
    chapter(&screen, "selection");
    let checkbox = screen.get_by_role(Role::CheckBox, "全権限");
    checkbox.focus();
    screen.press_key(Key::Char('/'));
    assert!(checkbox.is_focused());
    let slider = screen.get_by_role(Role::Slider, "CPU 上限");
    slider.focus();
    screen.press_key(Key::Char('/'));
    assert!(slider.is_focused());
    let select = screen.get_by_role(Role::ComboBox, "担当部署");
    select.focus();
    screen.press_key(Key::Char('/'));
    assert!(select.is_focused());
    chapter(&screen, "table");
    let select_row = screen.get_by_role(Role::CheckBox, "OPS-2041 を選択");
    select_row.focus();
    screen.press_key(Key::Char('/'));
    assert!(select_row.is_focused());
    evidence(&screen, "game-keyboard-search-and-filtered-command");
    clean(&screen);
}

#[test]
fn game_token_swatches_are_buttons_named_by_their_hex_label() {
    let screen = mount();
    let swatch = screen.get_by_test_id("game-swatch---c-primary");
    assert_eq!(swatch.name(), "#1283F0");
    screen.get_by_role(Role::Button, "#1283F0");
    screen.get_by_test_id("game-theme-toggle").click();
    assert_eq!(swatch.name(), "#2B8CEB");
    clean(&screen);
}

#[test]
fn game_window_frame_binds_the_explorer_width_to_its_splitter() {
    let screen = mount();
    chapter(&screen, "layout");
    let frame = screen.get_by_test_id("LAY-01");
    let splitter = frame.get_by_role(Role::Splitter, "エクスプローラー幅");
    assert_eq!(splitter.value_range(), (Some(140.), Some(420.), Some(240.)));
    splitter.a11y(A11yAction::Increment);
    let widened = splitter.value_range().2.expect("a current width");
    assert!(widened > 240., "{widened}");
    splitter.press_key(Key::End);
    assert_eq!(splitter.value_range().2, Some(420.));
    splitter.press_key(Key::Home);
    assert_eq!(splitter.value_range().2, Some(140.));
    clean(&screen);
}

#[test]
fn game_same_named_file_uploads_advance_independently() {
    let screen = mount();
    chapter(&screen, "inputs");
    screen.respond_files_dialog(Some(vec![
        std::path::PathBuf::from("folder-a/report.csv"),
        std::path::PathBuf::from("folder-b/report.csv"),
    ]));
    screen
        .get_by_role(Role::Button, "ファイルをアップロード")
        .click();
    screen.run_until_idle();
    screen.advance(Duration::from_millis(220));
    assert_eq!(screen.get_by_test_id("game-file-progress-1").text(), "10%");
    assert_eq!(screen.get_by_test_id("game-file-progress-2").text(), "10%");
    for _ in 0..9 {
        screen.advance(Duration::from_millis(220));
    }
    assert_eq!(screen.get_by_test_id("game-file-progress-1").text(), "100%");
    assert_eq!(screen.get_by_test_id("game-file-progress-2").text(), "100%");
    evidence(&screen, "game-independent-file-upload-progress");
    clean(&screen);
}

#[test]
fn game_window_chrome_owns_commands_editor_mode_and_saved_content() {
    let screen = mount();
    chapter(&screen, "layout");
    let frame = screen.get_by_test_id("LAY-01");
    frame.get_by_role(Role::Button, "ファイル").click();
    screen.get_by_role(Role::MenuItem, "新規").click();
    assert_eq!(
        screen.get_by_test_id("game-window-status").text(),
        "New document"
    );
    let source = frame.get_by_role(Role::TextBox, "Document source");
    fill(source, "Catalog source text");
    assert_eq!(
        screen.get_by_test_id("game-window-save-state").text(),
        "Modified"
    );
    frame.get_by_role(Role::Button, "保存").click();
    assert_eq!(
        screen.get_by_test_id("game-window-save-state").text(),
        "Saved"
    );
    assert_eq!(
        screen.get_by_test_id("game-window-status").text(),
        "Saved untitled.md"
    );
    frame.get_by_role(Role::RadioButton, "プレビュー").click();
    assert_eq!(
        screen.get_by_test_id("game-window-preview-source").text(),
        "Catalog source text"
    );
    frame.get_by_role(Role::Button, "実行").click();
    assert_eq!(
        screen.get_by_test_id("game-window-status").text(),
        "Running"
    );
    frame.get_by_role(Role::Button, "一時停止").click();
    assert_eq!(screen.get_by_test_id("game-window-status").text(), "Paused");
    evidence(&screen, "game-window-application-chrome");
    chapter(&screen, "display");
    screen.get_by_text("1,284");
    screen.get_by_text("99.97");
    screen.get_by_text("4.2");
    evidence(&screen, "game-kpi-application-trends-and-format");
    chapter(&screen, "inputs");
    screen.get_by_text("または クリックして選択 · CSV / JSON · 最大 20MB");
    screen.respond_files_dialog(Some(vec![std::path::PathBuf::from("application.json")]));
    screen
        .get_by_role(Role::Button, "ファイルをアップロード")
        .click();
    screen.run_until_idle();
    screen.get_by_text("application.json");
    evidence(&screen, "game-dropzone-application-guidance");
    clean(&screen);
}

#[test]
fn game_tags_and_alerts_apply_application_requests() {
    let screen = mount();
    chapter(&screen, "inputs");
    let input = screen.get_by_role(Role::TextBox, "ラベル");
    input.type_text("catalog-tag");
    input.press_key(Key::Enter);
    assert!(
        screen
            .get_by_role(Role::Button, "catalog-tag を削除")
            .exists()
    );
    input.press_key(Key::Backspace);
    assert!(
        screen
            .query_by_role(Role::Button, "catalog-tag を削除")
            .is_none()
    );
    input.type_text("evidence-tag");
    input.press_key(Key::Enter);
    evidence(&screen, "game-tags-application-requests");
    chapter(&screen, "feedback");
    assert_eq!(
        screen.all_by_role(Role::Button, "アラートを閉じる").len(),
        4
    );
    screen.all_by_role(Role::Button, "アラートを閉じる")[0].click();
    assert_eq!(
        screen.all_by_role(Role::Button, "アラートを閉じる").len(),
        3
    );
    evidence(&screen, "game-alert-application-dismissal");
    chapter(&screen, "layout");
    screen.get_by_role(Role::TreeItem, "OPS-2042.md").click();
    assert_eq!(
        screen.get_by_test_id("game-document-title").text(),
        "OPS-2042.md"
    );
    evidence(&screen, "game-tree-application-selection");
    clean(&screen);
}
