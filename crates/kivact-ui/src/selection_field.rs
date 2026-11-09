//! Role: Present caller-owned chooser content in a popup or touch drawer.
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::variants::Variants;
use kivact_core::geometry::Rect;
use kivact_core::state::{
    Binding, IntoBinding, IntoText, IntoValue, Machine as MachineTrait, Step, Value, binding,
    machine, signal,
};
use kivact_core::style::{Length, style, use_theme};
use kivact_core::view::{
    Element, Handled, IntoView, Node, PointerEvent, View, ViewFn, icon, node, node_ref, scroll,
    text,
};
use kivact_interaction::{DialogMode, InteractionPolicy, PopupState};

use crate::select::machine::{Outcome, Readiness, SheetGesture, gesture};

/// The surface the current opening shows.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Surface {
    Popup,
    Sheet,
}

/// The chooser's open state: closed with the gesture it last saw, or open on
/// the surface that opening chose.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum FieldState {
    Closed { gesture: SheetGesture },
    Open { surface: Surface },
}

pub(crate) enum FieldEvent {
    /// The caller's binding became this value.
    OpenRequested(bool),
    /// A pointer event on the trigger, while closed.
    Pointer(PointerEvent),
    /// Discard the last completed tap.
    Reset,
}

pub(crate) enum FieldEffect {
    /// Make the caller's binding this value.
    WriteOpen(bool),
}

/// The construction constants of one chooser's machine.
pub(crate) struct FieldMachine {
    presentation: crate::SelectPresentation,
    // xtask-allow(state-fields): independent: the trigger's measured bounds, separately from its availability
    bounds: Option<Rect>,
    // xtask-allow(state-fields): independent: the caller's availability, separately from the measured bounds
    disabled: bool,
    slop: f64,
}

impl MachineTrait for FieldMachine {
    type State = FieldState;
    type Event = FieldEvent;
    type Effect = FieldEffect;
    fn transition(
        &self,
        state: &FieldState,
        event: FieldEvent,
    ) -> Step<FieldState, FieldEvent, FieldEffect> {
        match (*state, event) {
            (FieldState::Open { .. }, FieldEvent::OpenRequested(true)) => Step::stay(),
            (FieldState::Closed { gesture }, FieldEvent::OpenRequested(true)) => {
                let tap = matches!(gesture, SheetGesture::Released(Outcome::Tap));
                let surface = if tap && self.presentation.uses_sheet() {
                    Surface::Sheet
                } else {
                    Surface::Popup
                };
                Step::to(FieldState::Open { surface }).with(FieldEffect::WriteOpen(true))
            }
            (FieldState::Closed { .. }, FieldEvent::OpenRequested(false)) => Step::stay(),
            (FieldState::Open { .. }, FieldEvent::OpenRequested(false)) => {
                Step::to(FieldState::Closed {
                    gesture: SheetGesture::Idle,
                })
                .with(FieldEffect::WriteOpen(false))
            }
            (FieldState::Open { .. }, FieldEvent::Pointer(_)) => Step::stay(),
            (FieldState::Closed { gesture: current }, FieldEvent::Pointer(event)) => {
                let readiness = if self.disabled {
                    Readiness::Unavailable
                } else {
                    Readiness::Ready
                };
                match gesture(current, readiness, self.bounds, self.slop, event) {
                    Some(next) => Step::to(FieldState::Closed { gesture: next }),
                    None => Step::stay(),
                }
            }
            (FieldState::Open { .. }, FieldEvent::Reset) => Step::stay(),
            (FieldState::Closed { gesture }, FieldEvent::Reset) => {
                if matches!(gesture, SheetGesture::Released(_)) {
                    Step::to(FieldState::Closed {
                        gesture: SheetGesture::Idle,
                    })
                } else {
                    Step::stay()
                }
            }
        }
    }
}

