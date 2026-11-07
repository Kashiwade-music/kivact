//! Role: Local style painting follows layout and preserves drawing layers.

use std::cell::Cell;
use std::rc::Rc;

use super::paint::{BLUE, GREEN, RED, WHITE, at, canvas_of, expect};
use super::support::mount;
use kivact::ErrorKind;
use kivact::geometry::Rect;
use kivact::prelude::*;
use kivact::style::provide_theme;
use kivact::test::render;

#[test]
fn paint_only_style_tracks_size_signals_theme_and_removal() {
    let evaluations = Rc::new(Cell::new(0));
    let count = evaluations.clone();
    let (screen, (width, color, enabled, theme)) = mount(move || {
        let width = signal(40.0_f64);
        let color = signal(false);
        let enabled = signal(false);
        let theme = signal(Theme::light());
        let painted = node().style(move || {
            count.set(count.get() + 1);
            let base = at(0, 0, 40, 30).width(width.get());
            if enabled.get() {
                base.paint(move |draw, size| {
                    draw.fill_rect(
                        Rect::new(0.0, 0.0, size.width, size.height),
                        if color.get() {
                            ColorToken::Accent.into()
                        } else {
                            rgb(RED)
                        },
                    );
                })
            } else {
                base
            }
        });
        (
            (width, color, enabled, theme),
            provide_theme(theme, move || canvas_of(painted)),
        )
    });
    expect(
        &screen.screenshot(),
        30,
        10,
        WHITE,
        0,
        "style initially has no paint",
    );
    enabled.set(true);
    screen.run_until_idle();
    expect(&screen.screenshot(), 30, 10, RED, 0, "paint-only node");
    expect(&screen.screenshot(), 60, 10, WHITE, 0, "initial bounds");
    width.set(80.0);
    screen.run_until_idle();
    expect(&screen.screenshot(), 60, 10, RED, 0, "resized paint");
    theme.set(accent_theme(rgb(BLUE)));
    screen.run_until_idle();
    let before = evaluations.get();
    color.set(true);
    screen.run_until_idle();
    expect(
        &screen.screenshot(),
        60,
        10,
        BLUE,
        0,
        "paint signal dependency",
    );
    assert_eq!(
        evaluations.get(),
        before,
        "paint does not evaluate style again"
    );
    theme.set(accent_theme(rgb(GREEN)));
    screen.run_until_idle();
    expect(&screen.screenshot(), 60, 10, GREEN, 0, "theme dependency");
    enabled.set(false);
    screen.run_until_idle();
    expect(
        &screen.screenshot(),
        60,
        10,
        WHITE,
        0,
        "old commands removed",
    );
    enabled.set(true);
    screen.run_until_idle();
    expect(&screen.screenshot(), 60, 10, GREEN, 0, "paint returns");
    assert!(screen.errors().is_empty());
}

#[test]
fn merged_paint_runs_between_caller_draw_and_children_and_clears_independently() {
    let (screen, enabled) = mount(|| {
        let enabled = signal(true);
        let painted = node()
            .style(at(0, 0, 100, 30))
            .style(move || {
                if enabled.get() {
                    style()
                        .paint(|draw, _| {
                            draw.fill_rect(Rect::new(0.0, 0.0, 100.0, 30.0), rgb(GREEN));
                        })
                        .merge(&style().paint(|draw, _| {
                            draw.fill_rect(Rect::new(0.0, 0.0, 80.0, 30.0), rgb(BLUE));
                        }))
                        .merge(&style().height(30))
                } else {
                    style()
                }
            })
            .draw(|draw, _| {
                draw.fill_rect(Rect::new(0.0, 0.0, 100.0, 30.0), rgb(RED));
            })
            .child(node().style(at(0, 0, 20, 30).background(rgb(GREEN))))
            .draw_over(|draw, _| {
                draw.fill_rect(Rect::new(20.0, 0.0, 20.0, 30.0), rgb(WHITE));
            });
        (enabled, canvas_of(painted))
    });
    let shot = screen.screenshot();
    expect(&shot, 10, 10, GREEN, 0, "child over style paint");
    expect(&shot, 30, 10, WHITE, 0, "draw_over over style paint");
    expect(
        &shot,
        60,
        10,
        BLUE,
        0,
        "last explicit paint over caller draw",
    );
    expect(
        &shot,
        90,
        10,
        RED,
        0,
        "earlier paint replaced, caller draw retained",
    );
    enabled.set(false);
    screen.run_until_idle();
    let shot = screen.screenshot();
    expect(&shot, 10, 10, GREEN, 0, "child retained");
    expect(&shot, 30, 10, WHITE, 0, "draw_over retained");
    expect(&shot, 60, 10, RED, 0, "only stale style commands cleared");
    assert!(screen.errors().is_empty());
}

