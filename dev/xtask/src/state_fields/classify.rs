//! Role: Classify a field or static type as phase, stored state or neither.

use crate::check_policy::strings;
use crate::check_source::Source;
use crate::public_surface::Surface;
use std::collections::BTreeSet;

/// How a field or static takes part in a state combination.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Class {
    None,
    Phase,
    Stored,
    /// A named counter that counts only beside another stored field (SL-3).
    Helper,
}

/// The workspace facts that classify one field.
#[derive(Clone, Copy)]
pub(super) struct RuleContext<'a> {
    pub(super) source: &'a Source,
    pub(super) surface: &'a Surface,
    pub(super) enums: &'a BTreeSet<String>,
    pub(super) policy: &'a toml::Table,
}

/// The outermost type name, ignoring references and `Box`.
pub(super) fn outer(ty: &syn::Type) -> String {
    let Some(segment) = last_segment(ty) else {
        return String::new();
    };
    let name = segment.ident.to_string();
    if name == "Box"
        && let Some(inner) = single_type_argument(segment)
    {
        return outer(inner);
    }
    name
}

/// Strip the outermost wrappers named in `names`, one type argument at a time.
pub(super) fn peel(ty: &syn::Type, names: &[&str]) -> syn::Type {
    let mut current = ty.clone();
    while let Some(segment) = last_segment(&current) {
        if !names.contains(&segment.ident.to_string().as_str()) {
            break;
        }
        let Some(inner) = single_type_argument(segment).cloned() else {
            break;
        };
        current = inner;
    }
    current
}

/// Classify one field or static, honouring Api/role/record exemptions.
pub(super) fn classify(rules: &RuleContext<'_>, name: &str, ty: &syn::Type, exempt: bool) -> Class {
    let bare = peel(ty, &["Cell", "RefCell", "Rc", "Arc"]);
    let bare_name = outer(&bare);
    if matches!(bare_name.as_str(), "Value" | "Binding") {
        return Class::None;
    }
    if strings(rules.policy, "named_helpers").contains(&bare_name) {
        return if is_named_counter_helper(&bare_name) {
            Class::Helper
        } else {
            Class::None
        };
    }
    if is_lazy_slot(&bare_name) {
        return Class::Stored;
    }
    let phase = peel(ty, &["Cell", "RefCell", "Signal", "Var", "Rc"]);
    if !exempt && is_phase(&phase, rules.source, rules.surface, rules.enums) {
        return Class::Phase;
    }
    if exempt {
        return if is_atomic(ty) || is_counted_mutable(ty) {
            Class::Stored
        } else {
            Class::None
        };
    }
    if is_option(ty)
        || outer(ty) == "bool"
        || is_atomic(ty)
        || is_counted_mutable(ty)
        || is_named_counter(name, ty)
        || is_state_collection(name, ty)
    {
        return Class::Stored;
    }
    Class::None
}

/// The named counters that count beside another stored field (SL-3).
fn is_named_counter_helper(name: &str) -> bool {
    matches!(name, "Generation" | "Sent" | "Revision" | "Epoch")
}

/// A slot filled after construction (SL-1).
fn is_lazy_slot(name: &str) -> bool {
    matches!(name, "OnceCell" | "OnceLock" | "LazyCell" | "LazyLock")
}

