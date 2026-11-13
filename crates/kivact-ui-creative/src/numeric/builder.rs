//! Role: Build Creative numeric controls and store their construction options.

use kivact::interaction::{ScalarMapping, ScalarRange, ScalarStep};
use kivact::state::{Binding, Callback, IntoBinding, IntoText, IntoValue, Value, callback, memo};
use kivact::style::use_theme;
use kivact::style::{Cursor, Style, Theme, style};
use kivact::view::EditPhase;
use kivact::view::HandlerResult;
use kivact::view::{Element, Node, Orientation, Role, node};
use std::rc::Rc;

use super::text::{Formatter, TextParser};

/// Numeric geometry, with identical binding and keyboard behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericKind {
    Number,
    Slider,
    Knob,
    Fader,
}

/// Animation-state tint used by the source inspector.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PropertyState {
    #[default]
    Plain,
    Keyed,
    Animated,
    Driven,
}

kivact::state::constant_value!(PropertyState);

/// A range control supporting pointer scrubbing, keyboard, wheel and direct editing.
pub struct Numeric {
    pub(super) root: Node,
    pub(super) label: Value<String>,
    pub(super) state: Binding<f64>,
    pub(super) kind: NumericKind,
    pub(super) range: Value<ScalarRange>,
    pub(super) step: Value<ScalarStep>,
    pub(super) reset: Value<f64>,
    pub(super) precision: Value<usize>,
    pub(super) unit: Value<String>,
    pub(super) tint: Value<PropertyState>,
    pub(super) mapping: ScalarMapping,
    pub(super) format: Option<Formatter>,
    pub(super) drag_distance: Option<Value<f64>>,
    pub(super) text_entry: bool,
    pub(super) timecode: Option<Value<u32>>,
    pub(super) label_visible: Value<bool>,
    pub(super) accent: Option<Value<kivact::style::Color>>,
    pub(super) bipolar: Value<bool>,
    pub(super) horizontal: Value<bool>,
    pub(super) edit: Option<Callback<EditPhase>>,
    pub(super) text_parser: Option<TextParser>,
    pub(super) mixed: Value<bool>,
}

#[track_caller]
fn make(label: impl IntoText, value: Binding<f64>, kind: NumericKind) -> Numeric {
    let label = label.into_text_value();
    let root = node()
        .focusable(true)
        .name(label)
        .role(if kind == NumericKind::Number {
            Role::SpinButton
        } else {
            Role::Slider
        })
        .style(move || {
            let t = &use_theme().get();
            numeric_style(t, kind)
        });
    let range = ScalarRange::new(0.0, 1.0).expect("invariant: 0..1 is a valid range");
    let mapping = if kind == NumericKind::Fader {
        ScalarMapping::skewed(0.5).expect("invariant: 0.5 is a valid exponent")
    } else {
        ScalarMapping::linear()
    };
    Numeric {
        root,
        label,
        state: value,
        kind,
        range: range.into_value(),
        step: ScalarStep::Value(0.01).into_value(),
        reset: 0.0.into_value(),
        precision: 3.into_value(),
        unit: String::new().into_value(),
        tint: PropertyState::Plain.into_value(),
        mapping,
        format: None,
        drag_distance: None,
        text_entry: matches!(kind, NumericKind::Number | NumericKind::Slider),
        timecode: None,
        label_visible: true.into_value(),
        accent: None,
        bipolar: false.into_value(),
        horizontal: false.into_value(),
        edit: None,
        text_parser: None,
        mixed: false.into_value(),
    }
}

/// A label/value field editing `value`; horizontal drag scrubs, a click
/// opens direct editing.
///
/// The value is a [`use_scalar`](kivact::interaction::use_scalar) of the
/// configured [`range`](Numeric::range) and [`step`](Numeric::step): every
/// write is normalized into the range and is an edit of that scalar
/// ([`on_edit`](Numeric::on_edit)).
#[track_caller]
pub fn number(label: impl IntoText, value: impl IntoBinding<f64>) -> Numeric {
    make(label, value.into_binding(), NumericKind::Number)
}

/// A field with a tinted fill and an explicit value marker, editing `value`
/// like [`number`]. A slider field scrubs and takes typed text; the plain
/// track-and-thumb slider is `kivact::ui::slider`.
#[track_caller]
pub fn value_slider(label: impl IntoText, value: impl IntoBinding<f64>) -> Numeric {
    make(label, value.into_binding(), NumericKind::Slider)
}

/// A rotary control editing `value`: vertical drag, wheel, arrow keys and
/// double-click reset.
#[track_caller]
pub fn knob(label: impl IntoText, value: impl IntoBinding<f64>) -> Numeric {
    make(label, value.into_binding(), NumericKind::Knob)
}

/// A channel fader editing `value`, with the square-root mapping
/// [`ScalarMapping::skewed(0.5)`] over the configured range.
#[track_caller]
pub fn fader(label: impl IntoText, value: impl IntoBinding<f64>) -> Numeric {
    make(label, value.into_binding(), NumericKind::Fader)
}

