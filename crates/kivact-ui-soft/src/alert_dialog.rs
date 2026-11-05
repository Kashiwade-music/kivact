//! Role: Compose alert confirmation with the standard modal behavior.
use kivact::interaction::DialogMode;
use kivact::state::Binding;
use kivact::state::Callback;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::style::Justify;
use kivact::style::Length;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::button;
use kivact::ui::dialog;
use kivact::view::IntoView;
use kivact::view::NodeBuilder;
use kivact::view::View;
use kivact::view::column;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::show;
use std::rc::Rc;

pub struct AlertDialog {
    dialog: kivact::ui::Dialog,
    open: Binding<bool>,
    action: Option<(String, Rc<dyn Fn()>)>,
    cancel: Option<String>,
    variant: ButtonVariant,
    loading: Value<bool>,
}

pub fn alert_dialog(
    title: impl IntoText,
    description: impl IntoText,
    open: impl IntoBinding<bool>,
) -> AlertDialog {
    let open = open.into_binding();
    AlertDialog {
        dialog: dialog(title, open)
            .mode(DialogMode::AlertModal)
            .description(description)
            .width(Length::from(400))
            .closable(false),
        open,
        action: None,
        cancel: None,
        variant: ButtonVariant::Danger,
        loading: false.into_value(),
    }
}

impl AlertDialog {
    pub fn action(mut self, label: impl Into<String>, action: impl Into<Callback<()>>) -> Self {
        let action: Callback<()> = action.into();
        self.action = Some((label.into(), Rc::new(move || action.call(()))));
        self
    }

    pub fn cancel_label(mut self, label: impl Into<String>) -> Self {
        self.cancel = Some(label.into());
        self
    }

    pub fn action_variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn action_loading(mut self, loading: impl IntoValue<bool>) -> Self {
        self.loading = loading.into_value();
        self
    }

    pub fn width(mut self, width: impl IntoValue<Length>) -> Self {
        self.dialog = self.dialog.width(width);
        self
    }
}

impl IntoView for AlertDialog {
    fn into_view(self) -> View {
        let cancel_ref = node_ref();
        let open = self.open;
        let cancel = self.cancel;
        let action = self.action;
        let variant = self.variant;
        let loading = self.loading;
        self.dialog
            .initial_focus(cancel_ref)
            .footer(move || {
                let actions = {
                    let cancel = cancel.clone();
                    let action = action.clone();
                    move |compact| {
                        let cancel = button(cancel.clone().unwrap_or_else(|| {
                            kivact::i18n::message("soft.alert_dialog.cancel", "Cancel").get()
                        }))
                        .variant(ButtonVariant::Ghost)
                        .node_ref(cancel_ref)
                        .on_press(move || open.set(false));
                        let action = action.as_ref().map(|(label, action)| {
                            let action = action.clone();
                            button(label.clone())
                                .variant(variant)
                                .busy(loading)
                                .on_press(move || action())
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
                                .style(style().gap(8).wrap(true).justify_content(Justify::End))
                        }
                    }
                };
                let desktop = actions.clone();
                (
                    show(
                        || {
                            kivact::platform::window_size().get().width
                                <= crate::Breakpoint::Sm.px()
                        },
                        move || actions(true),
                    ),
                    show(
                        || kivact::platform::window_size().get().width > crate::Breakpoint::Sm.px(),
                        move || desktop(false),
                    ),
                )
            })
            .into_view()
    }
}
