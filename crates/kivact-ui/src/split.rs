//! Role: Render panes sized by fractions with a themed splitter handle between each pair.
//!
//! The panes share the room the handles leave, by the fractions in the
//! caller's `Binding<Vec<f64>>`, which only the handles write. A handle is an
//! [`use_splitter`] whose value is the boundary after its pane, in percent of
//! the shared room; the pixels the panes really take are read back from their
//! rects, so a handle of any thickness (the theme's splitter recipe) moves the
//! boundary exactly as far as the pointer.
use super::layer::with_recipe_style;
use super::values::{AxisState, ControlSize};
use super::variants::{Variant, Variants};
use kivact_core::custom::NodeExt;
use kivact_core::i18n::message;
use kivact_core::state::{Binding, Callback, Value, binding, memo};
use kivact_core::state::{IntoValue, callback};
use kivact_core::style::{Direction, pct, style};
use kivact_core::view::IntoChildren;
use kivact_core::view::{
    HandlerResult, IntoView, NodeRef, Orientation, ReadingDirection, View, current_direction,
    node_ref, show,
};
use kivact_core::view::{Node, NodeBuilder, node};
use kivact_interaction::{EditPhase, ScalarRange, use_splitter};
use std::rc::Rc;

/// The pixels used until the host has laid the panes out.
const UNMEASURED: f64 = 800.0;

/// Panes separated by handles, created by [`split`].
pub struct Split {
    root: Node,
    orientation: Orientation,
    sizes: Binding<Vec<f64>>,
    panes: Vec<View>,
    grip: Value<bool>,
    min_size: f64,
    on_edit: Option<Callback<EditPhase>>,
    variants: Variants,
}

/// A group of `panes`, a tuple of views or a `Vec` of them, whose sizes are
/// the fractions in `sizes`.
///
/// `sizes` holds one fraction per pane, kept by the application; the handles
/// are the only writers and keep the sum of the two panes they separate.
/// Fractions are normalized to sum to 1 when drawn, and a list of the wrong
/// length draws equal panes. Each handle is a `Role::Splitter`
/// ([`use_splitter`](kivact_interaction::use_splitter)): dragging moves the
/// boundary, the arrow keys along the group's axis move it by 1 % (Home and
/// End jump to the limits), Enter or a double click share the two panes
/// equally, and no pane gets smaller than [`Split::min_size`] (10 % of the
/// group unless set). Each key press, action and drag is one edit
/// ([`on_edit`](Split::on_edit)); Escape or a canceled pointer during a drag
/// writes the sizes from before the press back. The panes sit side by side
/// (in a right-to-left subtree the first is on the right and the keys and the
/// drag follow it); [`Split::orientation`] with [`Orientation::Vertical`]
/// stacks them. The group fills its parent, so give the parent a size. The
/// look of a handle is the theme's
/// [`SplitterRecipes::handle`](super::SplitterRecipes), and of its
/// [`grip`](Split::grip) the theme's [`SplitRecipes`](super::SplitRecipes).
///
/// For two panes whose first size is in pixels, use [`splitter`](crate::splitter).
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::ui::split;
///
/// fn workspace() -> impl IntoView {
///     let sizes = signal(vec![0.25f64, 0.75]);
///     split((text("Files"), text("Editor")), sizes)
///         .grip(true)
///         .style(style().height(200).width(400))
/// }
/// ```
#[track_caller]
pub fn split(panes: impl IntoChildren, sizes: impl Into<Binding<Vec<f64>>>) -> Split {
    Split {
        root: node(),
        orientation: Orientation::Horizontal,
        sizes: sizes.into(),
        panes: panes.into_views(),
        grip: false.into_value(),
        min_size: 0.1,
        on_edit: None,
        variants: Variants::default(),
    }
}

impl Split {
    /// Lay the panes out side by side ([`Orientation::Horizontal`], the
    /// default; the handles move left and right) or stacked
    /// ([`Orientation::Vertical`]; the handles move up and down). A constant:
    /// it is the axis the handles drag and the keys move along, fixed when the
    /// group is built.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Draw a grip on every handle while true (constant or reactive); the
    /// theme's [`SplitRecipes::grip`](super::SplitRecipes) styles it.
    #[track_caller]
    pub fn grip(mut self, grip: impl IntoValue<bool>) -> Self {
        self.grip = grip.into_value();
        self
    }

    /// The smallest fraction of the group a pane may take (0.1 by default).
    pub fn min_size(mut self, fraction: f64) -> Self {
        self.min_size = if fraction.is_finite() {
            fraction.clamp(0.0, 0.5)
        } else {
            0.1
        };
        self
    }

