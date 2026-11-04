//! Role: Nodes expose shared element builders and their modifier contract.
//!
//! Every element function (`node()`, the containers, `text`, `rich_text`,
//! `image`, `canvas`) returns a `Node`, so the arms of a `match` that build
//! different elements have one type without `.into_view()` (an arm that
//! builds a component such as `button` needs `.into_view()` on every arm).
//! A component's `#[track_caller]` constructor sets its defaults (role,
//! focusability, base style) first, so the caller's modifiers win. The
//! standard components
//! (`kivact::ui`) and those of the official UI kits hold a root `Node` and
//! implement [`NodeBuilder`] like any external component — role, focus,
//! key/pointer/a11y input, draw layers, regions and size hints are all
//! public.
//!
//! Invariant: what a node is (a box, a text, an image) is fixed by the
//! function that made it. A setting that only one kind of node can show
//! (`child`, `draw`, `fit`, ...) given to another kind is reported once at
//! mount as `ErrorKind::Unsupported`, located at the node, and dropped; it
//! never reaches the host.

use std::panic::Location;

use kivact_engine::api::PropValue;
use kivact_engine::contract::NodeType;

use crate::view::common::Common;
use crate::view::descriptor::Mcx;
use crate::view::descriptor::{mount, view_fn};
use crate::view::mount::mount_host;
use crate::view::node::{Content, Node, NodeBuilder};
use crate::view::state::ViewState;
use crate::view::{IntoText, IntoValue, IntoView, Role, View};
use kivact_runtime::Prop as InputProp;
use kivact_runtime::{ErrorKind, UiError};
use kivact_style::asset::Asset;

/// Start a custom element: an empty box, located at the call of `node()`
/// (snapshots and diagnostics name nodes by their location).
#[track_caller]
pub fn node() -> Node {
    Node::new(Content::Box, Some(Location::caller()))
}

impl Node {
    /// The accessible name set so far, if any (a menu names its panel after
    /// the trigger it opens from).
    pub(crate) fn name_source(&self) -> Option<InputProp<String>> {
        self.common.name.clone()
    }

    /// The role set so far, if any.
    pub(crate) fn role_source(&self) -> Option<crate::view::Role> {
        self.common.role
    }

    /// The expanded state set so far, if any (a button that a popup or a
    /// disclosure attached its trigger to).
    pub(crate) fn expanded_source(&self) -> Option<InputProp<bool>> {
        self.common.expanded.clone()
    }

    /// The accessible description set so far, if any (a tooltip fills in its
    /// label only when the trigger has none of its own).
    pub(crate) fn description_source(&self) -> Option<InputProp<String>> {
        self.common.description.clone()
    }

    /// Preserved label relations after caller and part sources are merged.
    pub(crate) fn labelled_by_sources(&self) -> Vec<crate::view::NodeRef> {
        self.common
            .semantics
            .as_ref()
            .map(|sources| sources.labelled_by.clone())
            .unwrap_or_default()
    }

    /// Preserved description relations after caller and part sources are merged.
    pub(crate) fn described_by_sources(&self) -> Vec<crate::view::NodeRef> {
        self.common
            .semantics
            .as_ref()
            .map(|sources| sources.described_by.clone())
            .unwrap_or_default()
    }

    /// The disabled state the caller set so far, if any.
    pub(crate) fn disabled_source(&self) -> Option<InputProp<bool>> {
        self.common.disabled.clone()
    }

    /// The invalid state set so far, if any (a text input or a switch draws
    /// itself as invalid from what the caller or a form field set on it).
    pub(crate) fn invalid_source(&self) -> Option<InputProp<bool>> {
        self.common.invalid.clone()
    }

