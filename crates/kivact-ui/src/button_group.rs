//! Role: Group arbitrary action controls into one native roving focus scope.
use crate::ControlSize;
use crate::layer::with_base_recipe_style;
use crate::variants::{Variant, Variants};
use kivact_core::state::{IntoText, IntoValue, Value, provide, try_use_context};
use kivact_core::style::{Align, Color, Direction, style};
use kivact_core::view::{
    IntoChildren, IntoView, Node, NodeBuilder, Orientation, ReadingDirection, Role, View, node,
    row, text, use_direction,
};

/// Passive button group metrics and colors.
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonGroupTokens {
    pub gap: f64,
    pub radius: f64,
    pub divider: Color,
    pub solid_divider: Color,
    pub text_background: Color,
    pub text_color: Color,
}
impl Default for ButtonGroupTokens {
    fn default() -> Self {
        Self {
            gap: 0.0,
            radius: 0.0,
            divider: Color::TRANSPARENT,
            solid_divider: Color::TRANSPARENT,
            text_background: Color::TRANSPARENT,
            text_color: Color::BLACK,
        }
    }
}

/// Layout supplied to group and auxiliary cell recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonGroupState {
    pub orientation: Orientation,
    pub size: ControlSize,
    pub rtl: bool,
}

#[derive(Clone, Copy)]
struct GroupAxis(Orientation);

#[derive(Clone, Copy)]
struct GroupSize(ControlSize);

/// A named set of action controls with one keyboard Tab stop.
pub struct ButtonGroup<C> {
    root: Node,
    children: C,
    orientation: Orientation,
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
        orientation: Orientation::Horizontal,
        size: crate::values::default_control_size().into_value(),
        variants: Variants::default(),
    }
}
impl<C> ButtonGroup<C> {
    /// The layout and keyboard axis, fixed for this group.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    /// Inherited size for children without an explicit size.
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }
    /// Add a theme presentation variant.
    pub fn variant(self, variant: impl Variant) -> Self {
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
        let rtl = use_direction().get() == ReadingDirection::Rtl;
        let children = provide(GroupAxis(orientation), move || {
            provide(GroupSize(size.get()), move || {
                crate::provide_control_size(size.get(), move || self.children.into_views())
            })
        });
        with_base_recipe_style(
            self.root.orientation(orientation).roving_focus(orientation),
            |r| r.button_group_recipes().root_recipe(),
            &self.variants,
            move || ButtonGroupState {
                orientation,
                size: size.get(),
                rtl,
            },
        )
        .child(children)
        .into_view()
    }
}
/// A non-focusable line between controls, perpendicular to the group axis.
#[track_caller]
pub fn button_group_separator() -> impl kivact_core::view::Component {
    let orientation =
        try_use_context::<GroupAxis>().map_or(Orientation::Horizontal, |axis| axis.get().0);
    let state = ButtonGroupState {
        orientation,
        size: crate::values::default_control_size(),
        rtl: use_direction().get() == ReadingDirection::Rtl,
    };
    with_base_recipe_style(
        node()
            .role(Role::Separator)
            .orientation(if orientation == Orientation::Vertical {
                Orientation::Horizontal
            } else {
                Orientation::Vertical
            }),
        |r| r.button_group_recipes().separator_recipe(),
        &Variants::default(),
        move || state,
    )
}
/// A non-focusable prefix or suffix cell in a button group.
#[track_caller]
pub fn button_group_text(label: impl IntoText) -> impl kivact_core::view::Component {
    let orientation =
        try_use_context::<GroupAxis>().map_or(Orientation::Horizontal, |axis| axis.get().0);
    let state = ButtonGroupState {
        orientation,
        size: crate::values::default_control_size(),
        rtl: use_direction().get() == ReadingDirection::Rtl,
    };
    with_base_recipe_style(
        row(text(label)),
        |r| r.button_group_recipes().text_recipe(),
        &Variants::default(),
        move || state,
    )
}
pub(crate) fn root_style(
    theme: &kivact_core::style::Theme,
    look: &crate::Look<ButtonGroupState>,
) -> kivact_core::style::Style {
    let t = &theme.ext::<crate::UiTokens>().button_group;
    style()
        .direction(if look.orientation == Orientation::Vertical {
            Direction::Column
        } else if look.rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .align_items(Align::Stretch)
        .gap(t.gap)
        .radius(t.radius)
}
pub(crate) fn separator_style(
    theme: &kivact_core::style::Theme,
    look: &crate::Look<ButtonGroupState>,
) -> kivact_core::style::Style {
    let base = style()
        .shrink(0)
        .align_self(Align::Stretch)
        .background(theme.ext::<crate::UiTokens>().button_group.divider);
    if look.orientation == Orientation::Vertical {
        base.height(1).margin_x(1)
    } else {
        base.width(1).margin_y(1)
    }
}
pub(crate) fn text_style(
    theme: &kivact_core::style::Theme,
    _: &crate::Look<ButtonGroupState>,
) -> kivact_core::style::Style {
    let t = &theme.ext::<crate::UiTokens>().button_group;
    style()
        .align_items(Align::Center)
        .gap(8)
        .padding_x(10)
        .border(1, t.divider)
        .background(t.text_background)
        .color(t.text_color)
        .font_size(12)
        .weight(500)
        .text_wrap(false)
}

/// An action button with explicit joined-group presentation options.
/// Ordinary buttons can also be supplied directly to [`button_group`].
pub struct GroupMember {
    button: crate::Button,
    solid: bool,
    keep_size: bool,
}
impl From<crate::Button> for GroupMember {
    fn from(button: crate::Button) -> Self {
        Self {
            button,
            solid: false,
            keep_size: false,
        }
    }
}
impl GroupMember {
    /// Use the theme's on-accent divider color for a solid button.
    pub fn solid(mut self, solid: bool) -> Self {
        self.solid = solid;
        self
    }
    /// Keep this button's explicit size instead of inheriting the group size.
    pub fn keep_size(mut self, keep: bool) -> Self {
        self.keep_size = keep;
        self
    }
}
impl IntoView for GroupMember {
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
                let divider = if solid {
                    t.button_group.solid_divider
                } else {
                    t.button_group.divider
                };
                style().border_left(1, divider).pressed(style().scale(1))
            })
            .into_view()
    }
}
