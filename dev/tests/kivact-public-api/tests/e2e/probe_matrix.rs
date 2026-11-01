//! Role: Tests for handle methods in all context combinations.
//! the six contexts {setup, handler, pure (memo), after dispose, other App,
//! while the same value is borrowed} plus "dropped App", with the outcome
//! fixed as a table (`tests/e2e/probe_matrix.golden.md`). The free functions
//! and constructors, which take no handle, have their own table with the
//! contexts the plain-function API introduced (`probe_free_functions.rs`).
//!
//! The outcome labels are listed in `support/probe.rs`; `-` here means the
//! handle has no borrowable value.
//!
//! Regenerate after an intended behaviour change with
//! `KIVACT_PROBE_BLESS=1 cargo test -p kivact-public-api-tests --test e2e -- probe_matrix::`.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kivact::binding;
use kivact::motion::FrameRate;
use kivact::prelude::*;
use kivact::test::{Screen, render};

use crate::support::probe::{
    Capture, Ctx, H, ProbeFn, check_golden, classify, click, err_kinds, kind_name, make, ok,
    reset_done, run_probe, settle, take_diags, take_done,
};

#[derive(Clone, Copy, PartialEq, Debug)]
enum Borrow {
    None,
    Sig,
    Val,
    Var,
    Anim,
    Bind,
}

