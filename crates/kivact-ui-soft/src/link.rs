//! Role: Neutral text links with explicit native activation.

use crate::feedback::icon_box;
use crate::recipe_style::ring;
use crate::{TextKind, tokens};
use kivact::control::{Node, NodeBuilder, icon, row, text};
use kivact::primitive::{NodeExt, part};
use kivact::style::{Align, Cursor, Direction, TextDecorationLine, pct, style};
use kivact::view::{
    Handled, HandlerResult, IntoText, IntoView, Key, ReadingDirection, Role, View, callback,
    current_direction,
};
use kivact::{Callback, Value, memo};

/// Foreground of a text link.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LinkColor {
    #[default]
    Accent,
    Primary,
    Secondary,
    Disabled,
    Placeholder,
    Inherit,
}

/// Inline-sized link or link-shaped action.
pub struct Link {
    root: Node,
    label: Option<Value<String>>,
    content: View,
    press: Option<Callback<()>>,
    new_tab_label: Option<Value<String>>,
    underline: bool,
    disabled: bool,
    external: bool,
    standalone: bool,
    kind: TextKind,
    size: Option<f32>,
    weight: Option<u16>,
    color: LinkColor,
    max_lines: u32,
    as_button: bool,
}

#[track_caller]
pub fn link(label: impl IntoText) -> Link {
    let label = label.into_text_value();
    let mut link = link_with(text(label));
    link.label = Some(label);
    link
}

#[track_caller]
pub fn link_with(content: impl IntoView) -> Link {
    Link {
        root: row(()),
        label: None,
        content: content.into_view(),
        press: None,
        new_tab_label: None,
        underline: false,
        disabled: false,
        external: false,
        standalone: false,
        kind: TextKind::Body,
        size: None,
        weight: None,
        color: LinkColor::Accent,
        max_lines: 0,
        as_button: false,
    }
}

impl Link {
    pub fn name(mut self, label: impl IntoText) -> Self {
        self.label = Some(label.into_text_value());
        self
    }

    pub fn on_press<R: HandlerResult>(mut self, press: impl Fn() -> R + 'static) -> Self {
        self.press = Some(callback(move |_| press()));
        self
    }

    /// Use the Space and Enter behavior of an action button.
    pub fn as_button(mut self, value: bool) -> Self {
        self.as_button = value;
        self
    }

    pub fn underline(mut self, value: bool) -> Self {
        self.underline = value;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn external(mut self, value: bool) -> Self {
        self.external = value;
        self
    }

    pub fn new_tab_label(mut self, label: impl IntoText) -> Self {
        self.new_tab_label = Some(label.into_text_value());
        self
    }

    pub fn standalone(mut self, value: bool) -> Self {
        self.standalone = value;
        self
    }

    pub fn text_type(mut self, kind: TextKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn text_size(mut self, px: f32) -> Self {
        self.size = Some(px);
        self
    }

    pub fn weight(mut self, weight: u16) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn color(mut self, color: LinkColor) -> Self {
        self.color = color;
        self
    }

    pub fn max_lines(mut self, lines: u32) -> Self {
        self.max_lines = lines;
        self
    }
}

impl NodeBuilder for Link {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Link {
    fn into_view(self) -> View {
        let disabled = self.disabled;
        let interactive = self.press.is_some();
        let attachment = part(|state| state);
        let input = attachment.state();
        let mut root = self.root.part(attachment);
        if let Some(press) = self.press {
            root = root
                .role(if self.as_button {
                    Role::Button
                } else {
                    Role::Link
                })
                .focusable(true)
                .disabled(disabled)
                .on_press(move || {
                    if !disabled {
                        press.call(());
                    }
                });
            if !self.as_button {
                root = root.on_key(|event| {
                    if event.key == Key::Space {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                });
            }
        }
        if let Some(label) = self.label {
            let label = if self.external {
                let suffix = self.new_tab_label.unwrap_or_else(|| {
                    kivact::i18n::message("soft.link.new_tab", "(opens in new tab)")
                });
                memo(move || format!("{} {}", label.get(), suffix.get()))
            } else {
                label
            };
            root = root.name(label);
        }
        let direction = if current_direction() == ReadingDirection::Rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        };
        let kind = self.kind;
        let size = self.size;
        let weight = self.weight;
        let underline = self.underline;
        let color = self.color;
        let standalone = self.standalone;
        let lines = self.max_lines;
        root.base_style(move || {
            let t = tokens();
            let typography = &t.typography;
            let metrics = match kind {
                TextKind::Body => Some(typography.body),
                TextKind::Label => Some(typography.label),
                TextKind::Supporting => Some(typography.supporting),
                TextKind::Large => Some(typography.large),
                TextKind::Code => Some(typography.code),
                TextKind::Display1 => Some(typography.display1),
                TextKind::Display2 => Some(typography.display2),
                TextKind::Display3 => Some(typography.display3),
                TextKind::Inherit if standalone => Some(typography.body),
                TextKind::Inherit => None,
            };
            let ink = match color {
                LinkColor::Accent => Some(t.colors.text_accent),
                LinkColor::Primary => Some(t.colors.text_primary),
                LinkColor::Secondary | LinkColor::Placeholder => Some(t.colors.text_secondary),
                LinkColor::Disabled => Some(t.colors.text_disabled),
                LinkColor::Inherit => None,
            };
            let mut base = style()
                .direction(direction)
                .align_items(Align::Center)
                .gap(2)
                .min_width(0)
                .max_width(pct(100));
            if let Some(metrics) = metrics {
                base = base
                    .font_family(metrics.family)
                    .font_size(metrics.size_px)
                    .weight(metrics.weight)
                    .line_height(metrics.line_height_px / metrics.size_px);
            }
            if let Some(ink) = ink {
                base = base.color(ink);
            }
            if let Some(size) = size {
                base = base.font_size(size);
            }
            if let Some(weight) = weight {
                base = base.weight(weight);
            }
            if underline || (interactive && !disabled && input.get().hovered) {
                base = base.text_decoration(TextDecorationLine::UNDERLINE);
            }
            if disabled {
                base.opacity(t.sizes.disabled_opacity)
                    .cursor(Cursor::Default)
            } else if interactive {
                base.cursor(Cursor::Pointer)
                    .pressed(style().background(t.colors.overlay_pressed))
                    .focus_visible(ring(&t, t.colors.focus_ring))
            } else {
                base
            }
        })
        .child(row(self.content).style(style().min_width(0).line_clamp(lines)))
        .child(
            self.external
                .then(|| icon_box(icon("external-link").into_view(), 12.0)),
        )
        .into_view()
    }
}
