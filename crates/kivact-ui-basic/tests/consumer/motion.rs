//! Role: Motion tests: spinners under reduced motion, accordion and carousel animation.

use super::assert_clean;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::Screen;
use kivact_ui_basic as basic;
use std::time::Duration;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn width(screen: &Screen, id: &str) -> f32 {
    screen.get_by_test_id(id).rect().expect("laid out").width
}

#[test]
fn spinners_keep_turning_under_reduced_motion() {
    // `mount` renders with reduced motion on.
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            column((
                basic::spinner().test_id("spinner"),
                basic::attachment("upload.png")
                    .media(move || kivact::control::icon("image"))
                    .state(basic::AttachmentState::Uploading),
            ))
            .style(style().padding(24).gap(16).align_items(Align::Start)),
        )
    });
    // A 16 px square turned by 45° is 22.6 px wide.
    let diagonal = 16.0 * std::f32::consts::SQRT_2;
    let start = width(&screen, "spinner");
    screen.advance(ms(125));
    let turned = width(&screen, "spinner");
    assert!(
        (turned - diagonal).abs() <= 1.0,
        "an eighth of a second turns it by 45°: {start} wide, then {turned}"
    );
    screen.advance(ms(125));
    let quarter = width(&screen, "spinner");
    assert!(quarter < turned - 4.0, "it turns on toward 90°: {quarter}");
    let spinners = screen
        .accessibility_tree()
        .matches("Status \"Loading\"")
        .count();
    assert_eq!(spinners, 2, "the attachment shows the same spinner");
    assert!(screen.stats().animations >= 2, "both spinners keep moving");
    assert_clean(&screen);
}

#[test]
fn a_loading_button_turns_its_indicator_under_reduced_motion() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            column(button("Save").loading(true).test_id("save"))
                .style(style().padding(24).align_items(Align::Start)),
        )
    });
    let r = screen.get_by_test_id("save").rect().expect("laid out");
    let scale = screen.scale();
    let indicator = || {
        screen
            .screenshot()
            .crop(
                (r.x * scale) as u32,
                (r.y * scale) as u32,
                (r.height * scale) as u32,
                (r.height * scale) as u32,
            )
            .hash()
    };
    let before = indicator();
    // A quarter of its 0.7 s turn.
    screen.advance(ms(175));
    assert_ne!(indicator(), before, "the indicator has turned");
    assert_clean(&screen);
}

fn faq() -> impl IntoView {
    column((
        kivact::ui::disclosure("Shipping", signal(false), || {
            column((
                text("We ship worldwide."),
                text("Orders leave within two days."),
                text("Tracking comes by mail."),
            ))
        })
        .animated(true),
        kivact::ui::disclosure("Returns", signal(false), || text("30 day returns.")).animated(true),
    ))
    .style(style().width(320))
}

fn top(screen: &Screen, name: &str) -> f32 {
    screen
        .get_by_role(Role::Button, name)
        .rect()
        .expect("laid out")
        .y
}

#[test]
fn an_accordion_section_opens_and_closes_by_moving_its_height() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            column(faq()).style(style().padding(24).align_items(Align::Start)),
        )
    });
    screen.set_reduced_motion(false);
    let closed = top(&screen, "Returns");
    screen.get_by_role(Role::Button, "Shipping").click();
    screen.advance(ms(60));
    let opening = top(&screen, "Returns");
    screen.advance(ms(300));
    let open = top(&screen, "Returns");
    assert!(
        closed < opening && opening < open,
        "the next section moves down while it opens: {closed} -> {opening} -> {open}"
    );
    assert!(open - closed > 40.0, "three lines open: {}", open - closed);
    let shipping = screen.get_by_role(Role::Button, "Shipping");
    assert_eq!(shipping.is_expanded(), Some(true));

    shipping.click();
    screen.advance(ms(60));
    assert_eq!(
        screen.get_by_role(Role::Button, "Shipping").is_expanded(),
        Some(false),
        "the trigger reports the closing at once"
    );
    let closing = top(&screen, "Returns");
    assert!(
        closed < closing && closing < open,
        "the next section moves up while it closes: {open} -> {closing}"
    );
    assert!(
        screen.get_by_text("We ship worldwide.").is_visible(),
        "the content is drawn while it closes"
    );
    screen.advance(ms(300));
    assert_eq!(top(&screen, "Returns"), closed);
    assert!(screen.query_by_text("We ship worldwide.").is_none());
    assert_clean(&screen);
}

#[test]
fn an_accordion_section_opens_at_once_under_reduced_motion() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            column(faq()).style(style().padding(24).align_items(Align::Start)),
        )
    });
    let closed = top(&screen, "Returns");
    screen.get_by_role(Role::Button, "Shipping").click();
    screen.run_until_idle();
    let open = top(&screen, "Returns");
    assert!(open - closed > 40.0, "open in the same turn");
    screen.get_by_role(Role::Button, "Shipping").click();
    screen.run_until_idle();
    assert_eq!(top(&screen, "Returns"), closed);
    assert!(screen.query_by_text("We ship worldwide.").is_none());
    assert_clean(&screen);
}

#[test]
fn a_carousel_slides_to_the_next_slide() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        let index = signal(0usize);
        (
            (),
            column(
                basic::carousel(
                    (1..=3)
                        .map(|n| text(format!("Slide {n}")).test_id(format!("slide-{n}")))
                        .collect::<Vec<_>>(),
                    index,
                )
                .style(style().width(240).margin_x(48)),
            )
            .style(style().padding(24).align_items(Align::Start)),
        )
    });
    screen.set_reduced_motion(false);
    let x = |screen: &Screen, n: u32| {
        screen
            .get_by_test_id(&format!("slide-{n}"))
            .rect()
            .expect("laid out")
            .x
    };
    // One step moves a slide by the distance between two slides.
    let step = x(&screen, 3) - x(&screen, 2);
    let x = |screen: &Screen| x(screen, 2);
    let start = x(&screen);
    screen.get_by_role(Role::Button, "Next slide").click();
    screen.advance(ms(100));
    let moving = x(&screen);
    screen.advance(ms(400));
    let end = x(&screen);
    assert!(
        end < moving && moving < start,
        "the second slide slides in: {start} -> {moving} -> {end}"
    );
    assert!(
        (start - end - step).abs() <= 1.0,
        "by one slide: {start} -> {end}"
    );
    assert_clean(&screen);
}
