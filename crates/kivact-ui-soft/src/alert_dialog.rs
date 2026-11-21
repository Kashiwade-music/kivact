//! Role: Compose alert confirmation with the standard modal behavior.
use crate::media_query::Breakpoint;
use kivact::component;
use kivact::state::Callback;
use kivact::state::Signal;
use kivact::state::Value;
use kivact::style::Justify;
use kivact::style::Length;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::DialogKind;
use kivact::ui::button;
use kivact::ui::dialog;
use kivact::view::Element;
use kivact::view::column;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::show;

/// Confirmation dialog with a cancel button and an optional application action.
///
/// The dialog is 400 pixels wide unless `width` is set, and has no close
/// affordance. `on_press` runs when the action button is pressed; the
/// callback decides whether and when to close the dialog.
#[component]
pub fn alert_dialog(
    title: Value<String>,
    description: Value<String>,
    open: Signal<bool>,
    action_label: Option<String>,
    on_press: Option<Callback<()>>,
    cancel_label: Option<String>,
    #[prop(default = ButtonVariant::Danger)] action_variant: ButtonVariant,
    #[prop(default = false)] action_loading: Value<bool>,
    width: Option<Value<Length>>,
) -> impl Element {
    let cancel_ref = node_ref();
    let mut modal = dialog(title, open)
        .mode(DialogKind::AlertModal)
        .description(description)
        .width(Length::from(400))
        .closable(false);
    if let Some(width) = width {
        modal = modal.width(width);
    }
    let footer = move || {
        let actions = {
            let cancel_label = cancel_label.clone();
            let action_label = action_label.clone();
            move |compact| {
                let cancel = button(cancel_label.clone().unwrap_or_else(|| {
                    kivact::i18n::message("soft.alert_dialog.cancel", "Cancel").get()
                }))
                .variant(ButtonVariant::Ghost)
                .node_ref(cancel_ref)
                .on_press(move || open.set(false));
                let action = on_press.and_then(|on_press| {
                    action_label.as_ref().map(|label| {
                        button(label.clone())
                            .variant(action_variant)
                            .busy(action_loading)
                            .on_press(move || on_press.call(()))
                    })
                });
                if compact {
                    let full = style().width(pct(100));
                    column((
                        action.map(|action| action.style(full.clone())),
                        cancel.style(full),
                    ))
                    .style(style().gap(8).width(pct(100)))
                } else {
                    row((cancel, action))
                        .style(style().gap(8).flex_wrap(true).justify_content(Justify::End))
                }
            }
        };
        let desktop = actions.clone();
        (
            show(
                || kivact::platform::window_size().get().width <= Breakpoint::Sm.px(),
                move || actions(true),
            ),
            show(
                || kivact::platform::window_size().get().width > Breakpoint::Sm.px(),
                move || desktop(false),
            ),
        )
    };
    node().child(modal.initial_focus(cancel_ref).footer(footer))
}
