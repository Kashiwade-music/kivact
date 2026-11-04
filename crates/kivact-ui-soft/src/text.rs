//! Role: Render themed body, supporting, code, and display text.

use crate::{TextStyle, Tokens, Typography, tokens};
use kivact::Value;
use kivact::control::{Node, NodeBuilder, text as control_text};
use kivact::primitive::NodeExt;
use kivact::style::{Style, TextAlign, TextDecorationLine, style};
use kivact::ui::tooltip;
use kivact::view::{
    IntoText, IntoValue, IntoView, Placement, ReadingDirection, Role, View, current_direction,
    node_ref,
};

/// The text kind option.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TextKind {
    #[default]
    Body,
    Large,
    Label,
    Supporting,
    Code,
    Display1,
    Display2,
    Display3,
    Inherit,
}

/// The text size option.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextSize {
    Xs4,
    Xs3,
    Xs2,
    Xs,
    Sm,
    Base,
    Lg,
    Xl,
    Xl2,
    Xl3,
    Xl4,
}

/// The text color option.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextColor {
    Primary,
    Secondary,
    Disabled,
    Placeholder,
    Accent,
    Inherit,
}

/// The text weight option.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWeight {
    Normal,
    Medium,
    Semibold,
    Bold,
}

/// The truncate tooltip option.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TruncateTooltip {
    Off,
    #[default]
    Above,
    Below,
    Start,
    End,
}

/// The word break option.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordBreak {
    BreakWord,
    BreakAll,
}

/// The text wrap option.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWrap {
    Wrap,
    NoWrap,
    Balance,
    Pretty,
}

/// The text justify option.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TextJustify {
    #[default]
    Start,
    Center,
    End,
}

/// A text node with an Astryx Neutral typography role.
pub struct Text {
    root: Node,
    value: Value<String>,
    kind: TextKind,
    heading: Option<u32>,
    size: Option<TextSize>,
    color: Option<TextColor>,
    weight: Option<TextWeight>,
    max_lines: u32,
    truncate_tooltip: TruncateTooltip,
    wrap: Option<TextWrap>,
    justify: TextJustify,
    strikethrough: Value<bool>,
    tabular_numbers: bool,
    capsize: bool,
}

/// Show a string or a reactive text value.
#[track_caller]
pub fn text(value: impl IntoText) -> Text {
    let value = value.into_text_value();
    Text {
        root: control_text(value),
        value,
        kind: TextKind::default(),
        heading: None,
        size: None,
        color: None,
        weight: None,
        max_lines: 0,
        truncate_tooltip: TruncateTooltip::default(),
        wrap: None,
        justify: TextJustify::default(),
        strikethrough: false.into_value(),
        tabular_numbers: false,
        capsize: false,
    }
}

impl Text {
    /// Set the kind.
    pub fn kind(mut self, value: TextKind) -> Self {
        self.kind = value;
        self
    }

    /// Set the size.
    pub fn size(mut self, value: TextSize) -> Self {
        self.size = Some(value);
        self
    }

    /// Set the color.
    pub fn color(mut self, value: TextColor) -> Self {
        self.color = Some(value);
        self
    }

    /// Set the weight.
    pub fn weight(mut self, value: TextWeight) -> Self {
        self.weight = Some(value);
        self
    }

    /// Set the max lines.
    pub fn max_lines(mut self, value: u32) -> Self {
        self.max_lines = value;
        self
    }

    /// Set the truncate tooltip.
    pub fn truncate_tooltip(mut self, value: TruncateTooltip) -> Self {
        self.truncate_tooltip = value;
        self
    }

    /// Set the wrap.
    pub fn wrap(mut self, value: TextWrap) -> Self {
        self.wrap = Some(value);
        self
    }

    /// Set the justify.
    pub fn justify(mut self, value: TextJustify) -> Self {
        self.justify = value;
        self
    }

    /// Set the tabular numbers.
    pub fn tabular_numbers(mut self, value: bool) -> Self {
        self.tabular_numbers = value;
        self
    }

    /// Request a word break mode. Both use the native wrapping rules.
    pub fn word_break(self, _value: WordBreak) -> Self {
        self
    }

    /// Trim the text box to the first line’s cap height and last baseline.
    pub fn capsize(mut self, on: bool) -> Self {
        self.capsize = on;
        self
    }

    /// Set a static or reactive strikethrough.
    pub fn strikethrough(mut self, on: impl IntoValue<bool>) -> Self {
        self.strikethrough = on.into_value();
        self
    }

    /// Give the text heading semantics without changing its appearance.
    pub fn heading_level(mut self, level: u32) -> Self {
        self.root = self.root.role(Role::Heading).level(level).name(self.value);
        self
    }