fn probes() -> Vec<(&'static str, Borrow, ProbeFn)> {
    use Borrow as B;
    vec![
        // ---- Signal
        ("Signal::get", B::Sig, |h| h.sig.get().to_string()),
        ("Signal::get_untracked", B::Sig, |h| {
            h.sig.get_untracked().to_string()
        }),
        ("Signal::with", B::Sig, |h| h.sig.with(|v| v.to_string())),
        ("Signal::with_untracked", B::Sig, |h| {
            h.sig.with_untracked(|v| v.to_string())
        }),
        ("Signal::set", B::Sig, |h| {
            h.sig.set(5);
            ok()
        }),
        ("Signal::update", B::Sig, |h| {
            h.sig.update(|v| *v += 1);
            ok()
        }),
        ("Signal::notify", B::Sig, |h| {
            h.sig.notify();
            ok()
        }),
        ("Signal::field", B::Sig, |h| {
            h.sig.field(|v| v, |v| v);
            ok()
        }),
        ("Signal::try_get", B::Sig, |h| {
            format!("{:?}", h.sig.try_get())
        }),
        ("Signal::into_value", B::Sig, |h| {
            h.sig.into_value();
            ok()
        }),
        // ---- Value (memo)
        ("Value::get", B::Val, |h| h.val.get().to_string()),
        ("Value::get_untracked", B::Val, |h| {
            h.val.get_untracked().to_string()
        }),
        ("Value::with", B::Val, |h| h.val.with(|v| v.to_string())),
        ("Value::with_untracked", B::Val, |h| {
            h.val.with_untracked(|v| v.to_string())
        }),
        ("Value::try_get", B::Val, |h| {
            format!("{:?}", h.val.try_get())
        }),
        ("IntoValue::into_value (constant)", B::None, |_| {
            let _: Value<i32> = 3.into_value();
            ok()
        }),
        // ---- Var
        ("Var::get", B::Var, |h| h.var.get().to_string()),
        ("Var::with", B::Var, |h| h.var.with(|v| v.to_string())),
        ("Var::set", B::Var, |h| {
            h.var.set(4);
            ok()
        }),
        ("Var::update", B::Var, |h| {
            h.var.update(|v| *v += 1);
            ok()
        }),
        // ---- Callback
        ("Callback::call", B::Sig, |h| {
            h.cb.call(7);
            ok()
        }),
        ("Callback::try_call", B::Sig, |h| {
            format!("{:?}", h.cb.try_call(7).map_err(|e| kind_name(e.kind())))
        }),
        ("Callback::bind", B::None, |h| {
            h.cb.bind(8);
            ok()
        }),
        ("Callback::to_fn", B::Sig, |h| {
            (h.cb.to_fn())(9);
            ok()
        }),
        // ---- Binding
        ("Binding::get", B::Bind, |h| h.bind.get().to_string()),
        ("Binding::with", B::Bind, |h| h.bind.with(|v| v.to_string())),
        ("Binding::set", B::Bind, |h| {
            h.bind.set(6);
            ok()
        }),
        ("binding", B::None, |h| {
            let cb = h.cb;
            binding(h.val, move |value| cb.try_call(value));
            ok()
        }),
        // ---- Selector
        ("Selector::is", B::Sig, |h| h.sel.is(&1).to_string()),
        // ---- NodeRef
        ("NodeRef::focus", B::None, |h| {
            h.nref.focus();
            ok()
        }),
        ("NodeRef::scroll_into_view", B::None, |h| {
            h.nref.scroll_into_view();
            ok()
        }),
        ("NodeRef::rect", B::None, |h| {
            h.nref.rect();
            ok()
        }),
        ("NodeRef::focused", B::None, |h| {
            h.nref.focused();
            ok()
        }),
        ("NodeRef::focus_within", B::None, |h| {
            h.nref.focus_within();
            ok()
        }),
        // ---- Resource
        ("Resource::latest", B::None, |h| {
            format!("{:?}", h.res.latest())
        }),
        ("Resource::refetch", B::None, |h| {
            h.res.refetch();
            ok()
        }),
        ("Resource::cancel", B::None, |h| {
            h.res.cancel();
            ok()
        }),
        ("Resource::state", B::None, |h| {
            format!("{:?}", h.res.state())
        }),
        // ---- Action
        ("Action::run", B::None, |h| {
            h.act.run(1);
            ok()
        }),
        ("Action::retry", B::None, |h| {
            h.act.retry();
            ok()
        }),
        ("Action::cancel", B::None, |h| {
            h.act.cancel();
            ok()
        }),
        ("Action::state", B::None, |h| format!("{:?}", h.act.state())),
        ("Action::latest", B::None, |h| {
            format!("{:?}", h.act.latest())
        }),
        ("Action::input", B::None, |h| format!("{:?}", h.act.input())),
        ("Action::attempt", B::None, |h| h.act.attempt().to_string()),
        ("Action::on_done", B::None, |h| {
            h.act.on_done(|_| {});
            ok()
        }),
        ("Action::policy", B::None, |h| {
            let _ = h.act.policy(ActionPolicy::Latest);
            ok()
        }),
        // ---- Action with ActionPolicy::Queue
        ("Action::run (Queue)", B::None, |h| {
            h.q.run(1);
            ok()
        }),
        ("Action::state (Queue)", B::None, |h| {
            let _ = h.q.state();
            ok()
        }),
        ("Action::pending_len (Queue)", B::None, |h| {
            h.q.pending_len().to_string()
        }),
        ("Action::on_done (Queue)", B::None, |h| {
            h.q.on_done(|_| {});
            ok()
        }),
        ("Action::close (Queue)", B::None, |h| {
            drop(h.q.close(Duration::from_millis(10)));
            ok()
        }),
        // ---- Task
        ("Task::cancel", B::None, |h| {
            h.task.cancel();
            ok()
        }),
        ("Task::is_finished", B::None, |h| {
            h.task.is_finished().to_string()
        }),
        // ---- FrameLoop
        ("FrameLoop::start", B::None, |h| {
            h.fl.start(FrameRate::Hz(30.0));
            ok()
        }),
        ("FrameLoop::stop", B::None, |h| {
            h.fl.stop();
            ok()
        }),
        ("FrameLoop::rate", B::None, |h| {
            h.fl.rate();
            ok()
        }),
        // ---- Animated
        ("Animated::get", B::Anim, |h| h.anim.get().to_string()),
        ("Animated::get_untracked", B::Anim, |h| {
            h.anim.get_untracked().to_string()
        }),
        ("Animated::with", B::Anim, |h| {
            h.anim.with(|v| v.to_string())
        }),
        ("Animated::value", B::Anim, |h| {
            h.anim.value();
            ok()
        }),
        ("Animated::jump_to", B::Anim, |h| {
            h.anim.jump_to(9.0);
            ok()
        }),
        ("Animated::is_running", B::Anim, |h| {
            h.anim.is_running().to_string()
        }),
        // ---- free functions
        ("untrack", B::None, |h| {
            kivact::untrack(|| h.sig.get()).to_string()
        }),
        ("blocking", B::None, |_| {
            drop(kivact::task::blocking(|| 1));
            ok()
        }),
        ("task::sleep", B::None, |_| {
            drop(kivact::task::sleep(Duration::from_millis(1)));
            ok()
        }),
        ("task::timeout", B::None, |_| {
            drop(kivact::task::timeout(Duration::from_millis(1), async {}));
            ok()
        }),
        ("diagnostics::why", B::None, |h| {
            kivact::diagnostics::why(&h.nref).is_some().to_string()
        }),
    ]
}

fn borrow(b: Borrow, h: &H, f: &mut dyn FnMut()) -> bool {
    match b {
        Borrow::None => return false,
        Borrow::Sig => h.sig.with(|_| f()),
        Borrow::Val => h.val.with(|_| f()),
        Borrow::Var => h.var.with(|_| f()),
        Borrow::Anim => h.anim.with(|_| f()),
        Borrow::Bind => h.bind.with(|_| f()),
    }
    true
}

// ---------------------------------------------------------------------------
// contexts

#[derive(Clone, Copy, PartialEq, Debug)]
enum C {
    Setup,
    Handler,
    Pure,
    Disposed,
    OtherApp,
    DroppedApp,
    Borrowed,
}

const CONTEXTS: [(C, &str); 7] = [
    (C::Setup, "setup"),
    (C::Handler, "handler"),
    (C::Pure, "pure (memo)"),
    (C::Disposed, "after dispose"),
    (C::OtherApp, "other App"),
    (C::DroppedApp, "dropped App"),
    (C::Borrowed, "borrowed"),
];

type Stash = Rc<Cell<Option<H>>>;

