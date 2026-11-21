//! Role: Native-select look on the select interaction: bordered trigger and chevron.

use crate::recipe_parts::{field_fill, focus_ring, row_style, surface_style};
use crate::{DESTRUCTIVE, FOREGROUND, INPUT, MUTED_FOREGROUND};
use kivact::component;
use kivact::custom::Caller;
use kivact::interaction::{ListItem, SelectApi, use_select};
use kivact::state::{IntoValue, Signal, Value, memo, signal};
use kivact::style::{
    Align, FlexDirection, Justify, Position, Style, ThemeMode, pct, style, use_theme,
};
use kivact::view::{
    Element, IntoView, NodeRef, Placement, View, column, for_each, icon, node, node_ref, overlay,
    scroll, show, text,
};

use kivact::Prop;

/// The heights of a [`NativeSelect`] (`data-size`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum NativeSelectSize {
    /// `h-8`.
    #[default]
    Default,
    /// `h-7`.
    Sm,
}

/// A select drawn like the browser's native select (`NativeSelect`).
///
/// Behavior (keys, typeahead, the open list) comes from
/// `kivact::interaction::use_select`. Give it an accessible name with
/// `.name(..)`, its height with `.size(..)` and the text shown while nothing
/// is selected with `.placeholder(..)`.
///
/// ```no_run
/// use kivact::interaction::{ListItem};
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn fruit() -> impl IntoView {
///     let choice = signal(None::<&'static str>);
///     basic::native_select(
///         vec![ListItem::new("apple", "Apple"), ListItem::new("pear", "Pear")],
///         choice,
///     )
///     .placeholder("Select fruit")
///     .name("Fruit")
/// }
/// ```
#[component]
pub fn native_select<K: kivact::state::ItemKey>(
    options: Value<Vec<ListItem<K>>>,
    selected: Signal<Option<K>>,
    #[prop(default = NativeSelectSize::Default)] size: Value<NativeSelectSize>,
    placeholder: Option<Value<String>>,
    #[caller] caller: Caller,
) -> impl Element {
    // `.invalid(..)` shows the destructive border and ring.
    let invalid = caller.invalid();
    let name = caller.name();
    let root = node().base_style(move || trigger_style(Cfg { size, invalid }));
    let mut behavior = use_select(options, selected, signal(false));
    if let Some(placeholder) = placeholder {
        behavior = behavior.placeholder(placeholder);
    }
    let trigger = behavior.trigger();
    let anchor = node_ref();
    let current = selected.into_value();
    let has_value = memo(move || {
        current.with(|key| {
            key.as_ref()
                .is_some_and(|key| options.with(|all| all.iter().any(|o| o.key() == key)))
        })
    });
    let label = behavior.label();
    let popup_name = memo(move || name.as_ref().map(|read| read.get()).unwrap_or_default());
    root.with_node(|root| root.part(trigger))
        .node_ref(anchor)
        .child(text(label).style(move || {
            style()
                .flex_grow(1.0)
                .single_line()
                .color(if has_value.get() {
                    FOREGROUND
                } else {
                    MUTED_FOREGROUND
                })
        }))
        .child(
            node().child(icon("chevron-down")).style(
                style()
                    .position(Position::Absolute)
                    .right(10)
                    .top(pct(50) - 8)
                    .size(16, 16)
                    .color(MUTED_FOREGROUND),
            ),
        )
        .child(select_popup(behavior, options, anchor, popup_name))
}

/// What the trigger's default style reads.
#[derive(Clone, Copy)]
struct Cfg {
    size: Value<NativeSelectSize>,
    invalid: Option<Value<bool>>,
}

/// native-select.tsx: `h-8 border-input pl-2.5 pr-8 text-xs`, `h-7` when small.
fn trigger_style(cfg: Cfg) -> Style {
    let theme = use_theme().get();
    let dark = theme.mode() == ThemeMode::Dark;
    let height = match cfg.size.get() {
        NativeSelectSize::Default => 32,
        NativeSelectSize::Sm => 28,
    };
    let bad = cfg.invalid.is_some_and(|v| v.get());
    let s = style()
        .position(Position::Relative)
        .flex_shrink(0.0)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .height(height)
        .min_width(128)
        .padding_left(10)
        .padding_right(32)
        .padding_y(if height == 32 { 4 } else { 2 })
        .border_radius(0.0)
        .border(1, INPUT)
        .background(field_fill(dark))
        .color(FOREGROUND)
        .font_size(12)
        .text_wrap(false)
        .focus_visible(focus_ring(&theme));
    let s = if dark {
        s.hover(style().background(INPUT.with_alpha(0.5)))
    } else {
        s
    };
    if bad {
        let ring = DESTRUCTIVE.with_alpha(if dark { 0.4 } else { 0.2 });
        s.border_color(DESTRUCTIVE).outline(1.0, ring)
    } else {
        s
    }
}

/// The open list of `behavior`, anchored below `anchor` and as wide as it.
pub(crate) fn select_popup<K: kivact::state::ItemKey>(
    behavior: SelectApi<K>,
    options: Value<Vec<ListItem<K>>>,
    anchor: NodeRef,
    name: Value<String>,
) -> View {
    show(behavior.is_open(), move || {
        let part = behavior.content();
        let rows = for_each(
            options,
            |item| item.key().clone(),
            move |item| {
                let key = item.get().key().clone();
                let part = behavior.item(key);
                let state = part.state();
                node()
                    .with_node(|root| root.part(part))
                    .style(move || row_style(&state.get()))
                    .child(text(move || item.get().label()))
            },
        );
        overlay(
            anchor,
            Placement::Below,
            scroll(column(rows)).style(move || surface_style().max_height(320.0)),
        )
        .with_node(|root| root.part(part))
        .name(name)
        .offset(4.0)
        .at_least_anchor_width()
    })
    .into_view()
}
