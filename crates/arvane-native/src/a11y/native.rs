//! Role: Native accessibility bridge.
//! Provider callbacks only read the tree, queue
//! actions and wake the UI thread.
//!
//! The window's tree ([`PlatformTree`]) answers activation with the whole tree
//! synchronously and checks actions against itself (the node and its
//! ancestors shown and enabled, the action declared). Changes reach
//! AccessKit through [`NativeAccessibility::publish`]: one update built,
//! inside AccessKit's update call, from the tree and what it has not been
//! handed, so an update never carries a node older than what activation
//! returned. The adapter must precede window visibility. AccessKit has a
//! single default action, so a native click is reported as `Activate`; the
//! receiver resolves it against the actions the node declared.

use kivact_types::{SemanticsTree, SemanticsUpdate};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use super::conversion;
use arvane_shell::{AccessibilityError, PlatformTree, PlatformUpdate, ShellEvent, ShellWaker};

struct Activation {
    shared: PlatformTree,
    waker: ShellWaker,
}

impl accesskit::ActivationHandler for Activation {
    fn request_initial_tree(&mut self) -> Option<accesskit::TreeUpdate> {
        let tree = self.shared.activate_with(convert_update);
        // The application builds the whole tree when woken: AccessKit shows
        // a placeholder until that first (complete) update, or, after a
        // deactivation, the tree accepted before it until the application
        // (which stopped keeping it meanwhile) replaces it.
        self.waker.wake();
        tree
    }
}

struct Actions {
    shared: PlatformTree,
    waker: ShellWaker,
}

impl accesskit::ActionHandler for Actions {
    fn do_action(&mut self, request: accesskit::ActionRequest) {
        let Some((node, action, value)) = translate_action(&request) else {
            return;
        };
        if self.shared.queue_action(node, action, value.as_deref()) {
            self.waker.wake();
        }
    }
}

struct Deactivation {
    shared: PlatformTree,
}

impl accesskit::DeactivationHandler for Deactivation {
    /// Assistive technology let go of the tree (AT-SPI): nothing is handed
    /// over until the next activation, which takes the whole tree, so the
    /// application stops building updates and nothing is remembered.
    fn deactivate_accessibility(&mut self) {
        self.shared.deactivate();
    }
}

pub(crate) struct NativeAccessibility {
    adapter: accesskit_winit::Adapter,
    shared: PlatformTree,
    /// Bounds follow the window's scale factor (a test window maps them
    /// with the scale its frames are drawn at instead).
    follows_window: bool,
}

impl NativeAccessibility {
    pub(crate) fn new(event_loop: &ActiveEventLoop, window: &Window, waker: ShellWaker) -> Self {
        Self::create(event_loop, window, waker, window.scale_factor(), true)
    }

    /// A bridge whose logical bounds are mapped to the window's pixels with
    /// `scale`, whatever the window's own scale factor.
    #[cfg(feature = "testing")]
    pub(crate) fn with_scale(
        event_loop: &ActiveEventLoop,
        window: &Window,
        waker: ShellWaker,
        scale: f64,
    ) -> Self {
        Self::create(event_loop, window, waker, scale, false)
    }

    fn create(
        event_loop: &ActiveEventLoop,
        window: &Window,
        waker: ShellWaker,
        scale: f64,
        follows_window: bool,
    ) -> Self {
        let shared = PlatformTree::new(scale);
        let adapter = accesskit_winit::Adapter::with_direct_handlers(
            event_loop,
            window,
            Activation {
                shared: shared.clone(),
                waker: waker.clone(),
            },
            Actions {
                shared: shared.clone(),
                waker,
            },
            Deactivation {
                shared: shared.clone(),
            },
        );
        Self {
            adapter,
            shared,
            follows_window,
        }
    }

    /// The window's tree, for a handle on another thread.
    #[cfg(feature = "testing")]
    pub(crate) fn platform_tree(&self) -> PlatformTree {
        self.shared.clone()
    }

