//! Role: The numeric components (number, value_slider, knob, fader) and their options.

use super::text::{Formatter, TextParser};
use super::types::{NumericKind, PropertyState, Setup};
use crate::color::{FIELD, TEXT};
use crate::size::ROW;
use kivact::component;
use kivact::interaction::{ScalarMapping, ScalarRange, ScalarStep};
use kivact::state::{Callback, IntoValue, Signal, Value, memo};
use kivact::style::{Color, Cursor, Style, color, style};
use kivact::view::{EditPhase, Node, Orientation, Role, node};

/// The props of a numeric component besides its label, value and kind, as
/// its builder function receives them.
struct Options {
    range: Option<Value<ScalarRange>>,
    step: Option<Value<ScalarStep>>,
    reset_to: Option<Value<f64>>,
    precision: Option<Value<usize>>,
    unit: Option<Value<String>>,
    property_state: Value<PropertyState>,
    mapping: Option<ScalarMapping>,
    format: Option<Formatter>,
    text_parser: Option<TextParser>,
    mixed: Value<bool>,
    accent_color: Option<Value<Color>>,
    bipolar: Value<bool>,
    orientation: Option<Value<Orientation>>,
    timecode: Option<Value<u32>>,
    label_visible: Value<bool>,
    on_edit: Option<Callback<EditPhase>>,
    drag_distance: Option<Value<f64>>,
    text_entry: bool,
}

/// A field editing `value` through its label and typed text: horizontal drag
/// scrubs, a click opens direct editing. The value is a
/// [`use_scalar`](kivact::interaction::use_scalar) of `range` and `step`:
/// every write is normalized into the range and is an edit of that scalar.
///
/// `range` (default `0.0..=1.0`, continuous) bounds the value; `step` (default
/// `0.01`, or `1` with `timecode`) quantizes it, and a step that is not a
/// positive finite number acts as `0.01`. `reset_to` (default `0.0`) is what
/// Backspace and a double click reset to. `precision` (default 3, at most 6)
/// and `unit` shape the built-in text; `format` replaces that text, and
/// `text_parser` reads the typed text instead of the built-in numbers,
/// timecode and arithmetic expressions. `timecode` shows whole frames as
/// HH:MM:SS:FF at the given frames per second. `drag_distance` is the pointer
/// travel, in px, that moves a scrub across the whole range. `text_entry`
/// (default true) lets a click open the text field. `label_visible` (default
/// true) hides the label while keeping it as the accessible name, and
/// `property_state` tints the control as keyed, animated or driven. `mixed`
/// shows that the selected owners do not share one value. `accent_color`
/// overrides the accent of the dial or fill, and `bipolar` draws a knob from
/// its midpoint. `mapping` maps the value to the travel of the drag (a fader's
/// default is `ScalarMapping::skewed(0.5)`). `on_edit` reports each edit as an
/// [`EditPhase`], so the app can make one undo step of it.
#[component]
pub fn number(
    label: Value<String>,
    value: Signal<f64>,
    range: Option<Value<ScalarRange>>,
    step: Option<Value<ScalarStep>>,
    reset_to: Option<Value<f64>>,
    precision: Option<Value<usize>>,
    unit: Option<Value<String>>,
    #[prop(default = PropertyState::Plain)] property_state: Value<PropertyState>,
    mapping: Option<ScalarMapping>,
    format: Option<Rc<dyn Fn(f64) -> String>>,
    text_parser: Option<Rc<dyn Fn(&str, f64) -> Result<f64, String>>>,
    #[prop(default = false)] mixed: Value<bool>,
    accent_color: Option<Value<Color>>,
    #[prop(default = false)] bipolar: Value<bool>,
    orientation: Option<Value<Orientation>>,
    timecode: Option<Value<u32>>,
    #[prop(default = true)] label_visible: Value<bool>,
    on_edit: Option<Callback<EditPhase>>,
    drag_distance: Option<Value<f64>>,
    #[prop(default = true)] text_entry: bool,
) -> impl Element {
    numeric(
        label,
        value,
        NumericKind::Number,
        Options {
            range,
            step,
            reset_to,
            precision,
            unit,
            property_state,
            mapping,
            format,
            text_parser,
            mixed,
            accent_color,
            bipolar,
            orientation,
            timecode,
            label_visible,
            on_edit,
            drag_distance,
            text_entry,
        },
    )
}

