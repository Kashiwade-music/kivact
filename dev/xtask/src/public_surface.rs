//! Role: Resolve public reexports to definition identities for API checks.
//!
//! Visibility is evaluated from facade and kit roots, while aliases, grouped
//! imports and glob reexports resolve through private implementation modules.
use crate::check_source::Source;
use std::collections::{BTreeMap, BTreeSet};
use syn::spanned::Spanned;

#[derive(Default)]
pub(crate) struct Surface {
    pub(crate) exports: BTreeMap<String, String>,
    pub(crate) reachable: BTreeSet<String>,
    aliases: BTreeMap<String, String>,
    pub(crate) origins: BTreeMap<String, (String, String, proc_macro2::Span)>,
}
struct Binding {
    path: String,
    target: String,
    public: bool,
    origin: (String, String, proc_macro2::Span),
}

pub(crate) fn definition(source: &Source, name: &str) -> String {
    let package = source.identity_root();
    if source.module.is_empty() {
        format!("{package}::{name}")
    } else {
        format!("{package}::{}::{name}", source.module)
    }
}

#[cfg(test)]
pub(crate) fn build(sources: &[Source]) -> Surface {
    build_for(sources, true)
}

pub(crate) fn build_for(sources: &[Source], public_needed: bool) -> Surface {
    let mut bindings = Vec::new();
    let mut globs = Vec::new();
    let mut modules = BTreeSet::new();
    for source in sources {
        let scope = definition(source, "").trim_end_matches("::").to_owned();
        modules.insert(scope.clone());
        for item in &source.items {
            if let syn::Item::Use(item) = item {
                let mut entries = Vec::new();
                flatten(&item.tree, "", &mut entries);
                for (alias, target) in entries {
                    let target = absolutize(&scope, &target);
                    if alias == "*" {
                        globs.push((
                            scope.clone(),
                            target,
                            matches!(item.vis, syn::Visibility::Public(_)),
                        ));
                    } else {
                        bindings.push(Binding {
                            path: format!("{scope}::{alias}"),
                            target,
                            public: matches!(item.vis, syn::Visibility::Public(_)),
                            origin: (source.package.clone(), source.module.clone(), item.span()),
                        });
                    }
                }
                continue;
            }
            if let Some((name, public)) = item_name(item) {
                let path = format!("{scope}::{name}");
                bindings.push(Binding {
                    target: path.clone(),
                    path,
                    public,
                    origin: (source.package.clone(), source.module.clone(), item.span()),
                });
            }
        }
    }
    // Glob expansion is a fixed point because a glob may itself reexport a glob.
    for _ in 0..=sources.len() {
        let aliases: BTreeMap<_, _> = bindings
            .iter()
            .map(|b| (b.path.clone(), b.target.clone()))
            .collect();
        let mut additions = Vec::new();
        let mut known: BTreeSet<String> = bindings.iter().map(|b| b.path.clone()).collect();
        for (scope, target, public) in &globs {
            let target = resolve(target, &aliases);
            for binding in &bindings {
                if binding.public
                    && binding
                        .path
                        .rsplit_once("::")
                        .is_some_and(|(parent, _)| parent == target)
                {
                    let name = binding.path.rsplit("::").next().unwrap_or("");
                    let path = format!("{scope}::{name}");
                    if known.insert(path.clone()) {
                        additions.push(Binding {
                            path,
                            target: binding.target.clone(),
                            public: *public,
                            origin: binding.origin.clone(),
                        });
                    }
                }
            }
        }
        if additions.is_empty() {
            break;
        }
        bindings.extend(additions);
    }
    let aliases: BTreeMap<_, _> = bindings
        .iter()
        .map(|b| (b.path.clone(), b.target.clone()))
        .collect();
    let mut surface = Surface {
        aliases: aliases.clone(),
        ..Surface::default()
    };
    if !public_needed {
        return surface;
    }
    let roots: BTreeSet<_> = sources
        .iter()
        .filter(|s| s.library && (s.package == "kivact" || crate::check_policy::is_kit(&s.package)))
        .map(Source::identity_root)
        .collect();
    let mut pending: Vec<_> = roots
        .into_iter()
        .map(|r| (r.clone(), r.clone(), BTreeSet::from([r])))
        .collect();
    let mut visited = BTreeSet::new();
    while let Some((public_scope, internal_scope, ancestors)) = pending.pop() {
        if !visited.insert((public_scope.clone(), internal_scope.clone())) {
            continue;
        }
        if public_scope.split("::").count() > sources.len() + 4 {
            continue;
        }
        for binding in &bindings {
            if !binding.public
                || !binding
                    .path
                    .rsplit_once("::")
                    .is_some_and(|(parent, _)| parent == internal_scope)
            {
                continue;
            }
            let target = resolve(&binding.target, &aliases);
            let name = binding.path.rsplit("::").next().unwrap_or("");
            let path = format!("{public_scope}::{name}");
            surface.origins.insert(path.clone(), binding.origin.clone());
            if modules.contains(&target) {
                surface.exports.insert(path.clone(), target.clone());
                if !ancestors.contains(&target) {
                    let mut next = ancestors.clone();
                    next.insert(target.clone());
                    pending.push((path, target, next));
                }
            } else {
                surface.reachable.insert(target.clone());
                surface.exports.insert(path, target);
            }
        }
    }
    surface
}
fn resolve(path: &str, aliases: &BTreeMap<String, String>) -> String {
    let mut current = path.to_owned();
    let mut seen = BTreeSet::new();
    let mut used_aliases = BTreeSet::new();
    while seen.insert(current.clone()) {
        let mut changed = false;
        let segments: Vec<_> = current.split("::").collect();
        for length in (1..=segments.len()).rev() {
            let prefix = segments[..length].join("::");
            if let Some(target) = aliases.get(&prefix).filter(|target| **target != prefix) {
                if !used_aliases.insert(prefix.clone()) {
                    return current;
                }
                current = if length == segments.len() {
                    target.clone()
                } else {
                    format!("{target}::{}", segments[length..].join("::"))
                };
                changed = true;
                break;
            }
        }
        if !changed {
            break;
        }
    }
    current
}
fn absolutize(scope: &str, path: &str) -> String {
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
                .map_or(base.clone(), |(p, _)| p.to_owned());
            rest = tail;
        }
        return format!("{base}::{rest}");
    }
    // Workspace crates use their Cargo crate name; other paths are local.
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
fn flatten(tree: &syn::UseTree, prefix: &str, output: &mut Vec<(String, String)>) {
    match tree {
        syn::UseTree::Path(p) => flatten(&p.tree, &format!("{prefix}{}::", p.ident), output),
        syn::UseTree::Name(n) => {
            let target = if n.ident == "self" {
                prefix.trim_end_matches("::").to_owned()
            } else {
                format!("{prefix}{}", n.ident)
            };
            let alias = target.rsplit("::").next().unwrap_or("").to_owned();
            output.push((alias, target));
        }
        syn::UseTree::Rename(n) => {
            let target = if n.ident == "self" {
                prefix.trim_end_matches("::").to_owned()
            } else {
                format!("{prefix}{}", n.ident)
            };
            output.push((n.rename.to_string(), target));
        }
        syn::UseTree::Glob(_) => {
            output.push(("*".into(), prefix.trim_end_matches("::").to_owned()))
        }
        syn::UseTree::Group(g) => {
            for item in &g.items {
                flatten(item, prefix, output);
            }
        }
    }
}
pub(crate) fn item_name(item: &syn::Item) -> Option<(String, bool)> {
    let (name, vis) = match item {
        syn::Item::Struct(i) => (&i.ident, &i.vis),
        syn::Item::Enum(i) => (&i.ident, &i.vis),
        syn::Item::Trait(i) => (&i.ident, &i.vis),
        syn::Item::Type(i) => (&i.ident, &i.vis),
        syn::Item::Fn(i) => (&i.sig.ident, &i.vis),
        syn::Item::Const(i) => (&i.ident, &i.vis),
        syn::Item::Static(i) => (&i.ident, &i.vis),
        syn::Item::Mod(i) => (&i.ident, &i.vis),
        _ => return None,
    };
    Some((name.to_string(), matches!(vis, syn::Visibility::Public(_))))
}

