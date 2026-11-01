//! Role: Shared fixtures and contract checks for time and graph lane extensions.
use super::*;
use kivact::style::pct;
use kivact::ui::EditPhase;
use kivact::view::callback;

#[derive(Clone, Copy)]
pub(super) struct Editor {
    name: &'static str,
    graph: bool,
    keys: bool,
    mount: fn() -> (Screen, Fixture),
}
pub(super) const TIME_EDITORS: [Editor; 2] = [
    Editor {
        name: "Time",
        graph: false,
        keys: false,
        mount: timeline,
    },
    Editor {
        name: "Time",
        graph: false,
        keys: true,
        mount: dope_sheet,
    },
];
pub(super) const GRAPH_EDITORS: [Editor; 1] = [Editor {
    name: "Graph",
    graph: true,
    keys: false,
    mount: graph,
}];
impl Editor {
    fn item(self) -> u64 {
        if self.graph { 11 } else { 1 }
    }
    fn lane(self) -> u64 {
        if self.graph { 1 } else { 10 }
    }
    fn other(self) -> u64 {
        if self.graph { 2 } else { 20 }
    }
    fn time(self) -> f64 {
        if self.graph { 2.0 } else { 40.0 }
    }
    fn item_name(self) -> &'static str {
        if self.graph { "Alpha key 2" } else { "Item" }
    }
}
#[derive(Clone, Copy)]
pub(super) struct Fixture {
    editor: Editor,
    pub(super) clips: Signal<Vec<ui::Clip>>,
    pub(super) keys: Signal<Vec<ui::Keyframe>>,
    curves: Signal<Vec<ui::Curve>>,
    selected: Signal<Vec<u64>>,
    lanes: Signal<Vec<ui::Lane>>,
    pub(super) zoom: Signal<f64>,
    pan: Signal<Point>,
    view: Signal<ui::GraphView>,
    pub(super) phases: Signal<Vec<EditPhase>>,
    lane_calls: Signal<Vec<u64>>,
    item_calls: Signal<Vec<u64>>,
    chosen: Signal<Vec<(ui::EditorContext, &'static str)>>,
    buttons: Signal<usize>,
    flag: Signal<bool>,
    disabled: Signal<bool>,
    hidden: Signal<bool>,
}
impl Fixture {
    fn new(editor: Editor) -> Self {
        Self {
            editor,
            clips: signal(vec![
                ui::Clip::new(1, 10, 40.0, 50.0, "Item"),
                ui::Clip::new(2, 20, 140.0, 30.0, "Other"),
            ]),
            keys: signal(vec![
                ui::Keyframe::new(1, 10, 40.0, "Item"),
                ui::Keyframe::new(2, 20, 140.0, "Other"),
            ]),
            curves: signal(vec![
                ui::Curve::new(1, "Alpha", vec![ui::CurveKey::new(11, 2.0, 0.0)]),
                ui::Curve::new(2, "Beta", vec![ui::CurveKey::new(22, 8.0, 1.0)]),
            ]),
            selected: signal(vec![editor.item()]),
            lanes: signal(vec![
                ui::Lane::new(editor.lane(), "Alpha"),
                ui::Lane::new(editor.other(), "Beta"),
            ]),
            zoom: signal(2.0),
            pan: signal(Point::new(10.0, 7.0)),
            view: signal(ui::GraphView {
                time: (0.0, 10.0),
                value: (-2.0, 3.0),
            }),
            phases: signal(Vec::new()),
            lane_calls: signal(Vec::new()),
            item_calls: signal(Vec::new()),
            chosen: signal(Vec::new()),
            buttons: signal(0),
            flag: signal(false),
            disabled: signal(false),
            hidden: signal(false),
        }
    }
    pub(super) fn value(self) -> f64 {
        if self.editor.graph {
            self.curves.get_untracked()[0].keys[0].time
        } else {
            value(self.clips, self.keys, self.editor.keys)
        }
    }
    fn retime(self) {
        if self.editor.graph {
            self.curves.update(|v| v[0].keys[0].time = 2.5);
        } else {
            self.clips.update(|v| v[0].start = 45.0);
            self.keys.update(|v| v[0].time = 45.0);
        }
    }
    fn snapshot(self) -> String {
        format!(
            "{:?}{:?}{:?}{:?}{:?}{:?}",
            self.clips.get_untracked(),
            self.keys.get_untracked(),
            self.curves.get_untracked(),
            self.pan.get_untracked(),
            self.zoom.get_untracked(),
            self.view.get_untracked()
        )
    }
}
macro_rules! creative_extension_fixture {
    ($builder:expr, $m:ident, $observer:expr) => {
        $builder
            .lanes($m.lanes)
            .lane_cell(40.0, move |lane| {
                kivact::ui::button(move || format!("Cell {}", lane.get().label))
                    .on_press(move || $m.buttons.update(|n| *n += 1))
                    .disabled($m.disabled)
                    .style(style().width(pct(100)).height(20).min_width(0))
            })
            .lane_cell(30.0, move |lane| {
                kivact::ui::checkbox(move || format!("Toggle {}", lane.get().label), $m.flag)
                    .disabled($m.disabled)
                    .style(style().width(pct(100)).height(20).min_width(0))
            })
            .lane_cell(f32::NAN, |_| text("Invalid cell"))
            .on_lane_activate(move |id| $m.lane_calls.update(|v| v.push(id)))
            .on_item_activate(move |id| $m.item_calls.update(|v| v.push(id)))
            .on_edit($observer)
            .context_menu(
                |ctx| {
                    vec![MenuEntry::item(
                        "inspect",
                        format!("Inspect {:?} {:?} {}", ctx.item, ctx.lane, ctx.time),
                    )]
                },
                move |choice| $m.chosen.update(|v| v.push(choice)),
            )
            .name($m.editor.name)
            .disabled($m.disabled)
            .hidden($m.hidden)
            .style(style().size(700, if $m.editor.graph { 322 } else { 280 }))
            .into_view()
    };
}
fn timeline() -> (Screen, Fixture) {
    kivact::test::render_with_handle(crate::options(), || {
        let m = Fixture::new(TIME_EDITORS[0]);
        (
            m,
            creative_extension_fixture!(
                app_timeline(
                    m.clips,
                    ui::TimeState {
                        selection: m.selected.into(),
                        playhead: signal(3.0).into(),
                        zoom: m.zoom.into(),
                        pan: m.pan.into(),
                    }
                ),
                m,
                time_edits(m.clips, apply_clips, move |p| m
                    .phases
                    .update(|v| v.push(p)))
            ),
        )
    })
}
fn dope_sheet() -> (Screen, Fixture) {
    kivact::test::render_with_handle(crate::options(), || {
        let m = Fixture::new(TIME_EDITORS[1]);
        (
            m,
            creative_extension_fixture!(
                app_dope_sheet(
                    m.keys,
                    ui::TimeState {
                        selection: m.selected.into(),
                        playhead: signal(3.0).into(),
                        zoom: m.zoom.into(),
                        pan: m.pan.into(),
                    }
                ),
                m,
                time_edits(m.keys, apply_keys, move |p| m.phases.update(|v| v.push(p)))
            ),
        )
    })
}
fn graph() -> (Screen, Fixture) {
    kivact::test::render_with_handle(crate::options(), || {
        let m = Fixture::new(GRAPH_EDITORS[0]);
        (
            m,
            creative_extension_fixture!(
                app_graph(
                    m.curves,
                    ui::GraphState {
                        view: m.view.into(),
                        selection: m.selected.into(),
                        playhead: signal(3.0).into(),
                    }
                ),
                m,
                graph_edits(m.curves, move |p| m.phases.update(|v| v.push(p)))
            ),
        )
    })
}
pub(super) fn fixture(editor: Editor) -> (Screen, Fixture) {
    (editor.mount)()
}
fn no_callbacks(m: Fixture) {
    assert!(m.phases.get_untracked().is_empty());
    assert!(m.lane_calls.get_untracked().is_empty());
    assert!(m.item_calls.get_untracked().is_empty());
}
pub(super) fn cells(editor: Editor) {
    let (screen, m) = fixture(editor);
    let before = m.snapshot();
    let cell = screen.get_by_role(Role::Button, "Cell Alpha");
    let a = cell.rect().expect("fixture target exists");
    let b = screen
        .get_by_role(Role::CheckBox, "Toggle Alpha")
        .rect()
        .expect("fixture target exists");
    assert!(near(a.width, 40.0, 1e-6));
    assert!(near(b.width, 30.0, 1e-6));
    assert!(near(b.x - a.x, 40.0, 1e-6));
    if !editor.graph {
        assert!(near(b.x + b.width, 200.0, 1e-5));
    }
    cell.click();
    for key in [Key::Enter, Key::Space, Key::ArrowRight, Key::ContextMenu] {
        cell.press_key(key);
    }
    cell.press_key_with(
        Key::F(10),
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert!(screen.query_by_role(Role::Menu, editor.name).is_none());
    screen.get_by_role(Role::CheckBox, "Toggle Alpha").click();
    assert_eq!(m.buttons.get_untracked(), 3);
    assert!(m.flag.get_untracked());
    assert_eq!(m.selected.get_untracked(), [editor.item()]);
    assert_eq!(m.snapshot(), before);
    no_callbacks(m);
    m.lanes.set(vec![
        ui::Lane::new(editor.other(), "Renamed"),
        ui::Lane::new(editor.lane(), "Alpha"),
    ]);
    screen.run_until_idle();
    let a = screen
        .get_by_role(Role::Button, "Cell Alpha")
        .rect()
        .expect("fixture target exists");
    let b = screen
        .get_by_role(Role::Button, "Cell Renamed")
        .rect()
        .expect("fixture target exists");
    if editor.graph {
        assert!(b.y < a.y);
    } else {
        assert!(near(a.y - b.y, if editor.keys { 20.0 } else { 54.0 }, 1e-5));
    }
    m.lanes.set(vec![ui::Lane::new(editor.other(), "Renamed")]);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Button, "Cell Alpha").is_none());
    assert!(screen.query_by_text("Invalid cell").is_none());
    clean(&screen);
}
/// The second press of a double click activates nothing when its lane is
/// gone by the release.
pub(super) fn activation_needs_a_live_target(editor: Editor) {
    let (screen, m) = fixture(editor);
    let r = screen
        .get_by_role(Role::Group, "Alpha")
        .rect()
        .expect("fixture target exists");
    screen.mouse_move(r.x + 4.0, r.y + r.height * 0.5);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    screen.mouse_down(MouseButton::Primary);
    m.lanes.set(vec![ui::Lane::new(editor.other(), "Beta")]);
    screen.run_until_idle();
    screen.mouse_up(MouseButton::Primary);
    assert!(m.lane_calls.get_untracked().is_empty());
    clean(&screen);
}
pub(super) fn activation(editor: Editor) {
    let (screen, m) = fixture(editor);
    let lane = screen.get_by_role(Role::Group, "Alpha");
    lane.press_key(Key::Enter);
    let r = lane.rect().expect("fixture target exists");
    screen.mouse_move(r.x + 4.0, r.y + r.height * 0.5);
    for _ in 0..2 {
        screen.mouse_down(MouseButton::Primary);
        screen.mouse_up(MouseButton::Primary);
    }
    assert_eq!(m.lane_calls.get_untracked(), [editor.lane(), editor.lane()]);
    let item = screen.get_by_role(Role::Button, editor.item_name());
    item.press_key(Key::Enter);
    item.double_click();
    assert_eq!(m.item_calls.get_untracked(), [editor.item(), editor.item()]);
    if !editor.graph {
        assert!(m.phases.get_untracked().is_empty());
    }
    let p = item.rect().expect("fixture target exists").center();
    if editor.graph {
        item.drag(p, Point::new(p.x + 10.0, p.y));
        assert!(m.phases.get_untracked().contains(&EditPhase::Commit));
    } else {
        screen.mouse_move(p.x, p.y);
        screen.mouse_down(MouseButton::Primary);
        screen.mouse_move(p.x + 12.0, p.y);
        screen.press_key(Key::Enter);
        screen.press_key(Key::ContextMenu);
        assert!(screen.query_by_role(Role::Menu, editor.name).is_none());
        screen.mouse_up(MouseButton::Primary);
        assert!(m.value() > 40.0);
    }
    assert_eq!(m.item_calls.get_untracked(), [editor.item(), editor.item()]);
    clean(&screen);
}
fn request(screen: &Screen, m: Fixture, item: Option<u64>, lane: Option<u64>, time: f64) {
    inspect(screen, m.editor.name, item, lane, time);
    assert_eq!(
        m.chosen
            .get_untracked()
            .last()
            .expect("fixture target exists")
            .0,
        ui::EditorContext { item, lane, time }
    );
}
pub(super) fn context(editor: Editor) {
    let (screen, m) = fixture(editor);
    let item = screen.get_by_role(Role::Button, editor.item_name());
    item.focus();
    item.press_key(Key::ContextMenu);
    request(
        &screen,
        m,
        Some(editor.item()),
        Some(editor.lane()),
        editor.time(),
    );
    assert!(item.is_focused());
    let shift = Modifiers {
        shift: true,
        ..Default::default()
    };
    if editor.graph {
        item.press_key_with(Key::F(10), shift);
        screen.press_key(Key::Escape);
        assert!(screen.query_by_role(Role::Menu, editor.name).is_none());
        assert!(item.is_focused());
        let p = screen
            .get_by_role(Role::Group, "Beta")
            .rect()
            .expect("fixture target exists")
            .center();
        screen.mouse_move(4.0, p.y);
        screen.mouse_down(MouseButton::Secondary);
        m.lanes.update(|v| v[1].label = "Changed".into());
        screen.run_until_idle();
        screen.mouse_up(MouseButton::Secondary);
        request(&screen, m, None, Some(2), 3.0);
    } else {
        let lane = screen.get_by_role(Role::Group, "Beta");
        lane.press_key_with(Key::F(10), shift);
        screen.press_key(Key::Escape);
        assert!(lane.is_focused());
    }
    // Preserve the graph's empty-space request before its item retiming request.
    if editor.graph {
        empty_request(&screen, m);
    }
    let p = item.rect().expect("fixture target exists").center();
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Secondary);
    m.retime();
    screen.run_until_idle();
    screen.mouse_up(MouseButton::Secondary);
    request(
        &screen,
        m,
        Some(editor.item()),
        Some(editor.lane()),
        editor.time(),
    );
    if !editor.graph {
        empty_request(&screen, m);
        assert_eq!(m.selected.get_untracked(), [1]);
        screen
            .get_by_role(Role::Canvas, editor.name)
            .a11y(A11yAction::ShowMenu);
        request(&screen, m, Some(1), Some(10), 45.0);
    } else {
        let lane = screen.get_by_role(Role::Group, "Changed");
        lane.press_key_with(Key::F(10), shift);
        screen.press_key(Key::Escape);
        assert!(lane.is_focused());
        m.selected.set(vec![22, 11]);
        let graph = screen.get_by_role(Role::Canvas, editor.name);
        graph.focus();
        graph.press_key(Key::ContextMenu);
        request(&screen, m, Some(22), Some(2), 8.0);
        m.selected.set(Vec::new());
        graph.press_key(Key::ContextMenu);
        request(&screen, m, None, None, 3.0);
    }
    assert!(m.phases.get_untracked().is_empty());
    clean(&screen);
}
fn empty_request(screen: &Screen, m: Fixture) {
    screen.mouse_move(450.0, if m.editor.graph { 290.0 } else { 250.0 });
    screen.mouse_down(MouseButton::Secondary);
    screen.mouse_up(MouseButton::Secondary);
    request(
        screen,
        m,
        None,
        None,
        if m.editor.graph { 5.0 } else { 120.0 },
    );
}
pub(super) fn constraints(editor: Editor) {
    let (screen, m) = fixture(editor);
    let before = m.value();
    m.disabled.set(true);
    screen.run_until_idle();
    let cell = screen.get_by_role(Role::Button, "Cell Alpha");
    assert!(cell.is_disabled());
    cell.force_click();
    if !editor.graph {
        screen
            .get_by_role(Role::Button, editor.item_name())
            .force_click();
        screen.mouse_move(350.0, 120.0);
        screen.mouse_down(MouseButton::Secondary);
        screen.mouse_up(MouseButton::Secondary);
    }
    assert_eq!(m.buttons.get_untracked(), 0);
    assert_eq!(m.value(), before);
    assert!(m.lane_calls.get_untracked().is_empty());
    if !editor.graph {
        assert!(m.item_calls.get_untracked().is_empty());
        assert!(screen.query_by_role(Role::Menu, editor.name).is_none());
    }
    m.disabled.set(false);
    m.hidden.set(true);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Button, "Cell Alpha").is_none());
    assert!(screen.query_by_role(Role::Menu, editor.name).is_none());
    if !editor.graph {
        assert!(screen.query_by_role(Role::Canvas, editor.name).is_none());
    }
    clean(&screen);
}
pub(super) fn failure(editor: Editor) {
    let (screen, m) = kivact::test::render_with_handle(crate::options(), move || {
        let m = Fixture::new(editor);
        let choose = callback(
            |_: (ui::EditorContext, &'static str)| -> kivact::Result<()> {
                Err(kivact::UiError::msg("command failed"))
            },
        );
        let entries = |_| vec![MenuEntry::item("fail", "Fail command")];
        let view = if editor.graph {
            app_graph(
                m.curves,
                ui::GraphState {
                    selection: m.selected.into(),
                    ..ui::GraphState::new()
                },
            )
            .context_menu(entries, move |value| choose.try_call(value))
            .name(editor.name)
            .style(style().size(700, 322))
            .into_view()
        } else {
            creative_time_fixture!(editor.keys, m.clips, m.keys, ui::TimeState { selection: m.selected.into(), ..ui::TimeState::new() },
                .context_menu(entries, move |value| choose.try_call(value))
                .name(editor.name)
                .style(style().size(700, 280))
            )
        };
        (m, view)
    });
    let before = m.snapshot();
    screen
        .get_by_role(Role::Button, editor.item_name())
        .press_key(Key::ContextMenu);
    screen.get_by_role(Role::MenuItem, "Fail command").click();
    screen.get_by_role(Role::Menu, editor.name);
    assert_eq!(m.snapshot(), before);
    assert_eq!(m.selected.get_untracked(), [editor.item()]);
    assert_eq!(screen.errors().len(), 1, "{:?}", screen.errors());
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Menu, editor.name).is_none());
}
