//! Role: Require every structural shortcut to appear in the reviewed option list.
//!
//! Public zero-argument methods returning Self select structure, and keeping
//! their list explicit prevents arbitrary convenience methods from growing API.
use crate::check_source::Source;
use syn::spanned::Spanned;
pub(crate) fn check(source: &Source, policy: &toml::Table, errors: &mut Vec<String>) {
    let allowed = crate::foundation::strings(policy, "list");
    for item in &source.items {
        let syn::Item::Impl(item) = item else {
            continue;
        };
        if item.trait_.is_some() {
            continue;
        }
        for member in &item.items {
            let syn::ImplItem::Fn(method) = member else {
                continue;
            };
            if !matches!(method.vis, syn::Visibility::Public(_))
                || method
                    .sig
                    .inputs
                    .iter()
                    .any(|arg| matches!(arg, syn::FnArg::Typed(_)))
            {
                continue;
            }
            if !matches!(&method.sig.output, syn::ReturnType::Type(_, ty) if matches!(ty.as_ref(), syn::Type::Path(p) if p.path.is_ident("Self")))
            {
                continue;
            }
            let type_name = match item.self_ty.as_ref() {
                syn::Type::Path(p) => p
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default(),
                _ => continue,
            };
            let short = format!("{type_name}::{}", method.sig.ident);
            let qualified = format!("{}::{}::{short}", source.package, source.module);
            if !allowed.contains(&short) && !allowed.contains(&qualified) {
                source.error(
                    "structural-options",
                    method.span(),
                    &format!("unlisted structural option {short}"),
                    errors,
                );
            }
        }
    }
}
#[cfg(test)]
mod tests {
    //! Role: Distinguish structural shortcuts from parameterized builders.
    #[test]
    fn requires_reviewed_zero_argument_self_methods() {
        let source = crate::check_source::fixture(
            "impl<T> Button<T> { pub fn compact(self) -> Self { self } pub fn width(self, width: f64) -> Self { self } fn hidden(self) -> Self { self } }",
        );
        let mut errors = Vec::new();
        super::check(&source, &toml::Table::new(), &mut errors);
        assert_eq!(errors.len(), 1);
        errors.clear();
        super::check(
            &source,
            &"list = ['Button::compact']".parse().unwrap(),
            &mut errors,
        );
        assert!(errors.is_empty());
    }
}
