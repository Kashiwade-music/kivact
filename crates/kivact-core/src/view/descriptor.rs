//! Role: Unmounted descriptions.
//! Builders implement [`IntoView`]; nothing is
//! created until the description is mounted under an owner.

use std::panic::Location;
use std::rc::{Rc, Weak};

use kivact_engine::api::PropValue;
use kivact_engine::contract::NodeType;
use kivact_runtime::Prop as InputProp;
use kivact_runtime::{EffectKind, NodeId as EffectId, Owner, Runtime, Value};
use kivact_types::NodeId;

use super::state::ViewState;
use kivact_style::Theme;

/// Mount context: the runtime and the owner new nodes belong to.
pub(crate) struct Mcx<'a> {
    /// The runtime of the App.
    pub rt: &'a Rc<Runtime>,
    /// The owner new nodes and effects belong to.
    pub owner: Owner,
}

impl Mcx<'_> {
    /// A new fragment node.
    pub(crate) fn fragment(&self) -> NodeId {
        ViewState::tree_mut(self.rt).create(None, self.owner)
    }

    /// A new host node of `kind`.
    pub(crate) fn host(&self, kind: NodeType) -> NodeId {
        ViewState::tree_mut(self.rt).create(Some(kind), self.owner)
    }

    /// Append `child` to `parent`.
    pub(crate) fn append(&self, parent: NodeId, child: NodeId) {
        ViewState::tree_mut(self.rt).insert(parent, child, None);
    }

    /// Write `value` to node `id`.
    pub(crate) fn update(&self, id: NodeId, value: PropValue) {
        let writer = self.rt.running_effect();
        ViewState::tree_mut(self.rt).update(id, value, writer);
    }

    /// A weak handle of the runtime (for effects and handlers).
    pub(crate) fn weak(&self) -> Weak<Runtime> {
        Rc::downgrade(self.rt)
    }

    /// Create an effect of `kind` under this context's owner.
    pub(crate) fn effect(
        &self,
        kind: EffectKind,
        at: &'static Location<'static>,
        f: impl Fn() + 'static,
    ) -> EffectId {
        self.owner.effect(kind, f, at)
    }

    /// Bind a source to one typed property: `value` builds the property's
    /// value from the source's. Constant values are set once; dynamic
    /// values get a render effect.
    pub(crate) fn bind<T: Clone + 'static>(
        &self,
        id: NodeId,
        src: InputProp<T>,
        value: fn(T) -> PropValue,
        at: &'static Location<'static>,
    ) {
        if let Some(v) = src.fixed() {
            self.update(id, value(v));
            return;
        }
        let weak = self.weak();
        self.effect(EffectKind::Render, at, move || {
            let v = src.get();
            if let Some(rt) = weak.upgrade() {
                let writer = rt.running_effect();
                ViewState::tree_mut(&rt).update(id, value(v), writer);
            }
        });
    }

    /// Mount `view` in a fresh setup frame of this context's owner.
    pub(crate) fn mount_child(&self, view: View) -> NodeId {
        self.owner.with_setup(|| mount(view, self))
    }

    /// The theme visible from this owner.
    pub(crate) fn theme(&self) -> Option<Value<Theme>> {
        Theme::of(self.owner)
    }
}

type MountFn = Box<dyn FnOnce(&Mcx<'_>) -> NodeId>;

/// A type-erased, unmounted view. Mounting it creates nodes and effects
/// exactly once.
pub struct View {
    mount: Option<MountFn>,
}

impl View {
    fn new(f: impl FnOnce(&Mcx<'_>) -> NodeId + 'static) -> Self {
        View {
            mount: Some(Box::new(f)),
        }
    }

    /// A view that renders nothing.
    pub fn empty() -> Self {
        View { mount: None }
    }

    /// Whether this view is [`View::empty`], such as what `()` becomes.
    pub(crate) fn is_empty(&self) -> bool {
        self.mount.is_none()
    }

    fn mount(self, cx: &Mcx<'_>) -> NodeId {
        match self.mount {
            Some(f) => f(cx),
            None => cx.fragment(),
        }
    }
}

/// A view that runs `f` when it is mounted (the constructor every builder
/// uses in its `IntoView` impl).
pub(crate) fn view_fn(f: impl FnOnce(&Mcx<'_>) -> NodeId + 'static) -> View {
    View::new(f)
}

/// Mount `v` under the owner of `cx`; returns its host node (a fragment for
/// an empty view).
pub(crate) fn mount(v: View, cx: &Mcx<'_>) -> NodeId {
    v.mount(cx)
}

/// Anything that can be placed in the view tree: builders, `View`, strings
/// (as text), tuples (up to 16, mixed types), `Vec<V>` and `Option<V>`.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be shown as a view",
    label = "not a view",
    note = "element functions (`text`, `column`, `node`, ..), `String`/`&str`, tuples of up to 16 views, `Vec`/`Option` of views and components returning `impl IntoView` are views"
)]
pub trait IntoView: 'static {
    fn into_view(self) -> View;
}

impl IntoView for View {
    fn into_view(self) -> View {
        self
    }
}

impl IntoView for () {
    fn into_view(self) -> View {
        View::empty()
    }
}

impl<V: IntoView> IntoView for Option<V> {
    fn into_view(self) -> View {
        match self {
            Some(v) => v.into_view(),
            None => View::empty(),
        }
    }
}

impl<V: IntoView> IntoView for Vec<V> {
    fn into_view(self) -> View {
        View::new(move |cx| {
            let frag = cx.fragment();
            for v in self {
                let child = v.into_view().mount(cx);
                cx.append(frag, child);
            }
            frag
        })
    }
}

macro_rules! tuple_into_view {
    ($($name:ident),+) => {
        // Not recommended in diagnostics: the note of `IntoView` says tuples
        // are views, instead of sixteen impls being listed.
        #[diagnostic::do_not_recommend]
        impl<$($name: IntoView),+> IntoView for ($($name,)+) {
            #[allow(non_snake_case, reason = "tuple element bindings reuse the type parameter names")]
            fn into_view(self) -> View {
                let ($($name,)+) = self;
                View::new(move |cx| {
                    let frag = cx.fragment();
                    $(
                        let child = $name.into_view().mount(cx);
                        cx.append(frag, child);
                    )+
                    frag
                })
            }
        }
    };
}

tuple_into_view!(A);
tuple_into_view!(A, B);
tuple_into_view!(A, B, C);
tuple_into_view!(A, B, C, D);
tuple_into_view!(A, B, C, D, E);
tuple_into_view!(A, B, C, D, E, F);
tuple_into_view!(A, B, C, D, E, F, G);
tuple_into_view!(A, B, C, D, E, F, G, H);
tuple_into_view!(A, B, C, D, E, F, G, H, I);
tuple_into_view!(A, B, C, D, E, F, G, H, I, J);
tuple_into_view!(A, B, C, D, E, F, G, H, I, J, K);
tuple_into_view!(A, B, C, D, E, F, G, H, I, J, K, L);
tuple_into_view!(A, B, C, D, E, F, G, H, I, J, K, L, M);
tuple_into_view!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
tuple_into_view!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
tuple_into_view!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

/// Whether the view is explicitly empty, including `()`.
#[track_caller]
pub fn view_is_empty(view: &View) -> bool {
    view.is_empty()
}
