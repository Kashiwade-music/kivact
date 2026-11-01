//! Role: Typed themes, motion lifecycle and host state transitions on real windows.
use crate::support::mount;
use kivact::draw::Point;
use kivact::motion::{Animated, Easing, Transition, animated};
use kivact::prelude::*;
use kivact::style::Color;
use kivact::style::current_theme;
use kivact::style::{TransitionProp, hsl};
use kivact::view::provide_theme;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
struct Tokens {
    color: Color,
    width: f32,
}

impl kivact::style::ThemeExtension for Tokens {
    fn fallback(_: &Theme) -> Self {
        Tokens {
            color: Color::BLACK,
            width: 10.0,
        }
    }
}

#[test]
fn typed_tokens_and_color_operations_follow_the_subtree_theme() {
    fn transferable<T: Send + Sync>() {}
    transferable::<Theme>();
    let (screen, theme) = mount(|| {
        let theme = signal(Theme::light().with(Tokens {
            color: hsl(0, 1, 0.5),
            width: 40.0,
        }));
        let view = provide_theme(theme, || {
            node().test_id("token").style(move || {
                let t = &current_theme();
                let tokens = t.get::<Tokens>();
                style()
                    .size(tokens.width, 40)
                    .background(tokens.color.mix_in(Color::BLACK, 0.5, t))
            })
        });
        (theme, view)
    });
    assert_eq!(screen.screenshot().pixel(20, 20), Some(0x800000));
    let next = Theme::dark().with(Tokens {
        color: hsl(120, 1, 0.5),
        width: 60.0,
    });
    assert_eq!(
        next,
        next.clone().with(Tokens {
            color: hsl(120, 1, 0.5),
            width: 60.0
        })
    );
    // A theme without the tokens reads their fallback.
    assert_eq!(Theme::dark().get::<Tokens>().width, 10.0);
    assert_eq!(
        Color::from(ColorToken::Accent).components_in(&next),
        next.colors().accent.components()
    );
    theme.set(next);
    screen.run_until_idle();
    assert_eq!(screen.screenshot().pixel(50, 20), Some(0x008000));
    assert_eq!(
        screen
            .get_by_test_id("token")
            .rect()
            .expect("laid out")
            .width,
        60.0
    );
    assert!(screen.errors().is_empty());
}

