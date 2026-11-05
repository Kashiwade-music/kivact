//! Role: Neutral entity tokens with independently operable removal.
use kivact::state::IntoValue;

use crate::feedback::icon_box;
use crate::recipe_style::{overlay, ring};
use crate::{HueColors, Hues};
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::icon;
use kivact::view::row;
use kivact::view::text;

use kivact::state::Callback;
use kivact::state::IntoText;
use kivact::state::Value;
use kivact::state::callback;
use kivact::style::Align;
use kivact::style::Cursor;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::pct;
use kivact::style::style;
use kivact::style::use_theme;
use kivact::ui::ControlSize;
use kivact::view::IntoView;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::node_ref;
use kivact::view::use_direction;

/// Decorative token palette.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
    fn hue(self, hues: &Hues) -> Option<HueColors> {
        Some(match self {
            Self::Default => return None,
            Self::Red => hues.red,
            Self::Orange => hues.orange,
            Self::Yellow => hues.yellow,
            Self::Green => hues.green,
            Self::Teal => hues.teal,
            Self::Cyan => hues.cyan,
            Self::Blue => hues.blue,
            Self::Purple => hues.purple,
            Self::Pink => hues.pink,
            Self::Gray => hues.gray,
        })
    }
}

/// A static, pressable, or removable entity chip.
pub struct Token {
    root: Node,
    label: Value<String>,
    size: Value<ControlSize>,
    color: Value<TokenColor>,
    icon: Option<View>,
    end: Option<View>,
    press: Option<Callback<()>>,
    remove: Option<Callback<()>>,
    description: Option<Value<String>>,
    label_hidden: Value<bool>,
    as_link: Value<bool>,
}

#[track_caller]
/// Build a compact labelled token with optional main and remove actions.
pub fn token(label: impl IntoText) -> Token {
    Token {
        root: row(()),
        label: label.into_text_value(),
        size: (ControlSize::Medium).into_value(),
        color: (TokenColor::Default).into_value(),
        icon: None,
        end: None,
        press: None,
        remove: None,
        description: None,
        label_hidden: (false).into_value(),
        as_link: (false).into_value(),
    }
}

impl Token {
    /// Set the token control size.
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }

    /// Set the reactive categorical token palette.
    pub fn color(mut self, color: impl IntoValue<TokenColor>) -> Self {
        self.color = color.into_value();
        self
    }

    /// Place a leading icon before the label.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    /// Place additional content after the label.
    pub fn end(mut self, end: impl IntoView) -> Self {
        self.end = Some(end.into_view());
        self
    }

    /// Run a callback when the main token action is activated.
    pub fn on_press(mut self, press: impl Into<Callback<()>>) -> Self {
        let press: Callback<()> = press.into();
        self.press = Some(callback(move |_| press.call(())));
        self
    }

    /// Add a remove action and run the callback when it is activated.
    pub fn on_remove(mut self, remove: impl Into<Callback<()>>) -> Self {
        let remove: Callback<()> = remove.into();
        self.remove = Some(callback(move |_| remove.call(())));
        self
    }

    /// Set the accessible description of the main token action.
    pub fn description(mut self, description: impl IntoText) -> Self {
        self.description = Some(description.into_text_value());
        self
    }

    /// Hide the visible label while keeping its accessible name.
    pub fn label_hidden(mut self, hidden: impl IntoValue<bool>) -> Self {
        self.label_hidden = hidden.into_value();
        self
    }

    /// Expose the token main action as a link when enabled.
    pub fn as_link(mut self, link: impl IntoValue<bool>) -> Self {
        self.as_link = link.into_value();
        self
    }
}

impl NodeBuilder for Token {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Token {
    fn into_view(mut self) -> View {
        let label = self.label;
        let disabled = self
            .root
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        let color = self.color.get();
        let main_ref = node_ref();
        let focus = main_ref.focus_visible();
        let height = match self.size.get() {
            ControlSize::ExtraSmall | ControlSize::Small => 20,
            ControlSize::Medium => 24,
            ControlSize::Large => 28,
        };
        let direction = if use_direction().get() == ReadingDirection::Rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        };
        let interactive = self.press.is_some();
        let removable = self.remove.is_some();
        let mut main =
            row(())
                .with_node(move |node| {
                    node.base_style(move || {
                        let t = kivact::style::use_tokens::<crate::Tokens>();
                        let mut base = style()
                            .direction(direction)
                            .gap(4)
                            .min_width(0)
                            .height(height)
                            .align_items(Align::Center)
                            .padding_x(8)
                            .radius(t.radius.inner);
                        if removable {
                            base = match direction {
                                Direction::RowReverse => base.padding_left(0),
                                _ => base.padding_right(0),
                            };
                        }
                        if interactive && !disabled.get() {
                            let theme = use_theme().get();
                            let (fill, _) = color
                                .hue(&t.colors.hue)
                                .map_or((t.colors.neutral, t.colors.text_primary), |hue| {
                                    (hue.bg, hue.text)
                                });
                            base = base
                                .cursor(Cursor::Pointer)
                                .hover(style().background(overlay(
                                    &theme,
                                    fill,
                                    t.colors.overlay_hover,
                                )))
                                .pressed(style().background(overlay(
                                    &theme,
                                    fill,
                                    t.colors.overlay_pressed,
                                )))
                                .focus_visible(style().ring_none());
                        }
                        base
                    })
                })
                .child(self.icon.map(|icon| icon_box(icon, 12.0)))
                .child((!self.label_hidden.get()).then(|| {
                    text(label).style(style().min_width(0).text_wrap(false).line_clamp(1))
                }))
                .child(self.end);
        if let Some(press) = self.press {
            main = main
                .node_ref(main_ref)
                .with_node(move |node| {
                    node.role(if self.as_link.get() {
                        Role::Link
                    } else {
                        Role::Button
                    })
                })
                .name(label)
                .disabled(disabled)
                .on_press(move || press.call(()));
            if let Some(description) = self.description {
                main = main.description(description);
            }
        } else if self.label_hidden.get() {
            main = main.name(label);
        }
        let remove = self.remove.map(|remove| {
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
                        let t = kivact::style::use_tokens::<crate::Tokens>();
                        style()
                            .size(24, 24)
                            .shrink(0)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center)
                            .radius(t.radius.full)
                            .cursor(if disabled.get() {
                                Cursor::Default
                            } else {
                                Cursor::Pointer
                            })
                            .focus_visible(ring(&t, t.colors.focus_ring))
                    })
                })
        });
        self.root
            .with_node(move |node| {
                node.base_style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    let (fill, ink) = color
                        .hue(&t.colors.hue)
                        .map_or((t.colors.neutral, t.colors.text_primary), |hue| {
                            (hue.bg, hue.text)
                        });
                    let base = style()
                        .direction(direction)
                        .height(height)
                        .min_width(0)
                        .max_width(pct(100))
                        .radius(t.radius.inner)
                        .background(fill)
                        .color(ink)
                        .align_items(Align::Center)
                        .opacity(if disabled.get() {
                            t.sizes.disabled_opacity
                        } else {
                            1.0
                        })
                        .font_size(12)
                        .line_height(20.0 / 12.0)
                        .weight(t.typography.weight_medium);
                    if interactive && !disabled.get() && focus.get() {
                        base.merge(&ring(&t, t.colors.focus_ring))
                    } else {
                        base
                    }
                })
            })
            .child(main)
            .child(remove)
            .into_view()
    }
}

kivact::state::constant_value!(TokenColor);
