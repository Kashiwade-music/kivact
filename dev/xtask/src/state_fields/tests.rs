//! Role: Verify state-field classification, markers and module statics.

use super::check::check;
use super::mode::Mode;
use crate::check_source::fixture;
use std::collections::BTreeSet;

fn run(text: &str, policy: &str, enums: &[&str]) -> Vec<String> {
    run_with_report(text, policy, enums, false)
}

fn run_with_report(text: &str, policy: &str, enums: &[&str], report: bool) -> Vec<String> {
    run_mode(
        text,
        policy,
        enums,
        Mode {
            report,
            audit: false,
        },
    )
}

fn run_mode(text: &str, policy: &str, enums: &[&str], mode: Mode) -> Vec<String> {
    let source = fixture(text);
    let policy: toml::Table = policy.parse().unwrap();
    let enums: BTreeSet<String> = enums.iter().map(|name| (*name).to_owned()).collect();
    let mut errors = Vec::new();
    let _ = check(
        &source,
        &policy,
        &Default::default(),
        &enums,
        &Default::default(),
        mode,
        &mut errors,
    );
    errors
}

#[test]
fn detects_wrapped_fields_and_inline_exceptions() {
    let errors = run(
        "struct Hidden { open: Cell<bool>, pending: Option<Job> }\nstruct Fact { open: bool }\nstruct Single { state: Option<LoadPhase> }\nstruct Facts {\n// xtask-allow(state-fields): independent: facts\na: bool,\n// xtask-allow(state-fields): independent: facts\nb: bool,\n}\nstruct Mixed { pending: Option<Job>, phase: LoadPhase }",
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
        Mode {
            report: false,
            audit: false,
        },
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
        Mode {
            report: false,
            audit: false,
        },
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
fn api_exemption_is_disabled_by_mutable_cells() {
    let errors = run(
        "struct ProbeApi { plain: Option<Job>, wrapped: Signal<Option<Job>>, counter: Cell<u64> }",
        "",
        &[],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("ProbeApi combines state fields: plain, wrapped, counter"),
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
        "struct Probe {\n// xtask-allow(state-fields): independent: fact\na: bool,\n// xtask-allow(state-fields): independent: fact\nb: Option<Job>,\n}",
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
        "struct Pair {\n// xtask-allow(state-fields): independent: passive record\nvalues: (bool, Option<Job>),\n}",
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
fn local_variables_are_noted_in_report_mode_and_combine_as_errors() {
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
    let errors = run_with_report(text, "", &[], false);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("function combines state locals: flag, pending, count, nested"),
        "{errors:?}"
    );
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
    let user = source_in("user", "use crate::T; impl Element for T {}");
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
            Mode {
                report: false,
                audit: false,
            },
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

#[test]
fn counts_slots_filled_after_construction() {
    let text = "struct Probe { job: OnceCell<Job>, other: OnceLock<Job> }";
    let errors = run(text, "", &[]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].starts_with("state-fields:"), "{errors:?}");
    assert!(
        errors[0].contains("Probe combines state fields: job, other"),
        "{errors:?}"
    );
}

fn audit_rows(text: &str, policy: &str, enums: &[&str]) -> Vec<super::audit::AuditRow> {
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
        Mode {
            report: false,
            audit: true,
        },
        &mut errors,
    )
}

#[test]
fn audit_lists_counted_fields_of_multi_state_items_only() {
    let rows = audit_rows(
        "struct Single { open: bool, note: String }\nstruct Mixed { pending: Option<Job>, phase: LoadPhase }\nstruct Closed {\n// xtask-allow(state-fields): projection: written only by publish\na: bool,\n// xtask-allow(state-fields): independent: all four combinations are valid\nb: bool,\n}",
        "",
        &["kivact_ui_soft::widgets::LoadPhase"],
    );
    assert_eq!(super::audit::open_count(&rows), 1, "{rows:?}");
    let closure = |field: &str| {
        rows.iter()
            .find(|row| row.field == field)
            .map(|row| row.closure.as_str())
    };
    assert_eq!(closure("Single.open"), None);
    assert_eq!(closure("Mixed.pending"), Some("OPEN"));
    assert_eq!(closure("Mixed.phase"), Some("type"));
    assert_eq!(closure("Closed.a"), Some("projection"));
    assert_eq!(closure("Closed.b"), Some("independent"));
}

#[test]
fn audit_file_records_open_rows() {
    let rows = audit_rows(
        "struct Mixed { pending: Option<Job>, phase: LoadPhase }",
        "",
        &["kivact_ui_soft::widgets::LoadPhase"],
    );
    let root = std::env::temp_dir().join(format!("kivact-xtask-audit-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    super::audit::write(&root, &rows).unwrap();
    let text = std::fs::read_to_string(root.join("dev/xtask/state-audit.md")).unwrap();
    assert!(text.contains("| OPEN |"), "{text}");
    assert!(text.contains("Mixed"), "{text}");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn an_arc_callback_option_is_a_setting() {
    let errors = run(
        "struct Probe { a: Option<u8>, b: Option<Job>, f: Option<std::sync::Arc<dyn Fn(u32) -> bool + Send + Sync>> }",
        "",
        &[],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("Probe combines state fields: a, b"),
        "{errors:?}"
    );
    assert!(!errors[0].contains(", f"), "{errors:?}");
}

#[test]
fn enum_variants_named_none_are_not_option_locals() {
    let text =
        "fn sample() {\nlet section = signal(0usize);\nlet overlay = signal(Overlay::None);\n}";
    assert!(run(text, "", &[]).is_empty());
    let text = "fn sample() {\nlet section = signal(0usize);\nlet empty = signal(Option::None);\n}";
    let errors = run(text, "", &[]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("function combines state locals: section, empty"));
}

#[test]
fn one_unmarked_field_beside_a_marked_one_is_an_error() {
    let errors = run(
        "struct Probe {\n// xtask-allow(state-fields): independent: probe fact\na: bool,\nb: Option<Job>,\n}",
        "",
        &[],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("Probe combines state fields: b"),
        "{errors:?}"
    );
}

#[test]
fn one_unmarked_local_beside_a_marked_one_is_an_error() {
    let text = "fn sample() {\n// xtask-allow(state-fields): independent: probe fact\nlet a = signal(false);\nlet b = signal(None);\n}";
    let errors = run(text, "", &[]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("function combines state locals: b"),
        "{errors:?}"
    );
}

/// Run X4 and X5 over fixtures, one per `(module, text)`, each in its own file.
fn verify_sources(modules: &[(&str, &str)]) -> Vec<String> {
    let sources: Vec<crate::check_source::Source> = modules
        .iter()
        .map(|(module, text)| {
            let mut source = fixture(text);
            source.module = (*module).to_owned();
            source.path =
                format!("crates/kivact-ui-soft/src/{}.rs", module.replace("::", "/")).into();
            source
        })
        .collect();
    let mut errors = Vec::new();
    super::verify(&sources, &|_| true, &mut errors);
    errors
}

/// A struct whose field `a` carries the projection `reason`.
fn projected(reason: &str, visibility: &str) -> String {
    format!(
        "struct S {{\n// xtask-allow(state-fields): projection: {reason}\n{visibility} a: Cell<bool>,\n// xtask-allow(state-fields): independent: probe b\nb: Cell<bool>,\n}}\n"
    )
}

/// `S` with the writer `S::publish`, followed by `rest`.
fn published(rest: &str) -> String {
    projected("written only by S::publish", "")
        + "impl S { fn publish(&self) { self.a.set(true); } }\n"
        + rest
}

/// The errors of `published(rest)` alone in module `widgets::list`.
fn verify_published(rest: &str) -> Vec<String> {
    verify_sources(&[("widgets::list", &published(rest))])
}

#[test]
fn projection_needs_an_existing_single_writer() {
    let errors = verify_published("impl S { fn other(&self) { self.a.set(false) } }");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("S.a is also written in other"),
        "{errors:?}"
    );
}

#[test]
fn projection_writer_must_exist() {
    let text = projected("written only by missing", "")
        + "impl S { fn publish(&self) { self.a.set(true) } }";
    let errors = verify_sources(&[("widgets::list", &text)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("projection writer missing of S.a does not exist"),
        "{errors:?}"
    );
}

#[test]
fn projection_allows_empty_initial_values_in_constructors() {
    let text = projected("written only by S::publish", "")
        + "impl S {\nfn new() -> Self { Self { a: Cell::new(false), b: Cell::new(false) } }\nfn publish(&self) { self.a.set(true) }\n}\n\
           struct T {\n// xtask-allow(state-fields): projection: written only by T::report\nsent: Sent<Option<u8>>,\n// xtask-allow(state-fields): independent: probe c\nc: bool,\n}\n\
           impl T {\nfn new() -> Self { Self { sent: { let mut sent = Sent::new(); sent.update(None); sent }, c: true } }\nfn report(&mut self) { self.sent.update(Some(1)); }\n}";
    let errors = verify_sources(&[("widgets::list", &text)]);
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn projection_marker_must_name_one_writer() {
    let text = projected("readonly snapshot", "");
    let errors = verify_sources(&[("widgets::list", &text)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("must name one writer"), "{errors:?}");
}

#[test]
fn projection_fields_visible_to_the_whole_crate_are_rejected() {
    for (module, visibility) in [
        ("widgets::list", "pub(crate)"),
        ("widgets::list", "pub"),
        ("widgets", "pub(super)"),
        ("", ""),
    ] {
        let text = projected("written only by S::publish", visibility)
            + "impl S { fn publish(&self) { self.a.set(true) } }";
        let errors = verify_sources(&[(module, &text)]);
        assert_eq!(errors.len(), 1, "{module} {visibility}: {errors:?}");
        assert!(
            errors[0].contains("visible to the whole crate"),
            "{errors:?}"
        );
    }
}

#[test]
fn projection_fields_are_checked_in_the_module_their_visibility_allows() {
    let other = "struct T { a: Cell<bool> }\nfn touch(t: &T) { t.a.set(true) }";
    let private = published("");
    let errors = verify_sources(&[("widgets::list", &private), ("widgets::other", other)]);
    assert!(errors.is_empty(), "{errors:?}");
    let shared = projected("written only by S::publish", "pub(super)")
        + "impl S { fn publish(&self) { self.a.set(true) } }";
    let errors = verify_sources(&[("widgets::list", &shared), ("widgets::other", other)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("S.a is also written in touch"),
        "{errors:?}"
    );
}

#[test]
fn a_same_name_field_is_the_projection_unless_its_receiver_is_another_types_self() {
    let errors = verify_published(
        "struct T { a: Cell<bool> }\nimpl T { fn touch(&self) { self.a.set(true); } }",
    );
    assert!(errors.is_empty(), "{errors:?}");
    let errors = verify_published("struct T { a: Cell<bool> }\nfn touch(t: &T) { t.a.set(true); }");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("S.a is also written in touch"),
        "{errors:?}"
    );
}

#[test]
fn an_alias_of_the_field_is_unverified() {
    let errors = verify_published("fn other(s: &S) { let cell = &s.a; cell.set(false); }");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("S.a may be changed in other") && errors[0].contains("borrowed with &"),
        "{errors:?}"
    );
}

#[test]
fn a_reference_passed_to_another_function_is_unverified() {
    let errors = verify_published(
        "impl S { fn other(&mut self) { flip(&mut self.a); } }\nfn flip(cell: &mut Cell<bool>) { cell.set(false); }",
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("borrowed with &mut"), "{errors:?}");
}

#[test]
fn an_unknown_method_on_the_field_is_unverified_and_known_reads_pass() {
    let errors = verify_published(
        "impl S { fn read(&self) -> bool { self.a.get() && self.a.clone().get() } fn other(&self) { self.a.toggle(); } }",
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("may be changed in other") && errors[0].contains("calls `toggle`"),
        "{errors:?}"
    );
}

#[test]
fn writes_through_parts_handles_and_literals_are_writes() {
    let text = "struct S {\n// xtask-allow(state-fields): projection: written only by S::publish\na: RefCell<Inner>,\n// xtask-allow(state-fields): independent: probe b\nb: Cell<bool>,\n}\n\
                impl S {\nfn publish(&self) { self.a.borrow_mut().x = 1; }\nfn part(&mut self) { self.a.get_mut().x = 2; }\nfn handle(&self) { *self.a.borrow_mut() = Inner::default(); }\nfn copy(&self) -> Self { Self { b: Cell::new(true), ..self.clone() } }\n}";
    let errors = verify_sources(&[("widgets::list", text)]);
    assert_eq!(errors.len(), 3, "{errors:?}");
    for function in ["part", "handle", "copy"] {
        assert!(
            errors
                .iter()
                .any(|error| error.contains(&format!("S.a is also written in {function}"))),
            "{function}: {errors:?}"
        );
    }
}

#[test]
fn a_pattern_binding_of_the_field_is_unverified() {
    let errors = verify_published(
        "fn ignore(s: &S) { let S { a: _, .. } = s; }\nfn bind(s: &mut S) { let S { a, .. } = s; a.set(false); }",
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("may be changed in bind") && errors[0].contains("bound by a pattern"),
        "{errors:?}"
    );
}

#[test]
fn uses_inside_macros_are_checked() {
    let errors = verify_published(
        "impl S { fn checked(&self) { assert!(self.a.replace(true)); } fn opaque(&self) { let _ = vec![self.a.take(); 1]; } fn custom(&self) { flip!(self.a); } fn shown(&self) { println!(\"{}\", self.a.get()); } }",
    );
    assert_eq!(errors.len(), 3, "{errors:?}");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("S.a is also written in checked")),
        "{errors:?}"
    );
    for function in ["opaque", "custom"] {
        assert!(
            errors.iter().any(
                |error| error.contains(&format!("may be changed in {function}"))
                    && error.contains("inside a macro")
            ),
            "{function}: {errors:?}"
        );
    }
}

#[test]
fn a_writer_returning_a_reference_is_rejected() {
    let text = projected("written only by S::publish", "")
        + "impl S { fn publish(&self) -> &Cell<bool> { &self.a } }";
    let errors = verify_sources(&[("widgets::list", &text)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("returns a reference"), "{errors:?}");
}

#[test]
fn machine_state_projections_name_the_machine_transition() {
    let machine = "pub(super) enum State {\nClosed,\nOpen {\n// xtask-allow(state-fields): projection: written only by Machine::transition on Event::Preview\npreviewed: Option<u8>,\n// xtask-allow(state-fields): independent: probe hovered\nhovered: Option<u8>,\n},\n}\n\
                   pub(super) struct Machine;\n\
                   impl MachineTrait for Machine {\ntype State = State;\nfn transition(&self, state: &State, key: Option<u8>) -> State {\nlet mut next = State::Open { previewed: None, hovered: None };\nif let State::Open { previewed, .. } = &mut next { *previewed = key; }\nnext\n}\n}";
    let errors = verify_sources(&[("select::machine", machine)]);
    assert!(errors.is_empty(), "{errors:?}");
    let reset = "fn reset() -> State { State::Open { previewed: Some(1), hovered: None } }";
    let errors = verify_sources(&[("select::machine", machine), ("select::handle", reset)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("State::Open.previewed is also written in reset"),
        "{errors:?}"
    );
    let named = "fn transition() -> State { State::Open { previewed: Some(1), hovered: None } }";
    let errors = verify_sources(&[("select::machine", machine), ("select::handle", named)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("State::Open.previewed is also written in transition"),
        "{errors:?}"
    );
}

#[test]
fn independent_reasons_are_not_shared_across_files() {
    let text = "struct P {\n// xtask-allow(state-fields): independent: probe fact\na: bool,\n// xtask-allow(state-fields): independent: probe fact\nb: bool,\n}";
    let errors = verify_sources(&[("widgets", text)]);
    assert!(errors.is_empty(), "{errors:?}");
    let errors = verify_sources(&[("widgets", text), ("other", text)]);
    assert_eq!(errors.len(), 4, "{errors:?}");
    assert!(
        errors
            .iter()
            .all(|error| error.contains("reason repeats in")),
        "{errors:?}"
    );
}