fn numeric_style(t: &Theme, kind: NumericKind) -> Style {
    let p = t.ext::<crate::tokens::Tokens>();
    let s = style().radius(7).color(p.text).shrink(0);
    match kind {
        NumericKind::Number | NumericKind::Slider => s
            .height(p.row)
            .width(160)
            .background(p.field)
            .hover(style().background(kivact::style::ColorToken::Hover))
            .cursor(Cursor::EwResize),
        NumericKind::Knob => s.size(58, 82).cursor(Cursor::NsResize),
        NumericKind::Fader => s.size(48, 184).cursor(Cursor::NsResize),
    }
}

impl Numeric {
    /// Report the start and end of each edit as an [`EditPhase`], so the app
    /// can make one undo step of it.
    ///
    /// A scrub with the pointer is one edit: `Begin` when the pointer is
    /// pressed, `Commit` when it is released. Escape during the scrub, or the
    /// engine canceling the pointer, writes the value from the start of the
    /// scrub back and reports `Cancel`. Each arrow, PageUp, PageDown, Home, End
    /// or Backspace key press, each wheel notch and each assistive technology
    /// action is its own edit: `Begin`, the write, `Commit`. Confirming typed
    /// text (Enter, or leaving the field with a valid draft) is one too;
    /// discarding the draft with Escape reports nothing, because nothing was
    /// written. While a scrub is in progress, the keys, the wheel and assistive
    /// technology do not write. [`EditPhase`] has the rules every component
    /// follows.
    pub fn on_edit<R: HandlerResult>(mut self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.edit = Some(callback(f));
        self
    }

    /// Read the typed text with `parser` instead of the built-in numbers,
    /// timecode and arithmetic expressions.
    ///
    /// It receives the text and the value the edit started from (for relative
    /// entries such as `+5`). It runs while the text is typed, to show whether
    /// it is valid, and again when it is confirmed, so it should be cheap and
    /// free of side effects. `Ok(v)` with a finite `v` is written as a built-in
    /// parse would be: clamped to the range and displayed with the configured
    /// precision. `Err(reason)` marks the field invalid and shows `reason`
    /// under it, which the field is `described_by`. A non-finite `Ok` is
    /// refused with the text of the message `creative.numeric.not_finite`.
    /// Confirming text that does not parse writes nothing and keeps the field
    /// and its text open, even after the focus leaves; Escape discards it. It
    /// also reads an assistive technology `SetValue` whose text is not a plain
    /// number.
    pub fn text_parser(
        mut self,
        parser: impl Fn(&str, f64) -> Result<f64, String> + 'static,
    ) -> Self {
        self.text_parser = Some(Rc::new(parser));
        self
    }

    /// Show the value as `format` returns it: in the field, on the knob or the
    /// fader, to assistive technology (its value text), and as the text the
    /// text field opens with.
    ///
    /// It replaces the built-in display ([`precision`](Self::precision) and
    /// [`unit`](Self::unit), or [`timecode`](Self::timecode)); those still set
    /// the step and the built-in parser. `format` receives the value clamped to
    /// the range. Signals it reads are tracked, so the text follows them (a
    /// locale, a unit choice). A mixed value shows the message
    /// `creative.numeric.mixed` without calling it. When the text is not in the
    /// value's own units (a linear gain shown in dB), also give a
    /// [`text_parser`](Self::text_parser) that reads that text back, or the
    /// typed text is read as a plain number. A later call replaces the earlier
    /// one. Setup only.
    pub fn format(mut self, format: impl Fn(f64) -> String + 'static) -> Self {
        self.format = Some(Rc::new(format));
        self
    }

    /// Show that the selected owners do not share one value.
    ///
    /// The field shows the text of the message `creative.numeric.mixed` instead
    /// of the value, and assistive technology reads it as indeterminate.
    /// Scrubbing, the arrow keys, PageUp, PageDown, Home, End, Backspace, the
    /// wheel, Increment, Decrement and the reset click are off, since they
    /// would move each owner from a different value. A primary press (or
    /// Enter) on a number or slider opens its text field, which starts from the
    /// bound value; a knob or fader opens it with Enter when
    /// [`text_entry`](Self::text_entry) is on. Confirming that text, or an
    /// assistive technology `SetValue`, are the only ways to write.
    pub fn mixed(mut self, mixed: impl IntoValue<bool>) -> Self {
        self.mixed = mixed.into_value();
        self
    }

    /// Override the semantic accent used by this control's dial or fill.
    pub fn accent_color(mut self, color: impl IntoValue<kivact::style::Color>) -> Self {
        self.accent = Some(color.into_value());
        self
    }

    /// Draw a knob's amount from its midpoint, as used for pan and bipolar
    /// gain, while true (constant or reactive).
    #[track_caller]
    pub fn bipolar(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.bipolar = enabled.into_value();
        self
    }

