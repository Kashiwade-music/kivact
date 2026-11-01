//! Role: Owns validated window semantics and portable accessibility acceptance.
//!
//! A rejected update retains the accepted rooted hierarchy, focus and text runs.
//! Backends convert current accepted values under PlatformTree's lock and notify
//! application code only after releasing it. No native accessibility type enters
//! this module; native and injected requests use one portable acceptance rule.

mod action;
pub(crate) mod error;
mod runs;
pub(crate) mod shared;
pub(crate) mod state;
mod validation;

use crate::events::ShellEvent;
use error::AccessibilityError;
use kivact_types::{A11yAction, SemanticsTree, SemanticsUpdate};
use shared::PlatformTree;

/// A window's validated semantics, updated transactionally and safe to share
/// between application and backend threads. Backend conversion uses the same
/// [`PlatformTree`] as this handle; change callbacks run after its lock is freed.
pub struct Accessibility {
    platform: PlatformTree,
    changed: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Default for Accessibility {
    fn default() -> Self {
        Self::new()
    }
}

impl Accessibility {
    /// An empty tree with no window or platform reader.
    pub fn new() -> Self {
        Self {
            platform: PlatformTree::new(1.0),
            changed: None,
        }
    }

    /// Attach to a backend's tree. `changed` notifies its window after successful
    /// changes, outside the tree lock; it may schedule publication on the UI
    /// thread. The backend retains a clone for activation and native requests.
    pub fn of_window(platform: PlatformTree, changed: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            platform,
            changed: Some(Box::new(changed)),
        }
    }

    /// Validate and replace a complete tree, then notify its backend. Rejection
    /// leaves the accepted hierarchy unchanged and sends no notification.
    pub fn replace(&self, tree: &SemanticsTree) -> Result<(), AccessibilityError> {
        self.platform.replace(tree)?;
        self.notify();
        Ok(())
    }

    /// Validate and apply a partial update. A new root requires a complete tree.
    /// Only changed accepted values notify the backend; rejection changes nothing.
    pub fn update(&self, update: &SemanticsUpdate) -> Result<(), AccessibilityError> {
        if self.platform.update(update)? {
            self.notify();
        }
        Ok(())
    }

    /// A complete copy of the accepted hierarchy, in parent-before-child order.
    pub fn tree(&self) -> Option<SemanticsTree> {
        self.platform.tree()
    }

    /// Check an assistive request against the accepted tree, returning the event
    /// the window delivers. Targets and ancestors must be shown and enabled.
    /// `Activate`, `Toggle` and `Select` are equivalent default declarations and
    /// arrive as `Activate`; focusability declares `Focus`. Every other action
    /// requires its exact declaration. This is also the native backend's rule.
    pub fn request_action(
        &self,
        node: u64,
        action: A11yAction,
        value: Option<&str>,
    ) -> Option<ShellEvent> {
        self.platform.request_action(node, action, value)
    }

    fn notify(&self) {
        if let Some(changed) = &self.changed {
            changed();
        }
    }
}
