//! Role: The Overlays page of the game catalog: Menus, dialogs, drawers, tooltips, popovers and the command palette.

use crate::support::common::{cards, line, list, specimen};
use crate::support::dialogs::overlay_layer;
use crate::support::state::{Demo, Overlay};
use catalog_shell::{Example, Page};
use kivact::platform;
use kivact::prelude::*;
use kivact::view::Node;
use kivact_ui_game as g;

/// The page: its one demo, with its explanation.
pub(crate) fn page() -> Page {
    Page::new("overlays", "Overlays", "Console", include_str!("overlays.rs"))
        .description("Menus, dialogs, drawers, tooltips, popovers and the command palette")
        .example(Example::new("overlays", "Overlays", "Dropdown and context menus, modal and drawer dialogs, tooltips, a popover and the command palette.", demo))
}

// example: overlays
fn demo() -> impl IntoView {
    let scene = Demo::new();
    column((overlays(scene), overlay_layer(scene)))
}
// end example

// xtask-allow(state): independent: pinned and archived are two dropdown checkbox entries flipped by their own keys; user_open is the user popover's dialog binding, opened by its trigger and closed by the dialog
pub(crate) fn overlays(scene: Demo) -> Node {
    let pinned = signal(true);
    let archived = signal(false);
    let user_open = signal(false);
    cards(
        (
            dropdown_menu(scene, pinned, archived),
            modal_dialog(scene),
            drawer_commands(scene),
            tooltip_anchors(),
            user_popover(scene, user_open),
        ),
        330.,
    )
}

fn dropdown_menu(
    scene: Demo,
    pinned: kivact::state::Signal<bool>,
    archived: kivact::state::Signal<bool>,
) -> impl kivact::view::Element {
    specimen(
        "OVL-01",
        "Dropdown menu",
        line((
            kivact::ui::menu("Actions ▾", move || {
                vec![
                    MenuEntry::item("rename", "Rename").with_shortcut("F2"),
                    MenuEntry::item("copy", "Duplicate").with_shortcut("Ctrl+D"),
                    MenuEntry::item("window", "Open in new window"),
                    MenuEntry::separator(),
                    MenuEntry::checkbox("pinned", "Show detail column", pinned.get()),
                    MenuEntry::checkbox("archived", "Include archived", archived.get()),
                    MenuEntry::separator(),
                    MenuEntry::item("delete", "Delete…").with_shortcut("Del"),
                ]
            })
            .on_activate(move |key: &'static str| match key {
                "rename" => scene.show(Overlay::Create),
                "copy" => {
                    spawn(platform::copy_text("OPS-2041"));
                }
                "window" => scene.show(Overlay::Drawer),
                "pinned" => pinned.update(|on| *on = !*on),
                "archived" => archived.update(|on| *on = !*on),
                "delete" => scene.show(Overlay::Delete),
                _ => {}
            })
            .offset(0.)
            .attach(
                kivact::ui::button("Actions ▾")
                    .variant(kivact::ui::Slant)
                    .variant(kivact::ui::ButtonVariant::Secondary),
            ),
            g::icon_menu(
                "Other",
                vec![
                    MenuEntry::item("favorite", "Favorites"),
                    MenuEntry::item("export", "Export"),
                ],
            )
            .icon("dots")
            .on_activate(move |key: &'static str| match key {
                "favorite" => scene.show(Overlay::Drawer),
                _ => scene.show(Overlay::Palette),
            }),
        )),
    )
}

fn modal_dialog(scene: Demo) -> impl kivact::view::Element {
    specimen(
        "OVL-02",
        "Modal dialog",
        line((
            kivact::ui::button("Form dialog")
                .variant(kivact::ui::Slant)
                .icon(kivact::view::icon("plus"))
                .test_id("game-create-open")
                .on_press(move || scene.show(Overlay::Create)),
            kivact::ui::button("Confirm dialog")
                .variant(kivact::ui::Slant)
                .icon(kivact::view::icon("alert-triangle"))
                .variant(kivact::ui::ButtonVariant::Danger)
                .on_press(move || scene.show(Overlay::Delete)),
        )),
    )
}

