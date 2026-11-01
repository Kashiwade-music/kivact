//! Role: A docking workspace: splits of tabbed containers with retained panels.
//!
//! The layout is an application-owned `Binding<DockLayout<K>>`. Containers,
//! splitters and panel contents are siblings positioned from the model, so a
//! panel's view stays mounted while its tab switches, moves to another
//! container or the tree is restructured.

mod chrome;
mod data;
mod geometry;
mod model;
mod operations;

use crate::Color;
use kivact::primitive::NodeExt;
use kivact::style::current_theme;
use kivact::view::HandlerResult;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use kivact::control::{Node, NodeBuilder, node, text};
use kivact::i18n::{message, message_with};
use kivact::interaction::use_splitter;
use kivact::style::{Cursor, Direction, Overflow, Position, pct, style};
use kivact::ui::{EditPhase, MenuEntry, ScalarRange};
use kivact::view::{
    Handled, IntoValue, IntoView, Key, Orientation, Role, View, callback, for_each, node_ref,
};
use kivact::{Binding, Callback, Value, binding, memo, signal, var};

use chrome::AppMenu;
use data::{DockCx, Item, MenuBuilder, Shared, Texts};
use geometry::{Divider, Frame, MIN_EXTENT, STRIP, regions};
pub use model::{
    DockAxis, DockChild, DockLayout, DockNode, DockPlacement, DockSide, DockTab, DockTabs,
};

/// A docking workspace bound to an application-owned [`DockLayout`].
///
/// Every edit (activation, drag-and-drop, container menu commands, rename,
/// close, maximize and splitter changes) writes a complete normalized layout
/// through the binding, so the application can persist it or record undo. A
/// splitter drag writes the layout on every move.
///
/// Panel views are created once per tab key by the factory and stay mounted
/// while the key exists: switching the active tab hides the others, and
/// moving a tab only repositions its retained view. Closing a tab disposes
/// its view; reopening the key creates a new one.
///
/// The texts the dock shows itself read through `kivact::i18n::message` with
/// the keys `dock.*`.
pub struct Dock<K: Clone + Eq + Hash + 'static> {
    root: Node,
    layout: Binding<DockLayout<K>>,
    panel: Rc<dyn Fn(K) -> View>,
    menu: MenuBuilder<K>,
    on_close: Option<Callback<K>>,
    on_edit: Option<Callback<EditPhase>>,
}

/// A docking workspace rendering `panel(key)` for every tab of `layout`.
///
/// Give the root a size with ordinary modifiers, e.g.
/// `.style(style().height(600))`.
#[track_caller]
pub fn dock<K, V>(
    layout: impl Into<Binding<DockLayout<K>>>,
    panel: impl Fn(K) -> V + 'static,
) -> Dock<K>
where
    K: Clone + Eq + Hash + 'static,
    V: IntoView,
{
    Dock {
        root: node()
            .role(Role::Group)
            .name(message("creative.dock.workspace", "Dock workspace"))
            .style(
                style()
                    .min_width(0)
                    .min_height(0)
                    .overflow(Overflow::Hidden),
            ),
        layout: layout.into(),
        panel: Rc::new(move |key| panel(key).into_view()),
        menu: Rc::new(|input| chrome::container_menu::<K, ()>(input, None)),
        on_close: None,
        on_edit: None,
    }
}