    /// Declare behavior for a canvas region before painting its geometry.
    ///
    /// Each identifier may be declared once per node. Geometry is supplied by
    /// `Draw::region`, `Draw::region_shape` or `Draw::region_path`; a declaration
    /// without geometry does not create a native focus or accessibility target.
    pub fn region<S: Clone + PartialEq + 'static>(
        mut self,
        id: kivact_style::draw::RegionId,
        part: crate::view::Part<S>,
    ) -> Self {
        let mut region = Common::default();
        part.attach(&mut region);
        self.common
            .regions
            .push(crate::view::common::Declaration::Static(
                id,
                Box::new(region),
            ));
        self
    }

    /// Declare a keyed collection of canvas parts without replacing the canvas.
    ///
    /// Each key keeps its item setup owner and part while present. Value updates
    /// reach the existing part; reordering changes focus order without rebinding.
    /// Removing a key disposes its state, refs and input callbacks. Declarations
    /// share one identifier namespace with `region`, in builder call order.
    #[track_caller]
    pub fn regions<T: Clone + PartialEq + 'static, S: Clone + PartialEq + 'static>(
        mut self,
        items: impl IntoValue<Vec<T>>,
        key: impl Fn(&T) -> kivact_style::draw::RegionId + 'static,
        make: impl Fn(kivact_style::draw::RegionId, crate::Value<T>) -> crate::view::Part<S> + 'static,
    ) -> Self {
        self.common
            .regions
            .push(crate::view::mount::region_collection::declaration(
                items.into_prop(),
                key,
                move |id, item| {
                    let mut common = Common::default();
                    make(id, item).attach(&mut common);
                    common
                },
                Location::caller(),
            ));
        self
    }

    /// Attach behavior, semantic relationships and input defaults to this
    /// element.
    ///
    /// Same as [`NodeExt::part`](crate::primitive::NodeExt::part).
    pub fn part<S: Clone + PartialEq + 'static>(mut self, part: crate::view::Part<S>) -> Self {
        part.attach(&mut self.common);
        self
    }

    /// Accessibility role, overriding the function's default role.
    ///
    /// Same as [`NodeExt::role`](crate::primitive::NodeExt::role).
    pub fn role(mut self, r: Role) -> Self {
        self.common.role = Some(r);
        self
    }

    /// Accessible value text (e.g. of a slider or progress bar).
    ///
    /// Same as [`NodeExt::value_text`](crate::primitive::NodeExt::value_text).
    pub fn value_text(mut self, v: impl IntoText) -> Self {
        self.common.value = Some(v.into_text());
        self
    }

    /// Selected state (semantics and the `selected` style).
    ///
    /// Same as [`NodeExt::selected`](crate::primitive::NodeExt::selected).
    pub fn selected(mut self, v: impl IntoValue<bool>) -> Self {
        self.common.selected = Some(v.into_prop());
        self
    }

    /// Binary checked state for a check box or switch.
    ///
    /// Same as [`NodeExt::checked`](crate::primitive::NodeExt::checked).
    pub fn checked(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.checked = Some(value.into_prop());
        self
    }

    /// Expanded state of a disclosure, menu or combo box.
    ///
    /// Same as [`NodeExt::expanded`](crate::primitive::NodeExt::expanded).
    pub fn expanded(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.expanded = Some(value.into_prop());
        self
    }

    /// Append a child. Only a box has children: on a text or an image the
    /// child is not mounted and the misuse is reported once at mount
    /// (`ErrorKind::Unsupported`).
    pub fn child(mut self, v: impl IntoView) -> Self {
        self.children.push(v.into_view());
        self
    }

    /// How an image fills the node's box, as CSS `object-fit`
    /// (`ImageFit::Contain` by default). The image is centered; where it
    /// overflows (`Cover`, `None`), the box clips it. Image nodes only: on
    /// another node it is reported once at mount (`ErrorKind::Unsupported`).
    pub fn fit(mut self, f: impl IntoValue<kivact_style::asset::ImageFit>) -> Self {
        self.common.image_fit = Some(f.into_prop());
        self
    }

    /// Paint an image with its alpha only, in the node's text color
    /// (`style().color(..)`, inherited, and the hover, pressed and disabled
    /// state styles), like an icon drawn in `currentColor`. Image nodes
    /// only: on another node it is reported once at mount
    /// (`ErrorKind::Unsupported`).
    pub fn monochrome(mut self, on: impl IntoValue<bool>) -> Self {
        self.common.monochrome = Some(on.into_prop());
        self
    }

    /// Handle a failed image load or decode in the image's owner.
    ///
    /// A handler replaces the default asset diagnostic. It may use the reason
    /// to show a fallback. This modifier only applies to image nodes.
    pub fn on_image_error<R: crate::view::HandlerResult>(
        mut self,
        handler: impl Fn(&str) -> R + 'static,
    ) -> Self {
        self.common.handlers.image_error = Some(std::rc::Rc::new(move |reason| {
            handler(reason).into_result()
        }));
        self
    }
}

