//! Role: Render themed sliders over the shared scalar track and thumb parts.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::values::{AxisState, ControlSize};
use super::variants::{Variant, Variants};
use kivact_core::control::{Node, NodeBuilder, node};
use kivact_core::draw::Rect;
use kivact_core::primitive::{NodeExt, PartState};
use kivact_core::style::{Length, Position, Style, style};
use kivact_core::view::{
    HandlerResult, IntoText, IntoValue, IntoView, Orientation, ReadingDirection, Role, View,
    callback, current_direction, node_ref,
};
use kivact_core::{Binding, Callback, Value};
use kivact_interaction::{
    EditPhase, IntoScalarRange, RangeThumb, Scalar, ScalarRange, use_range_scalar, use_scalar,
};

/// The values a slider edits: one, or the two ends of a range.
enum Values {
    One(Binding<f64>),
    Two(Binding<f64>, Binding<f64>),
}

/// A themed slider, created by [`slider`] or [`range_slider`].
pub struct Slider {
    root: Node,
    label: Value<String>,
    values: Values,
    range: Option<Value<ScalarRange>>,
    step: Option<f64>,
    orientation: Orientation,
    size: ControlSize,
    on_edit: Option<Callback<EditPhase>>,
    variants: Variants,
}

/// A themed slider named `label` (its thumb's accessible name) that edits
/// `value` in 0 to 100 unless [`range`](Slider::range) says otherwise.
///
/// The thumb is a `Role::Slider` with the value and its bounds. The arrow
/// keys step it (ArrowLeft increases in a right-to-left subtree, where the
/// track runs from the right), PageUp and PageDown take ten steps, Home and
/// End go to the bounds, and assistive technology increments, decrements
/// and sets it. A press on the track moves the value to the pointer and a
/// drag follows it; Escape or a canceled pointer during the drag writes the
/// value from before the press back. Each key press, action and drag is one
/// edit ([`on_edit`](Slider::on_edit)).
///
/// ```no_run
/// use kivact::prelude::*;
///
/// fn volume() -> impl IntoView {
///     let volume = signal(40.0);
///     slider("Volume", volume)
/// }
/// ```
#[track_caller]
pub fn slider(label: impl IntoText, value: impl Into<Binding<f64>>) -> Slider {
    create(label.into_text_value(), Values::One(value.into()))
}

/// A themed slider named `label` with two thumbs editing the ends of a
/// range, `low` and `high`, which never cross.
///
/// The group is named `label` and the thumbs are named by the messages
/// `slider.minimum` and `slider.maximum` ("Minimum", "Maximum"). Keys,
/// pointer, assistive technology and edits are as [`slider`]'s; a press on
/// the track moves the thumb nearest to it, and a drag of the filled range
/// between the thumbs moves both ([`RangeScalar::body`](kivact_interaction::RangeScalar::body)).
#[track_caller]
pub fn range_slider(
    label: impl IntoText,
    low: impl Into<Binding<f64>>,
    high: impl Into<Binding<f64>>,
) -> Slider {
    create(
        label.into_text_value(),
        Values::Two(low.into(), high.into()),
    )
}

fn create(label: Value<String>, values: Values) -> Slider {
    // The variants are shared: `.variant` adds to them before mount.
    let variants = Variants::default();
    Slider {
        root: node().style(style().position(Position::Relative)),
        label,
        values,
        range: None,
        step: None,
        orientation: Orientation::Horizontal,
        size: super::values::default_control_size(),
        on_edit: None,
        variants,
    }
}

impl Slider {
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

impl Slider {
    /// The bounds of the value (default `0.0..=100.0`): constant bounds
    /// (`0.0..=1.0`), a [`ScalarRange`], or a closure, `Signal` or `Value` of
    /// one when the bounds change.
    #[track_caller]
    pub fn range(mut self, range: impl IntoScalarRange) -> Self {
        self.range = Some(range.into_range());
        self
    }

    /// Snap the value to multiples of `step` from the range's minimum; the
    /// keys step by it too. The default is continuous (the keys step by 1).
    /// A step that is not a positive finite number is a usage error.
    #[track_caller]
    pub fn step(mut self, step: f64) -> Self {
        if !(step.is_finite() && step > 0.0) {
            kivact_core::primitive::raise(
                kivact_core::diagnostics::ErrorKind::Message,
                format!("Slider::step: the step {step} is not a positive finite number"),
                std::panic::Location::caller(),
            );
        }
        self.step = Some(step);
        self
    }

