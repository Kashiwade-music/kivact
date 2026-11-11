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
/// Lifecycle setters take `FnOnce`, not an event callback (API-3); the
/// embedding `Parent::on_resize_request` is a host integration hook whose
/// return value is the host's acceptance, not a `HandlerResult`.
const LIFECYCLE: [&str; 4] = ["on_mount", "on_cleanup", "on_attach", "on_resize_request"];

pub(crate) fn check(source: &Source, surface: &Surface, errors: &mut Vec<String>) {
    if !source.library() {
        return;
    }
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
                    if let syn::TraitItem::Fn(method) = member
                        && !source.internal_contract(&method.attrs, method.sig.span())
                    {
                        check_signature(source, &method.sig, true, errors);
                    }
                }
            }
            syn::Item::Type(item)
                if surface
                    .reachable
                    .contains(&definition(source, &item.ident.to_string())) =>
            {
                check_type(source, &item.ty, true, errors)
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
                check_impl(source, surface, item, errors);
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
    if source.allows("api-shape", sig.span()) {
        return;
    }
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
        && !LIFECYCLE.contains(&name.as_str())
        && !args.last().is_some_and(|ty| handler_callback(ty, sig))
    {
        source.error(
            "api-shape",
            sig.span(),
            "on_ method requires a last argument of impl Into<Callback<()>> or impl Fn(..) -> R with R: HandlerResult",
            errors,
        );
    }
    if method
        && sig.receiver().is_some()
        && returns_self(sig)
        && args.iter().any(|ty| unit_handler(ty, sig))
    {
        source.error(
            "api-shape",
            sig.span(),
            "a builder method takes a handler without a payload as impl Into<Callback<()>> (API-3)",
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
/// The last argument of an `on_*` setter follows API-3.
fn handler_callback(ty: &syn::Type, sig: &syn::Signature) -> bool {
    into_unit_callback(ty) || fn_output_handler(ty, sig)
}

/// `impl Into<Callback<()>>`, also with a qualified path or `+ 'static`.
fn into_unit_callback(ty: &syn::Type) -> bool {
    let syn::Type::ImplTrait(ty) = ty else {
        return false;
    };
    ty.bounds.iter().any(|bound| {
        let syn::TypeParamBound::Trait(bound) = bound else {
            return false;
        };
        let Some(segment) = bound.path.segments.last() else {
            return false;
        };
        if segment.ident != "Into" {
            return false;
        }
        let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            return false;
        };
        arguments.args.iter().any(|argument| {
            let syn::GenericArgument::Type(ty) = argument else {
                return false;
            };
            callback_unit(ty)
        })
    })
}

/// Whether `ty` is `Callback<()>`.
fn callback_unit(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    if segment.ident != "Callback" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    arguments.args.len() == 1
        && matches!(
            arguments.args.first(),
            Some(syn::GenericArgument::Type(syn::Type::Tuple(tuple))) if tuple.elems.is_empty()
        )
}

/// `impl Fn(..) -> R` or a generic `F: Fn(..) -> R`, where `R` has a
/// `HandlerResult` bound. A handler takes a payload: `Fn()` is not one.
fn fn_output_handler(ty: &syn::Type, sig: &syn::Signature) -> bool {
    bounds_of(ty, sig).into_iter().any(|bound| {
        fn_arguments(bound).is_some_and(|arguments| {
            !arguments.inputs.is_empty()
                && matches!(&arguments.output, syn::ReturnType::Type(_, output)
                    if handler_type_param(output, sig))
        })
    })
}

/// `-> Self`.
fn returns_self(sig: &syn::Signature) -> bool {
    matches!(&sig.output, syn::ReturnType::Type(_, ty)
        if matches!(ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("Self")))
}

/// A handler without a payload taken as a closure: an `Fn` bound with no
/// input that returns nothing (`Fn()`, `Fn() -> ()`) or a `HandlerResult`
/// (`Fn() -> R`), written as `impl ..` or as a type parameter bounded in
/// `<..>` or `where`. A factory `Fn() -> V` returns a value and is not one.
fn unit_handler(ty: &syn::Type, sig: &syn::Signature) -> bool {
    bounds_of(ty, sig).into_iter().any(|bound| {
        fn_arguments(bound).is_some_and(|arguments| {
            arguments.inputs.is_empty()
                && match &arguments.output {
                    syn::ReturnType::Default => true,
                    syn::ReturnType::Type(_, output) => {
                        matches!(output.as_ref(), syn::Type::Tuple(tuple) if tuple.elems.is_empty())
                            || handler_type_param(output, sig)
                    }
                }
        })
    })
}

/// The `(..) -> ..` of an `Fn` bound.
fn fn_arguments(bound: &syn::TypeParamBound) -> Option<&syn::ParenthesizedGenericArguments> {
    let syn::TypeParamBound::Trait(bound) = bound else {
        return None;
    };
    let segment = bound.path.segments.last()?;
    match &segment.arguments {
        syn::PathArguments::Parenthesized(arguments) if segment.ident == "Fn" => Some(arguments),
        _ => None,
    }
}

/// The bounds of an argument type: those of `impl ..`, or of the type
/// parameter it names, from `<..>` and from the `where` clause.
fn bounds_of<'a>(ty: &'a syn::Type, sig: &'a syn::Signature) -> Vec<&'a syn::TypeParamBound> {
    match ty {
        syn::Type::ImplTrait(ty) => ty.bounds.iter().collect(),
        syn::Type::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
            let name = &path.path.segments[0].ident;
            let inline = sig
                .generics
                .type_params()
                .filter(|parameter| parameter.ident == *name)
                .flat_map(|parameter| parameter.bounds.iter());
            let clause = sig
                .generics
                .where_clause
                .iter()
                .flat_map(|clause| clause.predicates.iter())
                .filter_map(|predicate| match predicate {
                    syn::WherePredicate::Type(predicate) if predicate.bounded_ty == *ty => {
                        Some(predicate.bounds.iter())
                    }
                    _ => None,
                })
                .flatten();
            inline.chain(clause).collect()
        }
        _ => Vec::new(),
    }
}

/// A type parameter with a `HandlerResult` bound.
fn handler_type_param(ty: &syn::Type, sig: &syn::Signature) -> bool {
    matches!(ty, syn::Type::Path(_))
        && bounds_of(ty, sig).into_iter().any(|bound| {
            matches!(bound, syn::TypeParamBound::Trait(bound)
                if bound.path.segments.last().is_some_and(|segment| segment.ident == "HandlerResult"))
        })
}
fn check_impl(source: &Source, surface: &Surface, item: &syn::ItemImpl, errors: &mut Vec<String>) {
    if !surface
        .resolve_type(source, &item.self_ty)
        .is_some_and(|name| surface.reachable.contains(&name))
    {
        return;
    }
    for member in &item.items {
        if let syn::ImplItem::Fn(method) = member
            && matches!(method.vis, syn::Visibility::Public(_))
            && !source.internal_contract(&method.attrs, method.sig.span())
        {
            check_signature(source, &method.sig, true, errors);
        }
    }
}

#[cfg(test)]
mod tests {
    //! Role: Check public fields, callbacks and ownership only on reachable items.
    #[test]
    fn checks_reachable_public_shapes() {
        let mut source = crate::check_source::fixture(
            "pub struct Widget { pub x: f32, hidden: f32 } impl Widget { pub fn on_press(self, f: impl Into<Callback<()>>) {} pub fn on_bad(self, f: bool) {} pub fn on_plain(self, f: impl Fn()) {} pub fn content(self, f: impl FnOnce()) {} pub fn shared(self) -> Rc<Data> { loop {} } pub fn trigger(self, f: impl Fn()) -> Self { self } } struct Hidden { pub x: f32 }",
        );
        source.module.clear();
        let surface = crate::public_surface::build(std::slice::from_ref(&source));
        let mut errors = Vec::new();
        super::check(&source, &surface, &mut errors);
        assert_eq!(errors.len(), 6);
    }
    #[test]
    fn accepts_handler_bounds_and_lifecycle_exceptions() {
        let mut source = crate::check_source::fixture(
            "pub struct Widget; impl Widget { pub fn on_inline<R: HandlerResult>(self, f: impl Fn() -> R) {} pub fn on_generic<F, R>(self, f: F) where F: Fn(u8) -> R, R: HandlerResult {} pub fn on_path<R>(self, f: impl Fn() -> R) where R: HandlerResult {} pub fn on_callback(self, f: impl Into<Callback<()>>) {} pub fn on_many(self, a: bool, f: impl Into<Callback<()>>) {} pub fn on_mount(self, f: impl FnOnce()) {} pub fn on_cleanup(self, f: impl FnOnce()) {} pub fn on_attach(self, f: impl FnOnce()) {} pub fn on_fake(self, f: impl Into<bool>) {} pub fn on_plain(self, f: impl Fn()) {} }",
        );
        source.module.clear();
        let surface = crate::public_surface::build(std::slice::from_ref(&source));
        let mut errors = Vec::new();
        super::check(&source, &surface, &mut errors);
        assert_eq!(errors.len(), 4, "{errors:?}");
        assert!(errors.iter().all(|error| error.contains("on_ method")));
    }
    #[test]
    fn rejects_a_builder_handler_without_payload_in_every_spelling() {
        let mut source = crate::check_source::fixture(
            "pub struct Widget; impl Widget { pub fn a(self, f: impl Fn()) -> Self { self } pub fn b(self, f: impl Fn() -> () + 'static) -> Self { self } pub fn c<F: Fn()>(self, f: F) -> Self { self } pub fn d<F>(self, f: F) -> Self where F: Fn() -> () + 'static { self } pub fn e<R: HandlerResult>(self, f: impl Fn() -> R) -> Self { self } pub fn slot<V: IntoView>(self, f: impl Fn() -> V) -> Self { self } pub fn pick(self, f: impl Fn(u8)) -> Self { self } pub fn shared(self, f: impl Into<Callback<()>>) -> Self { self } pub fn run(self, f: impl Fn()) {} }",
        );
        source.module.clear();
        let surface = crate::public_surface::build(std::slice::from_ref(&source));
        let mut errors = Vec::new();
        super::check(&source, &surface, &mut errors);
        assert_eq!(errors.len(), 5, "{errors:?}");
        assert!(
            errors.iter().all(|error| error.contains("(API-3)")),
            "{errors:?}"
        );
    }
    #[test]
    fn follows_alias_fields_but_not_private_fields() {
        let mut source = crate::check_source::fixture(
            "pub struct Widget { pub values: Vec<Alias>, private: Hidden } type Alias = Option<f32>; struct Hidden { pub value: f32 }",
        );
        source.module.clear();
        let surface = crate::public_surface::build(std::slice::from_ref(&source));
        let mut errors = Vec::new();
        super::check(&source, &surface, &mut errors);
        assert_eq!(errors.len(), 1);
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