impl<K: Clone + Eq + Hash + 'static> Dock<K> {
    /// Observe splitter edits using the core splitter's Begin/Commit/Cancel
    /// contract. Tab activation, tab drags and other dock commands do not
    /// report these phases. Escape and disposal restore an active splitter edit.
    pub fn on_edit<R: HandlerResult>(
        mut self,
        observer: impl Fn(EditPhase) -> R + 'static,
    ) -> Self {
        self.on_edit = Some(callback(observer));
        self
    }

    /// Add application entries to each container's menu. They come first,
    /// then a separator and the built-in commands.
    ///
    /// `entries` receives the container; `choose` receives its active tab and
    /// the key of the chosen entry.
    pub fn menu<C: Clone + Eq + Hash + 'static, R: HandlerResult>(
        mut self,
        entries: impl Fn(&DockTabs<K>) -> Vec<MenuEntry<C>> + 'static,
        choose: impl Fn((K, C)) -> R + 'static,
    ) -> Self {
        let app = AppMenu {
            entries: Rc::new(entries),
            choose: callback(choose),
        };
        self.menu = Rc::new(move |input| chrome::container_menu(input, Some(app.clone())));
        self
    }
    /// Observe each tab removed by a close command, after the layout write.
    pub fn on_close<R: HandlerResult>(mut self, f: impl Fn(K) -> R + 'static) -> Self {
        self.on_close = Some(callback(f));
        self
    }
}

impl<K: Clone + Eq + Hash + 'static> NodeBuilder for Dock<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ItemKey<K> {
    /// A container, named by its first tab.
    Chrome(K),
    Content(K),
    /// The split's path, the boundary's index and the split's axis. A splitter
    /// keeps the axis it was built with, so a split that turns gets a new one.
    Divider(Vec<usize>, usize, DockAxis),
}

impl<K> Item<K> {
    fn frame(&self) -> Frame {
        match self {
            Self::Chrome { frame, .. } | Self::Content { frame, .. } => *frame,
            Self::Divider { divider, .. } => divider.span,
        }
    }
    fn shown(&self) -> bool {
        match self {
            Self::Chrome { shown, .. }
            | Self::Content { shown, .. }
            | Self::Divider { shown, .. } => *shown,
        }
    }
}

fn first_title<K: Clone + PartialEq>(node: &DockNode<K>) -> String {
    match node {
        DockNode::Tabs(tabs) => tabs
            .active_tab()
            .map(|tab| tab.title.clone())
            .unwrap_or_default(),
        DockNode::Split { children, .. } => children
            .first()
            .map(|child| first_title(&child.node))
            .unwrap_or_default(),
    }
}

fn split_children<'a, K>(root: &'a DockNode<K>, path: &[usize]) -> Option<&'a [DockChild<K>]> {
    let node = path.iter().try_fold(root, |node, index| match node {
        DockNode::Split { children, .. } => children.get(*index).map(|child| &child.node),
        DockNode::Tabs(_) => None,
    })?;
    match node {
        DockNode::Split { children, .. } => Some(children),
        DockNode::Tabs(_) => None,
    }
}

type Entry<K> = (ItemKey<K>, Item<K>);

/// Every positioned sibling of the dock root, keyed.
fn build_items<K: Clone + Eq + Hash>(layout: &DockLayout<K>) -> Vec<Entry<K>> {
    let (regions, dividers) = regions(&layout.root);
    let maximized = layout.maximized_container();
    let mut items = Vec::new();
    for region in regions {
        let Some(first) = region.tabs.tabs.first() else {
            continue;
        };
        let this = maximized.is_some_and(|max| std::ptr::eq(max, region.tabs));
        let (frame, shown) = match maximized {
            Some(_) if this => (Frame::FULL, true),
            Some(_) => (region.frame, false),
            None => (region.frame, true),
        };
        items.push((
            ItemKey::Chrome(first.id.clone()),
            Item::Chrome {
                tabs: region.tabs.clone(),
                frame,
                shown,
                maximized: this,
            },
        ));
        for tab in &region.tabs.tabs {
            items.push((
                ItemKey::Content(tab.id.clone()),
                Item::Content {
                    title: tab.title.clone(),
                    frame,
                    shown: shown && region.tabs.active.as_ref() == Some(&tab.id),
                },
            ));
        }
    }
    for divider in dividers {
        let children = split_children(&layout.root, &divider.path);
        let title = |index: usize| {
            children
                .and_then(|children| children.get(index))
                .map(|child| first_title(&child.node))
                .unwrap_or_default()
        };
        items.push((
            ItemKey::Divider(divider.path.clone(), divider.index, divider.axis),
            Item::Divider {
                leading: title(divider.index),
                trailing: title(divider.index + 1),
                divider,
                shown: maximized.is_none(),
            },
        ));
    }
    items
}

