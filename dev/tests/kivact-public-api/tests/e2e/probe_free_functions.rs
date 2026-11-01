//! Role: Tests for free functions and constructors with context.
//! take no handle, so what matters is the context they are called in. Each
//! is called in the contexts the plain-function API introduced, with the
//! outcome fixed as a table (`tests/e2e/probe_free_functions.golden.md`):
//!
//! | column | where the probe runs |
//! |---|---|
//! | setup | setup of the probing owner |
//! | handler | a click handler of the probing owner |
//! | pure (memo) | a memo of the probing owner |
//! | task after `.await` | a task spawned in setup, after its first `.await` |
//! | queued run | the synchronous part of a run of an `ActionPolicy::Queue` action of the probing owner |
//! | queued run after dispose | a run of that action still draining after the owner was disposed |
//! | `Action::on_done` (Queue) | a handler the probing owner registered on a queued action of its parent |
//! | `Action::on_done` | a handler the probing owner registered on an action of its parent |
//! | frame callback | the first tick of a `FrameLoop` of the probing owner |
//! | `on_mount` | an `on_mount` of the probing owner |
//! | cleanup (disposal) | an `on_cleanup` of the probing owner, while it is disposed |
//! | Callback from setup / effect / handler | a `Callback` of the probing owner, called from the setup, a user effect or a handler of a sibling |
//!
//! The probing owner provides `Ctx(1)`; its parent (which owns the two
//! actions of the `on_done` columns) and the sibling that calls the
//! callbacks see `Ctx(2)`. So `view::context` shows `ok=1` exactly where the
//! code runs for the probing owner. The outcome labels are listed in
//! `support/probe.rs`.
//!
//! Regenerate after an intended behaviour change with
//! `KIVACT_PROBE_BLESS=1 cargo test -p kivact-public-api-tests --test e2e -- probe_free_functions::`.

use kivact::view::{callback, selector};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kivact::motion::FrameRate;
use kivact::motion::{Easing, Transition, animated, frame_loop};
use kivact::platform;
use kivact::platform::FileDialog;
use kivact::prelude::*;
use kivact::task::sleep;
use kivact::task::spawn;
use kivact::test::{Screen, render};
use kivact::{Callback, effect_on};

use crate::support::probe::{
    Capture, Ctx, ProbeFn, check_golden, classify, click, err_kinds, kind_name, make, ok,
    reset_done, run_probe, settle, take_diags, take_done,
};

fn probes() -> Vec<(&'static str, ProbeFn)> {
    vec![
        ("signal", |_| {
            signal(0);
            ok()
        }),
        ("memo", |_| {
            memo(|| 0);
            ok()
        }),
        ("effect", |_| {
            effect(|| {});
            ok()
        }),
        ("effect_on", |h| {
            effect_on(h.sig, |_| {});
            ok()
        }),
        ("on_cleanup", |_| {
            on_cleanup(|| {});
            ok()
        }),
        ("on_mount", |_| {
            on_mount(|| {});
            ok()
        }),
        ("view::context", |_| {
            format!("{:?}", Some(context::<Ctx>().0))
        }),
        ("task::spawn", |_| {
            spawn(async {});
            ok()
        }),
        ("var", |_| {
            var(0);
            ok()
        }),
        ("selector", |_| {
            selector(|| None::<u8>);
            ok()
        }),
        ("callback", |_| {
            callback(|_: ()| {});
            ok()
        }),
        ("resource", |_| {
            resource(|| (), |()| async { Ok(0u8) });
            ok()
        }),
        ("action", |_| {
            action(|| async { Ok(0u8) });
            ok()
        }),
        ("node_ref", |_| {
            node_ref();
            ok()
        }),
        ("NodeRef::focused", |h| {
            h.nref.focused();
            ok()
        }),
        ("NodeRef::focus_within", |h| {
            h.nref.focus_within();
            ok()
        }),
        ("animated", |_| {
            animated::<f32>(1.0, Transition::tween(10, Easing::Linear));
            ok()
        }),
        ("frame_loop", |_| {
            frame_loop(|_| {});
            ok()
        }),
        ("IntoValue::into_value (constant)", |_| {
            IntoValue::<String>::into_value("label");
            ok()
        }),
        ("IntoValue::into_value (closure)", |_| {
            (|| 1).into_value();
            ok()
        }),
        ("platform::request_window_size", |_| {
            drop(platform::request_window_size(320, 240));
            ok()
        }),
        ("platform::set_fullscreen", |_| {
            drop(platform::set_fullscreen(true));
            ok()
        }),
        ("platform::copy_text", |_| {
            drop(platform::copy_text("x"));
            ok()
        }),
        ("platform::read_text", |_| {
            drop(platform::read_text());
            ok()
        }),
        ("platform::pick_file", |_| {
            drop(platform::pick_file(FileDialog::default()));
            ok()
        }),
        ("platform::pick_files", |_| {
            drop(platform::pick_files(FileDialog::default()));
            ok()
        }),
        ("platform::pick_folder", |_| {
            drop(platform::pick_folder(FileDialog::default()));
            ok()
        }),
        ("platform::save_file", |_| {
            drop(platform::save_file(FileDialog::default()));
            ok()
        }),
        ("platform::open_path", |_| {
            drop(platform::open_path("x"));
            ok()
        }),
        ("platform::reveal_path", |_| {
            drop(platform::reveal_path("x"));
            ok()
        }),
        ("platform::fullscreen", |_| {
            format!("{:?}", platform::fullscreen())
        }),
        ("platform::reduced_motion", |_| {
            format!("{:?}", platform::reduced_motion())
        }),
        ("platform::scale_factor", |_| {
            format!("{:?}", platform::scale_factor())
        }),
        ("platform::surface_active", |_| {
            format!("{:?}", platform::surface_active())
        }),
        ("platform::window_size", |_| {
            format!("{:?}", platform::window_size())
        }),
        ("platform::safe_area", |_| {
            format!("{:?}", platform::safe_area())
        }),
        ("platform::capabilities", |_| {
            format!("{:?}", platform::capabilities())
        }),
        ("style::current_theme", |_| {
            kivact::style::current_theme();
            ok()
        }),
    ]
}

