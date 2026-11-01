//! Role: Tests for style and skin invariance across components.
//!
//! Style/skin invariance over the external components and the skinned
//! standard controls: the Business and Console skins
//! change paint (gradient, glow shadows, radius, focus ring width, hover and
//! pressed transforms) but not semantics or hit regions. Canvas text follows
//! the same drawing transform as every other primitive.

use kivact::Binding;
use kivact::control::canvas;
use kivact::draw::{Draw, Point, Size};
use kivact::draw::{Layer, LineCap, LineJoin, Stroke, Transform};
use kivact::draw::{Path, TextStyle};
use kivact::prelude::*;
use kivact::style::current_theme;
use kivact::test::{RenderOptions, Screen, SemanticNode, render, render_with};
use kivact::ui::ListItem;
use kivact_external_component::skin::{self, Skin};
use kivact_external_component::{Span, lazy_list, provide_skin, search_box, sparkline, timeline};

fn board(initial: Skin) -> impl IntoView {
    let skin = signal(initial);
    provide_skin(skin, move || {
        let q = signal(String::new());
        let sel = signal(None::<Span>);
        let data = signal((0..24).map(|i| (i % 7) as f32).collect::<Vec<f32>>());
        let on = signal(false);
        let pick = signal(Some("auto"));
        let rows = signal((0..200u32).collect::<Vec<_>>());
        let look = skin::look();
        column((
            {
                let label = (move || format!("Skin: {}", skin.get().name())).into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
                    .style(
                        style()
                            .padding(8)
                            .min_height(32)
                            .background(ColorToken::SurfaceAlt)
                            .focus(style().ring(2, ColorToken::Focus)),
                    )
            }
            .on_press(move || skin.update(|s| *s = s.other()))
            .style(move || {
                let t = &current_theme();
                skin::button(look(), t)
            })
            .test_id("switch"),
            search_box("Search", q),
            timeline("Load", data, sel).height(60.0),
            {
                let checked: Binding<bool> = (on).into();
                node()
                    .role(Role::CheckBox)
                    .focusable(true)
                    .name("Streaming")
                    .checked(checked)
                    .on_press(move || checked.set(!checked.get_untracked()))
                    .child(text(move || {
                        (if checked.get() { "[x]" } else { "[ ]" }).to_owned()
                    }))
                    .child(text("Streaming"))
                    .style(
                        style()
                            .padding(8)
                            .min_height(32)
                            .focus(style().ring(2, ColorToken::Focus)),
                    )
            },
            {
                let options = vec![
                    ListItem::new("auto", "Auto"),
                    ListItem::new("manual", "Manual"),
                ];
                let group = kivact::interaction::use_list(
                    options.clone(),
                    kivact::interaction::ListSelection::Single(pick.into()),
                )
                .policy(kivact::interaction::ListPolicy::radio());
                let members = options
                    .into_iter()
                    .map(|option| {
                        let label = option.label();
                        node()
                            .part(
                                group
                                    .item(*option.key())
                                    .role(Role::Option)
                                    .name(label.clone()),
                            )
                            .child(text(label))
                    })
                    .collect::<Vec<_>>();
                column(members)
                    .part(group.list().role(Role::ListBox))
                    .style(style().gap(4))
            }
            .name("Mode")
            .style(move || {
                let t = &current_theme();
                skin::field(look(), t)
            }),
            lazy_list(
                "Rows",
                rows,
                |n| *n,
                |_, n: Value<u32>| {
                    // The timeline's drawing function reused in a local draw layer.
                    node()
                        .style(style().size(60, 12))
                        .draw(move |d, s| {
                            let v = n.get();
                            sparkline(d, s, &[v as f32 % 5.0, 3.0, 1.0], ColorToken::Accent)
                        })
                        .test_id("spark")
                },
            )
            .row_height(24.0)
            .style(style().height(120))
            .item_name(|n| format!("row {n}")),
        ))
        .style(move || {
            let t = &current_theme();
            skin::panel(look(), t)
        })
        .style(style().width(500))
    })
}