/// A state style can change paint only; a layout property or a nested state in it is reported once.
#[test]
fn a_property_its_state_cannot_change_is_reported() {
    let screen = render(|| {
        node().style(
            style()
                .size(40, 40)
                .hover(style().width(80).opacity(0.5).hover(style().opacity(1.0))),
        )
    });
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(*errors[0].kind(), ErrorKind::Unsupported);
    assert!(
        errors[0].message().contains("hover.width"),
        "{}",
        errors[0].message()
    );
    assert!(
        errors[0].message().contains("hover.hover"),
        "{}",
        errors[0].message()
    );
}

#[test]
fn text_color_transitions_with_paint() {
    use kivact::motion::{Easing, Transition};
    use kivact::style::TransitionProp;
    use std::time::Duration;
    let (screen, changed) = super::support::mount_with(
        kivact::test::RenderOptions::from_env().frame_period_ns(1_000_000),
        || {
            let changed = signal(false);
            (
                changed,
                text("MMMM").test_id("ink").style(move || {
                    style()
                        .font_size(48)
                        .weight(700)
                        .color(if changed.get() { rgb(BLUE) } else { rgb(RED) })
                        .transition(TransitionProp::All, Transition::tween(240, Easing::Linear))
                }),
            )
        },
    );
    let ink = screen.get_by_test_id("ink");
    let red = |pixel: &u32| (*pixel >> 16) & 255 > 180 && *pixel & 255 < 60;
    assert!(ink.screenshot().pixels.iter().any(red));
    changed.set(true);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(100));
    assert!(
        ink.screenshot().pixels.iter().any(|pixel| {
            // Oklab (§5.4.2) keeps some green mid-transition, so require a
            // pixel that mixes both ends, not an end or the background.
            let (r, b) = ((pixel >> 16) & 255, pixel & 255);
            r > 25 && b > 25 && r < 230 && b < 230
        }),
        "glyphs pass through an intermediate text color"
    );
    screen.advance(Duration::from_millis(300));
    let shot = ink.screenshot();
    assert!(
        shot.pixels
            .iter()
            .any(|pixel| *pixel & 255 > 180 && (*pixel >> 16) & 255 < 60)
    );
    assert!(!shot.pixels.iter().any(red));
    assert!(screen.errors().is_empty());
}

#[test]
fn text_decoration_style_draws_wavy_lines() {
    use kivact::style::{TextDecorationLine, TextDecorationStyle};
    let (screen, pattern) = mount(|| {
        let pattern = signal(TextDecorationStyle::Solid);
        (
            pattern,
            text("MMMMMMMM").test_id("decorated").style(
                style()
                    .font_size(40)
                    .color(rgb(0))
                    .text_decoration(TextDecorationLine::UNDERLINE)
                    .text_decoration_color(rgb(RED))
                    .text_decoration_thickness(2)
                    .text_decoration_style(pattern),
            ),
        )
    });
    let extent = || {
        let shot = screen.get_by_test_id("decorated").screenshot();
        let rows: Vec<_> = shot
            .pixels
            .chunks(shot.width as usize)
            .enumerate()
            .filter(|(_, row)| row.iter().any(|p| (*p >> 16) & 255 > 150 && *p & 255 < 80))
            .map(|(y, _)| y)
            .collect();
        assert!(!rows.is_empty(), "decoration is drawn");
        rows.last().unwrap() - rows.first().unwrap()
    };
    let solid = extent();
    pattern.set(TextDecorationStyle::Wavy);
    screen.run_until_idle();
    assert!(
        extent() > solid,
        "a wave extends above and below a straight underline"
    );
    for next in [TextDecorationStyle::Dashed, TextDecorationStyle::Dotted] {
        pattern.set(next);
        screen.run_until_idle();
        extent();
    }
    assert!(screen.errors().is_empty());
}

