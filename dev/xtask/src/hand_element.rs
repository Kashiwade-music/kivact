//! Role: Allow hand-written Element and IntoView impls only in the core view module.
//!
//! The one other place is listed in the policy's `allowed_impls` as
//! "<definition path> <file>", e.g. `kivact_ui::tooltip::Tooltip crates/kivact-ui/src/tooltip.rs`.

use crate::check_policy::strings;
use crate::check_source::{Source, self_type_name};
use crate::public_surface::definition;
use syn::spanned::Spanned;

pub(crate) fn check(source: &Source, policy: &toml::Table, errors: &mut Vec<String>) {
    if source.package == "kivact-core"
        && (source.module == "view" || source.module.starts_with("view::"))
    {
        return;
    }
    let allowed = strings(policy, "allowed_impls");
    let file = source.path.to_string_lossy().replace('\\', "/");
    for item in &source.items {
        let syn::Item::Impl(item) = item else {
            continue;
        };
        let Some((_, path, _)) = &item.trait_ else {
            continue;
        };
        let Some(segment) = path.segments.last() else {
            continue;
        };
        if segment.ident != "Element" && segment.ident != "IntoView" {
            continue;
        }
        let identity = self_type_name(&item.self_ty).map(|name| definition(source, &name));
        let listed = identity.is_some_and(|identity| {
            allowed.iter().any(|entry| {
                entry.split_once(' ').is_some_and(|(path, listed_file)| {
                    path == identity && file.ends_with(listed_file)
                })
            })
        });
        if !listed {
            source.error_at(
                "hand-element",
                item.span(),
                &format!(
                    "hand-written `impl {} for`; write a `#[component]` function (only kivact-core's view module and `[hand_element] allowed_impls` define elements)",
                    segment.ident
                ),
                errors,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    //! Role: Check where element impls may be written by hand.
    fn run(package: &str, module: &str, policy: &str) -> usize {
        let mut source = crate::check_source::fixture(
            "struct Card; impl Element for Card {} impl IntoView for Card {} impl Clone for Card {}",
        );
        source.package = package.into();
        source.module = module.into();
        let policy: toml::Table = policy.parse().expect("the fixture policy parses");
        let mut errors = Vec::new();
        super::check(&source, &policy, &mut errors);
        errors.len()
    }

    #[test]
    fn only_core_view_and_listed_impls_write_element_impls() {
        assert_eq!(run("kivact-ui-soft", "widgets", ""), 2);
        assert_eq!(run("kivact-core", "view::elements", ""), 0);
        assert_eq!(run("kivact-core", "custom", ""), 2);
        let listed = "allowed_impls = [\"kivact_ui_soft::widgets::Card crates/kivact-ui-soft/src/widgets.rs\"]";
        assert_eq!(run("kivact-ui-soft", "widgets", listed), 0);
        let other_file = "allowed_impls = [\"kivact_ui_soft::widgets::Card crates/kivact-ui-soft/src/other.rs\"]";
        assert_eq!(run("kivact-ui-soft", "widgets", other_file), 2);
    }
}
