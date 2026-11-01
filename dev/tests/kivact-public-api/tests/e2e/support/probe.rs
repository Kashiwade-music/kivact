//! Role: Shared fixture for probe matrix testing infrastructure.
//!
//! Shared fixture of the probe matrices (`probe_matrix.rs` for the handle
//! methods, `probe_free_functions.rs` for the free functions and
//! constructors): the diagnostics capture, the handles every probe may
//! use, the outcome labels and the click/settle helpers.
//!
//! Outcome labels:
//! - `ok` — returned normally, no error, no diagnostic (`ok=<v>` shows the value)
//! - `err:<Kind>` — aborted by a located misuse panic caught at the boundary
//! - `ok+err:<Kind>` — returned, and an error was reported (deferred or routed)
//! - `ign+diag:<Kind>` — returned, ignored, and a diagnostic was recorded
//! - `ABORT` — aborted with no error recorded anywhere (silent)
//! - `ESCAPED` — the panic escaped kivact's boundaries
//! - `-` — not applicable

use kivact::view::{callback, selector};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kivact::Selector;
use kivact::binding;
use kivact::diagnostics::ErrorKind;
use kivact::motion::{Animated, Easing, FrameLoop, Transition, animated, frame_loop};
use kivact::prelude::*;
use kivact::task::{Task, spawn};
use kivact::test::Screen;
use kivact::{Binding, Callback};

// ---------------------------------------------------------------------------
// diagnostics capture (tracing debug events of target "kivact")

/// A subscriber recording the `kind` of every debug event of `kivact`.
pub(crate) struct Capture(pub(crate) Arc<Mutex<Vec<String>>>);

struct KindVisitor(Option<String>);

impl tracing::field::Visit for KindVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "kind" {
            self.0 = Some(format!("{value:?}"));
        }
    }
}

impl tracing::Subscriber for Capture {
    fn register_callsite(
        &self,
        _: &'static tracing::Metadata<'static>,
    ) -> tracing::subscriber::Interest {
        tracing::subscriber::Interest::always()
    }
    fn enabled(&self, m: &tracing::Metadata<'_>) -> bool {
        m.target() == "kivact"
    }
    fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }
    fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}
    fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
    fn event(&self, e: &tracing::Event<'_>) {
        if *e.metadata().level() == tracing::Level::DEBUG {
            let mut v = KindVisitor(None);
            e.record(&mut v);
            if let Some(k) = v.0
                && let Ok(mut l) = self.0.lock()
            {
                l.push(short_kind_str(&k));
            }
        }
    }
    fn enter(&self, _: &tracing::span::Id) {}
    fn exit(&self, _: &tracing::span::Id) {}
}

fn short_kind_str(debug: &str) -> String {
    debug
        .split(|c: char| !c.is_alphanumeric())
        .next()
        .unwrap_or("?")
        .to_string()
}

/// The variant name of an error kind (`SetupOnly`, `Disposed`, ...).
pub(crate) fn kind_name(k: &ErrorKind) -> String {
    short_kind_str(&format!("{k:?}"))
}

/// Take the captured diagnostics.
pub(crate) fn take_diags(diags: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
    std::mem::take(&mut *diags.lock().unwrap_or_else(|e| e.into_inner()))
}

// ---------------------------------------------------------------------------
// handles

/// The context the probes read (`view::context`).
#[derive(Clone, Copy)]
pub(crate) struct Ctx(pub(crate) u8);

/// One handle of every public kind.
#[derive(Clone, Copy)]
pub(crate) struct H {
    pub(crate) sig: Signal<i32>,
    pub(crate) val: Value<i32>,
    pub(crate) var: Var<i32>,
    pub(crate) cb: Callback<i32>,
    pub(crate) bind: Binding<i32>,
    pub(crate) sel: Selector<i32>,
    pub(crate) nref: NodeRef,
    pub(crate) res: Resource<i32>,
    pub(crate) act: Action<i32, i32>,
    pub(crate) q: Action<(), i32>,
    pub(crate) task: Task,
    pub(crate) fl: FrameLoop,
    pub(crate) anim: Animated<f32>,
}