    /// Lay a fader out along `orientation`: [`Orientation::Horizontal`] uses
    /// a horizontal track and horizontal pointer motion, anything else the
    /// default vertical fader (constant or reactive).
    #[track_caller]
    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        let orientation = orientation.into_value();
        let enabled = memo(move || orientation.get() == Orientation::Horizontal);
        self.horizontal = enabled;
        if self.kind == NumericKind::Fader {
            self.root = self.root.style(move || {
                if enabled.get() {
                    style().size(240, 24).cursor(Cursor::EwResize)
                } else {
                    style().size(48, 184).cursor(Cursor::NsResize)
                }
            });
        }
        self
    }

    /// Display whole frames as HH:MM:SS:FF; accept one to four time fields or relative frames.
    pub fn timecode(mut self, fps: impl IntoValue<u32>) -> Self {
        let fps = fps.into_value();
        self.timecode = Some(memo(move || fps.get().clamp(1, 240)));
        self.step = ScalarStep::Value(1.0).into_value();
        self.precision = 0.into_value();
        self
    }

    /// Keep the accessible name while showing only a centered value in the
    /// field when false (constant or reactive).
    #[track_caller]
    pub fn label_visible(mut self, visible: impl IntoValue<bool>) -> Self {
        self.label_visible = visible.into_value();
        self
    }

    /// The bounds and quantization of the value (default `0.0..=1.0`,
    /// continuous): a [`ScalarRange`], whose step quantizes every write, given
    /// as a constant, a closure, a `Signal` or a `Value` of one.
    pub fn range(mut self, range: impl IntoValue<ScalarRange>) -> Self {
        self.range = range.into_value();
        self
    }

    /// The keyboard, wheel and assistive step (default `0.01` in value units,
    /// or `1` with [`timecode`](Self::timecode)): `.step(1.0)` in value units,
    /// `.step(ScalarStep::Position(0.01))` in travel. A number field scrubs one
    /// step per pixel unless [`drag_distance`](Self::drag_distance) is set. An
    /// invalid step acts as `0.01`.
    pub fn step(mut self, step: impl IntoValue<ScalarStep>) -> Self {
        let step = step.into_value();
        self.step = memo(move || sane_step(step.get()));
        self
    }

    /// Pixels of pointer travel that move a scrub across the whole range,
    /// through the mapping (constant or reactive). Default: a number field
    /// moves one step per pixel; a value_slider, knob or fader moves the range
    /// over its own length along the drag (at least 60 px). A value that is not
    /// positive and finite acts as the default.
    pub fn drag_distance(mut self, pixels: impl IntoValue<f64>) -> Self {
        self.drag_distance = Some(pixels.into_value());
        self
    }

    /// Whether the control opens a text field to type its value (default true
    /// for [`number`] and [`value_slider`], false for [`knob`] and [`fader`]).
    ///
    /// Enter opens it on every kind when this is true; a number or value_slider
    /// also opens it on a click that does not move. A knob or fader shows the
    /// field over itself. Typing, confirming and Escape behave as on a number
    /// field. A structural option.
    pub fn text_entry(mut self, enabled: bool) -> Self {
        self.text_entry = enabled;
        self
    }

    /// The value Backspace and a double click reset to.
    pub fn reset_to(mut self, value: impl IntoValue<f64>) -> Self {
        let value = value.into_value();
        self.reset = memo(move || {
            let value = value.get();
            if value.is_finite() { value } else { 0.0 }
        });
        self
    }

    /// Display precision, limited to six places. Without a custom
    /// [`format`](Self::format), it sets how many decimals the built-in text
    /// shows.
    pub fn precision(mut self, precision: impl IntoValue<usize>) -> Self {
        let precision = precision.into_value();
        self.precision = memo(move || precision.get().min(6));
        self
    }

    /// Visible unit suffix of the built-in display. Without a custom
    /// [`format`](Self::format), it is appended to the number.
    pub fn unit(mut self, unit: impl IntoText) -> Self {
        self.unit = unit.into_text_value();
        self
    }

    /// Show a keyed, animated or driven property tint.
    ///
    /// A closure, `Signal` or `Value` changes the tint without rebuilding the
    /// field; a plain `PropertyState` is a constant.
    pub fn property_state(mut self, tint: impl IntoValue<PropertyState>) -> Self {
        self.tint = tint.into_value();
        self
    }

    /// Map the value to the travel of the drag and the drawing (default
    /// linear; a fader's default is [`ScalarMapping::skewed(0.5)`]).
    ///
    /// The text, the bounds, value steps and [`reset_to`](Self::reset_to) stay
    /// in the value's units. A structural option: a constant. Setup only.
    pub fn mapping(mut self, mapping: ScalarMapping) -> Self {
        self.mapping = mapping;
        self
    }
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

impl Element for Numeric {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
