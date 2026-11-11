//! Role: Read-only actual construction facts for source-code recipes.
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

/// Rendered source facts supplied to code-block recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeBlockState {
    // xtask-allow(state-fields): independent: CodeBlock::line_numbers shows the gutter in every collapse state, wrapped or not
    pub numbered: bool,
    // xtask-allow(state-fields): independent: CodeBlock::wrap wraps long lines whether or not the gutter shows
    pub wrapped: bool,
    pub collapse: CodeCollapse,
    // xtask-allow(state-fields): independent: use_copy_feedback holds it for copy_feedback_ms after Copy; the feedback shows in any collapse state and Expand works while it shows
    pub copied: bool,
    // xtask-allow(state-fields): independent: CodeBlock::title, language, copy_button or end_content fill the header; a collapsible block shows its header even when this is false
    /// The header has a title, a language label, a copy button or end
    /// content, besides any collapse toggle.
    pub has_header_content: bool,
    // xtask-allow(state-fields): independent: CodeBlock::footer fills the footer slot, with or without header content
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
