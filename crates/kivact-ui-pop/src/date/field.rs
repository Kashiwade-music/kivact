//! Role: Parts both date inputs share: text field, strip, picker button, popup.

use super::text::DateText;
use crate::Tokens;
use crate::form::input_style;
use kivact::draw::Shadow;
use kivact::i18n::{CalendarDate, message};
use kivact::interaction::PopupApi;
use kivact::view::{Node, NodeBuilder, icon, node, row, text};

use kivact::state::{Binding, Value};
use kivact::style::use_theme;
use kivact::style::{Align, Color, ColorToken, style};
use kivact::ui::{ButtonVariant, button, text_input};
use kivact::view::{
    A11yAction, IntoView, LiveRegion, NodeRef, Overlay, Placement, Role, overlay, show,
};
use std::rc::Rc;

/// An error line that is there while `shown`, announced politely. `node` is
/// bound to it, for `described_by`.
pub(super) fn status(shown: Value<bool>, content: Value<String>, node: NodeRef) -> impl IntoView {
    show(shown, move || {
        text(content)
            .with_node(|root| root.role(Role::Status))
            .live_region(LiveRegion::Polite)
            .node_ref(node)
            .style(style().font_size(12).color(ColorToken::Danger))
    })
}

/// The text field of one date, shared by the single and the range input:
/// the text follows `text`, and a date it completes is written to `value`.
/// `bad` marks the field invalid.
pub(super) fn date_text_field(
    label: Value<String>,
    text: DateText,
    value: Binding<CalendarDate>,
    bad: Value<bool>,
    described_by: &[NodeRef],
) -> kivact::ui::TextInput {
    let DateText {
        draft,
        checked,
        node,
        ..
    } = text;
    let typed = draft.text();
    let edited = kivact::state::binding(typed, move |text: String| {
        typed.set(text.clone());
        checked.set(false);
        if let Some(date) = draft.parse(&text) {
            value.set(date);
        }
    });
    let mut field = text_input(label, edited)
        .node_ref(node)
        .invalid(bad)
        .on_submit(move || checked.set(true))
        .style(
            style()
                .grow(1)
                .height(34)
                .min_width(80)
                .padding_x(0)
                .border(0, Color::TRANSPARENT)
                .shadow(Shadow::new(0, 0, 0, Color::TRANSPARENT)),
        );
    for node in described_by {
        field = field.with_node(|root| root.described_by(*node));
    }
    field
}

/// The bordered strip around the fields, the calendar icon and the button.
/// It is the anchor of the calendar and turns red while `bad`.
pub(super) fn field_strip(anchor: NodeRef, bad: Value<bool>, children: impl IntoView) -> Node {
    row(children).node_ref(anchor).style(move || {
        let theme = &use_theme().get();
        let strip = input_style(theme)
            .height(36)
            .padding_left(12)
            .padding_right(4)
            .gap(8)
            .align_items(Align::Center);
        if bad.get() {
            strip.border_color(theme.ext::<Tokens>().red)
        } else {
            strip
        }
    })
}

/// The button that opens the calendar; `on_open` runs before it opens.
pub(super) fn picker_button(
    popup: PopupApi,
    on_open: impl Into<kivact::state::Callback<()>>,
) -> impl kivact::view::Component {
    let on_open = on_open.into();
    let prepare = Rc::new(move || {
        if !popup.is_open().get_untracked() {
            on_open.call(());
        }
    });
    button(message("pop.date.open", "Choose a date"))
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .icon(icon("calendar"))
        .with_node(|node| node.part(popup.trigger()))
        .on_press({
            let prepare = prepare.clone();
            move || prepare()
        })
        .with_node(|node| node.a11y_action(A11yAction::Expand, move |_| prepare()))
        .style(style().size(28, 28))
}

/// `content` in a panel below `anchor`, closed by Escape or a press outside.
pub(super) fn calendar_popup(
    anchor: NodeRef,
    popup: PopupApi,
    name: Value<String>,
    content: impl IntoView,
) -> Overlay {
    overlay(
        anchor,
        Placement::Below,
        node()
            .child(content)
            .style(
                style()
                    .background(crate::Color::Surface)
                    .border(1, ColorToken::Border)
                    .radius(10)
                    .shadow(Shadow::new(0, 10, 15, ColorToken::Shadow)),
            )
            .style(style().width(320).padding(16)),
    )
    .offset(4)
    .with_node(|root| root.part(popup.content()))
    .with_node(|node| node.role(Role::Dialog).name(name))
}
