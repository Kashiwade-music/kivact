//! Role: The chat composer component: the message field and its send action.

use super::look::ComposerStatus;
use crate::chat_composer::look::{ChatComposerState, ChatComposerTheme};
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::custom::Caller;
use kivact_core::state::{Callback, IntoValue, Signal, Value, callback};
use kivact_core::view::{
    Element, Handled, Key, KeyEvent, Modifiers, Node, Role, Variants, View, node, node_ref, text,
};
use kivact_interaction::{HistoryApi, use_history};
use kivact_macros::{Prop, component};

/// Edit a draft; Enter submits, Shift+Enter inserts a newline, and IME owns composition keys.
///
/// Enter reports the trimmed nonempty draft to `on_submit` after clearing
/// `value`. `on_stop` replaces Send with Stop while `stop_shown` is true.
/// `history` enables recall of submitted drafts at the editor's Up and Down
/// edges. `status` shows a validation diagnostic above or below the surface,
/// as `status_position` places it. The slots fill the drawer, the header, the
/// footer, the send control and its icons.
#[component]
pub fn chat_composer(
    value: Signal<String>,
    on_submit: Option<Callback<String>>,
    on_stop: Option<Callback<()>>,
    #[prop(default = false)] stop_shown: Value<bool>,
    #[prop(default = kivact_core::i18n::message("chat_composer.placeholder", "Write a message..."))]
    placeholder: Value<String>,
    #[prop(default = true)] history: Value<bool>,
    #[prop(default = None::<ComposerStatus>)] status: Value<Option<ComposerStatus>>,
    #[prop(default = ComposerStatusPosition::Bottom)] status_position: Value<
        ComposerStatusPosition,
    >,
    drawer: Option<View>,
    header_actions: Option<View>,
    header_context: Option<View>,
    footer_actions: Option<View>,
    send_actions: Option<View>,
    send_icon: Option<View>,
    stop_icon: Option<View>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let stopped = stop_shown;
    let state = move || ChatComposerState {
        stopped: stopped.get(),
        status: status.get(),
    };
    let disabled = caller.disabled().unwrap_or_else(|| false.into_value());
    let links = ComposerLinks {
        value,
        stopped,
        submit: on_submit,
        stop: on_stop,
        disabled,
    };
    let (input, send) = composer_input(links, placeholder, history, &variants, state);
    let send_icon = send_icon.unwrap_or_else(|| node().into_view());
    let stop_icon = stop_icon.unwrap_or_else(|| node().into_view());
    let control = send_control(links, send, send_icon, stop_icon, &variants, state);
    let header = composer_header(header_context, header_actions, &variants, state);
    let footer = composer_footer(footer_actions, send_actions, control, &variants, state);
    let surface = with_recipe_style(
        node().child((header, input, footer)),
        |r: &ChatComposerTheme| r.surface_recipe(),
        &variants,
        state,
    );
    let top = diagnostic(
        ComposerStatusPosition::Top,
        status,
        status_position,
        &variants,
        state,
    );
    let bottom = diagnostic(
        ComposerStatusPosition::Bottom,
        status,
        status_position,
        &variants,
        state,
    );
    let drawer = composer_slot(
        drawer,
        |r: &ChatComposerTheme| r.drawer_recipe(),
        &variants,
        state,
    );
    with_base_recipe_style(
        node(),
        |r: &ChatComposerTheme| r.root_recipe(),
        &variants,
        state,
    )
    .child((top, drawer, surface, bottom))
}

/// The handles the composer's parts share, read from its props.
#[derive(Clone, Copy)]
struct ComposerLinks {
    value: Signal<String>,
    stopped: Value<bool>,
    submit: Option<Callback<String>>,
    stop: Option<Callback<()>>,
    disabled: Value<bool>,
}

fn submission(links: ComposerLinks, history: HistoryApi) -> Callback<()> {
    let (value, stopped, submit, disabled) =
        (links.value, links.stopped, links.submit, links.disabled);
    callback(move |()| {
        if disabled.get_untracked() || stopped.get_untracked() {
            return;
        }
        let draft = value.get_untracked();
        let draft = draft.trim();
        if let Some(submit) = submit.filter(|_| !draft.is_empty()) {
            history.push(draft);
            value.set(String::new());
            submit.call(draft.to_owned());
        }
    })
}

