//! Role: Neutral entity tokens with independently operable removal.
use crate::color as c;
use crate::feedback::icon_box;
use crate::metrics as m;
use crate::radius as r;
use crate::recipe_parts::{overlay, ring};
use crate::type_scale as t;

use kivact::Prop;
use kivact::component;
use kivact::custom::Caller;
use kivact::state::Callback;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::style::Align;
use kivact::style::Color;
use kivact::style::Cursor;
use kivact::style::FlexDirection;
use kivact::style::Justify;
use kivact::style::Style;
use kivact::style::pct;
use kivact::style::style;
use kivact::style::use_theme;
use kivact::ui::ControlSize;
use kivact::view::Element;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::icon;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::text;
use kivact::view::use_direction;

/// Decorative token palette.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum TokenColor {
    #[default]
    /// Use the default neutral token palette.
    Default,
    /// Use the red categorical token palette.
    Red,
    /// Use the orange categorical token palette.
    Orange,
    /// Use the yellow categorical token palette.
    Yellow,
    /// Use the green categorical token palette.
    Green,
    /// Use the teal categorical token palette.
    Teal,
    /// Use the cyan categorical token palette.
    Cyan,
    /// Use the blue categorical token palette.
    Blue,
    /// Use the purple categorical token palette.
    Purple,
    /// Use the pink categorical token palette.
    Pink,
    /// Use the gray categorical token palette.
    Gray,
}

impl TokenColor {
    /// The fill and ink of the palette, or `None` for the neutral default.
    fn hue(self) -> Option<(Color, Color)> {
        Some(match self {
            Self::Default => return None,
            Self::Red => (c::HUE_RED_BG, c::HUE_RED_TEXT),
            Self::Orange => (c::HUE_ORANGE_BG, c::HUE_ORANGE_TEXT),
            Self::Yellow => (c::HUE_YELLOW_BG, c::HUE_YELLOW_TEXT),
            Self::Green => (c::HUE_GREEN_BG, c::HUE_GREEN_TEXT),
            Self::Teal => (c::HUE_TEAL_BG, c::HUE_TEAL_TEXT),
            Self::Cyan => (c::HUE_CYAN_BG, c::HUE_CYAN_TEXT),
            Self::Blue => (c::HUE_BLUE_BG, c::HUE_BLUE_TEXT),
            Self::Purple => (c::HUE_PURPLE_BG, c::HUE_PURPLE_TEXT),
            Self::Pink => (c::HUE_PINK_BG, c::HUE_PINK_TEXT),
            Self::Gray => (c::HUE_GRAY_BG, c::HUE_GRAY_TEXT),
        })
    }

    /// The fill and ink of this palette; the neutral default when it has no hue.
    fn colors(self) -> (Color, Color) {
        self.hue().unwrap_or((c::NEUTRAL, c::TEXT_PRIMARY))
    }
}

