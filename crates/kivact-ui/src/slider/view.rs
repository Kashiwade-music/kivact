//! Role: The slider components, their track geometry and thumb rendering.

use crate::control_size::{ControlSize, default_control_size};
use crate::helpers::report_unnamed;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::slider::look::AxisState;
use crate::slider::look::SliderTheme;
use kivact_core::custom::{Caller, PartState};
use kivact_core::geometry::Rect;
use kivact_core::motion::Transition;
use kivact_core::state::{Callback, IntoValue, Signal, Value};
use kivact_core::style::{Length, Position, Style, TransitionProp, style, use_theme};
use kivact_core::view::EditPhase;
use kivact_core::view::Variants;
use kivact_core::view::{
    IntoView, Orientation, ReadingDirection, Role, View, node_ref, use_direction,
};
use kivact_core::view::{Node, node};
use kivact_interaction::SliderTouchMotion;
use kivact_interaction::{RangeThumb, ScalarApi, ScalarRange, use_range_scalar, use_scalar};
use kivact_macros::component;

/// The values a slider edits: one, or the two ends of a range.
enum Values {
    One(Signal<f64>),
    Two(Signal<f64>, Signal<f64>),
}

/// The props the slider components share besides their values.
struct Setup {
    label: Option<Value<String>>,
    range: Option<Value<ScalarRange>>,
    step: Option<f64>,
    orientation: Orientation,
    size: Option<Value<ControlSize>>,
    on_edit: Option<Callback<EditPhase>>,
}

/// A themed slider that edits `value` in 0 to 100 unless `range` says
/// otherwise. Its thumb's accessible name is `label`.
///
/// The thumb is a `Role::Slider` with the value and its bounds. The arrow
/// keys step it (ArrowLeft increases in a right-to-left subtree, where the
/// track runs from the right), PageUp and PageDown take ten steps, Home and
/// End go to the bounds, and assistive technology increments, decrements
/// and sets it. A press on the track moves the value to the pointer and a
/// drag follows it; Escape or a canceled pointer during the drag writes the
/// value from before the press back. Each key press, action and drag is one
/// edit (`on_edit`). `step` snaps the value to multiples of the step from the
/// range's minimum and the keys step by it too; a step that is not a
/// positive finite number is a usage error. `orientation` (default
/// horizontal) is the direction of the track, fixed when the slider is
/// built, and `size` (default medium) is the control size of its recipes.
///
/// ```no_run
/// use kivact::prelude::*;
///
/// fn volume() -> impl IntoView {
///     let volume = signal(40.0);
///     slider(volume).label("Volume")
/// }
/// ```
#[component]
pub fn slider(
    value: Signal<f64>,
    label: Option<Value<String>>,
    range: Option<Value<ScalarRange>>,
    step: Option<f64>,
    #[prop(default = Orientation::Horizontal)] orientation: Orientation,
    size: Option<Value<ControlSize>>,
    on_edit: Option<Callback<EditPhase>>,
    #[caller] caller: Caller,
) -> impl Element {
    let setup = Setup {
        label,
        range,
        step,
        orientation,
        size,
        on_edit,
    };
    build(Values::One(value), setup, &caller)
}

/// A themed slider with two thumbs editing the ends of a range, `low` and
/// `high`, which never cross.
///
/// The group is named by `label` and the thumbs are named by the messages
/// `slider.minimum` and `slider.maximum` ("Minimum", "Maximum"). Keys,
/// pointer, assistive technology and edits are as [`slider`]'s; a press on
/// the track moves the thumb nearest to it, and a drag of the filled range
/// between the thumbs moves both ([`RangeScalarApi::body`](kivact_interaction::RangeScalarApi::body)).
#[component]
pub fn range_slider(
    low: Signal<f64>,
    high: Signal<f64>,
    label: Option<Value<String>>,
    range: Option<Value<ScalarRange>>,
    step: Option<f64>,
    #[prop(default = Orientation::Horizontal)] orientation: Orientation,
    size: Option<Value<ControlSize>>,
    on_edit: Option<Callback<EditPhase>>,
    #[caller] caller: Caller,
) -> impl Element {
    let setup = Setup {
        label,
        range,
        step,
        orientation,
        size,
        on_edit,
    };
    build(Values::Two(low, high), setup, &caller)
}