pub(crate) fn export_span(source: &Source) -> proc_macro2::Span {
    source
        .items
        .first()
        .map(Spanned::span)
        .unwrap_or_else(proc_macro2::Span::call_site)
}

#[cfg(test)]
mod tests {
    //! Role: Follow aliases through private modules without exposing private items.
    #[test]
    fn growing_alias_cycles_terminate() {
        let aliases = [
            ("kivact::a".into(), "kivact::b".into()),
            ("kivact::b".into(), "kivact::a::inner".into()),
        ]
        .into();
        assert!(super::resolve("kivact::a", &aliases).len() < 100);
    }
    #[test]
    fn private_module_reexports_and_globs() {
        let mut root = crate::check_source::fixture(
            "mod hidden; pub mod ui; pub use hidden::Thing as PublicThing;",
        );
        root.package = "kivact".into();
        root.module.clear();
        let mut hidden = crate::check_source::fixture("pub struct Thing; pub struct Internal;");
        hidden.package = "kivact".into();
        hidden.module = "hidden".into();
        let mut ui = crate::check_source::fixture("pub use crate::hidden::*;");
        ui.package = "kivact".into();
        ui.module = "ui".into();
        let surface = super::build(&[root, hidden, ui]);
        assert_eq!(
            surface
                .exports
                .get("kivact::PublicThing")
                .map(String::as_str),
            Some("kivact::hidden::Thing")
        );
        assert!(surface.exports.contains_key("kivact::ui::Internal"));
        assert!(!surface.exports.contains_key("kivact::hidden::Thing"));
    }
}

impl Surface {
    pub(crate) fn resolve_type(&self, source: &Source, ty: &syn::Type) -> Option<String> {
        let syn::Type::Path(path) = ty else {
            return None;
        };
        let name = path
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        let scope = definition(source, "");
        Some(resolve(
            &absolutize(scope.trim_end_matches("::"), &name),
            &self.aliases,
        ))
    }
}
