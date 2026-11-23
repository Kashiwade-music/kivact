//! Role: Bridge style values and recording to core host machinery.

pub use crate::draw::recorder::{RecordContext, record};
use crate::resolve::Interactive;
pub use crate::resolve::{compile_style, describe_state_styles, describe_style};
use crate::sheet::PropGroup;
use crate::sheet::props::Decl;
pub use crate::sheet::{StyleProperty, with_scroll_axes};

/// How many declarations of `style` set a property an earlier one of it sets
/// (core counts this per style layer in debug builds).
pub fn duplicates(style: &crate::sheet::Style) -> u32 {
    style.duplicates()
}

/// Drop the spare capacity of `style`'s declarations (core keeps a style
/// layer whose value never changes for the node's lifetime).
pub fn shrink(style: &mut crate::sheet::Style) {
    style.decls.shrink_to_fit();
}

/// The property `name` (a setter's or a longhand's name) in DevTools form.
pub fn style_property(name: &str) -> Option<&'static StyleProperty> {
    crate::sheet::props::STYLE_PROPERTIES
        .iter()
        .find(|p| p.name == name)
}
use crate::theme::value::Theme;
pub use crate::theme::value::{install_theme, theme_of};
use kivact_types as p;

/// A text declaration a span cannot carry: it belongs to the paragraph
/// (lines, alignment, overflow, clamping, case, editing colors) or to the
/// whole text's paint (shadow, paint order, optical sizing).
fn span_drops(d: &Decl) -> bool {
    matches!(
        d,
        Decl::LineHeight(_)
            | Decl::TextWrap(_)
            | Decl::TextAlign(_)
            | Decl::TextOverflow(_)
            | Decl::LineClamp(_)
            | Decl::TextCapsize(_)
            | Decl::TextTransform(_)
            | Decl::PaintOrder(_)
            | Decl::FontOpticalSizing(_)
            | Decl::CaretColor(_)
            | Decl::SelectionColor(_)
            | Decl::PlaceholderColor(_)
            | Decl::TextShadow(_)
    )
}

/// A span's or an editing range's style compiled against `theme`: its text
/// properties, its background, and how many of its values the host cannot
/// take (a declaration that is neither text nor `background`, and the values
/// the host drops), reported as `Unsupported`. Declarations a span cannot
/// carry (`span_drops`) are counted with the rest.
pub fn compile_span(
    style: &crate::sheet::Style,
    theme: &Theme,
) -> (p::TextStyle, Option<p::Fill>, u32) {
    let rejected = style
        .decls
        .iter()
        .filter(|d| {
            (d.group() != PropGroup::Text && !matches!(d, Decl::Background(_))) || span_drops(d)
        })
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

pub use crate::theme::value::try_use_theme;

#[cfg(test)]
mod tests {
    //! Role: Tests of the style bridges.
    use super::*;
    use crate::rgb;
    use crate::sheet::style;

    #[test]
    fn a_span_counts_the_text_declarations_it_cannot_carry() {
        let theme = Theme::light();
        let paragraph = style()
            .line_height(2.0)
            .text_align(p::TextAlign::Center)
            .color(rgb(0x2563eb))
            .font_weight(700);
        assert_eq!(compile_span(&paragraph, &theme).2, 2);
        let inline = style().color(rgb(0x2563eb)).tabular_nums();
        assert_eq!(compile_span(&inline, &theme).2, 0);
    }
}
