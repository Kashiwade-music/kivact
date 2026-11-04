//! Role: Neutral tree list over Kivact's public tree interaction.

use std::hash::Hash;
use std::rc::Rc;

use crate::{ItemDensity, tokens};
use kivact::control::{Node, NodeBuilder, column, icon, node, text};
use kivact::i18n::message;
use kivact::interaction::{ListSelection, TreeItemState, TreeRow, use_tree};
use kivact::motion::{Easing, Transition};
use kivact::primitive::{NodeExt, part};
use kivact::style::{Align, Direction, Justify, Position, TransitionProp, style};
use kivact::ui::{TreeNode, tree_rows, tree_rows_with_reveal};
use kivact::view::{
    HandlerResult, IntoText, IntoValue, IntoView, ReadingDirection, Role, View, callback,
    current_direction, node_ref,
};
use kivact::{Binding, Callback, Value, memo, signal};

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
    animated: bool,
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
        animated: false,
    }
}

impl<K: Clone + Eq + Hash + 'static> NodeBuilder for TreeList<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<K: Clone + Eq + Hash + 'static> TreeList<K> {
    /// Enable animated branch reveals and retained exits.
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

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
            animated,
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
        let rows = tree.rows();
        let has_branches =
            memo(move || rows.with(|rows| rows.iter().any(|row| row.branch.has_children())));
        let rtl = current_direction() == ReadingDirection::Rtl;
        let guides = (variant == TreeVariant::LineGuides).then(|| tree.ancestor_last());
        let render =
            move |key: K,
                  row: Value<TreeRow<K>>,
                  attachment: kivact::primitive::Part<TreeItemState>| {
                let state = attachment.state();
                let branch = row.with_untracked(|row| row.branch.has_children());
                let row_key = key.clone();
                let chevron = branch.then(|| {
                    node()
                        .part(
                            part(|input| input)
                                .role(Role::Button)
                                .disabled(move || state.get().input.disabled)
                                .expanded(move || state.get().branch.is_expanded())
                                .name(message("soft.tree_list.toggle_children", "Toggle children"))
                                .on_press(move || tree.toggle(key.clone())),
                        )
                        .style(
                            style()
                                .direction(Direction::Row)
                                .size(16, 16)
                                .shrink(0)
                                .align_items(Align::Center)
                                .justify_content(Justify::Center),
                        )
                        .child(
                            icon(if rtl { "chevron-left" } else { "chevron-right" }).style(
                                move || {
                                    let p = tokens();
                                    let angle = if state.get().branch.is_expanded() {
                                        if rtl { -90.0 } else { 90.0 }
                                    } else {
                                        0.0
                                    };
                                    style()
                                        .size(16, 16)
                                        .color(p.colors.icon_secondary)
                                        .rotate(angle)
                                        .transition(
                                            TransitionProp::Paint,
                                            kivact::motion::Transition::tween(
                                                125.0,
                                                p.motion.ease_standard,
                                            ),
                                        )
                                },
                            ),
                        )
                });
                let label = if let Some(render) = &renderer {
                    render(row, state)
                } else {
                    text(tree.label(row_key.clone()))
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
                            .direction(if rtl {
                                Direction::RowReverse
                            } else {
                                Direction::Row
                            })
                            .position(Position::Relative)
                            .align_items(Align::Center)
                            .min_width(0)
                            .gap(8)
                            .padding_y(density.padding())
                            .padding_left(if rtl { 8 } else { 8 + level * 16 })
                            .padding_right(if rtl { 8 + level * 16 } else { 8 })
                            .radius(p.radius.element)
                            .color(p.colors.text_primary)
                            .hover(style().background(p.colors.overlay_hover))
                            .focus_visible(
                                style().ring(p.sizes.focus_ring_width, p.colors.focus_ring),
                            );
                        if state.selected {
                            style = style.background(p.colors.accent_muted);
                        }
                        if let Some(ancestors) = guides
                            .and_then(|guides| guides.with(|guides| guides.get(&row_key).cloned()))
                            && !ancestors.is_empty()
                        {
                            let mut lines: Vec<_> = ancestors
                                .into_iter()
                                .skip(1)
                                .map(|last| (!last).then_some(true))
                                .collect();
                            lines.push(Some(!row.with(|row| row.is_last())));
                            let guide = p.colors.border_emphasized;
                            style = style.paint(move |draw, size| {
                                for (ancestor, extend) in lines.iter().enumerate() {
                                    if let Some(extend) = extend {
                                        let inset = 16.0 + ancestor as f32 * 16.0;
                                        let x = if rtl {
                                            size.width - inset - 0.5
                                        } else {
                                            inset - 0.5
                                        };
                                        let height = if *extend {
                                            size.height + 2.0
                                        } else {
                                            size.height
                                        };
                                        draw.rounded_rect(
                                            kivact::draw::Rect::new(x, 0.0, 1.0, height),
                                            0.5,
                                            guide,
                                        );
                                    }
                                }
                            });
                        }
                        style
                    })
                    .child((
                        chevron,
                        (!branch).then(|| {
                            node()
                                .hidden(move || !has_branches.get())
                                .style(style().width(16).shrink(0))
                        }),
                        label,
                    ))
            };
        let rows = if animated {
            tree_rows_with_reveal(tree, 2.0, tree_reveal_transition(), render)
        } else {
            tree_rows(tree, |_| (), render)
        };
        column((
            header.map(|header| node().node_ref(header_ref).child(header())),
            root.part(semantics)
                .base_style(style().direction(Direction::Column).gap(2))
                .child(rows),
        ))
        .into_view()
    }
}

fn tree_reveal_transition() -> Transition {
    // The critical spring's 734.77 ms response, approximated by a
    // bounded monotonic Bézier so height and host exit finish together.
    Transition::tween(734.77, Easing::CubicBezier(0.327, 0.694, 0.116, 1.0))
}
