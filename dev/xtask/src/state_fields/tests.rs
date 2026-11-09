//! Role: Verify state-field classification, markers and module statics.

use crate::check_source::fixture;
use crate::state_fields::check;
use std::collections::BTreeSet;

fn run(text: &str, policy: &str, enums: &[&str]) -> Vec<String> {
    run_with_report(text, policy, enums, false)
}

fn run_with_report(text: &str, policy: &str, enums: &[&str], report: bool) -> Vec<String> {
    let source = fixture(text);
    let policy: toml::Table = policy.parse().unwrap();
    let enums: BTreeSet<String> = enums.iter().map(|name| (*name).to_owned()).collect();
    let mut errors = Vec::new();
    check(
        &source,
        &policy,
        &Default::default(),
        &enums,
        &Default::default(),
        report,
        &mut errors,
    );
    errors
}

#[test]
fn detects_wrapped_fields_and_inline_exceptions() {
    let errors = run(
        "struct Hidden { open: Cell<bool>, pending: Option<Job> }\nstruct Fact { open: bool }\nstruct Single { state: Option<LoadPhase> }\nstruct Facts {\n// xtask-allow(state-fields): independent facts\na: bool,\n// xtask-allow(state-fields): independent facts\nb: bool,\n}\nstruct Mixed { pending: Option<Job>, phase: LoadPhase }",
        "",
        &["kivact_ui_soft::widgets::LoadPhase"],
    );
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(errors.iter().all(|error| !error.contains("unused marker")));
}

