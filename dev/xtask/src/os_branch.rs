//! Role: Reject operating-system branches outside declared platform boundaries.
//!
//! Core behavior consumes platform capabilities, so nested cfg predicates and
//! cfg! expressions must not silently introduce OS-specific behavior.
use crate::check_source::Source;
use syn::spanned::Spanned;
use syn::visit::Visit;
pub(crate) fn check(source: &Source, policy: &toml::Table, errors: &mut Vec<String>) {
    let path = source.path.to_string_lossy().replace('\\', "/");
    if path.contains("/tests/")
        || source
            .module
            .split("::")
            .any(|m| matches!(m, "test" | "testing" | "tests"))
        || crate::check_policy::strings(policy, "allowed_paths")
            .iter()
            .any(|p| path.contains(p))
    {
        return;
    }
    let mut visitor = Branches { source, errors };
    for attr in &source.attrs {
        visitor.visit_attribute(attr);
    }
    for item in &source.items {
        if let syn::Item::Mod(module) = item {
            for attr in &module.attrs {
                visitor.visit_attribute(attr);
            }
        } else {
            visitor.visit_item(item);
        }
    }
}
fn has_os(tokens: proc_macro2::TokenStream) -> bool {
    tokens.into_iter().any(|t| match t {
        proc_macro2::TokenTree::Ident(i) => matches!(
            i.to_string().as_str(),
            "target_os" | "target_family" | "windows" | "unix"
        ),
        proc_macro2::TokenTree::Group(g) => has_os(g.stream()),
        _ => false,
    })
}
struct Branches<'a> {
    source: &'a Source,
    errors: &'a mut Vec<String>,
}
impl<'ast> Visit<'ast> for Branches<'_> {
    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        if (attr.path().is_ident("cfg") || attr.path().is_ident("cfg_attr"))
            && matches!(&attr.meta, syn::Meta::List(list) if has_os(list.tokens.clone()))
        {
            self.source.error(
                "os-branch",
                attr.span(),
                "OS predicate outside platform boundary",
                self.errors,
            );
        }
    }
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if mac.path.is_ident("cfg") && has_os(mac.tokens.clone()) {
            self.source.error(
                "os-branch",
                mac.span(),
                "OS predicate outside platform boundary",
                self.errors,
            );
        }
    }
}
#[cfg(test)]
mod tests {
    //! Role: Cover nested predicates, cfg_attr and cfg expressions.
    #[test]
    fn nested_os_predicates_are_rejected() {
        let source = crate::check_source::fixture(
            "#![cfg(windows)] #[cfg(all(feature = \"x\", not(unix)))] fn a() {} #[cfg_attr(target_os = \"windows\", inline)] fn b() { let x = cfg!(any(windows, unix)); } #[cfg(feature = \"unix\")] fn c() {}",
        );
        let mut errors = Vec::new();
        super::check(&source, &toml::Table::new(), &mut errors);
        assert_eq!(errors.len(), 4);
    }
}
