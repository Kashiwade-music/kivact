//! Role: Keep kit public names distinct from core and standard prelude names.
//!
//! Kits can be imported alongside kivact::prelude without ambiguous names.
use crate::check_source::Source;
use crate::public_surface::{Surface, export_span};
use std::collections::BTreeSet;
const STANDARD: &[&str] = &[
    "AsRef",
    "AsMut",
    "Clone",
    "Copy",
    "Default",
    "Drop",
    "Eq",
    "PartialEq",
    "Ord",
    "PartialOrd",
    "Sized",
    "Send",
    "Sync",
    "Unpin",
    "Fn",
    "FnMut",
    "FnOnce",
    "Into",
    "From",
    "TryFrom",
    "TryInto",
    "IntoIterator",
    "Iterator",
    "DoubleEndedIterator",
    "ExactSizeIterator",
    "Extend",
    "FromIterator",
    "ToOwned",
    "ToString",
    "Box",
    "String",
    "Vec",
    "Option",
    "Some",
    "None",
    "Result",
    "Ok",
    "Err",
    "Future",
    "IntoFuture",
    "AsyncFn",
    "AsyncFnMut",
    "AsyncFnOnce",
];
pub(crate) fn check(source: &Source, surface: &Surface, errors: &mut Vec<String>) {
    if !crate::foundation::is_kit(&source.package) {
        return;
    }
    let mut reserved: BTreeSet<&str> = STANDARD.iter().copied().collect();
    for path in surface.exports.keys() {
        if path.starts_with("kivact::prelude::") || path.starts_with("kivact::ui::") {
            if let Some(name) = path.rsplit("::").next() {
                reserved.insert(name);
            }
        }
    }
    let prefix = source.package.replace('-', "_") + "::";
    for path in surface
        .exports
        .keys()
        .filter(|path| path.starts_with(&prefix))
    {
        let name = path.rsplit("::").next().unwrap_or("");
        if reserved.contains(name) {
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
                "name-collision",
                span,
                &format!("kit export {path} collides with core or std prelude"),
                errors,
            );
        }
    }
}
#[cfg(test)]
mod tests {
    //! Role: Check standard and facade names with harmless kit names accepted.
    #[test]
    fn detects_std_and_core_collisions() {
        let mut source = crate::check_source::fixture(
            "pub struct String; pub struct Button; pub struct OwnWidget;",
        );
        source.module.clear();
        let mut surface = crate::public_surface::build(std::slice::from_ref(&source));
        surface
            .exports
            .insert("kivact::ui::Button".into(), "kivact_ui::Button".into());
        let mut errors = Vec::new();
        super::check(&source, &surface, &mut errors);
        assert_eq!(errors.len(), 2);
    }
    #[test]
    fn includes_module_names_and_respects_inline_markers() {
        let mut source = crate::check_source::fixture(
            "pub mod String {}
// xtask-allow(name-collision): deliberate type bridge
pub struct Vec;",
        );
        source.module.clear();
        let surface = crate::public_surface::build(std::slice::from_ref(&source));
        let mut errors = Vec::new();
        super::check(&source, &surface, &mut errors);
        assert_eq!(errors.len(), 1);
    }
}
