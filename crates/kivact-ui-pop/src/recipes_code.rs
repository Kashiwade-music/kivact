//! Role: The Pop looks of the source-code block: its frame, header, text and gutter.

use crate::color;
use kivact::style::{Style, Theme};
use kivact::ui::theme::{CodeBlockState, CodeBlockTheme, Look};

/// Install the kit's recipes of the code block.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        CodeBlockTheme::default()
            .root(code_root)
            .header(code_header)
            .code(code_text)
            .gutter(code_gutter),
    )
}

fn code_root(_theme: &Theme, _: &Look<CodeBlockState>, base: Style) -> Style {
    base.border_radius(10)
        .border_color(color::LINE)
        .background(color::SURFACE)
}

fn code_header(_theme: &Theme, _: &Look<CodeBlockState>, base: Style) -> Style {
    base.background(color::SURFACE_MUTED)
        .padding_x(12)
        .padding_y(8)
}

fn code_text(_theme: &Theme, _: &Look<CodeBlockState>, base: Style) -> Style {
    base.font_size(13).color(color::TEXT)
}

fn code_gutter(_theme: &Theme, _: &Look<CodeBlockState>, base: Style) -> Style {
    base.color(color::TEXT_FAINT)
        .background(color::SURFACE_MUTED)
}