/// A [`number`] with a tinted fill and an explicit value marker, taking the
/// same props. A value slider scrubs and takes typed text; the plain
/// track-and-thumb slider is `kivact::ui::slider`.
#[component]
pub fn value_slider(
    label: Value<String>,
    value: Signal<f64>,
    range: Option<Value<ScalarRange>>,
    step: Option<Value<ScalarStep>>,
    reset_to: Option<Value<f64>>,
    precision: Option<Value<usize>>,
    unit: Option<Value<String>>,
    #[prop(default = PropertyState::Plain)] property_state: Value<PropertyState>,
    mapping: Option<ScalarMapping>,
    format: Option<Rc<dyn Fn(f64) -> String>>,
    text_parser: Option<Rc<dyn Fn(&str, f64) -> Result<f64, String>>>,
    #[prop(default = false)] mixed: Value<bool>,
    accent_color: Option<Value<Color>>,
    #[prop(default = false)] bipolar: Value<bool>,
    orientation: Option<Value<Orientation>>,
    timecode: Option<Value<u32>>,
    #[prop(default = true)] label_visible: Value<bool>,
    on_edit: Option<Callback<EditPhase>>,
    drag_distance: Option<Value<f64>>,
    #[prop(default = true)] text_entry: bool,
) -> impl Element {
    numeric(
        label,
        value,
        NumericKind::Slider,
        Options {
            range,
            step,
            reset_to,
            precision,
            unit,
            property_state,
            mapping,
            format,
            text_parser,
            mixed,
            accent_color,
            bipolar,
            orientation,
            timecode,
            label_visible,
            on_edit,
            drag_distance,
            text_entry,
        },
    )
}

/// A rotary [`number`] control: vertical drag, wheel, arrow keys and
/// double-click reset. `text_entry` is false by default.
#[component]
pub fn knob(
    label: Value<String>,
    value: Signal<f64>,
    range: Option<Value<ScalarRange>>,
    step: Option<Value<ScalarStep>>,
    reset_to: Option<Value<f64>>,
    precision: Option<Value<usize>>,
    unit: Option<Value<String>>,
    #[prop(default = PropertyState::Plain)] property_state: Value<PropertyState>,
    mapping: Option<ScalarMapping>,
    format: Option<Rc<dyn Fn(f64) -> String>>,
    text_parser: Option<Rc<dyn Fn(&str, f64) -> Result<f64, String>>>,
    #[prop(default = false)] mixed: Value<bool>,
    accent_color: Option<Value<Color>>,
    #[prop(default = false)] bipolar: Value<bool>,
    orientation: Option<Value<Orientation>>,
    timecode: Option<Value<u32>>,
    #[prop(default = true)] label_visible: Value<bool>,
    on_edit: Option<Callback<EditPhase>>,
    drag_distance: Option<Value<f64>>,
    #[prop(default = false)] text_entry: bool,
) -> impl Element {
    numeric(
        label,
        value,
        NumericKind::Knob,
        Options {
            range,
            step,
            reset_to,
            precision,
            unit,
            property_state,
            mapping,
            format,
            text_parser,
            mixed,
            accent_color,
            bipolar,
            orientation,
            timecode,
            label_visible,
            on_edit,
            drag_distance,
            text_entry,
        },
    )
}

/// A channel fader, a [`number`] with the square-root mapping
/// `ScalarMapping::skewed(0.5)` by default. `orientation` lays it out
/// horizontally. `text_entry` is false by default.
#[component]
pub fn fader(
    label: Value<String>,
    value: Signal<f64>,
    range: Option<Value<ScalarRange>>,
    step: Option<Value<ScalarStep>>,
    reset_to: Option<Value<f64>>,
    precision: Option<Value<usize>>,
    unit: Option<Value<String>>,
    #[prop(default = PropertyState::Plain)] property_state: Value<PropertyState>,
    mapping: Option<ScalarMapping>,
    format: Option<Rc<dyn Fn(f64) -> String>>,
    text_parser: Option<Rc<dyn Fn(&str, f64) -> Result<f64, String>>>,
    #[prop(default = false)] mixed: Value<bool>,
    accent_color: Option<Value<Color>>,
    #[prop(default = false)] bipolar: Value<bool>,
    orientation: Option<Value<Orientation>>,
    timecode: Option<Value<u32>>,
    #[prop(default = true)] label_visible: Value<bool>,
    on_edit: Option<Callback<EditPhase>>,
    drag_distance: Option<Value<f64>>,
    #[prop(default = false)] text_entry: bool,
) -> impl Element {
    numeric(
        label,
        value,
        NumericKind::Fader,
        Options {
            range,
            step,
            reset_to,
            precision,
            unit,
            property_state,
            mapping,
            format,
            text_parser,
            mixed,
            accent_color,
            bipolar,
            orientation,
            timecode,
            label_visible,
            on_edit,
            drag_distance,
            text_entry,
        },
    )
}

