//! Role: Keep input-driven Machines and signal-writing effects out of kivact-ui.
//!
//! The policy's `allowed_types` lists the presentation timers that stay in
//! kivact-ui (button busy reveal, progress motion, alert collapse, empty
//! state changes) by definition path.

use crate::check_policy::strings;
use crate::check_source::{Source, self_type_name};
use crate::public_surface::definition;
use syn::spanned::Spanned;

pub(crate) fn check(source: &Source, policy: &toml::Table, errors: &mut Vec<String>) {
    if source.package != "kivact-ui" {
        return;
    }
    let allowed = strings(policy, "allowed_types");
    for item in &source.items {
        let syn::Item::Impl(item) = item else {
            continue;
        };
        let machine = item.trait_.as_ref().is_some_and(|(_, path, _)| {
            path.segments
                .last()
                .is_some_and(|segment| segment.ident == "Machine")
        });
        let Some(name) = self_type_name(&item.self_ty) else {
            continue;
        };
        if machine && !allowed.contains(&definition(source, &name)) {
            source.error_at(
                "ui-machine",
                item.span(),
                &format!(
                    "kivact-ui implements `Machine` for `{name}`; input-driven behavior belongs in kivact-interaction as a `use_*` Api"
                ),
                errors,
            );
        }
    }
    crate::effect_writes::check_strict(source, "ui-machine", errors);
}

#[cfg(test)]
mod tests {
    //! Role: Check Machines and effect writes in kivact-ui.
    fn run(policy: &str) -> Vec<String> {
        let mut source = crate::check_source::fixture(
            "struct Drag; impl Machine for Drag { type State = u8; }\nfn setup(open: Signal<bool>) { effect(move || open.set(true)); }",
        );
        source.package = "kivact-ui".into();
        let policy: toml::Table = policy.parse().expect("the fixture policy parses");
        let mut errors = Vec::new();
        super::check(&source, &policy, &mut errors);
        errors
    }

    #[test]
    fn ui_has_no_machines_and_no_effect_writes() {
        assert_eq!(run("").len(), 2);
        assert_eq!(
            run("allowed_types = [\"kivact_ui::widgets::Drag\"]").len(),
            1
        );
    }
}
