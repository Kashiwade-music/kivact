//! Role: Mounts a drawable's root: its content, its role and its binding.
//!
//! The root is an absolutely positioned child at (0, 0) of the node that
//! mounts it (`Node::drawable`, `Viewport::drawable`). The binding is
//! checked before anything is mounted: a refused mount reports once and
//! leaves an empty fragment in its place.

use std::panic::Location;

use kivact_engine::api::PropValue;
use kivact_engine::contract::{DrawableMode, DrawableRole};
use kivact_style::{Position, style};

use super::handle::Drawable;
use crate::view::descriptor::{Mcx, ViewFn, mount, view_fn};
use crate::view::element_data::Node;
use crate::view::elements::builders::node;
use crate::view::state::ViewState;
use crate::view::{IntoView, IntoViewFn, View, Viewport};

impl Node {
    /// Mount `content` as a drawable of this node: an absolutely positioned
    /// child at (0, 0) of this node's padding box (its own style may move or
    /// size it). The content is drawn only where this node's `draw` or
    /// `draw_over` scene calls `Draw::drawable`.
    #[track_caller]
    pub fn drawable<V: IntoView>(self, drawable: Drawable, content: impl IntoViewFn<V>) -> Self {
        let at = Location::caller();
        self.child(root_view(
            drawable,
            content.into_view_fn(),
            DrawableMode::Canvas,
            at,
        ))
    }
}

impl Viewport {
    /// Mount `content` as an exported drawable: laid out like `Node::drawable`,
    /// never drawn on screen, rendered for `Drawable::share`.
    #[track_caller]
    pub fn drawable<V: IntoView>(
        mut self,
        drawable: Drawable,
        content: impl IntoViewFn<V>,
    ) -> Self {
        let at = Location::caller();
        self.root = self.root.child(root_view(
            drawable,
            content.into_view_fn(),
            DrawableMode::Export,
            at,
        ));
        self
    }
}

/// The view that mounts `content` as the root of `drawable`, shown in `mode`.
fn root_view(
    drawable: Drawable,
    content: ViewFn<()>,
    mode: DrawableMode,
    at: &'static Location<'static>,
) -> View {
    view_fn(move |cx: &Mcx<'_>| mount_root(drawable, &content, mode, at, cx))
}

/// Mount the root under `cx` (after the binding checks) and write its role.
fn mount_root(
    drawable: Drawable,
    content: &ViewFn<()>,
    mode: DrawableMode,
    at: &'static Location<'static>,
    cx: &Mcx<'_>,
) -> kivact_types::NodeId {
    if !drawable.can_mount(cx.rt, at) {
        return cx.fragment();
    }
    let root = node()
        .style(style().position(Position::Absolute).top(0).left(0))
        .node_ref(drawable.root_ref())
        .child(content.call(()));
    let id = mount(root.into_view(), cx);
    let writer = cx.rt.running_effect();
    ViewState::tree_mut(cx.rt).update(
        id,
        PropValue::Drawable(Some(DrawableRole {
            id: drawable.id(),
            mode,
        })),
        writer,
    );
    let owner = ViewState::tree(cx.rt)
        .get(id)
        .map_or(cx.owner, |node| node.owner);
    drawable.bind(id, owner);
    id
}
