//! Role: Find implicit state combinations in structs, enum variants and statics.

use crate::check_policy::strings;
use crate::check_source::Source;
use crate::public_surface::definition;
use crate::state_fields::classify::{Class, RuleContext, classify};
use crate::state_fields::locals;
use crate::state_fields::{Mode, Strictness};
use std::collections::BTreeSet;
use syn::spanned::Spanned;

/// The marker that explains a field, static or local.
const MARKER: &str = "state-fields";

/// The openings of a reason that closes a state (design 6.2, SL-8).
const REASON_PREFIXES: [&str; 6] = [
    "independent:",
    "projection:",
    "cache:",
    "identity:",
    "statistic:",
    "option:",
];

pub(crate) fn check(
    source: &Source,
    policy: &toml::Table,
    builders: &BTreeSet<String>,
    enums: &BTreeSet<String>,
    surface: &crate::public_surface::Surface,
    mode: Mode,
    errors: &mut Vec<String>,
) {
    let context = RuleContext {
        source,
        surface,
        enums,
        policy,
        strictness: mode.strictness,
    };
    let skip = is_value_module(source, policy, mode.strictness);
    let mut struct_markers = BTreeSet::new();
    let mut statics = Vec::new();
    if !skip {
        for item in &source.items {
            match item {
                syn::Item::Struct(item) => {
                    check_struct(&context, item, builders, &mut struct_markers, errors)
                }
                syn::Item::Enum(item) => check_enum(&context, item, errors),
                syn::Item::Static(item) => collect_static(&context, item, &mut statics),
                syn::Item::Macro(item) => collect_thread_local(&context, item, &mut statics),
                _ => {}
            }
        }
    }
    report_statics(&context, &statics, errors);
    let functions = locals::scan(source);
    if mode.strictness == Strictness::Strict {
        locals::check(source, &functions, errors);
    } else if mode.report {
        errors.extend(locals::collect(source, &functions));
    }
    report_unused_markers(source, mode.strictness, &struct_markers, errors);
}

fn is_value_module(source: &Source, policy: &toml::Table, strictness: Strictness) -> bool {
    let root = if source.library {
        source.package.clone()
    } else {
        source.identity_root()
    };
    let qualified = format!("{root}::{}", source.module);
    strings(policy, "value_modules")
        .iter()
        .filter(|module| {
            strictness != Strictness::Strict || module.as_str() != "kivact-engine::contract"
        })
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
fn struct_exempt(
    context: &RuleContext<'_>,
    name: &str,
    fields: &syn::Fields,
    strictness: Strictness,
) -> bool {
    is_exempt(context.policy, name)
        && !(strictness == Strictness::Strict && has_mutable_cell(fields))
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
    errors: &mut Vec<String>,
) {
    let name = item.ident.to_string();
    let rule = context.strictness.rule();
    if let Some(line) = context.source.marker_line(MARKER, item.span()) {
        struct_markers.insert(line);
        context.source.error_at(
            rule,
            item.span(),
            "struct-level marker; put a marker before each field instead",
            errors,
        );
    }
    if builders.contains(&definition(context.source, &name)) {
        return;
    }
    let exempt = struct_exempt(context, &name, &item.fields, context.strictness);
    let (stored, phase) = count_fields(context, &item.fields, exempt, "", context.strictness);
    report_combination(context, item.span(), &name, &stored, phase, errors);
    let other = opposite(context.strictness);
    let other_exempt = struct_exempt(context, &name, &item.fields, other);
    let _ = count_fields(context, &item.fields, other_exempt, "", other);
}

fn check_enum(context: &RuleContext<'_>, item: &syn::ItemEnum, errors: &mut Vec<String>) {
    let enum_name = item.ident.to_string();
    let exempt = is_exempt(context.policy, &enum_name);
    let other = opposite(context.strictness);
    for variant in &item.variants {
        if !matches!(variant.fields, syn::Fields::Named(_)) {
            continue;
        }
        let prefix = variant.ident.to_string();
        let (stored, phase) = count_fields(
            context,
            &variant.fields,
            exempt,
            &prefix,
            context.strictness,
        );
        let name = format!("{enum_name}::{}", variant.ident);
        report_combination(context, variant.span(), &name, &stored, phase, errors);
        let _ = count_fields(context, &variant.fields, exempt, &prefix, other);
    }
}

fn opposite(strictness: Strictness) -> Strictness {
    match strictness {
        Strictness::Current => Strictness::Strict,
        Strictness::Strict => Strictness::Current,
    }
}

/// One field or tuple element: its report label, class, and marker.
type Entry = (String, Class, bool, Option<usize>);

fn count_fields(
    context: &RuleContext<'_>,
    fields: &syn::Fields,
    exempt: bool,
    prefix: &str,
    strictness: Strictness,
) -> (Vec<String>, usize) {
    let rules = RuleContext {
        strictness,
        ..*context
    };
    let mut entries: Vec<Entry> = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let marker = context.source.allow_line(MARKER, field.span());
        let ident = field.ident.as_ref().map(ToString::to_string);
        let field_name = ident.as_deref().unwrap_or(prefix);
        match &field.ty {
            syn::Type::Tuple(tuple) => {
                for (element, ty) in tuple.elems.iter().enumerate() {
                    let label = match &ident {
                        Some(name) => format!("{name}.{element}"),
                        None => format!("{index}.{element}"),
                    };
                    let class = classify(&rules, field_name, ty, exempt);
                    entries.push((label, class, marker.is_some(), marker));
                }
            }
            ty => {
                let label = ident.clone().unwrap_or_else(|| index.to_string());
                let class = classify(&rules, field_name, ty, exempt);
                entries.push((label, class, marker.is_some(), marker));
            }
        }
    }
    let promote = strictness == Strictness::Strict
        && entries
            .iter()
            .any(|(_, class, _, _)| *class == Class::Stored);
    let mut stored = Vec::new();
    let mut phase = 0;
    for (label, class, marked, marker) in &entries {
        let counted = match class {
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
        if counted && let Some(line) = marker {
            context.source.use_allow(*line);
        }
    }
    (stored, phase)
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
            context.strictness.rule(),
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
        return;
    }
    // A marker also explains the static when only the other rule set counts it.
    if let Some(line) = marker {
        let rules = RuleContext {
            strictness: opposite(context.strictness),
            ..*context
        };
        if classify(&rules, &name.to_lowercase(), &item.ty, false) == Class::Stored {
            context.source.use_allow(line);
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
        context.strictness.rule(),
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
    strictness: Strictness,
    struct_markers: &BTreeSet<usize>,
    errors: &mut Vec<String>,
) {
    let used = source.used_allows();
    let rule = strictness.rule();
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
            source.error_at_line(rule, number, "unused marker", errors);
        } else if strictness == Strictness::Strict && !has_known_reason(reason) {
            source.error_at_line(
                rule,
                number,
                "marker reason must start with independent:, projection:, cache:, identity:, statistic: or option:",
                errors,
            );
        }
    }
}