#[test]
fn visibility_hidden_keeps_layout_but_hides_the_node() {
    let (screen, (hidden, presses)) = visibility_fixture();
    let target = screen.get_by_test_id("hidden-target");
    let before = target.rect().unwrap();
    let after = screen.get_by_test_id("after");
    let sibling_before = after.rect().unwrap();
    target.focus();
    assert!(target.is_focused());
    target.click();
    let initial_presses = presses.get();
    hidden.set(true);
    screen.run_until_idle();
    assert_eq!(target.rect(), Some(before), "hidden box retains its layout");
    assert_eq!(
        after.rect(),
        Some(sibling_before),
        "siblings keep their positions"
    );
    assert!(!target.is_visible());
    assert!(!target.is_focused());
    let accessibility = screen.accessibility_tree();
    for name in ["target", "child"] {
        let exposed = accessibility
            .lines()
            .find(|line| line.contains(&format!("\"{name}\"")))
            .expect("retained accessibility node");
        assert!(
            exposed.contains(" hidden"),
            "hidden semantics reach the native accessibility tree"
        );
        assert!(!exposed.contains(" focusable"));
    }
    assert!(
        target.a11y_actions().is_empty(),
        "hidden node offers no accessibility actions"
    );
    assert!(
        screen
            .get_by_role(Role::Button, "child")
            .a11y_actions()
            .is_empty(),
        "hidden descendants offer no accessibility actions"
    );
    screen.click_at(
        before.x + before.width / 2.0,
        before.y + before.height / 2.0,
    );
    assert_eq!(
        presses.get(),
        initial_presses,
        "hidden subtree cannot be hit"
    );
    assert!(
        !screen
            .screenshot()
            .pixels
            .iter()
            .any(|p| (*p >> 16) & 255 > 180 && *p & 255 < 60)
    );
    screen.press_tab(false);
    assert!(after.is_focused(), "Tab skips the hidden subtree");
    hidden.set(false);
    screen.run_until_idle();
    assert!(target.is_visible());
    assert!(screen.query_by_role(Role::Button, "child").is_some());
    target.click();
    assert!(presses.get() > initial_presses);
    assert!(screen.errors().is_empty());
}

fn accent_theme(accent: kivact::style::Color) -> Theme {
    let colors = kivact::style::theme::Palette {
        accent,
        ..Theme::light().colors().clone()
    };
    Theme::light().with_colors(colors)
}

fn visibility_fixture() -> (kivact::test::Screen, (Signal<bool>, Signal<i32>)) {
    mount(|| {
        let hidden = signal(false);
        let presses = signal(0);
        let target = node()
            .test_id("hidden-target")
            .name("target")
            .role(Role::Button)
            .focusable(true)
            .on_press(move || presses.update(|n| *n += 1))
            .style(style().size(90, 40).background(rgb(RED)))
            .child(
                text("child")
                    .role(Role::Button)
                    .name("child")
                    .focusable(true),
            );
        (
            (hidden, presses),
            row((
                node()
                    .style(move || {
                        style()
                            .display(kivact::style::Display::Contents)
                            .visibility(if hidden.get() {
                                kivact::style::Visibility::Hidden
                            } else {
                                kivact::style::Visibility::Visible
                            })
                    })
                    .child(target),
                node()
                    .test_id("after")
                    .role(Role::Button)
                    .name("after")
                    .focusable(true)
                    .style(style().size(40, 40)),
            )),
        )
    })
}