    pub(crate) fn heading_appearance(mut self, level: u32) -> Self {
        self.heading = Some(level);
        self
    }
}

impl NodeBuilder for Text {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Text {
    fn into_view(self) -> View {
        let Self {
            root,
            value,
            kind,
            heading,
            size,
            color,
            weight,
            max_lines,
            truncate_tooltip,
            wrap,
            justify,
            strikethrough,
            tabular_numbers,
            capsize,
        } = self;
        let root = root.base_style(move || {
            let tokens = tokens();
            let mut look = type_style(&tokens, kind, heading, size, color, weight);
            if capsize {
                look = look.text_capsize(true);
            }
            look = look.text_align(match (justify, current_direction()) {
                (TextJustify::Start, ReadingDirection::Ltr)
                | (TextJustify::End, ReadingDirection::Rtl) => TextAlign::Start,
                (TextJustify::Center, _) => TextAlign::Center,
                _ => TextAlign::End,
            });
            if let Some(wrap) = wrap {
                look = look.text_wrap(wrap != TextWrap::NoWrap);
            }
            look = match max_lines {
                0 => look,
                1 => look.single_line(),
                n => look.min_width(0).line_clamp(n),
            };
            if strikethrough.get() {
                look = look.text_decoration(TextDecorationLine::LINE_THROUGH);
            }
            if tabular_numbers {
                look = look.tabular_nums();
            }
            look
        });
        let placement = match truncate_tooltip {
            TruncateTooltip::Off => None,
            TruncateTooltip::Above => Some(Placement::Above),
            TruncateTooltip::Below => Some(Placement::Below),
            TruncateTooltip::Start => Some(Placement::Start),
            TruncateTooltip::End => Some(Placement::End),
        };
        match placement.filter(|_| max_lines > 0) {
            Some(placement) => {
                let anchor = node_ref();
                tooltip(value)
                    .placement(placement)
                    .enabled(anchor.text_truncated())
                    .attach(root.node_ref(anchor))
            }
            None => root.into_view(),
        }
    }
}

fn type_style(
    tokens: &Tokens,
    kind: TextKind,
    heading: Option<u32>,
    size: Option<TextSize>,
    color: Option<TextColor>,
    weight: Option<TextWeight>,
) -> Style {
    let t = &tokens.typography;
    let metrics = match kind {
        TextKind::Body => Some(match heading {
            Some(1) => t.heading1,
            Some(2) => t.heading2,
            Some(3) => t.heading3,
            Some(4) => t.heading4,
            Some(5) => t.heading5,
            Some(_) => t.heading6,
            None => t.body,
        }),
        TextKind::Large => Some(t.large),
        TextKind::Label => Some(t.label),
        TextKind::Supporting => Some(t.supporting),
        TextKind::Code => Some(t.code),
        TextKind::Display1 => Some(t.display1),
        TextKind::Display2 => Some(t.display2),
        TextKind::Display3 => Some(t.display3),
        TextKind::Inherit => None,
    };
    let mut look = metrics.map_or_else(style, metric_style);
    if let Some(size) = size {
        look = look.font_size(font_size(t, size));
    }
    if let Some(weight) = weight {
        look = look.weight(match weight {
            TextWeight::Normal => t.weight_normal,
            TextWeight::Medium => t.weight_medium,
            TextWeight::Semibold => t.weight_semibold,
            TextWeight::Bold => t.weight_bold,
        });
    }
    let p = &tokens.colors;
    let color = color.unwrap_or(match kind {
        TextKind::Supporting => TextColor::Secondary,
        TextKind::Inherit => TextColor::Inherit,
        _ => TextColor::Primary,
    });
    match color {
        TextColor::Primary => look.color(p.text_primary),
        TextColor::Secondary | TextColor::Placeholder => look.color(p.text_secondary),
        TextColor::Disabled => look.color(p.text_disabled),
        TextColor::Accent => look.color(p.text_accent),
        TextColor::Inherit => look,
    }
}

fn metric_style(metrics: TextStyle) -> Style {
    style()
        .font_size(metrics.size_px)
        .line_height(metrics.line_height_px / metrics.size_px)
        .font_family(metrics.family)
        .weight(metrics.weight)
}

fn font_size(t: &Typography, size: TextSize) -> f32 {
    match size {
        TextSize::Xs4 => t.size_4xs,
        TextSize::Xs3 => t.size_3xs,
        TextSize::Xs2 => t.size_2xs,
        TextSize::Xs => t.size_xs,
        TextSize::Sm => t.size_sm,
        TextSize::Base => t.size_md,
        TextSize::Lg => t.size_lg,
        TextSize::Xl => t.size_xl,
        TextSize::Xl2 => t.size_2xl,
        TextSize::Xl3 => t.size_3xl,
        TextSize::Xl4 => t.size_4xl,
    }
}
