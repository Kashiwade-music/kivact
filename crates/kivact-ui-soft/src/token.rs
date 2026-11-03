//! Role: Neutral entity tokens with independently operable removal.

use crate::feedback::icon_box;
use crate::recipe_style::{overlay, ring};
use crate::{HueColors, Hues, tokens};
use kivact::control::{Node, NodeBuilder, icon, row, text};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Cursor, Direction, Justify, current_theme, pct, style};
use kivact::ui::ControlSize;
use kivact::view::{
    HandlerResult, IntoText, IntoView, ReadingDirection, Role, View, callback, current_direction,
    node_ref,
};
use kivact::{Callback, Value};

/// Decorative token palette.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TokenColor {
    #[default]
    Default,
    Red,
    Orange,
    Yellow,
    Green,
    Teal,
    Cyan,
    Blue,
    Purple,
    Pink,
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
    size: ControlSize,
    color: TokenColor,
    icon: Option<View>,
    end: Option<View>,
    press: Option<Callback<()>>,
    remove: Option<Callback<()>>,
    description: Option<Value<String>>,
    label_hidden: bool,
    disabled: bool,
    as_link: bool,
}

#[track_caller]
pub fn token(label: impl IntoText) -> Token {
    Token {
        root: row(()),
        label: label.into_text_value(),
        size: ControlSize::Medium,
        color: TokenColor::Default,
        icon: None,
        end: None,
        press: None,
        remove: None,
        description: None,
        label_hidden: false,
        disabled: false,
        as_link: false,
    }
}

impl Token {
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: TokenColor) -> Self {
        self.color = color;
        self
    }

    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    pub fn end(mut self, end: impl IntoView) -> Self {
        self.end = Some(end.into_view());
        self
    }

    pub fn on_press<R: HandlerResult>(mut self, press: impl Fn() -> R + 'static) -> Self {
        self.press = Some(callback(move |_| press()));
        self
    }

    pub fn on_remove<R: HandlerResult>(mut self, remove: impl Fn() -> R + 'static) -> Self {
        self.remove = Some(callback(move |_| remove()));
        self
    }

    pub fn description(mut self, description: impl IntoText) -> Self {
        self.description = Some(description.into_text_value());
        self
    }

    pub fn label_hidden(mut self, hidden: bool) -> Self {
        self.label_hidden = hidden;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn as_link(mut self, link: bool) -> Self {
        self.as_link = link;
        self
    }
}

impl NodeBuilder for Token {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Token {
    fn into_view(self) -> View {
        let label = self.label;
        let disabled = self.disabled;
        let color = self.color;
        let main_ref = node_ref();
        let focus = main_ref.focus_visible();
        let height = match self.size {
            ControlSize::ExtraSmall | ControlSize::Small => 20,
            ControlSize::Medium => 24,
            ControlSize::Large => 28,
        };
        let direction = if current_direction() == ReadingDirection::Rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        };
        let interactive = self.press.is_some();
        let removable = self.remove.is_some();
        let mut main = row(())
            .base_style(move || {
                let t = tokens();
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
                if interactive && !disabled {
                    let theme = current_theme();
                    let (fill, _) = color
                        .hue(&t.colors.hue)
                        .map_or((t.colors.neutral, t.colors.text_primary), |hue| {
                            (hue.bg, hue.text)
                        });
                    base = base
                        .cursor(Cursor::Pointer)
                        .hover(style().background(overlay(&theme, fill, t.colors.overlay_hover)))
                        .pressed(style().background(overlay(
                            &theme,
                            fill,
                            t.colors.overlay_pressed,
                        )))
                        .focus_visible(style().ring_none());
                }
                base
            })
            .child(self.icon.map(|icon| icon_box(icon, 12.0)))
            .child(
                (!self.label_hidden).then(|| {
                    text(label).style(style().min_width(0).text_wrap(false).line_clamp(1))
                }),
            )
            .child(self.end);
        if let Some(press) = self.press {
            main = main
                .node_ref(main_ref)
                .role(if self.as_link {
                    Role::Link
                } else {
                    Role::Button
                })
                .name(label)
                .disabled(disabled)
                .on_press(move || press.call(()));
            if let Some(description) = self.description {
                main = main.description(description);
            }
        } else if self.label_hidden {
            main = main.name(label);
        }
        let remove = self.remove.map(|remove| {
            row(icon_box(icon("x").into_view(), 12.0))
                .role(Role::Button)
                .name(kivact::i18n::message_with(
                    "soft.token.remove",
                    "Remove {label}",
                    move || [("label", label.get())],
                ))
                .disabled(disabled)
                .on_press(move || remove.call(()))
                .base_style(move || {
                    let t = tokens();
                    style()
                        .size(24, 24)
                        .shrink(0)
                        .align_items(Align::Center)
                        .justify_content(Justify::Center)
                        .radius(t.radius.full)
                        .cursor(if disabled {
                            Cursor::Default
                        } else {
                            Cursor::Pointer
                        })
                        .focus_visible(ring(&t, t.colors.focus_ring))
                })
        });
        self.root
            .base_style(move || {
                let t = tokens();
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
                    .opacity(if disabled {
                        t.sizes.disabled_opacity
                    } else {
                        1.0
                    })
                    .font_size(12)
                    .line_height(20.0 / 12.0)
                    .weight(t.typography.weight_medium);
                if interactive && !disabled && focus.get() {
                    base.merge(&ring(&t, t.colors.focus_ring))
                } else {
                    base
                }
            })
            .child(main)
            .child(remove)
            .into_view()
    }
}
