//! Role: The traits DevTools implements and the view calls through.
//!
//! These traits live in a leaf module so that the view's state and tree can
//! name them without depending on `devtools_hooks`, which reads the state.
//! The DevTools target (`crate::devtools`, feature `devtools`) implements
//! them; `devtools_hooks` holds the free functions the view calls.

use std::panic::Location;
use std::rc::Rc;

use kivact_engine::api::Prop;
use kivact_engine::contract::HostEvent;
use kivact_runtime::NodeId as EffectId;
use kivact_runtime::driver::Runtime;
use kivact_runtime::{Handled, Owner, Signal, Value};
use kivact_style::{Style, Theme};
use kivact_types::{Capabilities, FocusTarget, NodeId, StateSet, WindowId};

use crate::i18n::Locale;

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

/// What DevTools does to the style of one node. Implemented by the DevTools
/// inspector only.
pub(crate) trait NodeStyleHook {
    /// `style` with the node's overrides merged over it (tracked: it reads
    /// the DevTools override signal).
    fn apply(&self, style: Style) -> Style;
    /// The states a tool forces on the node.
    fn forced(&self) -> StateSet;
}
