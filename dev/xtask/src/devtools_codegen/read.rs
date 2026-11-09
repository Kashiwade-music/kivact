//! Role: Read the protocol crate's Rust source into the generator's model with syn.

use std::path::Path;

use crate::devtools_codegen::model::{
    ErrorCode, Field, Item, MethodDecl, NotificationDecl, Protocol, Reference, Ty,
};

/// Files of the protocol crate that hold no wire types (design 4.2.3).
const SKIPPED_FILES: [&str; 7] = [
    "lib.rs",
    "rpc.rs",
    "time.rs",
    "review_markdown.rs",
    "discovery.rs",
    "trace.rs",
    "schema.rs",
];

/// Read every wire type, method, notification and error code below `root`.
pub(super) fn read_protocol(root: &Path) -> Result<Protocol, String> {
    let directory = root
        .join("crates")
        .join("kivact-devtools-protocol")
        .join("src");
    let mut names = Vec::new();
    let entries = std::fs::read_dir(&directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", directory.display()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".rs") && !SKIPPED_FILES.contains(&name.as_str()) {
            names.push(name);
        }
    }
    names.sort();
    let mut protocol = Protocol::default();
    for name in names {
        let path = directory.join(&name);
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        read_source(
            &format!("crates/kivact-devtools-protocol/src/{name}"),
            &text,
            &mut protocol,
        )?;
    }
    if protocol.version == 0 {
        return Err(
            "crates/kivact-devtools-protocol/src/app.rs: PROTOCOL_VERSION is missing".into(),
        );
    }
    check_references(&protocol)?;
    Ok(protocol)
}

/// Every type name that the protocol names must be a type of the protocol (design 4.2.3).
pub(super) fn check_references(protocol: &Protocol) -> Result<(), String> {
    for reference in &protocol.references {
        let known = protocol
            .items
            .iter()
            .any(|item| item.name() == reference.name);
        if !known {
            return Err(format!(
                "{}: {}: unknown type {} (design 4.2.3)",
                reference.file, reference.context, reference.name
            ));
        }
    }
    Ok(())
}

/// Read one file of Rust source into `out`.
pub(super) fn read_source(file: &str, text: &str, out: &mut Protocol) -> Result<(), String> {
    let parsed = syn::parse_file(text).map_err(|error| format!("{file}: {error}"))?;
    for item in &parsed.items {
        match item {
            syn::Item::Struct(item) => read_struct(file, item, out)?,
            syn::Item::Enum(item) => read_enum(file, item, out)?,
            syn::Item::Type(item) => read_alias(file, item, out)?,
            syn::Item::Const(item) => read_const(file, item, out)?,
            syn::Item::Macro(item) => read_macro(file, item, out)?,
            _ => {}
        }
    }
    Ok(())
}

fn read_struct(file: &str, item: &syn::ItemStruct, out: &mut Protocol) -> Result<(), String> {
    let name = item.ident.to_string();
    let syn::Fields::Named(fields) = &item.fields else {
        return Err(format!(
            "{file}: {name}: a tuple or unit struct is not a wire type; use named fields (design 4.2.3)"
        ));
    };
    expect_serde(file, &name, &item.attrs, &[])?;
    let config = name == "DevtoolsConfig";
    let mut read = Vec::new();
    for field in &fields.named {
        if let Some(field) = read_field(file, &name, field, config, out)? {
            read.push(field);
        }
    }
    out.items.push(Item::Record {
        name,
        doc: doc_text(&item.attrs),
        fields: read,
    });
    Ok(())
}