    /// Report the start and end of each edit ([`EditPhase`]): a drag from
    /// the press to the release, or one key press or action.
    pub fn on_edit<R: HandlerResult>(mut self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.on_edit = Some(callback(f));
        self
    }

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

impl NodeBuilder for Split {
    /// The row (or column) of the panes and handles.
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// The normalized shares of `count` panes from the application's `sizes`.
fn shares(sizes: &[f64], count: usize) -> Vec<f64> {
    let valid = sizes.len() == count
        && sizes.iter().all(|s| s.is_finite() && *s >= 0.0)
        && sizes.iter().sum::<f64>() > 0.0;
    if !valid {
        return vec![1.0 / count.max(1) as f64; count];
    }
    let total: f64 = sizes.iter().sum();
    sizes.iter().map(|s| *s / total).collect()
}

/// What every handle of a group shares.
#[derive(Clone)]
struct Group {
    sizes: Binding<Vec<f64>>,
    shares: Value<Vec<f64>>,
    shared: Value<f64>,
    axis: Orientation,
    grip: Value<bool>,
    min_size: f64,
    on_edit: Option<Callback<EditPhase>>,
    disabled: Option<Rc<dyn Fn() -> bool>>,
    variants: Variants,
}

impl IntoView for Split {
    fn into_view(self) -> View {
        let Split {
            mut root,
            orientation,
            sizes,
            panes,
            grip,
            min_size,
            on_edit,
            variants,
        } = self;
        let count = panes.len();
        let stacked = orientation == Orientation::Vertical;
        let axis = if stacked {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        };
        let current = sizes.into_value();
        let shares_now = memo(move || current.with(|sizes| shares(sizes, count)));
        let refs: Vec<NodeRef> = (0..count).map(|_| node_ref()).collect();
        let rects: Vec<_> = refs.iter().map(|r| r.rect()).collect();
        // The pixels the panes share: what they take once laid out.
        let shared = memo(move || {
            let lengths: Option<Vec<f64>> = rects
                .iter()
                .map(|rect| rect.get().map(|r| if stacked { r.height } else { r.width }))
                .collect();
            lengths
                .map_or(UNMEASURED, |lengths| lengths.iter().sum())
                .max(1.0)
        });
        // `.disabled(..)` on the group ignores input and assistive technology
        // and cancels a drag in progress.
        let disabled = root.accessible_disabled();
        let group = Group {
            sizes,
            shares: shares_now,
            shared,
            axis,
            grip,
            min_size,
            on_edit,
            disabled,
            variants,
        };
        let mut children: Vec<View> = Vec::with_capacity(count * 2);
        for (at, (pane, reference)) in panes.into_iter().zip(refs).enumerate() {
            children.push(
                node()
                    .node_ref(reference)
                    .style(move || {
                        style()
                            .direction(Direction::Column)
                            .grow(shares_now.get().get(at).copied().unwrap_or(0.0))
                            .basis(0)
                            .min_width(0)
                            .min_height(0)
                    })
                    .child(pane)
                    .into_view(),
            );
            if at + 1 < count {
                children.push(handle(at, &group).into_view());
            }
        }
        let direction = match (stacked, current_direction()) {
            (true, _) => Direction::Column,
            (false, ReadingDirection::Ltr) => Direction::Row,
            (false, ReadingDirection::Rtl) => Direction::RowReverse,
        };
        root.base_style(
            style()
                .direction(direction)
                .width(pct(100))
                .height(pct(100))
                .min_width(0)
                .min_height(0),
        )
        .child(children)
        .into_view()
    }
}

/// The handle between pane `at` and pane `at + 1`.
fn handle(at: usize, group: &Group) -> Node {
    let Group {
        sizes,
        shares: shares_now,
        shared,
        axis,
        grip,
        min_size,
        on_edit,
        disabled,
        variants,
    } = group.clone();
    // The boundary after pane `at`, in percent of the shared room.
    let position = memo(move || shares_now.with(|s| s[..=at].iter().sum::<f64>() * 100.0));
    let write = callback(move |boundary: f64| {
        let mut next = shares_now.get();
        let before: f64 = next[..at].iter().sum();
        let pair = next[at] + next[at + 1];
        let first = (boundary / 100.0 - before).clamp(0.0, pair);
        next[at] = first;
        next[at + 1] = pair - first;
        sizes.set(next);
    });
    let range = memo(move || {
        let s = shares_now.get();
        let low: f64 = s[..at].iter().sum::<f64>() * 100.0;
        let high: f64 = s[..at + 2].iter().sum::<f64>() * 100.0;
        let least = min_size * 100.0;
        ScalarRange::new(low + least, high - least)
            .or_else(|_| ScalarRange::new(low, high))
            .or_else(|_| ScalarRange::new(0.0, 100.0))
            .expect("invariant: the whole 0 to 100 range is valid")
    });
    let mut behavior = use_splitter(
        binding(position, move |value| write.try_call(value)),
        range,
        axis,
    )
    .key_step(1.0)
    .expect("invariant: positive finite key step")
    .reset_to(memo(move || {
        let s = shares_now.get();
        let low: f64 = s[..at].iter().sum::<f64>() * 100.0;
        let high: f64 = s[..at + 2].iter().sum::<f64>() * 100.0;
        Some((low + high) / 2.0)
    }))
    .pointer_scale(move || 100.0 / shared.get())
    .on_edit(move |phase| {
        if let Some(on_edit) = on_edit {
            on_edit.call(phase);
        }
    });
    if let Some(disabled) = disabled {
        behavior = behavior.disabled(move || disabled());
    }
    let part = behavior.handle();
    let input = part.state();
    let state = move || AxisState {
        input: input.get().input,
        orientation: axis,
        size: ControlSize::Medium,
    };
    let mark = {
        let variants = variants.clone();
        move || {
            with_recipe_style(
                node(),
                |recipes| recipes.split_recipes().grip_recipe(),
                &variants,
                state,
            )
        }
    };
    with_recipe_style(
        node().part(part).name(message("split.handle", "Resize")),
        |recipes| recipes.splitter_recipes().handle_recipe(),
        &variants,
        state,
    )
    .child(show(grip, mark))
}
