//! Role: Render two themed panes and the shared splitter handle between them.
use super::layer::with_recipe_style;
use super::values::{AxisState, ControlSize};
use super::variants::{Variant, Variants};
use kivact_core::state::IntoValue;
use kivact_core::state::{Binding, Callback, Value};
use kivact_core::state::{IntoText, callback};
use kivact_core::style::{Direction, style};
use kivact_core::view::EditPhase;
use kivact_core::view::{
    HandlerResult, IntoView, Orientation, ReadingDirection, Role, View, node_ref, use_direction,
};
use kivact_core::view::{Node, NodeBuilder, node};
use kivact_interaction::{ScalarRange, ScalarRangeError, use_splitter};

// xtask-allow(state-fields): Setup-only NodeBuilder options; optional panes and observer are independent construction inputs.
/// Two panes and a handle between them, created by [`splitter`].
pub struct Splitter {
    root: Node,
    name: Value<String>,
    size: Binding<f64>,
    range: Value<ScalarRange>,
    orientation: Orientation,
    start: Option<View>,
    end: Option<View>,
    on_edit: Option<Callback<EditPhase>>,
    variants: Variants,
    collapse: Option<f64>,
    snaps: Option<Value<Vec<f64>>>,
    pointer_scale: Value<f64>,
    key_step: Option<f64>,
    handle: Option<View>,
}

/// Two panes side by side with a themed handle between them, named `name`,
/// that edits `size`: the leading pane's width in pixels, within `range`.
///
/// The handle is a `Role::Splitter` with the value and its bounds. While it
/// is focused the arrows step the size (ArrowLeft shrinks the leading pane,
/// ArrowUp when the panes are stacked), Home and End go to the bounds, and
/// assistive technology increments, decrements and sets it. A drag moves it
/// with the pointer; Escape or a canceled pointer during the drag writes the
/// size from before the press back. Each key press, action and drag is one
/// edit ([`on_edit`](Splitter::on_edit)). In a right-to-left subtree the
/// leading pane is on the right and the keys and the drag follow it.
///
/// ```no_run
/// use kivact::interaction::ScalarRange;
/// use kivact::prelude::*;
/// use kivact::ui::splitter;
///
/// fn editor() -> impl IntoView {
///     let sidebar = signal(240.0);
///     let range = ScalarRange::new(120.0, 480.0).expect("a valid range");
///     splitter("Sidebar width", sidebar, range)
///         .start(text("Files"))
///         .end(text("Editor"))
/// }
/// ```
#[track_caller]
pub fn splitter(
    name: impl IntoText,
    size: impl kivact_core::state::IntoBinding<f64>,
    range: impl IntoValue<ScalarRange>,
) -> Splitter {
    Splitter {
        root: node(),
        name: name.into_text_value(),
        size: size.into_binding(),
        range: range.into_value(),
        orientation: Orientation::Horizontal,
        start: None,
        end: None,
        on_edit: None,
        variants: Variants::default(),
        collapse: None,
        snaps: None,
        pointer_scale: 1.0.into_value(),
        key_step: None,
        handle: None,
    }
}

impl Splitter {
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
    pub fn variant(mut self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }
}

impl Splitter {
    /// Collapse on release below this fraction of the current range.
    pub fn collapsible(mut self, threshold: impl Into<f64>) -> Self {
        self.collapse = Some(threshold.into());
        self
    }

    /// Snap on release to the nearest fraction of the current range.
    pub fn snaps(mut self, stops: impl IntoValue<Vec<f64>>) -> Self {
        self.snaps = Some(stops.into_value());
        self
    }

    /// Scale pointer travel into the caller's size units.
    pub fn pointer_scale(mut self, scale: impl IntoValue<f64>) -> Self {
        self.pointer_scale = scale.into_value();
        self
    }

    /// Set a positive finite keyboard step in the caller's size units.
    pub fn key_step(mut self, step: impl Into<f64>) -> Result<Self, ScalarRangeError> {
        let step = step.into();
        if !step.is_finite() || step <= 0.0 {
            return Err(ScalarRangeError::InvalidStep);
        }
        self.key_step = Some(step);
        Ok(self)
    }

    /// Supply the handle's visual content; its canonical input part stays attached.
    pub fn handle(mut self, content: impl IntoView) -> Self {
        self.handle = Some(content.into_view());
        self
    }

    /// The direction the panes are laid out in (default
    /// [`Orientation::Horizontal`]: side by side, `size` is the leading pane's
    /// width). [`Orientation::Vertical`] stacks them top to bottom and `size`
    /// is the top pane's height. A constant: it is the shape of the layout and
    /// the axis the keys and the drag follow.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// The leading pane (left, right in a right-to-left subtree, or top).
    pub fn start(mut self, pane: impl IntoView) -> Self {
        self.start = Some(pane.into_view());
        self
    }

    /// The trailing pane, which takes the rest of the room.
    pub fn end(mut self, pane: impl IntoView) -> Self {
        self.end = Some(pane.into_view());
        self
    }

    /// Report the start and end of each edit ([`EditPhase`]): a drag from
    /// the press to the release, or one key press or action.
    pub fn on_edit<R: HandlerResult>(mut self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.on_edit = Some(callback(f));
        self
    }
}

impl NodeBuilder for Splitter {
    /// The row (or column) of the panes.
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Splitter {
    fn into_view(self) -> View {
        let Splitter {
            mut root,
            name,
            size,
            range,
            orientation,
            start,
            end,
            on_edit,
            variants,
            collapse,
            snaps,
            pointer_scale,
            key_step,
            handle,
        } = self;
        let vertical = orientation == Orientation::Vertical;
        let mut behavior = use_splitter(size, range, orientation).on_edit(move |phase| {
            if let Some(on_edit) = on_edit {
                on_edit.call(phase);
            }
        });
        behavior = behavior.pointer_scale(pointer_scale);
        if let Some(threshold) = collapse {
            behavior = behavior.collapsible(threshold);
        }
        if let Some(stops) = snaps {
            behavior = behavior.snaps(stops);
        }
        if let Some(step) = key_step {
            behavior = behavior
                .key_step(step)
                .expect("invariant: setup validated the keyboard step");
        }
        // `.disabled(..)` on the splitter ignores input and assistive
        // technology and cancels a drag in progress.
        if let Some(disabled) = root.accessible_disabled() {
            behavior = behavior.disabled(move || disabled.get());
        }
        let part = behavior.handle();
        let input = part.state();
        let primary = node_ref();
        let handle = with_recipe_style(
            node().part(part).name(name).controls(primary).child(handle),
            |r| r.splitter_recipes().handle_recipe(),
            &variants,
            move || AxisState {
                input: input.get().input,
                orientation,
                size: ControlSize::Medium,
            },
        );
        let leading = node()
            .role(Role::Group)
            .node_ref(primary)
            .name(name)
            .child(start)
            .style(move || {
                let px = size.get();
                let s = style().shrink(0).min_width(0).min_height(0);
                if vertical { s.height(px) } else { s.width(px) }
            });
        let trailing = node()
            .child(end)
            .style(style().grow(1).basis(0).min_width(0).min_height(0));
        let direction = match (vertical, use_direction().get()) {
            (true, _) => Direction::Column,
            (false, ReadingDirection::Ltr) => Direction::Row,
            (false, ReadingDirection::Rtl) => Direction::RowReverse,
        };
        root.style(style().direction(direction).min_width(0).min_height(0))
            .child(leading)
            .child(handle)
            .child(trailing)
            .into_view()
    }
}
