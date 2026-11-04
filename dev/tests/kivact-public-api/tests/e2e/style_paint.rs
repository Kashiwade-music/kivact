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
    let accent_theme = |accent| {
        let colors = kivact::style::theme::Palette {
            accent,
            ..Theme::light().colors().clone()
        };
        Theme::light().with_colors(colors)
    };
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
