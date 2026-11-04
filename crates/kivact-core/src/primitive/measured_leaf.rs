//! Role: The `Primitive` builder: a leaf node whose author measures its size.

use super::measurement::{Constraints, Measurement, constraints, sanitize};
use crate::control::node;
use crate::view::descriptor::Mcx;
use crate::view::descriptor::{mount, view_fn};
use crate::view::node::{Node, NodeBuilder};
use crate::view::state::ViewState;
use crate::view::sync_hook::{Current, SyncHook};
use crate::view::{IntoValue, IntoView, View};
use kivact_engine::api::PropValue;
use kivact_engine::contract::LeafMeasure;
use kivact_runtime::EffectKind;
use kivact_runtime::Prop as InputProp;
use kivact_types::{self as values, NodeId};
use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

type BindMeasure = Box<dyn FnOnce(&Mcx<'_>, NodeId, &'static Location<'static>)>;

/// **Experimental.** A new primitive element: a leaf box measured by its
/// author. Built by [`primitive`]; the standard modifiers (`draw`,
/// `on_pointer`, `role`, `test_id`, `style`, ...) come from
/// [`NodeBuilder`] and apply to its node. See the
/// [module](super) for what it supports.
pub struct Primitive {
    root: Node,
    measure: Option<BindMeasure>,
    at: &'static Location<'static>,
}

/// **Experimental.** Start a new primitive, located at the call (snapshots
/// and diagnostics name it by its location). Without
/// [`measure`](Primitive::measure) it is sized by its style and
/// `intrinsic_size` like an empty `node()`.
#[track_caller]
pub fn primitive() -> Primitive {
    Primitive {
        root: node(),
        measure: None,
        at: Location::caller(),
    }
}

impl Primitive {
    /// Size the primitive by `f(content, constraints)` (a [`Size`](crate::draw::Size), or a
    /// [`Measurement`] that adds a first baseline), which the layout
    /// calls synchronously, as it shapes a text. `content` (a constant, a
    /// `Signal`, a `Value` or a closure) holds everything the size depends
    /// on: the layout measures again only when a new content differs
    /// (`PartialEq`) from the last, and then only if the new sizes differ
    /// under the requests it measured before; otherwise it keeps every
    /// size it measured (it never measures every frame). The size is
    /// given in logical pixels; a known dimension of the constraints wins
    /// over it.
    ///
    /// `f` runs inside the layout: it must be a plain function of its
    /// arguments (nothing that waits). It runs pure and untracked: a signal
    /// write in it is rejected (a `WriteInPureContext` misuse panic) and a
    /// signal read subscribes nothing. A panic in it propagates through the
    /// layout and ends the App. A later call replaces an earlier one.
    pub fn measure<T, F, R>(mut self, content: impl IntoValue<T>, f: F) -> Self
    where
        T: Clone + PartialEq + 'static,
        R: Into<Measurement>,
        F: Fn(&T, Constraints<'_>) -> R + 'static,
    {
        let src = content.into_prop();
        let f = Rc::new(f);
        self.measure = Some(Box::new(move |cx, id, at| {
            bind_measure(cx, id, src, f, at);
        }));
        self
    }

    /// Toggle state for a check box or switch primitive (semantics and
    /// the `checked` reading of assistive technology).
    pub fn checked(mut self, v: impl IntoValue<bool>) -> Self {
        self.root = self.root.checked(v);
        self
    }

    /// Expanded state for a disclosure, combo box or menu primitive.
    pub fn expanded(mut self, v: impl IntoValue<bool>) -> Self {
        self.root = self.root.expanded(v);
        self
    }
}

/// Write the leaf's measure function for every new content of `src`: a
/// constant once (in the node's creation), anything else from one render
/// effect that skips a content equal to the last.
///
/// The engine calls the function inside its layout, with its tree
/// borrowed: a panic there propagates and ends the App. The current
/// content lives in the owner (until a new
/// content or its cleanup) and is always the one the engine holds when it
/// lays out: a new content is written in the turn that replaced it, and
/// the engine lays out only between turns.
fn bind_measure<T, F, R>(
    cx: &Mcx<'_>,
    id: NodeId,
    src: InputProp<T>,
    f: Rc<F>,
    at: &'static Location<'static>,
) where
    T: Clone + PartialEq + 'static,
    R: Into<Measurement>,
    F: Fn(&T, Constraints<'_>) -> R + 'static,
{
    let owner = cx.owner;
    let current: Current<T, Rc<F>> = Rc::new(RefCell::new(None));
    let held = current.clone();
    owner.on_cleanup(move || drop(held.borrow_mut().take()));
    let now = current.clone();
    let leaf = move |content: T| {
        let (hook, hooked) = SyncHook::new(owner, content, f.clone());
        *current.borrow_mut() = Some(hooked);
        LeafMeasure::new(move |request, text| {
            hook.call(|content, f| f(content, constraints(request, text)))
                .map_or_else(values::LeafSize::default, |measured| {
                    sanitize(measured.into())
                })
        })
    };
    if let Some(v) = src.fixed() {
        let measure = leaf(v);
        cx.update(id, PropValue::Measure(measure));
        return;
    }
    let weak = cx.weak();
    cx.effect(EffectKind::Render, at, move || {
        let v = src.get();
        // Compared with the content the engine measures (no second copy).
        let same = now.borrow().as_ref().is_some_and(|m| m.content == v);
        if same {
            return;
        }
        // The last content is dropped here, before the view tree is
        // borrowed: its `Drop` may read the tree.
        let prop = leaf(v);
        if let Some(rt) = weak.upgrade() {
            ViewState::tree_mut(&rt).update(id, PropValue::Measure(prop), rt.running_effect());
        }
    });
}

impl NodeBuilder for Primitive {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Primitive {
    fn into_view(self) -> View {
        view_fn(move |cx| {
            let Primitive { root, measure, at } = self;
            let id = mount(root.into_view(), cx);
            if let Some(bind) = measure {
                bind(cx, id, at);
            }
            id
        })
    }
}
