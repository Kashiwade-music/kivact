//! Role: The kit's icons, label tones and key glyphs: what a consumer draws with them.

use super::*;
use kivact::draw::{Draw, Offscreen, Rect, RgbaImage};

/// The screenshot pixel at `(x, y)` logical px (the window is not scaled).
fn pixel_at(screen: &Screen, x: f32, y: f32) -> u32 {
    screen
        .screenshot()
        .pixel(x as u32, y as u32)
        .expect("a pixel in the window")
}

/// How many pixels of the 16px box at `rect` differ from `background`.
fn inked(screen: &Screen, rect: Rect, background: u32) -> usize {
    screen
        .screenshot()
        .crop(rect.x as u32, rect.y as u32, 16, 16)
        .pixels
        .iter()
        .filter(|pixel| **pixel != background)
        .count()
}

#[test]
fn the_added_icons_draw_inside_their_sixteen_pixel_box() {
    let icons = [
        "eye-off",
        "lock-open",
        "pin",
        "chevrons-left",
        "chevrons-right",
        "step-back",
        "step-forward",
        "play-backward",
        "clock",
        "link-off",
    ];
    let count = icons.len();
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), move || {
        (
            (),
            row(icons
                .into_iter()
                .enumerate()
                .map(|(index, glyph)| kivact::control::icon(glyph).test_id(format!("icon-{index}")))
                .collect::<Vec<_>>())
            .style(style().gap(24).padding(24).background(rgb(0x000000))),
        )
    });
    for index in 0..count {
        let rect = screen
            .get_by_test_id(&format!("icon-{index}"))
            .rect()
            .expect("a drawn icon");
        assert_eq!((rect.width, rect.height), (16.0, 16.0), "icon {index}");
        assert!(inked(&screen, rect, 0x000000) > 8, "icon {index} is empty");
    }
    clean(&screen);
}

#[test]
fn the_theme_puts_every_kit_icon_in_its_icon_set() {
    let theme = || ui::theme(kivact::style::ThemeMode::Dark);
    assert!(
        theme() == theme(),
        "the kit's themes share one recipe table and icon set, so an equal theme compares equal"
    );
    let kit = theme();
    let icons = kit.icons();
    for name in ["x", "chevron-down", "pin", "play", "link-off"] {
        assert!(
            icons.resolve(&kivact::style::Icon::named(name)).is_some(),
            "{name} is in the kit's icon set"
        );
    }
    // The kit draws its own close mark and chevron instead of the standard ones.
    let standard = kivact::style::IconSet::standard();
    assert_ne!(icons.get("x"), standard.get("x"));
    assert_ne!(icons.get("chevron-down"), standard.get("chevron-down"));
}

#[test]
fn label_tones_resolve_against_the_palette() {
    let iris = ui::theme(kivact::style::ThemeMode::Dark).get::<ui::Tokens>();
    let coral = ui::theme(kivact::style::ThemeMode::Dark)
        .accent(ui::Accent::Coral)
        .get::<ui::Tokens>();
    assert_eq!(ui::LabelTone::default(), ui::LabelTone::Normal);
    assert_eq!(ui::LabelTone::Normal.color(&iris), iris.text);
    assert_eq!(ui::LabelTone::Muted.color(&iris), iris.muted);
    assert_eq!(ui::LabelTone::Accent.color(&iris), iris.accent_high);
    assert_eq!(ui::LabelTone::Warning.color(&iris), iris.warning);
    assert_eq!(ui::LabelTone::Error.color(&iris), iris.x);
    assert_ne!(
        ui::LabelTone::Accent.color(&iris),
        ui::LabelTone::Accent.color(&coral)
    );
    let custom = rgb(0x123456);
    assert_eq!(ui::LabelTone::Custom(custom).color(&iris), custom);
    assert_eq!(ui::LabelTone::Custom(custom).color(&coral), custom);
}

/// `glyph` painted white at the center of a 40px transparent image.
fn glyph_image(glyph: ui::KeyGlyph) -> RgbaImage {
    Offscreen::new()
        .render(40, 40, |draw: &mut Draw, _| {
            glyph.paint(draw, Point::new(20.0, 20.0), 10.0, rgb(0xffffff));
        })
        .expect("an offscreen render")
}

