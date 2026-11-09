//! Role: Read-only actual construction facts for source-code recipes.
/// Rendered source facts supplied to code-block recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeBlockState {
    // xtask-allow(state-fields): projection: readonly snapshot of the code block's state for recipes, written only by CodeBlock::into_view
    pub numbered: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the code block's state for recipes, written only by CodeBlock::into_view
    pub wrapped: bool,
    // xtask-allow(state-fields): independent: the disclosure and the copy feedback are separate parts
    pub expanded: bool,
    // xtask-allow(state-fields): independent: the disclosure and the copy feedback are separate parts
    pub copied: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the code block's state for recipes, written only by CodeBlock::into_view
    pub has_header: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the code block's state for recipes, written only by CodeBlock::into_view
    pub has_footer: bool,
}
