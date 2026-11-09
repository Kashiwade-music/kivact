//! Role: Build the JSON Schemas of the protocol and of the configuration file.

use crate::devtools_codegen::json::{Json, obj, s};
use crate::devtools_codegen::model::{Field, Item, Protocol, Ty, snake};

/// The schema of every wire type, method, notification and error code.
pub(super) fn schema(protocol: &Protocol) -> String {
    let errors = protocol
        .errors
        .iter()
        .map(|error| {
            obj(vec![
                ("name", s(&error.name)),
                ("code", Json::Int(error.code)),
                ("description", s(&error.doc)),
            ])
        })
        .collect();
    let defs = protocol
        .items
        .iter()
        .map(|item| (item.name().to_owned(), item_schema(item)))
        .collect();
    let methods = protocol
        .methods
        .iter()
        .map(|method| {
            obj(vec![
                ("name", s(&method.wire)),
                ("params", s(&method.params)),
                ("result", s(&method.result)),
                ("description", s(&method.doc)),
            ])
        })
        .collect();
    let notifications = protocol
        .notifications
        .iter()
        .map(|notification| {
            obj(vec![
                ("name", s(&notification.wire)),
                ("params", s(&notification.params)),
                ("topic", s(&snake(&notification.topic))),
                ("description", s(&notification.doc)),
            ])
        })
        .collect();
    Json::Object(vec![
        (
            "$schema".into(),
            s("https://json-schema.org/draft/2020-12/schema"),
        ),
        ("$id".into(), s("kivact-devtools.schema.json")),
        ("title".into(), s("Kivact DevTools protocol")),
        ("x-protocol".into(), Json::Int(i64::from(protocol.version))),
        ("x-errors".into(), Json::Array(errors)),
        ("$defs".into(), Json::Object(defs)),
        ("x-methods".into(), Json::Array(methods)),
        ("x-notifications".into(), Json::Array(notifications)),
    ])
    .render()
}

/// The schema of `.kivact/devtools.json`, rooted at `DevtoolsConfig`.
pub(super) fn config_schema(protocol: &Protocol) -> String {
    let (_, fields) = protocol
        .items
        .iter()
        .find_map(|item| match item {
            Item::Record { name, doc, fields } if name == "DevtoolsConfig" => Some((doc, fields)),
            _ => None,
        })
        .expect("invariant: the protocol declares DevtoolsConfig");
    let mut properties = Vec::new();
    if protocol.config_schema_key {
        properties.push(("$schema".to_owned(), obj(vec![("type", s("string"))])));
    }
    for field in fields {
        properties.push((field.name.clone(), field_schema(field)));
    }
    let key_config = protocol
        .items
        .iter()
        .find_map(|item| match item {
            Item::Record { name, doc, fields } if name == "KeyConfig" => {
                Some(record_schema(doc, fields))
            }
            _ => None,
        })
        .expect("invariant: the protocol declares KeyConfig");
    Json::Object(vec![
        (
            "$schema".into(),
            s("https://json-schema.org/draft/2020-12/schema"),
        ),
        ("title".into(), s("Kivact DevTools configuration")),
        ("type".into(), s("object")),
        ("properties".into(), Json::Object(properties)),
        ("additionalProperties".into(), Json::Bool(false)),
        (
            "$defs".into(),
            Json::Object(vec![("KeyConfig".into(), key_config)]),
        ),
    ])
    .render()
}

fn item_schema(item: &Item) -> Json {
    match item {
        Item::Record { doc, fields, .. } => record_schema(doc, fields),
        Item::Choice { doc, variants, .. } => choice_schema(doc, variants),
        Item::Union { doc, variants, .. } => union_schema(doc, variants),
        Item::Alias { doc, ty, .. } => described(ty_schema(ty), doc),
    }
}

fn record_schema(doc: &str, fields: &[Field]) -> Json {
    let mut properties = Vec::new();
    let mut required = Vec::new();
    for field in fields {
        properties.push((field.name.clone(), field_schema(field)));
        if !has_default(&field.ty) {
            required.push(s(&field.name));
        }
    }
    obj(vec![
        ("description", s(doc)),
        ("type", s("object")),
        ("properties", Json::Object(properties)),
        ("required", Json::Array(required)),
        ("additionalProperties", Json::Bool(false)),
    ])
}

fn choice_schema(doc: &str, variants: &[(String, String)]) -> Json {
    let values = variants.iter().map(|(name, _)| s(&snake(name))).collect();
    obj(vec![
        ("description", s(doc)),
        ("type", s("string")),
        ("enum", Json::Array(values)),
    ])
}

fn union_schema(doc: &str, variants: &[(String, String, Vec<Field>)]) -> Json {
    let variants = variants
        .iter()
        .map(|(variant, variant_doc, fields)| {
            let mut properties =
                vec![("kind".to_owned(), obj(vec![("const", s(&snake(variant)))]))];
            let mut required = vec![s("kind")];
            for field in fields {
                properties.push((field.name.clone(), field_schema(field)));
                if !has_default(&field.ty) {
                    required.push(s(&field.name));
                }
            }
            obj(vec![
                ("description", s(variant_doc)),
                ("type", s("object")),
                ("properties", Json::Object(properties)),
                ("required", Json::Array(required)),
                ("additionalProperties", Json::Bool(false)),
            ])
        })
        .collect();
    obj(vec![
        ("description", s(doc)),
        ("oneOf", Json::Array(variants)),
    ])
}

fn field_schema(field: &Field) -> Json {
    described(ty_schema(&field.ty), &field.doc)
}

fn ty_schema(ty: &Ty) -> Json {
    match ty {
        Ty::Bool => obj(vec![("type", s("boolean"))]),
        Ty::U32 | Ty::U64 => obj(vec![("type", s("integer")), ("minimum", Json::Int(0))]),
        Ty::I64 => obj(vec![("type", s("integer"))]),
        Ty::F64 => obj(vec![("type", s("number"))]),
        Ty::Str => obj(vec![("type", s("string"))]),
        Ty::Option(inner) => ty_schema(inner),
        Ty::Vec(inner) => obj(vec![("type", s("array")), ("items", ty_schema(inner))]),
        Ty::Named(name) => obj(vec![("$ref", s(&format!("#/$defs/{name}")))]),
    }
}

fn described(schema: Json, doc: &str) -> Json {
    match schema {
        Json::Object(mut fields) => {
            fields.insert(0, ("description".to_owned(), s(doc)));
            Json::Object(fields)
        }
        other => other,
    }
}

fn has_default(ty: &Ty) -> bool {
    matches!(ty, Ty::Option(_) | Ty::Vec(_))
}