fn drawer_commands(scene: Demo) -> impl kivact::view::Element {
    specimen(
        "OVL-03",
        "Drawer / palette",
        line((
            kivact::ui::button("Details drawer")
                .variant(kivact::ui::Slant)
                .test_id("game-drawer-open")
                .variant(kivact::ui::ButtonVariant::Secondary)
                .on_press(move || scene.show(Overlay::Drawer)),
            kivact::ui::button("Command palette")
                .variant(kivact::ui::Slant)
                .icon(kivact::view::icon("command"))
                .variant(kivact::ui::ButtonVariant::Secondary)
                .on_press(move || scene.show(Overlay::Palette)),
        )),
    )
}

fn tooltip_anchors() -> impl kivact::view::Element {
    specimen(
        "OVL-04",
        "Tooltip",
        line((
            kivact::ui::tooltip(
                "Save  Ctrl+S",
                g::tool_button("Save").test_id("game-tooltip-save"),
            )
            .placement(kivact::view::Placement::Below)
            .align(kivact::view::OverlayAlign::Center)
            .offset(6.),
            kivact::ui::tooltip("Reload", g::tool_button("Reload"))
                .placement(kivact::view::Placement::Below)
                .align(kivact::view::OverlayAlign::Center)
                .offset(6.),
            kivact::ui::tooltip(
                "Permission: administrators only",
                kivact::ui::button("Locked")
                    .variant(kivact::ui::Slant)
                    .size(kivact::ui::ControlSize::Small)
                    .icon(kivact::view::icon("lock"))
                    .variant(kivact::ui::ButtonVariant::Secondary),
            )
            .placement(kivact::view::Placement::Below)
            .align(kivact::view::OverlayAlign::Center)
            .offset(6.),
        )),
    )
}

fn user_popover(scene: Demo, user_open: kivact::state::Signal<bool>) -> impl kivact::view::Element {
    specimen(
        "OVL-05",
        "User popover",
        (kivact::ui::dialog("User details", user_open)
            .mode(kivact::ui::DialogKind::Popover)
            .content(move || {
                list((
                    row((
                        kivact::ui::avatar("MS")
                            .fallback_content(|| text("MS"))
                            .variant(g::AvatarVariant {
                                side: 44.,
                                color: rgb(0x0a62c2),
                            })
                            .status(None),
                        column((
                            text("Sou Mashiba").style(style().font_weight(700)),
                            g::mono("ADMIN · Maintenance").style(style().color(color::TEXT_MUTED)),
                        )),
                    ))
                    .style(style().gap(12)),
                    row((
                        text("Status").style(style().color(color::TEXT_MUTED)),
                        spacer(),
                        kivact::ui::badge("Online").tone(kivact::interaction::Tone::Success),
                    )),
                    row((
                        text("In progress").style(style().color(color::TEXT_MUTED)),
                        spacer(),
                        g::mono("12 items"),
                    )),
                    kivact::ui::button("Send a message")
                        .variant(kivact::ui::Slant)
                        .size(kivact::ui::ControlSize::Small)
                        .on_press(move || scene.show(Overlay::Drawer)),
                ))
            })
            .attach(
                node()
                    .role(Role::Button)
                    .name("User details")
                    .focusable(true)
                    .child(
                        row((
                            kivact::ui::avatar("MS")
                                .fallback_content(|| text("MS"))
                                .variant(g::AvatarVariant {
                                    side: 32.,
                                    color: rgb(0x0a62c2),
                                })
                                .status(Some(kivact::interaction::Tone::Success)),
                            column((
                                text("Sou Mashiba").style(style().font_weight(700)),
                                text("Click for details")
                                    .style(style().font_size(11).color(color::TEXT_MUTED)),
                            )),
                        ))
                        .style(style().gap(10).align_items(Align::Center)),
                    ),
            ),),
    )
}
