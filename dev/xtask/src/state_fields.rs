//! Role: Detect combinations of state fields that hide a state machine.
//!
//! Two bool/Option/counter fields, or an Option beside a State/Phase enum,
//! require an explicit state model. Role suffixes, records, value modules,
//! NodeBuilder implementations and Api option builders describe data instead.

use crate::check_source::{Source, type_names};
use std::collections::BTreeSet;
use syn::spanned::Spanned;

pub(crate) fn check(
    source: &Source,
    policy: &toml::Table,
    builders: &BTreeSet<String>,
    enums: &BTreeSet<String>,
    surface: &crate::public_surface::Surface,
    errors: &mut Vec<String>,
) {
    let qualified = format!(
        "{}::{}",
        if source.library {
            source.package.clone()
        } else {
            source.identity_root()
        },
        source.module
    );
    if crate::check_policy::strings(policy, "value_modules")
        .iter()
        .any(|v| qualified == *v || qualified.starts_with(&format!("{v}::")))
    {
        return;
    }
    for item in &source.items {
        let syn::Item::Struct(item) = item else {
            continue;
        };
        let name = item.ident.to_string();
        if name.ends_with("Api")
            || builders.contains(&crate::public_surface::definition(source, &name))
            || crate::check_policy::strings(policy, "role_suffixes")
                .iter()
                .any(|suffix| name.ends_with(suffix))
            || crate::check_policy::strings(policy, "record_types").contains(&name)
        {
            continue;
        }
        let mut count = 0;
        let mut option = false;
        let mut phase = false;
        for field in &item.fields {
            if source.allows("state-fields", field.span()) {
                continue;
            }
            let names = type_names(&field.ty);
            let has_option = names.contains("Option");
            option |= has_option;
            phase |= !has_option
                && surface
                    .resolve_type(source, &field.ty)
                    .is_some_and(|identity| {
                        (identity.ends_with("State") || identity.ends_with("Phase"))
                            && enums.contains(&identity)
                    });
            let counter = names
                .iter()
                .any(|n| matches!(n.as_str(), "Cell" | "Signal" | "Var"))
                && names
                    .iter()
                    .any(|n| matches!(n.as_str(), "u64" | "u32" | "usize"));
            if has_option || names.contains("bool") || names.contains("AtomicBool") || counter {
                count += 1;
            }
        }
        if count >= 2 || (option && phase) {
            source.error(
                "state-fields",
                item.span(),
                &format!("{name} combines state fields"),
                errors,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    //! Role: Distinguish state combinations from independent facts and role data.
    use super::check;
    #[test]
    fn detects_wrapped_fields_and_inline_exceptions() {
        let source = crate::check_source::fixture(
            "struct Hidden { open: Cell<bool>, pending: Option<Job> }\nstruct Fact { open: bool } struct Single { state: Option<LoadPhase> }\n// xtask-allow(state-fields): independent facts\nstruct Facts { a: bool, b: bool }\nstruct Mixed { pending: Option<Job>, phase: LoadPhase }",
        );
        let mut errors = Vec::new();
        check(
            &source,
            &toml::Table::new(),
            &Default::default(),
            &["kivact_ui_soft::widgets::LoadPhase".into()].into(),
            &crate::public_surface::Surface::default(),
            &mut errors,
        );
        assert_eq!(errors.len(), 2);
    }
    #[test]
    fn exempts_roles_and_builder_implementations() {
        let source = crate::check_source::fixture(
            "struct ButtonOptions { a: bool, b: bool } struct Widget { a: bool, b: bool } struct ButtonApi { a: bool, b: bool }",
        );
        let policy: toml::Table = "role_suffixes = ['Options']".parse().unwrap();
        let mut errors = Vec::new();
        check(
            &source,
            &policy,
            &["kivact_ui_soft::widgets::Widget".into()].into(),
            &Default::default(),
            &crate::public_surface::Surface::default(),
            &mut errors,
        );
        assert!(errors.is_empty());
    }
    #[test]
    fn ordinary_integers_are_not_mutable_counters() {
        let source = crate::check_source::fixture(
            "struct Size { width: u32, height: usize } struct Counter { n: Cell<u64>, pending: Option<Job> }",
        );
        let mut errors = Vec::new();
        super::check(
            &source,
            &toml::Table::new(),
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &mut errors,
        );
        assert_eq!(errors.len(), 1);
    }
    #[test]
    fn enum_and_builder_identities_do_not_leak_across_modules() {
        let source = crate::check_source::fixture(
            "struct LoadState; struct Model { pending: Option<Job>, phase: LoadState } struct Widget { a: bool, b: bool }",
        );
        let mut errors = Vec::new();
        super::check(
            &source,
            &toml::Table::new(),
            &["kivact_ui_soft::other::Widget".into()].into(),
            &["kivact_ui_soft::other::LoadState".into()].into(),
            &Default::default(),
            &mut errors,
        );
        assert_eq!(errors.len(), 1);
    }
}
