//! Role: The one seam through which DevTools observes and shapes the view.
//!
//! The view knows only the traits in `devtools_hook_traits`; the DevTools
//! target (`crate::devtools`, feature `devtools`) implements them. An App
//! without DevTools holds `None`, so every hook below costs one `if let`.

#[cfg(feature = "devtools")]
use std::collections::BTreeMap;
use std::panic::Location;
use std::rc::Rc;

use kivact_engine::contract::HostEvent;
use kivact_runtime::{Handled, Owner, Runtime, Value};
use kivact_style::internal::ForcedStates;
use kivact_style::{Style, Theme};
use kivact_types::{FocusTarget, NodeId, WindowId};

use crate::i18n::Locale;
use crate::view::devtools_hook_traits::{DevtoolsHooks, NodeStyleHook};
use crate::view::state::ViewState;

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

/// The DevTools style overrides, parsed once when they change: by node and
/// by call site (file, line, column). The node overrides win.
#[cfg(feature = "devtools")]
#[derive(Default)]
pub(crate) struct NodeOverrides {
    pub(crate) by_node: BTreeMap<NodeId, Style>,
    pub(crate) by_site: Vec<(String, u32, u32, Style)>,
}

/// The style overrides and forced states of one node, read by its style
/// effect. Empty for every node of an App without DevTools.
#[derive(Clone)]
pub(crate) struct StyleHook(Option<Rc<dyn NodeStyleHook>>);

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
