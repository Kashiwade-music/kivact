//! Role: Keep public callback and rendering inputs usable with ordinary closures.
//!
//! A public fn takes a callback as `impl Fn(T) -> R + 'static` (`R:
//! HandlerResult`), never as a bare `Callback` or `impl Into<Callback<_>>`
//! (design §3): a closure's argument is then inferred and a no-argument
//! notification is `|| ..`, not `|()| ..`. Nor does it take a boxed closure
//! (`Box<dyn Fn ..>`, `Rc<dyn Fn ..>`, `Arc<dyn Fn ..>`): the caller would
//! box it and annotate its arguments.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use syn::{
    FnArg, GenericArgument, GenericParam, Item, PathArguments, ReturnType, Signature, Type,
    TypeParamBound, Visibility, WherePredicate,
};

fn view(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().is_some_and(|s| s.ident == "View"))
}

fn render_bound(bound: &TypeParamBound, aliases: &BTreeMap<String, Type>) -> bool {
    let TypeParamBound::Trait(bound) = bound else {
        return false;
    };
    bound
        .path
        .segments
        .iter()
        .any(|segment| match &segment.arguments {
            PathArguments::Parenthesized(args) => {
                matches!(
                    segment.ident.to_string().as_str(),
                    "Fn" | "FnMut" | "FnOnce"
                ) && matches!(&args.output, ReturnType::Type(_, ty) if view(ty))
            }
            PathArguments::AngleBracketed(args) => args
                .args
                .iter()
                .any(|arg| matches!(arg, GenericArgument::Type(ty) if render_type(ty, aliases))),
            PathArguments::None => false,
        })
}

fn render_type(ty: &Type, aliases: &BTreeMap<String, Type>) -> bool {
    match ty {
        Type::ImplTrait(ty) => ty.bounds.iter().any(|b| render_bound(b, aliases)),
        Type::TraitObject(ty) => ty.bounds.iter().any(|b| render_bound(b, aliases)),
        Type::BareFn(ty) => matches!(&ty.output, ReturnType::Type(_, ty) if view(ty)),
        Type::Paren(ty) => render_type(&ty.elem, aliases),
        Type::Group(ty) => render_type(&ty.elem, aliases),
        Type::Reference(ty) => render_type(&ty.elem, aliases),
        Type::Path(ty) => ty.path.segments.iter().any(|segment| {
            if let Some(alias) = aliases.get(&segment.ident.to_string()) {
                // Resolve one alias at a time; recursive aliases must not loop.
                let mut remaining = aliases.clone();
                remaining.remove(&segment.ident.to_string());
                if render_type(alias, &remaining) {
                    return true;
                }
            }
            match &segment.arguments {
                PathArguments::AngleBracketed(args) => args.args.iter().any(
                    |arg| matches!(arg, GenericArgument::Type(ty) if render_type(ty, aliases)),
                ),
                _ => false,
            }
        }),
        _ => false,
    }
}

/// `Callback<_>` or `Option<Callback<_>>` (an optional observer is a
/// builder method taking a closure, not an argument).
fn bare_callback(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => path.path.segments.last().is_some_and(|s| {
            s.ident == "Callback"
                || (s.ident == "Option"
                    && matches!(&s.arguments, PathArguments::AngleBracketed(args)
                        if args.args.iter().any(|arg| matches!(arg, GenericArgument::Type(ty) if bare_callback(ty)))))
        }),
        Type::Paren(ty) => bare_callback(&ty.elem),
        Type::Group(ty) => bare_callback(&ty.elem),
        _ => false,
    }
}

/// A `Fn`, `FnMut` or `FnOnce` trait object.
fn closure_object(ty: &Type) -> bool {
    match ty {
        Type::TraitObject(ty) => ty.bounds.iter().any(|bound| {
            matches!(bound, TypeParamBound::Trait(bound) if bound.path.segments.last().is_some_and(|s| {
                matches!(s.ident.to_string().as_str(), "Fn" | "FnMut" | "FnOnce")
            }))
        }),
        Type::Paren(ty) => closure_object(&ty.elem),
        Type::Group(ty) => closure_object(&ty.elem),
        _ => false,
    }
}

