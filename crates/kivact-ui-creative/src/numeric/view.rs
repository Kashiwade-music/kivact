//! Role: Mount a numeric control, paint it and open its text field.

use kivact::draw::Draw;
use kivact::draw::linear_gradient;
use kivact::geometry::{Point, Rect, Size};
use kivact::interaction::{
    DragApi, DragCoordinates, FocusReturn, InlineEditApi, ScalarApi, ScalarRange, ScalarStep,
    ScrubPolicy, use_drag, use_popup, use_scalar,
};
use kivact::state::{IntoValue, MachineHandle, Value, Var, binding, machine, memo, signal, var};
use kivact::style::{Align, Color, ColorToken, Justify, Theme, pct, style, use_theme};
use kivact::view::{
    A11yAction, Element, IntoView, Node, NodeRef, Orientation, Placement, Role, View, node,
    node_ref, overlay, row, show, text,
};
use std::rc::Rc;

use super::builder::{Numeric, NumericKind, PropertyState};
use super::machine;
use super::paint::{draw_fader, draw_knob};
use super::text::{
    Formatter, TextParser, builtin_formatter, display_value, editor_formatter, scalar_parser,
};

/// The state a numeric control hands to its scrub, key and paint handlers.
pub(super) struct NumericView {
    pub(super) label: Value<String>,
    pub(super) kind: NumericKind,
    pub(super) range: Value<ScalarRange>,
    pub(super) step: Value<ScalarStep>,
    pub(super) reset: Value<f64>,
    pub(super) tint: Value<PropertyState>,
    pub(super) label_visible: Value<bool>,
    // xtask-allow(state-fields): option: absent means the theme's accent
    pub(super) accent: Option<Value<Color>>,
    pub(super) bipolar: Value<bool>,
    pub(super) horizontal: Value<bool>,
    pub(super) mixed: Value<bool>,
    pub(super) value: Value<f64>,
    pub(super) typing: InlineEditApi<()>,
    pub(super) mode: MachineHandle<machine::Machine>,
    pub(super) scalar: ScalarApi,
    pub(super) pointer: DragApi<f64>,
    pub(super) reference: NodeRef,
    pub(super) rect: Value<Option<Rect>>,
    pub(super) parse: TextParser,
    pub(super) shown: Value<String>,
    pub(super) edit_text: Formatter,
    pub(super) mapping: kivact::interaction::ScalarMapping,
    // xtask-allow(state-fields): option: absent means the kind's default scrub distance
    pub(super) drag_distance: Option<Value<f64>>,
    // xtask-allow(state-fields): option: the builder always sets this flag; false disables text entry
    pub(super) text_entry: bool,
    pub(super) theme: Value<Theme>,
    pub(super) wheel_rest: Var<f64>,
}

impl IntoView for Numeric {
    fn into_view(self) -> View {
        let (root, view) = self.prepare();
        let view = Rc::new(view);
        let display = view.display();
        let editor = view.editor();
        let scrub = NumericView::scrub(&view);
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
            range,
            step,
            reset,
            precision,
            unit,
            tint,
            mapping,
            format,
            drag_distance,
            text_entry,
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
        let (mode, typing) = mode_and_typing();
        let scalar = use_scalar(state, range)
            .mapping(mapping.clone())
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
        let shown_format = format
            .clone()
            .unwrap_or_else(|| builtin_formatter(precision, unit, timecode));
        let edit_text = editor_formatter(format, precision, timecode);
        let shown = display_value(mixed, value, range, shown_format);
        let view = NumericView {
            label,
            kind,
            range,
            step,
            reset,
            tint,
            label_visible,
            accent,
            bipolar,
            horizontal,
            mixed,
            value,
            typing,
            mode,
            scalar,
            pointer,
            reference,
            rect,
            parse,
            shown,
            edit_text,
            mapping,
            drag_distance,
            text_entry,
            theme: use_theme(),
            wheel_rest: var(0.0),
        };
        (root, view)
    }
}

