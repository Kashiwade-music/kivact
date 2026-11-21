//! Role: Keep kit public names distinct from core and standard prelude names.
//!
//! Kits can be imported alongside kivact::prelude without ambiguous names.
//! A core module's name is not reserved: a module is in the type namespace and does not make a
//! kit's function or value of the same name ambiguous.
//! Only direct children are compared: a glob import brings in a module's direct names, not the names inside its submodules.
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
    if !source.library() {
        return;
    }
    if !crate::check_policy::is_kit(&source.package) {
        return;
    }
    let mut reserved: BTreeSet<&str> = STANDARD.iter().copied().collect();
    for path in surface.exports.keys() {
        if surface.modules.contains(path) {
            continue;
        }
        if let Some(name) = path
            .strip_prefix("kivact::prelude::")
            .or_else(|| path.strip_prefix("kivact::ui::"))
            && !name.contains("::")
        {
            reserved.insert(name);
        }
    }
    let prefix = source.identity_root() + "::";
    for path in surface.exports.keys() {
        let Some(name) = path.strip_prefix(&prefix) else {
            continue;
        };
        if name.contains("::") || !reserved.contains(name) {
            continue;
        }
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
    fn a_core_module_name_is_not_reserved() {
        let mut source = crate::check_source::fixture("pub fn theme() {} pub struct Button;");
        source.module.clear();
        let mut surface = crate::public_surface::build(std::slice::from_ref(&source));
        surface
            .exports
            .insert("kivact::ui::theme".into(), "kivact::ui::theme".into());
        surface.modules.insert("kivact::ui::theme".into());
        surface
            .exports
            .insert("kivact::ui::Button".into(), "kivact_ui::Button".into());
        let mut errors = Vec::new();
        super::check(&source, &surface, &mut errors);
        assert_eq!(errors.len(), 1);
    }
    #[test]
    fn a_nested_core_name_is_not_reserved() {
        let mut source = crate::check_source::fixture(
            "pub const BACKGROUND: u32 = 0; pub struct Button;",
        );
        source.module.clear();
        let mut surface = crate::public_surface::build(std::slice::from_ref(&source));
        surface.exports.insert(
            "kivact::ui::toolbar::BACKGROUND".into(),
            "kivact_ui::toolbar::BACKGROUND".into(),
        );
        surface
            .exports
            .insert("kivact::ui::Button".into(), "kivact_ui::Button".into());
        let mut errors = Vec::new();
        super::check(&source, &surface, &mut errors);
        assert_eq!(errors.len(), 1);
    }
    #[test]
    fn a_kit_nested_name_is_not_checked() {
        let mut source = crate::check_source::fixture("pub mod a { pub struct Button; }");
        source.module.clear();
        let mut surface = crate::public_surface::build(std::slice::from_ref(&source));
        surface
            .exports
            .insert("kivact::ui::Button".into(), "kivact_ui::Button".into());
        let mut errors = Vec::new();
        super::check(&source, &surface, &mut errors);
        assert_eq!(errors.len(), 0);
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
