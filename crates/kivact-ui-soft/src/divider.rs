//! Role: Draw the Neutral separator and its optional label.
use crate::stack::ContainerInset;

use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::node;
use kivact::view::text;

use kivact::state::IntoText;
use kivact::state::try_use_context;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::Orientation;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::node_ref;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Theme border color used by a separator.
pub enum DividerVariant {
    #[default]
    /// Use the normal border color; the default.
    Subtle,
    /// Use the emphasized border color.
    Strong,
}

/// Semantic separator with optional content between two rule segments.
// Independent separator orientation, visual variant, optional label and bleed inputs; no runtime interaction phase or duplicated model is stored.
pub struct Divider {
    root: Node,
    orientation: Value<Orientation>,
    variant: Value<DividerVariant>,
    // xtask-allow(state-fields): Optional once-mounted separator content slot.
    label: Option<View>,
    // xtask-allow(state-fields): Independent layout compensation input, not interaction state.
    full_bleed: Value<bool>,
}

/// Create an unlabeled, horizontal, subtle separator without full bleed.
pub fn divider() -> Divider {
    Divider {
        root: node().with_node(move |node| node.role(Role::Separator)),
        orientation: (Orientation::Horizontal).into_value(),
        variant: (DividerVariant::Subtle).into_value(),
        label: None,
        full_bleed: (false).into_value(),
    }
}

impl Divider {
    /// Choose the separator axis at construction from a constant or reactive input.
    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        self.orientation = orientation.into_value();
        self
    }
    /// Set a constant or reactive subtle or emphasized border color.
    pub fn variant(mut self, variant: impl IntoValue<DividerVariant>) -> Self {
        self.variant = variant.into_value();
        self
    }
    /// Reactively extend across inherited insets along the axis; off by default.
    pub fn full_bleed(mut self, full_bleed: impl IntoValue<bool>) -> Self {
        self.full_bleed = full_bleed.into_value();
        self
    }
    /// Insert custom content between rule segments; `label_text` also supplies a text name.
    pub fn label(mut self, label: impl IntoView) -> Self {
        self.label = Some(kivact::view::component(move || label.into_view()));
        self
    }
    /// Insert constant or reactive text and use it as the accessible name.
    pub fn label_text(mut self, label: impl IntoText) -> Self {
        let label = label.into_text_value();
        self.root = self.root.name(label);
        self.label = Some(text(label).into_view());
        self
    }
}

impl NodeBuilder for Divider {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Divider {
    fn into_view(self) -> View {
        let Self {
            root,
            orientation,
            variant,
            label,
            full_bleed,
        } = self;
        kivact::view::component(move || {
            let vertical = orientation.get() == Orientation::Vertical;
            let mut root = root.orientation(orientation.get()).with_node(move |node| {
                node.base_style(move || {
                    let inset = try_use_context::<ContainerInset>()
                        .map(|value| value.get())
                        .unwrap_or_default();
                    let mut base = style()
                        .shrink(0)
                        .align_self(Align::Stretch)
                        .align_items(Align::Center);
                    if vertical {
                        base = base.direction(Direction::Column);
                        if full_bleed.get() {
                            base = base.margin_top(-inset.top).margin_bottom(-inset.bottom);
                        }
                    } else {
                        base = base.direction(Direction::Row);
                        if full_bleed.get() {
                            let (left, right) = inset.sides();
                            base = base.margin_left(-left).margin_right(-right);
                        }
                    }
                    base
                })
            });
            let rule = || {
                node().style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    let color = match variant.get() {
                        DividerVariant::Subtle => t.colors.border,
                        DividerVariant::Strong => t.colors.border_emphasized,
                    };
                    let rule = style().grow(1).shrink(1).background(color);
                    if vertical {
                        rule.width(t.sizes.border_width)
                    } else {
                        rule.height(t.sizes.border_width)
                    }
                })
            };
            root = root.child(rule());
            if let Some(label) = label {
                let label_ref = node_ref();
                root = root
                    .with_node(move |node| node.labelled_by(label_ref))
                    .child(
                        node()
                            .node_ref(label_ref)
                            .style(move || {
                                let t = kivact::style::use_tokens::<crate::Tokens>();
                                let label = style()
                                    .shrink(0)
                                    .font_size(t.typography.supporting.size_px)
                                    .line_height(
                                        t.typography.supporting.line_height_px
                                            / t.typography.supporting.size_px,
                                    )
                                    .color(t.colors.text_secondary);
                                if vertical {
                                    label.padding_y(t.spacing.s3)
                                } else {
                                    label.padding_x(t.spacing.s3)
                                }
                            })
                            .child(label),
                    )
                    .child(rule());
            }
            root
        })
    }
}

kivact::state::constant_value!(DividerVariant);