/// A closure in a `Box`, `Rc` or `Arc`, also inside an `Option` or behind a
/// type alias.
fn boxed_closure(ty: &Type, aliases: &BTreeMap<String, Type>) -> bool {
    match ty {
        Type::Paren(ty) => boxed_closure(&ty.elem, aliases),
        Type::Group(ty) => boxed_closure(&ty.elem, aliases),
        Type::Path(path) => path.path.segments.last().is_some_and(|segment| {
            if let Some(alias) = aliases.get(&segment.ident.to_string()) {
                let mut remaining = aliases.clone();
                remaining.remove(&segment.ident.to_string());
                if boxed_closure(alias, &remaining) {
                    return true;
                }
            }
            let PathArguments::AngleBracketed(args) = &segment.arguments else {
                return false;
            };
            let types = || {
                args.args.iter().filter_map(|arg| match arg {
                    GenericArgument::Type(ty) => Some(ty),
                    _ => None,
                })
            };
            match segment.ident.to_string().as_str() {
                "Box" | "Rc" | "Arc" => types().any(closure_object),
                "Option" => types().any(|ty| boxed_closure(ty, aliases)),
                _ => false,
            }
        }),
        _ => false,
    }
}

/// `impl Into<Callback<_>>` in `ty` (at any depth, any path to `Callback`).
fn into_callback(ty: &Type) -> bool {
    let bounds = match ty {
        Type::ImplTrait(ty) => &ty.bounds,
        Type::Paren(ty) => return into_callback(&ty.elem),
        Type::Group(ty) => return into_callback(&ty.elem),
        _ => return false,
    };
    bounds.iter().any(|bound| {
        let TypeParamBound::Trait(bound) = bound else {
            return false;
        };
        bound.path.segments.last().is_some_and(|segment| {
            segment.ident == "Into"
                && matches!(&segment.arguments, PathArguments::AngleBracketed(args)
                    if args.args.iter().any(|arg| matches!(arg, GenericArgument::Type(ty) if bare_callback(ty))))
        })
    })
}

fn signature(sig: &Signature, aliases: &BTreeMap<String, Type>) -> Vec<String> {
    let mut errors = Vec::new();
    for argument in &sig.inputs {
        if let FnArg::Typed(argument) = argument {
            if bare_callback(&argument.ty) || into_callback(&argument.ty) {
                errors.push(format!(
                    "{}: accept impl Fn(T) -> R + 'static, not Callback",
                    sig.ident
                ));
            }
            if boxed_closure(&argument.ty, aliases) {
                errors.push(format!(
                    "{}: accept impl Fn(T) -> R + 'static, not a boxed closure",
                    sig.ident
                ));
            }
            if render_type(&argument.ty, aliases) {
                errors.push(format!(
                    "{}: accept a renderer returning V: IntoView",
                    sig.ident
                ));
            }
        }
    }
    let generic_render = sig.generics.params.iter().any(|param| {
        matches!(param, GenericParam::Type(param) if param.bounds.iter().any(|b| render_bound(b, aliases)))
    });
    let where_render = sig.generics.where_clause.as_ref().is_some_and(|clause| {
        clause.predicates.iter().any(|predicate| {
            matches!(predicate, WherePredicate::Type(ty) if ty.bounds.iter().any(|b| render_bound(b, aliases)))
        })
    });
    if generic_render || where_render {
        errors.push(format!(
            "{}: renderer bounds must return V: IntoView",
            sig.ident
        ));
    }
    errors
}

