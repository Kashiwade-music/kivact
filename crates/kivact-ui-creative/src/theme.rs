//! Role: Construct the Creative theme and install standard UI recipes.
use crate::tokens::{Accent, Density, Tokens};
use kivact::style::theme::{Palette, Radii, Spacing, Typography};
use kivact::style::{LineHeight, Theme, ThemeMode, rgb, rgba};

/// A Kivact theme carrying the complete creative-tool palette in `mode`, in
/// the [`Iris`](Accent::Iris) accent and the
/// [`Comfortable`](Density::Comfortable) density; [`ThemeOptions`] chooses
/// others (`theme(ThemeMode::Dark).accent(Accent::Lime)`). The source tool is
/// dark; the light theme keeps its accents and metrics on light surfaces. Its
/// icon set has every icon name the kit draws over the standard marks.
///
/// The result is a plain [`Theme`]: give it to `App::theme` or
/// `kivact::view::provide_theme` as it is.
pub fn theme(mode: ThemeMode) -> Theme {
    let p = Tokens::new(mode, Accent::default(), Density::default());
    let dark = mode == ThemeMode::Dark;
    let base = if dark { Theme::dark() } else { Theme::light() };
    let colors = Palette {
        surface: p.island,
        surface_alt: rgb(if dark { 0x1b1c22 } else { 0xf3f4f6 }),
        raised: if dark { rgb(0x1b1c22) } else { p.island },
        hover: rgb(if dark { 0x23252c } else { 0xe9eaee }),
        pressed: p.strong,
        text: p.text,
        text_muted: p.muted,
        border: p.hair,
        danger: p.x,
        success: p.success,
        warning: p.warning,
        overlay: rgba(0x000000, if dark { 0.45 } else { 0.25 }),
        ..base.colors().clone()
    };
    let text = Typography {
        family: Some(p.body_font.clone()),
        size: 12.5,
        size_small: 10.5,
        size_large: 24.0,
        line_height: LineHeight::Scale(1.4),
        ..base.text().clone()
    };
    let radius = Radii {
        sm: 5.0,
        md: 7.0,
        lg: 12.0,
    };
    let t = base
        .with_colors(colors)
        .with_text(text)
        .with_radius(radius)
        // The ring every focus stop of the kit and the standard parts shows
        // while the focus is visible, in the accent (`colors.focus`).
        .with_focus_ring(2.0)
        .with_disabled_opacity(0.35)
        .with_icons(crate::icons::set());
    install(t, p).with(crate::recipes::recipes())
}

/// The Creative kit's choices on a [`theme`]: each returns the theme with
/// the kit's [`Tokens`] and the core colors and metrics that follow them
/// changed, keeping everything else.
///
/// On a theme that is not the kit's, the tokens start from the kit's
/// defaults in the theme's mode.
pub trait ThemeOptions {
    /// Use `accent` for the accent colors, the selection and the focus ring.
    #[must_use]
    fn accent(self, accent: Accent) -> Theme;
    /// Use `density` for the control row height and the gaps.
    #[must_use]
    fn density(self, density: Density) -> Theme;
}

impl ThemeOptions for Theme {
    fn accent(self, accent: Accent) -> Theme {
        let p = self.get::<Tokens>().as_ref().clone().with_accent(accent);
        install(self, p)
    }

    fn density(self, density: Density) -> Theme {
        let p = self.get::<Tokens>().as_ref().clone().with_density(density);
        install(self, p)
    }
}

/// `t` with the tokens `p` and the core values that follow the accent and
/// the density.
fn install(t: Theme, p: Tokens) -> Theme {
    let colors = Palette {
        accent: p.accent,
        accent_text: p.accent_ink,
        focus: p.accent,
        selection: p.selection,
        ..t.colors().clone()
    };
    let space = Spacing {
        md: p.gap,
        ..*t.space()
    };
    t.with_colors(colors)
        .with_space(space)
        .with_control_height(p.row)
        .with(p)
}
