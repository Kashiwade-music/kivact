//! Role: Neutral tree list over Kivact's public tree interaction.

use std::hash::Hash;
use std::rc::Rc;

use crate::{ItemDensity, tokens};
use kivact::control::{Node, NodeBuilder, column, node, text};
use kivact::interaction::{ListSelection, TreeItemState, TreeRow, use_tree};
use kivact::primitive::{NodeExt, part};
use kivact::style::{Align, Direction, Justify, Position, style};
use kivact::ui::TreeNode;
use kivact::view::{
    HandlerResult, IntoText, IntoValue, IntoView, Role, View, callback, for_each, node_ref,
};
use kivact::{Binding, Callback, Value, signal};

pub type TreeDensity = ItemDensity;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TreeVariant {
    #[default]
    LineGuides,
    NoGuides,
}

type TreeRenderer<K> = Rc<dyn Fn(Value<TreeRow<K>>, Value<TreeItemState>) -> View>;

pub struct TreeList<K: Clone + Eq + Hash + 'static> {
    root: Node,
    label: Value<String>,
    nodes: Value<Vec<TreeNode<K>>>,
    expanded: Binding<Vec<K>>,
    selection: Option<ListSelection<K>>,
    activation: Option<Callback<K>>,
    header: Option<Box<dyn FnOnce() -> View>>,
    renderer: Option<TreeRenderer<K>>,
    density: TreeDensity,
    variant: TreeVariant,
}

pub fn tree_list<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    nodes: impl IntoValue<Vec<TreeNode<K>>>,
    expanded: impl Into<Binding<Vec<K>>>,
) -> TreeList<K> {
    TreeList {
        root: node(),
        label: label.into_text_value(),
        nodes: nodes.into_value(),
        expanded: expanded.into(),
        selection: None,
        activation: None,
        header: None,
        renderer: None,
        density: TreeDensity::Balanced,
        variant: TreeVariant::LineGuides,
    }
}

impl<K: Clone + Eq + Hash + 'static> NodeBuilder for TreeList<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<K: Clone + Eq + Hash + 'static> TreeList<K> {
    pub fn density(mut self, density: TreeDensity) -> Self {
        self.density = density;
        self
    }

    pub fn variant(mut self, variant: TreeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn selection(mut self, selection: ListSelection<K>) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn on_activate<R: HandlerResult>(mut self, activate: impl Fn(K) -> R + 'static) -> Self {
        self.activation = Some(callback(activate));
        self
    }

    pub fn header<V: IntoView>(mut self, header: impl FnOnce() -> V + 'static) -> Self {
        self.header = Some(Box::new(move || header().into_view()));
        self
    }

    pub fn row<V: IntoView>(
        mut self,
        row: impl Fn(Value<TreeRow<K>>, Value<TreeItemState>) -> V + 'static,
    ) -> Self {
        self.renderer = Some(Rc::new(move |item, state| row(item, state).into_view()));
        self
    }
}

impl<K: Clone + Eq + Hash + 'static> IntoView for TreeList<K> {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            nodes,
            expanded,
            selection,
            activation,
            header,
            renderer,
            density,
            variant,
        } = self;
        let selection = selection.unwrap_or_else(|| ListSelection::Single(signal(None).into()));
        let mut tree = use_tree(nodes, expanded, selection);
        if let Some(activate) = activation {
            tree = tree.on_activate(move |key| activate.call(key));
        }
        let header_ref = node_ref();
        let mut semantics = tree.list().name(label);
        if header.is_some() {
            semantics = semantics.labelled_by(header_ref);
        }
        let rows = for_each(
            tree.rows(),
            |row| (row.key.clone(), row.branch.has_children()),
            move |(key, branch), row| {
                let attachment = tree.item(key.clone());
                let state = attachment.state();
                let chevron = branch.then(|| {
                    node()
                        .part(
                            part(|input| input)
                                .role(Role::Button)
                                .name(move || {
                                    if state.get().branch.is_expanded() {
                                        "Collapse"
                                    } else {
                                        "Expand"
                                    }
                                })
                                .on_press(move || tree.toggle(key.clone())),
                        )
                        .style(
                            style()
                                .direction(Direction::Row)
                                .size(20, 20)
                                .shrink(0)
                                .align_items(Align::Center)
                                .justify_content(Justify::Center),
                        )
                        .child(text(move || {
                            if state.get().branch.is_expanded() {
                                "⌄"
                            } else {
                                "›"
                            }
                        }))
                });
                let label = if let Some(render) = &renderer {
                    render(row, state)
                } else {
                    text(move || {
                        row.with(|row| {
                            nodes.with(|nodes| {
                                nodes
                                    .iter()
                                    .find(|node| node.key() == &row.key)
                                    .map(TreeNode::label)
                                    .unwrap_or_default()
                            })
                        })
                    })
                    .style(style().font_size(14).line_height(20.0 / 14.0).single_line())
                    .into_view()
                };
                node()
                    .part(attachment)
                    .base_style(move || {
                        let p = tokens();
                        let state = state.get();
                        let level = state.level.saturating_sub(1);
                        let mut style = style()
                            .direction(Direction::Row)
                            .position(Position::Relative)
                            .align_items(Align::Center)
                            .min_width(0)
                            .gap(4)
                            .padding_y(density.padding())
                            .padding_left(12 + level * 16)
                            .padding_right(12)
                            .radius(p.radius.element)
                            .color(p.colors.text_primary)
                            .hover(style().background(p.colors.overlay_hover))
                            .focus_visible(
                                style().ring(p.sizes.focus_ring_width, p.colors.focus_ring),
                            );
                        if state.selected {
                            style = style.background(p.colors.accent_muted);
                        }
                        if variant == TreeVariant::LineGuides && level > 0 {
                            let guide = p.colors.border_emphasized;
                            style = style.paint(move |draw, size| {
                                for ancestor in 0..level {
                                    let x = 21.0 + ancestor as f32 * 16.0;
                                    draw.rounded_rect(
                                        kivact::draw::Rect::new(x, 0.0, 1.0, size.height + 2.0),
                                        0.0,
                                        guide,
                                    );
                                }
                            });
                        }
                        style
                    })
                    .child((
                        chevron,
                        (!branch).then(|| node().style(style().width(20).shrink(0))),
                        label,
                    ))
            },
        );
        column((
            header.map(|header| node().node_ref(header_ref).child(header())),
            root.part(semantics)
                .base_style(style().direction(Direction::Column).gap(2))
                .child(rows),
        ))
        .into_view()
    }
}