fn inspect(items: &[Item]) -> Vec<String> {
    let aliases = items
        .iter()
        .filter_map(|item| match item {
            Item::Type(item) => Some((item.ident.to_string(), (*item.ty).clone())),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    let mut errors = Vec::new();
    for item in items {
        match item {
            Item::Fn(item) if matches!(item.vis, Visibility::Public(_)) => {
                errors.extend(signature(&item.sig, &aliases));
            }
            Item::Impl(item) => {
                for member in &item.items {
                    if let syn::ImplItem::Fn(method) = member
                        && matches!(method.vis, Visibility::Public(_))
                    {
                        errors.extend(signature(&method.sig, &aliases));
                    }
                }
            }
            Item::Mod(item) => {
                if let Some((_, items)) = &item.content {
                    errors.extend(inspect(items));
                }
            }
            Item::Trait(item) if matches!(item.vis, Visibility::Public(_)) => {
                for member in &item.items {
                    if let syn::TraitItem::Fn(method) = member {
                        errors.extend(signature(&method.sig, &aliases));
                    }
                }
            }
            _ => {}
        }
    }
    errors
}

fn sources(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in std::fs::read_dir(path).expect("source directory") {
            sources(&entry.expect("source entry").path(), files);
        }
    } else if path.extension().is_some_and(|ext| ext == "rs") {
        files.push(path.to_path_buf());
    }
}

#[test]
fn public_callbacks_accept_closures_and_renderers_accept_into_view() {
    let allowed = syn::parse_file(
        "pub fn f<V: IntoView, R: HandlerResult>(render: impl Fn() -> V, callback: impl Fn(u64) -> R + 'static) {}",
    )
    .expect("closure-friendly signature fixture");
    assert!(inspect(&allowed.items).is_empty());
    for source in [
        "pub fn f(callback:\n Callback<u64>) {}",
        "pub fn new(observer: Option<Callback<EditPhase>>) {}",
        "pub fn f(render: impl Fn(usize)\n -> kivact::view::View) {}",
        "pub fn f<F: Fn() -> View>(render: F) {}",
        "pub fn f<F>(render: F) where F: Fn() -> View {}",
        "type CellRenderer = Rc<dyn Fn(usize) -> View>; pub fn f(render: CellRenderer) {}",
        "impl Component { pub fn f(&self, callback: Callback<u64>) {} }",
        "pub trait Component { fn f(&self, render: impl Fn() -> View); }",
        "pub fn f(callback: Box<dyn Fn(u64)>) {}",
        "pub fn f(callback: std::rc::Rc<dyn Fn(&str) -> bool>) {}",
        "pub fn f(callback: Option<Arc<dyn FnMut() + Send>>) {}",
        "type Compare = Rc<dyn Fn(&u64, &u64) -> Ordering>; pub fn f(compare: Compare) {}",
    ] {
        let parsed = syn::parse_file(source).expect("signature fixture");
        assert!(!inspect(&parsed.items).is_empty(), "missed {source}");
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("workspace root");
    let mut files = Vec::new();
    for entry in std::fs::read_dir(root.join("crates")).expect("crates directory") {
        let src = entry.expect("crate entry").path().join("src");
        if src.is_dir() {
            sources(&src, &mut files);
        }
    }
    files.sort();
    // The runtime layers below the public facade (Arvane, the engine, the
    // reactive runtime) hand boxed closures between their own crates; the
    // boxed-closure rule is for the API that apps, kits and components call.
    let runtime_layer = |file: &Path| {
        let name = file
            .strip_prefix(root.join("crates"))
            .ok()
            .and_then(|path| path.components().next())
            .map(|crate_dir| crate_dir.as_os_str().to_string_lossy().into_owned())
            .unwrap_or_default();
        name.starts_with("arvane")
            || matches!(
                name.as_str(),
                "kivact-engine"
                    | "kivact-editing"
                    | "kivact-wire"
                    | "kivact-lower"
                    | "kivact-text"
                    | "kivact-reactive"
                    | "kivact-task"
                    | "kivact-types"
            )
    };
    let mut errors = Vec::new();
    for file in files {
        let source = std::fs::read_to_string(&file).expect("Rust source");
        let parsed = syn::parse_file(&source).expect("valid Rust source");
        let runtime = runtime_layer(&file);
        let found = inspect(&parsed.items)
            .into_iter()
            .filter(|error| !(runtime && error.ends_with("not a boxed closure")));
        errors.extend(found.map(|error| {
            format!(
                "{}: {error}",
                file.strip_prefix(root).expect("workspace source").display()
            )
        }));
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}
