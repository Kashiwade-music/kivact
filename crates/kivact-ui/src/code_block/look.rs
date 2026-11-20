//! Role: The code block recipe state, tokens, recipe table and default looks.
use crate::chat_thread::look::viewport;
use crate::look_common::{column, row};

use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::style::{Align, Direction, LineHeight, Style, Theme, pct, style};
pub(crate) fn code_root(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    column(t, s)
        .border(1, kivact_core::style::color::BORDER)
        .radius(t.get(&kivact_core::style::radius::MD_VAR))
        .background(kivact_core::style::color::SURFACE)
        .color(kivact_core::style::color::TEXT)
}

pub(crate) fn code_header(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    row(t, s).padding(t.get(&kivact_core::style::space::SM_VAR))
}

pub(crate) fn code_body(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    row(t, s)
        .direction(Direction::RowReverse)
        .align_items(Align::Start)
        .min_width(pct(100))
}

pub(crate) fn code_viewport(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    viewport(t, s).padding(t.get(&kivact_core::style::space::MD_VAR))
}

pub(crate) fn code_text(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    let tokens = t.ext::<crate::UiTokens>();
    let metrics = &tokens.code_block;
    style()
        .font_family("monospace")
        .font_size(metrics.font_size)
        .line_height(LineHeight::px(metrics.line_height))
        .text_wrap(s.wrapped)
        .grow(1)
        .min_width(0)
}

pub(crate) fn code_gutter(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    code_text(t, s)
        .grow(0)
        .shrink(0)
        .min_width(24)
        .text_wrap(false)
        .color(kivact_core::style::color::TEXT_MUTED)
        .text_align(kivact_core::style::TextAlign::End)
}

/// Rendered source facts supplied to code-block recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeBlockState {
    pub numbered: bool,
    pub wrapped: bool,
    pub collapse: CodeCollapse,
    pub copied: bool,
    /// The header has a title, a language label, a copy button or end
    /// content, besides any collapse toggle.
    pub has_header_content: bool,
    pub has_footer: bool,
}

impl CodeBlockState {
    /// Whether every line shows.
    pub fn is_expanded(&self) -> bool {
        self.collapse != CodeCollapse::Collapsed
    }

    /// Whether the header row shows: it has content or a collapse toggle.
    pub fn has_header(&self) -> bool {
        self.has_header_content || self.collapse != CodeCollapse::Fixed
    }
}

recipe_table! {
    /// Styles for the persistent parts of CodeBlock.
    CodeBlockRecipes {
        root(root_recipe): CodeBlockState = code_root,
        header(header_recipe): CodeBlockState = code_header,
        title(title_recipe): CodeBlockState = crate::look_common::slot,
        collapse(collapse_recipe): CodeBlockState = crate::look_common::slot,
        language(language_recipe): CodeBlockState = crate::look_common::muted,
        copy(copy_recipe): CodeBlockState = crate::look_common::slot,
        end_content(end_content_recipe): CodeBlockState = crate::look_common::slot,
        body(body_recipe): CodeBlockState = code_body,
        viewport(viewport_recipe): CodeBlockState = code_viewport,
        code(code_recipe): CodeBlockState = code_text,
        gutter(gutter_recipe): CodeBlockState = code_gutter,
        footer(footer_recipe): CodeBlockState = crate::look_common::row,
    }
}

/// Whether a code block can collapse and, if so, whether it shows every line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeCollapse {
    /// The block cannot collapse; every line shows.
    Fixed,
    /// The block can collapse and shows every line.
    Expanded,
    /// The block can collapse and shows its first lines only.
    Collapsed,
}
