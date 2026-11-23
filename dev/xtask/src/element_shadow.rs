//! Role: Forbid inherent methods that shadow Element methods on element types.

use crate::check_source::{Source, self_type_name};
use std::collections::BTreeSet;
use syn::spanned::Spanned;

/// The `Element` facts of the workspace.
#[derive(Default)]
pub(crate) struct Elements {
    /// The method names of kivact-core's `Element` trait.
    methods: BTreeSet<String>,
    /// `package::Type` of every type with an `impl Element for` in its package.
    types: BTreeSet<String>,
}

impl Elements {
    pub(crate) fn build(sources: &[Source]) -> Self {
        let mut elements = Self::default();
        for source in sources {
            for item in &source.items {
                match item {
                    syn::Item::Trait(item)
                        if source.package == "kivact-core" && item.ident == "Element" =>
                    {
                        for member in &item.items {
                            if let syn::TraitItem::Fn(method) = member {
                                elements.methods.insert(method.sig.ident.to_string());
                            }
                        }
                    }
                    syn::Item::Impl(item) if implements_element(item) => {
                        if let Some(name) = self_type_name(&item.self_ty) {
                            elements.types.insert(format!("{}::{name}", source.package));
                        }
                    }
                    _ => {}
                }
            }
        }
        elements
    }
}

fn implements_element(item: &syn::ItemImpl) -> bool {
    item.trait_.as_ref().is_some_and(|(_, path, _)| {
        path.segments
            .last()
            .is_some_and(|segment| segment.ident == "Element")
    })
}

pub(crate) fn check(source: &Source, elements: &Elements, errors: &mut Vec<String>) {
    if source.package == "kivact-core" {
        return;
    }
    for item in &source.items {
        let syn::Item::Impl(item) = item else {
            continue;
        };
        if item.trait_.is_some() {
            continue;
        }
        let Some(name) = self_type_name(&item.self_ty) else {
            continue;
        };
        if !elements
            .types
            .contains(&format!("{}::{name}", source.package))
        {
            continue;
        }
        for member in &item.items {
            if let syn::ImplItem::Fn(method) = member
                && elements.methods.contains(&method.sig.ident.to_string())
            {
                source.error_at(
                    "element-shadow",
                    method.sig.span(),
                    &format!(
                        "`{name}::{}` shadows the `Element` method of the same name; delete it and use the `Element` method",
                        method.sig.ident
                    ),
                    errors,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Role: Check that element types do not shadow `Element` methods.
    #[test]
    fn inherent_methods_may_not_shadow_element_methods() {
        let mut core = crate::check_source::fixture(
            "pub trait Element { fn disabled(self) -> Self; fn style(self) -> Self; }",
        );
        core.package = "kivact-core".into();
        let ui = crate::check_source::fixture(
            "struct Card; impl Element for Card {} impl Card { pub fn disabled(self) -> Self { self } pub fn title(self) -> Self { self } }",
        );
        let sources = [core, ui];
        let elements = super::Elements::build(&sources);
        let mut errors = Vec::new();
        super::check(&sources[1], &elements, &mut errors);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("Card::disabled"), "{errors:?}");
    }
}