fn read_field(
    file: &str,
    owner: &str,
    field: &syn::Field,
    config: bool,
    out: &mut Protocol,
) -> Result<Option<Field>, String> {
    let name = field
        .ident
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_default();
    let context = format!("{owner}.{name}");
    let ty = convert_type(file, &context, &field.ty)?;
    note_references(file, &context, &ty, out);
    let schema_key = config && name == "schema";
    let expected: Vec<&str> = if schema_key {
        vec![
            "default",
            "rename = \"$schema\"",
            "skip_serializing_if = \"Option::is_none\"",
        ]
    } else if matches!(&ty, Ty::Option(_)) {
        vec!["default", "skip_serializing_if = \"Option::is_none\""]
    } else if matches!(&ty, Ty::Vec(_)) {
        vec!["default", "skip_serializing_if = \"Vec::is_empty\""]
    } else {
        Vec::new()
    };
    expect_serde(file, &context, &field.attrs, &expected)?;
    if schema_key {
        out.config_schema_key = true;
        return Ok(None);
    }
    Ok(Some(Field {
        name,
        ty,
        doc: doc_text(&field.attrs),
    }))
}

fn read_enum(file: &str, item: &syn::ItemEnum, out: &mut Protocol) -> Result<(), String> {
    let name = item.ident.to_string();
    let entries = serde_entries(&item.attrs).map_err(|error| format!("{file}: {name}: {error}"))?;
    if entries.iter().any(|entry| entry == "tag = \"kind\"") {
        expect_serde(
            file,
            &name,
            &item.attrs,
            &["rename_all = \"snake_case\"", "tag = \"kind\""],
        )?;
        let mut variants = Vec::new();
        for variant in &item.variants {
            let variant_name = variant.ident.to_string();
            let context = format!("{name}.{variant_name}");
            expect_serde(file, &context, &variant.attrs, &[])?;
            let fields = match &variant.fields {
                syn::Fields::Named(fields) => {
                    let mut read = Vec::new();
                    for field in &fields.named {
                        if let Some(field) = read_field(file, &context, field, false, out)? {
                            read.push(field);
                        }
                    }
                    read
                }
                syn::Fields::Unit => Vec::new(),
                syn::Fields::Unnamed(_) => {
                    return Err(format!(
                        "{file}: {context}: unsupported fields in a tagged union (design 4.2.3)"
                    ));
                }
            };
            variants.push((variant_name, doc_text(&variant.attrs), fields));
        }
        out.items.push(Item::Union {
            name,
            doc: doc_text(&item.attrs),
            variants,
        });
    } else {
        expect_serde(file, &name, &item.attrs, &["rename_all = \"snake_case\""])?;
        let mut variants = Vec::new();
        for variant in &item.variants {
            let variant_name = variant.ident.to_string();
            if !matches!(variant.fields, syn::Fields::Unit) {
                return Err(format!(
                    "{file}: {name}.{variant_name}: unsupported fields in a choice (design 4.2.3)"
                ));
            }
            expect_serde(file, &format!("{name}.{variant_name}"), &variant.attrs, &[])?;
            variants.push((variant_name, doc_text(&variant.attrs)));
        }
        out.items.push(Item::Choice {
            name,
            doc: doc_text(&item.attrs),
            variants,
        });
    }
    Ok(())
}

fn read_alias(file: &str, item: &syn::ItemType, out: &mut Protocol) -> Result<(), String> {
    let name = item.ident.to_string();
    let ty = convert_type(file, &name, &item.ty)?;
    note_references(file, &name, &ty, out);
    out.items.push(Item::Alias {
        name,
        doc: doc_text(&item.attrs),
        ty,
    });
    Ok(())
}

fn read_const(file: &str, item: &syn::ItemConst, out: &mut Protocol) -> Result<(), String> {
    let name = item.ident.to_string();
    if name == "PROTOCOL_VERSION" {
        let value = const_int(&item.expr)
            .ok_or_else(|| format!("{file}: PROTOCOL_VERSION: expected an integer"))?;
        out.version = u32::try_from(value)
            .map_err(|_| format!("{file}: PROTOCOL_VERSION: value is out of range"))?;
    } else if is_code_file(file)
        && matches!(&*item.ty, syn::Type::Path(path) if path.path.is_ident("i32"))
    {
        let code =
            const_int(&item.expr).ok_or_else(|| format!("{file}: {name}: expected an integer"))?;
        out.errors.push(ErrorCode {
            name,
            code,
            doc: doc_text(&item.attrs),
        });
    }
    Ok(())
}

