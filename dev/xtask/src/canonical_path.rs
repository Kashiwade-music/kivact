//! Role: Prevent a public item from acquiring multiple non-prelude paths.
//!
//! One canonical public path keeps imports and documentation predictable;
//! prelude reexports are the deliberate convenience exception.
use crate::check_source::Source;
use crate::public_surface::{Surface, export_span};
use std::collections::BTreeMap;
pub(crate) fn check(source: &Source, surface: &Surface, errors: &mut Vec<String>) {
    let prefix = source.package.replace('-', "_") + "::";
    let mut paths: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (path, target) in &surface.exports {
        if path.starts_with(&prefix) && !path.split("::").any(|p| p == "prelude") {
            paths.entry(target).or_default().push(path);
        }
    }
    for names in paths.values().filter(|paths| paths.len() > 1) {
        let path = names.last().copied().unwrap_or("");
        let span = if let Some((package, module, span)) = surface.origins.get(path) {
            if *package != source.package || *module != source.module {
                continue;
            }
            *span
        } else {
            if !source.module.is_empty() {
                continue;
            }
            export_span(source)
        };
        source.error(
            "canonical-path",
            span,
            &format!("multiple public paths: {}", names.join(", ")),
            errors,
        );
    }
}
#[cfg(test)]
mod tests {
    //! Role: Permit prelude while rejecting a second public export path.
    #[test]
    fn rejects_duplicate_non_prelude_paths() {
        let mut source = crate::check_source::fixture("pub struct Widget;");
        source.module.clear();
        let target = "kivact_ui_demo::Widget".to_owned();
        let mut surface = crate::public_surface::Surface::default();
        for path in ["kivact_ui_demo::Widget", "kivact_ui_demo::prelude::Widget"] {
            surface.exports.insert(path.into(), target.clone());
        }
        let mut errors = Vec::new();
        super::check(&source, &surface, &mut errors);
        assert!(errors.is_empty());
        surface
            .exports
            .insert("kivact_ui_demo::other::Widget".into(), target);
        super::check(&source, &surface, &mut errors);
        assert_eq!(errors.len(), 1);
    }
}
