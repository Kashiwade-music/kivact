//! Role: These tests verify style, theme and primitive drawing controls.

use kivact::style::current_theme;
use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount;
use kivact::control::canvas;
use kivact::draw::TextStyle;
use kivact::draw::{Draw, IntrinsicSize, Point, Rect, RegionId, Size};
use kivact::draw::{Layer, Transform};
use kivact::draw::{Shadow, linear_gradient};
use kivact::prelude::*;
use kivact::style::Direction;
use kivact::style::theme::{Palette, Radii, Typography};
use kivact::style::{Align, pct};
use kivact::test::{RenderOptions, Screen, render, render_with};
use kivact::view::provide_theme;
use kivact::view::{Handled, PointerEvent, error_boundary};

/// A game-console theme: dark, large radius and type, thick focus ring.
fn console_theme() -> Theme {
    let t = Theme::dark();
    let colors = Palette {
        accent: rgb(0xff3366),
        focus: rgb(0xffcc00),
        ..t.colors().clone()
    };
    let radius = Radii {
        md: 16.0,
        ..*t.radius()
    };
    let text = Typography {
        size: 22.0,
        ..t.text().clone()
    };
    t.with_colors(colors)
        .with_radius(radius)
        .with_text(text)
        .with_focus_ring(4.0)
        .with_control_height(48.0)
}

fn gauge(d: &mut Draw, size: Size) {
    d.fill_rect(
        size.to_rect(),
        linear_gradient(90).colors([rgb(0x000000), ColorToken::Accent.into()]),
    );
    d.region(
        RegionId(1),
        Rect::new(0.0, 0.0, size.width / 2.0, size.height),
    );
    d.region(
        RegionId(2),
        Rect::new(size.width / 2.0, 0.0, size.width / 2.0, size.height),
    );
}