/// A queue of work to do, such as `pending` or `dirty` plans (SL-6).
fn is_state_collection(name: &str, ty: &syn::Type) -> bool {
    matches!(
        outer(ty).as_str(),
        "Vec" | "VecDeque" | "BTreeSet" | "HashSet"
    ) && ["pending", "dirty", "awaiting", "relayout", "stale"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

fn is_phase(ty: &syn::Type, source: &Source, surface: &Surface, enums: &BTreeSet<String>) -> bool {
    if outer(ty) == "MachineHandle" {
        return true;
    }
    phase_identity(source, surface, ty, enums).is_some()
}

fn phase_identity(
    source: &Source,
    surface: &Surface,
    ty: &syn::Type,
    enums: &BTreeSet<String>,
) -> Option<String> {
    let name = outer(ty);
    if let Some(identity) = surface.resolve_type(source, ty)
        && is_phase_enum(&identity, enums)
    {
        return Some(identity);
    }
    let scope = crate::public_surface::definition(source, "");
    let scope = scope.trim_end_matches("::");
    if source
        .items
        .iter()
        .any(|item| matches!(item, syn::Item::Enum(item) if item.ident == name))
    {
        let identity = crate::public_surface::definition(source, &name);
        if is_phase_enum(&identity, enums) {
            return Some(identity);
        }
    }
    for item in &source.items {
        let syn::Item::Use(item) = item else {
            continue;
        };
        let mut entries = Vec::new();
        flatten_use(&item.tree, "", &mut entries);
        for (alias, target) in entries {
            if alias != name && alias != "*" {
                continue;
            }
            let target = absolutize_path(scope, &target);
            let identity = if alias == "*" {
                format!("{target}::{name}")
            } else {
                target
            };
            if is_phase_enum(&identity, enums) {
                return Some(identity);
            }
        }
    }
    None
}

/// Whether `identity` names a workspace enum that is a phase (SL-2 widens the names).
fn is_phase_enum(identity: &str, enums: &BTreeSet<String>) -> bool {
    let named = [
        "State",
        "Phase",
        "Status",
        "Stage",
        "Connection",
        "Lifecycle",
        "Run",
        "Present",
    ]
    .iter()
    .any(|suffix| identity.ends_with(suffix));
    named && enums.contains(identity)
}

fn flatten_use(tree: &syn::UseTree, prefix: &str, output: &mut Vec<(String, String)>) {
    match tree {
        syn::UseTree::Path(path) => {
            flatten_use(&path.tree, &format!("{prefix}{}::", path.ident), output);
        }
        syn::UseTree::Name(name) => {
            let target = if name.ident == "self" {
                prefix.trim_end_matches("::").to_owned()
            } else {
                format!("{prefix}{}", name.ident)
            };
            let alias = target.rsplit("::").next().unwrap_or("").to_owned();
            output.push((alias, target));
        }
        syn::UseTree::Rename(rename) => {
            let target = if rename.ident == "self" {
                prefix.trim_end_matches("::").to_owned()
            } else {
                format!("{prefix}{}", rename.ident)
            };
            output.push((rename.rename.to_string(), target));
        }
        syn::UseTree::Glob(_) => {
            output.push(("*".into(), prefix.trim_end_matches("::").to_owned()));
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                flatten_use(item, prefix, output);
            }
        }
    }
}

fn absolutize_path(scope: &str, path: &str) -> String {
    let root = scope.split("::").next().unwrap_or(scope);
    if let Some(rest) = path.strip_prefix("crate::") {
        return format!("{root}::{rest}");
    }
    if let Some(rest) = path.strip_prefix("self::") {
        return format!("{scope}::{rest}");
    }
    if path.starts_with("super::") {
        let mut base = scope.to_owned();
        let mut rest = path;
        while let Some(tail) = rest.strip_prefix("super::") {
            base = base
                .rsplit_once("::")
                .map_or(base.clone(), |(parent, _)| parent.to_owned());
            rest = tail;
        }
        return format!("{base}::{rest}");
    }
    if path.starts_with("kivact")
        || path.starts_with("arvane")
        || path.starts_with("std::")
        || path.starts_with("core::")
    {
        path.to_owned()
    } else {
        format!("{scope}::{path}")
    }
}

fn is_option(ty: &syn::Type) -> bool {
    option_payload(ty).is_some_and(|payload| !is_setting_payload(payload))
}

fn option_payload(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(path) = unbox(ty) else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    single_type_argument(segment)
}

fn is_setting_payload(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = strip_borrow(ty) else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    match segment.ident.to_string().as_str() {
        "Callback" | "Handler" | "ViewFn" => true,
        "Rc" | "Arc" | "Box" => single_type_argument(segment).is_some_and(is_fn_object),
        _ => false,
    }
}

fn is_fn_object(ty: &syn::Type) -> bool {
    let syn::Type::TraitObject(object) = strip_borrow(ty) else {
        return false;
    };
    object.bounds.iter().any(|bound| {
        let syn::TypeParamBound::Trait(bound) = bound else {
            return false;
        };
        bound
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "Fn")
    })
}

fn is_atomic(ty: &syn::Type) -> bool {
    outer(&peel(ty, &["Rc", "Arc"])).starts_with("Atomic")
}

fn is_counted_mutable(ty: &syn::Type) -> bool {
    let mut current = peel(ty, &["Rc", "Arc"]);
    let mut stripped = false;
    loop {
        let name = outer(&current);
        if !matches!(
            name.as_str(),
            "Cell" | "RefCell" | "Signal" | "Var" | "Mutex" | "RwLock"
        ) {
            break;
        }
        let Some(inner) = last_segment(&current)
            .and_then(single_type_argument)
            .cloned()
        else {
            break;
        };
        current = peel(&inner, &["Rc", "Arc"]);
        stripped = true;
    }
    if !stripped {
        return false;
    }
    matches!(outer(&current).as_str(), "bool" | "Option") || is_integer(&outer(&current))
}

fn is_named_counter(name: &str, ty: &syn::Type) -> bool {
    if !is_integer(&outer(ty)) {
        return false;
    }
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

fn strip_borrow(ty: &syn::Type) -> &syn::Type {
    match ty {
        syn::Type::Reference(reference) => strip_borrow(&reference.elem),
        syn::Type::Paren(paren) => strip_borrow(&paren.elem),
        syn::Type::Group(group) => strip_borrow(&group.elem),
        _ => ty,
    }
}

fn unbox(ty: &syn::Type) -> &syn::Type {
    match ty {
        syn::Type::Reference(reference) => unbox(&reference.elem),
        syn::Type::Paren(paren) => unbox(&paren.elem),
        syn::Type::Group(group) => unbox(&group.elem),
        syn::Type::Path(path) => {
            let Some(segment) = path.path.segments.last() else {
                return ty;
            };
            if segment.ident == "Box"
                && let Some(inner) = single_type_argument(segment)
            {
                return unbox(inner);
            }
            ty
        }
        _ => ty,
    }
}

fn last_segment(ty: &syn::Type) -> Option<&syn::PathSegment> {
    match ty {
        syn::Type::Reference(reference) => last_segment(&reference.elem),
        syn::Type::Paren(paren) => last_segment(&paren.elem),
        syn::Type::Group(group) => last_segment(&group.elem),
        syn::Type::Path(path) => path.path.segments.last(),
        _ => None,
    }
}

fn single_type_argument(segment: &syn::PathSegment) -> Option<&syn::Type> {
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
