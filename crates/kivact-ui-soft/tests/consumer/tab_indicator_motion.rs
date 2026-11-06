//! Role: Check that Soft tab indicators crossfade in place.

use kivact::interaction::ListItem;
use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::test::RenderOptions;
use kivact::test::render_with_handle;
use kivact::ui::tabs;
use kivact::view::Role;
use std::time::Duration;

#[test]
fn selection_crossfades_both_indicators_in_place() {
    for reduced_motion in [false, true] {
        let (screen, selected) = render_with_handle(
            if reduced_motion {
                RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)).reduced_motion()
            } else {
                RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light))
            },
            || {
                let selected = signal("a");
                let tabs = tabs(
                    "Views",
                    vec![ListItem::new("a", "Alpha"), ListItem::new("b", "Bravo")],
                    selected,
                );
                (selected, tabs)
            },
        );
        let sample = |label| {
            let bounds = screen.get_by_role(Role::Tab, label).rect().unwrap();
            screen
                .screenshot()
                .pixel(
                    (bounds.x + bounds.width / 2.0) as u32,
                    (bounds.y + bounds.height - 1.0) as u32,
                )
                .unwrap()
        };
        let ink = sample("Alpha");
        let clear = sample("Bravo");
        assert_eq!(ink, 0x1b1b1b);
        assert!(clear > ink);
        selected.set("b");
        screen.run_until_idle();
        screen.advance(Duration::from_millis(48));
        for pixel in [sample("Alpha"), sample("Bravo")] {
            assert!(pixel > ink && pixel < clear, "pixel {pixel:06x}");
        }
        screen.advance(Duration::from_millis(120));
        assert_eq!(sample("Alpha"), clear);
        assert_eq!(sample("Bravo"), ink);
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
