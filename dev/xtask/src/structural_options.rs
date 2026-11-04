//! Role: Require every structural shortcut to appear in the reviewed option list.
//!
//! Facade and kit builder flags take no arguments and return Self. Their list
//! stays explicit so tree and behavior choices cannot grow unnoticed. Associated
//! constructors, private implementation types, passive data records and option
//! bundles do not configure mounted UI structure (design section 10.6).
use crate::check_source::Source;
use crate::public_surface::Surface;
use syn::spanned::Spanned;
pub(crate) fn check(
    source: &Source,
    policy: &toml::Table,
    surface: &Surface,
    errors: &mut Vec<String>,
) {
    if !source.library {
        return;
    }
    let records = crate::check_policy::strings(policy, "record_types");
    let allowed = crate::check_policy::strings(policy, "list");
    for item in &source.items {
        let syn::Item::Impl(item) = item else {
            continue;
        };
        if item.trait_.is_some()
            || !surface
                .resolve_type(source, &item.self_ty)
                .is_some_and(|name| surface.reachable.contains(&name))
        {
            continue;
        }
        for member in &item.items {
            let syn::ImplItem::Fn(method) = member else {
                continue;
            };
            if method.sig.receiver().is_none()
                || !matches!(method.vis, syn::Visibility::Public(_))
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
            if type_name.ends_with("Options") || records.contains(&type_name) {
                continue;
            }
            let short = format!("{type_name}::{}", method.sig.ident);
            let qualified = format!("{}::{}::{short}", source.package, source.module);
            if !allowed.contains(&short)
                && !allowed.contains(&qualified)
                && !allowed.contains(&method.sig.ident.to_string())
            {
                source.error(
                    "structural-options",
                    method.sig.span(),
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
    fn constructors_private_helpers_records_and_option_bundles_are_not_flags() {
        let mut source = crate::check_source::fixture(
            "pub struct Button; impl Button { pub fn new() -> Self { Self } } struct Hidden; impl Hidden { pub fn hidden(self) -> Self { self } } pub struct Row; impl Row { pub fn disabled(self) -> Self { self } } pub struct RenderOptions; impl RenderOptions { pub fn dark(self) -> Self { self } }",
        );
        source.module.clear();
        let surface = crate::public_surface::build(std::slice::from_ref(&source));
        let mut errors = Vec::new();
        super::check(
            &source,
            &"record_types = ['Row']".parse().unwrap(),
            &surface,
            &mut errors,
        );
        assert!(errors.is_empty());
    }
    #[test]
    fn requires_reviewed_zero_argument_self_methods() {
        let mut source = crate::check_source::fixture(
            "pub struct Button<T>(T); impl<T> Button<T> { pub fn compact(self) -> Self { self } pub fn width(self, width: f64) -> Self { self } fn hidden(self) -> Self { self } }",
        );
        source.module.clear();
        let surface = crate::public_surface::build(std::slice::from_ref(&source));
        let mut errors = Vec::new();
        super::check(&source, &toml::Table::new(), &surface, &mut errors);
        assert_eq!(errors.len(), 1);
        errors.clear();
        super::check(
            &source,
            &"list = ['Button::compact']".parse().unwrap(),
            &surface,
            &mut errors,
        );
        assert!(errors.is_empty());
    }
}
