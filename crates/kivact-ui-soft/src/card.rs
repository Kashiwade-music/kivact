//! Role: Style canonical cards and supply layout inset context without a card wrapper.
use crate::layout::LayoutInset;
use crate::stack::ContainerInset;
use crate::{Elevation, Space};
use kivact::draw::Shadow;
use kivact::motion::Transition;
use kivact::state::{IntoValue, Value, provide};
use kivact::style::{Color, Direction, Position, Style, Theme, TransitionProp, style};
use kivact::ui::{CardRecipes, CardState, Look};
use kivact::view::IntoView;
/// Background palette for a canonical card.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardVariant {
    /// Default card surface.
    #[default]
    Default,
    /// Transparent surface.
    Transparent,
    /// Muted neutral surface.
    Muted,
    /// Blue palette.
    Blue,
    /// Cyan palette.
    Cyan,
    /// Gray palette.
    Gray,
    /// Green palette.
    Green,
    /// Orange palette.
    Orange,
    /// Pink palette.
    Pink,
    /// Purple palette.
    Purple,
    /// Red palette.
    Red,
    /// Teal palette.
    Teal,
    /// Yellow palette.
    Yellow,
}

impl kivact::ui::Variant for CardVariant {}
/// Independent reactive visual choices applied to the standard card recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardAppearance {
    /// Read-only reactive surface palette.
    pub variant: Value<CardVariant>,
    /// Read-only reactive shadow elevation.
    pub elevation: Value<Elevation>,
    /// Optional spacing scale override; `None` uses the theme card inset.
    pub padding: Option<Space>,
}
impl CardAppearance {
    /// Combine palette, elevation and optional inset without owning card interaction state.
    pub fn new(
        variant: impl IntoValue<CardVariant>,
        elevation: impl IntoValue<Elevation>,
        padding: Option<Space>,
    ) -> Self {
        Self {
            variant: variant.into_value(),
            elevation: elevation.into_value(),
            padding,
        }
    }
}
impl kivact::ui::Variant for CardAppearance {}
impl Default for CardAppearance {
    fn default() -> Self {
        Self::new(CardVariant::Default, Elevation::None, None)
    }
}
fn variant(s: &Look<CardState>) -> CardVariant {
    s.variant::<CardAppearance>().map_or_else(
        || s.variant::<CardVariant>().copied().unwrap_or_default(),
        |v| v.variant.get(),
    )
}
fn padding(s: &Look<CardState>, t: &crate::Tokens) -> f64 {
    let logical = s
        .variant::<CardAppearance>()
        .and_then(|v| v.padding)
        .map_or(t.spacing.card_padding, Space::px);
    (logical
        - if variant(s) == CardVariant::Default {
            t.sizes.border_width
        } else {
            0.0
        })
    .max(0.0)
}
/// Provide the card's content inset to Soft layout children, without adding a control or node.
pub fn with_card_insets<V: IntoView>(
    appearance: CardAppearance,
    content: impl FnOnce() -> V + 'static,
) -> impl IntoView {
    let t = kivact::style::use_tokens::<crate::Tokens>();
    let logical = appearance.padding.map_or(t.spacing.card_padding, Space::px);
    let physical = (logical
        - if appearance.variant.get() == CardVariant::Default {
            t.sizes.border_width
        } else {
            0.0
        })
    .max(0.0);
    provide(ContainerInset::uniform(physical), move || {
        provide(
            LayoutInset::from_padding(ContainerInset::uniform(logical)),
            content,
        )
    })
}
pub(crate) fn card(r: CardRecipes) -> CardRecipes {
    r.root(|_| root)
        .content(|_| {
            |theme: &Theme, s: &Look<CardState>| {
                let t = theme.ext::<crate::Tokens>();
                style()
                    .direction(Direction::Column)
                    .padding(padding(s, &t))
                    .min_width(0)
            }
        })
        .header(|_| |_: &Theme, _: &Look<CardState>| style())
        .media(|_| |_: &Theme, _: &Look<CardState>| style())
        .footer(|_| |_: &Theme, _: &Look<CardState>| style())
        .overlay(|_| overlay)
}
fn root(theme: &Theme, s: &Look<CardState>) -> Style {
    let t = theme.ext::<crate::Tokens>();
    let c = t.colors;
    let variant = variant(s);
    let bordered = variant == CardVariant::Default;
    let bg = match variant {
        CardVariant::Default => c.bg_card,
        CardVariant::Transparent => Color::TRANSPARENT,
        CardVariant::Muted => c.bg_muted,
        CardVariant::Blue => c.hue.blue.bg,
        CardVariant::Cyan => c.hue.cyan.bg,
        CardVariant::Gray => c.hue.gray.bg,
        CardVariant::Green => c.hue.green.bg,
        CardVariant::Orange => c.hue.orange.bg,
        CardVariant::Pink => c.hue.pink.bg,
        CardVariant::Purple => c.hue.purple.bg,
        CardVariant::Red => c.hue.red.bg,
        CardVariant::Teal => c.hue.teal.bg,
        CardVariant::Yellow => c.hue.yellow.bg,
    };
    let ring = match variant {
        CardVariant::Default | CardVariant::Transparent | CardVariant::Muted => c.accent,
        CardVariant::Blue => c.hue.blue.border,
        CardVariant::Cyan => c.hue.cyan.border,
        CardVariant::Gray => c.hue.gray.border,
        CardVariant::Green => c.hue.green.border,
        CardVariant::Orange => c.hue.orange.border,
        CardVariant::Pink => c.hue.pink.border,
        CardVariant::Purple => c.hue.purple.border,
        CardVariant::Red => c.hue.red.border,
        CardVariant::Teal => c.hue.teal.border,
        CardVariant::Yellow => c.hue.yellow.border,
    };

    let selected = s.selected == Some(true);
    let mut base = style()
        .direction(Direction::Column)
        .position(Position::Relative)
        .min_width(0)
        .radius(t.radius.container)
        .clip(true)
        .background(bg);
    if bordered {
        base = base.border(t.sizes.border_width, if selected { ring } else { c.border });
    }
    if s.selected.is_some() {
        base = base.shadow(
            Shadow::new(0, 0, 0, if selected { ring } else { Color::TRANSPARENT })
                .spread(2)
                .inset(),
        );
    }
    if s.interactive {
        base = base
            .focus_visible(crate::recipe_style::ring(&t, c.accent))
            .transition(
                TransitionProp::All,
                Transition::tween(t.motion.fast as f64, t.motion.ease_standard.clone()),
            );
        if s.disabled {
            base = base.opacity(t.sizes.disabled_opacity);
        } else if s.selected.is_none() && bordered {
            base = base.hover(style().border_color(c.border_emphasized));
        }
    }
    s.variant::<CardAppearance>()
        .map_or(Elevation::None, |v| v.elevation.get())
        .apply(base, &t)
}
fn overlay(theme: &Theme, s: &Look<CardState>) -> Style {
    let t = theme.ext::<crate::Tokens>();
    let color = s
        .input
        .filter(|input| !input.disabled)
        .map_or(Color::TRANSPARENT, |input| {
            if input.pressed {
                t.colors.overlay_pressed
            } else if input.hovered {
                t.colors.overlay_hover
            } else {
                Color::TRANSPARENT
            }
        });
    style()
        .radius(t.radius.container)
        .background(color)
        .transition(
            TransitionProp::All,
            Transition::tween(t.motion.fast as f64, t.motion.ease_standard.clone()),
        )
}
kivact::state::constant_value!(CardVariant);
