//! Role: Verify the state rule on passive records, mutated types, markers and locals.

fn run(text: &str) -> Vec<String> {
    let source = crate::check_source::fixture(text);
    let index = super::Index::build(std::slice::from_ref(&source));
    let mut errors = Vec::new();
    super::check(&source, &index, &mut errors);
    errors
}

#[test]
fn passive_records_need_no_marker() {
    let text = "struct Record { a: bool, b: Option<u8> }\nimpl Record { fn new() -> Self { Self { a: false, b: None } } fn a(mut self, a: bool) -> Self { self.a = a; self } }";
    assert!(run(text).is_empty());
}

#[test]
fn a_mut_self_method_makes_fields_state() {
    let errors = run(
        "struct Model { open: bool, pending: Option<u8> }\nimpl Model { fn open(&mut self) { self.open = true; } }",
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("Model holds state fields open, pending"),
        "{errors:?}"
    );
}

#[test]
fn an_interior_mutable_field_makes_fields_state() {
    let errors = run("struct Cache { hit: Cell<bool>, last: Option<u8> }");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("interior-mutable field `hit`"),
        "{errors:?}"
    );
}

#[test]
fn a_signal_field_is_state_only_when_a_method_writes_it() {
    let written = "struct Handle { open: Signal<bool>, busy: Signal<bool> }\nimpl Handle { fn show(&self) { self.open.set(true); } }";
    assert_eq!(run(written).len(), 1);
    let read = "struct Handle { open: Signal<bool>, busy: Signal<bool> }\nimpl Handle { fn open(self) -> Signal<bool> { self.open } }";
    assert!(run(read).is_empty());
}

#[test]
fn one_marker_before_the_type_closes_it_and_an_unused_one_is_an_error() {
    let closed = "// xtask-allow(state): open and pending are set by separate calls; probe\nstruct Model { open: bool, pending: Option<u8> }\nimpl Model { fn open(&mut self) {} }";
    assert!(run(closed).is_empty());
    let unused = run("// xtask-allow(state): probe\nstruct Plain { a: bool }");
    assert_eq!(unused.len(), 1, "{unused:?}");
    assert!(unused[0].contains("unused marker"), "{unused:?}");
}

#[test]
fn names_and_derives_do_not_exempt_a_type() {
    let text = "struct ListApi { a: Cell<bool>, b: Cell<bool> }\n#[derive(Clone, PartialEq, Eq, Hash)]\nstruct Slot { a: bool, b: Option<u8> }\nimpl Slot { fn f(&mut self) {} }\nstruct ButtonState { a: Cell<bool>, b: Cell<bool> }\nstruct ViewOptions { a: Cell<bool>, b: Option<u8> }";
    assert_eq!(run(text).len(), 4);
}

#[test]
fn legacy_markers_close_a_type_and_are_counted() {
    let text = "// xtask-allow(state): legacy: open, pending; probe\nstruct Model { open: bool, pending: Option<u8> }\nimpl Model { fn open(&mut self) {} }";
    assert!(run(text).is_empty());
    let source = crate::check_source::fixture(text);
    assert_eq!(super::legacy_count(&source), 1);
    let plain = crate::check_source::fixture(
        "// xtask-allow(state): probe\nstruct Model { open: bool, pending: Option<u8> }",
    );
    assert_eq!(super::legacy_count(&plain), 0);
}

#[test]
fn enum_variants_need_one_marker_on_the_enum() {
    let errors = run("enum Mode { Open { a: Cell<bool>, b: Option<u8> }, Closed }");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("Mode has variants with state fields Open.a, Open.b"),
        "{errors:?}"
    );
    let closed = "// xtask-allow(state): legacy: Open.a, Open.b; probe\nenum Mode { Open { a: Cell<bool>, b: Option<u8> }, Closed }";
    assert!(run(closed).is_empty());
}

#[test]
fn state_locals_need_one_marker_on_the_function() {
    let errors = run("fn sample() {\nlet a = signal(false);\nlet b = signal(None);\n}");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("function sample combines state locals a, b"),
        "{errors:?}"
    );
    let closed = "// xtask-allow(state): legacy: a, b; probe\nfn sample() {\nlet a = signal(false);\nlet b = signal(None);\n}";
    assert!(run(closed).is_empty());
}

#[test]
fn the_replaced_marker_is_an_error() {
    let errors =
        run("struct Fact {\n// xtask-allow(state-fields): independent: probe\na: bool,\n}");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("obsolete"), "{errors:?}");
}

#[test]
fn callback_options_and_values_are_not_state() {
    let text = "struct Row { on_press: Option<Callback<()>>, render: Option<Box<dyn Fn()>>, title: String, open: bool }\nimpl Row { fn f(&mut self) {} }";
    assert!(run(text).is_empty());
}

#[test]
fn tuple_elements_count_one_by_one() {
    let errors =
        run("struct Pair { values: (bool, Option<u8>) }\nimpl Pair { fn f(&mut self) {} }");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("values.0, values.1"), "{errors:?}");
}
