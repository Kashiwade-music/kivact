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
    let (mut bindings, globs, modules) = source_bindings(sources);
    expand_globs(sources.len(), &mut bindings, &globs);
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
    let edges = crate::public_type_edges::collect(sources, definition, |source, ty| {
        surface.resolve_type(source, ty)
    });
    crate::public_type_edges::extend(&mut surface.reachable, edges);
    surface
}
fn internal_item(source: &Source, item: &syn::Item) -> bool {
    let attrs = match item {
        syn::Item::Mod(item) => &item.attrs,
        _ => return false,
    };
    source.internal_contract(attrs, item.span())
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

type Globs = Vec<(String, String, bool)>;
fn source_bindings(sources: &[Source]) -> (Vec<Binding>, Globs, BTreeSet<String>) {
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
                    public: public && !internal_item(source, item),
                    origin: (source.package.clone(), source.module.clone(), item.span()),
                });
            }
        }
    }
    (bindings, globs, modules)
}

fn expand_globs(source_count: usize, bindings: &mut Vec<Binding>, globs: &Globs) {
    // Glob expansion is a fixed point because a glob may itself reexport a glob.
    for _ in 0..=source_count {
        let aliases: BTreeMap<_, _> = bindings
            .iter()
            .map(|b| (b.path.clone(), b.target.clone()))
            .collect();
        let mut additions = Vec::new();
        let mut known: BTreeSet<String> = bindings.iter().map(|b| b.path.clone()).collect();
        for (scope, target, public) in globs {
            let target = resolve(target, &aliases);
            for binding in bindings.iter() {
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
}

#[cfg(test)]
mod tests {
    //! Role: Follow aliases through private modules without exposing private items.

    fn source(module: &str, text: &str) -> crate::check_source::Source {
        let mut source = crate::check_source::fixture(text);
        source.package = "kivact".into();
        source.module = module.into();
        source
    }

    #[test]
    fn exposed_signatures_follow_hidden_nested_fields_and_recursive_enum_payloads() {
        let sources = [
            source(
                "",
                "mod hidden; pub fn make() -> hidden::Wrapper { loop {} } pub fn accept(value: hidden::Payload) {}",
            ),
            source(
                "hidden",
                "pub struct Wrapper { pub values: Option<Vec<Color>>, pub recursive: Option<Box<Wrapper>>, private: Internal } pub struct Color { pub channel: f32 } pub enum Payload { Item(Color), Again(Box<Payload>) } pub struct Internal { pub value: f32 }",
            ),
        ];
        let surface = super::build(&sources);
        for name in ["Wrapper", "Color", "Payload"] {
            assert!(
                surface
                    .reachable
                    .contains(&format!("kivact::hidden::{name}"))
            );
        }
        assert!(!surface.reachable.contains("kivact::hidden::Internal"));
        assert_eq!(surface.exports.len(), 2);
        assert!(!surface.exports.contains_key("kivact::hidden::Color"));
        let mut errors = Vec::new();
        for source in &sources {
            crate::api_shape::check(source, &surface, &mut errors);
        }
        assert_eq!(errors.len(), 1);
        for source in &sources {
            crate::canonical_path::check(source, &surface, &mut errors);
        }
        assert_eq!(
            errors.len(),
            1,
            "derived type reachability must not create public export paths"
        );
    }

    #[test]
    fn public_inherent_methods_expose_input_and_return_types_across_modules() {
        let sources = [
            source("", "pub struct Widget; mod hidden; mod methods;"),
            source(
                "hidden",
                "pub struct Returned { pub x: f32 } pub struct Input { pub x: f32 } pub struct Private { pub x: f32 }",
            ),
            source(
                "methods",
                "use crate::Widget; impl Widget { pub fn get(&self) -> crate::hidden::Returned { loop {} } pub fn accept(&self, value: crate::hidden::Input) {} fn implementation(&self) -> crate::hidden::Private { loop {} } }",
            ),
        ];
        let surface = super::build(&sources);
        assert!(surface.reachable.contains("kivact::hidden::Returned"));
        assert!(surface.reachable.contains("kivact::hidden::Input"));
        assert!(!surface.reachable.contains("kivact::hidden::Private"));
        let mut errors = Vec::new();
        for source in &sources {
            crate::api_shape::check(source, &surface, &mut errors);
        }
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn generic_parameter_shadowing_and_private_implementation_types_stay_hidden() {
        let sources = [
            source(
                "",
                "mod hidden; pub fn make() -> hidden::Generic<()> { let implementation: hidden::Internal; loop {} }",
            ),
            source(
                "hidden",
                "pub struct Generic<T> { pub value: T, private: Internal } pub struct T { pub x: f32 } pub struct Internal { pub x: f32 } impl<X> Generic<X> { pub fn project<T>(&self, value: T) -> T { value } }",
            ),
        ];
        let surface = super::build(&sources);
        assert!(surface.reachable.contains("kivact::hidden::Generic"));
        assert!(!surface.reachable.contains("kivact::hidden::T"));
        assert!(!surface.reachable.contains("kivact::hidden::Internal"));
        let mut errors = Vec::new();
        for source in &sources {
            crate::api_shape::check(source, &surface, &mut errors);
        }
        assert!(errors.is_empty());
    }

    #[test]
    fn public_aliases_and_written_associated_generic_arguments_expose_types() {
        let sources = [
            source(
                "",
                "mod hidden; pub fn make() -> impl Iterator<Item = hidden::Alias> { loop {} }",
            ),
            source(
                "hidden",
                "pub type Alias = Option<Color>; pub struct Color { pub x: f32 }",
            ),
        ];
        let surface = super::build(&sources);
        assert!(surface.reachable.contains("kivact::hidden::Alias"));
        assert!(surface.reachable.contains("kivact::hidden::Color"));
    }
    #[test]
    fn only_explicit_hidden_contract_edges_leave_the_user_surface() {
        let sources = [
            source(
                "",
                "pub struct Widget; mod hidden;\n// xtask-allow(internal-contract): Macro implementation support.\n#[doc(hidden)] pub mod __private;\n",
            ),
            source(
                "hidden",
                "pub struct Visible { pub x: f32 } pub struct Internal { pub x: f32 } pub struct StillPublic { pub x: f32 }",
            ),
            source("__private", "pub struct MacroSupport { pub x: f32 }"),
            source(
                "methods",
                "use crate::Widget; impl Widget {\n// xtask-allow(internal-contract): Cross-crate mounting bridge.\n#[doc(hidden)] pub fn bridge(&self) -> crate::hidden::Internal { loop {} }\n#[doc(hidden)] pub fn visible(&self) -> crate::hidden::Visible { loop {} }\n// xtask-allow(internal-contract): Missing hidden attribute is not an internal contract.\npub fn still_public(&self) -> crate::hidden::StillPublic { loop {} } }",
            ),
        ];
        let surface = super::build(&sources);
        assert!(surface.reachable.contains("kivact::hidden::Visible"));
        assert!(surface.reachable.contains("kivact::hidden::StillPublic"));
        assert!(!surface.reachable.contains("kivact::hidden::Internal"));
        assert!(
            !surface
                .reachable
                .contains("kivact::__private::MacroSupport")
        );
        assert!(!surface.exports.contains_key("kivact::__private"));
        let mut errors = Vec::new();
        for source in &sources {
            crate::api_shape::check(source, &surface, &mut errors);
        }
        assert_eq!(errors.len(), 2);
    }
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
