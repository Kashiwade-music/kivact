//! Role: Bridge style values and recording to core host machinery.

pub use crate::draw::recorder::{RecordContext, record};
use crate::resolve::Interactive;
pub use crate::resolve::{compile_style, describe_state_styles, describe_style};
use crate::sheet::{Decl, PropGroup};
pub use crate::sheet::{StyleProperty, with_scroll_axes};

/// How many declarations of `style` set a property an earlier one of it sets
/// (core counts this per style layer in debug builds).
pub fn duplicates(style: &crate::sheet::Style) -> u32 {
    style.duplicates()
}

/// The property `name` (a setter's or a longhand's name) in DevTools form.
pub fn style_property(name: &str) -> Option<&'static StyleProperty> {
    crate::sheet::STYLE_PROPERTIES
        .iter()
        .find(|p| p.name == name)
}
use crate::theme::value::Theme;
pub use crate::theme::value::{install_theme, theme_of};
use kivact_types as p;

/// A span's or an editing range's style compiled against `theme`: its text
/// properties, its background, and how many of its values the host cannot
/// take (a declaration that is neither text nor `background`, and the values
/// the host drops), reported as `Unsupported`.
pub fn compile_span(
    style: &crate::sheet::Style,
    theme: &Theme,
) -> (p::TextStyle, Option<p::Fill>, u32) {
    let rejected = style
        .decls
        .iter()
        .filter(|d| d.group() != PropGroup::Text && !matches!(d, Decl::Background(_)))
        .count() as u32;
    let compiled = compile_style(style.clone(), theme, Interactive::default());
    let dropped = compiled.misplaced_lengths + compiled.unsupported_text;
    (
        compiled.text.unwrap_or_default(),
        compiled.paint.background,
        rejected + dropped,
    )
}

/// The color as the engine paints it, for core's text editing and custom paint bridges.
pub fn resolve_color(color: crate::Color, theme: &Theme) -> p::Rgba {
    color.resolve(theme)
}

/// The theme in effect for the current owner, if there is one: like
/// [`crate::use_theme`] but never a misuse and untracked (animation and
/// luminance read it outside setup).
pub fn try_use_theme() -> Option<Theme> {
    let owner = kivact_runtime::Owner::try_ambient()?;
    theme_of(owner).map(|theme| kivact_runtime::untrack(|| theme.get()))
}
