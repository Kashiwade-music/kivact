//! Role: The modal group's state: admitted pointer drags and adopted gestures.
//!
//! The drags admitted before a modal opened may finish outside it; the
//! gestures the client adopted decline their pointer's click.

use std::cell::RefCell;

use kivact_wire::{MouseButton, NodeId, PointerId, RegionId};

/// Admission metadata only: the node whose `Down` a pointer's drag began
/// at and the buttons it still holds. Pointer capture itself stays with
/// the pointer group.
#[derive(Clone, Copy)]
struct AdmittedDrag {
    pointer: PointerId,
    node: NodeId,
    buttons: u8,
}

fn button_bit(button: MouseButton) -> u8 {
    match button {
        MouseButton::Primary => 1,
        MouseButton::Secondary => 2,
        MouseButton::Middle => 4,
        MouseButton::Back => 8,
        MouseButton::Forward => 16,
    }
}

/// A pointer's primary gesture that a canvas region took over: its click
/// activates nothing at that region.
#[derive(Clone, Copy)]
struct Adoption {
    pointer: PointerId,
    node: NodeId,
    region: RegionId,
}

/// The drags that may finish outside a modal opened after they began, one
/// per pointer, and the gestures the client adopted, one per pointer. Both
/// follow the order the events are dispatched in (admission is asked right
/// before each one), not the order the engine produced them. Interior
/// mutability: admission is asked of a shared model
/// (`Reader::admit_input`), and every call borrows the table for its own
/// duration only, never across a callback.
#[derive(Default)]
pub(crate) struct Admission {
    drags: RefCell<Vec<AdmittedDrag>>,
    adoptions: RefCell<Vec<Adoption>>,
}

impl Admission {
    /// End the admissions of nodes `live` does not accept (removed or
    /// leaving): the engine sends no `Cancel` to a node that went away
    /// while it held a pointer.
    pub(crate) fn forget_missing(&self, live: impl Fn(NodeId) -> bool) {
        self.drags.borrow_mut().retain(|drag| live(drag.node));
        self.adoptions
            .borrow_mut()
            .retain(|adoption| live(adoption.node));
    }

    /// Record that `pointer`'s current gesture was adopted at `region` of
    /// `node`, replacing the pointer's earlier adoption.
    pub(crate) fn adopt(&self, pointer: PointerId, node: NodeId, region: RegionId) {
        let mut adoptions = self.adoptions.borrow_mut();
        adoptions.retain(|adoption| adoption.pointer != pointer);
        adoptions.push(Adoption {
            pointer,
            node,
            region,
        });
    }

    /// End the adoption of `pointer`, if any; whether it covers the
    /// activation of `region` of `node`. An activation of the node's own
    /// action (no region) is never covered.
    pub(crate) fn take_adoption(
        &self,
        pointer: PointerId,
        node: NodeId,
        region: Option<RegionId>,
    ) -> bool {
        let mut adoptions = self.adoptions.borrow_mut();
        let Some(at) = adoptions
            .iter()
            .position(|adoption| adoption.pointer == pointer)
        else {
            return false;
        };
        let adoption = adoptions.remove(at);
        adoption.node == node && region == Some(adoption.region)
    }

    /// End the adoption of `pointer`, if any.
    pub(crate) fn forget_adoption(&self, pointer: PointerId) {
        self.adoptions
            .borrow_mut()
            .retain(|adoption| adoption.pointer != pointer);
    }

    /// The node that got the `Down` of `pointer`'s drag in progress.
    pub(crate) fn drag(&self, pointer: PointerId) -> Option<NodeId> {
        self.drags
            .borrow()
            .iter()
            .find(|drag| drag.pointer == pointer)
            .map(|drag| drag.node)
    }

    /// Admit a button press of `pointer` at `node`, retaining the other
    /// buttons that listener already holds.
    pub(crate) fn press(&self, pointer: PointerId, node: NodeId, button: MouseButton) {
        let mut drags = self.drags.borrow_mut();
        let at = drags.iter().position(|drag| drag.pointer == pointer);
        let mut drag = at
            .map(|at| drags.remove(at))
            .filter(|drag| drag.node == node)
            .unwrap_or(AdmittedDrag {
                pointer,
                node,
                buttons: 0,
            });
        drag.buttons |= button_bit(button);
        drags.push(drag);
    }

    /// Release only this listener's `button` of `pointer`; its final
    /// release ends the admission (`None`: every button).
    pub(crate) fn release(&self, pointer: PointerId, node: NodeId, button: Option<MouseButton>) {
        let mut drags = self.drags.borrow_mut();
        let Some(at) = drags
            .iter()
            .position(|drag| drag.pointer == pointer && drag.node == node)
        else {
            return;
        };
        let drag = &mut drags[at];
        drag.buttons = button.map_or(0, |button| drag.buttons & !button_bit(button));
        if drag.buttons == 0 {
            drags.remove(at);
        }
    }

    /// End this listener's admission of `pointer` (the capture was
    /// cancelled).
    pub(crate) fn cancel(&self, pointer: PointerId, node: NodeId) {
        self.drags
            .borrow_mut()
            .retain(|drag| drag.pointer != pointer || drag.node != node);
    }
}
