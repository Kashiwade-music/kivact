//! Role: The one seam through which DevTools observes and shapes the view.
//!
//! The view knows only the trait [`DevtoolsHooks`]; the DevTools target
//! (`crate::devtools`, feature `devtools`) implements it. An App without
//! DevTools holds `None`, so every hook below costs one `if let`.

use std::panic::Location;
use std::rc::Rc;

use kivact_engine::api::Prop;
use kivact_engine::contract::HostEvent;
use kivact_runtime::NodeId as EffectId;
use kivact_runtime::{Handled, Owner, Runtime, Signal, Value};
use kivact_style::internal::ForcedStates;
use kivact_style::{Style, Theme};
use kivact_types::{Capabilities, FocusTarget, NodeId, WindowId};

use crate::i18n::Locale;
use crate::view::state::ViewState;

/// What DevTools observes and shapes in the view. Implemented by the
/// DevTools inspector only.
pub(crate) trait DevtoolsHooks {
    /// A prop of `id` was written (`changed`: the engine's value changed).
    fn prop_written(&self, id: NodeId, prop: Prop, writer: Option<EffectId>, changed: bool);
    /// An engine event the DevTools turn handles (keys, pick, injection).
    fn host_event(&self, event: &HostEvent);
    /// A pointer, wheel, key, IME or file event reached `window`.
    fn input_event(&self, rt: &Runtime, event: &HostEvent, window: WindowId);
    /// A handler of `node` ran for `event` (`None`: it failed).
    fn handler_ran(&self, node: NodeId, event: &str, outcome: Option<Handled>);
    /// The focus of `window` moved.
    fn focus_changed(
        &self,
        rt: &Runtime,
        window: WindowId,
        target: Option<FocusTarget>,
        visible: bool,
    );
    /// The core sent a programmatic focus request for `target`.
    fn focus_requested(&self, target: FocusTarget);
    /// The host answered a focus request.
    fn focus_result(&self);
    /// `theme` with the DevTools token overrides applied.
    fn wrap_theme(&self, owner: Owner, theme: Value<Theme>) -> Value<Theme>;
    /// `locale` with the emulated locale applied.
    fn wrap_locale(&self, owner: Owner, locale: Value<Locale>) -> Value<Locale>;
    /// The host capabilities with the emulated touch support applied.
    fn wrap_capabilities(&self, owner: Owner, host: Signal<Capabilities>) -> Value<Capabilities>;
    /// `text` in the pseudo-locale, when one is on.
    fn pseudo(&self, text: String) -> String;
    /// The overrides and forced states of node `id`.
    fn style_hook(
        &self,
        rt: &Rc<Runtime>,
        owner: Owner,
        id: NodeId,
        at: &'static Location<'static>,
    ) -> Rc<dyn NodeStyleHook>;
}

/// The DevTools hooks of `rt`'s App; `None` without DevTools.
#[inline]
pub(crate) fn hooks(rt: &Runtime) -> Option<&dyn DevtoolsHooks> {
    ViewState::of(rt).devtools.as_deref()
}

/// Queue an engine event for the DevTools turn.
pub(crate) fn host_event(rt: &Runtime, event: &HostEvent) {
    if let Some(h) = hooks(rt) {
        h.host_event(event);
    }
}

/// Add an input event to the DevTools input log.
pub(crate) fn input_event(rt: &Runtime, event: &HostEvent, window: WindowId) {
    if let Some(h) = hooks(rt) {
        h.input_event(rt, event, window);
    }
}

/// Add one handler run to the last input log entry.
pub(crate) fn handler_ran(rt: &Runtime, node: NodeId, event: &str, outcome: Option<Handled>) {
    if let Some(h) = hooks(rt) {
        h.handler_ran(node, event, outcome);
    }
}

/// Add the focus move of `window` to the recorded history.
pub(crate) fn focus_changed(
    rt: &Runtime,
    window: WindowId,
    target: Option<FocusTarget>,
    visible: bool,
) {
    if let Some(h) = hooks(rt) {
        h.focus_changed(rt, window, target, visible);
    }
}

/// Remember the target of the programmatic focus request just sent.
pub(crate) fn focus_requested(rt: &Runtime, target: FocusTarget) {
    if let Some(h) = hooks(rt) {
        h.focus_requested(target);
    }
}

/// Drop a stale programmatic focus request.
pub(crate) fn focus_result(rt: &Runtime) {
    if let Some(h) = hooks(rt) {
        h.focus_result();
    }
}

/// `theme` as DevTools shapes it.
pub(crate) fn wrap_theme(rt: &Runtime, owner: Owner, theme: Value<Theme>) -> Value<Theme> {
    match hooks(rt) {
        Some(h) => h.wrap_theme(owner, theme),
        None => theme,
    }
}

/// `locale` as DevTools shapes it.
pub(crate) fn wrap_locale(rt: &Runtime, owner: Owner, locale: Value<Locale>) -> Value<Locale> {
    match hooks(rt) {
        Some(h) => h.wrap_locale(owner, locale),
        None => locale,
    }
}

/// Apply the pseudo-locale to a user-visible text.
pub(crate) fn pseudo(text: String) -> String {
    let Some(rt) = Runtime::current() else {
        return text;
    };
    match hooks(&rt) {
        Some(h) => h.pseudo(text),
        None => text,
    }
}

/// The style overrides and forced states of one node, read by its style
/// effect. Empty for every node of an App without DevTools.
#[derive(Clone)]
pub(crate) struct StyleHook(Option<Rc<dyn NodeStyleHook>>);

/// What DevTools does to the style of one node. Implemented by the DevTools
/// inspector only.
pub(crate) trait NodeStyleHook {
    /// `style` with the node's overrides merged over it (tracked: it reads
    /// the DevTools override signal).
    fn apply(&self, style: Style) -> Style;
    /// The states a tool forces on the node.
    fn forced(&self) -> ForcedStates;
}

impl StyleHook {
    /// The hook of node `id`.
    pub(crate) fn new(
        rt: &Rc<Runtime>,
        owner: Owner,
        id: NodeId,
        at: &'static Location<'static>,
    ) -> StyleHook {
        StyleHook(hooks(rt).map(|h| h.style_hook(rt, owner, id, at)))
    }

    /// Apply the node's overrides over `style`.
    pub(crate) fn apply(&self, style: Style) -> Style {
        match &self.0 {
            Some(hook) => hook.apply(style),
            None => style,
        }
    }

    /// The states a tool forces on the node.
    pub(crate) fn forced(&self) -> ForcedStates {
        match &self.0 {
            Some(hook) => hook.forced(),
            None => ForcedStates::default(),
        }
    }
}
