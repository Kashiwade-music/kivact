//! Role: Hooks the view calls so the DevTools can read and shape what the App does.

use std::panic::Location;
use std::rc::Rc;

use kivact_engine::api::Prop;
use kivact_engine::contract::HostEvent;
use kivact_runtime::NodeId as EffectId;
use kivact_runtime::{Handled, Owner, Runtime, Signal, Value};
use kivact_style::Style;
use kivact_style::Theme;
use kivact_style::internal::ForcedStates;
use kivact_types::{Capabilities, FocusTarget, NodeId, WindowId};

use crate::i18n::Locale;

#[cfg(feature = "devtools")]
use kivact_devtools_protocol as p;

#[cfg(feature = "devtools")]
use kivact_runtime::memo;

#[cfg(feature = "devtools")]
use crate::view::devtools_apply;

#[cfg(feature = "devtools")]
use crate::view::devtools_state::{DevtoolsEvent, DevtoolsState, Pending, PropWrite};

/// Record a prop write while recording.
#[cfg(feature = "devtools")]
pub(crate) fn prop_written(id: NodeId, prop: Prop, writer: Option<EffectId>, changed: bool) {
    let Some(rt) = Runtime::current() else {
        return;
    };
    let DevtoolsState::Running(i) = DevtoolsState::of(&rt) else {
        return;
    };
    if !i.hooks.recording.borrow().updates {
        return;
    }
    i.hooks.prop_writes.borrow_mut().push(PropWrite {
        node: id,
        prop,
        effect: writer,
        changed,
    });
}

#[cfg(not(feature = "devtools"))]
pub(crate) fn prop_written(_: NodeId, _: Prop, _: Option<EffectId>, _: bool) {}

/// Queue an engine event for the DevTools turn.
#[cfg(feature = "devtools")]
pub(crate) fn host_event(rt: &Runtime, event: &HostEvent) {
    let DevtoolsState::Running(i) = DevtoolsState::of(rt) else {
        return;
    };
    let queued = match event {
        HostEvent::DevtoolsKey { window, pick } => Some(DevtoolsEvent::Key {
            window: *window,
            pick: *pick,
        }),
        HostEvent::DevtoolsPick {
            window,
            node,
            phase,
        } => Some(DevtoolsEvent::Pick {
            window: *window,
            node: *node,
            phase: *phase,
        }),
        HostEvent::InjectDelivered { request, .. } => {
            Some(DevtoolsEvent::InjectDelivered { request: *request })
        }
        _ => None,
    };
    if let Some(queued) = queued {
        i.session.events.borrow_mut().push(queued);
    }
}

#[cfg(not(feature = "devtools"))]
pub(crate) fn host_event(_: &Runtime, _: &HostEvent) {}

/// Add the focus move of `window` to the recorded history.
#[cfg(feature = "devtools")]
pub(crate) fn focus_changed(
    rt: &Runtime,
    window: WindowId,
    target: Option<FocusTarget>,
    visible: bool,
) {
    let DevtoolsState::Running(i) = DevtoolsState::of(rt) else {
        return;
    };
    if !i.hooks.recording.borrow().input {
        return;
    }
    let mut history = i.hooks.focus_history.borrow_mut();
    let moves = history.entry(window).or_default();
    let before = moves.back().and_then(|entry| entry.after);
    let reason = match i.hooks.programmatic_focus.take() {
        Some(requested) if Some(requested) == target => p::FocusReason::Programmatic,
        _ if visible => p::FocusReason::Keyboard,
        _ => p::FocusReason::Pointer,
    };
    if moves.len() == 100 {
        moves.pop_front();
    }
    moves.push_back(p::FocusMove {
        at_ns: rt.now_ns(),
        before,
        after: target.map(|target| target.node.0),
        reason,
    });
}

#[cfg(not(feature = "devtools"))]
pub(crate) fn focus_changed(_: &Runtime, _: WindowId, _: Option<FocusTarget>, _: bool) {}

/// Drop a stale programmatic focus request.
#[cfg(feature = "devtools")]
pub(crate) fn focus_result(rt: &Runtime) {
    let DevtoolsState::Running(i) = DevtoolsState::of(rt) else {
        return;
    };
    i.hooks.programmatic_focus.take();
}

#[cfg(not(feature = "devtools"))]
pub(crate) fn focus_result(_: &Runtime) {}

