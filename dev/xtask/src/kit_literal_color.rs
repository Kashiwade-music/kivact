//! Role: Keep kit literal color construction in the designated palette files.
//!
//! Recipes consume semantic tokens rather than introducing a second palette.
use crate::check_source::Source;
use syn::spanned::Spanned;
use syn::visit::Visit;
pub(crate) fn check(source: &Source, policy: &toml::Table, errors: &mut Vec<String>) {
    if !crate::foundation::is_kit(&source.package)
        || source
            .path
            .file_name()
            .and_then(|v| v.to_str())
            .is_some_and(|v| {
                crate::foundation::strings(policy, "allowed_files")
                    .iter()
                    .any(|a| a == v)
            })
    {
        return;
    }
    let mut visitor = Colors { source, errors };
    for item in &source.items {
        if !matches!(item, syn::Item::Mod(_)) {
            visitor.visit_item(item);
        }
    }
}
struct Colors<'a> {
    source: &'a Source,
    errors: &'a mut Vec<String>,
}
impl<'ast> Visit<'ast> for Colors<'_> {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if matches!(call.func.as_ref(), syn::Expr::Path(p) if p.path.segments.last().is_some_and(|s| matches!(s.ident.to_string().as_str(), "rgb" | "rgba" | "hsl")))
        {
            self.source.error(
                "kit-literal-color",
                call.span(),
                "construct color in tokens.rs, theme.rs or color.rs",
                self.errors,
            );
        }
        syn::visit::visit_expr_call(self, call);
    }
}
#[cfg(test)]
mod tests {
    //! Role: Verify qualified calls and designated palette exemptions.
    #[test]
    fn colors_are_calls_not_text() {
        let mut source = crate::check_source::fixture(
            "fn recipe() { Color::rgb(1,2,3); let text = \"rgba(\"; }",
        );
        let policy: toml::Table = "allowed_files = ['tokens.rs', 'theme.rs', 'color.rs']"
            .parse()
            .unwrap();
        let mut errors = Vec::new();
        super::check(&source, &policy, &mut errors);
        assert_eq!(errors.len(), 1);
        source.path = "tokens.rs".into();
        errors.clear();
        super::check(&source, &policy, &mut errors);
        assert!(errors.is_empty());
    }
}