fn widgets() -> impl IntoView {
    let on = signal(false);
    let choice = signal(None::<&'static str>);
    let name = signal(String::new());
    let picked = signal(String::from("-"));
    column((
        {
            let label = ("Primary").into_value();
            node()
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .child(text(label))
                .style(move || {
                    let t = &current_theme();
                    style()
                        .padding_x(t.space().lg)
                        .padding_y(t.space().sm)
                        .background(ColorToken::Accent)
                })
        }
        .style(
            style()
                .shadow(Shadow::new(0.0, 2.0, 4.0, ColorToken::Shadow))
                .hover(style().scale(1.05)),
        ),
        {
            let label = ("Disabled").into_value();
            node()
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .child(text(label))
                .style(move || {
                    let t = &current_theme();
                    style()
                        .padding_x(t.space().lg)
                        .padding_y(t.space().sm)
                        .background(ColorToken::Accent)
                })
        }
        .disabled(true),
        node()
            .role(Role::CheckBox)
            .focusable(true)
            .name("Remember")
            .child(text("Remember"))
            .checked(on)
            .on_press(move || on.update(|value| *value = !*value)),
        {
            let group = kivact::interaction::use_list(
                vec![
                    kivact::ui::ListItem::new("alpha", ""),
                    kivact::ui::ListItem::new("beta", ""),
                    kivact::ui::ListItem::new("gamma", ""),
                ],
                kivact::interaction::ListSelection::Single((choice).into()),
            )
            .policy(kivact::interaction::ListPolicy::radio());
            node().role(Role::ListBox).name("Choice").child(column((
                node()
                    .part(group.item("alpha").role(Role::Option).name("Alpha"))
                    .child(text("Alpha")),
                node()
                    .part(group.item("beta").role(Role::Option).name("Beta"))
                    .child(text("Beta")),
                node()
                    .part(group.item("gamma").role(Role::Option).name("Gamma"))
                    .child(text("Gamma")),
            )))
        },
        kivact::control::node()
            .name("Name")
            .part(kivact::interaction::use_text_edit(name).control()),
        canvas(gauge)
            .region(
                RegionId(1),
                kivact::primitive::part(|state| state)
                    .role(Role::Button)
                    .name("left half"),
            )
            .region(
                RegionId(2),
                kivact::primitive::part(|state| state)
                    .role(Role::Button)
                    .name("right half"),
            )
            .style(style().size(200, 40))
            .role(Role::Canvas)
            .name("gauge")
            .on_pointer(move |e: PointerEvent| {
                if let Some(r) = e.region() {
                    picked.set(format!("region {}", r.0));
                }
                Handled::Yes
            }),
        node()
            .style(style().size(200, 40).radius(8))
            .draw(gauge)
            .region(
                RegionId(1),
                kivact::primitive::part(|state| state)
                    .role(Role::Button)
                    .name("left half"),
            )
            .region(
                RegionId(2),
                kivact::primitive::part(|state| state)
                    .role(Role::Button)
                    .name("right half"),
            )
            // A role exposes the node, and with it its scene regions.
            .role(Role::Group)
            .name("layer"),
        text(move || picked.get()).test_id("picked"),
        text(move || format!("on={} choice={:?}", on.get(), choice.get())).test_id("state"),
    ))
    .style(style().gap(8).padding(8).background(ColorToken::Surface))
}

fn canvas_size(screen: &Screen) -> Option<(f32, f32)> {
    screen
        .get_by_role(Role::Canvas, "gauge")
        .rect()
        .map(|r| (r.width, r.height))
}

#[test]
fn two_themes_keep_semantics_and_hit_regions() {
    let a = render_with(
        RenderOptions {
            theme: Theme::light(),
            ..RenderOptions::from_env()
        },
        widgets,
    );
    let b = render_with(
        RenderOptions {
            theme: console_theme(),
            ..RenderOptions::from_env()
        },
        widgets,
    );
    assert_eq!(
        a.snapshot(),
        b.snapshot(),
        "semantics are theme independent"
    );
    assert_eq!(
        canvas_size(&a),
        canvas_size(&b),
        "hit regions are laid out on the same canvas size"
    );
    for s in [&a, &b] {
        // canvas region hit → pointer handler sees the region
        s.get_by_role(Role::Canvas, "gauge")
            .get_by_role(Role::Button, "right half")
            .click();
        assert_eq!(s.get_by_test_id("picked").text(), "region 2");
        assert!(s.errors().is_empty(), "{:?}", s.errors());
    }
}

#[test]
fn runtime_theme_switch_changes_paint_only() {
    let (screen, theme) = mount(|| {
        let theme = signal(Theme::light());
        (theme, provide_theme(theme, widgets))
    });
    let before_tree = screen.snapshot();
    let before = screen.stats();
    theme.set(console_theme());
    screen.run_until_idle();
    let after = screen.stats();
    assert!(after.set_props > before.set_props, "styles re-resolved");
    assert_eq!(after.creates, before.creates, "no remount");
    assert_eq!(screen.snapshot(), before_tree);
}

#[test]
fn buttons_focus_ring_and_tab_order_skip_static_text() {
    let screen = render(widgets);
    screen.press_tab(false);
    assert_eq!(
        screen.focused().map(|e| e.name()),
        Some("Primary".to_string())
    );
    screen.press_tab(false);
    // the disabled button is skipped
    assert_eq!(
        screen.focused().map(|e| e.name()),
        Some("Remember".to_string())
    );
    assert!(screen.get_by_role(Role::Button, "Disabled").is_disabled());
}

#[test]
fn draw_layer_and_canvas_share_the_vocabulary() {
    let screen = render(widgets);
    let canvas = screen.get_by_role(Role::Canvas, "gauge");
    let layer = screen.get_by_text("on=false choice=None");
    let _ = layer;
    let layered = screen.all_by_role(Role::Button, "left half");
    assert_eq!(layered.len(), 2, "regions from canvas and draw layer");
    assert_eq!(canvas.scene_len(), 1);
}

#[test]
fn canvas_rotation_and_text_are_supported() {
    let screen = render(|| {
        canvas(|d: &mut Draw, size: Size| {
            d.layer(Layer::new().transform(Transform::new().rotate(30)), |d| {
                d.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), ColorToken::Accent);
                d.text("tilted", Point::new(0.0, 0.0), TextStyle::default());
            });
            d.layer(Layer::new().opacity(0.5), |d| {
                d.fill_rect(size.to_rect(), ColorToken::Surface);
            });
        })
        .style(style().size(50, 50))
        .name("rot")
    });
    let c = screen.get_by_role(Role::Canvas, "rot");
    assert_eq!(
        c.scene_len(),
        7,
        "transform, rectangle, text, end of transform, layer, rectangle, end of layer"
    );
    let _ = screen.screenshot();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