/// Build a slider over `values`: its track, its filled span and its thumbs.
fn build(values: Values, setup: Setup, caller: &Caller) -> Node {
    let Setup {
        label,
        range,
        step,
        orientation,
        size,
        on_edit,
    } = setup;
    if let Some(step) = step
        && !(step.is_finite() && step > 0.0)
    {
        kivact_core::custom::raise(
            kivact_core::ErrorKind::Message,
            format!("slider: the step {step} is not a positive finite number"),
        );
    }
    let variants = caller.variants();
    let size = size.unwrap_or_else(|| default_control_size().into_value());
    let thumb_name = thumb_label(&values, label, caller);
    // `.disabled(..)` on the slider ends a drag in progress and ignores
    // input and assistive technology; the track is disabled with its
    // thumbs, so its recipe's disabled state style dims the whole slider.
    let disabled = caller.disabled();
    let vertical = orientation == Orientation::Vertical;
    let range = configured_range(range, step);
    let thumb_ref = node_ref();
    let touch = use_theme()
        .get()
        .ext::<kivact_interaction::InteractionPolicy>()
        .slider
        .touch;
    let SliderParts {
        track: track_part,
        thumbs,
        fractions,
        body,
    } = slider_parts(values, thumb_name, range, orientation, on_edit, touch);
    let track_state = track_part.state();
    let track_ref = node_ref();
    let geometry = std::rc::Rc::new(Geometry {
        track: track_ref.rect(),
        thumb: thumb_ref.rect(),
        flow: TrackFlow::new(vertical),
    });
    let state = move |input: PartState| AxisState {
        input,
        orientation,
        size: size.get(),
    };
    let filled = filled_span(
        geometry.clone(),
        fractions,
        body,
        track_state,
        state,
        &variants,
        disabled,
    );
    let thumbs = thumb_views(
        thumbs, thumb_ref, geometry, state, touch, &variants, disabled,
    );
    // The track is disabled with its thumbs, so a look dims the whole
    // slider. Its recipe is the base style, so the caller's styles win.
    let root = with_base_recipe_style(
        node().style(style().position(Position::Relative)),
        |r: &SliderTheme| r.track_recipe(),
        &variants,
        move || state(track_state.get()),
    )
    .node_ref(track_ref)
    .part(track_part);
    let root = if fractions.from.is_some() {
        range_group(root, label)
    } else {
        root
    };
    root.child(filled).child(thumbs)
}

/// The direction a slider's value grows along its track.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TrackFlow {
    /// A horizontal track in left-to-right text.
    LeftToRight,
    /// A horizontal track in right-to-left text.
    RightToLeft,
    /// A vertical track: the value grows upwards.
    BottomToTop,
}

impl TrackFlow {
    /// The flow of a track with this axis, reading the inherited
    /// direction at setup.
    fn new(vertical: bool) -> Self {
        if vertical {
            Self::BottomToTop
        } else if use_direction() == ReadingDirection::Rtl {
            Self::RightToLeft
        } else {
            Self::LeftToRight
        }
    }
}

/// Where the thumbs and the filled range go along the track, in the
/// track's own pixels, once both are laid out.
struct Geometry {
    track: Value<Option<Rect>>,
    thumb: Value<Option<Rect>>,
    flow: TrackFlow,
}

impl Geometry {
    fn vertical(&self) -> bool {
        self.flow == TrackFlow::BottomToTop
    }

    /// The offset of a thumb at `fraction` from the track's start edge
    /// (left, or top when vertical), and the thumb's offset across.
    fn thumb_at(&self, fraction: f64) -> Option<(f64, f64)> {
        let (track, thumb) = (self.track.get()?, self.thumb.get()?);
        let f = fraction.clamp(0.0, 1.0);
        Some(if self.vertical() {
            (
                (1.0 - f) * (track.height - thumb.height),
                (track.width - thumb.width) / 2.0,
            )
        } else if self.flow == TrackFlow::RightToLeft {
            (
                (1.0 - f) * (track.width - thumb.width),
                (track.height - thumb.height) / 2.0,
            )
        } else {
            (
                f * (track.width - thumb.width),
                (track.height - thumb.height) / 2.0,
            )
        })
    }

