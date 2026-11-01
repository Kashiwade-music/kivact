//! Role: Independent controls compose public interaction parts and local styles.

use kivact::primitive::NodeExt;
use kivact::view::HandlerResult;
use std::hash::Hash;

use kivact::Binding;
use kivact::interaction::{
    MenuApi, MenuRow, use_dialog as modal, use_menu as menu_behavior, use_select as select_behavior,
};
use kivact::prelude::*;
use kivact::style::{Align, Style, pct};
use kivact::style::{Position, current_theme};
use kivact::ui::{DialogKind, ListItem, MenuEntry};
use kivact::view::{Placement, View, overlay};

/// A named action whose activation is supplied with the ordinary `on_press`.
/// Native button activation owns click, Enter, Space and accessibility Activate.
#[track_caller]
pub fn button(label: impl IntoText) -> Node {
    let label = label.into_text_value();
    node()
        .role(Role::Button)
        .focusable(true)
        .name(label)
        .style(|| {
            let theme = current_theme();
            style()
                .min_height(theme.control_height())
                .padding_x(theme.space().md)
                .padding_y(theme.space().sm)
                .background(ColorToken::Surface)
                .color(ColorToken::Text)
                .border(1, ColorToken::Border)
                .radius(theme.radius().md)
        })
        .child(text(label))
}

/// A modal dialog with this design system's appearance and shared interaction.
/// Content is created in a fresh owner every time the dialog opens.
///
/// The design system draws the scrim, the panel and the title heading; the
/// shared dialog parts give them the role, the modal focus scope, the
/// dismissal and the title relation.
#[track_caller]
pub fn dialog(
    open: impl Into<Binding<bool>>,
    title: impl IntoText,
    content: impl Fn() -> View + 'static,
) -> impl IntoView {
    let title = title.into_text_value();
    let behavior = modal(open, DialogKind::Modal);
    show(behavior.shown(), move || {
        let heading = row(text(title)
            .part(behavior.title())
            .style(move || title_style(&current_theme()))
            .style(style().grow(1)))
        .style(move || header_style(&current_theme()));
        let body = node()
            .style(move || body_style(&current_theme()))
            .child(content());
        let panel = column((heading, body)).style(move || panel_style(&current_theme()));
        (
            node()
                .style(move || scrim_style(&current_theme()))
                .style(style().position(Position::Fixed).inset(0)),
            overlay(node_ref(), Placement::Center, panel)
                .part(behavior.content())
                .name(title),
        )
    })
}

fn panel_style(theme: &Theme) -> Style {
    style()
        .width(360)
        .max_width(pct(95))
        .padding(theme.space().lg)
        .gap(theme.space().md)
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
        .border(1, ColorToken::Border)
        .radius(theme.radius().md)
}

fn scrim_style(_: &Theme) -> Style {
    style().background(ColorToken::Overlay)
}

fn header_style(theme: &Theme) -> Style {
    style().align_items(Align::Center).gap(theme.space().sm)
}

fn title_style(theme: &Theme) -> Style {
    style()
        .font_size(theme.text().size_large)
        .weight(theme.text().weight_strong)
}

fn body_style(theme: &Theme) -> Style {
    style().gap(theme.space().sm)
}

/// A keyed choice with this design system's trigger and floating list.
#[track_caller]
pub fn select<K: Clone + Eq + Hash + 'static>(
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl Into<Binding<Option<K>>>,
) -> impl IntoView {
    let options = options.into_value();
    let behavior = select_behavior(options, value);
    let trigger = behavior.trigger();
    let anchor = trigger.node_ref();
    (
        node()
            .part(trigger)
            .name("Choice")
            .style(control_style())
            .child(text(behavior.label())),
        show(behavior.shown(), move || {
            let rows = for_each(
                options,
                |option| option.key().clone(),
                move |key, option| {
                    let part = behavior.item(key);
                    let state = part.state();
                    node()
                        .part(part)
                        .style(move || item_style(state.get().selected))
                        .child(text(move || option.with(|option| option.label())))
                },
            );
            overlay(
                anchor,
                Placement::Below,
                column(rows).style(surface_style()),
            )
            .part(behavior.content())
            .name("Choice")
        }),
    )
}

/// A named menu with keyed actions, separators and nested submenu surfaces.
#[track_caller]
pub fn menu<K: Clone + Eq + Hash + 'static, R: HandlerResult>(
    label: impl IntoText,
    entries: impl IntoValue<Vec<MenuEntry<K>>>,
    choose: impl Fn(K) -> R + 'static,
) -> impl IntoView {
    let label = label.into_text_value();
    let behavior = menu_behavior(entries, choose);
    let trigger = behavior.trigger();
    let anchor = trigger.node_ref();
    (
        node()
            .part(trigger)
            .name(label)
            .style(control_style())
            .child(text(label)),
        behavior.show(move || {
            overlay(anchor, Placement::Below, menu_level(behavior))
                .part(behavior.content())
                .name(label)
        }),
    )
}

fn menu_level<K: Clone + Eq + Hash + 'static>(behavior: MenuApi<K>) -> Node {
    let rows = for_each(
        behavior.entries(),
        |(key, _)| key.clone(),
        move |key, entry| match key {
            MenuRow::Separator(_) | MenuRow::Label(_) => node()
                .role(Role::Separator)
                .style(style().height(1).margin_y(3).background(ColorToken::Border))
                .into_view(),
            MenuRow::Item { key, submenu, .. } => {
                let part = behavior.item(key.clone());
                let anchor = part.node_ref();
                let state = part.state();
                let label = memo(move || entry.with(|(_, entry)| entry.label()));
                let row = node()
                    .part(part)
                    .style(move || item_style(state.get().input.focused))
                    .child(text(label));
                if submenu {
                    let child = behavior.submenu(key);
                    row.child(child.show(move || {
                        overlay(anchor, Placement::End, menu_level(child))
                            .part(child.content())
                            .name(label)
                    }))
                    .into_view()
                } else {
                    row.into_view()
                }
            }
        },
    );
    column(rows).style(surface_style())
}

fn control_style() -> Style {
    style()
        .min_width(160)
        .min_height(34)
        .padding_x(12)
        .padding_y(6)
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
        .border(2, ColorToken::Accent)
        .radius(2)
}

fn surface_style() -> Style {
    style()
        .min_width(180)
        .padding(6)
        .gap(2)
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
        .border(2, ColorToken::Accent)
        .radius(2)
}

fn item_style(active: bool) -> Style {
    style()
        .min_height(30)
        .padding_x(10)
        .padding_y(5)
        .background(if active {
            ColorToken::Accent
        } else {
            ColorToken::Surface
        })
        .color(if active {
            ColorToken::AccentText
        } else {
            ColorToken::Text
        })
}

mod code_editor;
mod text_input;
pub use code_editor::{COMMENT, CodeEditor, KEYWORD, code_editor};
pub use text_input::text_input;