#[test]
fn rotated_and_scaled_element_transforms_are_supported() {
    // Element transforms used to be translate + uniform scale only (others
    // were rejected as `Unsupported`); every affine transform is drawn now.
    let seen = Rc::new(RefCell::new(Vec::new()));
    let s2 = seen.clone();
    let screen = render(move || {
        error_boundary(|| {
            column((
                node()
                    .style(style().size(40, 40).rotate(45))
                    .test_id("rotated"),
                node()
                    .style(
                        style()
                            .size(40, 40)
                            .transform(Transform::new().translate(4, 0).scale(1.5)),
                    )
                    .test_id("scaled"),
            ))
        })
        .on_error(move |e| s2.borrow_mut().push(e.kind().clone()))
    });
    assert!(seen.borrow().is_empty(), "{:?}", seen.borrow());
    // The rect is where the box is drawn: the rotated square's bounds.
    let a = screen.get_by_test_id("rotated").rect().expect("rect");
    let diagonal = 40.0 * std::f32::consts::SQRT_2;
    assert!((a.width - diagonal).abs() < 0.01, "{a:?}");
    assert!((a.height - diagonal).abs() < 0.01, "{a:?}");
    let b = screen.get_by_test_id("scaled").rect().expect("rect");
    assert!(
        (b.width - 60.0).abs() < 0.01 && (b.height - 60.0).abs() < 0.01,
        "{b:?}"
    );
}

#[test]
fn duplicate_style_properties_keep_the_last_value_with_a_note() {
    let screen = render(|| node().style(style().width(10).width(20)).test_id("dup"));
    let rect = screen.get_by_test_id("dup").rect().expect("rect");
    assert_eq!(rect.width, 20.0, "last value wins");
    let notes: Vec<String> = screen.trace().into_iter().flat_map(|t| t.notes).collect();
    if cfg!(debug_assertions) {
        assert!(notes.iter().any(|n| n.contains("twice")), "{notes:?}");
    }
    assert!(screen.errors().is_empty());
}

#[test]
fn layout_containers_relate_as_expected() {
    let screen = render(|| {
        column((
            row((
                text("left").test_id("l"),
                spacer(),
                text("right").test_id("r"),
            ))
            .style(style().width(300)),
            stack((
                node().style(style().size(100, 50)).test_id("base"),
                text("over").test_id("over"),
            )),
            scroll(column(
                (0..50)
                    .map(|i| text(format!("line {i}")))
                    .collect::<Vec<_>>(),
            ))
            .style(style().height(100))
            .test_id("scroller"),
        ))
        .style(style().direction(Direction::Column))
    });
    let l = screen.get_by_test_id("l").rect().expect("rect");
    let r = screen.get_by_test_id("r").rect().expect("rect");
    assert!(r.x > l.x + 200.0, "spacer pushes right: {l:?} {r:?}");
    let base = screen.get_by_test_id("base").rect().expect("rect");
    let over = screen.get_by_test_id("over").rect().expect("rect");
    assert_eq!((over.x, over.y), (base.x, base.y), "stack layers overlap");
    let sc = screen.get_by_test_id("scroller");
    let first = screen.get_by_text("line 0").rect().expect("rect");
    sc.scroll(0.0, 60.0);
    let moved = screen.get_by_text("line 0").rect().expect("rect");
    assert_eq!(moved.y, first.y - 60.0);
}