fn screen(initial: Skin) -> Screen {
    render_with(
        RenderOptions {
            theme: initial.theme(),
            ..RenderOptions::from_env()
        },
        move || board(initial),
    )
}

fn strip(mut n: SemanticNode) -> SemanticNode {
    // The switch button's label names the skin; everything else must match.
    if n.name.starts_with("Skin: ") {
        n.name = "Skin".into();
    }
    n.focused = false;
    n.children = n.children.into_iter().map(strip).collect();
    n
}

#[test]
fn two_skins_keep_semantics_and_hit_regions() {
    let a = screen(Skin::Business);
    let b = screen(Skin::Console);
    assert_eq!(strip(a.snapshot()), strip(b.snapshot()));
    let rect = |s: &Screen| {
        s.get_by_role(Role::Canvas, "Load")
            .rect()
            .map(|r| (r.width, r.height))
    };
    assert_eq!(
        rect(&a),
        rect(&b),
        "the canvas (and its regions) has the same size"
    );
    for s in [&a, &b] {
        let t = s.get_by_role(Role::Canvas, "Load");
        t.drag(Point::new(10.0, 20.0), Point::new(200.0, 20.0));
        t.get_by_role(Role::Slider, "Selection end")
            .drag(Point::new(200.0, 20.0), Point::new(300.0, 20.0));
        s.get_by_role(Role::ListItem, "row 2").click();
        assert!(s.errors().is_empty(), "{:?}", s.errors());
    }
    assert_eq!(
        strip(a.snapshot()),
        strip(b.snapshot()),
        "same interactions, same result"
    );
    assert_eq!(
        a.get_by_role(Role::Canvas, "Load").value(),
        b.get_by_role(Role::Canvas, "Load").value()
    );
    assert!(a.all_by_test_id("spark").iter().all(|e| e.scene_len() > 0));
}

#[test]
fn switching_skin_at_runtime_repaints_without_remount() {
    let s = screen(Skin::Business);
    let before = s.snapshot();
    let stats = s.stats();
    s.get_by_role(Role::Button, "Skin: Business").click();
    s.get_by_role(Role::Button, "Skin: Console");
    let after = s.stats();
    assert_eq!(after.creates, stats.creates, "no node was recreated");
    assert!(after.set_props > stats.set_props + 10, "paint re-resolved");
    assert_eq!(strip(before), strip(s.snapshot()));
    // Back again.
    s.get_by_role(Role::Button, "Skin: Console")
        .press_key(Key::Enter);
    s.get_by_role(Role::Button, "Skin: Business");
    assert!(s.errors().is_empty(), "{:?}", s.errors());
}

#[test]
fn element_and_canvas_transforms_draw_rotated_text() {
    let s = render(|| {
        column((
            node()
                .style(move || {
                    let t = &current_theme();
                    skin::button(Skin::Console, t)
                })
                .test_id("ok"),
            // A rotated element is drawn rotated (it used to be rejected).
            node()
                .style(move || {
                    let t = &current_theme();
                    skin::button(Skin::Console, t)
                })
                .style(style().rotate(15))
                .test_id("rotated"),
        ))
    });
    assert!(s.errors().is_empty(), "{:?}", s.errors());
    // Canvas text and paths share the renderer's transform.
    let s = render(|| {
        canvas(|d: &mut Draw, _s: Size| {
            d.layer(Layer::new().transform(Transform::new().rotate(17)), |d| {
                d.stroke_path(
                    &Path::new()
                        .move_to(Point::new(0.0, 0.0))
                        .line_to(Point::new(10.0, 0.0)),
                    &Stroke::new(1.0).cap(LineCap::Round).join(LineJoin::Round),
                    ColorToken::Text,
                );
                d.text("tilted", Point::new(0.0, 0.0), TextStyle::default());
            });
        })
        .style(style().size(40, 40))
        .name("rot")
    });
    let _ = s.screenshot();
    assert!(s.errors().is_empty(), "{:?}", s.errors());
    assert_eq!(
        s.get_by_role(Role::Canvas, "rot").scene_len(),
        4,
        "transform, path, text and the end of the transform"
    );
}
