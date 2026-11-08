//! Role: The intermediate model of the DevTools protocol that the generators consume.

/// A protocol type, as written in the crate.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Ty {
    Bool,
    U32,
    U64,
    I64,
    F64,
    Str,
    Option(Box<Ty>),
    Vec(Box<Ty>),
    Named(String),
}

/// One named field of a record or a union variant.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Field {
    pub(super) name: String,
    pub(super) ty: Ty,
    pub(super) doc: String,
}

/// One wire type of the protocol.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Item {
    Record {
        name: String,
        doc: String,
        fields: Vec<Field>,
    },
    Choice {
        name: String,
        doc: String,
        variants: Vec<(String, String)>,
    },
    Union {
        name: String,
        doc: String,
        variants: Vec<(String, String, Vec<Field>)>,
    },
    Alias {
        name: String,
        doc: String,
        ty: Ty,
    },
}

impl Item {
    pub(super) fn name(&self) -> &str {
        match self {
            Item::Record { name, .. }
            | Item::Choice { name, .. }
            | Item::Union { name, .. }
            | Item::Alias { name, .. } => name,
        }
    }
}

/// One method of the protocol.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct MethodDecl {
    pub(super) variant: String,
    pub(super) wire: String,
    pub(super) params: String,
    pub(super) result: String,
    pub(super) doc: String,
}

/// One notification of the protocol.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct NotificationDecl {
    pub(super) variant: String,
    pub(super) wire: String,
    pub(super) params: String,
    pub(super) topic: String,
    pub(super) doc: String,
}

/// One JSON-RPC error code of the protocol.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct ErrorCode {
    pub(super) name: String,
    pub(super) code: i64,
    pub(super) doc: String,
}

/// Everything the generator reads from the protocol crate.
#[derive(Default, Debug, PartialEq)]
pub(super) struct Protocol {
    pub(super) version: u32,
    pub(super) items: Vec<Item>,
    pub(super) methods: Vec<MethodDecl>,
    pub(super) notifications: Vec<NotificationDecl>,
    pub(super) errors: Vec<ErrorCode>,
    /// `DevtoolsConfig` has the `#[serde(rename = "$schema")]` field; only the config schema lists it.
    pub(super) config_schema_key: bool,
}

/// The `snake_case` name of a Rust identifier, as `serde` writes it.
pub(super) fn snake(name: &str) -> String {
    let mut out = String::new();
    for (index, character) in name.chars().enumerate() {
        if index > 0 && character.is_uppercase() {
            out.push('_');
        }
        out.push(character.to_ascii_lowercase());
    }
    out
}

/// The `UPPER_SNAKE_CASE` name of a Rust identifier.
pub(super) fn upper_snake(name: &str) -> String {
    snake(name).to_uppercase()
}
