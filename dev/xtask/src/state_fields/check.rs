//! Role: Find implicit state combinations in structs, enum variants and statics.

use crate::check_policy::strings;
use crate::check_source::Source;
use crate::public_surface::Surface;
use crate::public_surface::definition;
use crate::state_fields::classify::{Class, classify};
use crate::state_fields::locals;
use std::collections::BTreeSet;
use syn::spanned::Spanned;

struct Context<'a> {
    source: &'a Source,
    policy: &'a toml::Table,
    enums: &'a BTreeSet<String>,
    surface: &'a Surface,
}

pub(crate) fn check(
    source: &Source,
    policy: &toml::Table,
    builders: &BTreeSet<String>,
    enums: &BTreeSet<String>,
    surface: &Surface,
    report: bool,
    errors: &mut Vec<String>,
) {
    let context = Context {
        source,
        policy,
        enums,
        surface,
    };
    let skip = is_value_module(source, policy);
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
    report_unused_markers(source, &struct_markers, errors);
    if report {
        errors.extend(locals::collect(source));
    }
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

fn check_struct(
    context: &Context<'_>,
    item: &syn::ItemStruct,
    builders: &BTreeSet<String>,
    struct_markers: &mut BTreeSet<usize>,
    errors: &mut Vec<String>,
) {
    let name = item.ident.to_string();
    if let Some(line) = context.source.marker_line("state-fields", item.span()) {
        struct_markers.insert(line);
        context.source.error_at(
            "state-fields",
            item.span(),
            "struct-level marker; put a marker before each field instead",
            errors,
        );
    }
    if builders.contains(&definition(context.source, &name)) {
        return;
    }
    let exempt = is_exempt(context.policy, &name);
    let (stored, phase) = count_fields(context, &item.fields, exempt, "");
    report_combination(context, item.span(), &name, &stored, phase, errors);
}

fn check_enum(context: &Context<'_>, item: &syn::ItemEnum, errors: &mut Vec<String>) {
    let enum_name = item.ident.to_string();
    let exempt = is_exempt(context.policy, &enum_name);
    for variant in &item.variants {
        if !matches!(variant.fields, syn::Fields::Named(_)) {
            continue;
        }
        let (stored, phase) =
            count_fields(context, &variant.fields, exempt, &variant.ident.to_string());
        let name = format!("{enum_name}::{}", variant.ident);
        report_combination(context, variant.span(), &name, &stored, phase, errors);
    }
}

fn count_fields(
    context: &Context<'_>,
    fields: &syn::Fields,
    exempt: bool,
    prefix: &str,
) -> (Vec<String>, usize) {
    let mut stored = Vec::new();
    let mut phase = 0;
    for (index, field) in fields.iter().enumerate() {
        let marker = context.source.allow_line("state-fields", field.span());
        let ident = field.ident.as_ref().map(ToString::to_string);
        let field_name = ident.as_deref().unwrap_or(prefix);
        let mut counted = false;
        match &field.ty {
            syn::Type::Tuple(tuple) => {
                for (element, ty) in tuple.elems.iter().enumerate() {
                    let label = match &ident {
                        Some(name) => format!("{name}.{element}"),
                        None => format!("{index}.{element}"),
                    };
                    counted |= count_one(
                        context,
                        &mut stored,
                        &mut phase,
                        field_name,
                        ty,
                        exempt,
                        marker.is_some(),
                        &label,
                    );
                }
            }
            ty => {
                let label = ident.clone().unwrap_or_else(|| index.to_string());
                counted = count_one(
                    context,
                    &mut stored,
                    &mut phase,
                    field_name,
                    ty,
                    exempt,
                    marker.is_some(),
                    &label,
                );
            }
        }
        if counted && let Some(line) = marker {
            context.source.use_allow(line);
        }
    }
    (stored, phase)
}

#[allow(
    clippy::too_many_arguments,
    reason = "the classification needs the field identity, its type and the enclosing exemptions"
)]
fn count_one(
    context: &Context<'_>,
    stored: &mut Vec<String>,
    phase: &mut usize,
    name: &str,
    ty: &syn::Type,
    exempt: bool,
    marked: bool,
    label: &str,
) -> bool {
    match classify(
        context.source,
        context.surface,
        name,
        ty,
        exempt,
        context.enums,
        context.policy,
    ) {
        Class::None => false,
        Class::Phase => {
            *phase += 1;
            true
        }
        Class::Stored if !marked => {
            stored.push(label.to_owned());
            true
        }
        Class::Stored => true,
    }
}

fn report_combination(
    context: &Context<'_>,
    span: proc_macro2::Span,
    name: &str,
    stored: &[String],
    phase: usize,
    errors: &mut Vec<String>,
) {
    if stored.len() >= 2 || (phase > 0 && !stored.is_empty()) {
        context.source.error_at(
            "state-fields",
            span,
            &format!("{name} combines state fields: {}", stored.join(", ")),
            errors,
        );
    }
}

fn collect_static(
    context: &Context<'_>,
    item: &syn::ItemStatic,
    statics: &mut Vec<(String, proc_macro2::Span)>,
) {
    let name = item.ident.to_string();
    let class = classify(
        context.source,
        context.surface,
        &name.to_lowercase(),
        &item.ty,
        false,
        context.enums,
        context.policy,
    );
    if class != Class::Stored {
        return;
    }
    match context.source.allow_line("state-fields", item.span()) {
        Some(line) => context.source.use_allow(line),
        None => statics.push((name, item.span())),
    }
}

fn collect_thread_local(
    context: &Context<'_>,
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
    context: &Context<'_>,
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
        "state-fields",
        statics[0].1,
        &format!(
            "{}: static flags/counters combine: {names}",
            context.source.path.display()
        ),
        errors,
    );
}

fn report_unused_markers(
    source: &Source,
    struct_markers: &BTreeSet<usize>,
    errors: &mut Vec<String>,
) {
    let used = source.used_allows();
    let marker = "// xtask-allow(state-fields):";
    for (index, line) in source.text.lines().enumerate() {
        let number = index + 1;
        if !line.trim_start().starts_with(marker)
            || !source.owns_line(number)
            || struct_markers.contains(&number)
            || used.contains(&number)
        {
            continue;
        }
        source.error_at_line("state-fields", number, "unused marker", errors);
    }
}