/// Typing text and scrubbing the value are one mode; the text field follows
/// the mode's `Typing` state.
fn mode_and_typing() -> (MachineHandle<machine::Machine>, InlineEditApi<()>) {
    let mode = machine(machine::State::Idle, || machine::Machine, |_, ()| {});
    let typing = kivact::interaction::use_inline_edit(binding(
        move || matches!(mode.get(), machine::State::Typing).then_some(()),
        move |key: Option<()>| {
            mode.send(if key.is_some() {
                machine::Event::BeginText
            } else {
                machine::Event::EndText
            });
        },
    ));
    (mode, typing)
}

impl NumericView {
    /// The theme's scrub policy, read from the value captured at setup.
    pub(super) fn scrub_policy(&self) -> ScrubPolicy {
        self.theme
            .get_untracked()
            .ext::<kivact::interaction::InteractionPolicy>()
            .scrub
            .clone()
    }

    fn display(&self) -> impl IntoView {
        let label = self.label;
        let kind = self.kind;
        let label_visible = self.label_visible;
        let mode = self.mode;
        let shown = self.shown;
        let text_open = move || matches!(mode.get(), machine::State::Typing);
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
        let mode = self.mode;
        let text_open = move || matches!(mode.get(), machine::State::Typing);
        let label = self.label;
        let mixed = self.mixed;
        let range = self.range;
        let value = self.value;
        let typing = self.typing;
        let scalar = self.scalar;
        let reference = self.reference;
        let parse = self.parse.clone();
        let edit_text = self.edit_text.clone();
        let kind = self.kind;
        let overlay_entry = matches!(kind, NumericKind::Knob | NumericKind::Fader);
        let set = move |v: f64| {
            scalar.once(|| scalar.set(v));
        };
        show(text_open, move || {
            let parse_commit = Rc::clone(&parse);
            let parse_reason = Rc::clone(&parse);
            let status = node_ref();
            let from = value.get_untracked();
            let bounds = range.get_untracked();
            let opened = edit_text(from.clamp(bounds.min(), bounds.max()));
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
            let field = node()
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
                .invalid(move || reason.get().is_some())
                .described_by(status);
            // A knob or fader shows the field over itself; a number or
            // value_slider keeps it inline in the row.
            let field = if overlay_entry {
                overlay(reference, Placement::Center, field)
                    .with_node(move |node| node.style(style().min_width(72)))
                    .into_view()
            } else {
                field.style(style().width(pct(100))).into_view()
            };
            (field, editor_hint(hint, reason, reference, status))
        })
    }

    fn metadata(&self, root: Node) -> Node {
        let kind = self.kind;
        let range = self.range;
        let horizontal = self.horizontal;
        let mixed = self.mixed;
        let value = self.value;
        let reference = self.reference;
        let shown = self.shown;
        root.node_ref(reference)
            .indeterminate(mixed)
            .value_text(shown)
            .value_min(move || range.get().min())
            .value_max(move || range.get().max())
            .value_now(move || {
                let range = range.get();
                value.get().clamp(range.min(), range.max())
            })
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
        let mapping = self.mapping.clone();

        let (horizontal, bipolar) = (horizontal.get(), bipolar.get());
        let p = d.theme().ext::<crate::tokens::Tokens>();
        let tint = tint.get();
        let color = match tint {
            PropertyState::Plain => accent.map_or(p.accent, |value| value.get()),
            PropertyState::Keyed => p.key,
            PropertyState::Animated => p.y,
            PropertyState::Driven => crate::color::driven_property_color(),
        };
        let n = mapping.position(value.get(), self.range.get());
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
}

/// The validation message the field is described by, anchored below the
/// control while the draft does not parse.
fn editor_hint(
    hint: kivact::interaction::PopupApi,
    reason: Value<Option<String>>,
    reference: NodeRef,
    status: NodeRef,
) -> impl IntoView {
    show(hint.is_open(), move || {
        let shown = move || reason.get().unwrap_or_default();
        let named = move || reason.get().unwrap_or_default();
        overlay(
            reference,
            Placement::Below,
            text(shown).style(
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
        .with_node(move |node| node.name(named))
        .with_node(move |node| node.node_ref(status))
        .offset(2)
    })
}
