//! Role: Compare constructor inputs and bounds without argument names or spacing.
use super::source::{Module, uses};
use syn::{
    GenericArgument, GenericParam, PathArguments, ReturnType, Type, TypeParamBound, WherePredicate,
};

fn facade(path: &str) -> &str {
    match path {
        "kivact::view::IntoText" | "kivact::prelude::IntoText" => "IntoText",
        "kivact::view::IntoValue" | "kivact::prelude::IntoValue" => "IntoValue",
        "kivact::Binding" | "kivact::prelude::Binding" => "Binding",
        _ => path,
    }
}

fn imported(name: &str, module: Option<&Module>) -> String {
    let Some(module) = module else {
        return name.to_owned();
    };
    for item in &module.items {
        let local = match item {
            syn::Item::Struct(item) => Some(&item.ident),
            syn::Item::Enum(item) => Some(&item.ident),
            syn::Item::Trait(item) => Some(&item.ident),
            syn::Item::Type(item) => Some(&item.ident),
            _ => None,
        };
        if local.is_some_and(|local| local == name) {
            return format!("{}::{}::{name}", module.crate_name, module.path.join("::"));
        }
        if let syn::Item::Use(item) = item {
            let mut leaves = Vec::new();
            uses(&item.tree, &[], &mut leaves);
            for (path, exposed) in leaves {
                if exposed == name {
                    return qualified(&path.join("::"), Some(module));
                }
            }
        }
    }
    qualified(name, Some(module))
}

fn qualified(name: &str, module: Option<&Module>) -> String {
    let canonical = facade(name);
    if canonical != name {
        return canonical.to_owned();
    }
    let Some(module) = module else {
        return name.to_owned();
    };
    if matches!(
        name,
        "bool"
            | "char"
            | "str"
            | "usize"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "f32"
            | "f64"
            | "String"
            | "Vec"
            | "Option"
            | "Result"
            | "Into"
            | "From"
            | "Clone"
            | "Copy"
            | "Send"
            | "Sync"
            | "Sized"
            | "Fn"
            | "FnMut"
            | "FnOnce"
    ) {
        return name.to_owned();
    }
    let parts = name.split("::").collect::<Vec<_>>();
    if matches!(parts[0], "std" | "core" | "alloc") || parts[0].starts_with("kivact") {
        return name.to_owned();
    }
    let mut prefix = module.path.clone();
    let mut rest = parts.as_slice();
    match rest[0] {
        "crate" => {
            prefix.clear();
            rest = &rest[1..];
        }
        "self" => rest = &rest[1..],
        _ => {}
    }
    while rest.first() == Some(&"super") {
        prefix.pop().expect("super has a parent");
        rest = &rest[1..];
    }
    prefix.extend(rest.iter().map(|part| (*part).to_owned()));
    format!("{}::{}", module.crate_name, prefix.join("::"))
}
fn path(path: &syn::Path, module: Option<&Module>) -> String {
    let names = path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");
    let mut name = if path.segments.len() == 1 {
        imported(&names, module)
    } else {
        qualified(&names, module)
    };
    // Generic arguments belong to their own segment, even for qualified types.
    if path
        .segments
        .iter()
        .take(path.segments.len().saturating_sub(1))
        .any(|s| !matches!(s.arguments, PathArguments::None))
    {
        panic!("generic module paths need explicit constructor signature policy support");
    }
    name.push_str(&arguments(
        &path.segments.last().expect("path segment").arguments,
        module,
    ));
    name
}

fn arguments(args: &PathArguments, module: Option<&Module>) -> String {
    match args {
        PathArguments::None => String::new(),
        PathArguments::Parenthesized(args) => format!("({}){}", args.inputs.iter().map(|ty| ty_name(ty, module)).collect::<Vec<_>>().join(","), output(&args.output, module)),
        PathArguments::AngleBracketed(args) => format!("<{}>", args.args.iter().map(|arg| match arg {
            GenericArgument::Type(ty) => ty_name(ty, module),
            GenericArgument::Lifetime(value) => value.to_string(),
            GenericArgument::Const(value) => expression(value),
            GenericArgument::AssocType(value) => format!("{}{}={}", value.ident, value.generics.as_ref().map_or_else(String::new, |args| arguments(&PathArguments::AngleBracketed(args.clone()), module)), ty_name(&value.ty, module)),
            GenericArgument::Constraint(value) => format!("{}:{}", value.ident, bounds(&value.bounds, module)),
            _ => panic!("unsupported constructor generic argument; extend signature policy explicitly"),
        }).collect::<Vec<_>>().join(",")),
    }
}

fn expression(expr: &syn::Expr) -> String {
    match expr {
        syn::Expr::Lit(expr) => match &expr.lit {
            syn::Lit::Int(value) => value.to_string(),
            syn::Lit::Bool(value) => value.value.to_string(),
            _ => panic!("unsupported constructor constant literal"),
        },
        syn::Expr::Path(expr) => path(&expr.path, None),
        _ => panic!("unsupported constructor constant expression"),
    }
}

