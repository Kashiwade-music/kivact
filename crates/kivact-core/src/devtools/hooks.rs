//! Role: The Inspector's answers to the view hooks: what DevTools records and shapes.

use std::panic::Location;
use std::rc::Rc;

use kivact_devtools_protocol as p;
use kivact_engine::api::Prop;
use kivact_engine::contract::HostEvent;
use kivact_runtime::NodeId as EffectId;
use kivact_runtime::{Handled, Owner, Runtime, Signal, Value};
use kivact_style::internal::ForcedStates;
use kivact_style::{Style, Theme};
use kivact_types::{Capabilities, FocusTarget, NodeId, WindowId};

use crate::i18n::Locale;
use crate::view::devtools_hooks::{DevtoolsHooks, NodeStyleHook};

use super::apply;
use super::state::{DevtoolsEvent, Inspector, Pending, PropWrite};

impl DevtoolsHooks for Inspector {
    fn prop_written(&self, id: NodeId, prop: Prop, writer: Option<EffectId>, changed: bool) {
        if !self.hooks.recording.borrow().updates {
            return;
        }
        self.hooks.prop_writes.borrow_mut().push(PropWrite {
            node: id,
            prop,
            effect: writer,
            changed,
        });
    }

    fn host_event(&self, event: &HostEvent) {
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
            self.session.events.borrow_mut().push(queued);
        }
    }

    fn input_event(&self, rt: &Runtime, event: &HostEvent, window: WindowId) {
        if !self.hooks.recording.borrow().input {
            return;
        }
        let Some((target, kind, detail)) = input_description(event) else {
            return;
        };
        let injected = self.session.pending.borrow().iter().any(|pending| {
            matches!(pending, Pending::Inject { window: pending_window, .. } if *pending_window == window)
        });
        let source = if injected {
            p::InputSource::Injected
        } else {
            p::InputSource::Platform
        };
        let seq = self.hooks.next_input.get() + 1;
        self.hooks.next_input.set(seq);
        let mut log = self.hooks.input_log.borrow_mut();
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

    fn handler_ran(&self, node: NodeId, event: &str, outcome: Option<Handled>) {
        if !self.hooks.recording.borrow().input {
            return;
        }
        let mut log = self.hooks.input_log.borrow_mut();
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

    fn focus_changed(
        &self,
        rt: &Runtime,
        window: WindowId,
        target: Option<FocusTarget>,
        visible: bool,
    ) {
        if !self.hooks.recording.borrow().input {
            return;
        }
        let mut history = self.hooks.focus_history.borrow_mut();
        let moves = history.entry(window).or_default();
        let before = moves.back().and_then(|entry| entry.after);
        let reason = match self.hooks.programmatic_focus.take() {
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

    fn focus_requested(&self, target: FocusTarget) {
        self.hooks.programmatic_focus.put(target);
    }

    fn focus_result(&self) {
        self.hooks.programmatic_focus.take();
    }

    fn wrap_theme(&self, owner: Owner, theme: Value<Theme>) -> Value<Theme> {
        let overrides = self.hooks.overrides;
        owner.with_setup(|| {
            kivact_runtime::memo(move || {
                apply::apply_token_overrides(theme.get(), &overrides.get())
            })
        })
    }

    fn wrap_locale(&self, owner: Owner, locale: Value<Locale>) -> Value<Locale> {
        let emulation = self.hooks.emulation;
        owner.with_setup(|| {
            kivact_runtime::memo(move || match emulation.get().locale {
                Some(tag) => Locale::new(&tag),
                None => locale.get(),
            })
        })
    }

    fn wrap_capabilities(&self, owner: Owner, host: Signal<Capabilities>) -> Value<Capabilities> {
        let emulation = self.hooks.emulation;
        owner.with_setup(|| {
            kivact_runtime::memo(move || {
                let mut capabilities = host.get();
                if let Some(touch) = emulation.get().touch {
                    capabilities.touch = touch;
                }
                capabilities
            })
        })
    }

    fn pseudo(&self, text: String) -> String {
        let mode = self.hooks.emulation.get().pseudo_locale;
        if mode == p::PseudoLocale::Off {
            return text;
        }
        apply::pseudo_locale(mode, &text)
    }

    fn style_hook(
        &self,
        rt: &Rc<Runtime>,
        owner: Owner,
        id: NodeId,
        at: &'static Location<'static>,
    ) -> Rc<dyn NodeStyleHook> {
        let forced = self.hooks.forced;
        Rc::new(NodeHook {
            id,
            sites: apply::call_sites(rt, owner, Some(at)),
            overrides: self.hooks.overrides,
            forced: owner.with_setup(|| {
                kivact_runtime::memo(move || {
                    forced.with(|all| all.get(&id).copied().unwrap_or_default())
                })
            }),
        })
    }
}

/// The DevTools style hook of one node.
struct NodeHook {
    id: NodeId,
    sites: Vec<&'static Location<'static>>,
    overrides: Signal<Vec<p::StyleOverride>>,
    forced: Value<ForcedStates>,
}

impl NodeStyleHook for NodeHook {
    fn apply(&self, style: Style) -> Style {
        let pairs = apply::node_overrides(&self.overrides.get(), self.id, &self.sites);
        let mut merged: Option<Style> = None;
        for (property, value) in pairs {
            if let Ok(over) = apply::override_style(&property, &value) {
                merged = Some(match merged {
                    Some(style) => style.merge(&over),
                    None => over,
                });
            }
        }
        match merged {
            Some(over) => style.merge(&over),
            None => style,
        }
    }

    fn forced(&self) -> ForcedStates {
        self.forced.get()
    }
}

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
