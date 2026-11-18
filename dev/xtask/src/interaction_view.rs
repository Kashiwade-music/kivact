//! Role: Keep views out of kivact-interaction except an Api `surface` (design 6.3).

use crate::check_source::{Source, type_names};
use syn::spanned::Spanned;

/// The type names that make a return value a view.
const VIEW_TYPES: [&str; 4] = ["View", "ViewFn", "IntoView", "Element"];

pub(crate) fn check(source: &Source, errors: &mut Vec<String>) {
    if source.package != "kivact-interaction" {
        return;
    }
    for item in &source.items {
        match item {
            syn::Item::Fn(item) => check_signature(source, &item.sig, errors),
            syn::Item::Impl(item) => {
                for member in &item.items {
                    if let syn::ImplItem::Fn(method) = member {
                        check_signature(source, &method.sig, errors);
                    }
                }
            }
            _ => {}
        }
    }
}

fn check_signature(source: &Source, signature: &syn::Signature, errors: &mut Vec<String>) {
    if signature.ident == "surface" {
        return;
    }
    let syn::ReturnType::Type(_, ty) = &signature.output else {
        return;
    };
    let names = type_names(ty);
    if VIEW_TYPES.iter().any(|name| names.contains(*name)) {
        source.error_at(
            "interaction-view",
            signature.span(),
            &format!(
                "`{}` returns a view; kivact-interaction returns Apis and Parts, and only `surface` returns a view",
                signature.ident
            ),
            errors,
        );
    }
}

#[cfg(test)]
mod tests {
    //! Role: Check that only `surface` returns a view in kivact-interaction.
    #[test]
    fn only_surface_returns_a_view() {
        let mut source = crate::check_source::fixture(
            "pub fn outlet() -> View { loop {} }\nimpl Api { pub fn surface(self) -> View { loop {} } pub fn while_open<V: IntoView>(self) -> impl IntoView { loop {} } pub fn trigger(self) -> Part<State> { loop {} } }",
        );
        source.package = "kivact-interaction".into();
        let mut errors = Vec::new();
        super::check(&source, &mut errors);
        assert_eq!(errors.len(), 2, "{errors:?}");
    }
}
