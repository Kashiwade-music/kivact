//! Role: Neutral text links with explicit native activation.
use crate::color as c;
use crate::feedback::icon_box;
use crate::metrics as m;
use crate::recipe_parts::ring;
use crate::text::TextKind;
use crate::type_scale::{self as t, TypeStyle};

use kivact::Prop;
use kivact::component;
use kivact::custom::Caller;
use kivact::custom::PartState;
use kivact::custom::part;
use kivact::state::Value;
use kivact::state::memo;
use kivact::style::Align;
use kivact::style::Color;
use kivact::style::Cursor;
use kivact::style::FlexDirection;
use kivact::style::Style;
use kivact::style::TextDecorationLine;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::Element;
use kivact::view::Handled;
use kivact::view::Key;
use kivact::view::Node;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::icon;
use kivact::view::row;
use kivact::view::text;
use kivact::view::use_direction;

/// Foreground of a text link.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum LinkColor {
    #[default]
    /// Use the theme's accent foreground.
    Accent,
    /// Use the primary text foreground.
    Primary,
    /// Use the secondary text foreground.
    Secondary,
    /// Use the disabled text foreground without changing activation availability.
    Disabled,
    /// Use the secondary foreground for placeholder-style links.
    Placeholder,
    /// Inherit the surrounding foreground color.
    Inherit,
}

/// Inline-sized link or link-shaped action; the caller's `on_press` runs when it is activated.
///
/// The caller performs navigation or another action in its `on_press`.
#[component]
pub fn link(
    label: Value<String>,
    #[prop(default = false)] as_button: Value<bool>,
    #[prop(default = false)] underline: Value<bool>,
    #[prop(default = false)] external: Value<bool>,
    new_tab_label: Option<Value<String>>,
    #[prop(default = false)] standalone: Value<bool>,
    #[prop(default = TextKind::Body)] text_type: Value<TextKind>,
    text_size: Option<Value<f64>>,
    weight: Option<u16>,
    #[prop(default = LinkColor::Accent)] color: Value<LinkColor>,
    #[prop(default = 0)] max_lines: Value<u32>,
    #[caller] caller: Caller,
) -> impl Element {
    let interactive = caller.has_press();
    let attachment = part(|state| state);
    let input = attachment.state();
    let root = row(()).with_node(move |node| node.part(attachment));
    let root = activate_link(root, interactive, as_button);
    let root = name_link(root, label, external, new_tab_label);
    let direction = if use_direction() == ReadingDirection::Rtl {
        FlexDirection::RowReverse
    } else {
        FlexDirection::Row
    };
    let appearance = LinkAppearance {
        kind: text_type.get(),
        size: text_size,
        weight,
        underline: underline.get(),
        color: color.get(),
        standalone: standalone.get(),
        direction,
        interactive,
    };
    root.with_node(move |node| node.base_style(move || appearance.paint(input)))
        .child(row(text(label)).style(move || style().min_width(0).line_clamp(max_lines.get())))
        .child(
            external
                .get()
                .then(|| icon_box(icon("external-link").into_view(), 12.0)),
        )
}

/// Make the root a focusable link or button when the caller gave it a press handler.
/// The handler itself is the caller's `on_press` on the same root.
fn activate_link(mut root: Node, interactive: bool, as_button: Value<bool>) -> Node {
    if interactive {
        root = root
            .with_node(move |node| {
                node.role(if as_button.get() {
                    Role::Button
                } else {
                    Role::Link
                })
            })
            .focusable(true);
        if !as_button.get() {
            root = root.on_key(|event| {
                if event.key() == Key::Space {
                    Handled::Yes
                } else {
                    Handled::No
                }
            });
        }
    }
    root
}

fn name_link(
    root: Node,
    label: Value<String>,
    external: Value<bool>,
    new_tab_label: Option<Value<String>>,
) -> Node {
    let label = if external.get() {
        let suffix = new_tab_label
            .unwrap_or_else(|| kivact::i18n::message("soft.link.new_tab", "(opens in new tab)"));
        memo(move || format!("{} {}", label.get(), suffix.get()))
    } else {
        label
    };
    root.name(label)
}

#[derive(Clone, Copy)]
struct LinkAppearance {
    kind: TextKind,
    size: Option<Value<f64>>,
    weight: Option<u16>,
    underline: bool,
    color: LinkColor,
    standalone: bool,
    direction: FlexDirection,
    interactive: bool,
}

impl LinkAppearance {
    fn paint(self, input: Value<PartState>) -> Style {
        let mut base = style()
            .flex_direction(self.direction)
            .align_items(Align::Center)
            .gap(2)
            .min_width(0)
            .max_width(pct(100));
        if let Some(metrics) = self.metrics() {
            base = base
                .font_family(metrics.family)
                .font_size(metrics.size_px)
                .font_weight(metrics.weight)
                .line_height(metrics.line_height_px / metrics.size_px);
        }
        if let Some(ink) = self.ink() {
            base = base.color(ink);
        }
        if let Some(size) = self.size {
            base = base.font_size(size.get());
        }
        if let Some(weight) = self.weight {
            base = base.font_weight(weight);
        }
        let hovered = self.interactive && !input.get().is_disabled() && input.get().is_hovered();
        if self.underline || hovered {
            base = base.text_decoration(TextDecorationLine::UNDERLINE);
        }
        if input.get().is_disabled() {
            base.opacity(m::DISABLED_OPACITY).cursor(Cursor::Default)
        } else if self.interactive {
            base.cursor(Cursor::Pointer)
                .active(style().background(c::OVERLAY_PRESSED))
                .focus_visible(ring(c::FOCUS_RING))
        } else {
            base
        }
    }

    fn metrics(self) -> Option<TypeStyle> {
        match self.kind {
            TextKind::Body => Some(t::BODY),
            TextKind::Label => Some(t::LABEL),
            TextKind::Supporting => Some(t::SUPPORTING),
            TextKind::Large => Some(t::LARGE),
            TextKind::Code => Some(t::CODE),
            TextKind::Display1 => Some(t::DISPLAY1),
            TextKind::Display2 => Some(t::DISPLAY2),
            TextKind::Display3 => Some(t::DISPLAY3),
            TextKind::Inherit if self.standalone => Some(t::BODY),
            TextKind::Inherit => None,
        }
    }

    fn ink(self) -> Option<Color> {
        match self.color {
            LinkColor::Accent => Some(c::TEXT_ACCENT),
            LinkColor::Primary => Some(c::TEXT_PRIMARY),
            LinkColor::Secondary | LinkColor::Placeholder => Some(c::TEXT_SECONDARY),
            LinkColor::Disabled => Some(c::TEXT_DISABLED),
            LinkColor::Inherit => None,
        }
    }
}
