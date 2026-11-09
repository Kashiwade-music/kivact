//! Role: Group arbitrary action controls into one native roving focus scope.
use crate::group_types::{ButtonGroupMemberState, ButtonGroupState};
use crate::layer::with_base_recipe_style;
use crate::values::ControlSize;
use crate::variants::{Variant, Variants};
use kivact_core::state::{IntoText, IntoValue, Value, provide, try_use_context};
use kivact_core::style::style;
use kivact_core::view::{
    IntoChildren, IntoView, Node, NodeBuilder, Orientation, ReadingDirection, Role, View, node,
    row, text, use_direction,
};

#[derive(Clone, Copy)]
struct GroupAxis(Value<Orientation>);

#[derive(Clone, Copy)]
struct GroupSize(ControlSize);

#[derive(Clone, Copy)]
struct GroupPosition(Value<ButtonGroupMemberState>);

/// A named set of action controls with one keyboard Tab stop.
pub struct ButtonGroup<C> {
    root: Node,
    children: C,
    orientation: Value<Orientation>,
    size: Value<ControlSize>,
    variants: Variants,
}
/// Join controls under an accessible name. Arrows along the axis move focus;
/// Home and End reach the first and last eligible controls. Non-control text
/// and separators do not participate in the keyboard sequence.
#[track_caller]
pub fn button_group<C: IntoChildren>(label: impl IntoText, children: C) -> ButtonGroup<C> {
    ButtonGroup {
        root: node().role(Role::Group).name(label),
        children,
        orientation: Orientation::Horizontal.into_value(),
        size: crate::values::default_control_size().into_value(),
        variants: Variants::default(),
    }
}
impl<C> ButtonGroup<C> {
    /// The constant or reactive layout and keyboard axis.
    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        self.orientation = orientation.into_value();
        self
    }
    /// Inherited size for children without an explicit size.
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }
    /// Add a theme presentation variant.
    pub fn variant(mut self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl<C> NodeBuilder for ButtonGroup<C> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl<C: IntoChildren> IntoView for ButtonGroup<C> {
    fn into_view(self) -> View {
        let orientation = self.orientation;
        let size = self.size;
        let direction = use_direction();
        let members = self.variants.clone();
        let children = provide(GroupAxis(orientation), move || {
            provide(GroupSize(size.get()), move || {
                crate::provide_control_size(size.get(), move || {
                    let children = self.children.into_views();
                    let count = children.len();
                    children
                        .into_iter()
                        .enumerate()
                        .map(move |(index, child)| {
                            let state = kivact_core::state::memo(move || ButtonGroupMemberState {
                                layout: ButtonGroupState {
                                    orientation: orientation.get(),
                                    size: size.get(),
                                    rtl: direction.get() == ReadingDirection::Rtl,
                                },
                                first: index == 0,
                                last: index + 1 == count,
                            });
                            with_base_recipe_style(
                                node(),
                                |r| r.button_group_recipes().member_recipe(),
                                &members,
                                move || state.get(),
                            )
                            .child(provide(GroupPosition(state), move || child))
                        })
                        .collect::<Vec<_>>()
                })
            })
        });
        with_base_recipe_style(
            self.root.orientation(orientation).roving_focus(orientation),
            |r| r.button_group_recipes().root_recipe(),
            &self.variants,
            move || ButtonGroupState {
                orientation: orientation.get(),
                size: size.get(),
                rtl: direction.get() == ReadingDirection::Rtl,
            },
        )
        .child(children)
        .into_view()
    }
}
/// A non-focusable auxiliary cell, created by [`button_group_separator`]
/// or [`button_group_text`].
pub struct ButtonGroupCell {
    root: Node,
    // xtask-allow(state-fields): Independent presentation or construction input, not interaction state.
    separator: bool,
}
impl NodeBuilder for ButtonGroupCell {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for ButtonGroupCell {
    fn into_view(self) -> View {
        let orientation = try_use_context::<GroupAxis>()
            .map_or_else(|| Orientation::Horizontal.into_value(), |axis| axis.get().0);
        let state = kivact_core::state::memo(move || ButtonGroupState {
            orientation: orientation.get(),
            size: crate::values::default_control_size(),
            rtl: use_direction().get() == ReadingDirection::Rtl,
        });
        let root = if self.separator {
            self.root.orientation(move || {
                if orientation.get() == Orientation::Vertical {
                    Orientation::Horizontal
                } else {
                    Orientation::Vertical
                }
            })
        } else {
            self.root
        };
        with_base_recipe_style(
            root,
            if self.separator {
                |r| r.button_group_recipes().separator_recipe()
            } else {
                |r| r.button_group_recipes().text_recipe()
            },
            &Variants::default(),
            move || state.get(),
        )
        .into_view()
    }
}
/// A non-focusable line perpendicular to the group's axis.
#[track_caller]
pub fn button_group_separator() -> ButtonGroupCell {
    ButtonGroupCell {
        root: node().role(Role::Separator),
        separator: true,
    }
}
/// A non-focusable prefix or suffix cell in a button group.
#[track_caller]
pub fn button_group_text(label: impl IntoText) -> ButtonGroupCell {
    ButtonGroupCell {
        root: row(text(label)),
        separator: false,
    }
}
/// An action button with explicit joined-group presentation options.
/// Ordinary buttons can also be supplied directly to [`button_group`].
pub struct ButtonGroupMember {
    button: crate::Button,
    solid: Value<bool>,
    // xtask-allow(state-fields): Independent presentation or construction input, not interaction state.
    keep_size: bool,
}
impl From<crate::Button> for ButtonGroupMember {
    fn from(button: crate::Button) -> Self {
        Self {
            button,
            solid: false.into_value(),
            keep_size: false,
        }
    }
}
impl ButtonGroupMember {
    /// Use the theme's on-accent divider color for a solid button.
    pub fn solid(mut self, solid: impl IntoValue<bool>) -> Self {
        self.solid = solid.into_value();
        self
    }
    /// Keep the button's configured geometry instead of inheriting the group size.
    /// This structural option is fixed before the group is mounted.
    pub fn keep_size(mut self, keep: bool) -> Self {
        self.keep_size = keep;
        self
    }
}
impl IntoView for ButtonGroupMember {
    fn into_view(self) -> View {
        let button = if self.keep_size {
            self.button
        } else {
            self.button.default_size(
                try_use_context::<GroupSize>()
                    .map_or(crate::values::default_control_size(), |size| size.get().0),
            )
        };
        let solid = self.solid;
        button
            .style(move || {
                let t = kivact_core::style::use_tokens::<crate::UiTokens>();
                let divider = if solid.get() {
                    t.button_group.solid_divider
                } else {
                    t.button_group.divider
                };
                let base = style().pressed(style().scale(1));
                let Some(position) =
                    try_use_context::<GroupPosition>().map(|position| position.get().0.get())
                else {
                    return base;
                };
                let r = t.button_group.radius;
                let first = if position.first { r } else { 0.0 };
                let last = if position.last { r } else { 0.0 };
                let vertical = position.layout.orientation == Orientation::Vertical;
                let (tl, tr, br, bl) = if vertical {
                    (first, first, last, last)
                } else if position.layout.rtl {
                    (last, first, first, last)
                } else {
                    (first, last, last, first)
                };
                let base = base.radius_corners(tl, tr, br, bl);
                if position.first {
                    base
                } else if vertical {
                    base.border_top(1, divider)
                } else if position.layout.rtl {
                    base.border_right(1, divider)
                } else {
                    base.border_left(1, divider)
                }
            })
            .into_view()
    }
}
