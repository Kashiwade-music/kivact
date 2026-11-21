//! Role: Build the chat thread log, its follow-end viewport, and its jump control.

use crate::chat_message::look::ChatAlign;
use crate::chat_thread::look::ChatThreadState;
use crate::chat_thread::look::ChatThreadTheme;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::custom::Caller;
use kivact_core::state::Value;
use kivact_core::style::{FlexDirection, style};
use kivact_core::view::{Element, NodeRef, Role, ScrollAxes, View, node, node_ref, scroll};
use kivact_interaction::use_follow_end;
use kivact_macros::component;

/// A scrollable message log that follows new content until the reader scrolls away.
///
/// `children` are the messages. `viewport_ref` attaches the caller's reference to
/// the scrolling viewport, and `history_revision` is the history generation: change
/// it in the batch that prepends earlier messages, and detached readers keep the
/// same visible content. `composer` is placed below the viewport. `empty` is the
/// empty-log content, shown while `empty_when` is true (default false).
/// `loading_top` shows an earlier-history loading status above the messages,
/// `streaming` shows a typing indicator after them, `gap` sets the spacing between
/// children, `align` places short history at the start or end of its viewport,
/// and `scroll_button` shows the jump-to-end action while the reader is away from
/// the end.
#[component]
pub fn chat_thread(
    children: View,
    viewport_ref: Option<NodeRef>,
    history_revision: Option<Value<u64>>,
    composer: Option<View>,
    empty: Option<View>,
    #[prop(default = false)] empty_when: Value<bool>,
    #[prop(default = false)] loading_top: Value<bool>,
    #[prop(default = false)] streaming: Value<bool>,
    gap: Option<Value<f64>>,
    #[prop(default = ChatAlign::Start)] align: Value<ChatAlign>,
    #[prop(default = true)] scroll_button: Value<bool>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let viewport = viewport_ref.unwrap_or_else(node_ref);
    let content = node_ref();
    let slack = follow_slack();
    let follow = use_follow_end(viewport, content, slack, history_revision);
    let distance = follow.distance();
    let state = move || ChatThreadState {
        distance: distance.get(),
    };
    let loading = crate::spinner().hidden(move || !loading_top.get());
    let typing = crate::typing_indicator().hidden(move || !streaming.get());
    let messages = with_recipe_style(
        node().node_ref(content).child((
            loading,
            empty.map(|view| node().hidden(move || !empty_when.get()).child(view)),
            children,
            typing,
        )),
        |r: &ChatThreadTheme| r.content_recipe(),
        &variants,
        state,
    )
    .style(move || {
        let s = style()
            .flex_direction(FlexDirection::Column)
            .flex_grow(1)
            .flex_shrink(0)
            .justify_content(if align.get() == ChatAlign::End {
                kivact_core::style::Justify::End
            } else {
                kivact_core::style::Justify::Start
            });
        match gap {
            Some(gap) => s.gap(gap.get()),
            None => s,
        }
    });
    let viewport = with_recipe_style(
        scroll(messages)
            .axes(ScrollAxes::Vertical)
            .node_ref(viewport),
        |r: &ChatThreadTheme| r.viewport_recipe(),
        &variants,
        state,
    );
    let shown = scroll_button;
    let jump = with_recipe_style(
        crate::button(kivact_core::i18n::message(
            "chat_thread.jump",
            "Jump to latest",
        ))
        .variant(crate::ButtonVariant::Outline)
        .icon(kivact_core::view::icon("arrow-down"))
        .hide_label(true)
        .hidden(move || !shown.get() || distance.get() <= slack)
        .on_press(move || follow.jump_to_end()),
        |r: &ChatThreadTheme| r.scroll_button_recipe(),
        &variants,
        state,
    );
    let composer = composer.map(|v| {
        with_recipe_style(
            node().child(v),
            |r: &ChatThreadTheme| r.composer_recipe(),
            &variants,
            state,
        )
    });
    with_base_recipe_style(
        node().role(Role::Log),
        |r: &ChatThreadTheme| r.root_recipe(),
        &variants,
        state,
    )
    .child((viewport, jump, composer))
}

/// The chat thread's follow slack in the theme in effect (tracked).
fn follow_slack() -> f64 {
    kivact_core::style::use_theme().with(|t| t.get(&crate::chat_thread::look::FOLLOW_SLACK_VAR))
}
