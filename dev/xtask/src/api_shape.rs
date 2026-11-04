//! Role: Keep facade and kit signatures consistent with the public API rules.
//!
//! Public APIs use f64, ergonomic callbacks and owned values. Slot factories
//! must be repeatable, and implementation ownership must not leak into APIs.
use crate::check_source::{Source, type_names};
use crate::public_surface::{Surface, definition};
use quote::ToTokens;
use syn::spanned::Spanned;
const SLOTS: &[&str] = &[
    "content", "header", "footer", "media", "image", "icon", "trailing", "actions", "item",
    "option", "fallback", "start", "center", "end",
];

pub(crate) fn check(source: &Source, surface: &Surface, errors: &mut Vec<String>) {
    for item in &source.items {
        match item {
            syn::Item::Struct(item)
                if surface
                    .reachable
                    .contains(&definition(source, &item.ident.to_string())) =>
            {
                for field in &item.fields {
                    if matches!(field.vis, syn::Visibility::Public(_)) {
                        check_type(source, &field.ty, true, errors);
                    }
                }
            }
            syn::Item::Enum(item)
                if surface
                    .reachable
                    .contains(&definition(source, &item.ident.to_string())) =>
            {
                for variant in &item.variants {
                    for field in &variant.fields {
                        check_type(source, &field.ty, true, errors);
                    }
                }
            }
            syn::Item::Fn(item)
                if surface
                    .reachable
                    .contains(&definition(source, &item.sig.ident.to_string())) =>
            {
                check_signature(source, &item.sig, false, errors)
            }
            syn::Item::Trait(item)
                if surface
                    .reachable
                    .contains(&definition(source, &item.ident.to_string())) =>
            {
                for member in &item.items {
                    if let syn::TraitItem::Fn(method) = member {
                        check_signature(source, &method.sig, true, errors);
                    }
                }
            }
            syn::Item::Type(item)
                if surface
                    .reachable
                    .contains(&definition(source, &item.ident.to_string())) =>
            {
                check_type(source, &item.ty, false, errors)
            }
            syn::Item::Const(item)
                if surface
                    .reachable
                    .contains(&definition(source, &item.ident.to_string())) =>
            {
                check_type(source, &item.ty, false, errors)
            }
            syn::Item::Static(item)
                if surface
                    .reachable
                    .contains(&definition(source, &item.ident.to_string())) =>
            {
                check_type(source, &item.ty, false, errors)
            }
            syn::Item::Impl(item) => {
                if !surface
                    .resolve_type(source, &item.self_ty)
                    .is_some_and(|name| surface.reachable.contains(&name))
                {
                    continue;
                }
                for member in &item.items {
                    if let syn::ImplItem::Fn(method) = member {
                        if matches!(method.vis, syn::Visibility::Public(_)) {
                            check_signature(source, &method.sig, true, errors);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
fn check_type(source: &Source, ty: &syn::Type, f32_forbidden: bool, errors: &mut Vec<String>) {
    let names = type_names(ty);
    if f32_forbidden && names.contains("f32") {
        source.error(
            "api-shape",
            ty.span(),
            "public value uses f32 instead of f64",
            errors,
        );
    }
    let text = ty.to_token_stream().to_string();
    if names.contains("Rc")
        || names.contains("Arc")
        || (names.contains("Box") && text.contains("dyn "))
        || (names.contains("Location") && text.contains("'static"))
    {
        source.error(
            "api-shape",
            ty.span(),
            "public signature exposes implementation ownership or caller Location",
            errors,
        );
    }
}
fn check_signature(source: &Source, sig: &syn::Signature, method: bool, errors: &mut Vec<String>) {
    let args: Vec<_> = sig
        .inputs
        .iter()
        .filter_map(|a| {
            if let syn::FnArg::Typed(arg) = a {
                Some(arg.ty.as_ref())
            } else {
                None
            }
        })
        .collect();
    for arg in &args {
        check_type(source, arg, true, errors);
    }
    if let syn::ReturnType::Type(_, ty) = &sig.output {
        check_type(source, ty, false, errors);
    }
    let name = sig.ident.to_string();
    if method
        && name.starts_with("on_")
        && (args.len() != 1
            || !args.first().is_some_and(|ty| {
                matches!(ty, syn::Type::ImplTrait(_))
                    && type_names(ty)
                        .iter()
                        .any(|n| matches!(n.as_str(), "Fn" | "IntoAction"))
            }))
    {
        source.error(
            "api-shape",
            sig.span(),
            "on_ method requires exactly one impl Fn or impl IntoAction callback",
            errors,
        );
    }
    if method
        && SLOTS.contains(&name.as_str())
        && args.iter().any(|ty| type_names(ty).contains("FnOnce"))
    {
        source.error(
            "api-shape",
            sig.span(),
            "slot factory must be repeatable, not FnOnce",
            errors,
        );
    }
}
#[cfg(test)]
mod tests {
    //! Role: Check public fields, callbacks and ownership only on reachable items.
    #[test]
    fn checks_reachable_public_shapes() {
        let mut source = crate::check_source::fixture(
            "pub struct Widget { pub x: f32, hidden: f32 } impl Widget { pub fn on_press(self, f: impl Fn()) {} pub fn on_bad(self, f: bool) {} pub fn content(self, f: impl FnOnce()) {} pub fn shared(self) -> Rc<Data> { loop {} } } struct Hidden { pub x: f32 }",
        );
        source.module.clear();
        let surface = crate::public_surface::build(std::slice::from_ref(&source));
        let mut errors = Vec::new();
        super::check(&source, &surface, &mut errors);
        assert_eq!(errors.len(), 4);
    }
    #[test]
    fn resolves_qualified_and_imported_impl_types() {
        let mut root = crate::check_source::fixture("pub struct Widget; mod implementation;");
        root.module.clear();
        let mut methods = crate::check_source::fixture(
            "use crate::Widget; impl Widget { pub fn bad(x: f32) {} } impl crate::Widget { pub fn also_bad(x: f32) {} }",
        );
        methods.module = "implementation".into();
        let mut errors = Vec::new();
        let sources = [root, methods];
        let surface = crate::public_surface::build(&sources);
        super::check(&sources[1], &surface, &mut errors);
        assert_eq!(errors.len(), 2);
    }
}
