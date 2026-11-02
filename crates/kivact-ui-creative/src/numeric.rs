//! Role: Numeric controls of the Creative kit: number, value_slider, knob and fader.

use kivact::control::{Node, NodeBuilder, node, row, text};
use kivact::draw::{Draw, Path, Point, Rect, Size, TextStyle};
use kivact::draw::{LineCap, LineJoin, Stroke, linear_gradient};
use kivact::i18n::message;
use kivact::interaction::{
    DragCoordinates, DragPhase, DragUpdate, FocusReturn, use_drag, use_popup, use_scalar,
};
use kivact::primitive::NodeExt;
use kivact::style::current_theme;
use kivact::style::{Align, Color, ColorToken, Cursor, Justify, Style, Theme, style};
use kivact::ui::{EditPhase, ScalarRange};
use kivact::view::HandlerResult;
use kivact::view::{
    A11yAction, A11yEvent, A11yRequest, ButtonPress, Handled, IntoText, IntoValue, IntoView, Key,
    MouseButton, Orientation, Placement, PointerPhase, Role, View, callback, node_ref, overlay,
    show,
};
use kivact::{Binding, Callback, Value, binding, memo, on_mount, signal, var};
use std::rc::Rc;

type TextParser = Rc<dyn Fn(&str, f64) -> Result<f64, String>>;

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

kivact::view::constant_value!(PropertyState);

