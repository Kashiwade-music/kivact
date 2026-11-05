//! Role: Theme density and halo pixels follow the public game theme.

use kivact::prelude::*;
use kivact::style::pct;
use kivact_ui_game as g;
use kivact_ui_game::ThemeOptions;

#[test]
fn consumer_theme_density_and_halo_pixels_follow_public_theme() {
    let (screen, (mode, density)) =
        kivact::test::render_with_handle(crate::support::options(true), || {
            let mode = signal(kivact::style::ThemeMode::Light);
            let density = signal(g::Density::Comfortable);
            (
                (mode, density),
                kivact::style::provide_theme(
                    move || {
                        g::theme_with(
                            mode.get(),
                            ThemeOptions {
                                density: density.get(),
                            },
                        )
                    },
                    || {
                        column((
                            kivact::ui::button("Action")
                                .variant(kivact_ui_game::Slant)
                                .test_id("themed-action"),
                            g::halo().style(style().size(34, 34)).test_id("small-halo"),
                        ))
                        .style(
                            style()
                                .size(pct(100), pct(100))
                                .padding(24)
                                .gap(24)
                                .background(ColorToken::Surface),
                        )
                    },
                ),
            )
        });
    let light = screen.screenshot();
    assert_eq!(light.pixel(790, 590), Some(0xffffff));
    assert_eq!(
        screen
            .get_by_test_id("themed-action")
            .rect()
            .map(|r| r.height),
        Some(34.)
    );
    assert_eq!(
        screen
            .get_by_test_id("small-halo")
            .rect()
            .map(|r| (r.width, r.height)),
        Some((34., 34.))
    );
    mode.set(kivact::style::ThemeMode::Dark);
    density.set(g::Density::Compact);
    screen.run_until_idle();
    let dark = screen.screenshot();
    assert_eq!(dark.pixel(790, 590), Some(0x0f1c2c));
    assert_eq!(
        screen
            .get_by_test_id("themed-action")
            .rect()
            .map(|r| r.height),
        Some(28.)
    );
    assert_ne!(light.hash(), dark.hash());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