#[test]
fn explicit_start_pause_retarget_resume_and_completion() {
    let completions = Rc::new(Cell::new(0));
    let count = completions.clone();
    let (screen, (target, shown)) = mount(move || {
        let target = signal(100.0f32);
        let shown = Animated::from(0.0, target, Transition::tween(200, Easing::Linear))
            .on_complete(move || count.set(count.get() + 1));
        ((target, shown), text(move || format!("{}", shown.get())))
    });
    screen.advance(Duration::from_millis(80));
    let at = shown.get_untracked();
    assert!((20.0..60.0).contains(&at));
    shown.pause();
    screen.advance(Duration::from_millis(500));
    assert_eq!(shown.get_untracked(), at);
    target.set(200.0);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(100));
    assert_eq!(shown.get_untracked(), at);
    shown.resume();
    screen.advance(Duration::from_millis(250));
    assert_eq!(shown.get_untracked(), 200.0);
    assert_eq!(completions.get(), 1);
    target.set(300.0);
    screen.run_until_idle();
    shown.jump_to(300.0);
    screen.advance(Duration::from_millis(300));
    assert_eq!(completions.get(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn steps_and_host_hover_transition_retarget_the_displayed_value() {
    let (screen, shown) = mount(|| {
        let shown = Animated::from(0.0f32, 1.0f32, Transition::tween(200, Easing::steps_end(4)));
        (
            shown,
            node().test_id("card").style(
                style()
                    .size(80, 40)
                    .background(rgb(0xff0000))
                    .hover(style().background(rgb(0x0000ff)))
                    .transition(
                        TransitionProp::Paint,
                        Transition::tween(200, Easing::Linear),
                    ),
            ),
        )
    });
    screen.advance(Duration::from_millis(40));
    assert_eq!(shown.get_untracked(), 0.0);
    screen.advance(Duration::from_millis(40));
    assert_eq!(shown.get_untracked(), 0.25);
    screen.get_by_test_id("card").hover();
    assert_eq!(screen.screenshot().pixel(20, 20), Some(0xff0000));
    screen.advance(Duration::from_millis(80));
    let color = screen.screenshot().pixel(20, 20).expect("pixel");
    assert!(color & 255 > 20 && (color >> 16) & 255 > 20, "{color:06x}");
    screen.mouse_move(200.0, 100.0);
    assert_eq!(screen.screenshot().pixel(20, 20), Some(color));
    screen.advance(Duration::from_millis(250));
    assert_eq!(screen.screenshot().pixel(20, 20), Some(0xff0000));
    assert_eq!(shown.get_untracked(), 1.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn entry_and_exit_keep_the_branch_until_completion_and_allow_reentry() {
    let (screen, (open, retained, opacity)) = mount(|| {
        let open = signal(true);
        let retained = signal(true);
        let opacity = Animated::from(
            0.0f32,
            move || if open.get() { 1.0 } else { 0.0 },
            Transition::tween(120, Easing::Linear),
        )
        .on_complete(move || {
            if !open.get() {
                retained.set(false);
            }
        });
        let view = show(retained, move || {
            text("toast")
                .test_id("toast")
                .style(move || style().opacity(opacity.get()))
        });
        ((open, retained, opacity), view)
    });
    assert!(screen.query_by_test_id("toast").is_some());
    screen.advance(Duration::from_millis(160));
    assert_eq!(opacity.get_untracked(), 1.0);
    open.set(false);
    screen.advance(Duration::from_millis(50));
    assert!(screen.query_by_test_id("toast").is_some());
    open.set(true);
    screen.advance(Duration::from_millis(160));
    assert!(screen.query_by_test_id("toast").is_some());
    open.set(false);
    screen.advance(Duration::from_millis(160));
    assert!(screen.query_by_test_id("toast").is_none());
    opacity.jump_to(0.0);
    retained.set(true);
    open.set(true);
    screen.advance(Duration::from_millis(160));
    assert!(screen.query_by_test_id("toast").is_some());
    assert_eq!(opacity.get_untracked(), 1.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn host_focus_pressed_surface_pause_reduced_motion_and_cleanup() {
    use kivact::view::MouseButton;
    let (screen, open) = mount(|| {
        let open = signal(true);
        (
            open,
            show(open, || {
                node()
                    .test_id("card")
                    .role(Role::Button)
                    .focusable(true)
                    .style(
                        style()
                            .size(80, 40)
                            .background(rgb(0xff0000))
                            .focus(style().background(rgb(0x00ff00)))
                            .hover(style().background(rgb(0x0000ff)))
                            .pressed(style().background(Color::BLACK))
                            .transition(
                                TransitionProp::Paint,
                                Transition::tween(200, Easing::Linear),
                            ),
                    )
            }),
        )
    });
    screen.get_by_test_id("card").focus();
    screen.advance(Duration::from_millis(250));
    assert_eq!(screen.screenshot().pixel(20, 20), Some(0x00ff00));
    screen.mouse_move(20.0, 20.0);
    screen.advance(Duration::from_millis(80));
    let halfway = screen.screenshot().pixel(20, 20);
    screen.lose_surface();
    screen.advance(Duration::from_secs(1));
    assert_eq!(
        screen.screenshot().pixel(20, 20),
        halfway,
        "no presentation while hidden"
    );
    screen.restore_surface();
    let resumed = screen.screenshot().pixel(20, 20).expect("resumed pixel");
    assert!(
        crate::paint::distance(resumed, halfway.expect("halfway pixel")) <= 25,
        "resume may present one display tick, without counting the hidden second"
    );
    screen.advance(Duration::from_millis(250));
    assert_eq!(screen.screenshot().pixel(20, 20), Some(0x0000ff));
    screen.mouse_down(MouseButton::Primary);
    screen.advance(Duration::from_millis(250));
    assert_eq!(screen.screenshot().pixel(20, 20), Some(0x000000));
    screen.set_reduced_motion(true);
    screen.mouse_move(200.0, 100.0);
    assert_eq!(screen.screenshot().pixel(20, 20), Some(0x00ff00));
    screen.mouse_up(MouseButton::Primary);
    screen.set_reduced_motion(false);
    screen.mouse_move(20.0, 20.0);
    screen.advance(Duration::from_millis(50));
    open.set(false);
    screen.run_until_idle();
    let ticks = screen.frame_ticks();
    screen.advance(Duration::from_secs(1));
    assert!(
        screen.frame_ticks() <= ticks + 1,
        "removed node leaves no frame work"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn focus_transform_keeps_hit_bounds_accessibility_and_ime_on_the_drawing() {
    let (screen, node) = mount(|| {
        let node = node_ref();
        (
            node,
            kivact::control::node()
                .name("field")
                .part(kivact::interaction::use_text_edit(kivact::signal(String::new())).control())
                .node_ref(node)
                .test_id("field")
                .style(
                    style()
                        .size(120, 28)
                        .focus(style().translate(80, 0))
                        .transition(
                            TransitionProp::Paint,
                            Transition::tween(200, Easing::Linear),
                        ),
                ),
        )
    });
    screen.get_by_test_id("field").focus();
    screen.advance(Duration::from_millis(80));
    let field = screen.get_by_test_id("field");
    let rect = field.rect().expect("drawn");
    assert!(rect.x > 10.0 && rect.x < 70.0, "{rect:?}");
    assert_eq!(field.accessibility_bounds().expect("exposed"), rect);
    assert!(node.hit_test(Point::new(rect.x + 5.0, rect.y + 5.0)));
    assert!(!node.hit_test(Point::new(1.0, 5.0)));
    let ime = screen.ime_cursor_area().expect("caret");
    assert!(ime.x >= rect.x && ime.x < rect.x + rect.width);
    screen.advance(Duration::from_millis(200));
    assert!((screen.get_by_test_id("field").rect().expect("drawn").x - 80.0).abs() < 0.1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn completion_is_once_per_repeat_and_callbacks_can_start_another_animation() {
    let hits = Rc::new(Cell::new(0));
    let count = hits.clone();
    let (screen, (target, next)) = mount(move || {
        let next_target = signal(0.0f32);
        let next = animated(next_target, Transition::tween(80, Easing::Linear));
        let target = signal(1.0f32);
        let shown = Animated::from(
            0.0,
            target,
            Transition::tween(80, Easing::Linear)
                .repeat(Some(2))
                .alternate(),
        )
        .on_complete(move || {
            count.set(count.get() + 1);
            next_target.set(10.0);
        });
        (
            (target, next),
            text(move || format!("{} {}", shown.get(), next.get())),
        )
    });
    screen.advance(Duration::from_millis(110));
    assert_eq!(hits.get(), 0);
    screen.advance(Duration::from_millis(160));
    assert_eq!(hits.get(), 1);
    assert_eq!(next.get_untracked(), 10.0);
    screen.set_reduced_motion(true);
    target.set(2.0);
    screen.run_until_idle();
    assert_eq!(hits.get(), 2);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn host_state_text_color_interpolates_and_restores_ancestor_inheritance() {
    let screen = kivact::test::render(|| {
        node().test_id("card").child(text("inherited color")).style(
            style()
                .size(180, 40)
                .color(rgb(0xff0000))
                .hover(style().color(rgb(0x0000ff)))
                .transition(
                    TransitionProp::Paint,
                    Transition::tween(200, Easing::Linear),
                ),
        )
    });
    let resting = screen.screenshot().hash();
    screen.get_by_test_id("card").hover();
    assert_eq!(screen.screenshot().hash(), resting);
    screen.advance(Duration::from_millis(80));
    let middle = screen.screenshot().hash();
    assert_ne!(middle, resting);
    screen.advance(Duration::from_millis(200));
    let end = screen.screenshot().hash();
    assert_ne!(middle, end);
    screen.mouse_move(300.0, 100.0);
    screen.advance(Duration::from_millis(250));
    assert_eq!(screen.screenshot().hash(), resting);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn removing_a_hover_style_retargets_from_the_current_host_value() {
    let (screen, enabled) = mount(|| {
        let enabled = signal(true);
        (
            enabled,
            node().test_id("card").style(move || {
                let s = style().size(80, 40).background(rgb(0xff0000)).transition(
                    TransitionProp::Paint,
                    Transition::tween(200, Easing::Linear),
                );
                if enabled.get() {
                    s.hover(style().background(rgb(0x0000ff)))
                } else {
                    s
                }
            }),
        )
    });
    screen.get_by_test_id("card").hover();
    screen.advance(Duration::from_millis(80));
    let before = screen.screenshot().pixel(20, 20);
    enabled.set(false);
    screen.run_until_idle();
    assert_eq!(screen.screenshot().pixel(20, 20), before);
    screen.advance(Duration::from_millis(250));
    assert_eq!(screen.screenshot().pixel(20, 20), Some(0xff0000));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn disposing_an_infinite_animation_never_notifies_completion() {
    let hits = Rc::new(Cell::new(0));
    let count = hits.clone();
    let (screen, open) = mount(move || {
        let open = signal(true);
        (
            open,
            show(open, move || {
                let count = count.clone();
                let shown = Animated::from(
                    0.0f32,
                    1.0f32,
                    Transition::tween(40, Easing::Linear).repeat(None),
                )
                .on_complete(move || count.set(count.get() + 1));
                text(move || format!("{}", shown.get()))
            }),
        )
    });
    screen.advance(Duration::from_millis(200));
    assert_eq!(hits.get(), 0);
    open.set(false);
    screen.advance(Duration::from_millis(200));
    assert_eq!(hits.get(), 0);
    assert_eq!(screen.stats().animations, 0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn paused_equal_or_reduced_motion_retarget_stays_paused_until_resume() {
    for decorative in [false, true] {
        let (screen, (target, shown)) = mount(move || {
            let target = signal(1.0f32);
            let shown = Animated::from(
                0.0,
                target,
                Transition::tween(200, Easing::Linear).decorative(decorative),
            );
            ((target, shown), text(move || format!("{}", shown.get())))
        });
        screen.advance(Duration::from_millis(80));
        shown.pause();
        let at = shown.get_untracked();
        target.set(at);
        screen.run_until_idle();
        if decorative {
            screen.set_reduced_motion(true);
        }
        target.set(2.0);
        screen.advance(Duration::from_millis(250));
        assert_eq!(
            shown.get_untracked(),
            at,
            "paused retarget (decorative={decorative})"
        );
        assert!(!shown.is_running());
        shown.resume();
        screen.advance(Duration::from_millis(250));
        assert_eq!(shown.get_untracked(), 2.0);
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn removing_dynamic_state_styles_restores_base_paint() {
    let (screen, enabled) = mount(|| {
        let enabled = signal(true);
        (
            enabled,
            node().test_id("card").style(move || {
                let s = style().size(40, 40).background(rgb(0xff0000));
                if enabled.get() {
                    s.hover(style().background(rgb(0x0000ff)))
                } else {
                    s
                }
            }),
        )
    });
    screen.get_by_test_id("card").hover();
    assert_eq!(screen.screenshot().pixel(20, 20), Some(0x0000ff));
    enabled.set(false);
    screen.run_until_idle();
    assert_eq!(screen.screenshot().pixel(20, 20), Some(0xff0000));
    assert!(screen.errors().is_empty());
}

#[test]
fn removing_dynamic_text_style_restores_inheritance() {
    let (screen, explicit) = mount(|| {
        let explicit = signal(true);
        let label = text("inherit").style(move || {
            if explicit.get() {
                style().color(rgb(0x0000ff))
            } else {
                style()
            }
        });
        (
            explicit,
            column((label,)).style(style().color(rgb(0xff0000))),
        )
    });
    explicit.set(false);
    screen.run_until_idle();
    let expected =
        kivact::test::render(|| column((text("inherit"),)).style(style().color(rgb(0xff0000))));
    assert_eq!(screen.screenshot().hash(), expected.screenshot().hash());
    assert!(screen.errors().is_empty());
}

#[test]
fn typed_theme_returns_from_blocking_worker_to_the_ui() {
    let (screen, loaded) = mount(|| {
        let loaded = resource(
            || (),
            |()| async {
                Ok(blocking(|| {
                    Theme::dark().with(Tokens {
                        color: rgb(0xff0000),
                        width: 42.0,
                    })
                })
                .await)
            },
        );
        (
            loaded,
            text(move || {
                loaded
                    .latest()
                    .map(|t| t.get::<Tokens>().width.to_string())
                    .unwrap_or_default()
            }),
        )
    });
    screen.find_by_text("42");
    assert_eq!(
        loaded.latest().unwrap().get::<Tokens>().color,
        rgb(0xff0000)
    );
    assert!(screen.errors().is_empty());
}

#[test]
fn host_transition_keeps_base_timing_when_state_replaces_effects() {
    use kivact::draw::Filter;
    let (screen, ()) = mount(|| {
        (
            (),
            node().test_id("card").style(
                style()
                    .size(80, 40)
                    .background(rgb(0xff0000))
                    .hover(
                        style()
                            .background(rgb(0x0000ff))
                            .filter(Filter::new().brightness(1)),
                    )
                    .transition(
                        TransitionProp::Paint,
                        Transition::tween(200, Easing::Linear),
                    ),
            ),
        )
    });
    screen.get_by_test_id("card").hover();
    assert_eq!(screen.screenshot().pixel(20, 20), Some(0xff0000));
    screen.advance(Duration::from_millis(80));
    let color = screen.screenshot().pixel(20, 20).expect("pixel");
    assert!(color & 255 > 20 && (color >> 16) & 255 > 20, "{color:06x}");
    screen.mouse_move(200.0, 100.0);
    assert_eq!(screen.screenshot().pixel(20, 20), Some(color));
    screen.advance(Duration::from_millis(250));
    assert_eq!(screen.screenshot().pixel(20, 20), Some(0xff0000));
    assert!(screen.errors().is_empty());
}
