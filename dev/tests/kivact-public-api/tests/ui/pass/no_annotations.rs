#![allow(dead_code, unused_variables)]
// Type inference: plain-function components, the flow
// closures, list rows and `callback` closures compile without type
// annotations on closure parameters (no E0282).
use std::time::Duration;

use kivact::motion::{Spring, Transition, animated, frame_loop};
use kivact::prelude::*;
use kivact::{Callback, effect_on};
use kivact::task::spawn;
use kivact::view::{RowHeight, error_boundary, callback, selector};
use kivact::view::provide_theme;
use kivact::task::sleep;
use kivact::view::context;
use kivact::Selector;

#[derive(Clone, PartialEq)]
struct Task {
    id: u64,
    title: String,
    done: bool,
}

#[derive(Clone, Copy)]
struct Board {
    tasks: Signal<Vec<Task>>,
    selected: Signal<Option<u64>>,
}

// A plain-function component with arguments (no props struct).
fn task_row(id: u64, task: Value<Task>) -> impl IntoView {
    let board = context::<Board>();
    let done = memo(move || task.with(|t| t.done));
    let r = node_ref();
    row((
        text(move || task.get().title),
        { let label = ("select").into_value(); node().role(Role::Button).focusable(true).name(label).child(text(label)) }.on_press(move || board.selected.set(Some(id))),
    ))
    .node_ref(r)
    .selected(done)
    .test_id(format!("row-{id}"))
}

fn task_editor(id: Value<u64>) -> impl IntoView {
    text(move || format!("editing {}", id.get()))
}

fn lane_title(i: usize, title: Value<String>) -> impl IntoView {
    text(move || format!("{i}: {}", title.get()))
}

fn mode_view(mode: u32) -> impl IntoView {
    text(format!("mode {mode}"))
}

// A child taking a callback prop.
fn picker(on_pick: Callback<u64>) -> impl IntoView {
    { let label = ("pick").into_value(); node().role(Role::Button).focusable(true).name(label).child(text(label)) }.on_press(move || on_pick.call(7))
}

fn board_view() -> impl IntoView {
    let tasks = signal(Vec::<Task>::new());
    let selected = signal(None::<u64>);
    let board = Board { tasks, selected };
    let titles = signal(vec![String::from("todo")]);
    let mode = signal(0u32);
    let open = signal(true);
    let theme = signal(Theme::dark());
    let sel = selector(selected);
    let local = var(0u32);
    let tick = signal(());
    let log = signal(Vec::<String>::new());

    // Callbacks: the parameter type comes from the annotation, the
    // expected type, the pattern or the later call. As with any Rust
    // closure, a parameter whose methods are called before anything fixes
    // its type needs `|s: String|`; nothing else is annotated (the `?`
    // below infers `Result<(), UiError>`).
    let a = callback(|v: i32| {});
    let b: Callback<String> = callback(move |s| log.update(|l| l.push(s)));
    let c = callback(move |()| open.set(false));
    let d = callback(move |n| selected.set(Some(n)));
    d.call(3u64);
    let parsed = callback(move |s: String| {
        let n: u64 = s.parse()?;
        selected.set(Some(n));
        Ok(())
    });

    effect(move || {
        tick.get();
        let _ = sel.is(&1);
    });
    effect_on(selected, move |id| local.set(id.map_or(0, |i| i as u32)));
    on_mount(|| {});
    on_cleanup(|| {});
    let _task = spawn(async move {
        sleep(Duration::from_millis(1)).await;
        mode.set(1);
    });

    let res = resource(move || mode.get(), |m| async move { Ok(m * 2) });
    let save = action(|t: Task| async move { Ok(t.id) });
    let reload = action(|| async { Ok(()) });
    let queue = action(|line: String| async move { Ok(()) }).policy(ActionPolicy::Queue);
    let x = animated(move || if open.get() { 1.0f32 } else { 0.0 }, Transition::spring(Spring::DEFAULT));
    let frames = frame_loop(|f| {
        let _ = f.delta;
    });

    provide(board, move || {
        column((
            show(open, || text("open")).fallback(|| text("closed")),
            show_some(selected, task_editor).fallback(|| text("none")),
            show_some(selected, |id| text(move || id.get().to_string())),
            dynamic(mode, |m| match m {
                0 => text("zero").into_view(),
                1 => mode_view(1).into_view(),
                _ => text("other").into_view(),
            }),
            for_each(tasks, |t| t.id, task_row),
            for_each(tasks, |t| t.id, |id, t| text(move || format!("{id} {}", t.get().title))),
            for_index(titles, lane_title),
            for_index(titles, |i, t| text(move || format!("{i} {}", t.get()))),
            virtual_list(tasks, |t| t.id, |id, t| task_row(id, t))
                .row_height(RowHeight::Fixed(24.0)),
            dynamic(mode, mode_view),
            dynamic(mode, |m| if m == 0 { text("a") } else { text("b") }),
            error_boundary(|| picker(callback(|id| {})))
                .on_error(|e| {
                    let _ = e.kind();
                })
                .fallback(|e, reset| { let label = (e.to_string()).into_value(); node().role(Role::Button).focusable(true).name(label).child(text(label)) }.on_press(move || reset())),
            provide_theme(theme, || text("dark")),
            provide_theme(Theme::light(), move || {
                let _ = context::<Board>();
                text("light")
            }),
        ))
    })
}

fn app() -> impl IntoView {
    board_view()
}

#[test]
fn renders() {
    let screen = kivact::test::render(app);
    let screen = kivact::test::render_with(kivact::test::RenderOptions::from_env(), || text("x"));
}

fn main() {
    let _ = || App::new("No annotations").run(app);
}