    /// The direction of the track (default [`Orientation::Horizontal`]); a
    /// vertical track runs from bottom to top. A constant: it is the shape of
    /// the layout and the axis the keys and the drag follow.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// The size (default [`ControlSize::Medium`]), given to the recipes in
    /// [`AxisState::size`].
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Report the start and end of each edit ([`EditPhase`]): a drag from
    /// the press to the release, or one key press or action.
    pub fn on_edit<R: HandlerResult>(mut self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.on_edit = Some(callback(f));
        self
    }
}

impl NodeBuilder for Slider {
    /// The track, so modifiers (a width, a test id) apply to it.
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// Where the thumbs and the filled range go along the track, in the
/// track's own pixels, once both are laid out.
struct Geometry {
    track: Value<Option<Rect>>,
    thumb: Value<Option<Rect>>,
    vertical: bool,
    rtl: bool,
}

impl Geometry {
    /// The offset of a thumb at `fraction` from the track's start edge
    /// (left, or top when vertical), and the thumb's offset across.
    fn thumb_at(&self, fraction: f64) -> Option<(f32, f32)> {
        let (track, thumb) = (self.track.get()?, self.thumb.get()?);
        let f = fraction.clamp(0.0, 1.0) as f32;
        Some(if self.vertical {
            (
                (1.0 - f) * (track.height - thumb.height),
                (track.width - thumb.width) / 2.0,
            )
        } else if self.rtl {
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
        if self.vertical {
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
        let half = if self.vertical {
            thumb.height / 2.0
        } else {
            thumb.width / 2.0
        };
        let center = |f: f64| self.thumb_at(f).map_or(0.0, |(along, _)| along + half);
        let length = if self.vertical {
            track.height
        } else {
            track.width
        };
        // The start edge in the track's coordinates: the bottom of a
        // vertical track and the right of a right-to-left one.
        let start = if self.vertical || self.rtl {
            length
        } else {
            0.0
        };
        let a = from.map_or(start, center);
        let b = center(to);
        let (lo, hi) = (a.min(b), a.max(b));
        if self.vertical {
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

impl IntoView for Slider {
    fn into_view(self) -> View {
        let Slider {
            mut root,
            label,
            values,
            range,
            step,
            orientation,
            size,
            on_edit,
            variants,
        } = self;
        // `.disabled(..)` on the slider ends a drag in progress and ignores
        // input and assistive technology; the track is disabled with its
        // thumbs, so its recipe's disabled state style dims the whole slider.
        let disabled = root
            .accessible_disabled()
            .map(|read| (move || read()).into_value());
        let vertical = orientation == Orientation::Vertical;
        let range = range.unwrap_or_else(|| {
            ScalarRange::new(0.0, 100.0)
                .expect("invariant: 0 to 100 is a valid range")
                .into_value()
        });
        let range = match step {
            Some(step) => kivact_core::memo(move || {
                let range = range.get();
                range.with_step(step).unwrap_or(range)
            }),
            None => range,
        };
        let axis = if vertical {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        };
        let orientation = if vertical {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        };
        let report = move |phase: EditPhase| {
            if let Some(on_edit) = on_edit {
                on_edit.call(phase);
            }
        };
        let with_disabled = move |scalar: Scalar| match disabled {
            Some(disabled) => scalar.disabled(disabled),
            None => scalar,
        };
        let thumb_ref = node_ref();
        // A range's filled span drags both ends.
        let mut body = None;
        let (track_part, thumbs, fractions): (_, Vec<(Scalar, Value<String>)>, _) = match values {
            Values::One(value) => {
                let scalar = with_disabled(use_scalar(value, range).on_edit(report));
                (
                    scalar.track(axis),
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
                let pair = match disabled {
                    Some(disabled) => pair.disabled(disabled),
                    None => pair,
                };
                body = Some(pair.body());
                (
                    pair.track(axis),
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
        let track_state = track_part.state();
        let track_ref = track_part.node_ref();
        let geometry = std::rc::Rc::new(Geometry {
            track: track_ref.rect(),
            thumb: thumb_ref.rect(),
            vertical,
            rtl: !vertical && current_direction() == ReadingDirection::Rtl,
        });
        let state = move |input: PartState| AxisState {
            input,
            orientation,
            size,
        };
        let filled = {
            let geometry = geometry.clone();
            let span = match body {
                Some(body) => node().part(body),
                None => node(),
            };
            with_recipe_style(
                span,
                |r| r.slider_recipes().range_recipe(),
                &variants,
                move || state(track_state.get()),
            )
            .style(move || {
                let (from, to) = fractions.get();
                geometry.range_style(from, to)
            })
        };
        let thumbs: Vec<View> = thumbs
            .into_iter()
            .enumerate()
            .map(|(index, (scalar, name))| {
                let part = scalar.thumb();
                let input = part.state();
                let geometry = geometry.clone();
                let mut thumb = with_recipe_style(
                    node().part(part).name(name),
                    |r| r.slider_recipes().thumb_recipe(),
                    &variants,
                    move || state(input.get().input),
                )
                .style(move || geometry.thumb_style(input.get().fraction));
                // Every thumb has the recipe's size; the first one is measured.
                if index == 0 {
                    thumb = thumb.node_ref(thumb_ref);
                }
                thumb.into_view()
            })
            .collect();
        // The track is disabled with its thumbs, so a look dims the whole
        // slider. Its recipe is the base style, so the caller's styles win.
        let root = with_base_recipe_style(
            root,
            |r| r.slider_recipes().track_recipe(),
            &variants,
            move || state(track_state.get()),
        )
        .part(track_part);
        let root = if fractions.from.is_some() {
            let mut root = root.role(Role::Group);
            // The caller's name of the group wins.
            if root.accessible_name().is_none() {
                root.name(label)
            } else {
                root
            }
        } else {
            root
        };
        root.child(filled).child(thumbs).into_view()
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