/// A range control supporting pointer scrubbing, keyboard, wheel and direct editing.
pub struct Numeric {
    root: Node,
    label: Value<String>,
    state: Binding<f64>,
    kind: NumericKind,
    min: f64,
    max: f64,
    step: f64,
    reset: f64,
    precision: usize,
    unit: String,
    tint: Value<PropertyState>,
    logarithmic: Value<bool>,
    timecode: Option<u32>,
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
            let t = &current_theme();
            numeric_style(t, kind)
        });
    Numeric {
        root,
        label,
        state: value,
        kind,
        min: 0.0,
        max: 1.0,
        step: 0.01,
        reset: 0.0,
        precision: 3,
        unit: String::new(),
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
pub fn number(label: impl IntoText, value: impl Into<Binding<f64>>) -> Numeric {
    make(label, value.into(), NumericKind::Number)
}
/// A field with a tinted fill and an explicit value marker, editing `value`
/// like [`number`]. A slider field scrubs and takes typed text; the plain
/// track-and-thumb slider is `kivact::ui::slider`.
#[track_caller]
pub fn value_slider(label: impl IntoText, value: impl Into<Binding<f64>>) -> Numeric {
    make(label, value.into(), NumericKind::Slider)
}
/// A rotary control editing `value`: vertical drag, wheel, arrow keys and
/// double-click reset.
#[track_caller]
pub fn knob(label: impl IntoText, value: impl Into<Binding<f64>>) -> Numeric {
    make(label, value.into(), NumericKind::Knob)
}
/// A channel fader editing `value`, with square-root normalization over the
/// configured range.
#[track_caller]
pub fn fader(label: impl IntoText, value: impl Into<Binding<f64>>) -> Numeric {
    make(label, value.into(), NumericKind::Fader)
}

fn numeric_style(t: &Theme, kind: NumericKind) -> Style {
    let p = t.get::<crate::tokens::Tokens>();
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
    pub fn timecode(mut self, fps: u32) -> Self {
        self.timecode = Some(fps.clamp(1, 240));
        self.step = 1.0;
        self.precision = 0;
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
    pub fn range(mut self, min: f64, max: f64) -> Self {
        if min.is_finite() && max.is_finite() && max > min {
            self.min = min;
            self.max = max;
        }
        self
    }
    /// Positive keyboard and wheel step.
    pub fn step(mut self, step: f64) -> Self {
        if step.is_finite() && step > 0.0 {
            self.step = step;
        }
        self
    }
    /// The value Backspace and a double click reset to, clamped to the
    /// configured range on use. The default is 0.
    pub fn reset_to(mut self, value: f64) -> Self {
        if value.is_finite() {
            self.reset = value;
        }
        self
    }
    /// Display precision, limited to six places; the consumer value retains its precision.
    pub fn precision(mut self, precision: usize) -> Self {
        self.precision = precision.min(6);
        self
    }
    /// Visible unit suffix.
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = unit.into();
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
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Numeric {
    fn into_view(self) -> View {
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
        let editing = signal(false);
        let draft = signal(String::new());
        // The value the open text field started from, which a relative entry
        // such as `+5` is added to, and the text it opened with; `None` while
        // the field is closed.
        let text_start = var(None::<(f64, String)>);
        // The value a scrub's delta is measured from.
        let origin = var(0.0);
        let moved = signal(false);
        let scalar = use_scalar(
            state,
            ScalarRange::new(min, max).expect("invariant: numeric bounds are finite"),
        )
        .key_step(step)
        .expect("invariant: numeric step is finite and positive");
        let scalar = match edit_observer {
            Some(observer) => scalar.on_edit(move |phase| observer.call(phase)),
            None => scalar,
        };
        // A scrub is one edit of the scalar from its press: Escape, the
        // engine and the disposal of the field end it as a Cancel, with the
        // value it started from written back first.
        let pointer = scalar.scrub(use_drag::<()>(DragCoordinates::Local));
        let reference = node_ref();
        let rect = reference.rect();
        let logarithmic = move || logarithmic.get() && min > 0.0;
        let normalized = move |v: f64| {
            if kind == NumericKind::Fader {
                ((v - min) / (max - min)).clamp(0.0, 1.0).sqrt()
            } else if logarithmic() {
                ((v.max(min) / min).ln() / (max / min).ln()).clamp(0.0, 1.0)
            } else {
                ((v - min) / (max - min)).clamp(0.0, 1.0)
            }
        };
        let from_n = move |n: f64| {
            let n = n.clamp(0.0, 1.0);
            if kind == NumericKind::Fader {
                min + (max - min) * n.powi(2)
            } else if logarithmic() {
                min * (max / min).powf(n)
            } else {
                min + (max - min) * n
            }
        };
        // A write that is not part of a scrub is an edit of its own.
        let set = move |v: f64| {
            scalar.once(|| scalar.set(v));
        };
        let not_finite = message("creative.numeric.not_finite", "Enter a finite number");
        let mixed_text = message("creative.numeric.mixed", "Mixed");
        let text_parser = var(text_parser);
        let parse = move |s: &str| -> Result<f64, String> {
            let from = text_start.with(|start| start.as_ref().map(|start| start.0));
            let from = from.unwrap_or_else(|| value.get_untracked());
            match text_parser.get() {
                Some(parser) => parser(s, from).and_then(|v| {
                    if v.is_finite() {
                        Ok(v)
                    } else {
                        Err(not_finite.get())
                    }
                }),
                None => parse_scalar(s, timecode, from).ok_or_else(|| not_finite.get()),
            }
        };
        let begin_text = move || {
            let initial = format_scalar(value.get_untracked(), precision, "", timecode);
            text_start.set(Some((value.get_untracked(), initial.clone())));
            draft.set(initial);
            editing.set(true);
        };
        let discard_text = move || {
            text_start.set(None);
            editing.set(false);
        };
        // Write the typed text when the draft commits: Enter, or leaving the
        // field with a changed text. A text left as it opened writes nothing,
        // unless the value is mixed, which Enter confirms. A text that does
        // not parse is `invalid`, which the draft does not commit.
        let commit_text = move |typed: String| {
            let Some((_, opened)) = text_start.get() else {
                return;
            };
            if typed == opened && !mixed.get_untracked() {
                return;
            }
            if let Ok(next) = parse(&typed) {
                set(next);
            }
        };
        let suffix = unit.clone();
        let shown = (move || {
            if mixed.get() {
                mixed_text.get()
            } else {
                format_scalar(value.get().clamp(min, max), precision, &suffix, timecode)
            }
        })
        .into_value();
        let display = show(
            move || !editing.get(),
            move || {
                if matches!(kind, NumericKind::Knob | NumericKind::Fader) {
                    return ().into_view();
                }
                row((
                    text(label)
                        .hidden(move || !label_visible.get())
                        .style(style().single_line().grow(1).shrink(1))
                        .style(style().color(crate::Color::Muted)),
                    text(shown)
                        .style(move || {
                            let t = &current_theme();
                            style().font_family(t.get::<crate::tokens::Tokens>().mono_font.as_str())
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
        );
        let edit = show(editing, move || {
            let field = node_ref();
            let status = node_ref();
            // Why the text does not parse, while it does not; it follows the
            // draft as it is typed.
            let reason = memo(move || parse(&draft.get()).err());
            let shown = memo(move || reason.get().is_some_and(|reason| !reason.is_empty()));
            let hint = use_popup(binding(shown, |_: bool| {}))
                .anchor(reference)
                .focus_return(FocusReturn::preserve());
            on_mount(move || {
                field.focus();
                field.select_all();
            });
            (
                kivact::ui::text_input(label, draft)
                    // A draft that does not parse stays open after the focus
                    // leaves; Escape discards it.
                    .draft(commit_text)
                    .on_edit(move |phase| {
                        if phase != EditPhase::Begin {
                            discard_text();
                        }
                    })
                    .return_focus(reference)
                    .style(move || {
                        let t = &current_theme();
                        style().font_family(t.get::<crate::tokens::Tokens>().mono_font.as_str())
                    })
                    .node_ref(field)
                    .style(style().width(kivact::style::pct(100)))
                    .invalid(move || reason.get().is_some())
                    .described_by(status),
                show(hint.shown(), move || {
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
                                .color(crate::Color::X)
                                .background(crate::Color::Header)
                                .border(1, crate::Color::HairStrong),
                        ),
                    )
                    .part(hint.content().role(Role::Status))
                    .name(text_reason)
                    .node_ref(status)
                    .offset(2)
                }),
            )
        });
        let scrub = pointer
            .on_start(move |_| {
                origin.set(value.get());
                moved.set(false);
            })
            .on_update(move |update: DragUpdate<()>| {
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
                    let start = origin.get();
                    if kind == NumericKind::Number {
                        scalar.set(start + f64::from(delta) * step * factor);
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
                        scalar.set(from_n(
                            normalized(start) + f64::from(delta / extent) * factor,
                        ));
                    }
                    moved.set(true);
                }
            })
            .on_end(move |update: DragUpdate<()>| {
                if update.phase == DragPhase::Cancel {
                    moved.set(false);
                } else if !moved.get() && matches!(kind, NumericKind::Number | NumericKind::Slider)
                {
                    begin_text();
                }
            });
        let root = root
            .node_ref(reference)
            .indeterminate(mixed)
            .value_text(shown)
            .value_min(min)
            .value_max(max)
            .value_now(move || value.get().clamp(min, max))
            .orientation(move || {
                if matches!(kind, NumericKind::Knob | NumericKind::Fader) && !horizontal.get() {
                    Orientation::Vertical
                } else {
                    Orientation::Horizontal
                }
            })
            .draw(move |d, size| {
                let (horizontal, bipolar) = (horizontal.get(), bipolar.get());
                let p = d.theme().get::<crate::tokens::Tokens>();
                let tint = tint.get();
                let color = match tint {
                    PropertyState::Plain => accent.map_or(p.accent, |value| value.get()),
                    PropertyState::Keyed => p.key,
                    PropertyState::Animated => p.y,
                    PropertyState::Driven => kivact::style::rgb(0xc084fc),
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
                            Rect::new(0.0, 0.0, size.width * n as f32, size.height),
                            7.0,
                            color.with_alpha(0.3),
                        );
                        d.rounded_rect(
                            Rect::new(
                                size.width * n as f32 - 1.0,
                                4.0,
                                2.0,
                                (size.height - 8.0).max(0.0),
                            ),
                            1.0,
                            color,
                        );
                    }
                    NumericKind::Knob => {
                        draw_knob(d, size, n, color, &label.get(), &shown.get(), bipolar)
                    }
                    NumericKind::Fader if horizontal => {
                        let y = size.height / 2.0;
                        d.rounded_rect(
                            Rect::new(8.0, y - 2.0, (size.width - 16.0).max(0.0), 4.0),
                            2.0,
                            p.strong,
                        );
                        let x = 8.0 + (size.width - 16.0) * n as f32;
                        d.rounded_rect(
                            Rect::new(x - 7.0, y - 10.0, 14.0, 20.0),
                            5.0,
                            linear_gradient(180)
                                .colors([p.strong, Color::token(ColorToken::Raised)]),
                        );
                        d.line(Point::new(x, y - 5.0), Point::new(x, y + 5.0), 1.5, p.text);
                    }
                    NumericKind::Fader => draw_fader(d, size, n, color, &shown.get()),
                }
            })
            .on_key_default(move |e| {
                let adjusts = matches!(
                    e.key,
                    Key::ArrowRight
                        | Key::ArrowUp
                        | Key::ArrowLeft
                        | Key::ArrowDown
                        | Key::Home
                        | Key::End
                        | Key::Backspace
                );
                if editing.get() {
                    // The text field runs first. Consume the keys it leaves
                    // unhandled at the ends of its text, so that none of them
                    // writes the value.
                    return if adjusts { Handled::Yes } else { Handled::No };
                }
                let text_field = matches!(kind, NumericKind::Number | NumericKind::Slider);
                if e.key == Key::Enter && !text_field {
                    return Handled::No;
                }
                // A scrub owns the value until it ends, and a mixed value
                // takes only typed text, which Enter opens.
                if (adjusts || e.key == Key::Enter)
                    && (pointer.is_active() || (mixed.get() && adjusts))
                {
                    return Handled::Yes;
                }
                let factor = if e.modifiers.ctrl {
                    0.1
                } else if e.modifiers.shift {
                    10.0
                } else {
                    1.0
                };
                match e.key {
                    Key::Enter if e.repeat => {}
                    Key::Enter => begin_text(),
                    Key::ArrowRight | Key::ArrowUp => set(value.get() + step * factor),
                    Key::ArrowLeft | Key::ArrowDown => set(value.get() - step * factor),
                    Key::Home => set(min),
                    Key::End => set(max),
                    Key::Backspace => set(reset),
                    _ => return Handled::No,
                }
                Handled::Yes
            })
            .on_wheel_default(move |e| {
                if editing.get() || mixed.get() {
                    return Handled::No;
                }
                if pointer.is_active() {
                    return Handled::Yes;
                }
                let direction = if e.delta.y > 0.0 { -1.0 } else { 1.0 };
                set(value.get() + step * direction * if e.modifiers.ctrl { 0.1 } else { 1.0 });
                Handled::Yes
            })
            // Runs before the scrub source below; `Yes` keeps the press from it.
            .on_pointer_default(move |e| {
                if editing.get() {
                    return Handled::Yes;
                }
                if mixed.get() {
                    if matches!(
                        e.phase,
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
                    e.phase,
                    PointerPhase::Down(ButtonPress {
                        button: MouseButton::Primary,
                        clicks: 2..
                    })
                ) {
                    pointer.cancel();
                    set(reset);
                    return Handled::Yes;
                }
                Handled::No
            })
            .part(scrub.source(()))
            .child(display)
            .child(edit);
        // The accessibility actions are edits of their own. A value needs a
        // finite number, and a scrub or a mixed value takes no relative step.
        let act = move |event: A11yEvent| {
            let writes = event.region.is_none()
                && !pointer.is_active()
                && match &event.request {
                    A11yRequest::SetValue(text) => {
                        text.parse::<f64>().ok().is_some_and(f64::is_finite)
                    }
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
        };
        root.on_a11y_event(A11yAction::Increment, act)
            .on_a11y_event(A11yAction::Decrement, act)
            .on_a11y_event(A11yAction::SetValue, act)
            .value_text(shown)
            .into_view()
    }
}

fn draw_knob(
    d: &mut Draw,
    size: Size,
    n: f64,
    color: kivact::style::Color,
    label: &str,
    value: &str,
    bipolar: bool,
) {
    let p = d.theme().get::<crate::tokens::Tokens>();
    let scale = (size.width / 58.0).clamp(0.5, 1.0);
    let center = Point::new(size.width / 2.0, 24.0 * scale);
    let point = |a: f32, radius: f32| {
        Point::new(
            center.x + radius * scale * a.sin(),
            center.y - radius * scale * a.cos(),
        )
    };
    let a0 = -2.35;
    let arc = |start: f32, end: f32| {
        let mut path = Path::new().move_to(point(start, 19.0));
        for i in 1..=48 {
            path = path.line_to(point(start + (end - start) * i as f32 / 48.0, 19.0));
        }
        path
    };
    let stroke = Stroke::new(3.5 * scale)
        .cap(LineCap::Round)
        .join(LineJoin::Round);
    d.stroke_path(&arc(a0, 2.35), &stroke, p.hair_strong);
    d.stroke_path(
        &arc(if bipolar { 0.0 } else { a0 }, a0 + n as f32 * 4.7),
        &stroke,
        color,
    );
    d.rounded_rect(
        Rect::new(
            center.x - 15.0 * scale,
            9.0 * scale,
            30.0 * scale,
            30.0 * scale,
        ),
        15.0 * scale,
        linear_gradient(180).colors([p.strong, Color::token(ColorToken::Raised)]),
    );
    d.line(
        point(a0 + n as f32 * 4.7, 7.0),
        point(a0 + n as f32 * 4.7, 12.0),
        2.6,
        p.text_high,
    );
    let label_style = TextStyle {
        size: 10.5,
        color: p.dim,
        align: kivact::style::TextAlign::Center,
        ..TextStyle::default()
    };
    d.text(label, Point::new(center.x, 48.0 * scale + 7.0), label_style);
    d.text(
        value,
        Point::new(center.x, 48.0 * scale + 22.0),
        TextStyle {
            size: 10.5,
            color: p.text,
            family: Some(p.mono_font.clone()),
            align: kivact::style::TextAlign::Center,
            ..TextStyle::default()
        },
    );
}

fn draw_fader(d: &mut Draw, size: Size, n: f64, color: kivact::style::Color, value: &str) {
    let p = d.theme().get::<crate::tokens::Tokens>();
    let x = size.width / 2.0;
    let top = 8.0;
    let height = (size.height - 30.0).max(1.0);
    d.rounded_rect(Rect::new(x - 3.0, top, 6.0, height), 3.0, p.canvas);
    d.rounded_rect(
        Rect::new(
            x - 2.0,
            top + height * (1.0 - n as f32),
            4.0,
            height * n as f32,
        ),
        2.0,
        color.with_alpha(0.6),
    );
    for i in 0..=10 {
        let y = top + i as f32 * height / 10.0;
        d.line(
            Point::new(x - 17.0, y),
            Point::new(x - 10.0, y),
            1.0,
            p.hair_strong,
        );
    }
    let y = top + height * (1.0 - n as f32);
    d.rounded_rect(
        Rect::new(x - 11.0, y - 5.0, 22.0, 11.0),
        3.0,
        linear_gradient(180).colors([p.muted, p.strong]),
    );
    d.line(
        Point::new(x - 9.0, y),
        Point::new(x + 9.0, y),
        1.0,
        p.canvas,
    );
    d.text(
        value,
        Point::new(x, size.height - 8.0),
        TextStyle {
            size: 10.0,
            color: p.text,
            align: kivact::style::TextAlign::Center,
            family: Some(p.mono_font.clone()),
            ..TextStyle::default()
        },
    );
}

fn format_scalar(value: f64, precision: usize, unit: &str, fps: Option<u32>) -> String {
    if let Some(fps) = fps {
        let f = value.max(0.0).floor() as u64;
        let fps = u64::from(fps);
        format!(
            "{:02}:{:02}:{:02}:{:02}",
            f / (fps * 3600),
            f / (fps * 60) % 60,
            f / fps % 60,
            f % fps
        )
    } else {
        format!("{value:.*}{unit}", precision)
    }
}
fn parse_scalar(s: &str, fps: Option<u32>, current: f64) -> Option<f64> {
    let s = s.trim();
    if let Some(fps) = fps {
        if s.contains(':') {
            let parts = s
                .split(':')
                .map(str::parse::<u64>)
                .collect::<std::result::Result<Vec<_>, _>>()
                .ok()?;
            if !(2..=4).contains(&parts.len()) {
                return None;
            }
            let mut fields = [0_u64; 4];
            fields[4 - parts.len()..].copy_from_slice(&parts);
            if fields[1] >= 60 || fields[2] >= 60 || fields[3] >= u64::from(fps) {
                return None;
            }
            let result = ((fields[0] as f64 * 60.0 + fields[1] as f64) * 60.0 + fields[2] as f64)
                * f64::from(fps)
                + fields[3] as f64;
            return result.is_finite().then_some(result);
        }
        if s.starts_with(['+', '-']) {
            return evaluate_expression(s).map(|delta| current + delta);
        }
    }
    evaluate_expression(s)
}
/// Evaluate finite scalar arithmetic (+, -, *, /, parentheses and pi).
pub fn evaluate_expression(s: &str) -> Option<f64> {
    struct Parser<'a> {
        input: &'a [u8],
        at: usize,
    }
    impl Parser<'_> {
        fn spaces(&mut self) {
            while self.input.get(self.at).is_some_and(u8::is_ascii_whitespace) {
                self.at += 1;
            }
        }
        fn expression(&mut self) -> Option<f64> {
            let mut value = self.product()?;
            loop {
                self.spaces();
                match self.input.get(self.at) {
                    Some(b'+') => {
                        self.at += 1;
                        value += self.product()?;
                    }
                    Some(b'-') => {
                        self.at += 1;
                        value -= self.product()?;
                    }
                    _ => return Some(value),
                }
            }
        }
        fn product(&mut self) -> Option<f64> {
            let mut value = self.atom()?;
            loop {
                self.spaces();
                match self.input.get(self.at) {
                    Some(b'*') => {
                        self.at += 1;
                        value *= self.atom()?;
                    }
                    Some(b'/') => {
                        self.at += 1;
                        value /= self.atom()?;
                    }
                    _ => return Some(value),
                }
            }
        }
        fn atom(&mut self) -> Option<f64> {
            self.spaces();
            if self.input.get(self.at) == Some(&b'-') {
                self.at += 1;
                return Some(-self.atom()?);
            }
            if self.input.get(self.at) == Some(&b'+') {
                self.at += 1;
                return self.atom();
            }
            if self.input.get(self.at) == Some(&b'(') {
                self.at += 1;
                let v = self.expression()?;
                self.spaces();
                if self.input.get(self.at) != Some(&b')') {
                    return None;
                }
                self.at += 1;
                return Some(v);
            }
            if self.input.get(self.at..self.at + 2) == Some(b"pi") {
                self.at += 2;
                return Some(std::f64::consts::PI);
            }
            let start = self.at;
            while self
                .input
                .get(self.at)
                .is_some_and(|c| c.is_ascii_digit() || *c == b'.')
            {
                self.at += 1;
            }
            std::str::from_utf8(self.input.get(start..self.at)?)
                .ok()?
                .parse()
                .ok()
        }
    }
    let mut p = Parser {
        input: s.as_bytes(),
        at: 0,
    };
    let value = p.expression()?;
    p.spaces();
    (p.at == p.input.len() && value.is_finite()).then_some(value)
}
