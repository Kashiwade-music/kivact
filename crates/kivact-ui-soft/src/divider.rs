//! Role: Draw the Neutral separator and its optional label.
use crate::stack::ContainerInset;
use crate::tokens;
use kivact::control::{Node, NodeBuilder, node, text};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Direction, style};
use kivact::view::{IntoText, IntoView, Orientation, Role, View, node_ref, provide, try_context};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DividerVariant {
    #[default]
    Subtle,
    Strong,
}

pub struct Divider {
    root: Node,
    orientation: Orientation,
    variant: DividerVariant,
    label: Option<View>,
    full_bleed: bool,
}

pub fn divider() -> Divider {
    Divider {
        root: node().role(Role::Separator),
        orientation: Orientation::Horizontal,
        variant: DividerVariant::Subtle,
        label: None,
        full_bleed: false,
    }
}

impl Divider {
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn variant(mut self, variant: DividerVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn full_bleed(mut self, full_bleed: bool) -> Self {
        self.full_bleed = full_bleed;
        self
    }
    pub fn label<V: IntoView>(mut self, label: impl FnOnce() -> V + 'static) -> Self {
        self.label = Some(provide((), label));
        self
    }
    pub fn label_text(mut self, label: impl IntoText) -> Self {
        self.label = Some(text(label).into_view());
        self
    }
}

impl NodeBuilder for Divider {
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
        provide((), move || {
            let vertical = orientation == Orientation::Vertical;
            let mut root = root.orientation(orientation).base_style(move || {
                let inset = try_context::<ContainerInset>().unwrap_or_default();
                let mut base = style()
                    .shrink(0)
                    .align_self(Align::Stretch)
                    .align_items(Align::Center);
                if vertical {
                    base = base.direction(Direction::Column);
                    if full_bleed {
                        base = base.margin_top(-inset.top).margin_bottom(-inset.bottom);
                    }
                } else {
                    base = base.direction(Direction::Row);
                    if full_bleed {
                        let (left, right) = inset.sides();
                        base = base.margin_left(-left).margin_right(-right);
                    }
                }
                base
            });
            let rule = || {
                node().style(move || {
                    let t = tokens();
                    let color = match variant {
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
                    .labelled_by(label_ref)
                    .child(
                        node()
                            .node_ref(label_ref)
                            .style(move || {
                                let t = tokens();
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
