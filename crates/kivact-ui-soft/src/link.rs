//! Role: Neutral text links with explicit native activation.
use kivact::state::IntoValue;

use crate::TextKind;
use crate::feedback::icon_box;
use crate::recipe_style::ring;
use kivact::custom::part;
use kivact::state::Callback;
use kivact::state::IntoText;
use kivact::state::Value;
use kivact::state::callback;
use kivact::state::memo;
use kivact::style::Align;
use kivact::style::Cursor;
use kivact::style::Direction;
use kivact::style::TextDecorationLine;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::Element;
use kivact::view::Handled;
use kivact::view::IntoView;
use kivact::view::Key;
use kivact::view::Node;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::icon;
use kivact::view::row;
use kivact::view::text;
use kivact::view::use_direction;

/// Foreground of a text link.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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

/// Inline-sized link or link-shaped action.
pub struct Link {
    root: Node,
    label: Option<Value<String>>,
    content: View,
    press: Option<Callback<()>>,
    new_tab_label: Option<Value<String>>,
    underline: Value<bool>,
    external: Value<bool>,
    standalone: Value<bool>,
    kind: Value<TextKind>,
    size: Option<Value<f64>>,
    weight: Option<u16>,
    color: Value<LinkColor>,
    max_lines: Value<u32>,
    as_button: Value<bool>,
}

#[track_caller]
/// Create a text link with the supplied visible and accessible label.
pub fn link(label: impl IntoText) -> Link {
    let label = label.into_text_value();
    let mut link = link_with(text(label));
    link.label = Some(label);
    link
}

#[track_caller]
/// Mount custom link content once; use `name` to give it an accessible label.
pub fn link_with(content: impl IntoView) -> Link {
    Link {
        root: row(()),
        label: None,
        content: content.into_view(),
        press: None,
        new_tab_label: None,
        underline: (false).into_value(),
        external: (false).into_value(),
        standalone: (false).into_value(),
        kind: (TextKind::Body).into_value(),
        size: None,
        weight: None,
        color: (LinkColor::Accent).into_value(),
        max_lines: 0.into_value(),
        as_button: (false).into_value(),
    }
}

impl Link {
    /// Set the accessible label independently of custom visible content.
    pub fn name(mut self, label: impl IntoText) -> Self {
        self.label = Some(label.into_text_value());
        self
    }

    /// Handle activation; the caller performs navigation or another action.
    pub fn on_press(mut self, press: impl Into<Callback<()>>) -> Self {
        let press: Callback<()> = press.into();
        self.press = Some(callback(move |_| press.call(())));
        self
    }

    /// Use the Space and Enter behavior of an action button.
    pub fn as_button(mut self, value: impl IntoValue<bool>) -> Self {
        self.as_button = value.into_value();
        self
    }

    /// Show or hide the link's underline.
    pub fn underline(mut self, value: impl IntoValue<bool>) -> Self {
        self.underline = value.into_value();
        self
    }

    /// Show the external-link indicator and announce the new-tab label.
    pub fn external(mut self, value: impl IntoValue<bool>) -> Self {
        self.external = value.into_value();
        self
    }

    /// Override the accessible text appended for an external link.
    pub fn new_tab_label(mut self, label: impl IntoText) -> Self {
        self.new_tab_label = Some(label.into_text_value());
        self
    }

    /// Use body typography when the selected text role is `TextKind::Inherit`.
    pub fn standalone(mut self, value: impl IntoValue<bool>) -> Self {
        self.standalone = value.into_value();
        self
    }

    /// Choose the theme typography role for the link.
    pub fn text_type(mut self, kind: impl IntoValue<TextKind>) -> Self {
        self.kind = kind.into_value();
        self
    }

    /// Override the text size in pixels, including reactive changes.
    pub fn text_size(mut self, px: impl IntoValue<f64>) -> Self {
        self.size = Some(px.into_value());
        self
    }

    /// Override the text weight.
    pub fn weight(mut self, weight: u16) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Choose the foreground palette role.
    pub fn color(mut self, color: impl IntoValue<LinkColor>) -> Self {
        self.color = color.into_value();
        self
    }

    /// Limit the visible line count; zero leaves it unrestricted.
    pub fn max_lines(mut self, lines: impl IntoValue<u32>) -> Self {
        self.max_lines = lines.into_value();
        self
    }
}