/// Whether `file` is the protocol's `code.rs`, the only place that holds the error codes.
fn is_code_file(file: &str) -> bool {
    Path::new(file).file_name().and_then(|name| name.to_str()) == Some("code.rs")
}

/// Record every type name in `ty`, as written at `context` in `file`.
fn note_references(file: &str, context: &str, ty: &Ty, out: &mut Protocol) {
    match ty {
        Ty::Named(name) => note_name(file, context, name, out),
        Ty::Option(inner) | Ty::Vec(inner) => note_references(file, context, inner, out),
        Ty::Bool | Ty::U32 | Ty::U64 | Ty::I64 | Ty::F64 | Ty::Str => {}
    }
}

fn note_name(file: &str, context: &str, name: &str, out: &mut Protocol) {
    out.references.push(Reference {
        file: file.to_owned(),
        context: context.to_owned(),
        name: name.to_owned(),
    });
}

fn read_macro(file: &str, item: &syn::ItemMacro, out: &mut Protocol) -> Result<(), String> {
    if item.ident.is_some() {
        return Ok(());
    }
    let Some(segment) = item.mac.path.segments.last() else {
        return Ok(());
    };
    match segment.ident.to_string().as_str() {
        "protocol_methods" => {
            let methods = parse_methods(item.mac.tokens.clone())
                .map_err(|error| format!("{file}: {error}"))?;
            for method in &methods {
                let context = format!("method {}", method.wire);
                note_name(file, &context, &method.params, out);
                note_name(file, &context, &method.result, out);
            }
            out.methods.extend(methods);
        }
        "protocol_notifications" => {
            let notifications = parse_notifications(item.mac.tokens.clone())
                .map_err(|error| format!("{file}: {error}"))?;
            for notification in &notifications {
                let context = format!("notification {}", notification.wire);
                note_name(file, &context, &notification.params, out);
            }
            out.notifications.extend(notifications);
        }
        _ => {}
    }
    Ok(())
}

fn parse_methods(tokens: proc_macro2::TokenStream) -> syn::Result<Vec<MethodDecl>> {
    let parser = |input: syn::parse::ParseStream| -> syn::Result<Vec<MethodDecl>> {
        let mut out = Vec::new();
        while !input.is_empty() {
            let attrs = input.call(syn::Attribute::parse_outer)?;
            let variant: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=>]>()?;
            let wire: syn::LitStr = input.parse()?;
            let params: syn::Type = input.parse()?;
            input.parse::<syn::Token![->]>()?;
            let result: syn::Type = input.parse()?;
            input.parse::<syn::Token![;]>()?;
            out.push(MethodDecl {
                variant: variant.to_string(),
                wire: wire.value(),
                params: type_name(&unparen(params))?,
                result: type_name(&unparen(result))?,
                doc: doc_text(&attrs),
            });
        }
        Ok(out)
    };
    syn::parse::Parser::parse2(parser, tokens)
}

fn parse_notifications(tokens: proc_macro2::TokenStream) -> syn::Result<Vec<NotificationDecl>> {
    let parser = |input: syn::parse::ParseStream| -> syn::Result<Vec<NotificationDecl>> {
        let mut out = Vec::new();
        while !input.is_empty() {
            let attrs = input.call(syn::Attribute::parse_outer)?;
            let variant: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=>]>()?;
            let wire: syn::LitStr = input.parse()?;
            let params: syn::Type = input.parse()?;
            input.parse::<syn::Token![in]>()?;
            let topic: syn::Ident = input.parse()?;
            input.parse::<syn::Token![;]>()?;
            out.push(NotificationDecl {
                variant: variant.to_string(),
                wire: wire.value(),
                params: type_name(&unparen(params))?,
                topic: topic.to_string(),
                doc: doc_text(&attrs),
            });
        }
        Ok(out)
    };
    syn::parse::Parser::parse2(parser, tokens)
}

fn unparen(ty: syn::Type) -> syn::Type {
    match ty {
        syn::Type::Paren(paren) => unparen(*paren.elem),
        other => other,
    }
}

