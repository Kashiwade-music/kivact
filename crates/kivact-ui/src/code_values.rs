//! Role: Read-only actual construction facts for source-code recipes.
/// Rendered source facts supplied to code-block recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeBlockState {
    // xtask-allow(state-fields): passive record - numbered
    pub numbered: bool,
    // xtask-allow(state-fields): passive record - wrapped
    pub wrapped: bool,
    // xtask-allow(state-fields): passive record - expanded
    pub expanded: bool,
    // xtask-allow(state-fields): passive record - copied
    pub copied: bool,
    // xtask-allow(state-fields): passive record - has header
    pub has_header: bool,
    // xtask-allow(state-fields): passive record - has footer
    pub has_footer: bool,
}
