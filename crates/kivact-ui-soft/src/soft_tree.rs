//! Role: Neutral tree list over Kivact's public tree interaction.
use kivact::interaction::IntoSelection;

use kivact::state::IntoBinding;
use kivact::state::ItemKey;

use crate::ItemDensity;
use kivact::i18n::message;
use kivact::interaction::Selection;
use kivact::interaction::TreeItemState;
use kivact::interaction::TreeNode;
use kivact::interaction::TreeRow;
use kivact::interaction::use_tree;
use kivact::motion::Easing;
use kivact::motion::Transition;
use kivact::state::Binding;
use kivact::state::Callback;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::callback;
use kivact::state::memo;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::TransitionProp;
use kivact::style::style;
use kivact::ui::tree_rows;
use kivact::view::HandlerResult;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::text;
use kivact::view::use_direction;

pub type TreeDensity = ItemDensity;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TreeVariant {
    #[default]
    LineGuides,
    NoGuides,
}

type TreeRenderer<K> = kivact::view::ViewFn<(Value<TreeRow<K>>, Value<TreeItemState>)>;

pub struct TreeList<K: ItemKey> {
    root: Node,
    label: Value<String>,
    nodes: Value<Vec<TreeNode<K>>>,
    expanded: Binding<Vec<K>>,
    selection: Option<Selection<K>>,
    activation: Option<Callback<K>>,
    header: Option<View>,
    renderer: Option<TreeRenderer<K>>,
    density: Value<TreeDensity>,
    variant: Value<TreeVariant>,
    animated: Value<bool>,
}

pub fn tree_list<K: ItemKey>(
    label: impl IntoText,
    nodes: impl IntoValue<Vec<TreeNode<K>>>,
    expanded: impl IntoBinding<Vec<K>>,
) -> TreeList<K> {
    TreeList {
        root: node(),
        label: label.into_text_value(),
        nodes: nodes.into_value(),
        expanded: expanded.into_binding(),
        selection: None,
        activation: None,
        header: None,
        renderer: None,
        density: (TreeDensity::Balanced).into_value(),
        variant: (TreeVariant::LineGuides).into_value(),
        animated: (false).into_value(),
    }
}

impl<K: ItemKey> NodeBuilder for TreeList<K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<K: ItemKey> TreeList<K> {
    /// Enable animated branch reveals and retained exits.
    pub fn animated(mut self, animated: impl IntoValue<bool>) -> Self {
        self.animated = animated.into_value();
        self
    }

    pub fn density(mut self, density: impl IntoValue<TreeDensity>) -> Self {
        self.density = density.into_value();
        self
    }

    pub fn variant(mut self, variant: impl IntoValue<TreeVariant>) -> Self {
        self.variant = variant.into_value();
        self
    }

    pub fn selection(mut self, selection: impl IntoSelection<K>) -> Self {
        self.selection = Some(selection.into_selection());
        self
    }

    pub fn on_activate<R: HandlerResult>(mut self, activate: impl Fn(K) -> R + 'static) -> Self {
        self.activation = Some(callback(activate));
        self
    }

    pub fn header(mut self, header: impl IntoView) -> Self {
        self.header = Some(header.into_view());
        self
    }

    pub fn row<V: IntoView>(
        mut self,
        row: impl Fn(Value<TreeRow<K>>, Value<TreeItemState>) -> V + 'static,
    ) -> Self {
        self.renderer = Some((move |(item, state)| row(item, state)).into());
        self
    }
}

impl<K: ItemKey> IntoView for TreeList<K> {
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
        let selection = selection.unwrap_or_else(|| Selection::Single(signal(None).into_binding()));
        let mut tree = use_tree(nodes, expanded, selection);
        if let Some(activate) = activation {
            tree = tree.on_activate(move |key| activate.call(key));
        }
        let header_ref = node_ref();
        let mut root = root
            .with_node(move |node| node.part(tree.list()))
            .name(label);
        if header.is_some() {
            root = root.with_node(move |node| node.labelled_by(header_ref));
        }
        let rows = tree.rows();
        let has_branches =
            memo(move || rows.with(|rows| rows.iter().any(|row| row.branch.has_children())));
        let rtl = use_direction().get() == ReadingDirection::Rtl;
        let guides = (variant.get() == TreeVariant::LineGuides).then(|| tree.ancestor_last());
        let render = move |key: K,
                           row: Value<TreeRow<K>>,
                           attachment: kivact::custom::Part<TreeItemState>| {
            let state = attachment.state();
            let branch = row.with_untracked(|row| row.branch.has_children());
            let row_key = key.clone();
            let chevron = branch.then(|| {
                node()
                    .with_node(move |node| node.role(Role::Button))
                    .disabled(move || state.get().input.disabled)
                    .with_node(move |node| node.expanded(move || state.get().branch.is_expanded()))
                    .name(message("soft.tree_list.toggle_children", "Toggle children"))
                    .on_press(move || tree.toggle(key.clone()))
                    .style(
                        style()
                            .direction(Direction::Row)
                            .size(16, 16)
                            .shrink(0)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center),
                    )
                    .child(
                        icon(if rtl { "chevron-left" } else { "chevron-right" }).style(move || {
                            let p = kivact::style::use_tokens::<crate::Tokens>();
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
                        }),
                    )
            });
            let label = if let Some(render) = &renderer {
                render.call((row, state))
            } else {
                text(tree.label(row_key.clone()))
                    .style(style().font_size(14).line_height(20.0 / 14.0).single_line())
                    .into_view()
            };
            node()
                .with_node(move |node| node.part(attachment))
                .with_node(move |node| {
                    node.base_style(move || {
                        let p = kivact::style::use_tokens::<crate::Tokens>();
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
                            .padding_y(density.get().padding())
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
                                        let inset = 16.0 + ancestor as f64 * 16.0;
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
                                            kivact::geometry::Rect::new(x, 0.0, 1.0, height),
                                            0.5,
                                            guide,
                                        );
                                    }
                                }
                            });
                        }
                        style
                    })
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
        let rows = if animated.get() {
            tree_rows(tree, |_| (), render).reveal(2.0, tree_reveal_transition())
        } else {
            tree_rows(tree, |_| (), render)
        };
        column((
            header.map(|header| node().node_ref(header_ref).child(header)),
            root.with_node(move |node| {
                node.base_style(style().direction(Direction::Column).gap(2))
            })
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

kivact::state::constant_value!(TreeVariant);
