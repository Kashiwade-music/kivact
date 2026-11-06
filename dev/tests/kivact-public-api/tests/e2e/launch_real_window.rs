//! Role: Real window and native integration tests.
//!
//! Smoke tests of `Screen` on real windows (the Arvane test driver): real
//! layout, shaping, rendering and accessibility, frames shown in real
//! windows (headless where none can open), a virtual clock, failure
//! artifacts, pooled windows and tests on parallel threads.
//!
//! Set `KIVACT_TEST_HOLD_MS` to keep each window open that long at the end
//! of a test (to look at it, or to read it with UI Automation).
//! Every settle draws requested frames.

use std::time::Instant;

use kivact::platform;
use kivact::state::signal;

use super::launch_support::button_box;
use kivact::prelude::{Component, Role, column, node, node_ref, row, show, style, text};
use kivact::test::Screen;
use kivact::test::{MirrorMode, RenderOptions, render, render_with};

fn counter() -> impl Component {
    let count = signal(1);
    let draft = signal(String::new());
    column((
        text(move || format!("count: {}", count.get())),
        row((
            button_box("-").on_press(move || count.update(|c| *c -= 1)),
            button_box("+").on_press(move || count.update(|c| *c += 1)),
        ))
        .style(style().gap(8.0)),
        super::launch_support::editor("draft", draft),
        text(move || format!("draft: {}", draft.get())),
    ))
    .style(style().gap(8.0).padding(16.0))
}

fn small(mirror: MirrorMode) -> RenderOptions {
    RenderOptions::default()
        .window_size(400.0, 300.0)
        .mirror(mirror)
}

#[test]
fn a_counter_is_clicked_typed_into_and_composed_in_a_real_window() {
    let t = Instant::now();
    let screen = render_with(small(MirrorMode::Auto), counter);
    let mount_time = t.elapsed();
    let turns = screen.stats().turns;
    let t = Instant::now();
    screen.get_by_role(Role::Button, "+").click();
    let click_time = t.elapsed();
    screen.get_by_text("count: 2");
    assert_eq!(
        screen.stats().turns - turns,
        1,
        "a click costs only the turn of its input events"
    );

    let field = screen.get_by_role(Role::TextBox, "draft");
    field.click();
    field.type_text("hi there");
    field.compose("にほ");
    field.type_text(" 日本");
    screen.get_by_text("draft: hi there 日本");
    assert_eq!(field.value().as_deref(), Some("hi there 日本"));
    assert!(
        screen.ime_cursor_area().is_some(),
        "the IME follows the caret"
    );

    let shot = screen.screenshot();
    assert_eq!((shot.width, shot.height), (400, 300));
    assert!(
        shot.pixels.iter().any(|p| *p != 0x00ff_ffff),
        "something but the background was drawn"
    );
    let plus = screen.get_by_role(Role::Button, "+").screenshot();
    let r = screen
        .get_by_role(Role::Button, "+")
        .rect()
        .expect("laid out");
    assert_eq!((plus.width, plus.height), (r.width as u32, r.height as u32));
    let _ = (mount_time, click_time, screen.has_window());
}

#[test]
fn frames_are_deterministic_and_headless_frames_match_real_ones() {
    let label = || {
        column(text("Hello, Kivact! 日本").style(style().font_size(20.0)))
            .style(style().padding(8.0))
    };
    let a = render_with(small(MirrorMode::Auto), label);
    let b = render_with(small(MirrorMode::Auto), label);
    let c = render_with(small(MirrorMode::Off), label);
    assert!(!c.has_window());
    assert_eq!(a.screenshot().hash(), b.screenshot().hash());
    assert_eq!(a.screenshot().hash(), c.screenshot().hash());
    // Text is drawn: a frame without it differs.
    let empty = render_with(small(MirrorMode::Off), || {
        column(()).style(style().padding(8.0))
    });
    assert_ne!(a.screenshot().hash(), empty.screenshot().hash());
}

#[test]
fn the_environment_and_platform_requests_are_the_tests() {
    let screen = render_with(small(MirrorMode::Auto), || {
        let status = signal("idle".to_string());
        column((
            text(move || format!("reduced motion: {}", platform::is_motion_reduced().get())),
            text(move || format!("scale: {}", platform::scale_factor().get())),
            text(move || status.get()),
            button_box("copy").on_press(move || {
                kivact::task::spawn(async move {
                    let result = platform::copy_text("copied").await;
                    status.set(match result {
                        Ok(()) => "copy ok".into(),
                        Err(e) => format!("copy failed: {}", e.message()),
                    });
                });
            }),
        ))
    });
    screen.get_by_text("reduced motion: false");
    screen.set_reduced_motion(true);
    screen.get_by_text("reduced motion: true");

    screen.set_scale(2.0);
    screen.get_by_text("scale: 2");
    let shot = screen.screenshot();
    assert_eq!((shot.width, shot.height), (800, 600));
    screen.resize(300.0, 200.0);
    let shot = screen.screenshot();
    assert_eq!((shot.width, shot.height), (600, 400));
    screen.set_dark(true);

    screen.fail_next_platform_request("no clipboard today");
    screen.get_by_role(Role::Button, "copy").click();
    screen.get_by_text("copy failed: no clipboard today");
    screen.get_by_role(Role::Button, "copy").click();
    screen.get_by_text("copy ok");
    assert_eq!(screen.clipboard().as_deref(), Some("copied"));

    assert!(screen.request_close(), "a close request ends the App");
}

fn modal_screen() -> Screen {
    render(|| {
        let open = signal(true);
        column((
            button_box("disabled").disabled(true),
            button_box("behind"),
            node().test_id("empty"),
            show(open, move || {
                // Without an anchor the modal is centred in the window.
                kivact::view::overlay(
                    node_ref(),
                    kivact::view::Placement::Center,
                    button_box("close").on_press(move || open.set(false)),
                )
                .modal()
            }),
        ))
    })
}

#[test]
#[should_panic(expected = "is behind the backdrop of the modal")]
fn clicking_behind_a_modal_fails() {
    let screen = modal_screen();
    screen.get_by_role(Role::Button, "behind").click();
}

#[test]
#[should_panic(expected = "is disabled")]
fn clicking_a_disabled_button_fails() {
    let screen = modal_screen();
    screen.get_by_role(Role::Button, "close").click();
    screen.get_by_role(Role::Button, "disabled").click();
}

#[test]
#[should_panic(expected = "has no size")]
fn clicking_an_element_without_size_fails() {
    let screen = modal_screen();
    screen.get_by_test_id("empty").click();
}

#[test]
#[should_panic(expected = "not exposed to assistive technology")]
fn focusing_an_element_hidden_from_assistive_technology_fails() {
    let screen = modal_screen();
    screen.get_by_test_id("empty").focus();
}
