//! Role: Consumer E2E: imports only the public Kivact facade and this library.
use std::time::Duration;

use kivact::control::{rich_text, span};
use kivact::platform;
use kivact::prelude::*;
use kivact::style::pct;
use kivact::test::{RenderOptions, render_with};
use kivact::ui::{Command, ListItem, MenuEntry, Toast, Tone, toaster};
use kivact::view::A11yAction;
use kivact_ui_game as g;
use kivact_ui_game::ThemeOptions as _;
#[path = "consumer/boundary.rs"]
mod boundary;
#[path = "consumer/cleanup.rs"]
mod cleanup;
#[path = "consumer/interactions.rs"]
mod interactions;
#[path = "consumer/item_boundary.rs"]
mod item_boundary;
#[path = "consumer/keyed.rs"]
mod keyed;
#[path = "consumer/messages.rs"]
mod messages;

fn options(reduced_motion: bool) -> RenderOptions {
    RenderOptions {
        theme: g::theme(kivact::style::ThemeMode::Light),
        reduced_motion,
        ..RenderOptions::from_env()
    }
}

fn choice_options(items: &[(&'static str, &'static str)]) -> Vec<ListItem<&'static str>> {
    items
        .iter()
        .map(|(key, label)| ListItem::new(*key, *label))
        .collect()
}

