//! Role: Single and multiple option selection and custom selector popovers.
use crate::recipes_field::frame;
use crate::theme_access::tokens;
use crate::typeahead::Typeahead;
use crate::{DialogPurpose, SheetHeight, bottom_sheet, heading};
use kivact::control::{Node, NodeBuilder, column, icon, scroll, text};
use kivact::interaction::{ListItemState, ListSelection};
use kivact::primitive::{NodeExt, PartState};
use kivact::style::{Align, Direction, Length, pct, style};
use kivact::ui::{
    ButtonVariant, ControlSize, DialogKind, ListItem, Select, Variant, button, dialog, select,
};
use kivact::view::{HandlerResult, IntoText, IntoValue, IntoView, PopupKind, View};
use kivact::{Binding, Value, signal};
use std::hash::Hash;

/// How touch activation presents a Selector's list. Keyboard and mouse keep
/// the anchored popup on every setting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectorPresentation {
    #[default]
    Popover,
    BottomSheet,
    Adaptive,
}

pub struct Selector<K: Clone + Eq + Hash + 'static> {
    inner: Select<K>,
    presentation: SelectorPresentation,
}

impl<K: Clone + Eq + Hash + 'static> Selector<K> {
    pub fn presentation(mut self, presentation: SelectorPresentation) -> Self {
        self.presentation = presentation;
        self
    }

    pub fn variant(mut self, variant: impl Variant) -> Self {
        self.inner = self.inner.variant(variant);
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.inner = self.inner.size(size);
        self
    }

    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.inner = self.inner.placeholder(text);
        self
    }

    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.inner = self.inner.icon(icon);
        self
    }

    pub fn open_on_arrow(mut self, enabled: bool) -> Self {
        self.inner = self.inner.open_on_arrow(enabled);
        self
    }

    pub fn retain_trigger_focus(mut self, enabled: bool) -> Self {
        self.inner = self.inner.retain_trigger_focus(enabled);
        self
    }

    pub fn on_preview<R: HandlerResult>(
        mut self,
        preview: impl Fn(Option<K>) -> R + 'static,
    ) -> Self {
        self.inner = self.inner.on_preview(preview);
        self
    }

    pub fn option<V: IntoView>(
        mut self,
        render: impl Fn(Value<ListItem<K>>, Value<ListItemState>) -> V + 'static,
    ) -> Self {
        self.inner = self.inner.option(render);
        self
    }
}

impl<K: Clone + Eq + Hash + 'static> NodeBuilder for Selector<K> {
    fn node_mut(&mut self) -> &mut Node {
        self.inner.node_mut()
    }
}

impl<K: Clone + Eq + Hash + 'static> IntoView for Selector<K> {
    fn into_view(self) -> View {
        let presentation = self.presentation;
        if presentation == SelectorPresentation::Popover {
            return self.inner.into_view();
        }
        self.inner
            .touch_sheet(
                move || {
                    let viewport = kivact::platform::window_size();
                    presentation == SelectorPresentation::BottomSheet
                        || viewport.width < 768.0
                        || viewport.height < 500.0
                },
                |label, open, trigger, list| {
                    bottom_sheet(label, open)
                        .purpose(DialogPurpose::Info)
                        .height(SheetHeight::Hug)
                        .padding(0)
                        .final_focus(trigger)
                        .content(move || {
                            column((
                                heading(3, label).style(
                                    style()
                                        .font_size(19)
                                        .line_height(28.0 / 19.0)
                                        .weight(700)
                                        .margin_x(12)
                                        .margin_bottom(8),
                                ),
                                list(),
                            ))
                            .style(style().width(pct(100)).padding_top(16).padding_x(4))
                        })
                },
            )
            .into_view()
    }
}
pub type MultiSelector<K> = Typeahead<K>;

pub fn selector<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl Into<Binding<Option<K>>>,
) -> Selector<K> {
    Selector {
        inner: select(label, options, value)
            .open_on_arrow(true)
            .retain_trigger_focus(true)
            .has_popup(|| Some(PopupKind::ListBox)),
        presentation: SelectorPresentation::Popover,
    }
}

pub fn multi_selector<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl Into<Binding<Vec<K>>>,
) -> MultiSelector<K> {
    let value = value.into();
    let mut field = Typeahead::new(label, options, ListSelection::Multiple(value));
    field.chips = Some(value);
    field
}

pub fn complex_selector<V: IntoView>(
    label: impl IntoText,
    value_label: impl IntoText,
    content: impl Fn() -> V + 'static,
) -> View {
    let label = label.into_text_value();
    let value_label = value_label.into_text_value();
    let open = signal(false);
    let trigger = button(label)
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .child(text(value_label))
        .trailing_icon(icon("chevron-down"))
        .expanded(open)
        .on_press(move || open.set(!open.get_untracked()))
        .style(move || {
            let t = tokens();
            frame(&t, &PartState::default(), false)
                .height(32)
                .padding_x(12)
                .direction(Direction::Row)
                .align_items(Align::Center)
                .gap(8)
        });
    dialog(label, open)
        .kind(DialogKind::Popover)
        .hide_title(true)
        .width(Length::Auto)
        .content(move || scroll(content()).style(style().max_height(480).padding(12)))
        .attach(trigger)
}
