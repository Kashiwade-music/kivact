//! Role: Render themed body, supporting, code, and display text.

use crate::{Tokens, TypeStyle, TypeTokens};
use kivact::state::Value;
use kivact::view::Element;
use kivact::view::Node;
use kivact::view::text as control_text;

use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::style::Style;
use kivact::style::TextAlign;
use kivact::style::TextDecorationLine;
use kivact::style::style;
use kivact::ui::tooltip;
use kivact::view::IntoView;
use kivact::view::Placement;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::node_ref;
use kivact::view::use_direction;

/// Named typography roles resolved from the current Soft theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TextKind {
    #[default]
    /// Use body typography; the default.
    Body,
    /// Use large body typography.
    Large,
    /// Use label typography.
    Label,
    /// Use supporting typography and secondary text color.
    Supporting,
    /// Use code typography and its configured font family.
    Code,
    /// Use the first display typography scale.
    Display1,
    /// Use the second display typography scale.
    Display2,
    /// Use the third display typography scale.
    Display3,
    /// Preserve inherited metrics and color unless explicitly overridden.
    Inherit,
}

/// Theme font-size steps that override a typography role’s size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextSize {
    /// Use the `size_4xs` font-size token in logical pixels.
    Xs4,
    /// Use the `size_3xs` font-size token in logical pixels.
    Xs3,
    /// Use the `size_2xs` font-size token in logical pixels.
    Xs2,
    /// Use the `size_xs` font-size token in logical pixels.
    Xs,
    /// Use the `size_sm` font-size token in logical pixels.
    Sm,
    /// Use the `size_md` font-size token in logical pixels.
    Base,
    /// Use the `size_lg` font-size token in logical pixels.
    Lg,
    /// Use the `size_xl` font-size token in logical pixels.
    Xl,
    /// Use the `size_2xl` font-size token in logical pixels.
    Xl2,
    /// Use the `size_3xl` font-size token in logical pixels.
    Xl3,
    /// Use the `size_4xl` font-size token in logical pixels.
    Xl4,
}

/// Semantic text colors resolved from Soft theme tokens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextColor {
    /// Use the primary text color.
    Primary,
    /// Use the secondary text color.
    Secondary,
    /// Use the disabled text color without changing interaction semantics.
    Disabled,
    /// Use the secondary text color for placeholder presentation.
    Placeholder,
    /// Use the accent text color.
    Accent,
    /// Leave the inherited text color unchanged.
    Inherit,
}

/// Theme font-weight choices for styled text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWeight {
    /// Use the theme’s normal weight.
    Normal,
    /// Use the theme’s medium weight.
    Medium,
    /// Use the theme’s semibold weight.
    Semibold,
    /// Use the theme’s bold weight.
    Bold,
}

/// Placement of the tooltip shown only when limited text is actually truncated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TruncateTooltip {
    /// Do not attach a truncation tooltip.
    Off,
    #[default]
    /// Place the truncation tooltip above the text; the default.
    Above,
    /// Place the truncation tooltip below the text.
    Below,
    /// Place the tooltip at the logical leading side.
    Start,
    /// Place the tooltip at the logical trailing side.
    End,
}

/// Word-breaking requests; both currently follow native wrapping rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordBreak {
    /// Request word breaking through the native wrapping rules.
    BreakWord,
    /// Request character breaking; currently uses the same native rules as BreakWord.
    BreakAll,
}

/// Line-wrapping requests supported by styled text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWrap {
    /// Enable native line wrapping.
    Wrap,
    /// Disable line wrapping.
    NoWrap,
    /// Enable wrapping; currently uses the same native rules as Wrap.
    Balance,
    /// Enable wrapping; currently uses the same native rules as Wrap.
    Pretty,
}

/// Text alignment relative to the current reading direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TextJustify {
    #[default]
    /// Align to the logical text start; the default.
    Start,
    /// Center the text within its line box.
    Center,
    /// Align to the logical text end.
    End,
}

/// Reactive text with a Soft typography role and optional truncation tooltip.
pub struct StyledText {
    root: Node,
    value: Value<String>,
    kind: Value<TextKind>,
    heading: Option<u32>,
    size: Option<Value<TextSize>>,
    color: Option<Value<TextColor>>,
    weight: Option<Value<TextWeight>>,
    max_lines: Value<u32>,
    truncate_tooltip: Value<TruncateTooltip>,
    wrap: Option<TextWrap>,
    justify: Value<TextJustify>,
    strikethrough: Value<bool>,
    tabular_numbers: Value<bool>,
    capsize: Value<bool>,
}

/// Show constant or reactive body text, with unlimited lines and logical start alignment.
#[track_caller]
pub fn styled_text(value: impl IntoText) -> StyledText {
    let value = value.into_text_value();
    StyledText {
        root: control_text(value),
        value,
        kind: (TextKind::default()).into_value(),
        heading: None,
        size: None,
        color: None,
        weight: None,
        max_lines: 0.into_value(),
        truncate_tooltip: (TruncateTooltip::default()).into_value(),
        wrap: None,
        justify: (TextJustify::default()).into_value(),
        strikethrough: false.into_value(),
        tabular_numbers: (false).into_value(),
        capsize: (false).into_value(),
    }
}