/// Add a pointer, wheel, key, IME or file event to the input log.
#[cfg(feature = "devtools")]
pub(crate) fn input_event(rt: &Runtime, event: &HostEvent, window: WindowId) {
    let DevtoolsState::Running(i) = DevtoolsState::of(rt) else {
        return;
    };
    if !i.hooks.recording.borrow().input {
        return;
    }
    let Some((target, kind, detail)) = input_description(event) else {
        return;
    };
    let injected = i.session.pending.borrow().iter().any(|pending| {
        matches!(pending, Pending::Inject { window: pending_window, .. } if *pending_window == window)
    });
    let source = if injected {
        p::InputSource::Injected
    } else {
        p::InputSource::Platform
    };
    let seq = i.hooks.next_input.get() + 1;
    i.hooks.next_input.set(seq);
    let mut log = i.hooks.input_log.borrow_mut();
    if log.len() == 500 {
        log.pop_front();
    }
    log.push_back(p::InputLogEntry {
        seq,
        at_ns: rt.now_ns(),
        window: window.get(),
        kind: kind.to_owned(),
        detail,
        source,
        target: target.map(|node| node.0),
        route: Vec::new(),
        handlers: Vec::new(),
    });
}

#[cfg(not(feature = "devtools"))]
pub(crate) fn input_event(_: &Runtime, _: &HostEvent, _: WindowId) {}

#[cfg(feature = "devtools")]
fn input_description(event: &HostEvent) -> Option<(Option<NodeId>, &'static str, String)> {
    match event {
        HostEvent::Pointer(route) => Some((
            route.target.map(|target| target.node),
            "pointer",
            format!("{:?}", route.input.phase),
        )),
        HostEvent::Wheel(route) => Some((
            route.target.map(|target| target.node),
            "wheel",
            format!("{:.1}, {:.1}", route.input.delta.x, route.input.delta.y),
        )),
        HostEvent::Key(route) => Some((
            route.target.map(|target| target.node),
            "key",
            format!(
                "{:?} {}",
                route.input.key,
                match route.input.phase {
                    kivact_types::KeyPhase::Up => "up",
                    kivact_types::KeyPhase::Down { .. } => "down",
                }
            ),
        )),
        HostEvent::Ime { node, event } => Some((Some(*node), "ime", format!("{event:?}"))),
        HostEvent::File(route) => Some((
            route.target.map(|target| target.node),
            "file",
            format!(
                "{:?} ({} files)",
                route.input.phase,
                route.input.files.len()
            ),
        )),
        _ => None,
    }
}

/// Add one handler run to the last input log entry.
#[cfg(feature = "devtools")]
pub(crate) fn handler_ran(rt: &Runtime, node: NodeId, event: &str, outcome: Option<Handled>) {
    let DevtoolsState::Running(i) = DevtoolsState::of(rt) else {
        return;
    };
    if !i.hooks.recording.borrow().input {
        return;
    }
    let mut log = i.hooks.input_log.borrow_mut();
    let Some(entry) = log.back_mut() else {
        return;
    };
    if entry.route.last() != Some(&node.0) {
        entry.route.push(node.0);
    }
    let outcome = match outcome {
        Some(Handled::Yes) => p::HandlerOutcome::Handled,
        Some(Handled::No) => p::HandlerOutcome::NotHandled,
        Some(Handled::PreventDefault) => p::HandlerOutcome::PreventDefault,
        None => p::HandlerOutcome::Failed,
    };
    entry.handlers.push(p::HandlerCall {
        node: node.0,
        event: event.to_owned(),
        outcome,
    });
}

#[cfg(not(feature = "devtools"))]
pub(crate) fn handler_ran(_: &Runtime, _: NodeId, _: &str, _: Option<Handled>) {}

/// Remember the target of the next programmatic focus request.
#[cfg(feature = "devtools")]
pub(crate) fn focus_requested(rt: &Runtime, target: FocusTarget) {
    let DevtoolsState::Running(i) = DevtoolsState::of(rt) else {
        return;
    };
    i.hooks.programmatic_focus.put(target);
}

#[cfg(not(feature = "devtools"))]
pub(crate) fn focus_requested(_: &Runtime, _: FocusTarget) {}

/// Whether the DevTools agent runs in this App.
#[cfg(feature = "devtools")]
pub(crate) fn is_running(rt: &Runtime) -> bool {
    matches!(DevtoolsState::of(rt), DevtoolsState::Running(_))
}

#[cfg(not(feature = "devtools"))]
#[allow(
    dead_code,
    reason = "the call site that asks is compiled only with the devtools feature"
)]
pub(crate) fn is_running(_: &Runtime) -> bool {
    false
}

/// Wrap `theme` so token overrides apply to it.
#[cfg(feature = "devtools")]
pub(crate) fn wrap_theme(owner: Owner, theme: Value<Theme>) -> Value<Theme> {
    let Some(rt) = Runtime::current() else {
        return theme;
    };
    let DevtoolsState::Running(i) = DevtoolsState::of(&rt) else {
        return theme;
    };
    let overrides = i.hooks.overrides;
    owner.with_setup(|| {
        memo(move || devtools_apply::apply_token_overrides(theme.get(), &overrides.get()))
    })
}

#[cfg(not(feature = "devtools"))]
pub(crate) fn wrap_theme(_owner: Owner, theme: Value<Theme>) -> Value<Theme> {
    theme
}