/// A static, pressable, or removable entity chip.
///
/// `on_press` runs when the main action is activated; `on_remove` adds a remove action.
#[component]
pub fn token(
    label: Value<String>,
    #[prop(default = ControlSize::Medium)] size: Value<ControlSize>,
    #[prop(default = TokenColor::Default)] color: Value<TokenColor>,
    icon: Option<View>,
    end: Option<View>,
    on_press: Option<Callback<()>>,
    on_remove: Option<Callback<()>>,
    description: Option<Value<String>>,
    #[prop(default = false)] label_hidden: Value<bool>,
    #[prop(default = false)] as_link: Value<bool>,
    #[caller] caller: Caller,
) -> impl Element {
    let disabled = caller.disabled().unwrap_or_else(|| false.into_value());
    let color = color.get();
    let main_ref = node_ref();
    let focus = main_ref.focus_visible();
    let height = match size.get() {
        ControlSize::ExtraSmall | ControlSize::Small => 20,
        ControlSize::Medium => 24,
        ControlSize::Large => 28,
    };
    let direction = if use_direction() == ReadingDirection::Rtl {
        FlexDirection::RowReverse
    } else {
        FlexDirection::Row
    };
    let interactive = on_press.is_some();
    let removable = on_remove.is_some();
    let mut main = row(())
        .with_node(move |node| {
            node.base_style(move || {
                token_main_style(direction, height, removable, interactive, disabled, color)
            })
        })
        .child(icon.map(|icon| icon_box(icon, 12.0)))
        .child(
            (!label_hidden.get())
                .then(|| text(label).style(style().min_width(0).text_wrap(false).line_clamp(1))),
        )
        .child(end);
    if let Some(press) = on_press {
        main = main
            .node_ref(main_ref)
            .with_node(move |node| {
                node.role(if as_link.get() {
                    Role::Link
                } else {
                    Role::Button
                })
            })
            .name(label)
            .disabled(disabled)
            .on_press(move || press.call(()));
        if let Some(description) = description {
            main = main.description(description);
        }
    } else if label_hidden.get() {
        main = main.name(label);
    }
    let remove = on_remove.map(|remove| token_remove_button(remove, label, disabled));
    row(())
        .with_node(move |node| {
            node.base_style(move || {
                token_style(direction, height, color, interactive, disabled, focus)
            })
        })
        .child(main)
        .child(remove)
}

fn token_main_style(
    direction: FlexDirection,
    height: i32,
    removable: bool,
    interactive: bool,
    disabled: Value<bool>,
    color: TokenColor,
) -> Style {
    let mut base = style()
        .flex_direction(direction)
        .gap(4)
        .min_width(0)
        .height(height)
        .align_items(Align::Center)
        .padding_x(8)
        .border_radius(r::RADIUS_INNER);
    if removable {
        base = match direction {
            FlexDirection::RowReverse => base.padding_left(0),
            _ => base.padding_right(0),
        };
    }
    if interactive && !disabled.get() {
        let theme = use_theme().get();
        let (fill, _) = color.colors();
        base = base
            .cursor(Cursor::Pointer)
            .hover(style().background(overlay(&theme, fill, c::OVERLAY_HOVER)))
            .active(style().background(overlay(&theme, fill, c::OVERLAY_PRESSED)))
            .focus_visible(style().outline_none());
    }
    base
}

fn token_remove_button(remove: Callback<()>, label: Value<String>, disabled: Value<bool>) -> Node {
    row(icon_box(icon("x").into_view(), 12.0))
        .with_node(move |node| node.role(Role::Button))
        .name(kivact::i18n::message_with(
            "soft.token.remove",
            "Remove {label}",
            move || [("label", label.get())],
        ))
        .disabled(disabled)
        .on_press(move || remove.call(()))
        .with_node(move |node| {
            node.base_style(move || {
                style()
                    .size(24, 24)
                    .flex_shrink(0)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .border_radius(r::RADIUS_FULL)
                    .cursor(if disabled.get() {
                        Cursor::Default
                    } else {
                        Cursor::Pointer
                    })
                    .focus_visible(ring(c::FOCUS_RING))
            })
        })
}

fn token_style(
    direction: FlexDirection,
    height: i32,
    color: TokenColor,
    interactive: bool,
    disabled: Value<bool>,
    focus: Value<bool>,
) -> Style {
    let (fill, ink) = color.colors();
    let base = style()
        .flex_direction(direction)
        .height(height)
        .min_width(0)
        .max_width(pct(100))
        .border_radius(r::RADIUS_INNER)
        .background(fill)
        .color(ink)
        .align_items(Align::Center)
        .opacity(if disabled.get() {
            m::DISABLED_OPACITY
        } else {
            1.0
        })
        .font_size(t::SIZE_SM)
        .line_height(20.0 / 12.0)
        .font_weight(t::WEIGHT_MEDIUM);
    if interactive && !disabled.get() && focus.get() {
        base.merge(&ring(c::FOCUS_RING))
    } else {
        base
    }
}