    pub(crate) fn process_event(&mut self, window: &Window, event: &WindowEvent) {
        self.adapter.process_event(window, event);
        if let WindowEvent::ScaleFactorChanged { scale_factor, .. } = event
            && self.follows_window
        {
            self.rescale(*scale_factor);
        }
    }

    /// Map bounds with `scale` from now on, the published tree included.
    pub(crate) fn rescale(&mut self, scale: f64) {
        self.shared.rescale(scale);
        self.hand_over(true);
    }

    /// Validate a complete tree, replace the accepted one with it and hand
    /// it to the platform. A rejected tree keeps the accepted one.
    pub(crate) fn replace(&mut self, tree: &SemanticsTree) -> Result<(), AccessibilityError> {
        self.shared.replace(tree)?;
        self.publish();
        Ok(())
    }

    /// Validate and apply the nodes that may have changed and hand those
    /// that did to the platform. A rejected update keeps the accepted tree.
    pub(crate) fn update(&mut self, update: &SemanticsUpdate) -> Result<(), AccessibilityError> {
        if self.shared.update(update)? {
            self.publish();
        }
        Ok(())
    }

    /// Hand the platform what changed since it was last handed the tree.
    pub(crate) fn publish(&mut self) {
        self.hand_over(false);
    }

    /// Hand over what the platform lacks (and the root, rescaled, when
    /// `root`). Built inside AccessKit's update call (only while active):
    /// activation and updates are serialized there, and an activation
    /// meanwhile already returned the whole tree.
    fn hand_over(&mut self, root: bool) {
        if !self.shared.has_pending(root) {
            return;
        }
        let shared = &self.shared;
        self.adapter.update_if_active(|| {
            shared
                .pending_with(root, convert_update)
                .expect("invariant: an active native publication retains its accepted tree")
        });
    }

    /// Whether assistive technology reads the tree (it asked for it and has
    /// not let go of it): otherwise an application need not build it.
    pub(crate) fn active(&self) -> bool {
        self.shared.active()
    }

    pub(crate) fn drain_actions(&mut self) -> Vec<ShellEvent> {
        self.shared.drain_actions()
    }

    /// Forget the accepted tree (a pooled test window handed to a new
    /// test): the platform keeps showing what it was handed until the next
    /// tree, which is complete.
    #[cfg(feature = "testing")]
    pub(crate) fn forget(&mut self) {
        self.shared.forget();
    }
}

/// Convert a borrowed current selection before releasing the shell tree lock.
pub(crate) fn convert_update(selection: PlatformUpdate<'_>) -> accesskit::TreeUpdate {
    let tree = selection.tree();
    let mut update = conversion::update(
        tree.root(),
        tree.focus(),
        selection.nodes(),
        selection.all_runs(),
    );
    if !selection.all_runs() {
        update
            .nodes
            .extend(selection.changed_runs().map(conversion::text_run));
    }
    scale_root(&mut update, selection.scale());
    update
}

/// Bounds are logical: the root (when the update carries it) maps them to
/// the window's pixels with `scale`, after its own transform.
fn scale_root(update: &mut accesskit::TreeUpdate, scale: f64) {
    if let Some(tree) = &update.tree
        && let Some((_, root)) = update.nodes.iter_mut().find(|(id, _)| *id == tree.root)
    {
        let own = root.transform().copied().unwrap_or_default();
        root.set_transform(accesskit::Affine::scale(scale) * own);
    }
}

/// Decode native vocabulary only; the shell validates and queues the request.
pub(crate) fn translate_action(
    request: &accesskit::ActionRequest,
) -> Option<(u64, kivact_types::A11yAction, Option<String>)> {
    if request.target_tree != accesskit::TreeId::ROOT {
        return None;
    }
    let action = conversion::action(request)?;
    let value = match &request.data {
        Some(accesskit::ActionData::Value(value)) => Some(value.to_string()),
        Some(accesskit::ActionData::NumericValue(value)) if value.is_finite() => {
            Some(value.to_string())
        }
        Some(accesskit::ActionData::NumericValue(_)) => return None,
        _ => None,
    };
    Some((request.target_node.0, action, value))
}