    /// The thumb at `fraction`, placed absolutely.
    fn thumb_style(&self, fraction: f64) -> Style {
        let (along, across) = self.thumb_at(fraction).unwrap_or((0.0, 0.0));
        let s = style().position(Position::Absolute);
        if self.vertical() {
            s.top(along).left(across)
        } else {
            s.left(along).top(across)
        }
    }

    /// The range between the thumbs' centres at `from` and `to`; a single
    /// thumb's range starts at the track's start (its value's minimum).
    fn range_style(&self, from: Option<f64>, to: f64) -> Style {
        let s = style().position(Position::Absolute);
        let (Some(track), Some(thumb)) = (self.track.get(), self.thumb.get()) else {
            return s.size(0, 0);
        };
        let half = if self.vertical() {
            thumb.height / 2.0
        } else {
            thumb.width / 2.0
        };
        let center = |f: f64| self.thumb_at(f).map_or(0.0, |(along, _)| along + half);
        let length = if self.vertical() {
            track.height
        } else {
            track.width
        };
        // The start edge in the track's coordinates: the bottom of a
        // vertical track and the right of a right-to-left one.
        let start = if self.flow != TrackFlow::LeftToRight {
            length
        } else {
            0.0
        };
        let a = from.map_or(start, center);
        let b = center(to);
        let (lo, hi) = (a.min(b), a.max(b));
        if self.vertical() {
            s.top(lo)
                .height(hi - lo)
                .left(0)
                .width(Length::Percent(100.0))
        } else {
            s.left(lo)
                .width(hi - lo)
                .top(0)
                .height(Length::Percent(100.0))
        }
    }
}

/// The values the filled range spans: from the low thumb's (none for a
/// single thumb, whose range starts at the track's start) to the thumb's.
#[derive(Clone, Copy)]
struct Ends {
    from: Option<Value<f64>>,
    to: Value<f64>,
    range: Value<ScalarRange>,
}

impl Ends {
    /// The ends as fractions of the range.
    fn get(self) -> (Option<f64>, f64) {
        let range = self.range.get();
        let span = range.max() - range.min();
        let fraction = |value: Value<f64>| {
            if span > 0.0 {
                (value.get() - range.min()) / span
            } else {
                0.0
            }
        };
        (self.from.map(fraction), fraction(self.to))
    }
}

/// The actual scalar parts and immutable range projections used for composition.
struct SliderParts {
    track: kivact_core::custom::Part<PartState>,
    thumbs: Vec<(ScalarApi, Value<String>)>,
    fractions: Ends,
    body: Option<kivact_core::custom::Part<PartState>>,
}

/// The name of a single slider's thumb. A single slider that nothing names
/// (no `label`, no `field(..)`) is reported; a range's thumbs are named by
/// messages.
fn thumb_label(values: &Values, label: Option<Value<String>>, caller: &Caller) -> Value<String> {
    if matches!(values, Values::One(_)) {
        let named = label.is_some() || caller.name().is_some() || !caller.labelled_by().is_empty();
        report_unnamed(named, "slider", caller.at());
    }
    label.unwrap_or_else(|| String::new().into_value())
}

/// A range's group takes the label as its name. A name the caller gives the
/// slider is set on the same node later, and it wins.
fn range_group(root: Node, label: Option<Value<String>>) -> Node {
    let root = root.role(Role::Group);
    match label {
        Some(label) => root.name(label),
        None => root,
    }
}