fn type_name(ty: &syn::Type) -> syn::Result<String> {
    match ty {
        syn::Type::Path(path) if path.qself.is_none() => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .ok_or_else(|| syn::Error::new_spanned(ty, "expected a type name")),
        _ => Err(syn::Error::new_spanned(ty, "expected a type name")),
    }
}

fn convert_type(file: &str, context: &str, ty: &syn::Type) -> Result<Ty, String> {
    let syn::Type::Path(path) = ty else {
        return Err(unsupported(file, context));
    };
    if path.qself.is_some() {
        return Err(unsupported(file, context));
    }
    let Some(segment) = path.path.segments.last() else {
        return Err(unsupported(file, context));
    };
    let name = segment.ident.to_string();
    let simple = match name.as_str() {
        "bool" => Some(Ty::Bool),
        "u32" => Some(Ty::U32),
        "u64" => Some(Ty::U64),
        "i64" => Some(Ty::I64),
        "f64" => Some(Ty::F64),
        "String" => Some(Ty::Str),
        _ => None,
    };
    if let Some(simple) = simple {
        if !matches!(segment.arguments, syn::PathArguments::None) {
            return Err(unsupported(file, context));
        }
        return Ok(simple);
    }
    if matches!(name.as_str(), "Option" | "Vec") {
        let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            return Err(unsupported(file, context));
        };
        let mut types = arguments.args.iter().filter_map(|argument| match argument {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        });
        let Some(inner) = types.next() else {
            return Err(unsupported(file, context));
        };
        if types.next().is_some() {
            return Err(unsupported(file, context));
        }
        let inner = convert_type(file, context, inner)?;
        return Ok(if name == "Option" {
            Ty::Option(Box::new(inner))
        } else {
            Ty::Vec(Box::new(inner))
        });
    }
    if matches!(
        name.as_str(),
        "Box" | "HashMap" | "BTreeMap" | "HashSet" | "Rc" | "Arc"
    ) {
        return Err(unsupported(file, context));
    }
    Ok(Ty::Named(name))
}

fn unsupported(file: &str, context: &str) -> String {
    format!("{file}: {context}: unsupported type in a protocol type (design 4.2.3)")
}

/// The `serde(..)` entries of `attrs` as `name` or `name = "value"`, sorted.
fn serde_entries(attrs: &[syn::Attribute]) -> Result<Vec<String>, String> {
    let mut entries = Vec::new();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
        attr.parse_nested_meta(|meta| {
            let name = meta
                .path
                .get_ident()
                .map(ToString::to_string)
                .ok_or_else(|| meta.error("unsupported serde path"))?;
            if meta.input.peek(syn::Token![=]) {
                let value: syn::LitStr = meta.value()?.parse()?;
                entries.push(format!("{name} = \"{}\"", value.value()));
            } else {
                entries.push(name);
            }
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    }
    entries.sort();
    Ok(entries)
}

fn expect_serde(
    file: &str,
    context: &str,
    attrs: &[syn::Attribute],
    expected: &[&str],
) -> Result<(), String> {
    let entries = serde_entries(attrs).map_err(|error| format!("{file}: {context}: {error}"))?;
    if entries
        .iter()
        .map(String::as_str)
        .ne(expected.iter().copied())
    {
        return Err(format!(
            "{file}: {context}: unexpected serde attribute {entries:?}"
        ));
    }
    Ok(())
}

fn doc_text(attrs: &[syn::Attribute]) -> String {
    let mut lines = Vec::new();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("doc")) {
        let syn::Meta::NameValue(value) = &attr.meta else {
            continue;
        };
        let syn::Expr::Lit(value) = &value.value else {
            continue;
        };
        let syn::Lit::Str(text) = &value.lit else {
            continue;
        };
        lines.push(text.value());
    }
    lines
        .iter()
        .map(|line| line.strip_prefix(' ').unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn const_int(expr: &syn::Expr) -> Option<i64> {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Int(value) => value.base10_parse().ok(),
            _ => None,
        },
        syn::Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => {
            const_int(&unary.expr).map(|value| -value)
        }
        _ => None,
    }
}
