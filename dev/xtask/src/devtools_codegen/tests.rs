//! Role: Verify reading protocol source, the schemas and the Python output.

use crate::devtools_codegen::model::Protocol;

const SOURCE: &str = r#"
/// A point.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// The horizontal coordinate.
    pub x: f64,
    /// The vertical coordinate.
    pub y: f64,
}

/// A snapshot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TreeSnapshot {
    /// The nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<Point>,
    /// The label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The window.
    pub window: u64,
}

/// The parameters of `get_tree`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetTreeParams {
    /// The window.
    pub window: u64,
}

/// No parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Empty {}

/// The app.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppInfo {
    /// The name.
    pub name: String,
}

/// A role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// A button.
    Button,
    /// A checkbox.
    CheckBox,
}

/// A scope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OverrideScope {
    /// One node.
    Node {
        /// The node.
        node: u64,
    },
    /// One window.
    Window {},
}

/// The id of a node.
pub type NodeId = u64;

protocol_methods! {
    /// The element tree of one window.
    GetTree => "get_tree" (GetTreeParams) -> TreeSnapshot;
    /// The app.
    GetAppInfo => "get_app_info" (Empty) -> AppInfo;
}

protocol_notifications! {
    /// The tree changed.
    TreeChanged => "tree_changed" (TreeSnapshot) in Tree;
}
"#;

fn protocol() -> Protocol {
    let mut protocol = Protocol::default();
    super::read::read_source("fixture.rs", SOURCE, &mut protocol).unwrap();
    protocol
}

#[test]
fn reads_records_choices_unions_aliases_and_methods() {
    let protocol = protocol();
    let schema = super::schema::schema(&protocol);
    let python = super::python::python(&protocol);
    assert!(schema.contains(r##""$ref": "#/$defs/Point""##), "{schema}");
    assert!(schema.contains(r#""x-methods""#), "{schema}");
    assert!(schema.contains(r#""name": "get_tree""#), "{schema}");
    assert!(python.contains("class Point:"), "{python}");
    assert!(python.contains("label: str | None = None"), "{python}");
    assert!(python.contains("class Role(str, Enum):"), "{python}");
    assert!(python.contains(r#"BUTTON = "button""#), "{python}");
    assert!(python.contains("class OverrideScopeNode:"), "{python}");
    assert!(
        python.contains("def get_tree(self, params: GetTreeParams) -> TreeSnapshot:"),
        "{python}"
    );
    assert!(
        python.contains("def get_app_info(self) -> AppInfo:"),
        "{python}"
    );
    assert!(python.contains("NodeId = int"), "{python}");
}

#[test]
fn refuses_unexpected_serde_attributes() {
    let mut protocol = Protocol::default();
    let error = super::read::read_source(
        "fixture.rs",
        r#"
/// A bad field.
pub struct Bad {
    /// The label.
    #[serde(default, rename = "x", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}
"#,
        &mut protocol,
    )
    .unwrap_err();
    assert!(error.contains("Bad"), "{error}");
}

#[test]
fn refuses_unsupported_types() {
    let mut protocol = Protocol::default();
    let error = super::read::read_source(
        "fixture.rs",
        "/// A map.\npub struct Map { /// The values.\npub values: HashMap<String, u32> }",
        &mut protocol,
    )
    .unwrap_err();
    assert!(error.contains("Map"), "{error}");
}

#[test]
fn refuses_choices_without_rename_all() {
    let mut protocol = Protocol::default();
    let error = super::read::read_source(
        "fixture.rs",
        "/// A plain enum.\npub enum Plain { /// One.\nOne }",
        &mut protocol,
    )
    .unwrap_err();
    assert!(error.contains("Plain"), "{error}");
}

#[test]
fn names_convert_to_snake_and_upper_snake() {
    assert_eq!(super::model::snake("TestId"), "test_id");
    assert_eq!(super::model::snake("FocusVisible"), "focus_visible");
    assert_eq!(super::model::upper_snake("CheckBox"), "CHECK_BOX");
}

#[test]
fn refuses_tuple_and_unit_structs() {
    for (source, name) in [
        ("/// A pair.\npub struct Pair(pub u32);", "Pair"),
        ("/// A marker.\npub struct Marker;", "Marker"),
    ] {
        let mut protocol = Protocol::default();
        let error = super::read::read_source("fixture.rs", source, &mut protocol).unwrap_err();
        assert!(error.contains("fixture.rs"), "{error}");
        assert!(error.contains(name), "{error}");
    }
}

#[test]
fn refuses_a_field_that_names_an_unknown_type() {
    let mut protocol = Protocol::default();
    super::read::read_source(
        "fixture.rs",
        "/// A holder.\npub struct Holder {\n    /// The missing one.\n    pub missing: Missing,\n}\n",
        &mut protocol,
    )
    .unwrap();
    let error = super::read::check_references(&protocol).unwrap_err();
    assert!(error.contains("fixture.rs"), "{error}");
    assert!(error.contains("Holder.missing"), "{error}");
    assert!(error.contains("Missing"), "{error}");
}

#[test]
fn reads_error_codes_only_from_code_rs() {
    let mut protocol = Protocol::default();
    super::read::read_source("app.rs", "pub const LIMIT: i32 = 5;", &mut protocol).unwrap();
    assert!(protocol.errors.is_empty());
    super::read::read_source(
        "code.rs",
        "/// A missing node.\npub const NODE_NOT_FOUND: i32 = -32010;",
        &mut protocol,
    )
    .unwrap();
    assert_eq!(protocol.errors.len(), 1);
    assert_eq!(protocol.errors[0].name, "NODE_NOT_FOUND");
}
