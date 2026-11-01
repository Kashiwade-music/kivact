//! Role: Public-driver state checks for the complete Pop catalogue.
use super::{evidence, mount};
use kivact::draw::{Point, Rect};
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::{Modifiers, MouseButton};

use std::time::Duration;

fn chapter(screen: &Screen, name: &str) {
    let nav = screen.get_by_test_id(&format!("pop-nav-{name}"));
    nav.click();
    screen.advance(Duration::from_millis(400));
    assert!(nav.is_selected());
    screen.assert_layout();
}

fn replace(screen: &Screen, label: &str, value: &str) {
    let input = screen.get_by_role(Role::TextBox, label);
    input.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    input.type_text(value);
    assert_eq!(input.value(), Some(value.to_string()));
}

#[test]
fn pop_forms_edit_drag_choose_date_and_keep_state_in_dark_theme() {
    let screen = mount();
    chapter(&screen, "forms");
    evidence(&screen, "pop-state-forms-light");
    let email = screen.get_by_role(Role::TextBox, "メールアドレス");
    assert!(email.invalid());
    assert_eq!(
        email
            .described_by()
            .iter()
            .map(|e| e.text())
            .collect::<Vec<_>>(),
        ["有効なメールアドレスを入力してください。"]
    );
    replace(&screen, "生徒名 *", "空崎 ミナト・調査");
    let memo = screen.get_by_role(Role::TextBox, "依頼メモ");
    memo.commit("現地の安全を確認してください。");
    assert_eq!(
        screen.get_by_role(Role::TextBox, "依頼メモ").value(),
        Some("現地の安全を確認してください。".into())
    );
    let initial_memo = memo.rect().expect("memo laid out");
    assert_eq!(initial_memo.height, 80.0);
    let resize = |delta: f32| {
        let grip = screen
            .get_by_test_id("pop-memo-resize")
            .get_by_role(Role::Splitter, "テキスト欄の高さを変更");
        grip.hover();
        let rect = grip.rect().expect("resize grip laid out");
        let x = rect.x + rect.width / 2.0;
        let y = rect.y + rect.height / 2.0;
        screen.mouse_move(x, y);
        screen.mouse_down(MouseButton::Primary);
        screen.mouse_move(x, y + delta / 2.0);
        screen.mouse_move(x, y + delta);
        screen.mouse_up(MouseButton::Primary);
    };
    resize(48.0);
    assert_eq!(memo.rect().unwrap().height, 128.0);
    assert_eq!(memo.rect().unwrap().width, initial_memo.width);
    assert!(memo.is_focused());
    assert_eq!(memo.value(), Some("現地の安全を確認してください。".into()));
    screen.type_text("継続入力");
    assert_eq!(
        memo.value(),
        Some("現地の安全を確認してください。継続入力".into())
    );
    evidence(&screen, "pop-state-textarea-expanded");
    resize(-200.0);
    assert_eq!(memo.rect().unwrap().height, 72.0);
    assert_eq!(memo.rect().unwrap().width, initial_memo.width);
    assert!(memo.is_focused());
    assert_eq!(
        memo.value(),
        Some("現地の安全を確認してください。継続入力".into())
    );
    evidence(&screen, "pop-state-textarea-minimum");
    let tags = screen.get_by_role(Role::TextBox, "タグを追加");
    tags.type_text("連携");
    tags.press_key(Key::Enter);
    screen.get_by_role(Role::Button, "連携 を削除");
    screen.get_by_role(Role::CheckBox, "通知を受け取る").click();
    assert_eq!(
        screen
            .get_by_role(Role::CheckBox, "通知を受け取る")
            .is_checked(),
        Some(false)
    );
    assert!(
        screen
            .get_by_role(Role::CheckBox, "部分選択（indeterminate）")
            .is_indeterminate()
    );
    let volume = screen.get_by_role(Role::Slider, "BGM音量");
    let track = screen.get_by_test_id("pop-volume");
    let width = track.rect().expect("slider laid out").width;
    track.drag(
        Point::new(width * 0.64, 10.0),
        Point::new(width * 0.82, 10.0),
    );
    let (_, _, now) = volume.value_range();
    assert!(now.is_some_and(|n| n > 75.0 && n < 90.0));
    screen.get_by_test_id("pop-password-reveal").click();
    evidence(&screen, "pop-state-password-revealed");
    screen.get_by_test_id("pop-password-reveal").click();
    screen.get_by_role(Role::Button, "日付を選択").click();
    evidence(&screen, "pop-state-date-overlay");
    screen
        .get_by_test_id("pop-date-input")
        .get_by_role(Role::GridCell, "2026-10-17")
        .click();
    assert_eq!(
        screen.get_by_role(Role::TextBox, "実施日").value(),
        Some("2026-10-17".into())
    );
    screen.get_by_test_id("pop-theme-toggle").click();
    assert_eq!(
        screen.get_by_role(Role::TextBox, "生徒名 *").value(),
        Some("空崎 ミナト・調査".into())
    );
    assert_eq!(
        screen.get_by_role(Role::TextBox, "実施日").value(),
        Some("2026-10-17".into())
    );
    evidence(&screen, "pop-state-forms-dark");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// The standard components of `kivact::ui` take the Halo look from the recipes
/// the kit's theme installs: its control height and its colors, not the
/// core's defaults (28 px, `#2563eb`).
#[test]
fn standard_components_take_the_look_of_the_kit_theme() {
    let screen = mount();
    chapter(&screen, "buttons");
    let standard = screen.get_by_role(Role::Button, "標準ボタン");
    let kit = screen.get_by_role(Role::Button, "プライマリ");
    let (standard_box, kit_box) = (
        standard.rect().expect("laid out"),
        kit.rect().expect("laid out"),
    );
    assert_eq!(standard_box.height, 36.0, "the kit's control height");
    assert_eq!(standard_box.height, kit_box.height);
    // The first pixels inside the rounded left edge are the kit's blue
    // gradient, not the core's accent.
    let shot = screen.screenshot();
    let at = |r: Rect| {
        shot.pixel((r.x + 12.0) as u32, (r.y + r.height / 2.0) as u32)
            .expect("in the frame")
    };
    assert_eq!(
        at(standard_box),
        at(kit_box),
        "the same fill as the kit's button"
    );
    assert_ne!(at(standard_box), 0x2563eb, "not the core default");
    let check = screen.get_by_role(Role::CheckBox, "標準チェック");
    assert_eq!(check.is_checked(), Some(true));
    check.click();
    assert_eq!(check.is_checked(), Some(false));
    let switch = screen.get_by_role(Role::Switch, "標準スイッチ");
    switch.press_key(Key::Space);
    assert_eq!(switch.is_checked(), Some(false));
    assert!(screen.get_by_role(Role::Button, "標準無効").is_disabled());
    let input = screen.get_by_role(Role::TextBox, "標準入力");
    assert_eq!(input.value().as_deref(), Some("標準の入力欄"));
    assert_eq!(input.rect().expect("laid out").height, 36.0);
    evidence(&screen, "pop-standard-components");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pop_button_notifications_pause_under_the_pointer_and_expire() {
    let screen = mount();
    chapter(&screen, "buttons");
    screen.get_by_role(Role::Button, "アクセント").click();
    let toast = screen.get_by_role(Role::Status, "アクション");
    assert!(toast.exists());
    toast.hover();
    screen.advance(Duration::from_secs(20));
    assert!(
        screen.query_by_role(Role::Status, "アクション").is_some(),
        "held while the pointer is over it"
    );
    screen.mouse_leave();
    screen.advance(Duration::from_secs(5));
    assert!(screen.query_by_role(Role::Status, "アクション").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pop_button_states_and_table_partial_selection_are_observable() {
    let screen = mount();
    chapter(&screen, "buttons");
    let button = screen.get_by_role(Role::Button, "プライマリ");
    button.hover();
    screen.advance(Duration::from_millis(160));
    evidence(&screen, "pop-state-button-hover");
    screen.mouse_down(MouseButton::Primary);
    evidence(&screen, "pop-state-button-pressed");
    screen.mouse_up(MouseButton::Primary);
    button.focus();
    assert!(button.is_focused());
    assert!(screen.get_by_role(Role::Button, "無効").is_disabled());
    evidence(&screen, "pop-state-button-focus-disabled");
    chapter(&screen, "table");
    let first_page = screen.get_by_role(Role::Button, "1");
    assert!(first_page.is_selected());
    screen.get_by_role(Role::Button, "2").click();
    assert!(screen.get_by_role(Role::Button, "2").is_selected());
    first_page.click();
    assert!(first_page.is_selected());
    screen
        .get_by_role(Role::CheckBox, "空崎 ミナト を選択")
        .click();
    assert!(
        screen
            .get_by_role(Role::CheckBox, "表示中の行を選択")
            .is_indeterminate()
    );
    evidence(&screen, "pop-state-table-partial");
    replace(&screen, "テーブルを検索", "ミナト");
    screen.get_by_text("全 1 件中 1–1 件を表示");
    assert!(
        screen
            .query_by_role(Role::CheckBox, "春日 ハルカ を選択")
            .is_none()
    );
    evidence(&screen, "pop-state-table-search");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pop_modals_drawer_calendar_and_conversation_use_live_models() {
    let screen = mount();
    chapter(&screen, "overlays");
    let opener = screen.get_by_role(Role::Button, "確認ダイアログ");
    opener.click();
    screen.get_by_role(Role::Dialog, "作戦を開始しますか？");
    screen
        .get_by_role(Role::CheckBox, "今後この確認を表示しない")
        .click();
    assert_eq!(
        screen
            .get_by_role(Role::CheckBox, "今後この確認を表示しない")
            .is_checked(),
        Some(true)
    );
    evidence(&screen, "pop-state-confirm-modal");
    screen.press_key(Key::Escape);
    assert!(opener.is_focused());
    assert!(
        screen
            .query_by_role(Role::Dialog, "作戦を開始しますか？")
            .is_none()
    );
    screen
        .get_by_role(Role::Button, "フォームダイアログ")
        .click();
    assert!(screen.get_by_test_id("pop-create-request").is_disabled());
    screen
        .get_by_test_id("pop-request-title")
        .type_text("中央区の巡回");
    assert!(!screen.get_by_test_id("pop-create-request").is_disabled());
    evidence(&screen, "pop-state-form-modal");
    screen.get_by_test_id("pop-create-request").click();
    assert!(
        screen
            .query_by_role(Role::Dialog, "新しい依頼を作成")
            .is_none()
    );
    screen.get_by_role(Role::Button, "ドロワーを開く").click();
    screen.get_by_role(Role::Dialog, "空崎 ミナト");
    evidence(&screen, "pop-state-profile-drawer");
    screen.get_by_test_id("pop-drawer-save").click();
    assert!(screen.query_by_role(Role::Dialog, "空崎 ミナト").is_none());
    chapter(&screen, "schedule");
    screen.get_by_role(Role::GridCell, "2026-10-10").click();
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-10")
            .is_selected()
    );
    screen.get_by_text("10月10日（土）の予定");
    evidence(&screen, "pop-state-calendar-selection");
    chapter(&screen, "messaging");
    let message = screen.get_by_role(Role::TextBox, "メッセージを入力");
    message.commit("了解しました。安全第一でお願いします。");
    message.press_key(Key::Enter);
    assert_eq!(message.value(), Some(String::new()));
    screen.get_by_text("了解しました。安全第一でお願いします。");
    assert!(screen.query_by_test_id("pop-reply-card").is_none());
    assert!(screen.query_by_role(Role::Status, "入力中").is_none());
    evidence(&screen, "pop-state-message-sent");
    screen.advance(Duration::from_millis(410));
    screen.get_by_role(Role::Status, "入力中");
    evidence(&screen, "pop-state-message-typing");
    screen.advance(Duration::from_millis(1310));
    assert!(screen.query_by_role(Role::Status, "入力中").is_none());
    screen.get_by_text("わかりました！すぐに確認しますね。");
    evidence(&screen, "pop-state-message-replied");
    let thread = screen.get_by_test_id("pop-thread-1");
    thread.get_by_text("1");
    thread.click();
    assert!(thread.is_selected());
    assert!(screen.within(thread).query_by_text("1").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pop_source_chips_step_completion_and_notification_read_state() {
    let screen = mount();
    chapter(&screen, "display");
    let chips = screen.get_by_test_id("pop-chip-list");
    chips.get_by_role(Role::Button, "追加").click();
    chips.get_by_role(Role::Button, "振動 を削除").click();
    assert!(
        screen
            .within(chips)
            .query_by_role(Role::Button, "振動 を削除")
            .is_none()
    );
    chapter(&screen, "navigation");
    let next = screen.get_by_test_id("pop-step-next");
    next.click();
    assert!(!next.is_disabled());
    next.click();
    assert!(next.is_disabled());
    screen.get_by_text("すべての工程が完了しました");
    evidence(&screen, "pop-state-steps-complete");
    screen.get_by_test_id("pop-step-prev").click();
    assert!(!next.is_disabled());
    let notifications = screen.get_by_test_id("pop-notifications");
    notifications.click();
    screen.get_by_role(Role::Group, "総力戦シーズン開始");
    assert!(
        screen
            .get_by_role(Role::Group, "ミナトから新着メッセージ")
            .is_selected()
    );
    evidence(&screen, "pop-state-notifications-unread");
    screen.get_by_role(Role::Button, "すべて既読").click();
    assert!(
        !screen
            .get_by_role(Role::Group, "ミナトから新着メッセージ")
            .is_selected()
    );
    assert_eq!(notifications.is_expanded(), Some(true));
    assert!(
        screen
            .query_by_role(Role::Status, "未読のお知らせ")
            .is_none()
    );
    evidence(&screen, "pop-state-notifications-read");
    screen.press_key(Key::Escape);
    assert_eq!(notifications.is_expanded(), Some(false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pop_source_commands_request_enter_and_row_specific_drawer_are_connected() {
    let screen = mount();
    let shortcut = Modifiers {
        ctrl: true,
        ..Default::default()
    };
    screen.get_by_test_id("pop-search-trigger").focus();
    screen.press_key_with(Key::Char('k'), shortcut);
    screen.get_by_role(Role::Dialog, "コマンドパレット");
    screen.get_by_role(Role::Option, "報酬を受け取る");
    screen.press_key_with(Key::Char('k'), shortcut);
    assert!(
        screen
            .query_by_role(Role::Dialog, "コマンドパレット")
            .is_none()
    );
    screen.get_by_test_id("pop-search-trigger").click();
    screen
        .get_by_role(Role::TextBox, "コマンドを検索")
        .type_text("ダイアログ");
    screen
        .get_by_role(Role::TextBox, "コマンドを検索")
        .press_key(Key::Enter);
    screen.get_by_role(Role::Dialog, "新しい依頼を作成");
    screen
        .get_by_test_id("pop-request-title")
        .type_text("Enterから作成");
    screen
        .get_by_test_id("pop-request-title")
        .press_key(Key::Enter);
    assert!(
        screen
            .query_by_role(Role::Dialog, "新しい依頼を作成")
            .is_none()
    );
    screen.get_by_text("依頼を作成しました");
    chapter(&screen, "table");
    screen.get_by_test_id("pop-student-detail-1").click();
    screen.get_by_role(Role::Dialog, "春日 ハルカ");
    evidence(&screen, "pop-state-row-specific-drawer");
    screen.press_key(Key::Escape);
    screen.get_by_role(Role::Button, "›").click();
    screen.get_by_role(Role::Button, "›").click();
    screen.get_by_text("全 24 件中 13–18 件を表示");
    screen.get_by_role(Role::RadioButton, "待機").click();
    screen.get_by_text("全 10 件中 1–6 件を表示");
    chapter(&screen, "overlays");
    screen.get_by_role(Role::Button, "ポップオーバー").click();
    screen.get_by_text("ヘイローとは？");
    screen.get_by_role(Role::Button, "わかった").click();
    assert!(screen.query_by_text("ヘイローとは？").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pop_source_file_drop_and_multiple_picker_prepend_real_sizes_and_finish() {
    let screen = mount();
    chapter(&screen, "forms");
    let prefix = format!("kivact-pop-upload-{}", std::process::id());
    let report = std::env::temp_dir().join(format!("{prefix}-調査.txt"));
    let large = std::env::temp_dir().join(format!("{prefix}-map.pdf"));
    std::fs::write(&report, vec![b'x'; 3072]).expect("dummy upload metadata");
    std::fs::write(&large, vec![b'x'; 2_097_152]).expect("dummy upload metadata");
    let zone = screen.get_by_role(Role::Button, "ここにファイルをドロップ");
    let rect = zone.rect().expect("dropzone laid out");
    let point = Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.file_drag(kivact::view::FilePhase::Enter, vec![report.clone()], point);
    evidence(&screen, "pop-state-file-dragover");
    screen.file_drag(kivact::view::FilePhase::Drop, vec![report.clone()], point);
    let first = screen.get_by_test_id("pop-upload-1");
    first.get_by_text("3KB");
    first.get_by_role(Role::Status, "アップロード中");
    assert!(
        first.rect().expect("new file laid out").y
            < screen
                .get_by_test_id("pop-upload-0")
                .rect()
                .expect("initial file laid out")
                .y
    );
    screen.advance(Duration::from_millis(180));
    evidence(&screen, "pop-state-file-upload-progress");
    for _ in 0..5 {
        screen.advance(Duration::from_millis(180));
    }
    first.get_by_role(Role::Status, "アップロード完了");
    screen.respond_files_dialog(Some(vec![report.clone(), large.clone()]));
    zone.press_key(Key::Enter);
    let second = screen.get_by_test_id("pop-upload-2");
    let third = screen.get_by_test_id("pop-upload-3");
    second.get_by_text("3KB");
    third.get_by_text("2.0MB");
    assert!(
        third.rect().expect("third file laid out").y
            < second.rect().expect("second file laid out").y
    );
    assert!(
        second.rect().expect("second file laid out").y
            < first.rect().expect("first file laid out").y
    );
    for _ in 0..6 {
        screen.advance(Duration::from_millis(180));
    }
    second.get_by_role(Role::Status, "アップロード完了");
    third.get_by_role(Role::Status, "アップロード完了");
    evidence(&screen, "pop-state-file-uploads-complete");
    std::fs::remove_file(report).expect("remove only the created dummy file");
    std::fs::remove_file(large).expect("remove only the created dummy file");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pop_table_keeps_selection_across_bound_pages_and_numeric_sort() {
    let screen = mount();
    chapter(&screen, "table");
    let first_page = screen.get_by_role(Role::Button, "1");
    assert!(first_page.is_selected());
    screen.get_by_role(Role::Button, "2").click();
    assert!(screen.get_by_role(Role::Button, "2").is_selected());
    first_page.click();
    assert!(first_page.is_selected());
    screen
        .get_by_role(Role::CheckBox, "空崎 ミナト を選択")
        .click();
    screen.get_by_role(Role::Button, "›").click();
    assert!(
        screen
            .query_by_role(Role::CheckBox, "空崎 ミナト を選択")
            .is_none()
    );
    screen
        .get_by_role(Role::CheckBox, "鷹宮 レイ を選択")
        .click();
    evidence(&screen, "pop-table-application-page-selection");
    screen.get_by_role(Role::Button, "‹").click();
    assert_eq!(
        screen
            .get_by_role(Role::CheckBox, "空崎 ミナト を選択")
            .is_checked(),
        Some(true)
    );
    screen.get_by_role(Role::ColumnHeader, "Lv.").click();
    assert!(
        screen
            .get_by_role(Role::CheckBox, "瀬川 ミオ を選択")
            .rect()
            .unwrap()
            .y
            < screen
                .get_by_role(Role::CheckBox, "桜井 ノノカ を選択")
                .rect()
                .unwrap()
                .y
    );
    evidence(&screen, "pop-table-application-numeric-sort");
    screen.get_by_role(Role::ColumnHeader, "Lv.").click();
    assert!(
        screen
            .query_by_role(Role::CheckBox, "瀬川 ミオ を選択")
            .is_none()
    );
    replace(&screen, "テーブルを検索", "ミナト");
    assert_eq!(
        screen
            .get_by_role(Role::CheckBox, "空崎 ミナト を選択")
            .is_checked(),
        Some(true)
    );
    replace(&screen, "テーブルを検索", "レイ");
    assert_eq!(
        screen
            .get_by_role(Role::CheckBox, "鷹宮 レイ を選択")
            .is_checked(),
        Some(true)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
