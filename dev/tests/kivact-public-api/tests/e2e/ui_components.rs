//! Role: Standard compound components apply recipes to shared interaction parts.
mod keyed;

use crate::support::{mount, mount_with};
use kivact::draw::Rect;
use kivact::prelude::*;
use kivact::style::{Align, Style};
use kivact::test::RenderOptions;
use kivact::ui::Look;
use kivact::ui::{self, ListItem, MenuEntry, UiRecipes};
use kivact::view::{OverlayAlign, Placement};
use std::time::Duration;

#[test]
fn select_keeps_reactive_options_placeholder_and_keyboard_selection() {
    let (screen, (selected, label)) = mount(|| {
        let selected = signal(None::<u32>);
        let label = signal("Alpha".to_owned());
        let options = vec![ListItem::new(1, label), ListItem::new(2, "Beta")];
        (
            (selected, label),
            ui::select("Original", options, selected)
                .name("Choice")
                .placeholder("Pick one"),
        )
    });
    screen.get_by_text("Pick one");
    screen.get_by_role(Role::ComboBox, "Choice").click();
    screen.get_by_role(Role::ListBox, "Choice");
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    assert_eq!(selected.get_untracked(), Some(2));
    selected.set(Some(1));
    label.set("Renamed".to_owned());
    screen.run_until_idle();
    screen.get_by_text("Renamed");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn select_option_shows_the_check_only_while_selected_and_a_slot_replaces_it() {
    fn check(_: &Theme, state: &Look<kivact::interaction::ListItemState>) -> Style {
        let color = if state.selected { 0xff0000 } else { 0x00ff00 };
        style().size(12, 12).background(kivact::style::rgb(color))
    }
    let recipes = UiRecipes::default().select(|select| select.check(|_| check));
    let (screen, ()) = mount_with(
        RenderOptions {
            theme: Theme::light().with(recipes),
            ..Default::default()
        },
        || {
            let options = vec![ListItem::new(1, "Alpha"), ListItem::new(2, "Beta")];
            (
                (),
                column((
                    ui::select("Marked", options.clone(), signal(Some(1))),
                    ui::select("Slotted", options, signal(Some(1)))
                        .option(|item, _| text(move || item.get().label())),
                )),
            )
        },
    );
    screen.get_by_role(Role::ComboBox, "Marked").click();
    let last_box = |name: &str| {
        let option = screen.get_by_role(Role::Option, name);
        let children = option.children();
        let mark = children.last().expect("the check box");
        mark.rect().expect("check layout")
    };
    let (selected, other) = (last_box("Alpha"), last_box("Beta"));
    assert_eq!((selected.width, other.width), (12.0, 12.0));
    let shot = screen.screenshot();
    // A corner of the box, clear of the mark the icon draws.
    let center =
        |rect: Rect| crate::paint::pixel(&shot, (rect.x + 1.0) as u32, (rect.y + 1.0) as u32);
    assert_eq!(center(selected), 0xff0000, "the selected option's state");
    assert_eq!(center(other), 0x00ff00, "the other option's state");
    screen.press_key(Key::Escape);
    screen.get_by_role(Role::ComboBox, "Slotted").click();
    let slotted = screen.get_by_role(Role::Option, "Alpha");
    assert!(
        slotted
            .children()
            .iter()
            .all(|child| child.rect().is_none_or(|rect| rect.width != 12.0)),
        "a slot row has no check box"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn menu_renders_nested_levels_and_custom_trigger_keeps_its_style() {
    let (screen, chosen) = mount(|| {
        let chosen = signal(None);
        (
            chosen,
            ui::menu(
                "Actions",
                vec![
                    MenuEntry::submenu(1, "Export", vec![MenuEntry::item(2, "PNG")]),
                    MenuEntry::separator(),
                    MenuEntry::item(3, "Grid").checked(true),
                ],
                move |key| chosen.set(Some(key)),
            )
            .attach(
                ui::button("Custom")
                    .style(style().size(140, 30))
                    .test_id("trigger")
                    .into_node(),
            ),
        )
    });
    let trigger = screen.get_by_test_id("trigger");
    assert_eq!(trigger.rect().expect("layout").width, 140.0);
    trigger.click();
    screen.get_by_role(Role::Menu, "Custom");
    let parent = screen
        .get_by_role(Role::Menu, "Custom")
        .rect()
        .expect("root menu");
    let anchor = trigger.rect().expect("trigger");
    assert!((parent.y - anchor.y - anchor.height - 4.0).abs() < 0.5);
    let export = screen.get_by_role(Role::MenuItem, "Export");
    let item = export.rect().expect("submenu item");
    export.press_key(Key::ArrowRight);
    let submenu = screen
        .get_by_role(Role::Menu, "Export")
        .rect()
        .expect("submenu");
    assert!(
        (submenu.x - item.x - item.width).abs() < 0.5,
        "submenu has no added gap: {submenu:?} {item:?}"
    );
    screen
        .get_by_role(Role::MenuItem, "PNG")
        .press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), Some(2));
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn dialog_preserves_fresh_content_dismissal_and_focus_restoration() {
    let (screen, (open, mounts)) = mount(|| {
        let open = signal(false);
        let mounts = signal(0);
        (
            (open, mounts),
            column((
                ui::button("Open").on_press(move || open.set(true)),
                ui::dialog("Settings", open)
                    .description("Preferences")
                    .content(move || {
                        mounts.update(|n| *n += 1);
                        ui::text_input("Name", signal(String::new()))
                    }),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Open").click();
    screen.get_by_role(Role::Dialog, "Settings");
    screen.get_by_role(Role::Heading, "Settings");
    assert_eq!(mounts.get_untracked(), 1);
    screen.get_by_role(Role::Button, "Close").click();
    assert!(!open.get_untracked());
    assert!(screen.get_by_role(Role::Button, "Open").is_focused());
    screen.get_by_role(Role::Button, "Open").click();
    assert_eq!(mounts.get_untracked(), 2);
    screen.press_key(Key::Escape);
    assert!(!open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn tabs_render_shared_key_navigation_and_selected_panel_lifetime() {
    let (screen, selected) = mount(|| {
        let selected = signal(1);
        (
            selected,
            ui::tabs(
                "Tabs",
                vec![ListItem::new(1, "One"), ListItem::new(2, "Two")],
                selected,
            )
            .content(|key| text(format!("Panel {key}"))),
        )
    });
    screen.get_by_role(Role::Tab, "One").focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(selected.get_untracked(), 2);
    screen.get_by_text("Panel 2");
    assert!(screen.query_by_text("Panel 1").is_none());
    screen.get_by_role(Role::TabPanel, "Two");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn tooltip_uses_shared_delay_and_native_text_area_remains_controlled() {
    let (screen, value) = mount(|| {
        let value = signal("Initial".to_owned());
        (
            value,
            column((
                ui::tooltip("A hint")
                    .content(|| text("Rich detail").style(style().weight(700)))
                    .placement(Placement::Below)
                    .attach(ui::button("Help")),
                ui::text_area("Notes", value),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Help").focus();
    screen.advance(Duration::from_millis(500));
    screen.get_by_role(Role::Tooltip, "A hint");
    screen.get_by_text("Rich detail");
    let field = screen.get_by_role(Role::TextBox, "Notes");
    field.press_key(Key::End);
    field.type_text(" more");
    assert_eq!(value.get_untracked(), "Initial more");
    value.set("Changed".to_owned());
    screen.run_until_idle();
    assert_eq!(field.value().as_deref(), Some("Changed"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn compound_recipes_are_replaceable_and_button_loading_blocks_activation() {
    fn trigger(_: &Theme, _: &Look<ui::SelectTriggerState>) -> Style {
        style().size(173, 37)
    }
    let recipes = UiRecipes::default().select(|select| select.trigger(|_| trigger));
    let (screen, (busy, hits)) = mount_with(
        RenderOptions {
            theme: Theme::light().with(recipes),
            reduced_motion: true,
            ..Default::default()
        },
        || {
            let busy = signal(true);
            let hits = signal(0);
            (
                (busy, hits),
                column((
                    ui::select("Pick", vec![ListItem::new(1, "First")], signal(None))
                        .test_id("pick"),
                    ui::button("Run")
                        .loading(busy)
                        .on_press(move || hits.update(|n| *n += 1)),
                )),
            )
        },
    );
    assert_eq!(
        screen.get_by_test_id("pick").rect().expect("layout").width,
        173.0
    );
    assert!(screen.get_by_role(Role::Button, "Run").is_disabled());
    busy.set(false);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Run").click();
    assert_eq!(hits.get_untracked(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn context_menu_reuses_renderer_for_keyboard_and_pointer_origins() {
    use kivact::view::MouseButton;
    let (screen, chosen) = mount(|| {
        let chosen = signal(0);
        (
            chosen,
            column((
                ui::button("Outside"),
                ui::menu("Context", vec![MenuEntry::item(1, "Inspect")], move |key| {
                    chosen.set(key)
                })
                .attach_context(
                    node()
                        .name("Target")
                        .role(Role::Button)
                        .focusable(true)
                        .style(style().size(180, 70))
                        .child(text("Target")),
                ),
            )),
        )
    });
    let target = screen.get_by_role(Role::Button, "Target");
    target.focus();
    screen.press_key(Key::ContextMenu);
    screen.get_by_role(Role::Menu, "Target");
    screen
        .get_by_role(Role::MenuItem, "Inspect")
        .press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), 1);
    assert!(target.is_focused());
    let outside = screen.get_by_role(Role::Button, "Outside");
    outside.focus();
    target.click_with(MouseButton::Secondary);
    screen.get_by_role(Role::MenuItem, "Inspect");
    screen.press_key(Key::Escape);
    assert!(
        target.is_focused(),
        "the native context press focused its target before opening"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn header_only_tabs_keep_modifiers_and_add_no_empty_panel_tab_stop() {
    let (screen, selected) = mount(|| {
        let selected = signal(1);
        (
            selected,
            column((
                ui::tabs(
                    "Sections",
                    vec![ListItem::new(1, "One"), ListItem::new(2, "Two")],
                    selected,
                )
                .test_id("tabs")
                .style(style().height(47)),
                ui::button("After"),
            )),
        )
    });
    assert_eq!(
        screen.get_by_test_id("tabs").rect().expect("layout").height,
        47.0
    );
    screen.get_by_role(Role::TabList, "Sections");
    screen.get_by_role(Role::Tab, "One").focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(selected.get_untracked(), 2);
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    assert!(screen.all_by_role(Role::TabPanel, "").is_empty());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn localized_switch_labels_are_setup_handles_read_reactively_by_paint() {
    use kivact::i18n::Messages;
    use kivact::view::provide_messages;
    fn localized(_: &Theme, state: &Look<ui::CheckState>) -> Style {
        let text = state.text.expect("a switch has a state text");
        style().size(80, 30).paint(move |draw, size| {
            draw.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                if text.get() == "NO" {
                    kivact::style::rgb(0x00ff00)
                } else {
                    kivact::style::rgb(0xff0000)
                },
            )
        })
    }
    let recipes = UiRecipes::default().switch(|_| localized);
    let (screen, messages) = mount_with(
        RenderOptions {
            theme: Theme::light().with(recipes),
            ..Default::default()
        },
        || {
            let messages = signal(Messages::new());
            (
                messages,
                provide_messages(messages, || {
                    ui::switch("Power", signal(false))
                        .hide_label(true)
                        .test_id("switch")
                }),
            )
        },
    );
    let sample = || {
        let rect = screen.get_by_test_id("switch").rect().expect("layout");
        crate::paint::pixel(
            &screen.screenshot(),
            (rect.x + 5.0) as u32,
            (rect.y + 15.0) as u32,
        )
    };
    assert_eq!(sample(), 0xff0000);
    messages.set(
        Messages::new()
            .with("switch.on", "YES")
            .with("switch.off", "NO"),
    );
    screen.run_until_idle();
    assert_eq!(sample(), 0x00ff00);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_dialog_can_keep_its_only_close_action_in_the_footer() {
    let (screen, open) = mount(|| {
        let open = signal(true);
        (
            open,
            ui::dialog("Footer dialog", open)
                .closable(false)
                .content(|| ui::button("First"))
                .footer(move || ui::button("Close").on_press(move || open.set(false))),
        )
    });
    assert_eq!(screen.all_by_role(Role::Button, "Close").len(), 1);
    screen.get_by_role(Role::Button, "First").focus();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "Close").is_focused());
    screen.get_by_role(Role::Button, "Close").click();
    assert!(!open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn missing_tabs_do_not_build_panels_and_readding_the_key_remounts_content() {
    let (screen, (options, selected, built, cleaned)) = mount(|| {
        let options = signal(vec![ListItem::new(1, "One"), ListItem::new(2, "Two")]);
        let selected = signal(1);
        let built = signal(Vec::new());
        let cleaned = signal(Vec::new());
        (
            (options, selected, built, cleaned),
            ui::tabs("Tabs", options, selected).content(move |key| {
                assert!(
                    options
                        .get_untracked()
                        .iter()
                        .any(|item| *item.key() == key)
                );
                built.update(|keys| keys.push(key));
                on_cleanup(move || cleaned.update(|keys| keys.push(key)));
                text(format!("Panel {key}"))
            }),
        )
    });
    assert_eq!(built.get_untracked(), vec![1]);
    options.set(vec![ListItem::new(2, "Two")]);
    screen.run_until_idle();
    assert_eq!(selected.get_untracked(), 1);
    assert!(screen.query_by_text("Panel 1").is_none());
    assert_eq!(cleaned.get_untracked(), vec![1]);
    selected.set(3);
    screen.run_until_idle();
    assert_eq!(built.get_untracked(), vec![1]);
    options.set(vec![ListItem::new(2, "Two"), ListItem::new(3, "Three")]);
    screen.run_until_idle();
    screen.get_by_text("Panel 3");
    assert_eq!(built.get_untracked(), vec![1, 3]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn kept_tab_panels_are_built_once_and_keep_their_state_while_hidden() {
    let (screen, (selected, built)) = mount(|| {
        let selected = signal(1);
        let built = signal(Vec::new());
        (
            (selected, built),
            ui::tabs(
                "Tabs",
                vec![ListItem::new(1, "One"), ListItem::new(2, "Two")],
                selected,
            )
            .keep_mounted(true)
            .content(move |key| {
                built.update(|keys| keys.push(key));
                ui::text_input(format!("Draft {key}"), signal(String::new()))
            }),
        )
    });
    assert_eq!(
        built.get_untracked(),
        vec![1, 2],
        "every panel is built at once"
    );
    let draft = screen.get_by_role(Role::TextBox, "Draft 1");
    draft.type_text("kept");
    assert!(
        screen.query_by_role(Role::TabPanel, "Two").is_none(),
        "hidden"
    );
    screen.get_by_role(Role::Tab, "Two").click();
    assert_eq!(selected.get_untracked(), 2);
    assert!(
        screen.query_by_role(Role::TabPanel, "One").is_none(),
        "hidden"
    );
    assert!(screen.get_by_role(Role::TabPanel, "Two").exists());
    screen.get_by_role(Role::Tab, "One").click();
    assert_eq!(
        draft.value().as_deref(),
        Some("kept"),
        "the hidden panel kept its draft"
    );
    assert_eq!(
        built.get_untracked(),
        vec![1, 2],
        "switching builds nothing"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_popover_has_no_header_close_target() {
    use kivact::ui::DialogKind;
    let (screen, ()) = mount(|| {
        (
            (),
            column((ui::dialog("Info", signal(true))
                .kind(DialogKind::Popover)
                .content(|| text("Popover body"))
                .attach(ui::button("Anchor")),)),
        )
    });
    screen.get_by_role(Role::Dialog, "Info");
    assert!(screen.query_by_role(Role::Button, "Close").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn tooltip_alignment_and_gap_are_layout_options_with_unchanged_defaults() {
    fn surface(_: &Theme, _: &Look<kivact::interaction::PopupState>) -> Style {
        style().size(80, 20)
    }
    for custom in [false, true] {
        let recipes = UiRecipes::default().tooltip(|tooltip| tooltip.surface(|_| surface));
        let (screen, ()) = mount_with(
            RenderOptions {
                theme: Theme::light().with(recipes),
                ..Default::default()
            },
            move || {
                let tooltip = ui::tooltip("Geometry").placement(Placement::Below);
                let tooltip = if custom {
                    tooltip.align(OverlayAlign::End).offset(11.0)
                } else {
                    tooltip
                };
                (
                    (),
                    column(tooltip.attach(ui::button("Anchor").style(style().size(160, 30))))
                        .style(style().padding(40).align_items(Align::Start)),
                )
            },
        );
        let anchor = screen.get_by_role(Role::Button, "Anchor");
        anchor.focus();
        screen.advance(Duration::from_millis(500));
        let anchor = anchor.rect().expect("anchor");
        let surface = screen
            .get_by_role(Role::Tooltip, "Geometry")
            .rect()
            .expect("tooltip");
        let expected_x = if custom {
            anchor.x + anchor.width - surface.width
        } else {
            anchor.x
        };
        let gap = if custom { 11.0 } else { 4.0 };
        assert!(
            (surface.x - expected_x).abs() < 0.5,
            "{surface:?} {anchor:?}"
        );
        assert!(
            (surface.y - anchor.y - anchor.height - gap).abs() < 0.5,
            "{surface:?} {anchor:?}"
        );
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn a_menu_can_keep_a_theme_specific_root_gap() {
    let (screen, ()) = mount(|| {
        (
            (),
            column(ui::menu("Actions", vec![MenuEntry::item(1, "Run")], |_| {}).offset(0.0))
                .style(style().padding(40).align_items(Align::Start)),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Actions");
    trigger.click();
    let anchor = trigger.rect().expect("trigger");
    let surface = screen
        .get_by_role(Role::Menu, "Actions")
        .rect()
        .expect("menu");
    assert!((surface.y - anchor.y - anchor.height).abs() < 0.5);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn checked_submenu_default_paints_a_leading_check_and_trailing_chevron() {
    let (screen, ()) = mount(|| {
        (
            (),
            ui::menu(
                "Marks",
                vec![
                    MenuEntry::submenu(1, "Checked submenu", vec![MenuEntry::item(2, "Child")])
                        .checked(true),
                ],
                |_| (),
            ),
        )
    });
    screen.get_by_role(Role::Button, "Marks").click();
    let item = screen.get_by_role(Role::MenuItemCheckBox, "Checked submenu");
    assert_eq!(item.is_checked(), Some(true));
    let children = item.children();
    let leading = children
        .first()
        .expect("leading check")
        .rect()
        .expect("check layout");
    let trailing = children
        .last()
        .expect("trailing chevron")
        .rect()
        .expect("chevron layout");
    assert!(
        leading.x + leading.width < trailing.x,
        "the marks occupy opposite ends"
    );
    let shot = screen.screenshot();
    let dark_pixels = |rect: Rect, xs: std::ops::Range<u32>, ys: std::ops::Range<u32>| {
        xs.flat_map(|x| ys.clone().map(move |y| (x, y)))
            .filter(|(x, y)| {
                let color = crate::paint::pixel(
                    &shot,
                    (rect.x + *x as f32) as u32,
                    (rect.y + *y as f32) as u32,
                );
                ((color >> 16) & 255) < 100 && ((color >> 8) & 255) < 100 && (color & 255) < 100
            })
            .count()
    };
    // The theme's 16 px `check` and `chevron-right` icons.
    assert!(
        dark_pixels(leading, 11..14, 3..7) > 0,
        "check has its upper right tip"
    );
    assert!(
        dark_pixels(trailing, 5..8, 3..6) > 0,
        "chevron has its upper left stroke; a second check does not"
    );
    assert_eq!(
        dark_pixels(trailing, 11..14, 3..6),
        0,
        "chevron must not be another check"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn checked_submenu_recipe_slots_both_receive_truthful_state() {
    use kivact::interaction::MenuItemState;
    fn check(_: &Theme, state: &Look<MenuItemState>) -> Style {
        assert_eq!(
            state.checked,
            Some(true),
            "check receives the actual checked item"
        );
        style()
            .size(12, 12)
            .background(kivact::style::rgb(0xff0000))
    }
    fn submenu(_: &Theme, state: &Look<MenuItemState>) -> Style {
        assert_eq!(
            state.checked,
            Some(true),
            "submenu receives the same actual checked item"
        );
        style()
            .size(12, 12)
            .background(kivact::style::rgb(0x0000ff))
    }
    let recipes = UiRecipes::default().menu(|menu| menu.check(|_| check).submenu(|_| submenu));
    let (screen, ()) = mount_with(
        RenderOptions {
            theme: Theme::light().with(recipes),
            ..Default::default()
        },
        || {
            (
                (),
                ui::menu(
                    "Custom marks",
                    vec![
                        MenuEntry::submenu(1, "Checked submenu", vec![MenuEntry::item(2, "Child")])
                            .checked(true),
                    ],
                    |_| (),
                ),
            )
        },
    );
    screen.get_by_role(Role::Button, "Custom marks").click();
    let item = screen.get_by_role(Role::MenuItemCheckBox, "Checked submenu");
    assert_eq!(item.is_checked(), Some(true));
    let children = item.children();
    let leading = children
        .first()
        .expect("check")
        .rect()
        .expect("check layout");
    let trailing = children
        .last()
        .expect("submenu")
        .rect()
        .expect("submenu layout");
    let label = screen
        .get_by_text("Checked submenu")
        .rect()
        .expect("label layout");
    assert!(leading.x + leading.width <= label.x && label.x + label.width <= trailing.x);
    let shot = screen.screenshot();
    // A corner of the box, clear of the mark the icon draws.
    let center =
        |rect: Rect| crate::paint::pixel(&shot, (rect.x + 1.0) as u32, (rect.y + 1.0) as u32);
    assert_eq!(
        center(leading),
        0xff0000,
        "check recipe paints the leading slot"
    );
    assert_eq!(
        center(trailing),
        0x0000ff,
        "submenu recipe paints the trailing slot"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn dialog_width_overrides_recipe_and_updates_reactively() {
    let (screen, ()) = mount(|| ((), ui::dialog("Default width", signal(true)).content(|| ())));
    let rect = screen
        .get_by_role(Role::Dialog, "Default width")
        .rect()
        .expect("dialog layout");
    assert!((rect.width - 480.0).abs() < 0.5, "{rect:?}");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    drop(screen);

    let (screen, width) = mount(|| {
        let width = signal(kivact::style::Length::from(640));
        (
            width,
            ui::dialog("Custom width", signal(true))
                .width(width)
                .content(|| ()),
        )
    });
    let panel_width = || {
        screen
            .get_by_role(Role::Dialog, "Custom width")
            .rect()
            .expect("dialog layout")
            .width
    };
    assert!((panel_width() - 640.0).abs() < 0.5);
    width.set(kivact::style::Length::from(560));
    screen.run_until_idle();
    assert!((panel_width() - 560.0).abs() < 0.5);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn canvas_regions_delegate_only_unhandled_context_requests_to_the_standard_menu() {
    use kivact::draw::{Rect, RegionId};
    use kivact::primitive::part;
    use kivact::view::{Handled, Modifiers, MouseButton, PointerPhase};
    let (screen, (consume, disabled, hidden, present, root_keys, root_presses)) = mount(|| {
        let consume = signal(false);
        let disabled = signal(false);
        let hidden = signal(false);
        let present = signal(true);
        let root_keys = signal(Vec::new());
        let root_presses = signal(Vec::new());
        let area = kivact::control::canvas(move |draw, _| {
            if present.get() {
                draw.region(RegionId(1), Rect::new(0.0, 0.0, 120.0, 60.0));
            }
        })
        .role(Role::Canvas)
        .name("Canvas menu")
        .focusable(true)
        .disabled(disabled)
        .hidden(hidden)
        .style(style().size(200, 100))
        .regions(
            move || {
                if present.get() {
                    vec![1u64]
                } else {
                    Vec::new()
                }
            },
            |id| RegionId(*id),
            move |_, _| {
                part(|state| state)
                    .role(Role::Slider)
                    .name("Canvas region")
                    .focusable(true)
                    .on_key(move |_| {
                        if consume.get_untracked() {
                            Handled::Yes
                        } else {
                            Handled::No
                        }
                    })
                    .on_pointer(move |event| {
                        if consume.get_untracked() && event.button == Some(MouseButton::Secondary) {
                            Handled::Yes
                        } else {
                            Handled::No
                        }
                    })
            },
        )
        .part(
            part(|state| state)
                .on_key(move |event| {
                    root_keys.update(|keys| keys.push(event.key));
                    Handled::No
                })
                .on_pointer(move |event| {
                    if matches!(event.phase, PointerPhase::Down | PointerPhase::Up) {
                        root_presses.update(|buttons| buttons.push(event.button));
                    }
                    Handled::No
                }),
        );
        (
            (consume, disabled, hidden, present, root_keys, root_presses),
            ui::menu("", vec![MenuEntry::item(1, "Inspect canvas")], |_| {}).attach_context(area),
        )
    });
    let region = screen.get_by_role(Role::Slider, "Canvas region");
    region.focus();
    region.press_key(Key::ArrowRight);
    region.click();
    assert!(root_keys.get_untracked().is_empty());
    assert!(root_presses.get_untracked().is_empty());
    for (key, modifiers) in [
        (Key::ContextMenu, Modifiers::default()),
        (
            Key::F(10),
            Modifiers {
                shift: true,
                ..Default::default()
            },
        ),
    ] {
        region.press_key_with(key, modifiers);
        screen.get_by_role(Role::MenuItem, "Inspect canvas");
        screen.press_key(Key::Escape);
        assert!(region.is_focused());
    }
    region.click_with(MouseButton::Secondary);
    screen.get_by_role(Role::MenuItem, "Inspect canvas");
    screen.press_key(Key::Escape);
    assert!(screen.get_by_role(Role::Canvas, "Canvas menu").is_focused());
    assert!(
        root_keys.get_untracked().is_empty(),
        "unrelated root parts stay isolated"
    );
    assert!(root_presses.get_untracked().is_empty());
    consume.set(true);
    region.press_key(Key::ContextMenu);
    region.click_with(MouseButton::Secondary);
    assert!(screen.query_by_role(Role::Menu, "Canvas menu").is_none());
    consume.set(false);
    disabled.set(true);
    screen.run_until_idle();
    region.force_click();
    assert!(screen.query_by_role(Role::Menu, "Canvas menu").is_none());
    disabled.set(false);
    screen.run_until_idle();
    let p = region.rect().unwrap().center();
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Secondary);
    present.set(false);
    screen.run_until_idle();
    screen.mouse_up(MouseButton::Secondary);
    assert!(screen.query_by_role(Role::Menu, "Canvas menu").is_none());
    present.set(true);
    hidden.set(true);
    screen.run_until_idle();
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Secondary);
    screen.mouse_up(MouseButton::Secondary);
    assert!(screen.query_by_role(Role::Menu, "Canvas menu").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn regions_without_opted_in_parts_do_not_fall_back_to_area_defaults() {
    use kivact::draw::{Rect, RegionId};
    use kivact::primitive::part;
    use kivact::view::{Handled, MouseButton, PointerPhase};
    let (screen, (keys, presses)) = mount(|| {
        let keys = signal(Vec::new());
        let presses = signal(0);
        (
            (keys, presses),
            kivact::control::canvas(|draw, _| {
                draw.region(RegionId(1), Rect::new(0.0, 0.0, 100.0, 60.0))
            })
            .name("Plain canvas")
            .style(style().size(180, 100))
            .region(
                RegionId(1),
                part(|state| state)
                    .role(Role::Slider)
                    .name("Plain region")
                    .focusable(true),
            )
            .part(
                part(|state| state)
                    .on_key(move |event| {
                        keys.update(|keys| keys.push(event.key));
                        Handled::No
                    })
                    .on_pointer(move |event| {
                        if matches!(event.phase, PointerPhase::Down | PointerPhase::Up) {
                            presses.update(|n| *n += 1);
                        }
                        Handled::No
                    }),
            ),
        )
    });
    let region = screen.get_by_role(Role::Slider, "Plain region");
    region.press_key(Key::ContextMenu);
    region.press_key(Key::ArrowRight);
    region.click_with(MouseButton::Secondary);
    region.click();
    assert!(keys.get_untracked().is_empty());
    assert_eq!(presses.get_untracked(), 0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
