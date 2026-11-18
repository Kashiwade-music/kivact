//! Role: Report mutated types and functions that combine state without a closing marker.

use super::fields::{Counted, Kind, counted};
use super::index::Index;
use super::locals;
use crate::check_source::Source;
use syn::spanned::Spanned;

/// The rule name and the name of its marker.
const RULE: &str = "state";
/// The marker of the replaced rule; every one left is an error.
const OBSOLETE: &str = "// xtask-allow(state-fields):";
/// A marker converted mechanically in W1 and not yet closed by the designer.
const LEGACY: &str = "// xtask-allow(state): legacy:";

/// Run the rule over one source.
pub(crate) fn check(source: &Source, index: &Index, errors: &mut Vec<String>) {
    for item in &source.items {
        match item {
            syn::Item::Struct(item) => check_struct(source, index, item, errors),
            syn::Item::Enum(item) => check_enum(source, index, item, errors),
            _ => {}
        }
    }
    locals::check(source, errors);
    report_markers(source, errors);
}

/// How many `legacy:` markers the lines this source owns hold.
pub(crate) fn legacy_count(source: &Source) -> usize {
    source
        .text
        .lines()
        .enumerate()
        .filter(|(index, line)| {
            source.owns_line(index + 1) && line.trim_start().starts_with(LEGACY)
        })
        .count()
}

fn check_struct(source: &Source, index: &Index, item: &syn::ItemStruct, errors: &mut Vec<String>) {
    let name = item.ident.to_string();
    let fields = counted(&item.fields, "");
    if fields.len() < 2 {
        return;
    }
    if let Some(why) = mutation(source, index, &name, &fields) {
        let what = format!("{name} holds state fields {}", labels(&fields));
        close(source, item.span(), &name, &what, &why, errors);
    }
}

fn check_enum(source: &Source, index: &Index, item: &syn::ItemEnum, errors: &mut Vec<String>) {
    let name = item.ident.to_string();
    let mut fields = Vec::new();
    for variant in &item.variants {
        let variant_fields = counted(&variant.fields, &variant.ident.to_string());
        if variant_fields.len() >= 2 {
            fields.extend(variant_fields);
        }
    }
    if fields.is_empty() {
        return;
    }
    if let Some(why) = mutation(source, index, &name, &fields) {
        let what = format!("{name} has variants with state fields {}", labels(&fields));
        close(source, item.span(), &name, &what, &why, errors);
    }
}

/// Why the type counts as mutated after construction; `None` for a passive record.
fn mutation(source: &Source, index: &Index, name: &str, fields: &[Counted]) -> Option<String> {
    if index.is_mutated(&source.package, name) {
        return Some("a `&mut self` method".to_owned());
    }
    if let Some(field) = fields.iter().find(|field| field.kind == Kind::Interior) {
        return Some(format!("interior-mutable field `{}`", field.label));
    }
    fields
        .iter()
        .find(|field| {
            field.kind == Kind::Handle && index.is_written(&source.package, name, &field.field)
        })
        .map(|field| format!("its methods write `{}`", field.label))
}

fn close(
    source: &Source,
    span: proc_macro2::Span,
    name: &str,
    what: &str,
    why: &str,
    errors: &mut Vec<String>,
) {
    match source.allow_line(RULE, span) {
        Some(line) => source.use_allow(line),
        None => source.error_at(
            RULE,
            span,
            &format!(
                "{what} and is mutated ({why}); close the combinations with an enum, or put one `// xtask-allow(state): <how the combinations are closed>` line before `{name}`"
            ),
            errors,
        ),
    }
}

fn labels(fields: &[Counted]) -> String {
    fields
        .iter()
        .map(|field| field.label.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Report the replaced rule's markers and `state` markers no counted item used.
fn report_markers(source: &Source, errors: &mut Vec<String>) {
    let used = source.used_allows();
    for (index, line) in source.text.lines().enumerate() {
        let number = index + 1;
        if !source.owns_line(number) {
            continue;
        }
        let line = line.trim_start();
        if line.starts_with(OBSOLETE) {
            source.error_at_line(
                RULE,
                number,
                "obsolete `xtask-allow(state-fields)` marker; delete it (the `state` rule takes one marker per type)",
                errors,
            );
        } else if line.starts_with("// xtask-allow(state):") && !used.contains(&number) {
            source.error_at_line(
                RULE,
                number,
                "unused marker: no mutated state combination follows it",
                errors,
            );
        }
    }
}
