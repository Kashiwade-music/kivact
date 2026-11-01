//! Role: Carry out an assistive technology action on the node or region it targets.

use crate::model::Model;
use kivact_wire as wire;

impl Model {
    /// Carry out assistive technology's request on a node or canvas region,
    /// if the node (or region) offers it now. A node that is not exposed
    /// (no role) takes no request, for itself or its regions.
    pub(crate) fn accessibility_action(
        &mut self,
        native: u64,
        action: kivact_types::A11yAction,
        value: Option<String>,
    ) {
        use wire::A11yAction as A;
        let (id, region) = self
            .a11y
            .native_regions
            .get(&native)
            .copied()
            .map_or((wire::NodeId(native), None), |(id, region)| {
                (id, Some(region))
            });
        let Some(exposed) = self.exposed(id) else {
            return;
        };
        let offered = match region {
            Some(region) => self.region_actions(id, region),
            None => exposed.actions,
        };
        match action {
            A::Focus => {
                // Fails (keeping the focus) off a tab stop or outside the modal.
                let _ = self.request_focus(wire::FocusTarget { node: id, region }, true);
            }
            action @ (A::Activate | A::Toggle | A::Select) => {
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
                    self.push(wire::HostEvent::A11y {
                        node: id,
                        region,
                        action,
                        value,
                    });
                } else if offered.contains(&A::Activate) {
                    self.push(wire::HostEvent::Activate {
                        node: id,
                        region,
                        pointer: None,
                    });
                }
            }
            A::SetValue if region.is_none() && offered.contains(&A::SetValue) => {
                // Explicit spin actions normalize numbers through their writer;
                // ordinary native editors still receive text like typing.
                let input = self.node(id).is_some_and(|n| {
                    self.editors[n.index.0]
                        .as_deref()
                        .map(|editor| editor.editor.state())
                        .is_some()
                        && !(n.role() == Some(wire::Role::SpinButton)
                            && n.actions().contains(&A::SetValue))
                });
                match value {
                    // A text input's value is the host's: set like typing.
                    Some(v) if input => self.set_input_value(id, &v),
                    value => self.push(wire::HostEvent::A11y {
                        node: id,
                        region,
                        action: A::SetValue,
                        value,
                    }),
                }
            }
            action if offered.contains(&action) => self.push(wire::HostEvent::A11y {
                node: id,
                region,
                action,
                value,
            }),
            _ => {}
        }
        self.sync();
    }
}
