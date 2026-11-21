//! Role: Compose alert confirmation with the standard modal behavior.
use crate::media_query::Breakpoint;
use kivact::state::Callback;
use kivact::state::IntoHandler;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Signal;
use kivact::state::Value;
use kivact::style::Justify;
use kivact::style::Length;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::Dialog;
use kivact::ui::DialogKind;
use kivact::ui::button;
use kivact::ui::dialog;
use kivact::view::IntoView;
use kivact::view::View;
use kivact::view::column;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::show;

/// Confirmation dialog with a cancel button and an optional application action.
///
/// The dialog is 400 pixels wide unless `width` is set, and has no close
/// affordance. The action button appears when both `action_label` and
/// `on_press` are set; the callback decides whether and when to close the dialog.
pub struct AlertDialog {
    dialog: Dialog,
    open: Signal<bool>,
    action_label: Option<String>,
    on_press: Option<Callback<()>>,
    cancel_label: Option<String>,
    action_variant: ButtonVariant,
    action_loading: Value<bool>,
}

/// Create a 400-pixel alert modal with cancel focused initially and no close affordance.
pub fn alert_dialog(title: impl IntoText, open: Signal<bool>) -> AlertDialog {
    AlertDialog {
        dialog: dialog(title, open)
            .mode(DialogKind::AlertModal)
            .width(Length::from(400))
            .closable(false),
        open,
        action_label: None,
        on_press: None,
        cancel_label: None,
        action_variant: ButtonVariant::Danger,
        action_loading: false.into_value(),
    }
}

impl AlertDialog {
    /// Set the supporting text below the title.
    pub fn description(mut self, description: impl IntoText) -> Self {
        self.dialog = self.dialog.description(description);
        self
    }

    /// Set the dialog width; defaults to 400 pixels.
    pub fn width(mut self, width: impl IntoValue<Length>) -> Self {
        self.dialog = self.dialog.width(width);
        self
    }

    /// Set the label of the action button.
    pub fn action_label(mut self, label: impl Into<String>) -> Self {
        self.action_label = Some(label.into());
        self
    }

    /// Replace the localized cancel-button label.
    pub fn cancel_label(mut self, label: impl Into<String>) -> Self {
        self.cancel_label = Some(label.into());
        self
    }

    /// Choose the action button's appearance; danger by default.
    pub fn action_variant(mut self, variant: ButtonVariant) -> Self {
        self.action_variant = variant;
        self
    }

    /// Show the action button as busy from a read-only input; false by default.
    pub fn action_loading(mut self, loading: impl IntoValue<bool>) -> Self {
        self.action_loading = loading.into_value();
        self
    }

    /// Called when the action button is pressed.
    pub fn on_press<M>(mut self, action: impl IntoHandler<(), M>) -> Self {
        self.on_press = Some(Callback::from_handler(IntoHandler::into_handler(action)));
        self
    }
}

impl IntoView for AlertDialog {
    fn into_view(self) -> View {
        let cancel_ref = node_ref();
        let open = self.open;
        let cancel_label = self.cancel_label;
        let action_label = self.action_label;
        let on_press = self.on_press;
        let action_variant = self.action_variant;
        let action_loading = self.action_loading;
        self.dialog
            .initial_focus(cancel_ref)
            .footer(move || {
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
            })
            .into_view()
    }
}
