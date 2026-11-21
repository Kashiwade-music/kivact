//! Role: The split component, its handles and the boundary arithmetic.

use crate::control_size::ControlSize;
use crate::layer::with_recipe_style;
use crate::slider::look::AxisState;
use crate::split::look::SplitTheme;
use crate::splitter::look::SplitterTheme;
use kivact_core::custom::Caller;
use kivact_core::i18n::message;
use kivact_core::state::{Callback, Signal, Value, binding, memo};
use kivact_core::state::{IntoValue, callback};
use kivact_core::style::{FlexDirection, pct, style};
use kivact_core::view::EditPhase;
use kivact_core::view::Variants;
use kivact_core::view::{
    IntoView, NodeRef, Orientation, ReadingDirection, Role, View, node_ref, show, use_direction,
};
use kivact_core::view::{Node, node};
use kivact_interaction::{ScalarRange, use_splitter};
use kivact_macros::component;

/// The pixels used until the host has laid the panes out.
const UNMEASURED: f64 = 800.0;

/// A group of `panes`, a tuple of views or a `Vec` of them, whose sizes are
/// the fractions in `sizes`.
///
/// `sizes` holds one fraction per pane, kept by the application; the handles
/// are the only writers and keep the sum of the two panes they separate.
/// Fractions are normalized to sum to 1 when drawn, and a list of the wrong
/// length draws equal panes. Each handle is a `Role::Splitter`
/// ([`use_splitter`](kivact_interaction::use_splitter)): dragging moves the
/// boundary, the arrow keys along the group's axis move it by 1 % (Home and
/// End jump to the limits), Enter collapses and restores the primary pane,
/// a double click shares the two panes equally, and no pane gets smaller than
/// `min_size` (a fraction from 0 to 0.5, 0.1 by default). Each key press,
/// action and drag is one edit (`on_edit`); Escape or a canceled pointer
/// during a drag writes the sizes from before the press back. The panes sit
/// side by side (in a right-to-left subtree the first is on the right and the
/// keys and the drag follow it); `orientation` set to
/// [`Orientation::Vertical`] stacks them. The group fills its parent, so give
/// the parent a size. The look of a handle is the theme's
/// [`SplitterTheme::handle`](crate::theme::SplitterTheme), and of its `grip`
/// (shown while `grip` is true) the theme's
/// [`SplitTheme`](crate::split::look::SplitTheme).
///
/// For two panes whose first size is in pixels, use [`splitter`](crate::splitter).
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::ui::split;
///
/// fn workspace() -> impl IntoView {
///     let sizes = signal(vec![0.25f64, 0.75]);
///     split(sizes, (text("Files"), text("Editor")))
///         .grip(true)
///         .style(style().height(200).width(400))
/// }
/// ```
#[component]
pub fn split(
    sizes: Signal<Vec<f64>>,
    panes: Vec<View>,
    #[prop(default = Orientation::Horizontal)] orientation: Orientation,
    grip: Option<Value<bool>>,
    #[prop(default = 0.1)] min_size: f64,
    on_edit: Option<Callback<EditPhase>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
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
    // `.disabled(..)` on the group ignores input and assistive technology
    // and cancels a drag in progress.
    let group = SplitGroupOptions {
        sizes,
        shares: shares_now,
        shared: shared_room(&refs, stacked),
        axis,
        grip: grip.unwrap_or_else(|| false.into_value()),
        min_size: if min_size.is_finite() {
            min_size.clamp(0.0, 0.5)
        } else {
            0.1
        },
        on_edit,
        disabled: caller.disabled(),
        variants,
    };
    let children = panes_and_handles(panes, &refs, &group);
    let direction = match (stacked, use_direction()) {
        (true, _) => FlexDirection::Column,
        (false, ReadingDirection::Ltr) => FlexDirection::Row,
        (false, ReadingDirection::Rtl) => FlexDirection::RowReverse,
    };
    node()
        .base_style(
            style()
                .flex_direction(direction)
                .width(pct(100))
                .height(pct(100))
                .min_width(0)
                .min_height(0),
        )
        .child(children)
}

/// The pixels the panes share: what they take once laid out.
fn shared_room(refs: &[NodeRef], stacked: bool) -> Value<f64> {
    let rects: Vec<_> = refs.iter().map(|r| r.rect()).collect();
    memo(move || {
        let lengths: Option<Vec<f64>> = rects
            .iter()
            .map(|rect| rect.get().map(|r| if stacked { r.height } else { r.width }))
            .collect();
        lengths
            .map_or(UNMEASURED, |lengths| lengths.iter().sum())
            .max(1.0)
    })
}

/// Each pane in its own share of the room, with the handle after every pane
/// but the last.
fn panes_and_handles(panes: Vec<View>, refs: &[NodeRef], group: &SplitGroupOptions) -> Vec<View> {
    let count = panes.len();
    let shares_now = group.shares;
    let mut children: Vec<View> = Vec::with_capacity(count * 2);
    for (at, (pane, reference)) in panes.into_iter().zip(refs.iter().copied()).enumerate() {
        children.push(
            node()
                .role(Role::Group)
                .node_ref(reference)
                .style(move || {
                    style()
                        .flex_direction(FlexDirection::Column)
                        .flex_grow(shares_now.get().get(at).copied().unwrap_or(0.0))
                        .flex_basis(0)
                        .min_width(0)
                        .min_height(0)
                })
                .child(pane)
                .into_view(),
        );
        if at + 1 < count {
            children.push(handle(at, reference, group).into_view());
        }
    }
    children
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
struct SplitGroupOptions {
    sizes: Signal<Vec<f64>>,
    shares: Value<Vec<f64>>,
    shared: Value<f64>,
    axis: Orientation,
    grip: Value<bool>,
    min_size: f64,
    on_edit: Option<Callback<EditPhase>>,
    disabled: Option<Value<bool>>,
    variants: Variants,
}

/// The handle between pane `at` and pane `at + 1`.
fn handle(at: usize, primary: NodeRef, group: &SplitGroupOptions) -> Node {
    let SplitGroupOptions {
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
    let range = memo(move || boundary_range(&shares_now.get(), at, min_size));
    let behavior = use_splitter(
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
    let part = behavior.handle();
    let input = part.state();
    let state = move || AxisState {
        input: input.get().input,
        orientation: axis,
        size: ControlSize::Medium,
    };
    let mark = {
        let variants = variants.clone();
        move || with_recipe_style(node(), |r: &SplitTheme| r.grip_recipe(), &variants, state)
    };
    with_recipe_style(
        node()
            .disabled(move || disabled.is_some_and(|disabled| disabled.get()))
            .part(part)
            .name(message("split.handle", "Resize"))
            .controls(primary),
        |r: &SplitterTheme| r.handle_recipe(),
        &variants,
        state,
    )
    .child(show(grip, mark))
}

/// Constrain one boundary while preserving both panes' minimum shares.
fn boundary_range(shares: &[f64], at: usize, min_size: f64) -> ScalarRange {
    let low: f64 = shares[..at].iter().sum::<f64>() * 100.0;
    let high: f64 = shares[..at + 2].iter().sum::<f64>() * 100.0;
    let least = min_size * 100.0;
    ScalarRange::new(low + least, high - least)
        .or_else(|_| ScalarRange::new(low, high))
        .or_else(|_| ScalarRange::new(0.0, 100.0))
        .expect("invariant: the whole 0 to 100 range is valid")
}