// ---------------------------------------------------------------------------
// contexts

#[derive(Clone, Copy, PartialEq, Debug)]
enum F {
    Setup,
    Handler,
    Pure,
    AfterAwait,
    QueueJob,
    DrainingJob,
    QueueDone,
    ActionDone,
    Frame,
    Mount,
    Cleanup,
    CallbackFromSetup,
    CallbackFromEffect,
    CallbackFromHandler,
}

const CONTEXTS: [(F, &str); 14] = [
    (F::Setup, "setup"),
    (F::Handler, "handler"),
    (F::Pure, "pure (memo)"),
    (F::AfterAwait, "task after `.await`"),
    (F::QueueJob, "queued run"),
    (F::DrainingJob, "queued run after dispose"),
    (F::QueueDone, "`Action::on_done` (Queue)"),
    (F::ActionDone, "`Action::on_done`"),
    (F::Frame, "frame callback"),
    (F::Mount, "`on_mount`"),
    (F::Cleanup, "cleanup (disposal)"),
    (F::CallbackFromSetup, "Callback from setup"),
    (F::CallbackFromEffect, "Callback from effect"),
    (F::CallbackFromHandler, "Callback from handler"),
];

/// What `cell` reads after the probe ran.
struct Probed {
    screen: Screen,
    /// The queued action whose run ran the probe (`F::QueueJob`): a run's
    /// failure is its state, not a reported error.
    job_queue: Rc<Cell<Option<Action<()>>>>,
}

/// The probing owner (`Ctx(1)`) under a `show` of a parent (`Ctx(2)`)
/// that owns an action and a queued action, and a sibling (`Ctx(2)`) that calls
/// the probing owner's callback.
fn fixture(ctx: F, p: ProbeFn) -> Probed {
    let job_queue: Rc<Cell<Option<Action<()>>>> = Rc::new(Cell::new(None));
    let jq = job_queue.clone();
    let screen = render(move || {
        provide(Ctx(2), move || {
            let act = action(|v: u8| async move { Ok(v) });
            let parent_queue = action(|| async { Ok(()) }).policy(ActionPolicy::Queue);
            let alive = signal(true);
            let release = Rc::new(Cell::new(false));
            let callback: Rc<Cell<Option<Callback<()>>>> = Rc::new(Cell::new(None));
            let (jq, rel, cb_in, cb_out) =
                (jq.clone(), release.clone(), callback.clone(), callback);
            column((
                show(alive, move || {
                    let (jq, rel, cb_in) = (jq.clone(), rel.clone(), cb_in.clone());
                    provide(Ctx(1), move || {
                        probing_owner(ctx, p, act, parent_queue, &jq, &rel, &cb_in)
                    })
                }),
                show(
                    move || true,
                    move || {
                        let cb = cb_out.get().expect("the probing owner was set up first");
                        match ctx {
                            F::CallbackFromSetup => cb.call(()),
                            F::CallbackFromEffect => effect(move || cb.call(())),
                            _ => {}
                        }
                        {
                            let label = ("call").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .on_press(move || cb.call(()))
                    },
                ),
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
                    let label = ("release").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || release.set(true)),
            ))
        })
    });
    Probed { screen, job_queue }
}