impl IntoView for Node {
    fn into_view(self) -> View {
        view_fn(move |cx| {
            let Node {
                content,
                mut common,
                children,
                at,
            } = self;
            let kind = content.kind();
            let at = common.at.or(at);
            // Effects need a location; a string child's content is static,
            // so this one only shows up if a style or binding is added.
            let loc = at.unwrap_or_else(Location::caller);
            let children = drop_misused(cx, kind, &mut common, children, loc);
            let id = mount_host(cx, kind, common, loc);
            if at.is_none()
                && let Some(n) = ViewState::tree_mut(cx.rt).get_mut(id)
            {
                n.at = None;
            }
            match content {
                Content::Box => {
                    for c in children {
                        let n = mount(c, cx);
                        cx.append(id, n);
                    }
                }
                Content::Plain(content) => cx.bind(id, content, PropValue::Content, loc),
                Content::Rich(spans) => super::text_binding::bind_rich(cx, id, spans, loc),
                Content::Image(src) => cx.bind(id, src, PropValue::Image, loc),
            }
            id
        })
    }
}

/// Take out of `common` the settings a node of `kind` cannot show, and
/// report them once, located at the node; returns the children a box
/// keeps (none for a text or an image).
fn drop_misused(
    cx: &Mcx<'_>,
    kind: NodeType,
    common: &mut Common,
    children: Vec<View>,
    at: &'static Location<'static>,
) -> Vec<View> {
    let mut misused: Vec<&str> = Vec::new();
    let is_box = kind == NodeType::Box && common.text_editing.is_none();
    if !is_box && children.iter().any(|c| !c.is_empty()) {
        misused.push("child");
    }
    if !is_box && common.draw.take().is_some() {
        misused.push("draw");
    }
    if !is_box && common.draw_over.take().is_some() {
        misused.push("draw_over");
    }
    if !is_box && !common.regions.is_empty() {
        misused.push("region");
        common.regions.clear();
    }
    if kind == NodeType::Text && common.intrinsic.take().is_some() {
        misused.push("intrinsic_size");
    }
    if kind != NodeType::Image && common.image_fit.take().is_some() {
        misused.push("fit");
    }
    if kind != NodeType::Image && common.monochrome.take().is_some() {
        misused.push("monochrome");
    }
    if kind != NodeType::Image && common.handlers.image_error.take().is_some() {
        misused.push("on_image_error");
    }
    if !misused.is_empty() {
        let what = match kind {
            NodeType::Text => "a text",
            NodeType::Image => "an image",
            _ => "a box",
        };
        let list = misused
            .iter()
            .map(|m| format!("`.{m}(..)`"))
            .collect::<Vec<_>>()
            .join(", ");
        cx.rt.report(
            Some(cx.owner),
            UiError::new(
                ErrorKind::Unsupported,
                format!("{list} on {what} node, which cannot show it; dropped"),
                Some(at),
            ),
        );
    }
    if is_box { children } else { Vec::new() }
}