#[test]
fn consumer_buttons_checks_mixed_states_and_modifiers() {
    let (screen, (count, checked, mixed)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let count = signal(0);
            let checked = signal(false);
            let mixed = signal(true);
            (
                (count, checked, mixed),
                column((
                    kivact::ui::button("Approve")
                        .variant(kivact_ui_game::Slant)
                        .test_id("approve")
                        .style(style().height(46))
                        .on_press(move || count.update(|v| *v += 1)),
                    kivact::ui::button("Disabled")
                        .variant(kivact_ui_game::Slant)
                        .disabled(true),
                    kivact::ui::checkbox("Permissions", checked).indeterminate(mixed),
                    kivact::ui::switch("Network", checked),
                ))
                .style(style().gap(20).padding(20)),
            )
        });
    let button = screen.get_by_test_id("approve");
    assert_eq!(button.accessibility_bounds().map(|r| r.height), Some(46.));
    button.click();
    button.press_key(Key::Space);
    assert_eq!(count.get_untracked(), 2);
    assert!(screen.get_by_role(Role::Button, "Disabled").is_disabled());
    let check = screen.get_by_role(Role::CheckBox, "Permissions");
    assert!(check.is_indeterminate());
    check.a11y(A11yAction::Toggle);
    assert!(checked.get_untracked());
    mixed.set(false);
    screen.run_until_idle();
    assert!(!check.is_indeterminate());
    screen
        .get_by_role(Role::Switch, "Network")
        .press_key(Key::Space);
    assert!(!checked.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_checkbox_switch_and_disclosure_follow_the_shared_toggle_rules() {
    let (screen, (check, switch, open)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let check = signal(false);
            let switch = signal(false);
            let open = signal(false);
            (
                (check, switch, open),
                column((
                    kivact::ui::checkbox("Notify", check),
                    kivact::ui::switch("Network", switch),
                    kivact::ui::disclosure("Settings", open, || text("Inspector content")),
                    kivact::ui::checkbox("Locked", kivact::signal(false)).disabled(true),
                ))
                .style(style().padding(20).gap(16)),
            )
        });
    let check_box = screen.get_by_role(Role::CheckBox, "Notify");
    let switch_node = screen.get_by_role(Role::Switch, "Network");
    check_box.press_key(Key::Enter);
    switch_node.press_key(Key::Enter);
    assert!(!check.get_untracked() && !switch.get_untracked(), "Enter");
    for node in [&check_box, &switch_node] {
        node.press_key(Key::Space);
        assert_eq!(node.value(), None, "role and checked, no On/Off text");
    }
    assert!(check.get_untracked() && switch.get_untracked());
    check_box.a11y(A11yAction::Activate);
    switch_node.a11y(A11yAction::Toggle);
    assert!(!check.get_untracked() && !switch.get_untracked());
    let locked = screen.get_by_role(Role::CheckBox, "Locked");
    locked.force_click();
    locked.a11y(A11yAction::Toggle);
    assert_eq!(locked.is_checked(), Some(false));

    let settings = screen.get_by_role(Role::Button, "Settings");
    assert_eq!(settings.is_expanded(), Some(false));
    settings.press_key(Key::Enter);
    assert!(open.get_untracked());
    settings.press_key(Key::Space);
    assert!(!open.get_untracked());
    settings.a11y(A11yAction::Toggle);
    assert_eq!(settings.is_expanded(), Some(true));
    settings.a11y(A11yAction::Collapse);
    assert!(!open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_table_preserves_selection_and_updates_cells() {
    let (screen, (rows, selected, query)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let rows = signal(vec![("a", "Zulu".to_owned()), ("b", "Alpha".to_owned())]);
            let selected = signal(Vec::new());
            let query = signal(String::new());
            (
                (rows, selected, query),
                column((
                    kivact::ui::text_input("Filter", query),
                    kivact::ui::data_table(
                        rows,
                        |row: &(&'static str, String)| row.0,
                        vec![
                            kivact::ui::DataColumn::new("NAME", |row: &(&str, String)| {
                                row.1.clone()
                            })
                            .width(300.)
                            .compare(|a: &(&str, String), b: &(&str, String)| a.1.cmp(&b.1))
                            .filter(|row: &(&str, String), query: &str| row.1.contains(query)),
                        ],
                    )
                    .selection(selected)
                    .row_label(|row: &(&'static str, String)| row.0.to_owned())
                    .footer(|_| text("Footer renderer"))
                    .row_actions(100.0, |_| text("Row renderer"))
                    .toolbar(|_| text("Bulk renderer"))
                    .filter(query),
                ))
                .style(style().padding(20).gap(12)),
            )
        });
    screen.get_by_role(Role::CheckBox, "Select a").click();
    assert_eq!(selected.get_untracked(), vec!["a"]);
    screen.get_by_text("Footer renderer");
    screen.get_by_text("Bulk renderer");
    assert!(
        screen
            .get_by_role(Role::CheckBox, "Select all")
            .is_indeterminate()
    );
    screen
        .get_by_role(Role::ColumnHeader, "NAME")
        .a11y(A11yAction::Activate);
    query.set("Alpha".into());
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Row, "a").is_none());
    assert_eq!(selected.get_untracked(), vec!["a"]);
    screen.get_by_role(Role::CheckBox, "Select all").click();
    assert_eq!(selected.get_untracked().len(), 2);
    rows.update(|v| v[1].1 = "Changed".into());
    query.set(String::new());
    screen.run_until_idle();
    screen.get_by_text("Changed");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_modal_menu_command_search_and_focus_restore() {
    let (screen, (open, command_open, hits)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let open = signal(false);
            let command_open = signal(false);
            let hits = signal(0);
            (
                (open, command_open, hits),
                column((
                    kivact::ui::button("Create")
                        .variant(kivact_ui_game::Slant)
                        .on_press(move || open.set(true)),
                    kivact::ui::menu(
                        "Actions",
                        vec![MenuEntry::item("run", "Run")],
                        move |_: &'static str| hits.update(|v| *v += 1),
                    )
                    .offset(0.)
                    .attach(
                        kivact::ui::button("Actions")
                            .variant(kivact_ui_game::Slant)
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .into_node(),
                    ),
                    kivact::ui::button("Commands")
                        .variant(kivact_ui_game::Slant)
                        .on_press(move || command_open.set(true)),
                    kivact::ui::dialog("New operation", open)
                        .content(|| kivact::ui::text_input("Title", kivact::signal(String::new()))),
                    kivact::ui::command_palette(
                        command_open,
                        vec![
                            Command::new("deploy", "Deploy").hint("RUN"),
                            Command::new("help", "Help").hint("DOCS"),
                        ],
                        move |key: &'static str| {
                            if key == "deploy" {
                                hits.update(|v| *v += 1);
                            }
                        },
                    ),
                ))
                .style(style().padding(20).gap(20)),
            )
        });
    let create = screen.get_by_role(Role::Button, "Create");
    create.focus();
    create.click();
    assert!(open.get_untracked());
    screen.get_by_role(Role::Dialog, "New operation");
    screen.press_key(Key::Escape);
    assert!(!open.get_untracked());
    assert!(create.is_focused());
    screen.get_by_role(Role::Button, "Actions").click();
    screen
        .get_by_role(Role::MenuItem, "Run")
        .press_key(Key::Enter);
    assert_eq!(hits.get_untracked(), 1);
    screen.get_by_role(Role::Button, "Commands").click();
    let query = screen.get_by_role(Role::TextBox, "Search commands");
    query.type_text("Deploy");
    query.press_key(Key::Enter);
    assert_eq!(hits.get_untracked(), 2);
    assert!(!command_open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_filtered_command_identity_and_arrow_reveal_stay_consistent() {
    let (screen, (open, executed)) = kivact::test::render_with_handle(crate::options(true), || {
        let open = signal(false);
        let executed = signal(usize::MAX);
        let mut commands: Vec<_> = (0..14)
            .map(|index| Command::new(index, format!("Section {index}")).hint("SECTION"))
            .collect();
        commands.push(Command::new(14, "Theme").hint("THEME"));
        commands.push(Command::new(15, "Density").hint("DENSITY"));
        (
            (open, executed),
            (
                kivact::ui::button("Commands")
                    .variant(kivact_ui_game::Slant)
                    .on_press(move || open.set(true)),
                kivact::ui::command_palette(open, commands, move |key: usize| executed.set(key)),
            ),
        )
    });
    screen.get_by_role(Role::Button, "Commands").click();
    let query = screen.get_by_role(Role::TextBox, "Search commands");
    query.type_text(" Theme ");
    assert!(screen.get_by_text("Theme").exists());
    assert!(screen.query_by_role(Role::Option, "Section 0").is_none());
    let command = screen.get_by_role(Role::Option, "Theme");
    assert!(command.is_selected());
    command.click();
    assert_eq!(executed.get_untracked(), 14);
    assert!(!open.get_untracked());
    screen.get_by_role(Role::Button, "Commands").click();
    let query = screen.get_by_role(Role::TextBox, "Search commands");
    for _ in 0..15 {
        query.press_key(Key::ArrowDown);
    }
    let active = screen.get_by_role(Role::Option, "Density");
    assert!(active.is_selected());
    let viewport = screen
        .get_by_role(Role::ScrollView, "")
        .rect()
        .expect("palette scroll bounds");
    let rect = active.rect().expect("active option bounds");
    assert!(rect.y >= viewport.y);
    assert!(rect.y + rect.height <= viewport.y + viewport.height);
    query.press_key(Key::Enter);
    assert_eq!(executed.get_untracked(), 15);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_pager_focus_restore() {
    let (screen, page) = kivact::test::render_with_handle(crate::options(true), || {
        let page = signal(2_usize);
        (
            page,
            column((g::pagination(page, 12),)).style(style().padding(24).gap(28)),
        )
    });
    screen
        .get_by_role(Role::Button, "Next")
        .press_key(Key::Enter);
    assert_eq!(page.get_untracked(), 3);
    assert!(screen.get_by_role(Role::Button, "4").is_focused());
    screen.get_by_role(Role::Button, "12").click();
    assert_eq!(page.get_untracked(), 11);
    assert!(screen.get_by_role(Role::Button, "12").is_focused());
    assert!(screen.get_by_role(Role::Button, "Next").is_disabled());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_drawer_footer_stays_visible_below_long_scroll_body() {
    let (screen, (open, height)) = kivact::test::render_with_handle(crate::options(true), || {
        let open = signal(false);
        (
            (open, platform::window_size().height),
            (
                kivact::ui::button("Details")
                    .variant(kivact_ui_game::Slant)
                    .on_press(move || open.set(true)),
                kivact::ui::dialog("Record", open)
                    .kind(kivact::ui::DialogKind::Drawer(kivact::ui::Side::Right))
                    .content(|| {
                        node()
                            .style(style().height(2000))
                            .child(text("Long record"))
                    })
                    .footer(move || {
                        kivact::ui::button("Close record")
                            .variant(kivact_ui_game::Slant)
                            .on_press(move || open.set(false))
                    }),
            ),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Details");
    trigger.click();
    let footer = screen.get_by_role(Role::Button, "Close record");
    let rect = footer.rect().expect("fixed footer bounds");
    assert!(rect.y > height - 100.);
    assert!(rect.y + rect.height <= height);
    footer.click();
    assert!(!open.get_untracked());
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_toast_expiry_and_hover_pause() {
    let (screen, toaster) = kivact::test::render_with_handle(crate::options(true), || {
        let toaster = toaster().max_visible(4);
        (
            toaster,
            (
                column((
                    kivact::ui::button("Notify")
                        .variant(kivact_ui_game::Slant)
                        .on_press(move || {
                            toaster.push(
                                Toast::new("Saved")
                                    .body("Ready")
                                    .tone(Tone::Success)
                                    .duration(Some(Duration::from_millis(400))),
                            );
                        }),
                    g::banner("NOTICE", kivact::ui::Tone::Warning, ())
                        .content(move || {
                            rich_text(vec![
                                span("Maintenance:").weight(700).color(rgb(0xffc928)),
                                span(" read only"),
                            ])
                        })
                        .name("Service notice"),
                    g::activity(vec![g::ActivityEntry::new(
                        "09:40",
                        rich_text(vec![span("Review").weight(700), span(" passed")]),
                        kivact::ui::Tone::Success,
                    )]),
                )),
                kivact::ui::toast_region(toaster),
            ),
        )
    });
    assert!(screen.get_by_text("Maintenance: read only").exists());
    assert!(screen.get_by_text("Review passed").exists());
    screen.get_by_role(Role::Button, "Notify").click();
    assert!(screen.get_by_role(Role::Status, "Saved").exists());
    screen.get_by_text("Ready").hover();
    screen.advance(Duration::from_secs(1));
    assert_eq!(toaster.toasts().get_untracked().len(), 1);
    screen.mouse_leave();
    screen.advance(Duration::from_millis(399));
    assert_eq!(toaster.toasts().get_untracked().len(), 1);
    screen.advance(Duration::from_millis(1));
    assert!(toaster.toasts().get_untracked().is_empty());
    for _ in 0..5 {
        screen.get_by_role(Role::Button, "Notify").click();
    }
    assert_eq!(toaster.toasts().get_untracked().len(), 4, "four are shown");
    screen.advance(Duration::from_millis(400));
    assert_eq!(
        toaster.toasts().get_untracked().len(),
        1,
        "the fifth takes its place"
    );
    screen.advance(Duration::from_millis(400));
    assert!(toaster.toasts().get_untracked().is_empty());
    toaster.push(Toast::new("Failed").body("Rejected").tone(Tone::Danger));
    screen.run_until_idle();
    screen.get_by_role(Role::Alert, "Failed");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_theme_density_and_halo_pixels_follow_public_theme() {
    let (screen, (mode, density)) = kivact::test::render_with_handle(crate::options(true), || {
        let mode = signal(kivact::style::ThemeMode::Light);
        let density = signal(g::Density::Comfortable);
        (
            (mode, density),
            kivact::view::provide_theme(
                move || g::theme(mode.get()).density(density.get()),
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

#[test]
fn consumer_motion_intermediate_frames_and_reduced_motion_are_distinct() {
    let screen = render_with(options(false), || {
        column((
            kivact::ui::button("Processing")
                .variant(kivact_ui_game::Slant)
                .loading(true)
                .test_id("processing"),
            kivact::ui::progress(Some(0.68))
                .name("Synchronizing")
                .variant(kivact::ui::Tone::Info)
                .variant(g::Striped)
                .moving(true),
            kivact::ui::progress(None::<f64>)
                .name("Analyzing")
                .variant(kivact::ui::Tone::Info)
                .size(kivact::ui::ControlSize::Small),
            g::skeleton().rows(2),
            g::halo().style(style().size(64, 64)),
        ))
        .style(
            style()
                .padding(30)
                .gap(24)
                .width(420)
                .background(ColorToken::Surface),
        )
    });
    let initial = screen.screenshot();
    screen.advance(Duration::from_millis(270));
    let middle = screen.screenshot();
    assert_ne!(initial.hash(), middle.hash());
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/official-ui-validation/game-consumer");
    std::fs::create_dir_all(&directory).expect("create test artifact directory");
    initial
        .save_png(directory.join("motion-initial.png"))
        .expect("save initial frame");
    middle
        .save_png(directory.join("motion-270ms.png"))
        .expect("save intermediate frame");
    screen.set_reduced_motion(true);
    let reduced = screen.screenshot();
    // The busy button keeps turning (essential motion); the decorative
    // motion below it stands still.
    let button = screen
        .get_by_test_id("processing")
        .rect()
        .expect("laid out");
    let scale = screen.scale();
    let below = |shot: &kivact::test::Screenshot| {
        let top = ((button.y + button.height) * scale).ceil() as u32;
        shot.crop(0, top, shot.width, shot.height - top).hash()
    };
    let busy = |shot: &kivact::test::Screenshot| {
        shot.crop(
            (button.x * scale) as u32,
            (button.y * scale) as u32,
            (button.width * scale) as u32,
            (button.height * scale) as u32,
        )
        .hash()
    };
    screen.advance(Duration::from_millis(500));
    let later = screen.screenshot();
    assert_eq!(below(&reduced), below(&later));
    screen.advance(Duration::from_millis(175));
    assert_ne!(busy(&later), busy(&screen.screenshot()));
    reduced
        .save_png(directory.join("reduced-motion.png"))
        .expect("save reduced frame");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_progress_and_ring_start_on_first_visible_reveal() {
    let (screen, ring) = kivact::test::render_with_handle(options(false), move || {
        let ring = node_ref();
        (
            ring,
            scroll(
                column((
                    node().style(style().height(900)),
                    kivact::ui::progress(Some(0.75))
                        .name("Completed")
                        .variant(kivact::ui::Tone::Success),
                    g::ring("Storage", 80., kivact::ui::Tone::Info).node_ref(ring),
                ))
                .style(style().padding(24).gap(24)),
            )
            .style(style().size(pct(100), pct(100))),
        )
    });
    let before = screen.screenshot();
    screen.advance(Duration::from_secs(1));
    assert_eq!(before.hash(), screen.screenshot().hash());
    ring.scroll_into_view();
    screen.run_until_idle();
    let initial = screen.screenshot();
    screen.advance(Duration::from_millis(270));
    let middle = screen.screenshot();
    assert_ne!(initial.hash(), middle.hash());
    screen.advance(Duration::from_secs(1));
    let complete = screen.screenshot();
    assert_ne!(middle.hash(), complete.hash());
    screen.advance(Duration::from_secs(1));
    assert_eq!(complete.hash(), screen.screenshot().hash());
    assert_eq!(
        screen
            .get_by_role(Role::ProgressBar, "Completed")
            .value_range()
            .2,
        Some(0.75)
    );
    assert_eq!(
        screen.get_by_role(Role::Meter, "").value_range().2,
        Some(80.)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_tooltip_focus_delay_and_blur_use_setup_bound_observation() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(true), || {
        (
            (),
            row((
                kivact::ui::tooltip("Save with Ctrl+S")
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.)
                    .attach(kivact::ui::button("Save").variant(kivact_ui_game::Slant)),
                kivact::ui::button("Elsewhere").variant(kivact_ui_game::Slant),
            ))
            .style(style().padding(30).gap(24)),
        )
    });
    assert_eq!(
        screen.get_by_role(Role::Button, "Save").description(),
        "Save with Ctrl+S"
    );
    screen.get_by_role(Role::Button, "Save").focus();
    screen.advance(Duration::from_millis(499));
    assert!(
        screen
            .query_by_role(Role::Tooltip, "Save with Ctrl+S")
            .is_none()
    );
    screen.advance(Duration::from_millis(1));
    assert!(
        screen
            .query_by_role(Role::Tooltip, "Save with Ctrl+S")
            .is_some()
    );
    screen.get_by_role(Role::Button, "Elsewhere").focus();
    assert!(
        screen
            .query_by_role(Role::Tooltip, "Save with Ctrl+S")
            .is_none()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_grouped_menu_focus_and_disclosure_are_accessible() {
    let (screen, (checked, expanded)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let checked = signal(true);
            let expanded = signal(true);
            let user_open = signal(false);
            (
                (checked, expanded),
                column((
                    g::icon_menu("Actions", move || {
                        vec![
                            MenuEntry::item("unavailable", "Unavailable").disabled(true),
                            MenuEntry::separator(),
                            MenuEntry::item("columns", "Columns").checked(checked.get()),
                            MenuEntry::separator(),
                            MenuEntry::item("remove", "Remove"),
                        ]
                    })
                    .icon("dots")
                    .on_select(move |key: &'static str| {
                        if key == "columns" {
                            checked.update(|on| *on = !*on);
                        }
                    }),
                    kivact::ui::disclosure("Settings", expanded, || text("Inspector content")),
                    kivact::ui::dialog("User details", user_open)
                        .kind(kivact::ui::DialogKind::Popover)
                        .content(|| text("Online"))
                        .attach(g::tool_button("User").name("User details")),
                ))
                .style(style().padding(30).gap(24)),
            )
        });
    let trigger = screen.get_by_role(Role::Button, "Actions");
    trigger.click();
    assert_eq!(trigger.is_expanded(), Some(true));
    assert!(
        screen
            .get_by_role(Role::MenuItemCheckBox, "Columns")
            .is_focused()
    );
    screen
        .get_by_role(Role::MenuItemCheckBox, "Columns")
        .click();
    assert!(!checked.get_untracked());
    assert_eq!(
        trigger.is_expanded(),
        Some(false),
        "choosing a checked entry closes the menu"
    );
    trigger.click();
    assert_eq!(
        screen
            .get_by_role(Role::MenuItemCheckBox, "Columns")
            .is_checked(),
        Some(false)
    );
    screen.press_key(Key::Home);
    assert!(
        screen
            .get_by_role(Role::MenuItemCheckBox, "Columns")
            .is_focused()
    );
    screen.press_key(Key::Escape);
    assert_eq!(trigger.is_expanded(), Some(false));
    screen.get_by_role(Role::Button, "Settings").click();
    assert!(!expanded.get_untracked());
    assert!(screen.query_by_text("Inspector content").is_none());
    screen.get_by_role(Role::Button, "User details").click();
    assert!(screen.get_by_role(Role::Dialog, "User details").exists());
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Dialog, "User details").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_dialog_drawer_and_popover_are_named_dialogs_that_assistive_technology_can_dismiss() {
    let (screen, opens) = kivact::test::render_with_handle(crate::options(true), || {
        let (modal, sheet, popover) = (signal(false), signal(false), signal(false));
        (
            (modal, sheet, popover),
            column((
                kivact::ui::button("Open modal")
                    .variant(kivact_ui_game::Slant)
                    .on_press(move || modal.set(true)),
                kivact::ui::button("Open sheet")
                    .variant(kivact_ui_game::Slant)
                    .on_press(move || sheet.set(true)),
                kivact::ui::dialog("Rename", modal)
                    .description("The new name shows everywhere.")
                    .content(|| kivact::ui::text_input("Name", kivact::signal(String::new()))),
                kivact::ui::dialog("Details", sheet)
                    .kind(kivact::ui::DialogKind::Drawer(kivact::ui::Side::Right))
                    .content(|| text("Body")),
                kivact::ui::dialog("Help topic", popover)
                    .kind(kivact::ui::DialogKind::Popover)
                    .content(|| text("Help content"))
                    .attach(kivact::ui::button("Open help").variant(kivact_ui_game::Slant)),
            )),
        )
    });
    let (modal, sheet, popover) = opens;
    for (opener, title, open) in [
        ("Open modal", "Rename", modal),
        ("Open sheet", "Details", sheet),
        ("Open help", "Help topic", popover),
    ] {
        let opener = screen.get_by_role(Role::Button, opener);
        opener.focus();
        opener.click();
        let dialog = screen.get_by_role(Role::Dialog, title);
        screen.get_by_role(Role::Heading, title);
        dialog.a11y(A11yAction::Dismiss);
        assert!(!open.get_untracked(), "{title}: Dismiss closes it");
        assert!(opener.is_focused(), "{title}: focus returns to the opener");
    }
    screen.get_by_role(Role::Button, "Open modal").click();
    let described: Vec<String> = screen
        .get_by_role(Role::Dialog, "Rename")
        .described_by()
        .iter()
        .map(|e| e.text())
        .collect();
    assert_eq!(described, ["The new name shows everywhere."]);
    screen.get_by_role(Role::Button, "Close").click();
    assert!(!modal.get_untracked(), "the close button closes the dialog");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[path = "consumer/table_stage3.rs"]
mod table_stage3;
