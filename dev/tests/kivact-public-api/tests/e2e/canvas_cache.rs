//! Role: Retained canvas placement, inherited text, tint and raster regressions.
use super::paint::{RED, WHITE, pixel};
use kivact::control::canvas;
use kivact::draw::{Point, Rect, TextStyle};
use kivact::prelude::*;
use kivact::style::{Color, Position};
use kivact::view::provide_theme;

#[test]
fn a_retained_scene_moves_without_leaving_its_old_origin_and_survives_memory_pressure() {
    let (screen, offset) = crate::support::mount_with(
        kivact::test::RenderOptions {
            window: kivact::test::WindowMode::Real,
            ..Default::default()
        },
        || {
            let offset = signal(0.0);
            (
                offset,
                node()
                    .child(
                        canvas(|d, _| {
                            d.fill_rect(Rect::new(2., 2., 12., 12.), rgb(RED));
                            d.text(
                                "H",
                                Point::new(2., 18.),
                                TextStyle {
                                    color: Color::BLACK,
                                    size: 24.,
                                    ..Default::default()
                                },
                            );
                        })
                        .style(move || {
                            style()
                                .position(Position::Absolute)
                                .left(offset.get())
                                .size(60, 50)
                        }),
                    )
                    .style(style().size(180, 60).background(Color::WHITE)),
            )
        },
    );
    let before = screen.screenshot();
    offset.set(40.0);
    screen.run_until_idle();
    let moved = screen.screenshot();
    assert_eq!(pixel(&moved, 5, 5), WHITE);
    assert_eq!(pixel(&moved, 45, 5), RED);
    for y in 0..50 {
        for x in 0..60 {
            assert_eq!(
                pixel(&before, x, y),
                pixel(&moved, x + 40, y),
                "placement {x},{y}"
            );
        }
    }
    screen.memory_warning();
    screen.resize(181., 60.);
    let rebuilt = screen.screenshot();
    for y in 0..50 {
        for x in 0..100 {
            assert_eq!(pixel(&moved, x, y), pixel(&rebuilt, x, y));
        }
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn retained_text_tracks_inherited_theme_tint_and_raster() {
    let (screen, (theme, opacity)) = crate::support::mount_with(
        kivact::test::RenderOptions {
            window: kivact::test::WindowMode::Real,
            fonts: kivact::test::TestFonts::Only(vec![
                std::sync::Arc::from(
                    &include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../../crates/arvane/assets/fonts/Tuffy.ttf"
                    ))[..],
                ),
                std::sync::Arc::from(
                    &include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../../crates/arvane/tests/fonts/InterVariable.ttf"
                    ))[..],
                ),
            ]),
            ..Default::default()
        },
        || {
            let base = Theme::light();
            let theme = signal(base.clone().with_text(kivact::style::theme::Typography {
                family: Some("Tuffy".into()),
                ..base.text().clone()
            }));
            let opacity = signal(1.0);
            (
                (theme, opacity),
                provide_theme(theme, move || {
                    canvas(|d, _| {
                        d.text(
                            "WWWiii",
                            Point::new(5., 5.),
                            TextStyle {
                                color: Color::BLACK,
                                size: 24.,
                                ..Default::default()
                            },
                        );
                        d.fill_rect(Rect::new(70., 5., 12., 12.), rgb(RED));
                    })
                    .style(move || {
                        style()
                            .font_family(
                                kivact::style::current_theme()
                                    .text()
                                    .family
                                    .as_deref()
                                    .unwrap_or("Tuffy"),
                            )
                            .size(120, 70)
                            .opacity(opacity.get())
                            .background(Color::WHITE)
                    })
                }),
            )
        },
    );
    let before = screen.screenshot();
    let base = Theme::light();
    let changed = base.clone().with_text(kivact::style::theme::Typography {
        family: Some("Inter Variable".into()),
        ..base.text().clone()
    });
    theme.set(changed);
    screen.run_until_idle();
    let inherited = screen.screenshot();
    assert!(
        before.pixels != inherited.pixels,
        "inherited theme family changes retained glyph shapes"
    );
    opacity.set(0.5);
    screen.run_until_idle();
    let tinted = screen.screenshot();
    assert_ne!(
        pixel(&inherited, 75, 10),
        pixel(&tinted, 75, 10),
        "tint invalidates local color"
    );
    screen.set_scale(2.);
    let scaled = screen.screenshot();
    assert_eq!(scaled.width, tinted.width * 2);
    assert_eq!(pixel(&scaled, 150, 20), pixel(&tinted, 75, 10));
    screen.set_scale(1.);
    let restored = screen.screenshot();
    assert_eq!(restored.pixels, tinted.pixels, "raster scale round trip");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