impl StyledText {
    /// Set a constant or reactive typography role; defaults to body.
    pub fn kind(mut self, value: impl IntoValue<TextKind>) -> Self {
        self.kind = value.into_value();
        self
    }

    /// Override font size with a constant or reactive theme size step.
    pub fn size(mut self, value: impl IntoValue<TextSize>) -> Self {
        self.size = Some(value.into_value());
        self
    }

    /// Override the role’s text color with a constant or reactive semantic color.
    pub fn color(mut self, value: impl IntoValue<TextColor>) -> Self {
        self.color = Some(value.into_value());
        self
    }

    /// Override the role’s weight with a constant or reactive theme weight.
    pub fn weight(mut self, value: impl IntoValue<TextWeight>) -> Self {
        self.weight = Some(value.into_value());
        self
    }

    /// Set a reactive line limit: zero is unlimited, one is single-line, larger values clamp.
    pub fn max_lines(mut self, value: impl IntoValue<u32>) -> Self {
        self.max_lines = value.into_value();
        self
    }

    /// Choose tooltip placement at view construction; attached only when the initial line limit is nonzero.
    pub fn truncate_tooltip(mut self, value: impl IntoValue<TruncateTooltip>) -> Self {
        self.truncate_tooltip = value.into_value();
        self
    }

    /// Choose native wrapping; Balance and Pretty currently behave like Wrap.
    pub fn wrap(mut self, value: TextWrap) -> Self {
        self.wrap = Some(value);
        self
    }

    /// Set constant or reactive alignment, respecting the current reading direction.
    pub fn justify(mut self, value: impl IntoValue<TextJustify>) -> Self {
        self.justify = value.into_value();
        self
    }

    /// Reactively enable equal-width number glyphs; disabled by default.
    pub fn tabular_numbers(mut self, value: impl IntoValue<bool>) -> Self {
        self.tabular_numbers = value.into_value();
        self
    }

    /// Request word breaking; both choices currently leave native wrapping unchanged.
    pub fn word_break(self, _value: WordBreak) -> Self {
        self
    }

    /// Trim the text box to the first line’s cap height and last baseline.
    pub fn capsize(mut self, on: impl IntoValue<bool>) -> Self {
        self.capsize = on.into_value();
        self
    }

    /// Set a static or reactive strikethrough.
    pub fn strikethrough(mut self, on: impl IntoValue<bool>) -> Self {
        self.strikethrough = on.into_value();
        self
    }

    /// Give the text heading semantics and a reactive level without changing typography.
    pub fn heading_level(mut self, level: impl IntoValue<u32>) -> Self {
        self.root = self
            .root
            .with_node(move |node| node.role(Role::Heading))
            .with_node(move |node| node.level(level))
            .name(self.value);
        self
    }
}

impl Element for StyledText {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for StyledText {
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
        let root = root.with_node(move |node| {
            node.base_style(move || {
                let tokens = kivact::style::use_tokens::<crate::Tokens>();
                let mut look = type_style(
                    &tokens,
                    kind.get(),
                    heading,
                    size.map(|v| v.get()),
                    color.map(|v| v.get()),
                    weight.map(|v| v.get()),
                );
                if capsize.get() {
                    look = look.text_capsize(true);
                }
                look = look.text_align(match (justify.get(), use_direction().get()) {
                    (TextJustify::Start, ReadingDirection::Ltr)
                    | (TextJustify::End, ReadingDirection::Rtl) => TextAlign::Start,
                    (TextJustify::Center, _) => TextAlign::Center,
                    _ => TextAlign::End,
                });
                if let Some(wrap) = wrap {
                    look = look.text_wrap(wrap != TextWrap::NoWrap);
                }
                look = match max_lines.get() {
                    0 => look,
                    1 => look.single_line(),
                    n => look.min_width(0).line_clamp(n),
                };
                if strikethrough.get() {
                    look = look.text_decoration(TextDecorationLine::LINE_THROUGH);
                }
                if tabular_numbers.get() {
                    look = look.tabular_nums();
                }
                look
            })
        });
        let placement = match truncate_tooltip.get() {
            TruncateTooltip::Off => None,
            TruncateTooltip::Above => Some(Placement::Above),
            TruncateTooltip::Below => Some(Placement::Below),
            TruncateTooltip::Start => Some(Placement::Start),
            TruncateTooltip::End => Some(Placement::End),
        };
        match placement.filter(|_| max_lines.get() > 0) {
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

pub(crate) fn type_style(
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

fn metric_style(metrics: TypeStyle) -> Style {
    style()
        .font_size(metrics.size_px)
        .line_height(metrics.line_height_px / metrics.size_px)
        .font_family(metrics.family)
        .weight(metrics.weight)
}

fn font_size(t: &TypeTokens, size: TextSize) -> f64 {
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

kivact::state::constant_value!(
    TextColor,
    TextJustify,
    TextKind,
    TextSize,
    TextWeight,
    TruncateTooltip
);
