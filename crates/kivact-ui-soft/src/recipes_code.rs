//! Role: Install Astryx Neutral's recipes of the code group: the code block's frame, header, source and gutter.
use crate::color as c;
use crate::metrics as m;
use crate::radius as r;
use crate::space as s;
use crate::type_scale as t;
use crate::variants::{CodeBlockContainer, CodeBlockSize, CodeBlockWidth};
use kivact::style::{FlexDirection, Justify, Style, Theme, color, pct, style};
use kivact::ui::theme::{CodeBlockState, CodeBlockTheme, Look};

/// The outer frame of a code block: its width by the width variant, and the card's fill, border and corners.
fn root(_theme: &Theme, look: &Look<CodeBlockState>, _base: Style) -> Style {
    let width = look.variant::<CodeBlockWidth>().unwrap_or_default();
    let container = look.variant::<CodeBlockContainer>().unwrap_or_default();
    let base = style().flex_direction(FlexDirection::Column).min_width(0);
    let base = match width {
        CodeBlockWidth::Fit => base.max_width(pct(100)),
        CodeBlockWidth::Fill => base.width(pct(100)),
        CodeBlockWidth::Px(w) => base.width(w),
    };
    if container == CodeBlockContainer::Card {
        base.background(c::SYNTAX_BACKGROUND)
            .border(m::BORDER_WIDTH, color::BORDER)
            .border_radius(r::RADIUS_ELEMENT)
            .clip(true)
    } else {
        base
    }
}

/// The header line: the title and the actions spread apart, in the code font.
fn header(_theme: &Theme, _look: &Look<CodeBlockState>, _base: Style) -> Style {
    style()
        .justify_content(Justify::SpaceBetween)
        .padding_x(s::SPACE_S4)
        .padding_y(s::SPACE_S2)
        .gap(s::SPACE_S2)
        .font_family(t::FAMILY_CODE)
        .font_size(t::SIZE_SM)
        .color(c::TEXT_SECONDARY)
}

/// The type of the source text at the size variant: the code font at its small or medium size.
fn source_text(look: &Look<CodeBlockState>) -> Style {
    let size = look.variant::<CodeBlockSize>().unwrap_or_default();
    style()
        .font_family(t::FAMILY_CODE)
        .font_size(if size == CodeBlockSize::Sm {
            t::SIZE_SM
        } else {
            t::SIZE_MD
        })
}

/// The source text in the primary text color.
fn source(_theme: &Theme, look: &Look<CodeBlockState>, _base: Style) -> Style {
    source_text(look).color(c::TEXT_PRIMARY)
}

/// The line numbers: the source type in the secondary text color.
fn gutter(_theme: &Theme, look: &Look<CodeBlockState>, _base: Style) -> Style {
    source_text(look).color(c::TEXT_SECONDARY)
}

/// Install the kit's recipes of the code group.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        CodeBlockTheme::default()
            .root(root)
            .header(header)
            .code(source)
            .gutter(gutter),
    )
}