impl Element for Link {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Link {
    fn into_view(self) -> View {
        let interactive = self.press.is_some();
        let attachment = part(|state| state);
        let input = attachment.state();
        let mut root = self.root.with_node(move |node| node.part(attachment));
        root = activate_link(root, self.press, self.as_button);
        root = name_link(root, self.label, self.external, self.new_tab_label);
        let direction = if use_direction().get() == ReadingDirection::Rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        };
        let appearance = LinkAppearance {
            kind: self.kind.get(),
            size: self.size,
            weight: self.weight,
            underline: self.underline.get(),
            color: self.color.get(),
            standalone: self.standalone.get(),
            direction,
            interactive,
        };
        let lines = self.max_lines;
        root.with_node(move |node| node.base_style(move || appearance.paint(input)))
            .child(row(self.content).style(move || style().min_width(0).line_clamp(lines.get())))
            .child(
                self.external
                    .get()
                    .then(|| icon_box(icon("external-link").into_view(), 12.0)),
            )
            .into_view()
    }
}

kivact::state::constant_value!(LinkColor);

fn activate_link(mut root: Node, press: Option<Callback<()>>, as_button: Value<bool>) -> Node {
    if let Some(press) = press {
        root = root
            .with_node(move |node| {
                node.role(if as_button.get() {
                    Role::Button
                } else {
                    Role::Link
                })
            })
            .focusable(true)
            .on_press(move || press.call(()));
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
    mut root: Node,
    label: Option<Value<String>>,
    external: Value<bool>,
    new_tab_label: Option<Value<String>>,
) -> Node {
    if let Some(label) = label {
        let label = if external.get() {
            let suffix = new_tab_label.unwrap_or_else(|| {
                kivact::i18n::message("soft.link.new_tab", "(opens in new tab)")
            });
            memo(move || format!("{} {}", label.get(), suffix.get()))
        } else {
            label
        };
        root = root.name(label);
    }
    root
}
#[derive(Clone, Copy)]
struct LinkAppearance {
    kind: TextKind,
    // xtask-allow(state-fields): option: readonly font-size override shared with paint.
    size: Option<Value<f64>>,
    // xtask-allow(state-fields): option: immutable font-weight override.
    weight: Option<u16>,
    // xtask-allow(state-fields): option: caller choice from Link::underline sampled once in into_view; false underlines only on hover of an interactive link.
    underline: bool,
    color: LinkColor,
    // xtask-allow(state-fields): option: caller choice from Link::standalone sampled once in into_view; false inherits the surrounding typography.
    standalone: bool,
    direction: Direction,
    // xtask-allow(state-fields): option: whether the caller gave Link an activation callback; false paints a static link without pointer cursor or pressed state.
    interactive: bool,
}
impl LinkAppearance {
    fn paint(self, input: Value<kivact::custom::PartState>) -> kivact::style::Style {
        let Self {
            kind: _,
            size,
            weight,
            underline,
            color: _,
            standalone: _,
            direction,
            interactive,
        } = self;
        let t = kivact::style::use_tokens::<crate::Tokens>();
        let metrics = self.metrics(&t);
        let ink = self.ink(&t);
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
            base = base.font_size(size.get());
        }
        if let Some(weight) = weight {
            base = base.weight(weight);
        }
        if underline || (interactive && !input.get().disabled && input.get().hovered) {
            base = base.text_decoration(TextDecorationLine::UNDERLINE);
        }
        if input.get().disabled {
            base.opacity(t.sizes.disabled_opacity)
                .cursor(Cursor::Default)
        } else if interactive {
            base.cursor(Cursor::Pointer)
                .pressed(style().background(t.colors.overlay_pressed))
                .focus_visible(ring(&t, t.colors.focus_ring))
        } else {
            base
        }
    }
    fn metrics(self, t: &crate::Tokens) -> Option<crate::TypeStyle> {
        let typography = &t.typography;
        let kind = self.kind;
        let standalone = self.standalone;
        match kind {
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
        }
    }
    fn ink(self, t: &crate::Tokens) -> Option<kivact::style::Color> {
        let color = self.color;
        match color {
            LinkColor::Accent => Some(t.colors.text_accent),
            LinkColor::Primary => Some(t.colors.text_primary),
            LinkColor::Secondary | LinkColor::Placeholder => Some(t.colors.text_secondary),
            LinkColor::Disabled => Some(t.colors.text_disabled),
            LinkColor::Inherit => None,
        }
    }
}