#[test]
fn image_and_style_values_accept_all_inputs() {
    let screen = render(|| {
        let w = signal(40.0f32);
        column((
            image("logo.png")
                .name("logo")
                .intrinsic_size(IntrinsicSize {
                    preferred: Some(Size::new(32.0, 32.0)),
                    ..IntrinsicSize::default()
                }),
            node()
                .style(move || style().width(w.get()).height(pct(10)))
                .test_id("sized"),
        ))
        .style(style().align_items(Align::Start))
    });
    let logo = screen
        .get_by_role(Role::Image, "logo")
        .rect()
        .expect("rect");
    assert_eq!((logo.width, logo.height), (32.0, 32.0));
    let sized = screen.get_by_test_id("sized").rect().expect("rect");
    assert_eq!(sized.width, 40.0);
}

/// Regression (wheel handling on the host): `on_wheel` returning
/// `Handled::Yes` stops the host's native scroll; `Handled::No` lets it
/// continue (the core answers the host with `ScrollBy`).
#[test]
fn on_wheel_handled_stops_native_scroll() {
    let (screen, (handled, seen)) = mount(|| {
        let handled = signal(true);
        let seen = signal(0u32);
        (
            (handled, seen),
            scroll(column((
                node()
                    .style(style().height(60).shrink(0))
                    .test_id("zoomer")
                    .on_wheel(move |_e| {
                        seen.update(|n| *n += 1);
                        if handled.get_untracked() {
                            Handled::Yes
                        } else {
                            Handled::No
                        }
                    }),
                column(
                    (0..40)
                        .map(|i| text(format!("line {i}")))
                        .collect::<Vec<_>>(),
                ),
            )))
            .style(style().height(100))
            .test_id("scroller"),
        )
    });
    let top = || screen.get_by_text("line 0").rect().expect("rect").y;
    let y0 = top();
    screen.get_by_test_id("zoomer").scroll(0.0, 40.0);
    assert_eq!(seen.get_untracked(), 1);
    assert_eq!(top(), y0, "Handled::Yes stops the host's scroll");
    handled.set(false);
    screen.get_by_test_id("zoomer").scroll(0.0, 40.0);
    assert_eq!(seen.get_untracked(), 2);
    assert_eq!(top(), y0 - 40.0, "Handled::No lets the host scroll");
    // Wheel over content without a listener scrolls natively.
    screen.get_by_text("line 3").scroll(0.0, 10.0);
    assert_eq!(seen.get_untracked(), 2);
    assert_eq!(top(), y0 - 50.0);
}

/// Regression: moves of a drag carry the held button in `buttons`; only
/// the press and the release name it as their changed `button`.
#[test]
fn drag_moves_report_the_held_button() {
    use kivact::view::{MouseButton, PointerPhase};
    let (screen, log) = mount(|| {
        let log = signal(Vec::<(PointerPhase, bool, bool)>::new());
        (
            log,
            node()
                .style(style().size(100, 40))
                .test_id("pad")
                .on_pointer(move |e: PointerEvent| {
                    let held = e.buttons.contains(MouseButton::Primary);
                    log.update(|l| l.push((e.phase, e.button.is_some(), held)));
                    Handled::Yes
                }),
        )
    });
    screen
        .get_by_test_id("pad")
        .drag(Point::new(10.0, 10.0), Point::new(90.0, 10.0));
    // The pointer first enters the pad (no button held yet): a press
    // tracks the hover before it is delivered.
    assert_eq!(
        log.get_untracked(),
        vec![
            (PointerPhase::Enter, false, false),
            (PointerPhase::Down, true, true),
            (PointerPhase::Move, false, true),
            (PointerPhase::Move, false, true),
            (PointerPhase::Up, true, false),
        ]
    );
}

