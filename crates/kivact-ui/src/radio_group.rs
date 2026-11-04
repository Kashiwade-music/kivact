//! Role: Render a themed radio group over the shared keyed list parts.
use super::layer::{recipes, with_recipe_style};
use super::values::{CheckableState, ControlSize};
use super::variants::{Variant, Variants};
use kivact_core::state::{Binding, Callback, Value, binding};
use kivact_core::state::{IntoText, IntoValue, callback};
use kivact_core::style::use_theme;
use kivact_core::view::EditPhase;
use kivact_core::view::{
    CheckedState, HandlerResult, IntoView, Orientation, ReadingDirection, Role, View, for_each,
    show, use_direction,
};
use kivact_core::view::{Node, NodeBuilder, icon, node};
use kivact_interaction::{EditSession, ListItem, ListOptions, Selection, use_list};
use std::cell::Cell;
use std::rc::Rc;

/// What an option shows, from its item and its state.
type OptionContent<K> = kivact_core::view::ViewFn<(Value<ListItem<K>>, Value<CheckableState>)>;

/// A set of mutually exclusive choices (`Role::RadioGroup`), created by
/// [`radio_group`].
pub struct RadioGroup<K: kivact_core::state::ItemKey> {
    root: Node,
    options: Value<Vec<ListItem<K>>>,
    value: Binding<Option<K>>,
    orientation: Orientation,
    /// What the group's recipe layer reads once the group is built: its
    /// orientation and whether a row runs from the right.
    layout: Rc<Cell<(Orientation, bool)>>,
    size: ControlSize,
    on_edit: Option<Callback<EditPhase>>,
    item: Option<OptionContent<K>>,
    variants: Variants,
}

/// A themed radio group named `label` over `options` that writes the
/// chosen key to `value`.
///
/// Each option is a `Role::RadioButton` named by its label, showing the
/// theme's indicator (the `dot` icon while chosen), its icon and its label
/// over its description. The arrow keys move the focus and choose (from the
/// right in a right-to-left subtree), Tab enters at the chosen option, a
/// press or Space chooses, and a disabled option is skipped. Each choice is
/// one edit ([`on_edit`](RadioGroup::on_edit)). The label is the group's
/// accessible name; a group always has one, as the options alone do not say
/// what is being chosen. [`item`](RadioGroup::item) replaces what an option
/// shows beside its indicator.
///
/// ```no_run
/// use kivact::prelude::*;
///
/// fn density() -> impl IntoView {
///     let value = signal(Some("comfortable"));
///     radio_group(
///         "Density",
///         vec![
///             ListItem::new("comfortable", "Comfortable"),
///             ListItem::new("compact", "Compact"),
///         ],
///         value,
///     )
/// }
/// ```
#[track_caller]
pub fn radio_group<K: kivact_core::state::ItemKey>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl kivact_core::state::IntoBinding<Option<K>>,
) -> RadioGroup<K> {
    let layout = Rc::new(Cell::new((Orientation::Vertical, false)));
    let read = layout.clone();
    // The variants are shared: `.variant` adds to them before mount.
    let variants = Variants::default();
    // The recipe is the first style layer, so the caller's styles win.
    let root = with_recipe_style(
        node().role(Role::RadioGroup).name(label),
        |r| r.radio_group_recipes().group_recipe(),
        &variants,
        move || read.get().0,
    )
    .style({
        let read = layout.clone();
        move || {
            if read.get().1 {
                kivact_core::style::style().direction(kivact_core::style::Direction::RowReverse)
            } else {
                kivact_core::style::style()
            }
        }
    });
    RadioGroup {
        root,
        options: options.into_value(),
        value: value.into_binding(),
        orientation: Orientation::Vertical,
        layout,
        size: super::values::default_control_size(),
        on_edit: None,
        item: None,
        variants,
    }
}

impl<K: kivact_core::state::ItemKey> RadioGroup<K> {
    /// Give the component a [`Variant`]: a look beyond the meanings its
    /// state carries, which every recipe of the component reads with
    /// [`Look::variant`](crate::Look::variant). A later value of the same
    /// type replaces the earlier one; a theme that does not know the type
    /// draws the plain look.
    ///
    /// The value is a constant: the recipes read the values given before
    /// the component is mounted, and a value never changes afterwards. A
    /// look that changes while the component lives is a state the recipe
    /// reads, or a reactive `.style(..)` of the caller's.
    pub fn variant(self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }
}