fn slider_parts(
    values: Values,
    label: Value<String>,
    range: Value<ScalarRange>,
    orientation: Orientation,
    on_edit: Option<Callback<EditPhase>>,
    touch: SliderTouchMotion,
) -> SliderParts {
    let report = move |phase: EditPhase| {
        if let Some(on_edit) = on_edit {
            on_edit.call(phase);
        }
    };
    // A range's filled span drags both ends.
    let mut body = None;
    let (track_part, thumbs, fractions): (_, Vec<(ScalarApi, Value<String>)>, _) = match values {
        Values::One(value) => {
            let scalar = use_scalar(value, range).on_edit(report);
            if let SliderTouchMotion::Direct { gesture, .. } = touch {
                scalar.touch_gesture(gesture);
            }
            (
                scalar.track(orientation),
                vec![(scalar, label)],
                Ends {
                    from: None,
                    to: scalar.value(),
                    range,
                },
            )
        }
        Values::Two(low, high) => {
            let pair = use_range_scalar(low, high, range).on_edit(report);
            if let SliderTouchMotion::Direct { gesture, .. } = touch {
                pair.end(RangeThumb::Low).touch_gesture(gesture);
            }
            body = Some(pair.body());
            (
                pair.track(orientation),
                vec![
                    (
                        pair.end(RangeThumb::Low),
                        kivact_core::i18n::message("slider.minimum", "Minimum"),
                    ),
                    (
                        pair.end(RangeThumb::High),
                        kivact_core::i18n::message("slider.maximum", "Maximum"),
                    ),
                ],
                Ends {
                    from: Some(pair.end(RangeThumb::Low).value()),
                    to: pair.end(RangeThumb::High).value(),
                    range,
                },
            )
        }
    };
    SliderParts {
        track: track_part,
        thumbs,
        fractions,
        body,
    }
}
/// Render positioned thumbs with optional direct-touch lift motion.
fn thumb_views(
    thumbs: Vec<(ScalarApi, Value<String>)>,
    thumb_ref: kivact_core::view::NodeRef,
    geometry: std::rc::Rc<Geometry>,
    state: impl Fn(PartState) -> AxisState + Copy + 'static,
    touch: SliderTouchMotion,
    variants: &Variants,
    disabled: Option<Value<bool>>,
) -> Vec<View> {
    thumbs
        .into_iter()
        .enumerate()
        .map(|(index, (scalar, name))| {
            let part = scalar.thumb();
            let input = part.state();
            let geometry = geometry.clone();
            let mut thumb = with_recipe_style(
                node()
                    .disabled(move || disabled.is_some_and(|disabled| disabled.get()))
                    .part(part)
                    .name(name),
                |r: &SliderTheme| r.thumb_recipe(),
                variants,
                move || state(input.get().input),
            );
            // Every thumb has the recipe's size; the first one is measured.
            if index == 0 {
                thumb = thumb.node_ref(thumb_ref);
            }
            if let SliderTouchMotion::Direct {
                lifted_scale,
                lift,
                drop,
                ..
            } = touch
                && let Some(lifted) = scalar.touch_lifted()
            {
                node()
                    .child(thumb)
                    .style(move || {
                        let position = geometry.thumb_style(input.get().fraction);
                        if kivact_core::platform::is_motion_reduced().get() {
                            position
                        } else {
                            let lifted = lifted.get();
                            position
                                .scale(if lifted { lifted_scale } else { 1.0 })
                                .transition(
                                    TransitionProp::All,
                                    Transition::spring(if lifted { lift } else { drop }),
                                )
                        }
                    })
                    .into_view()
            } else {
                thumb
                    .style(move || geometry.thumb_style(input.get().fraction))
                    .into_view()
            }
        })
        .collect()
}

/// Position the filled span using actual thumb and track geometry.
fn filled_span(
    geometry: std::rc::Rc<Geometry>,
    fractions: Ends,
    body: Option<kivact_core::custom::Part<PartState>>,
    track_state: Value<PartState>,
    state: impl Fn(PartState) -> AxisState + Copy + 'static,
    variants: &Variants,
    disabled: Option<Value<bool>>,
) -> Node {
    let span = match body {
        Some(body) => node()
            .disabled(move || disabled.is_some_and(|disabled| disabled.get()))
            .part(body),
        None => node(),
    };
    with_recipe_style(
        span,
        |r: &SliderTheme| r.range_recipe(),
        variants,
        move || state(track_state.get()),
    )
    .style(move || {
        let (from, to) = fractions.get();
        geometry.range_style(from, to)
    })
}

/// Resolve default bounds and apply an independently configured keyboard step.
fn configured_range(range: Option<Value<ScalarRange>>, step: Option<f64>) -> Value<ScalarRange> {
    let range = range.unwrap_or_else(|| {
        ScalarRange::new(0.0, 100.0)
            .expect("invariant: 0 to 100 is a valid range")
            .into_value()
    });
    match step {
        Some(step) => kivact_core::state::memo(move || {
            let range = range.get();
            range.with_step(step).unwrap_or(range)
        }),
        None => range,
    }
}
