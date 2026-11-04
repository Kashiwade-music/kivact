//! Role: Toggle group built on the toggle and list interactions, and the toggle look.

use crate::theme_access::{tokens, wide_focus_ring};
use crate::tokens::Tokens;
use kivact::control::{Node, NodeBuilder, node, text};
use kivact::interaction::{ListPolicy, ListSelection, use_list, use_toggle_button};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Color, Direction, Justify, Style, style};
use kivact::ui::{ControlSize, ListItem, Variant};
use kivact::view::{IntoValue, IntoView, Orientation, Role, View, dynamic, for_each, show};
use kivact::{Binding, Signal, Value, binding, memo, signal};
use std::hash::Hash;

/// The two looks of a toggle: the variant of a `kivact::ui::toggle` and of
/// a [`ToggleGroup`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToggleVariant {
    /// Transparent.
    #[default]
    Default,
    /// With a 1 px `input` border.
    Outline,
}

impl Variant for ToggleVariant {}

/// toggle.tsx: `inline-flex center gap-1 text-xs font-medium`, muted when on,
/// with the wide ring while the focus is visible.
pub(crate) fn toggle_style(
    p: &Tokens,
    variant: ToggleVariant,
    size: ControlSize,
    pressed: bool,
    icon_start: bool,
    joined: bool,
) -> Style {
    let (height, pad_left) = match size {
        ControlSize::ExtraSmall | ControlSize::Small => (28.0, if icon_start { 6.0 } else { 10.0 }),
        ControlSize::Medium => (32.0, if icon_start { 8.0 } else { 10.0 }),
        ControlSize::Large => (36.0, if icon_start { 8.0 } else { 10.0 }),
    };
    // `group-data-[spacing=0]/toggle-group:px-2` and `has-data-[icon=inline-start]:pl-1.5`.
    let (pad_left, pad_right) = if joined {
        (if icon_start { 6.0 } else { 8.0 }, 8.0)
    } else {
        (pad_left, 10.0)
    };
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(4)
        .shrink(0)
        .height(height)
        .min_width(height)
        .padding_left(pad_left)
        .padding_right(pad_right)
        .radius(0)
        .border(
            1,
            if variant == ToggleVariant::Outline {
                p.input
            } else {
                Color::TRANSPARENT
            },
        )
        .font_size(12)
        .weight(500)
        .text_wrap(false)
        .color(p.foreground)
        .background(if pressed { p.muted } else { Color::TRANSPARENT })
        .hover(style().background(p.muted))
        // The focused item of a group draws its ring over its neighbours.
        .focus_visible(wide_focus_ring(p).z_index(1))
}

/// Which selection a [`ToggleGroup`] or a [`Combobox`](crate::Combobox) edits.
pub enum Selection<K: Clone + PartialEq + 'static> {
    Multiple(Binding<Vec<K>>),
    Single(Binding<Option<K>>),
}

/// The value a [`ToggleGroup`] or a [`Combobox`](crate::Combobox) edits: a
/// `Binding` or `Signal` of `Vec<K>` (any number of items chosen) or of
/// `Option<K>` (at most one).
///
/// The type of the value decides the component's behavior; it is
/// implemented for exactly those four types (the trait is sealed).
pub trait SelectionValue<K: Clone + PartialEq + 'static>:
    crate::sealed::SelectionValueSeal<K>
{
    #[doc(hidden)]
    fn into_selection(self) -> Selection<K>;
}

impl<K: Clone + PartialEq + 'static> crate::sealed::SelectionValueSeal<K> for Binding<Vec<K>> {}
impl<K: Clone + PartialEq + 'static> crate::sealed::SelectionValueSeal<K> for Signal<Vec<K>> {}
impl<K: Clone + PartialEq + 'static> crate::sealed::SelectionValueSeal<K> for Binding<Option<K>> {}
impl<K: Clone + PartialEq + 'static> crate::sealed::SelectionValueSeal<K> for Signal<Option<K>> {}

impl<K: Clone + PartialEq + 'static> SelectionValue<K> for Binding<Vec<K>> {
    fn into_selection(self) -> Selection<K> {
        Selection::Multiple(self)
    }
}
impl<K: Clone + PartialEq + 'static> SelectionValue<K> for Signal<Vec<K>> {
    fn into_selection(self) -> Selection<K> {
        Selection::Multiple(self.into())
    }
}
impl<K: Clone + PartialEq + 'static> SelectionValue<K> for Binding<Option<K>> {
    fn into_selection(self) -> Selection<K> {
        Selection::Single(self)
    }
}
impl<K: Clone + PartialEq + 'static> SelectionValue<K> for Signal<Option<K>> {
    fn into_selection(self) -> Selection<K> {
        Selection::Single(self.into())
    }
}

/// A row of toggles sharing a variant and size (`ToggleGroup`).
///
/// Created by [`toggle_group`]: several pressed at once when it edits a
/// `Vec<K>`, at most one when it edits an `Option<K>`. The arrow keys move the
/// focus among the items; Space and Enter press the focused one. Each item
/// shows its [`ListItem`]'s icon (a name in the theme's icon set, such as a
/// Lucide name) before its label.
pub struct ToggleGroup<K: Clone + Eq + Hash + 'static> {
    root: Node,
    items: Value<Vec<ListItem<K>>>,
    selection: Selection<K>,
    variant: ToggleVariant,
    size: ControlSize,
    orientation: Orientation,
    spacing: f64,
    icon_only: Value<bool>,
}

