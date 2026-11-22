//! Role: Render themed body, supporting, code, and display text.

use crate::color as c;
use crate::type_scale::{self as t, TypeStyle};
use kivact::Prop;
use kivact::component;
use kivact::state::Value;
use kivact::style::Style;
use kivact::style::TextAlign;
use kivact::style::TextDecorationLine;
use kivact::style::style;
use kivact::ui::{Tooltip, tooltip};
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::node_ref;
use kivact::view::text as control_text;
use kivact::view::use_direction;
use kivact::view::{Element, IntoView, Node, Placement, View};

/// Named typography roles resolved from the current Soft theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Prop)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Prop)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Prop)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Prop)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Prop)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Prop)]
pub enum TextJustify {
    #[default]
    /// Align to the logical text start; the default.
    Start,
    /// Center the text within its line box.
    Center,
    /// Align to the logical text end.
    End,
}

/// Show constant or reactive body text: a Soft typography role, an optional
/// truncation tooltip, and an optional heading level.
#[component]
pub fn styled_text(
    value: Value<String>,
    #[prop(default = TextKind::Body)] kind: Value<TextKind>,
    size: Option<Value<TextSize>>,
    color: Option<Value<TextColor>>,
    weight: Option<Value<TextWeight>>,
    #[prop(default = 0u32)] max_lines: Value<u32>,
    #[prop(default = TruncateTooltip::Above)] truncate_tooltip: Value<TruncateTooltip>,
    wrap: Option<TextWrap>,
    #[prop(default = TextJustify::Start)] justify: Value<TextJustify>,
    #[prop(default = false)] strikethrough: Value<bool>,
    #[prop(default = false)] tabular_numbers: Value<bool>,
    #[prop(default = false)] capsize: Value<bool>,
    heading_level: Option<Value<u32>>,
) -> impl Element {
    let mut root = control_text(value);
    if let Some(level) = heading_level {
        root = root
            .with_node(|node| node.role(Role::Heading))
            .with_node(move |node| node.level(level))
            .name(value);
    }
    let root = root.with_node(move |node| {
        node.base_style(move || {
            let mut look = type_style(
                kind.get(),
                size.map(|v| v.get()),
                color.map(|v| v.get()),
                weight.map(|v| v.get()),
            );
            if capsize.get() {
                look = look.text_capsize(true);
            }
            look = look.text_align(match (justify.get(), use_direction()) {
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
            TextBody::Tipped(
                tooltip(value, root.node_ref(anchor))
                    .placement(placement)
                    .enabled(anchor.text_truncated()),
            )
        }
        None => TextBody::Plain(root),
    }
}

/// The element `styled_text` returns: the text node itself, with its
/// truncation tooltip when one is asked for, so a caller's `node_ref` measures
/// the text and not a wrapper.
enum TextBody {
    Plain(Node),
    Tipped(Tooltip<Node>),
}

impl Element for TextBody {
    fn node_mut(&mut self) -> &mut Node {
        match self {
            Self::Plain(node) => node,
            Self::Tipped(tipped) => tipped.node_mut(),
        }
    }
}

impl IntoView for TextBody {
    fn into_view(self) -> View {
        match self {
            Self::Plain(node) => node.into_view(),
            Self::Tipped(tipped) => tipped.into_view(),
        }
    }
}

/// The style of a Soft typography role with its optional overrides.
pub(crate) fn type_style(
    kind: TextKind,
    size: Option<TextSize>,
    color: Option<TextColor>,
    weight: Option<TextWeight>,
) -> Style {
    let metrics = match kind {
        TextKind::Body => Some(t::BODY),
        TextKind::Large => Some(t::LARGE),
        TextKind::Label => Some(t::LABEL),
        TextKind::Supporting => Some(t::SUPPORTING),
        TextKind::Code => Some(t::CODE),
        TextKind::Display1 => Some(t::DISPLAY1),
        TextKind::Display2 => Some(t::DISPLAY2),
        TextKind::Display3 => Some(t::DISPLAY3),
        TextKind::Inherit => None,
    };
    let mut look = metrics.map_or_else(style, metric_style);
    if let Some(size) = size {
        look = look.font_size(font_size(size));
    }
    if let Some(weight) = weight {
        look = look.font_weight(match weight {
            TextWeight::Normal => t::WEIGHT_NORMAL,
            TextWeight::Medium => t::WEIGHT_MEDIUM,
            TextWeight::Semibold => t::WEIGHT_SEMIBOLD,
            TextWeight::Bold => t::WEIGHT_BOLD,
        });
    }
    let color = color.unwrap_or(match kind {
        TextKind::Supporting => TextColor::Secondary,
        TextKind::Inherit => TextColor::Inherit,
        _ => TextColor::Primary,
    });
    match color {
        TextColor::Primary => look.color(c::TEXT_PRIMARY),
        TextColor::Secondary | TextColor::Placeholder => look.color(c::TEXT_SECONDARY),
        TextColor::Disabled => look.color(c::TEXT_DISABLED),
        TextColor::Accent => look.color(c::TEXT_ACCENT),
        TextColor::Inherit => look,
    }
}

fn metric_style(metrics: TypeStyle) -> Style {
    style()
        .font_size(metrics.size_px)
        .line_height(metrics.line_height_px / metrics.size_px)
        .font_family(metrics.family)
        .font_weight(metrics.weight)
}

fn font_size(size: TextSize) -> f64 {
    match size {
        TextSize::Xs4 => t::SIZE_4XS,
        TextSize::Xs3 => t::SIZE_3XS,
        TextSize::Xs2 => t::SIZE_2XS,
        TextSize::Xs => t::SIZE_XS,
        TextSize::Sm => t::SIZE_SM,
        TextSize::Base => t::SIZE_MD,
        TextSize::Lg => t::SIZE_LG,
        TextSize::Xl => t::SIZE_XL,
        TextSize::Xl2 => t::SIZE_2XL,
        TextSize::Xl3 => t::SIZE_3XL,
        TextSize::Xl4 => t::SIZE_4XL,
    }
}