fn composer_input(
    links: ComposerLinks,
    placeholder: Value<String>,
    history_on: Value<bool>,
    variants: &Variants,
    state: impl Fn() -> ChatComposerState + Copy + 'static,
) -> (crate::TextInput, Callback<()>) {
    let input_ref = node_ref();
    let history = use_history(input_ref, history_on);
    let send = submission(links, history);
    let input = crate::text_area(links.value)
        .label(kivact_core::i18n::message(
            "chat_composer.input",
            "Message input",
        ))
        .placeholder(placeholder)
        .disabled(links.disabled)
        .node_ref(input_ref)
        .with_node(|node| node.part(history.input()))
        .forward_keys(vec![Key::Enter])
        .on_key(move |event| composer_key(event, send));
    (
        with_recipe_style(
            input,
            |r: &ChatComposerTheme| r.input_recipe(),
            variants,
            state,
        ),
        send,
    )
}

fn send_control(
    links: ComposerLinks,
    send: Callback<()>,
    send_icon: View,
    stop_icon: View,
    variants: &Variants,
    state: impl Fn() -> ChatComposerState + Copy + 'static,
) -> crate::Button {
    let (stopped, stop, submit, value) = (links.stopped, links.stop, links.submit, links.value);
    let disabled = links.disabled;
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
    .children((
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
        |r: &ChatComposerTheme| r.send_recipe(),
        variants,
        state,
    )
}

fn composer_header(
    context: Option<View>,
    actions: Option<View>,
    variants: &Variants,
    state: impl Fn() -> ChatComposerState + Copy + 'static,
) -> Node {
    let context = composer_slot(
        context,
        |r: &ChatComposerTheme| r.header_context_recipe(),
        variants,
        state,
    );
    let actions = composer_slot(
        actions,
        |r: &ChatComposerTheme| r.header_actions_recipe(),
        variants,
        state,
    );
    with_recipe_style(
        node().child((context, actions)),
        |r: &ChatComposerTheme| r.header_recipe(),
        variants,
        state,
    )
}

fn composer_footer(
    actions: Option<View>,
    send_actions: Option<View>,
    control: crate::Button,
    variants: &Variants,
    state: impl Fn() -> ChatComposerState + Copy + 'static,
) -> Node {
    let actions = composer_slot(
        actions,
        |r: &ChatComposerTheme| r.footer_actions_recipe(),
        variants,
        state,
    );
    let send_actions = composer_slot(
        send_actions,
        |r: &ChatComposerTheme| r.send_actions_recipe(),
        variants,
        state,
    );
    with_recipe_style(
        node().child((actions, send_actions, control)),
        |r: &ChatComposerTheme| r.footer_recipe(),
        variants,
        state,
    )
}

fn diagnostic(
    side: ComposerStatusPosition,
    status: Value<Option<ComposerStatus>>,
    position: Value<ComposerStatusPosition>,
    variants: &Variants,
    state: impl Fn() -> ChatComposerState + Copy + 'static,
) -> Node {
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
        |r: &ChatComposerTheme| r.status_recipe(),
        variants,
        state,
    )
}

fn composer_slot(
    view: Option<View>,
    recipe: fn(&ChatComposerTheme) -> &crate::recipe::Recipe<ChatComposerState>,
    variants: &Variants,
    state: impl Fn() -> ChatComposerState + Copy + 'static,
) -> Option<Node> {
    view.map(|view| with_recipe_style(node().child(view), recipe, variants, state))
}

fn composer_key(event: KeyEvent, send: Callback<()>) -> Handled {
    if event.key() == Key::Enter && event.modifiers() == Modifiers::default() {
        send.call(());
        return Handled::Yes;
    }
    Handled::No
}

/// Placement of a composer diagnostic.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum ComposerStatusPosition {
    Top,
    #[default]
    Bottom,
}