fn group<K: Clone + Eq + Hash + 'static>(
    items: Value<Vec<ListItem<K>>>,
    selection: Selection<K>,
) -> ToggleGroup<K> {
    ToggleGroup {
        root: node()
            .role(Role::Group)
            .orientation(Orientation::Horizontal),
        spacing: 2.0,
        items,
        selection,
        variant: ToggleVariant::Default,
        size: ControlSize::Medium,
        orientation: Orientation::Horizontal,
        icon_only: false.into_value(),
    }
}

/// A group of toggles over `items`, editing `value`.
///
/// A `Vec<K>` value lets any number of items be pressed; an `Option<K>` value
/// lets at most one be pressed (pressing it again releases it).
///
/// ```no_run
/// use kivact::ui::ListItem;
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn align() -> impl IntoView {
///     let value = signal(Some("left"));
///     basic::toggle_group(
///         vec![ListItem::new("left", "Left"), ListItem::new("right", "Right")],
///         value,
///     )
///     .variant(basic::ToggleVariant::Outline)
/// }
/// ```
#[track_caller]
pub fn toggle_group<K: Clone + Eq + Hash + 'static>(
    items: impl IntoValue<Vec<ListItem<K>>>,
    value: impl SelectionValue<K>,
) -> ToggleGroup<K> {
    group(items.into_value(), value.into_selection())
}

impl<K: Clone + Eq + Hash + 'static> ToggleGroup<K> {
    /// The look of every item.
    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    /// The size of every item.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Lay the items out in a row (default) or a column. A constant: it is
    /// the axis of the arrow keys and the shape of the layout.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self.root = self.root.orientation(orientation);
        self
    }

    /// The gap between items in 4 px units (default 2); 0 joins the items.
    pub fn spacing(mut self, spacing: f64) -> Self {
        self.spacing = spacing.max(0.0);
        self
    }

    /// Show only the icons while true (constant or reactive); labels remain
    /// the accessible names.
    #[track_caller]
    pub fn icon_only(mut self, icon_only: impl IntoValue<bool>) -> Self {
        self.icon_only = icon_only.into_value();
        self
    }
}

impl<K: Clone + Eq + Hash + 'static> NodeBuilder for ToggleGroup<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<K: Clone + Eq + Hash + 'static> IntoView for ToggleGroup<K> {
    fn into_view(self) -> View {
        let ToggleGroup {
            root,
            items,
            selection,
            variant,
            size,
            orientation,
            spacing,
            icon_only,
        } = self;
        // The list only roves the focus; the real selection is edited by each
        // item's own toggle.
        let roving = signal(None::<K>);
        let nav = use_list(
            items,
            ListSelection::Single(roving.into()),
            ListPolicy::radio(),
        );
        let joined = spacing == 0.0;
        let vertical_group = orientation == Orientation::Vertical;
        let rows = for_each(
            items,
            |item| item.key().clone(),
            move |key, item| {
                let member = nav.item(key.clone());
                let at = key.clone();
                let first_key = key.clone();
                let on = match &selection {
                    Selection::Multiple(value) => {
                        let value = *value;
                        binding(
                            move || value.with(|v| v.contains(&at)),
                            move |on: bool| {
                                let mut v = value.get();
                                v.retain(|k| k != &key);
                                if on {
                                    v.push(key.clone());
                                }
                                value.set(v);
                            },
                        )
                    }
                    Selection::Single(value) => {
                        let value = *value;
                        binding(
                            move || value.with(|v| v.as_ref() == Some(&at)),
                            move |on: bool| value.set(on.then(|| key.clone())),
                        )
                    }
                };
                let pressed = on.into_value();
                let control = use_toggle_button(on).control();
                let glyph = memo(move || item.with(|item| item.icon_value().cloned()));
                let label = memo(move || item.with(|item| item.label()));
                node()
                    .part(member.with(control))
                    .role(Role::Button)
                    .selected(false)
                    .name(label)
                    .child(dynamic(glyph, |glyph| glyph.map(kivact::control::icon)))
                    .child(show(move || !icon_only.get(), move || text(label)))
                    .style(move || {
                        toggle_style(
                            &tokens(),
                            variant,
                            size,
                            pressed.get(),
                            glyph.with(Option::is_some),
                            joined,
                        )
                    })
                    .style(move || {
                        let s = style();
                        let first =
                            items.with(|all| all.first().is_some_and(|i| *i.key() == first_key));
                        if joined && variant == ToggleVariant::Outline && !first {
                            // The neighbours share one border, drawn once
                            // (`border-l-0`, `border-t-0`).
                            if vertical_group {
                                s.border_top(0, Color::TRANSPARENT)
                            } else {
                                s.border_left(0, Color::TRANSPARENT)
                            }
                        } else {
                            s
                        }
                    })
            },
        );
        root.base_style(
            style()
                .direction(if vertical_group {
                    Direction::Column
                } else {
                    Direction::Row
                })
                .align_items(if vertical_group {
                    Align::Stretch
                } else {
                    Align::Center
                })
                .gap(spacing * 4.0)
                .shrink(0)
                .align_self(Align::Start),
        )
        .child(rows)
        .into_view()
    }
}
