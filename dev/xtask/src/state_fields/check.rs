//! Role: Find implicit state combinations in structs, enum variants and statics.

use super::audit::{AuditRow, marker_closure};
use super::classify::{Class, RuleContext, classify};
use super::locals;
use super::mode::Mode;
use crate::check_policy::strings;
use crate::check_source::Source;
use crate::public_surface::definition;
use std::collections::BTreeSet;
use syn::spanned::Spanned;

/// The marker that explains a field, static or local.
const MARKER: &str = "state-fields";

/// The rule name reported by this check.
const RULE: &str = "state-fields";

/// The openings of a reason that closes a state (design 6.2, SL-8).
const REASON_PREFIXES: [&str; 6] = [
    "independent:",
    "projection:",
    "cache:",
    "identity:",
    "statistic:",
    "option:",
];

/// Run the rule over one source, returning the audit rows of counted state.
pub(crate) fn check(
    source: &Source,
    policy: &toml::Table,
    builders: &BTreeSet<String>,
    enums: &BTreeSet<String>,
    surface: &crate::public_surface::Surface,
    mode: Mode,
    errors: &mut Vec<String>,
) -> Vec<AuditRow> {
    let context = RuleContext {
        source,
        surface,
        enums,
        policy,
    };
    let skip = is_value_module(source, policy);
    let mut struct_markers = BTreeSet::new();
    let mut statics = Vec::new();
    let mut rows = Vec::new();
    if !skip {
        for item in &source.items {
            match item {
                syn::Item::Struct(item) => check_struct(
                    &context,
                    item,
                    builders,
                    &mut struct_markers,
                    mode,
                    &mut rows,
                    errors,
                ),
                syn::Item::Enum(item) => check_enum(&context, item, mode, &mut rows, errors),
                syn::Item::Static(item) => collect_static(&context, item, &mut statics),
                syn::Item::Macro(item) => collect_thread_local(&context, item, &mut statics),
                _ => {}
            }
        }
    }
    report_statics(&context, &statics, errors);
    let functions = locals::scan(source);
    if mode.audit {
        rows.extend(locals::audit(source, &functions));
    }
    locals::check(source, &functions, errors);
    if mode.report {
        errors.extend(locals::collect(source, &functions));
    }
    report_unused_markers(source, &struct_markers, errors);
    rows
}

fn is_value_module(source: &Source, policy: &toml::Table) -> bool {
    let root = if source.library {
        source.package.clone()
    } else {
        source.identity_root()
    };
    let qualified = format!("{root}::{}", source.module);
    strings(policy, "value_modules")
        .iter()
        .any(|module| qualified == *module || qualified.starts_with(&format!("{module}::")))
}

fn is_exempt(policy: &toml::Table, name: &str) -> bool {
    name.ends_with("Api")
        || strings(policy, "role_suffixes")
            .iter()
            .any(|suffix| name.ends_with(suffix.as_str()))
        || strings(policy, "record_types")
            .iter()
            .any(|record| record == name)
}

/// Whether a data-shaped name still hides state behind a mutable cell (SL-4).
fn struct_exempt(context: &RuleContext<'_>, name: &str, fields: &syn::Fields) -> bool {
    is_exempt(context.policy, name) && !has_mutable_cell(fields)
}

fn has_mutable_cell(fields: &syn::Fields) -> bool {
    fields.iter().any(|field| {
        let names = crate::check_source::type_names(&field.ty);
        ["Cell", "RefCell", "Signal", "Var", "Mutex", "RwLock"]
            .iter()
            .any(|name| names.contains(*name))
    })
}

fn check_struct(
    context: &RuleContext<'_>,
    item: &syn::ItemStruct,
    builders: &BTreeSet<String>,
    struct_markers: &mut BTreeSet<usize>,
    mode: Mode,
    rows: &mut Vec<AuditRow>,
    errors: &mut Vec<String>,
) {
    let name = item.ident.to_string();
    if let Some(line) = context.source.marker_line(MARKER, item.span()) {
        struct_markers.insert(line);
        context.source.error_at(
            RULE,
            item.span(),
            "struct-level marker; put a marker before each field instead",
            errors,
        );
    }
    if builders.contains(&definition(context.source, &name)) {
        return;
    }
    let exempt = struct_exempt(context, &name, &item.fields);
    let counting = count_fields(context, &item.fields, exempt, "", mode.audit);
    report_combination(
        context,
        item.span(),
        &name,
        &counting.stored,
        counting.phase,
        errors,
    );
    rows.extend(counting.rows(context.source, &name));
}

fn check_enum(
    context: &RuleContext<'_>,
    item: &syn::ItemEnum,
    mode: Mode,
    rows: &mut Vec<AuditRow>,
    errors: &mut Vec<String>,
) {
    let enum_name = item.ident.to_string();
    let exempt = is_exempt(context.policy, &enum_name);
    for variant in &item.variants {
        if !matches!(variant.fields, syn::Fields::Named(_)) {
            continue;
        }
        let prefix = variant.ident.to_string();
        let counting = count_fields(context, &variant.fields, exempt, &prefix, mode.audit);
        let name = format!("{enum_name}::{}", variant.ident);
        report_combination(
            context,
            variant.span(),
            &name,
            &counting.stored,
            counting.phase,
            errors,
        );
        rows.extend(counting.rows(context.source, &name));
    }
}

/// One field or tuple element: its report label, class, and marker.
type Entry = (String, Class, bool, Option<usize>, usize);

/// A counted field with everything the audit records about it.
struct Counted {
    label: String,
    line: usize,
    marker: Option<usize>,
    class: Class,
}

/// The counted state of one struct, variant or static group.
struct Counting {
    stored: Vec<String>,
    phase: usize,
    counted: Vec<Counted>,
}

