//! Role: Carry out an assistive technology action on the node or region it targets.

use crate::model::Model;
use kivact_types as types;

impl Model {
    /// Carry out assistive technology's request on a node or canvas region,
    /// if the node (or region) offers it now. A node that is not exposed
    /// (no role) takes no request, for itself or its regions.
    pub(crate) fn accessibility_action(&mut self, native: u64, request: kivact_types::A11yRequest) {
        use types::A11yAction as A;
        let (id, region) = self
            .a11y
            .native_regions
            .get(&native)
            .copied()
            .map_or((types::NodeId(native), None), |(id, region)| {
                (id, Some(region))
            });
        let Some(exposed) = self.exposed(id) else {
            return;
        };
        let offered = match region {
            Some(region) => self.region_actions(id, region),
            None => exposed.actions,
        };
        match request {
            types::A11yRequest::Focus => {
                // Fails (keeping the focus) off a tab stop or outside the modal.
                let _ = self.request_focus(types::FocusTarget { node: id, region }, true);
            }
            types::A11yRequest::Activate
            | types::A11yRequest::Toggle
            | types::A11yRequest::Select => {
                let action = request.action();
                // AccessKit exposes Activate/Toggle/Select as Click and sends
                // Activate back. Prefer an offered Activate for legacy controls;
                // otherwise recover the separately offered semantic action.
                let explicit = if action != A::Activate && offered.contains(&action) {
                    Some(action)
                } else if !offered.contains(&A::Activate) {
                    [A::Toggle, A::Select]
                        .into_iter()
                        .find(|a| offered.contains(a))
                } else {
                    None
                };
                if let Some(action) = explicit {
                    self.push(crate::contract::HostEvent::A11y {
                        node: id,
                        region,
                        request: types::A11yRequest::from_action(action)
                            .expect("invariant: Toggle and Select carry no data"),
                    });
                } else if offered.contains(&A::Activate) {
                    self.push(crate::contract::HostEvent::Activate {
                        node: id,
                        region,
                        pointer: None,
                    });
                }
            }
            types::A11yRequest::SetValue(value)
                if region.is_none() && offered.contains(&A::SetValue) =>
            {
                // Explicit spin actions normalize numbers through their writer;
                // ordinary native editors still receive text like typing.
                let input = self.node(id).is_some_and(|n| {
                    self.editors[n.index.0]
                        .as_deref()
                        .map(|editor| editor.editor.state())
                        .is_some()
                        && !(n.role() == Some(types::Role::SpinButton)
                            && n.actions().contains(&A::SetValue))
                });
                if input {
                    // A text input's value is the host's: set like typing.
                    self.set_input_value(id, &value);
                } else {
                    self.push(crate::contract::HostEvent::A11y {
                        node: id,
                        region,
                        request: types::A11yRequest::SetValue(value),
                    });
                }
            }
            request if offered.contains(&request.action()) => {
                self.push(crate::contract::HostEvent::A11y {
                    node: id,
                    region,
                    request,
                });
            }
            _ => {}
        }
        self.sync();
    }
}