impl<K: kivact_core::state::ItemKey> RadioGroup<K> {
    /// Lay the options out in a column (the default) or a row; the recipe
    /// reads it. A constant: the orientation is the shape of the layout.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// The size (default [`ControlSize::Medium`]), given to the item and
    /// indicator recipes in [`CheckableState::size`].
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Report each choice as one edit: `Begin`, the write, `Commit`
    /// ([`EditPhase`]). A choice writes once, so there is nothing to cancel.
    pub fn on_edit<R: HandlerResult>(mut self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.on_edit = Some(callback(f));
        self
    }

    /// What each option shows beside its indicator, built once per option
    /// from its item and its state (both reactive: a renamed item or a
    /// chosen option updates the view). The option itself, its keys, its
    /// semantics, its indicator and its recipes stay the component's; a
    /// look without an indicator gives it no size. Without a slot an option
    /// shows its icon, its label and its description.
    ///
    /// ```no_run
    /// use kivact::prelude::*;
    ///
    /// fn periods() -> impl IntoView {
    ///     let value = signal(Some("week"));
    ///     radio_group(
    ///         "Period",
    ///         vec![ListItem::new("week", "Week"), ListItem::new("month", "Month")],
    ///         value,
    ///     )
    ///     .item(|item, state| {
    ///         text(move || {
    ///             let mark = if state.get().checked.is_checked() { "* " } else { "" };
    ///             format!("{mark}{}", item.get().label())
    ///         })
    ///     })
    /// }
    /// ```
    pub fn item<V: IntoView>(
        mut self,
        content: impl Fn(Value<ListItem<K>>, Value<CheckableState>) -> V + 'static,
    ) -> Self {
        self.item = Some(kivact_core::view::ViewFn::from(move |(item, state)| {
            content(item, state)
        }));
        self
    }
}

impl<K: kivact_core::state::ItemKey> NodeBuilder for RadioGroup<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<K: kivact_core::state::ItemKey> IntoView for RadioGroup<K> {
    fn into_view(self) -> View {
        let RadioGroup {
            root,
            options,
            value,
            orientation,
            layout,
            size,
            on_edit,
            item: slot,
            variants,
        } = self;
        let edit = EditSession::new(move |initial: Option<K>| value.set(initial));
        if let Some(on_edit) = on_edit {
            edit.on_edit(move |phase| on_edit.call(phase));
        }
        let chosen = binding(value.into_value(), move |next: Option<K>| {
            edit.once(value.get(), || value.set(next));
        });
        let group = use_list(options, Selection::Single(chosen), ListOptions::radio());
        let looks = variants.clone();
        let rows = for_each(
            options,
            |item| item.key().clone(),
            move |item| {
                let key = item.with_untracked(|item| item.key().clone());
                let member = group.item(key);
                let state = member.state();
                let check = kivact_core::state::memo(move || {
                    let state = state.get();
                    CheckableState {
                        input: state.input,
                        checked: CheckedState::from_bool(state.selected),
                        size,
                        invalid: false,
                        text: None,
                        switch_drag: None,
                    }
                });
                let lines = looks.clone();
                let mark_looks = looks.clone();
                let indicator = with_recipe_style(
                    node(),
                    |r| r.radio_group_recipes().indicator_recipe(),
                    &looks,
                    move || check.get(),
                )
                .child(show(
                    move || {
                        recipes(&use_theme().get())
                            .radio_group_recipes()
                            .has_custom_mark()
                            || check.get().checked.is_checked()
                    },
                    move || {
                        with_recipe_style(
                            icon("dot"),
                            |r| r.radio_group_recipes().mark_recipe(),
                            &mark_looks,
                            move || check.get(),
                        )
                    },
                ));
                let content = match &slot {
                    Some(slot) => slot.call((item, check)),
                    None => (
                        super::helpers::item_icon(move || item.get().icon().cloned()),
                        super::helpers::label_and_description(
                            move || item.get().label(),
                            move || item.get().description(),
                            move |line| {
                                with_recipe_style(
                                    line,
                                    |r| r.radio_group_recipes().description_recipe(),
                                    &lines,
                                    move || check.get(),
                                )
                            },
                        ),
                    )
                        .into_view(),
                };
                with_recipe_style(
                    node()
                        .part(member)
                        .role(Role::RadioButton)
                        .checked(move || state.get().selected),
                    |r| r.radio_group_recipes().item_recipe(),
                    &looks,
                    move || check.get(),
                )
                .child(indicator)
                .child(content)
            },
        );
        let reversed = orientation == Orientation::Horizontal
            && use_direction().get() == ReadingDirection::Rtl;
        layout.set((orientation, reversed));
        root.orientation(orientation).child(rows).into_view()
    }
}