impl<K: Clone + Eq + Hash + 'static> IntoView for Dock<K> {
    fn into_view(self) -> View {
        let Self {
            root,
            layout,
            panel,
            menu,
            on_close,
            on_edit,
        } = self;
        let source = layout.into_value();
        let current = memo(move || {
            let mut layout = source.get();
            layout.normalize();
            layout
        });
        let reference = node_ref();
        let cx = DockCx {
            layout,
            on_edit,
            current,
            drag: signal(None),
            renaming: signal(None),
            draft: signal(String::new()),
            root: reference.rect(),
            shared: var(Rc::new(Shared {
                rects: RefCell::new(HashMap::new()),
                next_rect: Cell::new(0),
                menu,
                on_close,
                texts: Texts::new(),
            })),
        };
        let items = memo(move || current.with(build_items));
        let rows = for_each(
            items,
            |(key, _)| key.clone(),
            move |key, entry| {
                let item = memo(move || entry.with(|(_, item)| item.clone()));
                match key {
                    ItemKey::Chrome(_) => chrome::container(cx, item),
                    ItemKey::Content(id) => content(cx, id, item, panel.clone()),
                    ItemKey::Divider(path, index, axis) => divider(cx, path, index, axis, item),
                }
            },
        );
        let empty = text(message("creative.dock.empty", "No panels"))
            .hidden(move || !current.with(DockLayout::is_empty))
            .style(
                style()
                    .position(Position::Absolute)
                    .left(pct(50) - 30)
                    .top(pct(50) - 8)
                    .color(Color::Dim),
            );
        let preview = memo(move || {
            cx.drag.with(|drag| {
                drag.as_ref()
                    .and_then(|drag| drag.target.as_ref())
                    .map(|target| {
                        let edge = matches!(target.placement, DockPlacement::Edge(_));
                        (target.preview, target.bar, edge)
                    })
            })
        });
        let overlay = node()
            .hidden(move || preview.get().is_none())
            .style(move || {
                let t = &current_theme();
                let p = t.get::<crate::tokens::Tokens>();
                let Some((rect, bar, edge)) = preview.get() else {
                    return style();
                };
                let tint = if edge { p.success } else { p.accent };
                let base = style()
                    .position(Position::Absolute)
                    .left(rect.x)
                    .top(rect.y)
                    .width(rect.width)
                    .height(rect.height)
                    .z_index(10)
                    .radius(4);
                if bar {
                    base.background(tint)
                } else {
                    base.background(tint.with_alpha(0.2)).border(2, tint)
                }
            });
        root.node_ref(reference)
            .on_key_default(move |event| {
                let maximized = current.with(|layout| layout.maximized.is_some());
                if event.key == Key::Escape && maximized {
                    cx.edit(DockLayout::restore);
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
            .child((rows, empty, overlay))
            .into_view()
    }
}

fn content<K: Clone + Eq + Hash + 'static>(
    cx: DockCx<K>,
    id: K,
    item: Value<Item<K>>,
    panel: Rc<dyn Fn(K) -> View>,
) -> View {
    let title = memo(move || match item.with(Clone::clone) {
        Item::Content { title, .. } => title,
        _ => String::new(),
    });
    let key = var(id.clone());
    node()
        .role(Role::TabPanel)
        .name(title)
        .hidden(move || !item.with(Item::shown))
        .style(move || {
            item.with(Item::frame)
                .absolute(STRIP, 1.0)
                .direction(Direction::Column)
                .overflow(Overflow::Hidden)
        })
        .on_key_default(move |event| cx.maximize_key(&key.get(), &event))
        .child(
            node()
                .style(
                    style()
                        .grow(1)
                        .basis(0)
                        .min_width(0)
                        .min_height(0)
                        .direction(Direction::Column),
                )
                .child(panel(id)),
        )
        .into_view()
}

/// The splitter between children `index` and `index + 1` of a split.
///
/// Its value is the leading neighbor's share of the two neighbors' weights, in
/// percent.
fn divider<K: Clone + Eq + Hash + 'static>(
    cx: DockCx<K>,
    path: Vec<usize>,
    index: usize,
    axis: DockAxis,
    item: Value<Item<K>>,
) -> View {
    let state = memo(move || match item.with(Clone::clone) {
        Item::Divider { divider, .. } => Some(divider),
        _ => None,
    });
    let titles = memo(move || match item.with(Clone::clone) {
        Item::Divider {
            leading, trailing, ..
        } => (leading, trailing),
        _ => (String::new(), String::new()),
    });
    let path = Rc::new(path);
    // The two neighbors' normalized weights.
    let pair = move || {
        state.with(|divider| {
            let weights = &divider.as_ref()?.weights;
            Some((*weights.get(index)?, *weights.get(index + 1)?))
        })
    };
    // The pixel length of the two neighbors together along the split's axis.
    let pair_extent = move || {
        let extent = state.with(|divider| {
            let divider = divider.as_ref()?;
            Some(f64::from(divider.extent(cx.root.get()?)))
        });
        let (first, second) = pair().unwrap_or((1.0, 0.0));
        ((first + second) * extent.unwrap_or(800.0)).max(1.0)
    };
    let percent = memo(move || {
        pair().map_or(50.0, |(first, second)| {
            first / (first + second).max(f64::EPSILON) * 100.0
        })
    });
    let set_percent = {
        let path = path.clone();
        move |value: f64| {
            let (Some((first, second)), Some(mut weights)) = (
                pair(),
                state.with_untracked(|divider| divider.as_ref().map(|d| d.weights.clone())),
            ) else {
                return;
            };
            let total = first + second;
            weights[index] = value / 100.0 * total;
            weights[index + 1] = total - weights[index];
            cx.edit(|layout| layout.set_weights(&path, &weights));
        }
    };
    // Each neighbor keeps MIN_EXTENT pixels, as a percentage of the pair.
    let range = move || {
        let low = (f64::from(MIN_EXTENT) / pair_extent() * 100.0).min(50.0);
        ScalarRange::new(low, 100.0 - low).expect("invariant: finite dock splitter bounds")
    };
    let cursor = match axis {
        DockAxis::Horizontal => Cursor::EwResize,
        DockAxis::Vertical => Cursor::NsResize,
    };
    let mut behavior = use_splitter(
        binding(percent, set_percent),
        range,
        match axis {
            DockAxis::Horizontal => Orientation::Horizontal,
            DockAxis::Vertical => Orientation::Vertical,
        },
    )
    .key_step(1.0)
    .expect("invariant: positive finite dock key step")
    .reset_to(Some(50.0))
    .pointer_scale(move || 100.0 / pair_extent());
    if let Some(observer) = cx.on_edit {
        behavior = behavior.on_edit(move |value| observer.try_call(value));
    }
    let handle = behavior.handle();
    let name = message_with(
        "creative.dock.splitter",
        "{leading} and {trailing}",
        move || {
            let (leading, trailing) = titles.get();
            [("leading", leading), ("trailing", trailing)]
        },
    );
    node()
        .part(handle)
        .name(name)
        .hidden(move || !item.with(Item::shown))
        .style(move || state.with(|divider| divider.as_ref().map_or_else(style, Divider::style)))
        .style(style().cursor(cursor))
        .style(move || crate::dock_style::splitter_look(&current_theme()))
        .into_view()
}