/// A custom chooser whose content can contain a search editor or arbitrary controls.
/// The caller owns the chosen value; this builder only owns the presentation.
pub struct SelectionField {
    root: Node,
    label: Value<String>,
    value_label: Value<String>,
    content: ViewFn,
    presentation: Value<crate::SelectPresentation>,
    open: Option<Binding<bool>>,
    variants: Variants,
}
/// Create a chooser trigger with an accessible name and a displayed value.
pub fn selection_field(label: impl IntoText, value_label: impl IntoText) -> SelectionField {
    SelectionField {
        root: node(),
        label: label.into_text_value(),
        value_label: value_label.into_text_value(),
        content: ViewFn::from(|()| ()),
        presentation: crate::SelectPresentation::Popover.into_value(),
        open: None,
        variants: Variants::default(),
    }
}
impl SelectionField {
    /// Build content in a fresh owner each time either surface opens.
    pub fn content<V: IntoView>(mut self, content: impl Fn() -> V + 'static) -> Self {
        self.content = ViewFn::from(move |()| content());
        self
    }
    /// Choose touch presentation; mouse and keyboard always open the popup.
    pub fn presentation(mut self, presentation: impl IntoValue<crate::SelectPresentation>) -> Self {
        self.presentation = presentation.into_value();
        self
    }
    /// Use the application's binding as the sole visibility authority.
    pub fn open(mut self, open: impl IntoBinding<bool>) -> Self {
        self.open = Some(open.into_binding());
        self
    }
    /// Supply a theme-specific visual treatment.
    pub fn variant(mut self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl Element for SelectionField {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
/// Own the chooser's open state, its chosen surface and its tap gesture.
fn controller(
    root: &mut Node,
    target: kivact_core::view::NodeRef,
    open: Binding<bool>,
    presentation: Value<crate::SelectPresentation>,
) -> kivact_core::state::MachineHandle<FieldMachine> {
    let bounds = target.rect();
    let disabled = root
        .accessible_disabled()
        .unwrap_or_else(|| false.into_value());
    let initial = if open.get_untracked() {
        FieldState::Open {
            surface: Surface::Popup,
        }
    } else {
        FieldState::Closed {
            gesture: SheetGesture::Idle,
        }
    };
    let slop = use_theme()
        .get()
        .ext::<InteractionPolicy>()
        .select
        .sheet_slop;
    let field = machine(
        initial,
        move || FieldMachine {
            presentation: presentation.get(),
            bounds: bounds.get(),
            disabled: disabled.get(),
            slop,
        },
        move |_, effect| match effect {
            FieldEffect::WriteOpen(write) => {
                if open.get_untracked() != write {
                    open.set(write);
                }
            }
        },
    );
    field.feed(open.into_value(), FieldEvent::OpenRequested);
    field
}

impl IntoView for SelectionField {
    fn into_view(mut self) -> View {
        let open = self.open.unwrap_or_else(|| signal(false).into_binding());
        let target = node_ref();
        let field = controller(&mut self.root, target, open, self.presentation);
        let part = kivact_core::custom::part(|input| input);
        let input = part.state();
        let trigger = with_base_recipe_style(
            self.root.part(part),
            |r| r.select_recipes().trigger_recipe(),
            &self.variants,
            move || crate::SelectTriggerState {
                input: input.get(),
                open: matches!(field.get(), FieldState::Open { .. }),
                placeholder: false,
                size: crate::ControlSize::Medium,
            },
        )
        .name(self.label)
        .node_ref(target)
        .expanded(open)
        .child(text(self.value_label).style(style().grow(1).single_line()))
        .child(indicator(open, &self.variants))
        .on_pointer(move |event| {
            field.send(FieldEvent::Pointer(event));
            Handled::No
        })
        .on_key(move |_| {
            field.send(FieldEvent::Reset);
            Handled::No
        });
        let popup_open = binding(
            move || {
                matches!(
                    field.get(),
                    FieldState::Open {
                        surface: Surface::Popup
                    }
                )
            },
            move |shown| field.send(FieldEvent::OpenRequested(shown)),
        );
        let content = self.content;
        let popup_body = content.clone();
        let variants = self.variants;
        let popup = crate::dialog(self.label, popup_open)
            .mode(DialogMode::Popover)
            .hide_title(true)
            .width(Length::Auto)
            .content(move || {
                with_recipe_style(
                    scroll(popup_body.call(())),
                    |r| r.select_recipes().surface_recipe(),
                    &variants,
                    move || PopupState {
                        input: Default::default(),
                        open: matches!(field.get(), FieldState::Open { .. }),
                    },
                )
            })
            .attach(trigger);
        let sheet_open = binding(
            move || {
                matches!(
                    field.get(),
                    FieldState::Open {
                        surface: Surface::Sheet
                    }
                )
            },
            move |shown| field.send(FieldEvent::OpenRequested(shown)),
        );
        let sheet = crate::dialog(self.label, sheet_open)
            .mode(DialogMode::Drawer(kivact_core::geometry::Edge::Bottom))
            .content(move || content.call(()));
        (popup, sheet).into_view()
    }
}

/// Draw the chooser indicator using the shared popup recipe state.
fn indicator(open: Binding<bool>, variants: &Variants) -> View {
    with_recipe_style(
        node(),
        |r| r.select_recipes().indicator_recipe(),
        variants,
        move || PopupState {
            input: Default::default(),
            open: open.get(),
        },
    )
    .child(icon("chevron-down"))
    .into_view()
}
