//! Role: Keep a semantic message log attached to growing content through follow-end.
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::variants::{Variant, Variants};
use crate::{ChatAlign, ChatThreadState};
use kivact_core::state::{IntoValue, Value};
use kivact_core::style::{Direction, style};
use kivact_core::view::{
    IntoView, Node, NodeBuilder, NodeRef, Role, ScrollAxes, View, node, node_ref, scroll,
};
use kivact_interaction::use_follow_end;
// xtask-allow(state-fields): Independent builder inputs and slots; use_follow_end owns the complete following and anchor state.
/// A scrollable message log that follows new content until the reader scrolls away.
pub struct ChatThread {
    root: Node,
    children: View,
    viewport: Option<NodeRef>,
    history_revision: Option<Value<u64>>,
    composer: Option<View>,
    empty: Option<(Value<bool>, View)>,
    loading_top: Value<bool>,
    streaming: Value<bool>,
    gap: Option<Value<f64>>,
    align: Value<ChatAlign>,
    scroll_button: Value<bool>,
    variants: Variants,
}
/// Place children in a `Role::Log` with shared scroll-follow behavior.
pub fn chat_thread(children: impl IntoView) -> ChatThread {
    ChatThread {
        root: node().role(Role::Log),
        children: children.into_view(),
        viewport: None,
        history_revision: None,
        composer: None,
        empty: None,
        loading_top: false.into_value(),
        streaming: false.into_value(),
        gap: None,
        align: ChatAlign::Start.into_value(),
        scroll_button: true.into_value(),
        variants: Variants::default(),
    }
}
impl ChatThread {
    /// Attach the caller's reference to the scrolling viewport.
    pub fn viewport_ref(mut self, reference: NodeRef) -> Self {
        self.viewport = Some(reference);
        self
    }
    /// The history generation; change it in the batch that prepends earlier messages.
    /// Detached readers keep the same visible content when that generation changes.
    pub fn history_revision(mut self, revision: impl IntoValue<u64>) -> Self {
        self.history_revision = Some(revision.into_value());
        self
    }
    /// Place a composer below the viewport.
    pub fn composer(mut self, content: impl IntoView) -> Self {
        self.composer = Some(content.into_view());
        self
    }
    /// Show empty-log content when the caller's collection is empty.
    pub fn empty(mut self, when: impl IntoValue<bool>, content: impl IntoView) -> Self {
        self.empty = Some((when.into_value(), content.into_view()));
        self
    }
    /// Show an earlier-history loading status above the messages.
    pub fn loading_top(mut self, loading: impl IntoValue<bool>) -> Self {
        self.loading_top = loading.into_value();
        self
    }
    /// Show a typing indicator after the messages.
    pub fn streaming(mut self, streaming: impl IntoValue<bool>) -> Self {
        self.streaming = streaming.into_value();
        self
    }
    /// Set the spacing between children.
    pub fn gap(mut self, gap: impl IntoValue<f64>) -> Self {
        self.gap = Some(gap.into_value());
        self
    }
    /// Align short history to the start or end of its viewport.
    pub fn align(mut self, align: impl IntoValue<ChatAlign>) -> Self {
        self.align = align.into_value();
        self
    }
    /// Show the jump-to-end action while the reader is away from the end.
    pub fn scroll_button(mut self, shown: impl IntoValue<bool>) -> Self {
        self.scroll_button = shown.into_value();
        self
    }
    /// Add an appearance read by every thread recipe.
    pub fn variant(mut self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for ChatThread {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for ChatThread {
    fn into_view(self) -> View {
        let viewport = self.viewport.unwrap_or_else(node_ref);
        let content = node_ref();
        let slack = kivact_core::style::use_tokens::<crate::UiTokens>()
            .chat
            .follow_slack;
        let follow = use_follow_end(viewport, content, slack, self.history_revision);
        let distance = follow.distance();
        let state = move || ChatThreadState {
            distance: distance.get(),
        };
        let loading = crate::spinner().hidden(move || !self.loading_top.get());
        let typing = crate::typing_indicator().hidden(move || !self.streaming.get());
        let gap = self.gap;
        let align = self.align;
        let messages = with_recipe_style(
            node().node_ref(content).child((
                loading,
                self.empty
                    .map(|(when, view)| node().hidden(move || !when.get()).child(view)),
                self.children,
                typing,
            )),
            |r| r.chat_thread_recipes().content_recipe(),
            &self.variants,
            state,
        )
        .style(move || {
            let s = style()
                .direction(Direction::Column)
                .grow(1)
                .shrink(0)
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
            |r| r.chat_thread_recipes().viewport_recipe(),
            &self.variants,
            state,
        );
        let shown = self.scroll_button;
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
            |r| r.chat_thread_recipes().scroll_button_recipe(),
            &self.variants,
            state,
        );
        let composer = self.composer.map(|v| {
            with_recipe_style(
                node().child(v),
                |r| r.chat_thread_recipes().composer_recipe(),
                &self.variants,
                state,
            )
        });
        with_base_recipe_style(
            self.root,
            |r| r.chat_thread_recipes().root_recipe(),
            &self.variants,
            state,
        )
        .child((viewport, jump, composer))
        .into_view()
    }
}