/// The alpha at the offset `(dx, dy)` from the glyph's center.
fn alpha(image: &RgbaImage, dx: i32, dy: i32) -> u8 {
    let (x, y) = ((20 + dx) as usize, (20 + dy) as usize);
    image.rgba[(y * image.width as usize + x) * 4 + 3]
}

#[test]
fn key_glyphs_have_four_distinct_shapes() {
    assert_eq!(ui::KeyGlyph::default(), ui::KeyGlyph::Diamond);
    // Diamond, radius 10: covers the axes out to 7, not the diagonal at 6.
    let diamond = glyph_image(ui::KeyGlyph::Diamond);
    assert_eq!(alpha(&diamond, 7, 0), 255);
    assert_eq!(alpha(&diamond, 0, -7), 255);
    assert_eq!(alpha(&diamond, -7, 0), 255);
    assert_eq!(alpha(&diamond, 6, 6), 0);
    // Square, half side 8: covers the diagonal, stops short of 9.
    let square = glyph_image(ui::KeyGlyph::Square);
    assert_eq!(alpha(&square, 7, 7), 255);
    assert_eq!(alpha(&square, -8, -8), 255);
    assert_eq!(alpha(&square, 9, 0), 0);
    // Circle, radius 8: inside on an axis, outside at the square's corner.
    let circle = glyph_image(ui::KeyGlyph::Circle);
    assert_eq!(alpha(&circle, 6, 0), 255);
    assert_eq!(alpha(&circle, 0, 6), 255);
    assert_eq!(alpha(&circle, 7, 7), 0);
    assert_eq!(alpha(&circle, 9, 0), 0);
    // Half diamond: the left half is dimmed to 35%, the right one is solid.
    let half = glyph_image(ui::KeyGlyph::HalfDiamond);
    assert_eq!(alpha(&half, 5, 0), 255);
    let dimmed = alpha(&half, -5, 0);
    assert!((80..=100).contains(&dimmed), "left half alpha {dimmed}");
    assert_eq!(alpha(&half, 7, 7), 0);
    assert_eq!(alpha(&half, -7, 7), 0);
}

#[test]
fn a_dope_sheet_key_is_drawn_with_its_glyph() {
    fn key(glyph: ui::KeyGlyph) -> Screen {
        kivact::test::render_with_handle(crate::options(), move || {
            let items = signal(vec![
                ui::Keyframe::new(1, 0, 40.0, "Key")
                    .color(rgb(0xffd25a))
                    .glyph(glyph),
            ]);
            (
                (),
                app_dope_sheet(items, ui::TimeState::new())
                    .style(style().size(400, 120))
                    .test_id("dope"),
            )
        })
        .0
    }
    // Logical time is the marker center; the row is centered below the ruler.
    let sample = |screen: &Screen, dx: f32, dy: f32| {
        let rect = screen.get_by_test_id("dope").rect().expect("drawn");
        pixel_at(screen, rect.x + 240.0 + dx, rect.y + 32.0 + dy)
    };
    let diamond = key(ui::KeyGlyph::Diamond);
    let bounds = diamond
        .get_by_role(Role::Button, "Key")
        .rect()
        .expect("key");
    assert_eq!(bounds.center(), Point::new(240.0, 32.0));
    let square = key(ui::KeyGlyph::Square);
    let circle = key(ui::KeyGlyph::Circle);
    let half = key(ui::KeyGlyph::HalfDiamond);
    let ink = 0xe4e5e9;
    // On the diagonal at (3, 3): only the square (half side 4) covers it.
    assert_ne!(sample(&diamond, 3.0, 3.0), ink);
    assert_eq!(sample(&square, 3.0, 3.0), ink);
    assert_ne!(sample(&circle, 3.0, 3.0), ink);
    // On the axis at 2: the diamond (radius 5) covers it as well.
    assert_eq!(sample(&diamond, 2.0, 0.0), ink);
    assert_ne!(sample(&square, 4.0, 0.0), ink);
    assert_ne!(sample(&circle, 4.0, 0.0), ink);
    // The left half of a half diamond is dimmed, the right one solid.
    assert_eq!(sample(&half, 3.0, 0.0), ink);
    assert_ne!(sample(&half, -3.0, 0.0), ink);
    assert_ne!(sample(&half, -3.0, 0.0), sample(&diamond, -3.0, 0.0));
    for screen in [&diamond, &square, &circle, &half] {
        clean(screen);
    }
}
