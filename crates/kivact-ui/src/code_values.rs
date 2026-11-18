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