/// The setup of the probing owner: arms the probe for `ctx`.
fn probing_owner(
    ctx: F,
    p: ProbeFn,
    act: Action<u8, u8>,
    parent_queue: Action<()>,
    job_queue: &Rc<Cell<Option<Action<()>>>>,
    release: &Rc<Cell<bool>>,
    slot: &Rc<Cell<Option<Callback<()>>>>,
) -> impl IntoView + use<> {
    let h = make();
    slot.set(Some(callback(move |()| run_probe(p, &h))));
    match ctx {
        F::Setup => run_probe(p, &h),
        F::AfterAwait => {
            spawn(async move {
                sleep(Duration::from_millis(1)).await;
                run_probe(p, &h);
            });
        }
        F::QueueJob => {
            let q = action(move || {
                run_probe(p, &h);
                async { Ok(()) }
            })
            .policy(ActionPolicy::Queue);
            q.run(());
            job_queue.set(Some(q));
        }
        F::DrainingJob => {
            let release = release.clone();
            let q = action(move || {
                let release = release.clone();
                async move {
                    while !release.get() {
                        sleep(Duration::from_millis(5)).await;
                    }
                    run_probe(p, &h);
                    Ok(())
                }
            })
            .policy(ActionPolicy::Queue);
            q.run(());
        }
        F::QueueDone => {
            parent_queue.on_done(move |_| run_probe(p, &h));
            parent_queue.run(());
        }
        F::ActionDone => {
            act.on_done(move |_| run_probe(p, &h));
            act.run(1);
        }
        F::Frame => {
            let fired = Cell::new(false);
            let fl = frame_loop(move |_| {
                if !fired.replace(true) {
                    run_probe(p, &h);
                }
            });
            fl.start(FrameRate::Hz(30.0));
        }
        F::Mount => on_mount(move || run_probe(p, &h)),
        F::Cleanup => on_cleanup(move || run_probe(p, &h)),
        _ => {}
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
    ))
}

/// The outcome of `p` in `ctx`.
fn cell(ctx: F, p: ProbeFn, diags: &Arc<Mutex<Vec<String>>>) -> String {
    reset_done();
    take_diags(diags);
    let Probed { screen, job_queue } = fixture(ctx, p);
    settle(&screen);
    // Contexts driven by a click start from the clean fixture.
    let clicks: &[&str] = match ctx {
        F::Handler => &["probe"],
        F::Pure => &["arm"],
        F::CallbackFromHandler => &["call"],
        F::Cleanup => &["kill"],
        F::DrainingJob => &["kill", "release"],
        _ => &[],
    };
    let base = if clicks.is_empty() {
        0
    } else {
        take_diags(diags);
        screen.errors().len()
    };
    let mut escaped = None;
    for name in clicks {
        if let Err(e) = click(&screen, name) {
            escaped.get_or_insert(e);
        }
    }
    let mut errs = err_kinds(&screen, base);
    if let Some(AsyncState::Failed(error)) = job_queue.get().map(|q| q.state()) {
        errs.push(kind_name(error.kind()));
    }
    classify(take_done(), &errs, &take_diags(diags), escaped)
}

fn generate() -> String {
    let diags = Arc::new(Mutex::new(Vec::new()));
    let sub = Capture(diags.clone());
    tracing::subscriber::with_default(sub, || {
        let mut out = String::from("| function |");
        for (_, n) in CONTEXTS {
            out.push_str(&format!(" {n} |"));
        }
        out.push_str("\n|---|");
        for _ in CONTEXTS {
            out.push_str("---|");
        }
        out.push('\n');
        for (name, p) in probes() {
            out.push_str(&format!("| `{name}` |"));
            for (c, _) in CONTEXTS {
                out.push_str(&format!(" {} |", cell(c, p, &diags)));
            }
            out.push('\n');
        }
        out
    })
}

#[test]
fn every_context_runs_the_probe_and_the_fixture_is_clean() {
    for (c, name) in CONTEXTS {
        let r = cell(
            c,
            |_| String::from("ran"),
            &Arc::new(Mutex::new(Vec::new())),
        );
        assert_eq!(r, "ok=ran", "context {name}");
    }
}

#[test]
fn probe_free_functions_match_the_golden_table() {
    check_golden("probe_free_functions.golden.md", &generate());
}