fn output(value: &ReturnType, module: Option<&Module>) -> String {
    match value {
        ReturnType::Default => String::new(),
        ReturnType::Type(_, ty) => format!("->{}", ty_name(ty, module)),
    }
}

fn lifetimes(value: &Option<syn::BoundLifetimes>) -> String {
    value.as_ref().map_or_else(String::new, |value| {
        format!(
            "for<{}>",
            value
                .lifetimes
                .iter()
                .map(|param| parameter(param, None))
                .collect::<Vec<_>>()
                .join(",")
        )
    })
}

fn bounds(
    value: &syn::punctuated::Punctuated<TypeParamBound, syn::Token![+]>,
    module: Option<&Module>,
) -> String {
    let mut values = value
        .iter()
        .map(|bound| match bound {
            TypeParamBound::Trait(value) => format!(
                "{}{}{}",
                if matches!(value.modifier, syn::TraitBoundModifier::Maybe(_)) {
                    "?"
                } else {
                    ""
                },
                lifetimes(&value.lifetimes),
                path(&value.path, module)
            ),
            TypeParamBound::Lifetime(value) => value.to_string(),
            _ => panic!("unsupported constructor bound; extend signature policy explicitly"),
        })
        .collect::<Vec<_>>();
    values.sort();
    values.join("+")
}

fn ty_name(ty: &Type, module: Option<&Module>) -> String {
    match ty {
        Type::Path(value) => {
            assert!(
                value.qself.is_none(),
                "qualified-self inputs need explicit signature policy support"
            );
            path(&value.path, module)
        }
        Type::ImplTrait(value) => format!("impl {}", bounds(&value.bounds, module)),
        Type::TraitObject(value) => format!("dyn {}", bounds(&value.bounds, module)),
        Type::Reference(value) => format!(
            "&{}{}{}",
            value
                .lifetime
                .as_ref()
                .map_or_else(String::new, ToString::to_string),
            if value.mutability.is_some() {
                "mut "
            } else {
                ""
            },
            ty_name(&value.elem, module)
        ),
        Type::Ptr(value) => format!(
            "*{} {}",
            if value.mutability.is_some() {
                "mut"
            } else {
                "const"
            },
            ty_name(&value.elem, module)
        ),
        Type::Tuple(value) => format!(
            "({})",
            value
                .elems
                .iter()
                .map(|ty| ty_name(ty, module))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Type::Slice(value) => format!("[{}]", ty_name(&value.elem, module)),
        Type::Array(value) => format!(
            "[{};{}]",
            ty_name(&value.elem, module),
            expression(&value.len)
        ),
        Type::Paren(value) => ty_name(&value.elem, module),
        Type::Group(value) => ty_name(&value.elem, module),
        Type::Never(_) => "!".into(),
        Type::Infer(_) => "_".into(),
        _ => panic!("unsupported constructor input type; extend signature policy explicitly"),
    }
}

fn parameter(param: &GenericParam, module: Option<&Module>) -> String {
    match param {
        GenericParam::Type(value) => format!(
            "{}:{}{}",
            value.ident,
            bounds(&value.bounds, module),
            value
                .default
                .as_ref()
                .map_or_else(String::new, |ty| format!("={}", ty_name(ty, module)))
        ),
        GenericParam::Lifetime(value) => {
            let mut bounds = value
                .bounds
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            bounds.sort();
            format!("{}:{}", value.lifetime, bounds.join("+"))
        }
        GenericParam::Const(value) => format!(
            "const {}:{}{}",
            value.ident,
            ty_name(&value.ty, module),
            value
                .default
                .as_ref()
                .map_or_else(String::new, |expr| format!("={}", expression(expr)))
        ),
    }
}

pub(crate) fn signature(sig: &syn::Signature, module: Option<&Module>) -> String {
    let generic = sig
        .generics
        .params
        .iter()
        .map(|param| parameter(param, module))
        .collect::<Vec<_>>()
        .join(",");
    let inputs = sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Typed(value) => ty_name(&value.ty, module),
            syn::FnArg::Receiver(_) => "self".into(),
        })
        .collect::<Vec<_>>()
        .join(",");
    let mut predicates = sig
        .generics
        .where_clause
        .as_ref()
        .map_or_else(Vec::new, |clause| {
            clause
                .predicates
                .iter()
                .map(|predicate| match predicate {
                    WherePredicate::Type(value) => format!(
                        "{}{}:{}",
                        lifetimes(&value.lifetimes),
                        ty_name(&value.bounded_ty, module),
                        bounds(&value.bounds, module)
                    ),
                    WherePredicate::Lifetime(value) => {
                        let mut bounds = value
                            .bounds
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>();
                        bounds.sort();
                        format!("{}:{}", value.lifetime, bounds.join("+"))
                    }
                    _ => panic!("unsupported constructor where predicate"),
                })
                .collect::<Vec<_>>()
        });
    predicates.sort();
    format!(
        "<{generic}>({inputs}) where {}{}",
        predicates.join(","),
        if sig.variadic.is_some() { ",..." } else { "" }
    )
}
