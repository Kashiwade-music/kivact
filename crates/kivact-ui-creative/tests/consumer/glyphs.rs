//! Role: The kit's icons, label tones and key glyphs: what a consumer draws with them.

use crate::support::clean;
use crate::time_support::app_dope_sheet;
use kivact::draw::Point;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact_ui_creative as ui;

/// The screenshot pixel at `(x, y)` logical px (the window is not scaled).
fn pixel_at(screen: &Screen, x: f32, y: f32) -> u32 {
    screen
        .screenshot()
        .pixel(x as u32, y as u32)
        .expect("a pixel in the window")
}

#[test]
fn a_dope_sheet_key_is_drawn_with_its_glyph() {
    fn key(glyph: ui::KeyGlyph) -> Screen {
        kivact::test::render_with_handle(crate::support::options(), move || {
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
