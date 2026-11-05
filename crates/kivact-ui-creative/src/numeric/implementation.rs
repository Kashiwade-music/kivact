//! Role: Numeric controls of the Creative kit: number, value_slider, knob and fader.
use super::machine;

use kivact::draw::Draw;
use kivact::draw::linear_gradient;
use kivact::geometry::{Point, Rect, Size};
use kivact::interaction::{
    DragCoordinates, DragPhase, DragUpdate, FocusReturn, use_drag, use_popup, use_scalar,
};
use kivact::state::IntoBinding;
use kivact::view::{Node, NodeBuilder, node, row, text};

use kivact::interaction::ScalarRange;
use kivact::state::{Binding, Callback, Value, binding, machine, memo, signal};
use kivact::state::{IntoText, IntoValue, callback};
use kivact::style::use_theme;
use kivact::style::{Align, Color, ColorToken, Cursor, Justify, Style, Theme, style};
use kivact::view::EditPhase;
use kivact::view::HandlerResult;
use kivact::view::{
    A11yAction, A11yEvent, A11yRequest, ButtonPress, Handled, IntoView, Key, MouseButton,
    Orientation, Placement, PointerPhase, Role, View, node_ref, overlay, show,
};
use std::rc::Rc;

use super::paint::{draw_fader, draw_knob};
use super::text::{TextParser, display_value, format_scalar, scalar_parser};

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
    root: Node,
    label: Value<String>,
    state: Binding<f64>,
    kind: NumericKind,
    min: Value<f64>,
    max: Value<f64>,
    step: Value<f64>,
    reset: Value<f64>,
    precision: Value<usize>,
    unit: Value<String>,
    tint: Value<PropertyState>,
    logarithmic: Value<bool>,
    timecode: Option<Value<u32>>,
    label_visible: Value<bool>,
    accent: Option<Value<kivact::style::Color>>,
    bipolar: Value<bool>,
    horizontal: Value<bool>,
    edit: Option<Callback<EditPhase>>,
    text_parser: Option<TextParser>,
    mixed: Value<bool>,
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
    Numeric {
        root,
        label,
        state: value,
        kind,
        min: 0.0.into_value(),
        max: 1.0.into_value(),
        step: 0.01.into_value(),
        reset: 0.0.into_value(),
        precision: 3.into_value(),
        unit: String::new().into_value(),
        tint: PropertyState::Plain.into_value(),
        logarithmic: false.into_value(),
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
/// The value is a [`use_scalar`] of the configured [`range`](Numeric::range)
/// and [`step`](Numeric::step): every write is normalized into the range and
/// is an edit of that scalar ([`on_edit`](Numeric::on_edit)).
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
/// A channel fader editing `value`, with square-root normalization over the
/// configured range.
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
            .hover(style().background(ColorToken::Hover))
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
    /// scrub back and reports `Cancel`. Each arrow, Home, End or Backspace key
    /// press, each wheel notch and each assistive technology action is its own
    /// edit: `Begin`, the write, `Commit`. Confirming typed text (Enter, or
    /// leaving the field with a valid draft) is one too; discarding the draft
    /// with Escape reports nothing, because nothing was written. While a scrub
    /// is in progress, the keys, the wheel and assistive technology do not
    /// write. [`EditPhase`] has the rules every component follows.
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
    /// refused with the text of the message `numeric.not_finite`. Confirming
    /// text that does not parse writes nothing and keeps the field and its text
    /// open, even after the focus leaves; Escape discards it.
    pub fn text_parser(
        mut self,
        parser: impl Fn(&str, f64) -> Result<f64, String> + 'static,
    ) -> Self {
        self.text_parser = Some(Rc::new(parser));
        self
    }
    /// Show that the selected owners do not share one value.
    ///
    /// The field shows the text of the message `numeric.mixed` instead of the
    /// value, and assistive technology reads it as indeterminate. Scrubbing,
    /// the arrow keys, Home, End, Backspace, the wheel, Increment, Decrement
    /// and the double click reset are off, since they would move each owner
    /// from a different value. A primary press (or Enter) on a number or slider
    /// opens its text field, which starts from the bound value, and confirming
    /// that text, or an assistive technology `SetValue`, are the only ways to
    /// write; a knob or fader has no text field, so only `SetValue` does.
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
        self.step = 1.0.into_value();
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
    /// Inclusive finite bounds. Invalid ranges become 0..1.
    pub fn range(mut self, range: impl IntoValue<ScalarRange>) -> Self {
        let range = range.into_value();
        self.min = memo(move || range.get().min());
        self.max = memo(move || range.get().max());
        self
    }
    /// Positive keyboard and wheel step.
    pub fn step(mut self, step: impl IntoValue<f64>) -> Self {
        let step = step.into_value();
        self.step = memo(move || {
            let step = step.get();
            if step.is_finite() && step > 0.0 {
                step
            } else {
                0.01
            }
        });
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
    /// Display precision, limited to six places.
    pub fn precision(mut self, precision: impl IntoValue<usize>) -> Self {
        let precision = precision.into_value();
        self.precision = memo(move || precision.get().min(6));
        self
    }
    /// Visible unit suffix.
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
    /// Logarithmic knob mapping when both configured bounds are positive
    /// (constant or reactive).
    #[track_caller]
    pub fn logarithmic(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.logarithmic = enabled.into_value();
        self
    }
}
impl NodeBuilder for Numeric {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Numeric {
    fn into_view(self) -> View {
        let (root, view) = self.prepare();
        let view = Rc::new(view);
        let display = view.display();
        let editor = view.editor();
        let scrub = view.scrub();
        let paint = view.clone();
        let key = view.clone();
        let wheel = view.clone();
        let pointer = view.clone();
        let act = view.clone();
        let root = view
            .metadata(root)
            .draw(move |draw, size| paint.paint(draw, size))
            .on_key_default(move |event| key.key(event))
            .on_wheel_default(move |event| wheel.wheel(event))
            .on_pointer_default(move |event| pointer.pointer(event))
            .part(scrub.source_with({
                let value = view.value;
                move || value.get_untracked()
            }))
            .child(display)
            .child(editor);
        let act = move |event| act.action(event);
        root.a11y_action(A11yAction::Increment, act.clone())
            .a11y_action(A11yAction::Decrement, act.clone())
            .a11y_action(A11yAction::SetValue, act)
            .value_text(view.shown)
            .into_view()
    }
}

impl Numeric {
    fn prepare(self) -> (Node, NumericView) {
        let Self {
            root,
            label,
            state,
            kind,
            min,
            max,
            step,
            reset,
            precision,
            unit,
            tint,
            logarithmic,
            timecode,
            label_visible,
            accent,
            bipolar,
            horizontal,
            edit: edit_observer,
            text_parser,
            mixed,
        } = self;
        let value = state.into_value();
        // Whether the text field is open. It ends itself on Enter, Escape
        // and when it is removed.
        let typing = kivact::interaction::use_inline_edit(signal(None::<()>));
        // The value a scrub's delta is measured from.
        let scrub_phase = machine(machine::State::Idle, || machine::Machine, |_, ()| {});
        let scalar = use_scalar(state, move || {
            ScalarRange::new(min.get(), max.get()).expect("invariant: numeric bounds are finite")
        })
        .key_step(step)
        .expect("invariant: numeric step is finite and positive");
        let scalar = match edit_observer {
            Some(observer) => scalar.on_edit(move |phase| observer.call(phase)),
            None => scalar,
        };
        // A scrub is one edit of the scalar from its press: Escape, the
        // engine and the disposal of the field end it as a Cancel, with the
        // value it started from written back first.
        let pointer = scalar.scrub(use_drag::<f64>(DragCoordinates::Local));
        let reference = node_ref();
        let rect = reference.rect();
        let parse = scalar_parser(text_parser, timecode);
        let shown = display_value(mixed, value, min, max, precision, unit, timecode);
        let mapping = Mapping {
            kind,
            min,
            max,
            logarithmic,
        };
        let view = NumericView {
            label,
            kind,
            min,
            max,
            step,
            reset,
            precision,
            tint,
            timecode,
            label_visible,
            accent,
            bipolar,
            horizontal,
            mixed,
            value,
            typing,
            scrub_phase,
            scalar,
            pointer,
            reference,
            rect,
            parse,
            shown,
            mapping,
        };
        (root, view)
    }
}

// xtask-allow(state-fields): Immutable control inputs and handles; inline editing, scalar scrubbing and scrub phase each own their lifecycle.
struct NumericView {
    label: Value<String>,
    kind: NumericKind,
    min: Value<f64>,
    max: Value<f64>,
    step: Value<f64>,
    reset: Value<f64>,
    precision: Value<usize>,
    tint: Value<PropertyState>,
    timecode: Option<Value<u32>>,
    label_visible: Value<bool>,
    accent: Option<Value<Color>>,
    bipolar: Value<bool>,
    horizontal: Value<bool>,
    mixed: Value<bool>,
    value: Value<f64>,
    typing: kivact::interaction::InlineEditApi<()>,
    scrub_phase: kivact::state::MachineHandle<machine::Machine>,
    scalar: kivact::interaction::ScalarApi,
    pointer: kivact::interaction::DragApi<f64>,
    reference: kivact::view::NodeRef,
    rect: Value<Option<Rect>>,
    parse: TextParser,
    shown: Value<String>,
    mapping: Mapping,
}

#[derive(Clone, Copy)]
struct Mapping {
    kind: NumericKind,
    min: Value<f64>,
    max: Value<f64>,
    logarithmic: Value<bool>,
}
impl Mapping {
    fn normalized(self, v: f64) -> f64 {
        let Self {
            kind,
            min,
            max,
            logarithmic,
        } = self;
        let logarithmic = move || logarithmic.get() && min.get() > 0.0;

        if kind == NumericKind::Fader {
            ((v - min.get()) / (max.get() - min.get()))
                .clamp(0.0, 1.0)
                .sqrt()
        } else if logarithmic() {
            ((v.max(min.get()) / min.get()).ln() / (max.get() / min.get()).ln()).clamp(0.0, 1.0)
        } else {
            ((v - min.get()) / (max.get() - min.get())).clamp(0.0, 1.0)
        }
    }
    fn value_from_fraction(self, n: f64) -> f64 {
        let Self {
            kind,
            min,
            max,
            logarithmic,
        } = self;
        let logarithmic = move || logarithmic.get() && min.get() > 0.0;

        let n = n.clamp(0.0, 1.0);
        if kind == NumericKind::Fader {
            min.get() + (max.get() - min.get()) * n.powi(2)
        } else if logarithmic() {
            min.get() * (max.get() / min.get()).powf(n)
        } else {
            min.get() + (max.get() - min.get()) * n
        }
    }
}
impl NumericView {
    fn display(&self) -> impl IntoView {
        let label = self.label;
        let kind = self.kind;
        let label_visible = self.label_visible;
        let typing = self.typing;
        let shown = self.shown;
        let text_open = move || typing.is_open(&());
        show(
            move || !text_open(),
            move || {
                if matches!(kind, NumericKind::Knob | NumericKind::Fader) {
                    return ().into_view();
                }
                row((
                    text(label)
                        .hidden(move || !label_visible.get())
                        .style(style().single_line().grow(1).shrink(1))
                        .style(style().color(crate::ColorName::Muted)),
                    text(shown)
                        .style(move || {
                            let t = &use_theme().get();
                            style().font_family(t.ext::<crate::tokens::Tokens>().mono_font.as_str())
                        })
                        .style(move || {
                            let visible = label_visible.get();
                            style()
                                .grow(if visible { 0 } else { 1 })
                                .shrink(0)
                                .text_wrap(false)
                                .text_align(if visible {
                                    kivact::style::TextAlign::End
                                } else {
                                    kivact::style::TextAlign::Center
                                })
                                .font_size(11.5)
                                .tabular_nums()
                        }),
                ))
                .style(
                    style()
                        .align_items(Align::Center)
                        .justify_content(Justify::SpaceBetween)
                        .height(kivact::style::pct(100))
                        .padding_x(9)
                        .gap(6),
                )
                .into_view()
            },
        )
    }
    fn editor(&self) -> impl IntoView {
        let typing = self.typing;
        let text_open = move || typing.is_open(&());
        let label = self.label;
        let precision = self.precision;
        let timecode = self.timecode;
        let mixed = self.mixed;
        let value = self.value;
        let typing = self.typing;
        let scalar = self.scalar;
        let reference = self.reference;
        let parse = self.parse.clone();
        let set = move |v: f64| {
            scalar.once(|| scalar.set(v));
        };
        show(text_open, move || {
            let parse_commit = Rc::clone(&parse);
            let parse_reason = Rc::clone(&parse);
            let status = node_ref();
            let from = value.get_untracked();
            let opened = format_scalar(from, precision.get(), "", timecode.map(|fps| fps.get()));
            let draft = signal(opened.clone());
            // Write the typed text when the draft commits: Enter, or leaving
            // the field with a changed text. A text left as it opened writes
            // nothing, unless the value is mixed, which Enter confirms. A text
            // that does not parse is `invalid`, which the draft does not
            // commit.
            let commit_text = move |typed: String| {
                if typed == opened && !mixed.get_untracked() {
                    return;
                }
                if let Ok(next) = parse_commit(&typed, from) {
                    set(next);
                }
            };
            // Why the text does not parse, while it does not; it follows the
            // draft as it is typed.
            let reason = memo(move || parse_reason(&draft.get(), from).err());
            let shown = memo(move || reason.get().is_some_and(|reason| !reason.is_empty()));
            let hint = use_popup(binding(shown, |_: bool| {}))
                .anchor(reference)
                .focus_return(FocusReturn::preserve());
            (
                // A draft that does not parse stays open after the focus
                // leaves; Escape discards it.
                node()
                    .part(
                        typing
                            .field((), label, draft)
                            .on_draft_commit(commit_text)
                            .return_focus(reference)
                            .control(),
                    )
                    .style(move || {
                        let t = &use_theme().get();
                        style().font_family(t.ext::<crate::tokens::Tokens>().mono_font.as_str())
                    })
                    .style(style().width(kivact::style::pct(100)))
                    .invalid(move || reason.get().is_some())
                    .described_by(status),
                show(hint.is_open(), move || {
                    let text_reason = move || reason.get().unwrap_or_default();
                    overlay(
                        reference,
                        Placement::Below,
                        text(text_reason).style(
                            style()
                                .padding_x(6)
                                .padding_y(3)
                                .max_width(300)
                                .radius(5)
                                .font_size(11)
                                .color(crate::ColorName::X)
                                .background(crate::ColorName::Header)
                                .border(1, crate::ColorName::HairStrong),
                        ),
                    )
                    .with_node(move |node| node.part(hint.content()))
                    .with_node(move |node| node.role(Role::Status))
                    .with_node(move |node| node.name(text_reason))
                    .with_node(move |node| node.node_ref(status))
                    .offset(2)
                }),
            )
        })
    }
    fn scrub(&self) -> kivact::interaction::DragApi<f64> {
        let kind = self.kind;
        let step = self.step;
        let horizontal = self.horizontal;
        let typing = self.typing;
        let scrub_phase = self.scrub_phase;
        let scalar = self.scalar;
        let pointer = self.pointer;
        let rect = self.rect;
        let mapping = self.mapping;
        let begin_text = move || typing.open(());
        let normalized = move |v| mapping.normalized(v);
        let from_n = move |v| mapping.value_from_fraction(v);
        pointer
            .on_start(move |_| {
                scrub_phase.send(machine::Event::Press);
            })
            .on_update(move |update: DragUpdate<f64>| {
                let horizontal = horizontal.get();
                let delta = if matches!(kind, NumericKind::Knob | NumericKind::Fader) && !horizontal
                {
                    -update.delta.y
                } else {
                    update.delta.x
                };
                if delta.abs() > 2.0 {
                    let factor = if update.modifiers.ctrl {
                        0.1
                    } else if update.modifiers.shift {
                        10.0
                    } else {
                        1.0
                    };
                    let start = update.payload;
                    if kind == NumericKind::Number {
                        scalar.set(start + delta * step.get() * factor);
                    } else {
                        let extent = rect
                            .get()
                            .map_or(160.0, |r| {
                                if kind == NumericKind::Slider || horizontal {
                                    r.width
                                } else {
                                    r.height
                                }
                            })
                            .max(60.0);
                        scalar.set(from_n(normalized(start) + (delta / extent) * factor));
                    }
                    scrub_phase.send(machine::Event::Move);
                }
            })
            .on_end(move |update: DragUpdate<f64>| {
                let moved = matches!(scrub_phase.get(), machine::State::Moved);
                scrub_phase.send(machine::Event::End);
                if update.phase != DragPhase::Cancel
                    && !moved
                    && matches!(kind, NumericKind::Number | NumericKind::Slider)
                {
                    begin_text();
                }
            })
    }
    fn metadata(&self, root: Node) -> Node {
        let kind = self.kind;
        let min = self.min;
        let max = self.max;
        let horizontal = self.horizontal;
        let mixed = self.mixed;
        let value = self.value;
        let reference = self.reference;
        let shown = self.shown;
        root.node_ref(reference)
            .indeterminate(mixed)
            .value_text(shown)
            .value_min(min)
            .value_max(max)
            .value_now(move || value.get().clamp(min.get(), max.get()))
            .orientation(move || {
                if matches!(kind, NumericKind::Knob | NumericKind::Fader) && !horizontal.get() {
                    Orientation::Vertical
                } else {
                    Orientation::Horizontal
                }
            })
    }
    fn paint(&self, d: &mut Draw, size: Size) {
        let label = self.label;
        let kind = self.kind;
        let tint = self.tint;
        let accent = self.accent;
        let bipolar = self.bipolar;
        let horizontal = self.horizontal;
        let value = self.value;
        let shown = self.shown;
        let mapping = self.mapping;
        let normalized = move |v| mapping.normalized(v);

        let (horizontal, bipolar) = (horizontal.get(), bipolar.get());
        let p = d.theme().ext::<crate::tokens::Tokens>();
        let tint = tint.get();
        let color = match tint {
            PropertyState::Plain => accent.map_or(p.accent, |value| value.get()),
            PropertyState::Keyed => p.key,
            PropertyState::Animated => p.y,
            PropertyState::Driven => crate::color::driven_property_color(),
        };
        let n = normalized(value.get());
        match kind {
            NumericKind::Number => {
                if tint != PropertyState::Plain {
                    d.rounded_rect(size.to_rect(), 7.0, color.with_alpha(0.13));
                }
            }
            NumericKind::Slider => {
                d.rounded_rect(
                    Rect::new(0.0, 0.0, size.width * n, size.height),
                    7.0,
                    color.with_alpha(0.3),
                );
                d.rounded_rect(
                    Rect::new(size.width * n - 1.0, 4.0, 2.0, (size.height - 8.0).max(0.0)),
                    1.0,
                    color,
                );
            }
            NumericKind::Knob => draw_knob(d, size, n, color, &label.get(), &shown.get(), bipolar),
            NumericKind::Fader if horizontal => {
                let y = size.height / 2.0;
                d.rounded_rect(
                    Rect::new(8.0, y - 2.0, (size.width - 16.0).max(0.0), 4.0),
                    2.0,
                    p.strong,
                );
                let x = 8.0 + (size.width - 16.0) * n;
                d.rounded_rect(
                    Rect::new(x - 7.0, y - 10.0, 14.0, 20.0),
                    5.0,
                    linear_gradient(180).colors([p.strong, Color::token(ColorToken::Raised)]),
                );
                d.line(Point::new(x, y - 5.0), Point::new(x, y + 5.0), 1.5, p.text);
            }
            NumericKind::Fader => draw_fader(d, size, n, color, &shown.get()),
        }
    }
    fn key(&self, e: kivact::view::KeyEvent) -> Handled {
        let kind = self.kind;
        let min = self.min;
        let max = self.max;
        let step = self.step;
        let reset = self.reset;
        let mixed = self.mixed;
        let value = self.value;
        let typing = self.typing;
        let scalar = self.scalar;
        let pointer = self.pointer;
        let text_open = move || typing.is_open(&());
        let begin_text = move || typing.open(());
        let set = move |v: f64| {
            scalar.once(|| scalar.set(v));
        };

        let adjusts = matches!(
            e.key(),
            Key::ArrowRight
                | Key::ArrowUp
                | Key::ArrowLeft
                | Key::ArrowDown
                | Key::Home
                | Key::End
                | Key::Backspace
        );
        if text_open() {
            // The text field runs first. Consume the keys it leaves
            // unhandled at the ends of its text, so that none of them
            // writes the value.
            return if adjusts { Handled::Yes } else { Handled::No };
        }
        let text_field = matches!(kind, NumericKind::Number | NumericKind::Slider);
        if e.key() == Key::Enter && !text_field {
            return Handled::No;
        }
        // A scrub owns the value until it ends, and a mixed value
        // takes only typed text, which Enter opens.
        if (adjusts || e.key() == Key::Enter) && (pointer.is_active() || (mixed.get() && adjusts)) {
            return Handled::Yes;
        }
        let factor = if e.modifiers().ctrl {
            0.1
        } else if e.modifiers().shift {
            10.0
        } else {
            1.0
        };
        match e.key() {
            Key::Enter if e.repeat() => {}
            Key::Enter => begin_text(),
            Key::ArrowRight | Key::ArrowUp => set(value.get() + step.get() * factor),
            Key::ArrowLeft | Key::ArrowDown => set(value.get() - step.get() * factor),
            Key::Home => set(min.get()),
            Key::End => set(max.get()),
            Key::Backspace => set(reset.get()),
            _ => return Handled::No,
        }
        Handled::Yes
    }
    fn wheel(&self, e: kivact::view::WheelEvent) -> Handled {
        let step = self.step;
        let mixed = self.mixed;
        let value = self.value;
        let typing = self.typing;
        let scalar = self.scalar;
        let pointer = self.pointer;
        let text_open = move || typing.is_open(&());
        let set = move |v: f64| {
            scalar.once(|| scalar.set(v));
        };

        if text_open() || mixed.get() {
            return Handled::No;
        }
        if pointer.is_active() {
            return Handled::Yes;
        }
        let direction = if e.delta().y > 0.0 { -1.0 } else { 1.0 };
        set(value.get() + step.get() * direction * if e.modifiers().ctrl { 0.1 } else { 1.0 });
        Handled::Yes
    }
    fn pointer(&self, e: kivact::view::PointerEvent) -> Handled {
        let kind = self.kind;
        let reset = self.reset;
        let mixed = self.mixed;
        let typing = self.typing;
        let scalar = self.scalar;
        let pointer = self.pointer;
        let text_open = move || typing.is_open(&());
        let begin_text = move || typing.open(());
        let set = move |v: f64| {
            scalar.once(|| scalar.set(v));
        };

        if text_open() {
            return Handled::Yes;
        }
        if mixed.get() {
            if matches!(
                e.phase(),
                PointerPhase::Down(ButtonPress {
                    button: MouseButton::Primary,
                    ..
                })
            ) && matches!(kind, NumericKind::Number | NumericKind::Slider)
            {
                begin_text();
            }
            return Handled::Yes;
        }
        if matches!(
            e.phase(),
            PointerPhase::Down(ButtonPress {
                button: MouseButton::Primary,
                clicks: 2..
            })
        ) {
            pointer.cancel();
            set(reset.get());
            return Handled::Yes;
        }
        Handled::No
    }
    fn action(&self, event: A11yEvent) {
        let mixed = self.mixed;
        let scalar = self.scalar;
        let pointer = self.pointer;

        let writes = event.region.is_none()
            && !pointer.is_active()
            && match &event.request {
                A11yRequest::SetValue(text) => text.parse::<f64>().ok().is_some_and(f64::is_finite),
                _ => !mixed.get_untracked(),
            };
        if writes {
            scalar.once(|| match &event.request {
                A11yRequest::Increment => scalar.step_by(1.0),
                A11yRequest::Decrement => scalar.step_by(-1.0),
                A11yRequest::SetValue(text) => {
                    text.parse::<f64>().ok().is_some_and(|v| scalar.set(v))
                }
                _ => false,
            });
        }
    }
}