#[test]
fn exempts_roles_and_builder_implementations() {
    let source = fixture(
        "struct ButtonOptions { a: bool, b: bool } struct Widget { a: bool, b: bool } struct ButtonApi { a: bool, b: bool }",
    );
    let policy: toml::Table = "role_suffixes = ['Options']".parse().unwrap();
    let mut errors = Vec::new();
    check(
        &source,
        &policy,
        &["kivact_ui_soft::widgets::Widget".into()].into(),
        &Default::default(),
        &Default::default(),
        false,
        &mut errors,
    );
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn ordinary_integers_are_not_mutable_counters() {
    let errors = run(
        "struct Size { width: u32, height: usize } struct Counter { n: Cell<u64>, pending: Option<Job> }",
        "",
        &[],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
}

#[test]
fn enum_and_builder_identities_do_not_leak_across_modules() {
    let source = fixture(
        "struct LoadState; struct Model { pending: Option<Job>, phase: LoadState } struct Widget { a: bool, b: bool }",
    );
    let mut errors = Vec::new();
    check(
        &source,
        &toml::Table::new(),
        &["kivact_ui_soft::other::Widget".into()].into(),
        &["kivact_ui_soft::other::LoadState".into()].into(),
        &Default::default(),
        false,
        &mut errors,
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
}

#[test]
fn classifies_wrapped_state_but_not_collections() {
    let errors = run(
        "struct Probe { history: Signal<Vec<bool>>, pending: Signal<Option<Job>>, job: Mutex<Option<Job>>, count: Arc<AtomicUsize> }",
        "",
        &[],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("Probe combines state fields: pending, job, count"),
        "{errors:?}"
    );
}

#[test]
fn option_callbacks_are_settings_but_wrapped_options_count() {
    let errors = run(
        "struct Probe { callback: Option<Callback<()>>, pending: Option<Job>, wrapped: Var<Option<Callback<()>>> }",
        "",
        &[],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("Probe combines state fields: pending, wrapped"),
        "{errors:?}"
    );
}

#[test]
fn named_helpers_and_value_fields_are_not_counted() {
    let errors = run(
        "struct Probe { stamp: Cell<Generation>, shared: Rc<Generation>, derived: Value<bool> }",
        "named_helpers = ['Generation']",
        &[],
    );
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn phase_beside_value_is_fine() {
    let errors = run(
        "struct Probe { phase: LoadPhase, note: Value<bool> }",
        "",
        &["kivact_ui_soft::widgets::LoadPhase"],
    );
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn machine_handle_beside_option_combines() {
    let errors = run(
        "struct Probe { handle: MachineHandle<LoadPhase>, pending: Option<Job> }",
        "",
        &[],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("Probe combines state fields: pending"),
        "{errors:?}"
    );
}

#[test]
fn api_exemption_counts_only_atomics_and_wrapped_integers() {
    let errors = run(
        "struct ProbeApi { plain: Option<Job>, wrapped: Signal<Option<Job>>, counter: Cell<u64> }",
        "",
        &[],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("ProbeApi combines state fields: wrapped, counter"),
        "{errors:?}"
    );
}

#[test]
fn enum_variant_options_combine() {
    let errors = run(
        "enum Choice { Pick { first: Option<A>, second: Option<B> }, Unit, Pair(Option<A>, Option<B>) }",
        "",
        &[],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("Choice::Pick combines state fields: first, second"),
        "{errors:?}"
    );
}

#[test]
fn module_statics_combine() {
    let errors = run(
        "static FIRST: AtomicBool = AtomicBool::new(false);\nstatic SECOND: AtomicBool = AtomicBool::new(false);",
        "",
        &[],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("static flags/counters combine: FIRST, SECOND"),
        "{errors:?}"
    );
}

#[test]
fn thread_local_statics_combine() {
    let errors = run(
        "thread_local! { static FIRST: AtomicBool = AtomicBool::new(false); static SECOND: AtomicBool = AtomicBool::new(false); }",
        "",
        &[],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("static flags/counters combine: FIRST, SECOND"),
        "{errors:?}"
    );
}

#[test]
fn unused_marker_needs_a_counted_field() {
    let errors = run(
        "struct Probe {\n// xtask-allow(state-fields): passive record\nname: String,\n}",
        "",
        &[],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("unused marker"), "{errors:?}");
}

#[test]
fn fully_marked_state_has_no_errors() {
    let errors = run(
        "struct Probe {\n// xtask-allow(state-fields): independent fact\na: bool,\n// xtask-allow(state-fields): independent fact\nb: Option<Job>,\n}",
        "",
        &[],
    );
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn struct_level_marker_is_rejected() {
    let errors = run(
        "// xtask-allow(state-fields): struct level\nstruct Probe {\na: bool,\nb: Option<Job>,\n}",
        "",
        &[],
    );
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("struct-level marker")),
        "{errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("Probe combines state fields: a, b")),
        "{errors:?}"
    );
    assert!(errors.iter().all(|error| !error.contains("unused marker")));
}

#[test]
fn tuple_fields_count_per_element() {
    let errors = run(
        "struct Pair((Cell<bool>, Cell<u32>));\nstruct Named { values: (bool, Option<Job>), other: String }",
        "",
        &[],
    );
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("Pair combines state fields: 0.0, 0.1")),
        "{errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("Named combines state fields: values.0, values.1")),
        "{errors:?}"
    );
}

#[test]
fn marker_before_tuple_field_covers_every_element() {
    let errors = run(
        "struct Pair {\n// xtask-allow(state-fields): passive record\nvalues: (bool, Option<Job>),\n}",
        "",
        &[],
    );
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn dimension_rule_uses_field_names() {
    let errors = run(
        "struct Probe { next_id: u64, pending_job: usize, cache_count: u32, plain: u64 }",
        "",
        &[],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("Probe combines state fields: next_id, pending_job, cache_count"),
        "{errors:?}"
    );
}

#[test]
fn local_variables_are_reported_only_in_report_mode() {
    let text = "fn sample() {\nlet flag = signal(false);\nlet pending = RefCell::new(None);\nlet count = Cell::new(0u32);\nlet nested = Rc::new(Cell::new(0u32));\nlet plain = 1;\n}";
    let reported = run_with_report(text, "", &[], true);
    assert_eq!(
        reported
            .iter()
            .filter(|line| line.starts_with("state-locals:"))
            .count(),
        4,
        "{reported:?}"
    );
    for name in ["flag", "pending", "count", "nested"] {
        assert!(
            reported
                .iter()
                .any(|line| line.ends_with(&format!(" {name}"))),
            "{reported:?}"
        );
    }
    assert!(reported.iter().all(|line| !line.ends_with(" plain")));
    assert!(run_with_report(text, "", &[], false).is_empty());
}

fn source_in(module: &str, text: &str) -> crate::check_source::Source {
    let mut source = fixture(text);
    source.module = module.into();
    source
}

/// Runs the state-fields rule over a workspace-shaped fixture; the root module re-exports the struct and a same-named function.
fn builder_errors(struct_body: &str) -> Vec<String> {
    let mut root = source_in("", "mod m; pub use m::{T, m};");
    root.module.clear();
    let module = source_in("m", &format!("{struct_body}\npub fn m() {{}}"));
    let user = source_in("user", "use crate::T; impl NodeBuilder for T {}");
    let sources = [root, module, user];
    let surface = crate::public_surface::build(&sources);
    let (builders, enums) = crate::local_checks::model_types(&sources, &surface);
    assert!(
        builders.contains("kivact_ui_soft::m::T"),
        "builders: {builders:?}"
    );
    let mut errors = Vec::new();
    for source in &sources {
        check(
            source,
            &toml::Table::new(),
            &builders,
            &enums,
            &surface,
            false,
            &mut errors,
        );
    }
    errors
}

#[test]
fn builder_exemption_survives_function_reexport_with_module_name() {
    let errors = builder_errors("pub struct T {\na: bool,\nb: bool,\n}");
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn marker_inside_builder_struct_is_unused() {
    let errors = builder_errors(
        "pub struct T {\n// xtask-allow(state-fields): Fixture fact.\na: bool,\nb: bool,\n}",
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("unused marker"), "{errors:?}");
}
