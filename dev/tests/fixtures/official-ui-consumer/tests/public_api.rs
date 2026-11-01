//! Role: Operate themed standard UI from an independent Cargo workspace.

use kivact::control::canvas;
use kivact::draw::{Rect, RegionId};
use kivact::prelude::*;
use kivact::primitive::NodeExt;
use kivact::test::{RenderOptions, Screen, WindowMode, render_with};
use kivact::ui::{ButtonVariant, button, checkbox};
use kivact::view::A11yAction;
use kivact::view::provide_theme;
use official_ui_external_consumer::external_action;
use std::time::Duration;

fn gpu_requested() -> bool {
    std::env::var("KIVACT_TEST_GPU").is_ok_and(|value| value == "1")
}

fn render<V: IntoView>(root: impl FnOnce() -> V + 'static) -> Screen {
    let gpu = gpu_requested();
    render_with(
        RenderOptions {
            gpu,
            window: WindowMode::Real,
            ..Default::default()
        },
        root,
    )
}

#[test]
fn official_and_third_party_builders_share_modifiers_and_activation() {
    let screen = render(|| {
        let count = signal(0);
        column((
            provide_theme(
                kivact_ui_pop::theme(kivact::style::ThemeMode::Light),
                move || {
                    button("Pop action")
                        .test_id("pop")
                        .on_press(move || count.update(|n| *n += 1))
                },
            ),
            provide_theme(
                kivact_ui_game::theme(kivact::style::ThemeMode::Light),
                move || {
                    button("Game action")
                        .test_id("game")
                        .on_press(move || count.update(|n| *n += 1))
                },
            ),
            provide_theme(
                kivact_ui_creative::theme(kivact::style::ThemeMode::Dark),
                move || {
                    button("Creative action")
                        .test_id("creative")
                        .on_press(move || count.update(|n| *n += 1))
                },
            ),
            external_action("External action")
                .test_id("external")
                .on_press(move || count.update(|n| *n += 1)),
            text(move || count.get().to_string()).test_id("count"),
        ))
        .style(style().padding(24).gap(12))
    });
    screen.get_by_test_id("pop").click();
    screen.get_by_test_id("game").press_key(Key::Space);
    screen.get_by_test_id("creative").a11y(A11yAction::Activate);
    screen.get_by_test_id("external").press_key(Key::Enter);
    assert_eq!(screen.get_by_test_id("count").text(), "4");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_third_party_can_publish_and_operate_drawn_items_and_mixed_controls() {
    let screen = render(|| {
        let checked = signal(false);
        let mixed = signal(true);
        let picked = signal(0);
        provide_theme(
            kivact_ui_pop::theme(kivact::style::ThemeMode::Light),
            move || {
                column((
                    checkbox("Partial selection", checked)
                        .indeterminate(mixed)
                        .test_id("partial"),
                    canvas(|draw, _| {
                        for (id, x) in [(10, 0.0), (20, 100.0)] {
                            draw.region(RegionId(id), Rect::new(x, 0.0, 90.0, 40.0));
                        }
                    })
                    .region(
                        RegionId(10),
                        kivact::primitive::part(|input| input)
                            .role(Role::Button)
                            .name("First item")
                            .on_a11y_event(A11yAction::Select, move |event| {
                                picked.set(event.region.map_or(0, |region| region.0));
                            }),
                    )
                    .region(
                        RegionId(20),
                        kivact::primitive::part(|input| input)
                            .role(Role::Button)
                            .name("Second item")
                            .on_a11y_event(A11yAction::Select, move |event| {
                                picked.set(event.region.map_or(0, |region| region.0));
                            }),
                    )
                    .name("Independent editor")
                    .style(style().size(200, 40))
                    .on_a11y_event(A11yAction::Select, move |event| {
                        picked.set(event.region.map_or(0, |region| region.0));
                    }),
                    text(move || picked.get().to_string()).test_id("picked"),
                    external_action("Clear partial").on_press(move || mixed.set(false)),
                ))
                .style(style().padding(20).gap(8))
            },
        )
    });
    let partial = screen.get_by_test_id("partial");
    assert!(partial.is_indeterminate());
    assert_eq!(partial.is_checked(), Some(false));
    screen
        .get_by_role(Role::Button, "Second item")
        .a11y(A11yAction::Select);
    assert_eq!(screen.get_by_test_id("picked").text(), "20");
    screen.get_by_role(Role::Button, "Clear partial").click();
    assert!(!partial.is_indeterminate());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn all_three_numeric_controls_receive_accessibility_values() {
    let screen = render(|| {
        let pop = signal(64.0);
        let game = signal(64.0);
        let creative = signal(64.0);
        column((
            provide_theme(
                kivact_ui_pop::theme(kivact::style::ThemeMode::Light),
                move || kivact::ui::slider("Pop range", pop),
            ),
            provide_theme(
                kivact_ui_game::theme(kivact::style::ThemeMode::Light),
                move || kivact::ui::slider("Game range", game),
            ),
            kivact_ui_creative::value_slider("Creative range", creative).range(0.0, 100.0),
        ))
        .style(style().width(400).gap(20).padding(20))
    });
    for name in ["Pop range", "Game range", "Creative range"] {
        let control = screen.get_by_role(Role::Slider, name);
        control.a11y_with_value(A11yAction::SetValue, "37");
        assert_eq!(
            control.value_range().2,
            Some(37.0),
            "{name}: requested value"
        );
        control.a11y_with_value(A11yAction::SetValue, "200");
        assert_eq!(control.value_range().2, Some(100.0), "{name}: range clamp");
        control.a11y_with_value(A11yAction::SetValue, "NaN");
        assert_eq!(
            control.value_range().2,
            Some(100.0),
            "{name}: finite values only"
        );
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_third_party_builder_can_change_tab_stops_without_replacing_nodes() {
    let screen = render(|| {
        let selected = signal(false);
        column((
            external_action("Before"),
            external_action("First").focusable(move || !selected.get()),
            external_action("Second").focusable(selected),
            external_action("After"),
            external_action("Select second").on_press(move || selected.set(true)),
        ))
    });
    screen.get_by_role(Role::Button, "Before").click();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "First").is_focused());
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    let created = screen.stats().creates;
    screen.get_by_role(Role::Button, "Select second").click();
    screen.get_by_role(Role::Button, "Before").click();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "Second").is_focused());
    assert_eq!(screen.stats().creates, created);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn component_motion_has_a_middle_frame_and_honors_reduced_motion() {
    let screen = kivact::test::render_with(
        kivact::test::RenderOptions {
            reduced_motion: false,
            gpu: gpu_requested(),
            window: WindowMode::Real,
            ..Default::default()
        },
        || {
            column((
                provide_theme(
                    kivact_ui_pop::theme(kivact::style::ThemeMode::Light),
                    || {
                        button("Pop transition")
                            .variant(kivact_ui_pop::ButtonTone::Soft)
                            .test_id("pop-motion")
                            .style(style().width(200))
                    },
                ),
                provide_theme(
                    kivact_ui_creative::theme(kivact::style::ThemeMode::Dark),
                    || {
                        node()
                            .style(style().background(rgb(0x141519)).padding(10))
                            .child(
                                button("Creative transition")
                                    .variant(ButtonVariant::Secondary)
                                    .test_id("creative-motion")
                                    .style(style().width(200)),
                            )
                    },
                ),
            ))
            .style(style().padding(20).gap(20))
        },
    );
    for id in ["pop-motion", "creative-motion"] {
        screen.mouse_move(600., 400.);
        screen.advance(Duration::from_millis(200));
        let control = screen.get_by_test_id(id);
        let rect = control.rect().expect("button laid out");
        let pixels = || {
            screen
                .screenshot()
                .crop(
                    rect.x as u32,
                    rect.y as u32,
                    rect.width as u32,
                    rect.height as u32,
                )
                .hash()
        };
        let base = pixels();
        control.hover();
        screen.advance(Duration::from_millis(50));
        let middle = pixels();
        screen.advance(Duration::from_millis(150));
        let end = pixels();
        assert_ne!(base, middle, "{id}: hover started");
        assert_ne!(middle, end, "{id}: distinct animation middle");
        screen.mouse_move(600., 400.);
        screen.advance(Duration::from_millis(200));
        screen.set_reduced_motion(true);
        control.hover();
        assert_eq!(
            pixels(),
            end,
            "{id}: reduced motion skips the intermediate frame"
        );
        screen.set_reduced_motion(false);
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

mod fonts {
    //! Role: Verify mixed-script families through the public native App API.

    use kivact::control::{rich_text, span};
    use kivact::prelude::*;
    use kivact::style::Align;
    use kivact::test::{RenderOptions, TestFonts, WindowMode, render_with};

    const INTER: &[u8] = include_bytes!(
        "../../../../../examples/kivact-ui-pop/assets/fonts/inter/Inter[opsz,wght].ttf"
    );
    const JAPANESE: &[u8] = include_bytes!(
        "../../../../../examples/kivact-ui-pop/assets/fonts/notosansjp/NotoSansJP[wght].ttf"
    );

    #[test]
    fn explicit_families_choose_latin_and_japanese_before_system_fallback() {
        let gpu = std::env::var("KIVACT_TEST_GPU").is_ok_and(|value| value == "1");
        let screen = render_with(
            RenderOptions {
                gpu,
                window: WindowMode::Real,
                fonts: TestFonts::Only(vec![INTER.into(), JAPANESE.into()]),
                ..Default::default()
            },
            || {
                column((
                    text("WWWW first")
                        .test_id("latin")
                        .style(style().font_size(16.0).font_family("Inter")),
                    text("プライマリ")
                        .test_id("japanese")
                        .style(style().font_size(16.0).font_family("Noto Sans JP")),
                    text("WWWW first").test_id("latin-list").style(
                        style().font_size(16.0).font_family(
                            "Missing optional face, Inter, 'Noto Sans JP', sans-serif",
                        ),
                    ),
                    text("プライマリ").test_id("japanese-list").style(
                        style()
                            .font_size(16.0)
                            .font_family("Inter, 'Noto Sans JP', sans-serif"),
                    ),
                    rich_text(
                        [span("プライマリ").font_family("Inter, 'Noto Sans JP', sans-serif")],
                    )
                    .test_id("rich-list")
                    .style(style().font_size(16.0)),
                ))
                .style(style().align_items(Align::Start).font_size(16.0))
            },
        );
        screen.run_until_idle();
        let width = |id| {
            screen
                .get_by_test_id(id)
                .rect()
                .expect("font measured")
                .width
        };
        assert!((width("latin") - width("latin-list")).abs() < 0.1);
        assert!(
            (width("japanese") - width("japanese-list")).abs() < 0.1,
            "explicit CJK fallback: {} vs {}",
            width("japanese"),
            width("japanese-list")
        );
        assert!((width("japanese") - width("rich-list")).abs() < 0.1);
        assert!(
            width("japanese") > 70.0,
            "five full-width Japanese glyphs at 16px"
        );
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[path = "public_api/interactions.rs"]
mod interactions;
