//! Role: One target and reason policy for returning focus when a surface closes.
//!
//! A popup owns a [`FocusReturn`]; menu, dialog popover and the scalar thumb
//! only say where the focus goes and for which close reasons. The core still
//! decides whether the target may take the focus (liveness, hiding, the open
//! modal): a request submits `restore_focus()` and nothing else.

use kivact_core::primitive::NodeToken;
use kivact_core::view::NodeRef;

/// Why an application closes a popup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PopupCloseReason {
    Escape,
    Selection,
    Accessibility,
    OutsidePress,
    /// Focus moved to a concrete target outside content and the exact anchor.
    Tab,
    AnchorRemoved,
    Programmatic,
    /// The mounted surface is removed without an accepted close request.
    Unmount,
}

/// Where the focus goes when a surface closes.
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum FocusReturnTarget {
    /// The node that had the focus when the surface mounted. When nothing had
    /// it, the popup's anchor.
    Opener,
    /// This reference, resolved when the focus is restored.
    Anchor(NodeRef),
}

/// A set of close reasons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReasonSet(u16);

impl ReasonSet {
    /// No reason: the focus is never restored.
    pub const NONE: Self = Self(0);
    /// Escape, a selection and assistive technology's dismissal.
    pub const DISMISS: Self = Self::one(PopupCloseReason::Escape)
        .union(Self::one(PopupCloseReason::Selection))
        .union(Self::one(PopupCloseReason::Accessibility));
    /// An accepted explicit close by the application.
    pub const PROGRAMMATIC: Self = Self::one(PopupCloseReason::Programmatic);
    /// A surface removed without an accepted close request.
    pub const UNMOUNT: Self = Self::one(PopupCloseReason::Unmount);

    /// The bit of `reason`, so that the set follows the enum's variants.
    const fn bit(reason: PopupCloseReason) -> u16 {
        1 << reason as u16
    }

    /// The reasons of both sets.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// The set of exactly `reason`.
    pub const fn one(reason: PopupCloseReason) -> Self {
        Self(Self::bit(reason))
    }

    /// Whether `reason` is in the set.
    pub const fn contains(self, reason: PopupCloseReason) -> bool {
        self.0 & Self::bit(reason) != 0
    }
}

/// The target of a surface's focus return and the reasons that use it.
///
/// An outside press and Tab keep the destination unless `on` names them. The
/// core decides, when the request is sent, whether the target can take the
/// focus.
/// Built with [`FocusReturn::new`].
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct FocusReturn {
    /// Where the focus is restored to.
    pub target: FocusReturnTarget,
    /// The close reasons that restore it.
    pub on: ReasonSet,
}

impl FocusReturn {
    /// Restore `node` on Escape, a selection and assistive technology's
    /// dismissal.
    pub fn anchor(node: NodeRef) -> Self {
        Self {
            target: FocusReturnTarget::Anchor(node),
            on: ReasonSet::DISMISS,
        }
    }

    /// Restore the node that had the focus before the surface mounted, on `on`.
    pub fn opener(on: ReasonSet) -> Self {
        Self {
            target: FocusReturnTarget::Opener,
            on,
        }
    }

    /// Never restore: every close keeps the destination.
    pub fn preserve() -> Self {
        Self::opener(ReasonSet::NONE)
    }

    /// Whether a close for `reason` restores.
    pub(crate) fn restores(self, reason: PopupCloseReason) -> bool {
        self.on.contains(reason)
    }

    /// Submit the restoration for `reason`, if the policy admits it.
    /// `opener` is the node the popup captured, already replaced by the
    /// anchor when nothing had the focus. Whether the target may take the
    /// focus is the core's decision, not checked here.
    pub(crate) fn request(self, reason: PopupCloseReason, opener: Option<NodeToken>) {
        if !self.restores(reason) {
            return;
        }
        match self.target {
            FocusReturnTarget::Opener => {
                if let Some(node) = opener {
                    node.restore_focus();
                }
            }
            FocusReturnTarget::Anchor(reference) => reference.restore_focus(),
        }
    }
}
