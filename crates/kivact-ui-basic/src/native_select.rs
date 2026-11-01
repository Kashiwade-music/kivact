//! Role: Native-select look on the select interaction: bordered trigger and chevron.

use crate::Color;
use crate::list_popup::select_popup;
use crate::theme_access::{field_fill, tokens};
use kivact::control::icon;
use kivact::control::{Node, NodeBuilder, node, text};
use kivact::interaction::use_select;
use kivact::primitive::NodeExt;
use kivact::style::{Align, Direction, Justify, Position, Style, pct, style};
use kivact::ui::ListItem;
use kivact::view::{IntoText, IntoValue, IntoView, View};
use kivact::{Binding, Value, memo};
use std::hash::Hash;

/// The heights of a [`NativeSelect`] (`data-size`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NativeSelectSize {
    /// `h-8`.
    #[default]
    Default,
    /// `h-7`.
    Sm,
}

/// A select drawn like the browser's native select (`NativeSelect`).
///
/// Created by [`native_select`]. Behavior (keys, typeahead, the open list)
/// comes from `kivact::interaction::use_select`. Give it an accessible name with
/// `.name(..)`.
pub struct NativeSelect<K: Clone + Eq + Hash + 'static> {
    root: Node,
    options: Value<Vec<ListItem<K>>>,
    value: Binding<Option<K>>,
    placeholder: Option<Value<String>>,
    size: NativeSelectSize,
}

/// A native-looking select over `options`.
///
/// ```no_run
/// use kivact::ui::ListItem;
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
#[track_caller]
pub fn native_select<K: Clone + Eq + Hash + 'static>(
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl Into<Binding<Option<K>>>,
) -> NativeSelect<K> {
    NativeSelect {
        root: node(),
        options: options.into_value(),
        value: value.into(),
        placeholder: None,
        size: NativeSelectSize::Default,
    }
}

impl<K: Clone + Eq + Hash + 'static> NativeSelect<K> {
    /// The height (default [`NativeSelectSize::Default`]).
    pub fn size(mut self, size: NativeSelectSize) -> Self {
        self.size = size;
        self
    }

    /// Text shown while nothing is selected (default `Select…`).
    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.placeholder = Some(text.into_text_value());
        self
    }
}

impl<K: Clone + Eq + Hash + 'static> NodeBuilder for NativeSelect<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<K: Clone + Eq + Hash + 'static> IntoView for NativeSelect<K> {
    fn into_view(self) -> View {
        let NativeSelect {
            mut root,
            options,
            value,
            placeholder,
            size,
        } = self;
        // `.invalid(..)` shows the destructive border and ring.
        let invalid = root
            .accessible_invalid()
            .map(|read| (move || read()).into_value());
        root = root.base_style(move || trigger_style(Cfg { size, invalid }));
        let mut behavior = use_select(options, value);
        if let Some(placeholder) = placeholder {
            behavior = behavior.placeholder(placeholder);
        }
        let trigger = behavior.trigger();
        let anchor = trigger.node_ref();
        let current = value.into_value();
        let has_value = memo(move || {
            current.with(|key| {
                key.as_ref()
                    .is_some_and(|key| options.with(|all| all.iter().any(|o| o.key() == key)))
            })
        });
        let name = root.accessible_name();
        let label = behavior.label();
        let popup_name = memo(move || name.as_ref().map(|read| read()).unwrap_or_default());
        let trigger_node = root
            .part(trigger)
            .child(text(label).style(move || {
                style().grow(1).single_line().color(if has_value.get() {
                    tokens().foreground
                } else {
                    tokens().muted_foreground
                })
            }))
            .child(
                node().child(icon("chevron-down")).style(
                    style()
                        .position(Position::Absolute)
                        .right(10)
                        .top(pct(50) - 8)
                        .size(16, 16)
                        .color(Color::MutedForeground),
                ),
            );
        (
            trigger_node,
            select_popup(behavior, options, anchor, popup_name),
        )
            .into_view()
    }
}

/// What the trigger's default style reads.
#[derive(Clone, Copy)]
struct Cfg {
    size: NativeSelectSize,
    invalid: Option<Value<bool>>,
}

/// native-select.tsx: `h-8 border-input pl-2.5 pr-8 text-xs`, `h-7` when small.
fn trigger_style(cfg: Cfg) -> Style {
    let p = tokens();
    let dark = p.is_dark();
    let height = match cfg.size {
        NativeSelectSize::Default => 32,
        NativeSelectSize::Sm => 28,
    };
    let bad = cfg.invalid.is_some_and(|v| v.get());
    let s = style()
        .position(Position::Relative)
        .shrink(0)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .height(height)
        .min_width(128)
        .padding_left(10)
        .padding_right(32)
        .padding_y(if height == 32 { 4 } else { 2 })
        .radius(0)
        .border(1, p.input)
        .background(field_fill(&p))
        .color(p.foreground)
        .font_size(12)
        .text_wrap(false)
        .focus_visible(crate::theme_access::focus_ring(&p));
    let s = if dark {
        s.hover(style().background(p.input.with_alpha(0.5)))
    } else {
        s
    };
    if bad {
        let ring = p.destructive.with_alpha(if dark { 0.4 } else { 0.2 });
        s.border_color(p.destructive).ring(1, ring)
    } else {
        s
    }
}