/// Create one handle of every kind (setup only).
pub(crate) fn make() -> H {
    let sig = signal(1);
    let val = memo(move || sig.get() * 2);
    let set = callback(move |v| sig.set(v));
    H {
        sig,
        val,
        var: var(0),
        cb: callback(move |v| sig.set(v)),
        bind: binding(val, move |value| set.try_call(value)),
        sel: selector(move || Some(sig.get())),
        nref: node_ref(),
        res: resource(move || sig.get(), |v: i32| async move { Ok(v) }),
        act: action(|v: i32| async move { Ok(v) }),
        q: action(|_v: i32| async move { Ok(()) }).policy(ActionPolicy::Queue),
        task: spawn(async {}),
        fl: frame_loop(|_| {}),
        anim: animated(
            memo(move || val.with(|v| *v as f32)),
            Transition::tween(10, Easing::Linear),
        ),
    }
}

/// A probe: one call of one public item; returns the value it shows.
pub(crate) type ProbeFn = fn(&H) -> String;

/// The value of a probe that returns nothing.
pub(crate) fn ok() -> String {
    String::new()
}

thread_local! {
    /// The value of the last probe that returned (None: it aborted).
    static DONE: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Run `p` and record that it returned.
pub(crate) fn run_probe(p: ProbeFn, h: &H) {
    let r = p(h);
    DONE.with(|d| *d.borrow_mut() = Some(r));
}

/// Forget the last probe's value.
pub(crate) fn reset_done() {
    DONE.with(|d| *d.borrow_mut() = None);
}

/// Take the last probe's value.
pub(crate) fn take_done() -> Option<String> {
    DONE.with(|d| d.borrow_mut().take())
}

// ---------------------------------------------------------------------------
// driving and classifying

/// Run the turn until idle and let 100 ms of timers and frames pass.
pub(crate) fn settle(s: &Screen) {
    s.run_until_idle();
    s.advance(Duration::from_millis(100));
}

/// Click the button `name` and settle; `Err` if a panic escaped.
pub(crate) fn click(s: &Screen, name: &str) -> Result<(), String> {
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        s.get_by_role(Role::Button, name).click();
        settle(s);
    }));
    r.map_err(|p| {
        p.downcast_ref::<String>()
            .cloned()
            .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_else(|| "opaque panic".into())
    })
}

/// The outcome label (see the module doc).
pub(crate) fn classify(
    done: Option<String>,
    errs: &[String],
    diags: &[String],
    escaped: Option<String>,
) -> String {
    if let Some(e) = escaped {
        return format!(
            "ESCAPED({})",
            e.lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(40)
                .collect::<String>()
        );
    }
    let err = errs.first().cloned();
    match (done, err) {
        (Some(v), None) if diags.is_empty() => {
            if v.is_empty() {
                "ok".into()
            } else {
                format!("ok={v}")
            }
        }
        (Some(_), None) => format!("ign+diag:{}", diags[0]),
        (Some(_), Some(e)) => format!("ok+err:{e}"),
        (None, Some(e)) => format!("err:{e}"),
        (None, None) => "ABORT".into(),
    }
}

/// The kinds of the errors recorded after the first `skip`.
pub(crate) fn err_kinds(s: &Screen, skip: usize) -> Vec<String> {
    s.errors()
        .iter()
        .skip(skip)
        .map(|e| kind_name(e.kind()))
        .collect()
}

/// Compare `table` with the golden file `name` under `tests/e2e/`, or rewrite
/// it when `KIVACT_PROBE_BLESS` is set.
pub(crate) fn check_golden(name: &str, table: &str) {
    let path = format!("{}/tests/e2e/{name}", env!("CARGO_MANIFEST_DIR"));
    if std::env::var_os("KIVACT_PROBE_BLESS").is_some() {
        if let Err(e) = std::fs::write(&path, table) {
            panic!("cannot write {path}: {e}");
        }
        return;
    }
    let golden = std::fs::read_to_string(&path).unwrap_or_default();
    let golden = golden.replace("\r\n", "\n");
    if golden != table {
        let diff: Vec<String> = golden
            .lines()
            .zip(table.lines())
            .filter(|(a, b)| a != b)
            .map(|(a, b)| format!("golden: {a}\n   now: {b}"))
            .collect();
        panic!(
            "Probe matrix outcomes changed ({} rows); re-bless if intended:\n{}",
            diff.len(),
            diff.join("\n")
        );
    }
}