/// Wrap `locale` so the emulated locale applies to it.
#[cfg(feature = "devtools")]
pub(crate) fn wrap_locale(owner: Owner, locale: Value<Locale>) -> Value<Locale> {
    let Some(rt) = Runtime::current() else {
        return locale;
    };
    let DevtoolsState::Running(i) = DevtoolsState::of(&rt) else {
        return locale;
    };
    let emulation = i.hooks.emulation;
    owner.with_setup(|| {
        memo(move || match emulation.get().locale {
            Some(tag) => Locale::new(&tag),
            None => locale.get(),
        })
    })
}

#[cfg(not(feature = "devtools"))]
pub(crate) fn wrap_locale(_owner: Owner, locale: Value<Locale>) -> Value<Locale> {
    locale
}

/// Apply the pseudo-locale to a user-visible text.
#[cfg(feature = "devtools")]
pub(crate) fn pseudo(text: String) -> String {
    let Some(rt) = Runtime::current() else {
        return text;
    };
    let DevtoolsState::Running(i) = DevtoolsState::of(&rt) else {
        return text;
    };
    let mode = i.hooks.emulation.get().pseudo_locale;
    if mode == p::PseudoLocale::Off {
        return text;
    }
    devtools_apply::pseudo_locale(mode, &text)
}

#[cfg(not(feature = "devtools"))]
pub(crate) fn pseudo(text: String) -> String {
    text
}

/// Wrap the host capabilities so the emulated touch support applies to them.
#[cfg(feature = "devtools")]
pub(crate) fn wrap_capabilities(owner: Owner, host: Signal<Capabilities>) -> Value<Capabilities> {
    owner.with_setup(|| {
        memo(move || {
            let mut capabilities = host.get();
            if let Some(rt) = Runtime::current()
                && let DevtoolsState::Running(i) = DevtoolsState::of(&rt)
                && let Some(touch) = i.hooks.emulation.get().touch
            {
                capabilities.touch = touch;
            }
            capabilities
        })
    })
}

#[cfg(not(feature = "devtools"))]
pub(crate) fn wrap_capabilities(owner: Owner, host: Signal<Capabilities>) -> Value<Capabilities> {
    owner.with_setup(|| kivact_runtime::memo(move || host.get()))
}

/// The style overrides and forced states of one node, read by its style effect.
#[cfg(feature = "devtools")]
#[derive(Clone, Copy)]
pub(crate) enum StyleHook {
    Off,
    On {
        overrides: Value<Vec<(String, String)>>,
        forced: Value<ForcedStates>,
    },
}

/// The style overrides and forced states of one node (none without DevTools).
#[cfg(not(feature = "devtools"))]
#[derive(Clone, Copy)]
pub(crate) struct StyleHook;

impl StyleHook {
    /// Resolve the overrides and forced states that apply to `id`.
    #[cfg(feature = "devtools")]
    pub(crate) fn new(
        rt: &Rc<Runtime>,
        owner: Owner,
        id: NodeId,
        at: &'static Location<'static>,
    ) -> StyleHook {
        let DevtoolsState::Running(i) = DevtoolsState::of(rt) else {
            return StyleHook::Off;
        };
        let sites = devtools_apply::call_sites(rt, owner, Some(at));
        let overrides = i.hooks.overrides;
        let forced = i.hooks.forced;
        owner.with_setup(|| StyleHook::On {
            overrides: memo(move || devtools_apply::node_overrides(&overrides.get(), id, &sites)),
            forced: memo(move || forced.with(|all| all.get(&id).copied().unwrap_or_default())),
        })
    }

    /// Resolve the overrides and forced states that apply to `id`.
    #[cfg(not(feature = "devtools"))]
    pub(crate) fn new(
        _rt: &Rc<Runtime>,
        _owner: Owner,
        _id: NodeId,
        _at: &'static Location<'static>,
    ) -> StyleHook {
        StyleHook
    }

    /// Apply the node's overrides over `style`.
    #[cfg(feature = "devtools")]
    pub(crate) fn apply(&self, style: Style) -> Style {
        let StyleHook::On { overrides, .. } = self else {
            return style;
        };
        let mut style = style;
        for (property, value) in overrides.get() {
            if let Ok(over) = devtools_apply::override_style(&property, &value) {
                style = style.merge(&over);
            }
        }
        style
    }

    /// Apply the node's overrides over `style`.
    #[cfg(not(feature = "devtools"))]
    pub(crate) fn apply(&self, style: Style) -> Style {
        style
    }

    /// The states a tool forces on the node.
    #[cfg(feature = "devtools")]
    pub(crate) fn forced(&self) -> ForcedStates {
        match self {
            StyleHook::Off => ForcedStates::default(),
            StyleHook::On { forced, .. } => forced.get(),
        }
    }

    /// The states a tool forces on the node.
    #[cfg(not(feature = "devtools"))]
    pub(crate) fn forced(&self) -> ForcedStates {
        ForcedStates::default()
    }
}
