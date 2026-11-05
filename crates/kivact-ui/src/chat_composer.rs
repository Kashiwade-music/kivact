//! Role: Submit multiline drafts through keyboard and send/stop controls.
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::variants::{Variant, Variants};
use crate::{ChatComposerState, ComposerStatus, ComposerStatusPosition};
use kivact_core::custom::NodeRefExt;
use kivact_core::state::{Binding, Callback, IntoBinding, IntoText, IntoValue, Value, callback};
use kivact_core::view::{
    Handled, HandlerResult, IntoView, Key, KeyEvent, Modifiers, Node, NodeBuilder, Role, View,
    node, node_ref, text,
};
// xtask-allow(state-fields): Independent draft binding, diagnostic inputs and slots; History owns recall and shared native editing owns the draft session.
/// A multiline draft with independent action slots and optional submission history.
pub struct ChatComposer {
    root: Node,
    value: Binding<String>,
    on_submit: Option<Callback<String>>,
    on_stop: Option<Callback<()>>,
    stop_shown: Value<bool>,
    placeholder: Value<String>,
    history: Value<bool>,
    status: Value<Option<ComposerStatus>>,
    status_position: Value<ComposerStatusPosition>,
    drawer: Option<View>,
    header_actions: Option<View>,
    header_context: Option<View>,
    footer_actions: Option<View>,
    send_actions: Option<View>,
    send_icon: Option<View>,
    stop_icon: Option<View>,
    variants: Variants,
}
/// Edit a draft; Enter submits, Shift+Enter inserts a newline, and IME owns composition keys.
pub fn chat_composer(value: impl IntoBinding<String>) -> ChatComposer {
    ChatComposer {
        root: node(),
        value: value.into_binding(),
        on_submit: None,
        on_stop: None,
        stop_shown: false.into_value(),
        placeholder: kivact_core::i18n::message("chat_composer.placeholder", "Write a message..."),
        history: true.into_value(),
        status: None.into_value(),
        status_position: ComposerStatusPosition::Bottom.into_value(),
        drawer: None,
        header_actions: None,
        header_context: None,
        footer_actions: None,
        send_actions: None,
        send_icon: None,
        stop_icon: None,
        variants: Variants::default(),
    }
}
impl ChatComposer {
    /// Report the trimmed nonempty draft after clearing the binding.
    pub fn on_submit<R: HandlerResult>(mut self, submit: impl Fn(String) -> R + 'static) -> Self {
        self.on_submit = Some(callback(submit));
        self
    }
    /// Report requests to stop the current response.
    pub fn on_stop(mut self, stop: impl Into<Callback<()>>) -> Self {
        self.on_stop = Some(stop.into());
        self
    }
    /// Replace Send with Stop while a response is in progress.
    pub fn stop_shown(mut self, shown: impl IntoValue<bool>) -> Self {
        self.stop_shown = shown.into_value();
        self
    }
    /// Set a localized draft hint.
    pub fn placeholder(mut self, hint: impl IntoText) -> Self {
        self.placeholder = hint.into_text_value();
        self
    }
    /// Enable recall of submitted drafts at the editor's Up and Down edges.
    pub fn history(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.history = enabled.into_value();
        self
    }
    /// Set a validation diagnostic.
    pub fn status(mut self, status: impl IntoValue<Option<ComposerStatus>>) -> Self {
        self.status = status.into_value();
        self
    }
    /// Place diagnostics above or below the surface.
    pub fn status_position(mut self, position: impl IntoValue<ComposerStatusPosition>) -> Self {
        self.status_position = position.into_value();
        self
    }
    /// Add an appearance read by every composer recipe.
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
    /// Fill the drawer slot.
    pub fn drawer(mut self, content: impl IntoView) -> Self {
        self.drawer = Some(content.into_view());
        self
    }
    /// Fill the header actions slot.
    pub fn header_actions(mut self, content: impl IntoView) -> Self {
        self.header_actions = Some(content.into_view());
        self
    }
    /// Fill the header context slot.
    pub fn header_context(mut self, content: impl IntoView) -> Self {
        self.header_context = Some(content.into_view());
        self
    }
    /// Fill the footer actions slot.
    pub fn footer_actions(mut self, content: impl IntoView) -> Self {
        self.footer_actions = Some(content.into_view());
        self
    }
    /// Fill the send actions slot.
    pub fn send_actions(mut self, content: impl IntoView) -> Self {
        self.send_actions = Some(content.into_view());
        self
    }
    /// Fill the send icon slot.
    pub fn send_icon(mut self, content: impl IntoView) -> Self {
        self.send_icon = Some(content.into_view());
        self
    }
    /// Fill the stop icon slot.
    pub fn stop_icon(mut self, content: impl IntoView) -> Self {
        self.stop_icon = Some(content.into_view());
        self
    }
}
impl NodeBuilder for ChatComposer {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl ChatComposer {
    fn submission(
        &self,
        disabled: Value<bool>,
        history: kivact_core::state::MachineHandle<crate::composer_history::Policy>,
    ) -> Callback<()> {
        let value = self.value;
        let stopped = self.stop_shown;
        let keep_history = self.history;
        let submit = self.on_submit;
        callback(move |()| {
            if disabled.get_untracked() || stopped.get_untracked() {
                return;
            }
            let draft = value.get_untracked();
            let draft = draft.trim();
            if let Some(submit) = submit.filter(|_| !draft.is_empty()) {
                if keep_history.get_untracked() {
                    history.send(crate::composer_history::Event::Submit(draft.to_owned()));
                }
                value.set(String::new());
                submit.call(draft.to_owned());
            }
        })
    }
    fn input_and_submission(
        &self,
        disabled: Value<bool>,
        state: impl Fn() -> ChatComposerState + Copy + 'static,
    ) -> (crate::TextInput, Callback<()>) {
        let input_ref = node_ref();
        let history = crate::composer_history::use_history(input_ref);
        let send = self.submission(disabled, history);
        let keep_history = self.history;
        let input = crate::text_area(
            kivact_core::i18n::message("chat_composer.input", "Message input"),
            self.value,
        )
        .placeholder(self.placeholder)
        .disabled(disabled)
        .node_ref(input_ref)
        .forward_keys(vec![Key::Enter])
        .on_key(move |event| composer_key(event, input_ref, send, history, keep_history));
        (
            with_recipe_style(
                input,
                |r| r.chat_composer_recipes().input_recipe(),
                &self.variants,
                state,
            ),
            send,
        )
    }
    fn send_control(
        &mut self,
        disabled: Value<bool>,
        send: Callback<()>,
        state: impl Fn() -> ChatComposerState + Copy + 'static,
    ) -> crate::Button {
        let stopped = self.stop_shown;
        let stop = self.on_stop;
        let submit = self.on_submit;
        let value = self.value;
        let send_icon = self.send_icon.take().unwrap_or_else(|| node().into_view());
        let stop_icon = self.stop_icon.take().unwrap_or_else(|| node().into_view());
        let stop_label = kivact_core::i18n::message("chat_composer.stop", "Stop");
        let send_label = kivact_core::i18n::message("chat_composer.send", "Send");
        let control = crate::button(move || {
            if stopped.get() {
                stop_label.get()
            } else {
                send_label.get()
            }
        })
        .disabled(move || {
            disabled.get()
                || if stopped.get() {
                    stop.is_none()
                } else {
                    submit.is_none() || value.with(|v| v.trim().is_empty())
                }
        })
        .child((
            node().hidden(stopped).child(send_icon),
            node().hidden(move || !stopped.get()).child(stop_icon),
        ))
        .on_press(move || {
            if stopped.get_untracked() {
                if let Some(stop) = stop {
                    stop.call(())
                }
            } else {
                send.call(())
            }
        });
        with_recipe_style(
            control,
            |r| r.chat_composer_recipes().send_recipe(),
            &self.variants,
            state,
        )
    }
    fn header(&mut self, state: impl Fn() -> ChatComposerState + Copy + 'static) -> Node {
        let context = composer_slot(
            self.header_context.take(),
            |r| r.chat_composer_recipes().header_context_recipe(),
            &self.variants,
            state,
        );
        let actions = composer_slot(
            self.header_actions.take(),
            |r| r.chat_composer_recipes().header_actions_recipe(),
            &self.variants,
            state,
        );
        with_recipe_style(
            node().child((context, actions)),
            |r| r.chat_composer_recipes().header_recipe(),
            &self.variants,
            state,
        )
    }
    fn footer(
        &mut self,
        control: crate::Button,
        state: impl Fn() -> ChatComposerState + Copy + 'static,
    ) -> Node {
        let actions = composer_slot(
            self.footer_actions.take(),
            |r| r.chat_composer_recipes().footer_actions_recipe(),
            &self.variants,
            state,
        );
        let send_actions = composer_slot(
            self.send_actions.take(),
            |r| r.chat_composer_recipes().send_actions_recipe(),
            &self.variants,
            state,
        );
        with_recipe_style(
            node().child((actions, send_actions, control)),
            |r| r.chat_composer_recipes().footer_recipe(),
            &self.variants,
            state,
        )
    }
    fn diagnostic(
        &self,
        side: ComposerStatusPosition,
        state: impl Fn() -> ChatComposerState + Copy + 'static,
    ) -> Node {
        let status = self.status;
        let position = self.status_position;
        let diagnostic = text(move || {
            status.with(|s| match s {
                Some(ComposerStatus::Error(s) | ComposerStatus::Warning(s)) => s.clone(),
                None => String::new(),
            })
        })
        .role(Role::Status)
        .hidden(move || status.with(Option::is_none) || position.get() != side);
        with_recipe_style(
            diagnostic,
            |r| r.chat_composer_recipes().status_recipe(),
            &self.variants,
            state,
        )
    }
}
fn composer_slot(
    view: Option<View>,
    recipe: fn(&crate::recipes::UiRecipes) -> &crate::recipes::Recipe<ChatComposerState>,
    variants: &Variants,
    state: impl Fn() -> ChatComposerState + Copy + 'static,
) -> Option<Node> {
    view.map(|view| with_recipe_style(node().child(view), recipe, variants, state))
}
fn composer_key(
    event: KeyEvent,
    input: kivact_core::view::NodeRef,
    send: Callback<()>,
    history: kivact_core::state::MachineHandle<crate::composer_history::Policy>,
    keep_history: Value<bool>,
) -> Handled {
    if event.key() == Key::Enter && event.modifiers() == Modifiers::default() {
        send.call(());
        return Handled::Yes;
    }
    if !keep_history.get_untracked()
        || event.modifiers() != Modifiers::default()
        || !matches!(event.key(), Key::ArrowUp | Key::ArrowDown)
    {
        return Handled::No;
    }
    let Some(editor) = input.edit_state() else {
        return Handled::No;
    };
    let selection = editor.selection();
    let at_start = selection.start == 0;
    let at_end = selection.end == editor.value().len();
    let eligible = match event.key() {
        Key::ArrowUp => at_start && (selection.is_empty() || at_end),
        Key::ArrowDown => at_end && (selection.is_empty() || at_start),
        _ => false,
    };
    if !eligible {
        return Handled::No;
    }
    history.send(crate::composer_history::Event::Recall {
        previous: event.key() == Key::ArrowUp,
        draft: editor.value().to_owned(),
    });
    Handled::Yes
}
impl IntoView for ChatComposer {
    fn into_view(mut self) -> View {
        let stopped = self.stop_shown;
        let status = self.status;
        let state = move || ChatComposerState {
            stopped: stopped.get(),
            status: status.get(),
        };
        let disabled = self
            .root
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        let (input, send) = self.input_and_submission(disabled, state);
        let control = self.send_control(disabled, send, state);
        let header = self.header(state);
        let footer = self.footer(control, state);
        let surface = with_recipe_style(
            node().child((header, input, footer)),
            |r| r.chat_composer_recipes().surface_recipe(),
            &self.variants,
            state,
        );
        let top = self.diagnostic(ComposerStatusPosition::Top, state);
        let bottom = self.diagnostic(ComposerStatusPosition::Bottom, state);
        let drawer = composer_slot(
            self.drawer.take(),
            |r| r.chat_composer_recipes().drawer_recipe(),
            &self.variants,
            state,
        );
        with_base_recipe_style(
            self.root,
            |r| r.chat_composer_recipes().root_recipe(),
            &self.variants,
            state,
        )
        .child((top, drawer, surface, bottom))
        .into_view()
    }
}