/// `.style()` layers merge in call order. A later
/// layer wins per property; what it leaves unset keeps the earlier value.
#[test]
fn style_layers_merge_in_call_order() {
    let screen = render(|| {
        column((
            node()
                .style(style().width(10).height(30))
                .style(style().width(20))
                .test_id("two"),
            node()
                .style(style().width(10).height(30))
                .style(move || {
                    let t = &current_theme();
                    style().width(t.control_height())
                })
                .style(style().height(5))
                .test_id("three"),
        ))
        .style(style().align_items(Align::Start))
    });
    let two = screen.get_by_test_id("two").rect().expect("rect");
    assert_eq!((two.width, two.height), (20.0, 30.0));
    let three = screen.get_by_test_id("three").rect().expect("rect");
    assert_eq!(
        (three.width, three.height),
        (Theme::light().control_height(), 5.0)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A `.style(..)` closure reading `current_theme()` sees the theme visible
/// at the node and follows a `Signal<Theme>` switch without remounting.
#[test]
fn a_style_reading_current_theme_follows_a_theme_change() {
    let (screen, theme) = mount(|| {
        let theme = signal(Theme::light());
        let view = provide_theme(theme, || {
            node()
                .style(move || {
                    let t = &current_theme();
                    style().width(t.control_height()).height(10)
                })
                .test_id("sized")
        });
        (theme, column(view).style(style().align_items(Align::Start)))
    });
    let el = screen.get_by_test_id("sized");
    assert_eq!(
        el.rect().expect("rect").width,
        Theme::light().control_height()
    );
    let creates = screen.stats().creates;
    theme.set(console_theme());
    screen.run_until_idle();
    assert_eq!(
        el.rect().expect("rect").width,
        console_theme().control_height()
    );
    assert_eq!(screen.stats().creates, creates, "no remount");
}

/// A theme switch that changes only paint values of a style layer reading
/// `current_theme()` re-sends only the node's paint.
#[test]
fn a_theme_switch_that_changes_only_paint_resends_only_paint() {
    let (screen, theme) = mount(|| {
        let theme = signal(Theme::light());
        let view = provide_theme(theme, || {
            node()
                .style(move || {
                    let t = &current_theme();
                    style().size(40, 40).radius(t.radius().md)
                })
                .test_id("card")
        });
        (theme, view)
    });
    let tree = screen.snapshot();
    let before = screen.stats();
    theme.set(console_theme());
    screen.run_until_idle();
    let after = screen.stats();
    assert_eq!(after.creates, before.creates, "no remount");
    assert_eq!(
        after.set_props,
        before.set_props + 1,
        "only Paint is re-sent"
    );
    assert_eq!(screen.snapshot(), tree);
}

/// A signal read next to `current_theme()` in a style closure is tracked.
#[test]
fn a_style_reading_the_theme_tracks_other_signals_it_reads() {
    let (screen, wide) = mount(|| {
        let wide = signal(false);
        let view = node()
            .style(move || {
                let t = &current_theme();
                let w = if wide.get() { 2.0 } else { 1.0 } * t.control_height();
                style().width(w).height(10)
            })
            .test_id("box");
        (wide, column(view).style(style().align_items(Align::Start)))
    });
    let h = Theme::light().control_height();
    let el = screen.get_by_test_id("box");
    assert_eq!(el.rect().expect("rect").width, h);
    wide.set(true);
    screen.run_until_idle();
    assert_eq!(el.rect().expect("rect").width, 2.0 * h);
}

/// A container's base style is its first layer: the user's `.style()`
/// overrides only the properties it sets.
#[test]
fn user_style_layers_over_the_base_style() {
    let screen = render(|| {
        column((
            column((text("a").test_id("a"), text("b").test_id("b")))
                .style(style().direction(Direction::Row)),
            row((text("c").test_id("c"), text("d").test_id("d"))).style(style().gap(10)),
        ))
        .style(style().align_items(Align::Start))
    });
    let (a, b) = (
        screen.get_by_test_id("a").rect().expect("rect"),
        screen.get_by_test_id("b").rect().expect("rect"),
    );
    assert!(
        b.x > a.x && b.y == a.y,
        "column turned into a row: {a:?} {b:?}"
    );
    let (c, d) = (
        screen.get_by_test_id("c").rect().expect("rect"),
        screen.get_by_test_id("d").rect().expect("rect"),
    );
    assert_eq!(
        d.x,
        c.x + c.width + 10.0,
        "row keeps its direction under a gap"
    );
}

/// A white 100 × 60 box at (100, 100) on white, casting `shadow`.
fn shadowed(shadow: Shadow) -> Screen {
    render(move || {
        column((node().style(
            style()
                .size(100, 60)
                .background(rgb(0xffffff))
                .shadow(shadow),
        ),))
        .style(
            style()
                .padding(100)
                .align_items(Align::Start)
                .background(rgb(0xffffff)),
        )
    })
}

/// The gray level (0 black, 255 white) of a pixel of a gray frame.
fn gray(shot: &kivact::test::Screenshot, x: u32, y: u32) -> u32 {
    shot.pixel(x, y).expect("in the frame") & 0xff
}

#[test]
fn box_shadows_blur_like_css_box_shadow() {
    // CSS: a blur radius of 16 is a gaussian of standard deviation 8.
    // Beside a long edge the shadow is 0.5 × erfc(d / (8 √2)) dark at a
    // distance d: 0.47 at 0.5 px, 0.14 at 8.5 px, 0.07 at 12.5 px, below
    // 0.1% beyond 3.6 deviations (29 px).
    let screen = shadowed(Shadow::new(0.0, 0.0, 16.0, rgb(0x000000)));
    let shot = screen.screenshot();
    let left = |d: u32| gray(&shot, 100 - d, 130);
    let expect = |d: u32, dark: f64| {
        let want = 255.0 * (1.0 - dark);
        let got = f64::from(left(d));
        assert!(
            (got - want).abs() <= 10.0,
            "{d} px left of the box: gray {got}, a gaussian gives {want:.0}"
        );
    };
    expect(1, 0.475);
    expect(9, 0.144);
    expect(13, 0.067);
    assert_eq!(left(30), 255, "nothing beyond the blur's reach");
    assert_eq!(gray(&shot, 150, 130), 255, "nothing under the box");
    // A spread grows the shape before the blur; an offset moves it.
    let screen = shadowed(Shadow::new(0.0, 10.0, 0.0, rgb(0x000000)).spread(4.0));
    let shot = screen.screenshot();
    assert_eq!(gray(&shot, 97, 130), 0, "the spread reaches 4 px out");
    assert_eq!(gray(&shot, 95, 130), 255);
    assert_eq!(gray(&shot, 150, 172), 0, "offset down by 10, spread by 4");
    assert_eq!(gray(&shot, 150, 175), 255);
}

#[test]
fn a_shadow_reaching_into_the_window_is_drawn_from_an_unseen_box() {
    // The box starts 44 px below the 600 px window; its blur (40: a
    // deviation of 20, drawn 50 px out) still darkens the bottom row.
    let screen = render(|| {
        column((
            node().style(style().size(10, 644)),
            node().style(
                style()
                    .size(200, 40)
                    .background(rgb(0xffffff))
                    .shadow(Shadow::new(0.0, 0.0, 40.0, rgb(0x000000))),
            ),
        ))
        .style(style().align_items(Align::Start).background(rgb(0xffffff)))
    });
    let shot = screen.screenshot();
    let bottom = gray(&shot, 100, shot.height - 1);
    assert!(
        bottom < 254,
        "the shadow shows at the bottom row: gray {bottom}"
    );
}
