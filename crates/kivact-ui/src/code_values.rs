//! Role: Read-only actual construction facts for source-code recipes.
// xtask-allow(state-fields): Read-only recipe projection of independent builder inputs, the shared disclosure owner and clipboard feedback owner.
/// Rendered source facts supplied to code-block recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeBlockState {
    pub numbered: bool,
    pub wrapped: bool,
    pub expanded: bool,
    pub copied: bool,
    pub has_header: bool,
    pub has_footer: bool,
}
