//! Role: Declared pointer focus destinations preserve exact input identity.

use crate::support::mount_with;
use kivact::control::canvas;
use kivact::draw::{Point, Rect, RegionId};
use kivact::prelude::*;
use kivact::primitive::part;
use kivact::test::{RenderOptions, WindowMode};
use kivact::view::{A11yAction, Placement};
use kivact::view::{Modifiers, MouseButton, overlay_at};

fn real_window() -> RenderOptions {
    RenderOptions {
        window: WindowMode::Real,
        ..RenderOptions::from_env()
    }
}

#[test]
fn disabled_node_and_region_keep_field_focus_without_enabling_actions_or_moving_caret() {
    let (screen, (calls, origins)) = mount_with(real_window(), || {
        let field = node_ref();
        let value = signal("search query".to_owned());
        let calls = signal(0);
        let origins = signal(Vec::new());
        let disabled = node()
            .part(part(|s| s).focus_on_press(field))
            .role(Role::Option)
            .name("Disabled node")
            .disabled(true)
            .on_press(move || calls.update(|n| *n += 1))
            .style(style().size(180, 35));
        let enabled = node()
            .part(part(|s| s).focus_on_press(field))
            .role(Role::Option)
            .name("Enabled node")
            .on_press(move || calls.update(|n| *n += 1))
            .style(style().size(180, 35));
        let regions = canvas(|d, _| {
            d.region(RegionId(1), Rect::new(0.0, 0.0, 85.0, 35.0));
            d.region(RegionId(2), Rect::new(95.0, 0.0, 85.0, 35.0));
        })
        .region(
            RegionId(1),
            part(|s| s)
                .role(Role::Option)
                .name("Disabled region")
                .disabled(true)
                .focus_on_press(field)
                .on_press(move || calls.update(|n| *n += 1)),
        )
        .region(
            RegionId(2),
            part(|s| s)
                .role(Role::Option)
                .name("Enabled region")
                .focus_on_press(field)
                .on_a11y_event(A11yAction::Activate, move |e| {
                    origins.update(|v| v.push(e.region));
                    calls.update(|n| *n += 1);
                }),
        )
        .style(style().size(180, 35));
        (
            (calls, origins),
            column((
                kivact::control::node()
                    .name("Search")
                    .part(kivact::interaction::use_text_edit(value).control())
                    .node_ref(field)
                    .style(style().size(180, 35)),
                disabled,
                enabled,
                regions,
            )),
        )
    });
    let field = screen.get_by_role(Role::TextBox, "Search");
    field.focus();
    field.press_key(Key::End);
    field.press_key_with(
        Key::ArrowLeft,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );
    let selected = screen.ime_text().unwrap().selection;
    for name in ["Disabled node", "Disabled region"] {
        let row = screen.get_by_role(Role::Option, name);
        let center = row.rect().unwrap().center();
        screen.mouse_move(center.x, center.y);
        screen.mouse_down(MouseButton::Primary);
        assert!(field.is_focused(), "{name} restores on Down");
        assert_eq!(screen.ime_text().unwrap().selection, selected);
        screen.mouse_up(MouseButton::Primary);
        row.a11y(A11yAction::Activate);
        assert_eq!(calls.get(), 0);
    }
    screen.get_by_role(Role::Option, "Enabled node").click();
    assert!(field.is_focused());
    assert_eq!(calls.get(), 1);
    screen.get_by_role(Role::Option, "Enabled region").click();
    assert!(field.is_focused());
    assert_eq!(screen.ime_text().unwrap().selection, selected);
    assert_eq!(calls.get(), 2);
    assert_eq!(origins.get(), vec![Some(RegionId(2))]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn late_binding_rebinding_and_invalid_destinations_obey_the_nearest_declaration() {
    let (screen, (shown, disabled, hidden, focusable)) = mount_with(real_window(), || {
        let outer = node_ref();
        let destination = node_ref();
        let shown = signal(false);
        let disabled = signal(false);
        let hidden = signal(false);
        let focusable = signal(true);
        let source = node().part(part(|s| s).focus_on_press(outer)).child(
            node()
                .part(part(|s| s).focus_on_press(destination))
                .role(Role::Button)
                .name("Source")
                .focusable(true)
                .style(style().size(140, 35)),
        );
        (
            (shown, disabled, hidden, focusable),
            column((
                kivact::control::node()
                    .name("Outer")
                    .part(
                        kivact::interaction::use_text_edit(kivact::signal(String::new())).control(),
                    )
                    .node_ref(outer)
                    .style(style().size(140, 35)),
                show(shown, move || {
                    kivact::control::node()
                        .name("Destination")
                        .part(
                            kivact::interaction::use_text_edit(kivact::signal(String::new()))
                                .control(),
                        )
                        .node_ref(destination)
                        .disabled(disabled)
                        .hidden(hidden)
                        .focusable(focusable)
                        .style(style().size(140, 35))
                }),
                source,
            )),
        )
    });
    let source = screen.get_by_role(Role::Button, "Source");
    source.click();
    assert!(
        screen.get_by_role(Role::TextBox, "Outer").is_focused(),
        "unbound declaration is absent"
    );
    shown.set(true);
    screen.run_until_idle();
    source.click();
    assert!(
        screen
            .get_by_role(Role::TextBox, "Destination")
            .is_focused()
    );
    disabled.set(true);
    screen.run_until_idle();
    source.click();
    assert!(
        source.is_focused(),
        "invalid bound declaration stops outer lookup"
    );
    disabled.set(false);
    hidden.set(true);
    screen.run_until_idle();
    source.click();
    assert!(source.is_focused());
    hidden.set(false);
    screen.run_until_idle();
    source.click();
    assert!(
        screen
            .get_by_role(Role::TextBox, "Destination")
            .is_focused()
    );
    focusable.set(false);
    screen.run_until_idle();
    source.click();
    assert!(source.is_focused(), "nonfocusable destination is invalid");
    focusable.set(true);
    screen.run_until_idle();
    shown.set(false);
    screen.run_until_idle();
    source.click();
    assert!(screen.get_by_role(Role::TextBox, "Outer").is_focused());
    shown.set(true);
    screen.run_until_idle();
    source.click();
    assert!(
        screen
            .get_by_role(Role::TextBox, "Destination")
            .is_focused(),
        "rebound ref follows new node"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn exact_region_destinations_and_region_declarations_override_the_canvas_owner() {
    let (screen, (present, disabled, hidden, visible)) = mount_with(real_window(), || {
        let field = node_ref();
        let exact = node_ref();
        let visible = exact.focus_visible();
        let present = signal(true);
        let disabled = signal(false);
        let hidden = signal(false);
        let destination = canvas(move |d, _| {
            if present.get() {
                d.region(RegionId(10), Rect::new(0.0, 0.0, 90.0, 35.0));
            }
        })
        .region(
            RegionId(10),
            part(|s| s)
                .reference(exact)
                .role(Role::Button)
                .name("Exact destination")
                .focusable(true)
                .disabled(disabled)
                .hidden(hidden),
        )
        .style(style().size(180, 35));
        let source = canvas(|d, _| {
            d.region(RegionId(1), Rect::new(0.0, 0.0, 90.0, 35.0));
            d.region(RegionId(2), Rect::new(95.0, 0.0, 85.0, 35.0));
        })
        .part(part(|s| s).focus_on_press(field))
        .region(
            RegionId(1),
            part(|s| s)
                .focus_on_press(exact)
                .role(Role::Button)
                .name("Region source")
                .focusable(true),
        )
        .region(
            RegionId(2),
            part(|s| s)
                .role(Role::Button)
                .name("Inherited source")
                .focusable(true),
        )
        .style(style().size(180, 35));
        (
            (present, disabled, hidden, visible),
            column((
                kivact::control::node()
                    .name("Owner destination")
                    .part(
                        kivact::interaction::use_text_edit(kivact::signal(String::new())).control(),
                    )
                    .node_ref(field)
                    .style(style().size(180, 35)),
                destination,
                source,
                node()
                    .part(part(|s| s).focus_on_press(exact))
                    .role(Role::Button)
                    .name("Node to region")
                    .focusable(true)
                    .style(style().size(180, 35)),
            )),
        )
    });
    let source = screen.get_by_role(Role::Button, "Region source");
    source.click();
    assert!(
        screen
            .get_by_role(Role::Button, "Exact destination")
            .is_focused()
    );
    assert!(!visible.get());
    screen.get_by_role(Role::Button, "Node to region").click();
    assert!(
        screen
            .get_by_role(Role::Button, "Exact destination")
            .is_focused()
    );
    screen.get_by_role(Role::Button, "Inherited source").click();
    assert!(
        screen
            .get_by_role(Role::TextBox, "Owner destination")
            .is_focused()
    );
    disabled.set(true);
    screen.run_until_idle();
    source.click();
    assert!(
        source.is_focused(),
        "invalid explicit region destination blocks owner declaration"
    );
    disabled.set(false);
    hidden.set(true);
    screen.run_until_idle();
    source.click();
    assert!(source.is_focused());
    hidden.set(false);
    present.set(false);
    screen.run_until_idle();
    source.click();
    assert!(
        screen
            .get_by_role(Role::TextBox, "Owner destination")
            .is_focused(),
        "missing scene region clears relation"
    );
    present.set(true);
    screen.run_until_idle();
    source.click();
    assert!(
        screen
            .get_by_role(Role::Button, "Exact destination")
            .is_focused()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn later_composed_parts_win_and_key_and_accessibility_focus_still_address_the_source() {
    let (screen, calls) = mount_with(real_window(), || {
        let first = node_ref();
        let second = node_ref();
        let calls = signal(0);
        let composed = part(|s| s)
            .focus_on_press(first)
            .with(part(|s| s).focus_on_press(second))
            .with(part(|s| s));
        (
            calls,
            column((
                kivact::control::node()
                    .name("First")
                    .part(
                        kivact::interaction::use_text_edit(kivact::signal(String::new())).control(),
                    )
                    .node_ref(first)
                    .style(style().size(140, 35)),
                kivact::control::node()
                    .name("Second")
                    .part(
                        kivact::interaction::use_text_edit(kivact::signal(String::new())).control(),
                    )
                    .node_ref(second)
                    .style(style().size(140, 35)),
                node()
                    .part(composed)
                    .role(Role::Button)
                    .name("Composed")
                    .focusable(true)
                    .on_press(move || calls.update(|n| *n += 1))
                    .style(style().size(140, 35)),
                node()
                    .part(part(|s| s).focus_on_press(first))
                    .part(part(|s| s).focus_on_press(second))
                    .role(Role::Button)
                    .name("Attached")
                    .focusable(true)
                    .style(style().size(140, 35)),
            )),
        )
    });
    let source = screen.get_by_role(Role::Button, "Composed");
    source.click();
    assert!(screen.get_by_role(Role::TextBox, "Second").is_focused());
    assert_eq!(calls.get(), 1);
    screen.press_tab(false);
    assert!(
        source.is_focused(),
        "Tab does not follow pointer declaration"
    );
    screen.press_key(Key::Enter);
    assert_eq!(calls.get(), 2);
    assert!(source.is_focused());
    screen.get_by_role(Role::TextBox, "First").focus();
    source.a11y(A11yAction::Focus);
    assert!(source.is_focused());
    screen.get_by_role(Role::Button, "Attached").click();
    assert!(screen.get_by_role(Role::TextBox, "Second").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn self_mutual_secondary_and_captured_presses_resolve_only_once() {
    let (screen, ()) = mount_with(real_window(), || {
        let first = node_ref();
        let second = node_ref();
        let own = node_ref();
        (
            (),
            column((
                node()
                    .node_ref(first)
                    .part(part(|s| s).focus_on_press(second))
                    .role(Role::Button)
                    .name("First")
                    .focusable(true)
                    .style(style().size(140, 35)),
                node()
                    .node_ref(second)
                    .part(part(|s| s).focus_on_press(first))
                    .role(Role::Button)
                    .name("Second")
                    .focusable(true)
                    .style(style().size(140, 35)),
                node()
                    .node_ref(own)
                    .part(part(|s| s).focus_on_press(own))
                    .role(Role::Button)
                    .name("Self")
                    .focusable(true)
                    .style(style().size(140, 35)),
            )),
        )
    });
    let first = screen.get_by_role(Role::Button, "First");
    let second = screen.get_by_role(Role::Button, "Second");
    let own = screen.get_by_role(Role::Button, "Self");
    first.click();
    assert!(second.is_focused());
    second.click();
    assert!(first.is_focused());
    own.click();
    assert!(own.is_focused());
    first.click_with(MouseButton::Secondary);
    assert!(second.is_focused());
    let center = first.rect().unwrap().center();
    screen.mouse_move(center.x, center.y);
    screen.mouse_down(MouseButton::Primary);
    assert!(second.is_focused());
    let center = own.rect().unwrap().center();
    screen.mouse_move(center.x, center.y);
    screen.mouse_down(MouseButton::Secondary);
    assert!(second.is_focused(), "captured gestures never refocus");
    screen.mouse_up(MouseButton::Secondary);
    screen.mouse_up(MouseButton::Primary);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_destination_outside_the_modal_falls_back_to_native_focus() {
    let (screen, ()) = mount_with(real_window(), || {
        let outside = node_ref();
        (
            (),
            column((
                kivact::control::node()
                    .name("Outside")
                    .part(
                        kivact::interaction::use_text_edit(kivact::signal(String::new())).control(),
                    )
                    .node_ref(outside)
                    .style(style().size(140, 35)),
                overlay_at(
                    Point::new(240.0, 80.0),
                    Placement::Center,
                    node()
                        .part(part(|s| s).focus_on_press(outside))
                        .role(Role::Button)
                        .name("Inside")
                        .focusable(true)
                        .style(style().size(140, 35)),
                )
                .modal(true),
            )),
        )
    });
    let inside = screen.get_by_role(Role::Button, "Inside");
    inside.click();
    assert!(inside.is_focused());
    assert!(!screen.get_by_role(Role::TextBox, "Outside").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_foreign_app_reference_is_reported_and_does_not_override_a_valid_declaration() {
    let (first_screen, foreign) = mount_with(real_window(), || {
        let reference = node_ref();
        (
            reference,
            kivact::control::node()
                .name("Foreign")
                .part(kivact::interaction::use_text_edit(kivact::signal(String::new())).control())
                .node_ref(reference),
        )
    });
    let (screen, ()) = mount_with(real_window(), move || {
        let own = node_ref();
        (
            (),
            column((
                kivact::control::node()
                    .name("Own")
                    .part(
                        kivact::interaction::use_text_edit(kivact::signal(String::new())).control(),
                    )
                    .node_ref(own)
                    .style(style().size(140, 35)),
                node()
                    .part(part(|s| s).focus_on_press(own).focus_on_press(foreign))
                    .role(Role::Button)
                    .name("Source")
                    .focusable(true)
                    .style(style().size(140, 35)),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Source").click();
    assert!(screen.get_by_role(Role::TextBox, "Own").is_focused());
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(matches!(
        errors[0].kind(),
        kivact::diagnostics::ErrorKind::WrongApp
    ));
    drop(first_screen);
}

#[test]
fn keyed_region_adoption_retains_the_pointer_focus_declaration() {
    let (screen, items) = mount_with(real_window(), || {
        let field = node_ref();
        let items = signal(vec![1u64]);
        let source = canvas(move |d, _| {
            for id in items.get() {
                d.region(RegionId(id), Rect::new(0.0, 0.0, 100.0, 35.0));
            }
        })
        .regions(
            items,
            |id| RegionId(*id),
            move |_, _| {
                part(|s| s)
                    .focus_on_press(field)
                    .role(Role::Option)
                    .name("Keyed source")
                    .disabled(true)
            },
        )
        .style(style().size(140, 35));
        (
            items,
            column((
                kivact::control::node()
                    .name("Field")
                    .part(
                        kivact::interaction::use_text_edit(kivact::signal(String::new())).control(),
                    )
                    .node_ref(field)
                    .style(style().size(140, 35)),
                source,
                node()
                    .role(Role::Button)
                    .name("Other")
                    .focusable(true)
                    .style(style().size(140, 35)),
            )),
        )
    });
    for ids in [vec![1], vec![2], vec![], vec![1]] {
        items.set(ids.clone());
        screen.run_until_idle();
        if ids.is_empty() {
            continue;
        }
        screen.get_by_role(Role::Button, "Other").focus();
        screen
            .get_by_role(Role::Option, "Keyed source")
            .force_click();
        assert!(screen.get_by_role(Role::TextBox, "Field").is_focused());
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
