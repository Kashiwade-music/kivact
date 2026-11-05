//! Role: Build finite API type dependencies from exposed Rust type positions.
//!
//! Public fields, enum payloads, aliases and function or inherent-method
//! signatures can expose types from private modules without reexporting their
//! names. Bodies, private fields and private methods do not expose their types.
//! This source check does not expand macros, infer impl Trait's concrete return
//! type, normalize associated types, or substitute generic implementations.
//! Generic arguments written in signatures are followed, while parameters that
//! shadow source type names are excluded from name resolution.

use crate::check_source::Source;
use std::collections::{BTreeMap, BTreeSet};
use syn::spanned::Spanned;
use syn::visit::Visit;

type Edges = BTreeMap<String, BTreeSet<String>>;

pub(crate) fn collect(
    sources: &[Source],
    definition: impl Fn(&Source, &str) -> String,
    resolve: impl Fn(&Source, &syn::Type) -> Option<String>,
) -> Edges {
    let mut edges = Edges::new();
    for source in sources.iter().filter(|s| s.library) {
        for item in &source.items {
            let (owner, generics) = match item {
                syn::Item::Struct(i) if public(&i.vis) => {
                    (definition(source, &i.ident.to_string()), &i.generics)
                }
                syn::Item::Enum(i) if public(&i.vis) => {
                    (definition(source, &i.ident.to_string()), &i.generics)
                }
                syn::Item::Type(i) => (definition(source, &i.ident.to_string()), &i.generics),
                syn::Item::Trait(i) if public(&i.vis) => {
                    (definition(source, &i.ident.to_string()), &i.generics)
                }
                syn::Item::Fn(i) if public(&i.vis) => (
                    definition(source, &i.sig.ident.to_string()),
                    &i.sig.generics,
                ),
                syn::Item::Const(i) if public(&i.vis) => {
                    (definition(source, &i.ident.to_string()), &i.generics)
                }
                syn::Item::Static(i) if public(&i.vis) => {
                    let mut visitor = Types::new(source, &resolve, &syn::Generics::default());
                    visitor.visit_type(&i.ty);
                    edges
                        .entry(definition(source, &i.ident.to_string()))
                        .or_default()
                        .extend(visitor.found);
                    continue;
                }
                syn::Item::Impl(i) if i.trait_.is_none() => {
                    let Some(owner) = resolve(source, &i.self_ty) else {
                        continue;
                    };
                    let mut visitor = Types::new(source, &resolve, &i.generics);
                    for member in &i.items {
                        if let syn::ImplItem::Fn(method) = member
                            && public(&method.vis)
                            && !source.internal_contract(&method.attrs, method.sig.span())
                        {
                            visitor.signature(&method.sig);
                        }
                    }
                    edges.entry(owner).or_default().extend(visitor.found);
                    continue;
                }
                _ => continue,
            };
            let mut visitor = Types::new(source, &resolve, generics);
            visitor.visit_generics(generics);
            exposed_types(item, source, &mut visitor);
            edges.entry(owner).or_default().extend(visitor.found);
        }
    }
    edges
}

fn public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

struct Types<'a, F> {
    source: &'a Source,
    resolve: &'a F,
    parameters: BTreeSet<String>,
    found: BTreeSet<String>,
}
impl<'a, F: Fn(&Source, &syn::Type) -> Option<String>> Types<'a, F> {
    fn new(source: &'a Source, resolve: &'a F, generics: &syn::Generics) -> Self {
        Self {
            source,
            resolve,
            parameters: generics
                .type_params()
                .map(|p| p.ident.to_string())
                .collect(),
            found: BTreeSet::new(),
        }
    }
    fn signature(&mut self, signature: &syn::Signature) {
        let previous = self.parameters.clone();
        self.parameters.extend(
            signature
                .generics
                .type_params()
                .map(|p| p.ident.to_string()),
        );
        self.visit_signature(signature);
        self.parameters = previous;
    }
}
impl<'ast, F: Fn(&Source, &syn::Type) -> Option<String>> Visit<'ast> for Types<'_, F> {
    fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
        if ty.qself.is_none()
            && !ty.path.segments.first().is_some_and(|s| {
                s.ident == "Self" || self.parameters.contains(&s.ident.to_string())
            })
            && let Some(identity) = (self.resolve)(self.source, &syn::Type::Path(ty.clone()))
        {
            self.found.insert(identity);
        }
        syn::visit::visit_type_path(self, ty);
    }
    fn visit_trait_bound(&mut self, bound: &'ast syn::TraitBound) {
        self.visit_type_path(&syn::TypePath {
            qself: None,
            path: bound.path.clone(),
        });
    }
}

/// Grow only over known declarations; recursive and mutually recursive types
/// terminate after each identity has been visited once. Export paths stay intact.
pub(crate) fn extend(reachable: &mut BTreeSet<String>, edges: Edges) {
    let mut pending: Vec<_> = reachable.iter().cloned().collect();
    while let Some(identity) = pending.pop() {
        if let Some(types) = edges.get(&identity) {
            for ty in types {
                if edges.contains_key(ty) && reachable.insert(ty.clone()) {
                    pending.push(ty.clone());
                }
            }
        }
    }
}

fn exposed_types<F: Fn(&Source, &syn::Type) -> Option<String>>(
    item: &syn::Item,
    source: &Source,
    visitor: &mut Types<'_, F>,
) {
    match item {
        syn::Item::Struct(i) => {
            for field in &i.fields {
                if public(&field.vis) {
                    visitor.visit_type(&field.ty);
                }
            }
        }
        syn::Item::Enum(i) => {
            for variant in &i.variants {
                for field in &variant.fields {
                    visitor.visit_type(&field.ty);
                }
            }
        }
        syn::Item::Type(i) => visitor.visit_type(&i.ty),
        syn::Item::Const(i) => visitor.visit_type(&i.ty),
        syn::Item::Fn(i) => visitor.signature(&i.sig),
        syn::Item::Trait(i) => {
            for bound in &i.supertraits {
                visitor.visit_type_param_bound(bound);
            }
            for member in &i.items {
                if let syn::TraitItem::Fn(method) = member
                    && !source.internal_contract(&method.attrs, method.sig.span())
                {
                    visitor.signature(&method.sig);
                }
            }
        }
        _ => {}
    }
}
