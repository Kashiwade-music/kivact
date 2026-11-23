//! Role: Theme-aware length resolution: `em` and theme variables become px.

use kivact_types as p;
use kivact_types::Length;

use crate::theme::value::Theme;

/// `l` with its theme variables and `em` resolved (`em_px` is the font
/// size `em` is of); an `Invalid` sum counts in `misplaced` and is `Auto`.
pub(crate) fn resolved(l: Length, theme: &Theme, em_px: f64, misplaced: &mut u32) -> Length {
    match l {
        Length::Var(v) => Length::Px(theme.get(v)),
        Length::Em(e) => Length::Px(e * em_px),
        Length::Calc {
            percent,
            px,
            em,
            var,
        } => {
            let px = px + em * em_px + var.map_or(0.0, |(v, k)| k * theme.get(v));
            if percent == 0.0 {
                Length::Px(px)
            } else {
                Length::Calc {
                    percent,
                    px,
                    em: 0.0,
                    var: None,
                }
            }
        }
        Length::Invalid => {
            *misplaced += 1;
            Length::Auto
        }
        other => other,
    }
}

/// What a compile resolves the lengths of a style with: the theme, the
/// font size `em` is of (the style's font size, else the theme's base size)
/// and the count of sums the host cannot take.
pub(crate) struct Resolve<'a> {
    pub(crate) theme: &'a Theme,
    pub(crate) em_px: f64,
    pub(crate) misplaced: &'a mut u32,
}

impl<'a> Resolve<'a> {
    /// The resolve of a drawing that has no style: `em` is of the theme's
    /// base size.
    pub(crate) fn of_theme(theme: &'a Theme, misplaced: &'a mut u32) -> Self {
        Resolve {
            theme,
            em_px: theme.get(&crate::font::SIZE_VAR),
            misplaced,
        }
    }

    /// `l` with its theme variables and `em` resolved.
    pub(crate) fn len(&mut self, l: Length) -> Length {
        resolved(l, self.theme, self.em_px, self.misplaced)
    }

    /// `l` in its paint form: resolved, and counted in the misplaced lengths
    /// when a paint position cannot take it (`auto`, a content size).
    pub(crate) fn paint_len(&mut self, l: Length) -> p::Len {
        resolved(l, self.theme, self.em_px, &mut 0).to_len(self.misplaced)
    }
}
