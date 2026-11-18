//! Role: Classify one field type as plain state, interior state, a handle or nothing.

/// How one field takes part in the state of its type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    /// A value, a collection, an enum or a callback setting.
    None,
    /// `bool`, `Option<_>` or a named integer counter.
    Plain,
    /// `Cell`, `RefCell`, `Mutex`, `RwLock` or `Atomic*`, written through `&self`.
    Interior,
    /// `Signal` or `Var`: state when a method of the type writes it.
    Handle,
}

/// One counted field: its label in reports, its field name and its kind.
pub(super) struct Counted {
    pub(super) label: String,
    pub(super) field: String,
    pub(super) kind: Kind,
}

/// The counted fields of a struct or a variant; labels start with `prefix.` when it is not empty.
pub(super) fn counted(fields: &syn::Fields, prefix: &str) -> Vec<Counted> {
    let mut out = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let name = field
            .ident
            .as_ref()
            .map_or_else(|| index.to_string(), ToString::to_string);
        let label = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        if let syn::Type::Tuple(tuple) = &field.ty {
            for (element, ty) in tuple.elems.iter().enumerate() {
                push(&mut out, format!("{label}.{element}"), &name, ty);
            }
        } else {
            push(&mut out, label, &name, &field.ty);
        }
    }
    out
}

fn push(out: &mut Vec<Counted>, label: String, field: &str, ty: &syn::Type) {
    let kind = kind(field, ty);
    if kind != Kind::None {
        out.push(Counted {
            label,
            field: field.to_owned(),
            kind,
        });
    }
}

/// The kind of a field named `name` of type `ty`.
fn kind(name: &str, ty: &syn::Type) -> Kind {
    let Some(segment) = last_segment(peel(ty)) else {
        return Kind::None;
    };
    let outer = segment.ident.to_string();
    match outer.as_str() {
        "Cell" | "RefCell" | "Mutex" | "RwLock" => Kind::Interior,
        "Signal" | "Var" => Kind::Handle,
        "bool" => Kind::Plain,
        "Option" if single_argument(segment).is_some_and(is_setting) => Kind::None,
        "Option" => Kind::Plain,
        _ if outer.starts_with("Atomic") => Kind::Interior,
        _ if is_integer(&outer) && is_counter_name(name) => Kind::Plain,
        _ => Kind::None,
    }
}

/// `ty` without references, parentheses and `Rc`/`Arc`/`Box` wrappers.
fn peel(ty: &syn::Type) -> &syn::Type {
    match ty {
        syn::Type::Reference(reference) => peel(&reference.elem),
        syn::Type::Paren(paren) => peel(&paren.elem),
        syn::Type::Group(group) => peel(&group.elem),
        syn::Type::Path(path) => match path.path.segments.last() {
            Some(segment) if matches!(segment.ident.to_string().as_str(), "Rc" | "Arc" | "Box") => {
                single_argument(segment).map_or(ty, peel)
            }
            _ => ty,
        },
        _ => ty,
    }
}

fn last_segment(ty: &syn::Type) -> Option<&syn::PathSegment> {
    match ty {
        syn::Type::Path(path) => path.path.segments.last(),
        _ => None,
    }
}

fn single_argument(segment: &syn::PathSegment) -> Option<&syn::Type> {
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    if arguments.args.len() != 1 {
        return None;
    }
    match arguments.args.first()? {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }
}

/// A callback or a view factory: an `Option` of one is a setting, not state.
fn is_setting(ty: &syn::Type) -> bool {
    match peel(ty) {
        syn::Type::TraitObject(object) => object.bounds.iter().any(|bound| {
            matches!(bound, syn::TypeParamBound::Trait(bound)
            if bound.path.segments.last().is_some_and(|segment| {
                matches!(segment.ident.to_string().as_str(), "Fn" | "FnMut" | "FnOnce")
            }))
        }),
        ty => last_segment(ty).is_some_and(|segment| {
            matches!(
                segment.ident.to_string().as_str(),
                "Callback" | "Handler" | "ViewFn"
            )
        }),
    }
}

fn is_counter_name(name: &str) -> bool {
    matches!(
        name,
        "generation" | "epoch" | "serial" | "seq" | "token" | "next"
    ) || name.starts_with("next_")
        || name.ends_with("_count")
        || name.starts_with("pending_")
}

fn is_integer(name: &str) -> bool {
    matches!(
        name,
        "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize"
    )
}
