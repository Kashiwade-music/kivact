//! Role: Single and multiple option selection and custom selector popovers.
use crate::recipes_field::frame;
use crate::selector_surface::{SelectorPresentation, sheet, touch_tap};
use crate::theme_access::tokens;
use crate::typeahead::Typeahead;
use kivact::custom::PartState;
use kivact::interaction::DialogMode;
use kivact::interaction::ListItem;
use kivact::interaction::ListItemState;
use kivact::interaction::Selection;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::ItemKey;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::memo;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Length;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::Select;
use kivact::ui::Variant;
use kivact::ui::button;
use kivact::ui::dialog;
use kivact::ui::select;
use kivact::view::Handled;
use kivact::view::HandlerResult;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::PointerPhase;
use kivact::view::PopupKind;
use kivact::view::View;
use kivact::view::icon;
use kivact::view::node_ref;
use kivact::view::scroll;
use kivact::view::text;

pub struct Selector<K: ItemKey> {
    inner: Select<K>,
    presentation: SelectorPresentation,
}

impl<K: ItemKey> Selector<K> {
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

    pub fn open_on_arrow(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.inner = self.inner.open_on_arrow(enabled);
        self
    }

    pub fn retain_trigger_focus(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.inner = self.inner.keep_trigger_focus(enabled);
        self
    }

    pub fn on_preview<R: HandlerResult>(
        mut self,
        preview: impl Fn(Option<K>) -> R + 'static,
    ) -> Self {
        self.inner = self.inner.on_active_change(preview);
        self
    }

    pub fn option<V: IntoView>(
        mut self,
        render: impl Fn(Value<ListItem<K>>, Value<ListItemState>) -> V + 'static,
    ) -> Self {
        self.inner = self.inner.item(render);
        self
    }
}

impl<K: ItemKey> NodeBuilder for Selector<K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        self.inner.node_mut()
    }
}

impl<K: ItemKey> IntoView for Selector<K> {
    fn into_view(self) -> View {
        let presentation = self.presentation;
        if presentation == SelectorPresentation::Popover {
            return self.inner.into_view();
        }
        self.inner
            .touch_sheet(
                move || presentation.uses_sheet(),
                |label, open, trigger, list| sheet(label, open, trigger, move || list.call(())),
            )
            .into_view()
    }
}
pub type MultiSelector<K> = Typeahead<K>;

pub fn selector<K: ItemKey>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl IntoBinding<Option<K>>,
) -> Selector<K> {
    Selector {
        inner: select(label, options, value)
            .open_on_arrow(true)
            .keep_trigger_focus(true)
            .has_popup(|| Some(PopupKind::ListBox)),
        presentation: SelectorPresentation::Popover,
    }
}

pub fn multi_selector<K: ItemKey>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl IntoBinding<Vec<K>>,
) -> MultiSelector<K> {
    let value = value.into_binding();
    let mut field = Typeahead::new(label, options, Selection::Multiple(value));
    field.chips = Some(value);
    field
}

pub fn complex_selector<V: IntoView>(
    label: impl IntoText,
    value_label: impl IntoText,
    content: impl kivact::view::IntoViewFn<V>,
) -> View {
    complex_selector_with_presentation(label, value_label, SelectorPresentation::Popover, content)
}

/// Use the same caller-owned content, including any search field, in an
/// anchored popup or a compact touch sheet.
pub fn complex_selector_with_presentation<V: IntoView>(
    label: impl IntoText,
    value_label: impl IntoText,
    presentation: SelectorPresentation,
    content: impl kivact::view::IntoViewFn<V>,
) -> View {
    let label = label.into_text_value();
    let value_label = value_label.into_text_value();
    let open = signal(false);
    let sheet_open = signal(false);
    let target = node_ref();
    let released = signal(false);
    let touch = touch_tap(target);
    let trigger = button(label)
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .child(text(value_label))
        .trailing(icon("chevron-down"))
        .with_node(move |node| node.expanded(open))
        .node_ref(target)
        .on_pointer(move |event| {
            if matches!(event.phase(), PointerPhase::Down(_)) {
                released.set(false);
            }
            if touch(event, !open.get_untracked() && presentation.uses_sheet()) {
                released.set(true);
            }
            Handled::No
        })
        .on_key(move |_| {
            released.set(false);
            Handled::No
        })
        .style(move || {
            let t = tokens();
            frame(&t, &PartState::default(), false)
                .height(32)
                .padding_x(12)
                .direction(Direction::Row)
                .align_items(Align::Center)
                .gap(8)
        });
    let content: kivact::view::ViewFn = content.into_view_fn();
    let popup_body = content.clone();
    let popup_open = binding(
        memo(move || open.get() && !sheet_open.get()),
        move |shown| {
            let sheet = shown && released.get_untracked();
            released.set(false);
            sheet_open.set(sheet);
            open.set(shown);
        },
    );
    let popup = dialog(label, popup_open)
        .mode(DialogMode::Popover)
        .hide_title(true)
        .width(Length::Auto)
        .content(move || scroll(popup_body.call(())).style(style().max_height(480).padding(12)))
        .attach(trigger);
    let sheet = (presentation != SelectorPresentation::Popover).then(|| {
        let shown = binding(memo(move || open && sheet_open), move |shown| {
            if !shown {
                open.set(false);
                sheet_open.set(false);
            }
        });
        sheet(label, shown, target, move || {
            scroll(content.call(())).style(style().max_height(480).padding(12))
        })
    });
    (popup, sheet).into_view()
}
