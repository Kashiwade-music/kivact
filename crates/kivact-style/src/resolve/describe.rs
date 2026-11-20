//! Role: Describe the properties a `Style` sets, for diagnostics.

use crate::sheet::{Decl, PropClass, Style};

/// The properties `s` sets (not its states), the last value of each,
/// by name, with the `Debug` of its value, in the order first set.
pub fn describe_style(s: &Style) -> Vec<(&'static str, String)> {
    let mut out: Vec<(&'static str, String)> = Vec::new();
    for d in &s.decls {
        if matches!(d, Decl::State(..) | Decl::Group) {
            continue;
        }
        let value = d.value_debug();
        match out.iter_mut().find(|(name, _)| *name == d.name()) {
            Some(slot) => slot.1 = value,
            None => out.push((d.name(), value)),
        }
    }
    out
}

/// The properties of each state style `s` sets, by state name.
pub fn describe_state_styles(s: &Style) -> Vec<(&'static str, Vec<(&'static str, String)>)> {
    kivact_types::StyleState::ALL
        .into_iter()
        .filter_map(|state| s.state(state).map(|st| (state.name(), describe_style(&st))))
        .collect()
}

/// The properties set in the host states of `s` that a state cannot
/// change as paint (`CompiledStyle::unsupported_in_states`, until ST-05).
pub(super) fn unsupported_in_states(s: &Style) -> Vec<String> {
    let mut out = Vec::new();
    for state in kivact_types::StyleState::HOST {
        if let Some(st) = s.state(state) {
            out.extend(
                st.decls
                    .iter()
                    .filter(|d| d.class() == PropClass::Rest)
                    .map(|d| format!("{}.{}", state.name(), d.name())),
            );
        }
    }
    out
}