impl Counting {
    fn rows(&self, source: &Source, owner: &str) -> Vec<AuditRow> {
        if self.counted.len() < 2 {
            return Vec::new();
        }
        self.counted
            .iter()
            .map(|field| {
                let (reason, closure) = match field.marker {
                    Some(line) => {
                        let reason = source.allow_reason(MARKER, line).unwrap_or_default();
                        let closure = marker_closure(&reason);
                        (reason, closure)
                    }
                    None if field.class == Class::Phase => ("-".to_owned(), "type".to_owned()),
                    None => ("-".to_owned(), "OPEN".to_owned()),
                };
                AuditRow {
                    path: source.path.clone(),
                    line: field.line,
                    field: format!("{owner}.{}", field.label),
                    reason,
                    closure,
                }
            })
            .collect()
    }
}

fn count_fields(
    context: &RuleContext<'_>,
    fields: &syn::Fields,
    exempt: bool,
    prefix: &str,
    audit: bool,
) -> Counting {
    let mut entries: Vec<Entry> = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let marker = context.source.allow_line(MARKER, field.span());
        let line = field.span().start().line;
        let ident = field.ident.as_ref().map(ToString::to_string);
        let field_name = ident.as_deref().unwrap_or(prefix);
        match &field.ty {
            syn::Type::Tuple(tuple) => {
                for (element, ty) in tuple.elems.iter().enumerate() {
                    let label = match &ident {
                        Some(name) => format!("{name}.{element}"),
                        None => format!("{index}.{element}"),
                    };
                    let class = classify(context, field_name, ty, exempt);
                    entries.push((label, class, marker.is_some(), marker, line));
                }
            }
            ty => {
                let label = ident.clone().unwrap_or_else(|| index.to_string());
                let class = classify(context, field_name, ty, exempt);
                entries.push((label, class, marker.is_some(), marker, line));
            }
        }
    }
    let promote = entries
        .iter()
        .any(|(_, class, _, _, _)| *class == Class::Stored);
    let mut stored = Vec::new();
    let mut phase = 0;
    let mut counted = Vec::new();
    for (label, class, marked, marker, line) in entries {
        let counted_field = match class {
            Class::None => false,
            Class::Phase => {
                phase += 1;
                true
            }
            Class::Stored => {
                if !marked {
                    stored.push(label.clone());
                }
                true
            }
            Class::Helper if promote => {
                if !marked {
                    stored.push(label.clone());
                }
                true
            }
            Class::Helper => false,
        };
        if counted_field {
            if let Some(line) = marker {
                context.source.use_allow(line);
            }
            if audit {
                counted.push(Counted {
                    label,
                    line,
                    marker,
                    class,
                });
            }
        }
    }
    Counting {
        stored,
        phase,
        counted,
    }
}

fn report_combination(
    context: &RuleContext<'_>,
    span: proc_macro2::Span,
    name: &str,
    stored: &[String],
    phase: usize,
    errors: &mut Vec<String>,
) {
    if stored.len() >= 2 || (phase > 0 && !stored.is_empty()) {
        context.source.error_at(
            RULE,
            span,
            &format!("{name} combines state fields: {}", stored.join(", ")),
            errors,
        );
    }
}

fn collect_static(
    context: &RuleContext<'_>,
    item: &syn::ItemStatic,
    statics: &mut Vec<(String, proc_macro2::Span)>,
) {
    let name = item.ident.to_string();
    let class = classify(context, &name.to_lowercase(), &item.ty, false);
    let marker = context.source.allow_line(MARKER, item.span());
    if class == Class::Stored {
        match marker {
            Some(line) => context.source.use_allow(line),
            None => statics.push((name, item.span())),
        }
    }
}

fn collect_thread_local(
    context: &RuleContext<'_>,
    item: &syn::ItemMacro,
    statics: &mut Vec<(String, proc_macro2::Span)>,
) {
    if !item
        .mac
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "thread_local")
    {
        return;
    }
    let Ok(file) = syn::parse2::<syn::File>(item.mac.tokens.clone()) else {
        return;
    };
    for item in &file.items {
        if let syn::Item::Static(item) = item {
            collect_static(context, item, statics);
        }
    }
}

fn report_statics(
    context: &RuleContext<'_>,
    statics: &[(String, proc_macro2::Span)],
    errors: &mut Vec<String>,
) {
    if statics.len() < 2 {
        return;
    }
    let names = statics
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    context.source.error_at(
        RULE,
        statics[0].1,
        &format!(
            "{}: static flags/counters combine: {names}",
            context.source.path.display()
        ),
        errors,
    );
}

/// Whether `reason` starts with a closure that design 6.2 allows (SL-8).
fn has_known_reason(reason: &str) -> bool {
    let reason = reason.trim();
    REASON_PREFIXES
        .iter()
        .any(|prefix| reason.starts_with(prefix))
}

fn report_unused_markers(
    source: &Source,
    struct_markers: &BTreeSet<usize>,
    errors: &mut Vec<String>,
) {
    let used = source.used_allows();
    let marker = "// xtask-allow(state-fields):";
    for (index, line) in source.text.lines().enumerate() {
        let Some(reason) = line.trim_start().strip_prefix(marker) else {
            continue;
        };
        let number = index + 1;
        if !source.owns_line(number) || struct_markers.contains(&number) {
            continue;
        }
        if !used.contains(&number) {
            source.error_at_line(RULE, number, "unused marker", errors);
        } else if !has_known_reason(reason) {
            source.error_at_line(
                RULE,
                number,
                "marker reason must start with independent:, projection:, cache:, identity:, statistic: or option:",
                errors,
            );
        }
    }
}