/// A drawing surface: `f(draw, size)` re-runs when what it reads or the
/// node's size changes.
#[track_caller]
pub fn canvas(
    f: impl Fn(&mut kivact_style::draw::Draw, kivact_style::draw::Size) + 'static,
) -> Node {
    node().role(Role::Canvas).draw(f)
}

/// An image (PNG, JPEG, GIF and WebP first frame, SVG, SVGZ; the format
/// comes from the content) from embedded bytes (`include_bytes!(..)`),
/// run-time bytes or a file path (`"cover.jpg"`), or a closure returning an
/// [`Asset`] (for run-time bytes, keep one `Asset` per content and clone
/// it: an asset is cached by identity, so `Asset::from(bytes.clone())`
/// makes a new one, decoded again, each time the closure runs). Without a size in its style it takes the image's natural
/// size; with one dimension the other follows the aspect ratio. Role
/// `Image`; `.name(..)` is its alternative text, and without one the image
/// is decorative (not exposed to assistive technology, as HTML
/// `<img alt="">`) unless it has actions (`.on_a11y(..)`) or draws scene
/// regions; a test id alone does not expose it. A failed image draws
/// nothing and is reported once as `ErrorKind::Asset`, located here, to
/// the App's default sink unless [`Node::on_image_error`] handles it. [`Node::fit`] and
/// [`Node::monochrome`] set how it is painted.
#[track_caller]
pub fn image(src: impl IntoValue<Asset>) -> Node {
    Node::new(Content::Image(src.into_prop()), Some(Location::caller())).role(Role::Image)
}

/// An image drawn in nothing, for an icon whose name the theme lacks.
static NO_ICON: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"/>"#;

/// A decorative 16 px icon drawn in the current text color: a name resolved
/// in the theme in effect ([`Theme::icons`](crate::style::Theme::icons)),
/// so a theme switch redraws it with the new theme's glyph, or an image
/// [`Asset`](crate::Asset). It takes what an [`Icon`](crate::style::Icon)
/// converts from (`icon("check")` is `Icon::named("check")`), an `Icon`,
/// or a closure, `Signal` or `Value` of one.
///
/// This is the one way to draw an icon: a kit that draws its own images
/// puts them in its theme's [`IconSet`](crate::style::IconSet) under a
/// name.
///
/// A name the theme's set lacks draws nothing; in a debug build it is also
/// reported once (`ErrorKind::Asset`, located at the call). The icon is an
/// [`image`] with [`monochrome`](Node::monochrome) on: a style sets its size
/// and color, and `.name(..)` exposes it to assistive technology (without
/// one it is decorative). In a box smaller than 16 px (a small button's
/// icon box) it shrinks to fit the box.
///
/// ```no_run
/// # use kivact_core as kivact;
/// use kivact::control::{NodeBuilder, icon};
/// use kivact::style::{ColorToken, style};
/// let check = icon("check").style(style().color(ColorToken::Accent));
/// # let _ = check;
/// ```
#[track_caller]
pub fn icon(icon: impl IntoValue<kivact_style::Icon>) -> Node {
    let at = Location::caller();
    let source = icon.into_prop();
    let reported = std::cell::Cell::new(false);
    image(move || {
        let icon = source.get();
        let theme = kivact_style::current_theme();
        theme.icons().resolve(&icon).unwrap_or_else(|| {
            if cfg!(debug_assertions) && !reported.replace(true) {
                let owner = kivact_runtime::Owner::ambient("icon", at);
                let rt = owner.runtime();
                rt.report_later(
                    Some(owner),
                    UiError::new(
                        ErrorKind::Asset,
                        format!(
                            "icon {:?} is not in the theme's icon set; it draws nothing",
                            icon.name().unwrap_or_default()
                        ),
                        Some(at),
                    ),
                );
            }
            Asset::from(NO_ICON)
        })
    })
    .monochrome(true)
    .style(
        kivact_style::style()
            .size(16, 16)
            .max_width(kivact_style::pct(100))
            .max_height(kivact_style::pct(100))
            .shrink(0),
    )
}
