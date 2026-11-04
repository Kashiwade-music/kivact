//! Role: All source chapters remain reachable in the complete, font-matched shell.
//! Optional artifacts are produced by the same public test driver with optional window mirroring
//! that consumers use; no library reaches into an implementation crate.
//! Public-driver state checks for the complete Pop catalogue.
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen, TestFonts, render_with};
use kivact::view::MouseButton;
use kivact_ui_pop_catalog::{catalog, fonts, locale, messages};
use std::time::Duration;

#[test]
fn pop_source_chapters_scroll_in_the_original_shell() {
    let screen = mount();
    evidence(&screen, "pop-initial");
    chapters(
        &screen,
        "pop",
        &[
            "overview",
            "tokens",
            "buttons",
            "forms",
            "selection",
            "display",
            "table",
            "feedback",
            "overlays",
            "navigation",
            "messaging",
            "schedule",
            "cards",
        ],
    );
}

fn mount() -> Screen {
    render_with(
        RenderOptions::from_env()
            .window_size(1846., 969.)
            .scale(1.)
            .locale(locale())
            .messages(messages())
            .reduced_motion()
            .fonts(TestFonts::Only(
                fonts().iter().map(|&bytes| bytes.into()).collect(),
            )),
        catalog,
    )
}

fn chapters(screen: &Screen, prefix: &str, chapters: &[&str]) {
    for (index, chapter) in chapters.iter().enumerate() {
        let nav = screen.get_by_test_id(&format!("{prefix}-nav-{chapter}"));
        if prefix == "creative" {
            let scroll = screen.get_by_test_id("creative-nav-scroll");
            let viewport = scroll.rect().expect("sidebar viewport");
            let item = nav.rect().expect("sidebar item");
            if item.y + item.height > viewport.y + viewport.height {
                scroll.scroll(
                    0.,
                    item.y + item.height - viewport.y - viewport.height + 12.,
                );
            }
        }
        nav.click();
        screen.advance(Duration::from_millis(400));
        assert!(nav.is_selected(), "{prefix}/{chapter} remains selected");
        let section_id = match prefix {
            "pop" => format!("pop-section-{chapter}"),
            "game" => format!("game-section-{index}"),
            "creative" => format!("creative-section-{}", index + 1),
            _ => unreachable!("known catalog scene"),
        };
        let section = screen
            .get_by_test_id(&section_id)
            .rect()
            .expect("chapter geometry");
        // Selecting the sidebar alone is insufficient: the chapter must actually
        // move into the viewport. The final sections clamp at the scroll limit.
        let final_id = match prefix {
            "pop" => format!(
                "pop-section-{}",
                chapters.last().expect("catalogue has chapters")
            ),
            "game" => format!("game-section-{}", chapters.len() - 1),
            "creative" => format!("creative-section-{}", chapters.len()),
            _ => unreachable!("known catalog scene"),
        };
        let end = screen
            .get_by_test_id(&final_id)
            .rect()
            .expect("final chapter geometry");
        let anchor = match prefix {
            "pop" => 76.0,
            "game" => 72.0,
            _ => 100.0,
        };
        let at_scroll_end = section.y > anchor + 8.0 && end.y + end.height <= 971.0;
        if at_scroll_end {
            assert!(
                (0.0..=600.0).contains(&section.y),
                "{prefix}/{chapter} visible at scroll limit: {section:?}"
            );
        } else {
            assert!(
                (section.y - anchor).abs() <= 8.0,
                "{prefix}/{chapter} scrolls to {anchor}: {section:?}"
            );
        }
        assert!(
            screen.errors().is_empty(),
            "{prefix}/{chapter}: {:?}",
            screen.errors()
        );
        screen.assert_layout();
        evidence(screen, &format!("{prefix}-{chapter}"));
    }
}

/// Opt-in leaves ordinary successful tests free of filesystem IO.
fn evidence(screen: &Screen, name: &str) {
    if let Some(directory) = std::env::var_os("KIVACT_CATALOG_EVIDENCE") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("create evidence directory");
        screen
            .screenshot()
            .save_png(directory.join(format!("{name}.png")))
            .expect("save evidence screenshot");
        std::fs::write(
            directory.join(format!("{name}-a11y.txt")),
            screen.accessibility_tree(),
        )
        .expect("save published accessibility tree");
        std::fs::write(
            directory.join(format!("{name}-retained.txt")),
            screen.debug(),
        )
        .expect("save retained semantic tree");
    }
}

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
    assert!(email.is_invalid());
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
    let resize = |delta: f64| {
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
            .checked(),
        Some(CheckedState::Unchecked)
    );
    assert_eq!(
        screen
            .get_by_role(Role::CheckBox, "部分選択（indeterminate）")
            .checked(),
        Some(CheckedState::Indeterminate)
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
            .checked(),
        Some(CheckedState::Checked)
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