/// App with a victim branch (holding every handle) under `show(alive)`.
fn fixture(ctx: C, p: ProbeFn, b: Borrow, stash: Stash) -> Screen {
    render(move || {
        provide(Ctx(1), move || {
            let alive = signal(true);
            let st = stash.clone();
            column((
                show(alive, move || {
                    let h = make();
                    st.set(Some(h));
                    if ctx == C::Setup {
                        run_probe(p, &h);
                    }
                    let arm = signal(false);
                    let m = memo(move || {
                        if arm.get() {
                            run_probe(p, &h);
                        }
                        0
                    });
                    column((
                        text(move || m.get().to_string()),
                        node()
                            .role(Role::Group)
                            .name("target")
                            .focusable(true)
                            .node_ref(h.nref),
                        {
                            let label = ("arm").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .on_press(move || arm.set(true)),
                        {
                            let label = ("probe").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .on_press(move || run_probe(p, &h)),
                        {
                            let label = ("probe-borrowed").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .on_press(move || {
                            borrow(b, &h, &mut || run_probe(p, &h));
                        }),
                    ))
                }),
                {
                    let label = ("kill").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || alive.set(false)),
                {
                    let label = ("outer-probe").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press({
                    let st = stash.clone();
                    move || {
                        if let Some(h) = st.get() {
                            run_probe(p, &h)
                        }
                    }
                }),
            ))
        })
    })
}

fn cell(ctx: C, p: ProbeFn, b: Borrow, diags: &Arc<Mutex<Vec<String>>>) -> String {
    if ctx == C::Borrowed && b == Borrow::None {
        return "-".into();
    }
    reset_done();
    let take_diags = || take_diags(diags);
    let stash: Stash = Rc::new(Cell::new(None));
    match ctx {
        C::Setup | C::Handler | C::Pure | C::Borrowed | C::Disposed => {
            // Fixture construction itself must be clean (checked by `fixture_is_clean`).
            let s = fixture(ctx, p, b, stash.clone());
            settle(&s);
            let base = s.errors().len();
            if ctx != C::Setup {
                take_diags();
            }
            let escaped = match ctx {
                C::Setup => None,
                C::Handler => click(&s, "probe").err(),
                C::Pure => click(&s, "arm").err(),
                C::Borrowed => click(&s, "probe-borrowed").err(),
                C::Disposed => {
                    let _ = click(&s, "kill");
                    let _ = take_diags();
                    reset_done();
                    let base2 = s.errors().len();
                    let e = click(&s, "outer-probe").err();
                    let done = take_done();
                    return classify(done, &err_kinds(&s, base2), &take_diags(), e);
                }
                _ => None,
            };
            let done = take_done();
            let skip = if ctx == C::Setup { 0 } else { base };
            classify(done, &err_kinds(&s, skip), &take_diags(), escaped)
        }
        C::OtherApp | C::DroppedApp => {
            let a = fixture(C::Handler, |_| ok(), Borrow::None, stash.clone());
            settle(&a);
            let h = stash.get().expect("fixture ran");
            let a = if ctx == C::DroppedApp {
                drop(a);
                None
            } else {
                Some(a)
            };
            let a_base = a.as_ref().map(|a| a.errors().len()).unwrap_or(0);
            let b = render(move || {
                {
                    let label = ("probe").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || run_probe(p, &h))
            });
            settle(&b);
            take_diags();
            reset_done();
            let escaped = click(&b, "probe").err();
            if let Some(a) = &a {
                settle(a);
            }
            let done = take_done();
            let mut errs = err_kinds(&b, 0);
            if let Some(a) = &a {
                errs.extend(
                    err_kinds(a, a_base)
                        .into_iter()
                        .map(|k| format!("{k}(in A)")),
                );
            }
            classify(done, &errs, &take_diags(), escaped)
        }
    }
}

fn generate() -> String {
    let diags = Arc::new(Mutex::new(Vec::new()));
    let sub = Capture(diags.clone());
    tracing::subscriber::with_default(sub, || {
        let mut out = String::new();
        out.push_str("| method |");
        for (_, n) in CONTEXTS {
            out.push_str(&format!(" {n} |"));
        }
        out.push('\n');
        out.push_str("|---|");
        for _ in CONTEXTS {
            out.push_str("---|");
        }
        out.push('\n');
        for (name, b, p) in probes() {
            out.push_str(&format!("| `{name}` |"));
            for (c, _) in CONTEXTS {
                out.push_str(&format!(" {} |", cell(c, p, b, &diags)));
            }
            out.push('\n');
        }
        out
    })
}

#[test]
fn fixture_is_clean() {
    let stash: Stash = Rc::new(Cell::new(None));
    let s = fixture(C::Handler, |_| ok(), Borrow::None, stash.clone());
    settle(&s);
    assert!(s.errors().is_empty(), "{:?}", s.errors());
    assert!(stash.get().is_some());
}

#[test]
fn probe_matrix_matches_the_golden_table() {
    check_golden("probe_matrix.golden.md", &generate());
}
