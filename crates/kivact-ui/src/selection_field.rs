//! Role: Present caller-owned chooser content through the standard popup and touch drawer.
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::variants::Variants;
use kivact_core::state::{
    Binding, IntoBinding, IntoText, IntoValue, Value, binding, machine, signal,
};
use kivact_core::style::{Length, style};
use kivact_core::view::{
    Handled, IntoView, Node, NodeBuilder, View, ViewFn, icon, node, node_ref, scroll, text,
};
use kivact_interaction::{DialogMode, PopupState};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Surface {
    Popup,
    Sheet,
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
    pub fn variant(self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for SelectionField {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for SelectionField {
    fn into_view(mut self) -> View {
        let open = self.open.unwrap_or_else(|| signal(false).into_binding());
        let surface = signal(Surface::Popup);
        let target = node_ref();
        let bounds = target.rect();
        let disabled = self
            .root
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        let gesture = sheet_gesture(bounds, disabled, open);
        let presentation = self.presentation;
        let part = kivact_core::custom::part(|input| input);
        let input = part.state();
        let trigger = with_base_recipe_style(
            self.root.part(part),
            |r| r.select_recipes().trigger_recipe(),
            &self.variants,
            move || crate::SelectTriggerState {
                input: input.get(),
                open: open.get(),
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
            gesture.send(crate::select::machine::Event::Pointer(event));
            Handled::No
        })
        .on_key(move |_| {
            gesture.send(crate::select::machine::Event::Reset);
            Handled::No
        });
        let popup_open = binding(
            move || open.get() && surface.get() == Surface::Popup,
            move |shown| {
                if shown {
                    let sheet = presentation.get().uses_sheet()
                        && matches!(
                            gesture.get(),
                            crate::select::machine::SheetGesture::Released(
                                crate::select::machine::Outcome::Tap
                            )
                        );
                    surface.set(if sheet {
                        Surface::Sheet
                    } else {
                        Surface::Popup
                    });
                    gesture.send(crate::select::machine::Event::Reset);
                }
                open.set(shown);
            },
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
                        open: open.get(),
                    },
                )
            })
            .attach(trigger);
        let sheet_open = binding(
            move || open.get() && surface.get() == Surface::Sheet,
            move |shown| open.set(shown),
        );
        let sheet = crate::dialog(self.label, sheet_open)
            .mode(DialogMode::Drawer(kivact_core::geometry::Edge::Bottom))
            .content(move || content.call(()));
        (popup, sheet).into_view()
    }
}

/// Recognize touch taps from current chooser availability and measured bounds.
fn sheet_gesture(
    bounds: Value<Option<kivact_core::geometry::Rect>>,
    disabled: Value<bool>,
    open: Binding<bool>,
) -> kivact_core::state::MachineHandle<crate::select::machine::Machine> {
    machine(
        crate::select::machine::SheetGesture::Idle,
        move || crate::select::machine::Machine {
            readiness: if !disabled.get() && !open.get() {
                crate::select::machine::Readiness::Ready
            } else {
                crate::select::machine::Readiness::Unavailable
            },
            bounds: bounds.get(),
            slop: kivact_core::style::use_theme()
                .get()
                .ext::<kivact_interaction::InteractionPolicy>()
                .select
                .sheet_slop,
        },
        |_, effect| match effect {},
    )
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
