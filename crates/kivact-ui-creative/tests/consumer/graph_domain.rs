//! Role: Public graph domain editing, view geometry and selective rollback contracts.
use super::*;
use kivact::ui::EditPhase;
#[derive(Clone, Copy)]
struct Model {
    curves: Signal<Vec<ui::Curve>>,
    selection: Signal<Vec<u64>>,
    view: Signal<ui::GraphView>,
    frame: Signal<f64>,
    lanes: Signal<Vec<ui::Lane>>,
    snap: Signal<bool>,
    phases: Signal<Vec<EditPhase>>,
    disabled: Signal<bool>,
    visible: Signal<bool>,
    present: Signal<bool>,
}
fn fixture() -> (Screen, Model) {
    kivact::test::render_with_handle(crate::options(), || {
        let model = Model {
            curves: signal(vec![
                ui::Curve::new(
                    10,
                    "Position",
                    vec![
                        ui::CurveKey::new(2, 8.0, 2.0),
                        ui::CurveKey::new(1, 2.0, -1.0),
                    ],
                ),
                ui::Curve::new(20, "Opacity", vec![ui::CurveKey::new(3, 5.0, 0.5)]),
            ]),
            selection: signal(Vec::new()),
            view: signal(ui::GraphView {
                time: (0.0, 10.0),
                value: (-2.0, 3.0),
            }),
            frame: signal(0.0),
            lanes: signal(vec![
                ui::Lane::new(10, "Position"),
                ui::Lane::new(20, "Opacity"),
            ]),
            snap: signal(false),
            phases: signal(Vec::new()),
            disabled: signal(false),
            visible: signal(true),
            present: signal(true),
        };
        (
            model,
            column((
                show(model.present, move || {
                    app_graph(
                        model.curves,
                        ui::GraphState {
                            view: model.view.into(),
                            selection: model.selection.into(),
                            playhead: model.frame.into(),
                        },
                    )
                    .lanes(model.lanes)
                    .snap(model.snap)
                    .step(0.5, 0.25)
                    .on_edit(graph_edits(model.curves, move |phase| {
                        model.phases.update(|p| p.push(phase))
                    }))
                    .name("Graph")
                    .disabled(model.disabled)
                    .hidden(move || !model.visible.get())
                    .style(style().size(700, 322))
                }),
                kivact::ui::button("Outside"),
            )),
        )
    })
}
fn key(model: Model, id: u64) -> ui::CurveKey {
    model
        .curves
        .get_untracked()
        .iter()
        .flat_map(|c| &c.keys)
        .find(|k| k.id == id)
        .expect("key")
        .clone()
}
#[test]
fn real_values_axes_selection_keyboard_and_accessibility() {
    let (screen, m) = fixture();
    let first = screen.get_by_role(Role::Button, "Position key 2");
    let second = screen.get_by_role(Role::Button, "Position key 8");
    let a = first.rect().unwrap().center();
    let b = second.rect().unwrap().center();
    assert!(near(f64::from(b.x - a.x), 300.0, 1e-5));
    assert!(near(f64::from(a.y - b.y), 180.0, 1e-5));
    first.a11y(A11yAction::Select);
    assert_eq!(m.selection.get_untracked(), [1]);
    first.press_key(Key::ArrowRight);
    assert!(near(key(m, 1).time, 2.5, 1e-5));
    assert!(near(key(m, 1).value, -1.0, 1e-5));
    screen
        .get_by_role(Role::Button, "Position key 2.5")
        .a11y_with_value(A11yAction::SetValue, "3, -0.5");
    assert!(near(key(m, 1).time, 3.0, 1e-5));
    assert!(near(key(m, 1).value, -0.5, 1e-5));
    screen
        .get_by_role(Role::Slider, "Playhead")
        .a11y_with_value(A11yAction::SetValue, "4.25");
    assert!(near(m.frame.get_untracked(), 4.25, 1e-5));
    screen
        .get_by_role(Role::Slider, "Playhead")
        .press_key(Key::ArrowDown);
    assert!(near(m.frame.get_untracked(), 3.75, 1e-5));
    assert_eq!(
        m.curves.get_untracked()[0]
            .keys
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        [2, 1]
    );
    m.lanes.set(vec![ui::Lane::new(20, "Opacity")]);
    screen.run_until_idle();
    assert!(
        screen
            .query_by_role(Role::Button, "Position key 3")
            .is_none()
    );
    let canvas = screen.get_by_role(Role::Canvas, "Graph");
    canvas.press_key(Key::Char('a'));
    clean(&screen);
}