/// Resolve the options of a numeric control of `kind` and build it.
fn numeric(label: Value<String>, value: Signal<f64>, kind: NumericKind, options: Options) -> Node {
    let Options {
        range,
        step,
        reset_to,
        precision,
        unit,
        property_state,
        mapping,
        format,
        text_parser,
        mixed,
        accent_color,
        bipolar,
        orientation,
        timecode,
        label_visible,
        on_edit,
        drag_distance,
        text_entry,
    } = options;
    let horizontal = orientation.map_or_else(
        || false.into_value(),
        |orientation| memo(move || orientation.get() == Orientation::Horizontal),
    );
    let root = node()
        .focusable(true)
        .name(label)
        .role(if kind == NumericKind::Number {
            Role::SpinButton
        } else {
            Role::Slider
        })
        .style(move || numeric_style(kind, horizontal.get()));
    let setup = Setup {
        label,
        state: value,
        kind,
        range: range.unwrap_or_else(|| {
            ScalarRange::new(0.0, 1.0)
                .expect("invariant: 0..1 is a valid range")
                .into_value()
        }),
        step: match step {
            Some(step) => memo(move || sane_step(step.get())),
            None if timecode.is_some() => ScalarStep::Value(1.0).into_value(),
            None => ScalarStep::Value(0.01).into_value(),
        },
        reset: match reset_to {
            Some(reset) => memo(move || {
                let reset = reset.get();
                if reset.is_finite() { reset } else { 0.0 }
            }),
            None => 0.0_f64.into_value(),
        },
        precision: match precision {
            Some(precision) => memo(move || precision.get().min(6)),
            None => 3_usize.into_value(),
        },
        unit: unit.unwrap_or_else(|| String::new().into_value()),
        tint: property_state,
        mapping: mapping.unwrap_or_else(|| {
            if kind == NumericKind::Fader {
                ScalarMapping::skewed(0.5).expect("invariant: 0.5 is a valid exponent")
            } else {
                ScalarMapping::linear()
            }
        }),
        format,
        drag_distance,
        text_entry,
        timecode: timecode.map(|fps| memo(move || fps.get().clamp(1, 240))),
        label_visible,
        accent: accent_color,
        bipolar,
        horizontal,
        edit: on_edit,
        text_parser,
        mixed,
    };
    super::view::mount(root, setup)
}

/// A step a control can use: a positive finite amount in either space, else
/// `0.01` in value units.
fn sane_step(step: ScalarStep) -> ScalarStep {
    match step {
        ScalarStep::Value(amount) if amount.is_finite() && amount > 0.0 => {
            ScalarStep::Value(amount)
        }
        ScalarStep::Position(amount) if amount.is_finite() && amount > 0.0 => {
            ScalarStep::Position(amount)
        }
        _ => ScalarStep::Value(0.01),
    }
}

/// The root's style of a numeric control of `kind`; a fader is laid out
/// horizontally when `horizontal`.
fn numeric_style(kind: NumericKind, horizontal: bool) -> Style {
    let s = style().border_radius(7).color(TEXT).flex_shrink(0);
    match kind {
        NumericKind::Number | NumericKind::Slider => s
            .height(ROW)
            .width(160)
            .background(FIELD)
            .hover(style().background(color::HOVER))
            .cursor(Cursor::EwResize),
        NumericKind::Knob => s.size(58, 82).cursor(Cursor::NsResize),
        NumericKind::Fader if horizontal => s.size(240, 24).cursor(Cursor::EwResize),
        NumericKind::Fader => s.size(48, 184).cursor(Cursor::NsResize),
    }
}
