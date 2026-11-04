//! Role: Node canvas pointer, keyboard and drag finalization.
use crate::editors::node::gesture::Target;
use crate::editors::node::gesture::{DragMode, NodeMode, State};
use crate::editors::node::layout as geometry;
use crate::editors::node::model::{NodeControlValue, NodeLink, NodeRequest};
use crate::editors::shared::selection;
use kivact::draw::{Point, Rect};
use kivact::interaction::{Drag as Gesture, ListGesture};
use kivact::view::{ButtonPress, Handled, Key, KeyEvent, MouseButton, PointerEvent, PointerPhase};

fn moved(state: State, point: Point) {
    let start = state.point(state.press.origin());
    match state.drag_state() {
        Some(DragMode::Items { origins }) => {
            let snap = |d: f64| {
                if state.snap.get() {
                    (d / 10.0).round() * 10.0
                } else {
                    d
                }
            };
            let delta = Point::new(snap(point.x - start.x), snap(point.y - start.y));
            if delta.x == 0.0 && delta.y == 0.0 && !state.edit.is_active() {
                return;
            }
            state.press.set_pending(None);
            state.gesture.change(|| {
                state.request(NodeRequest::Move(
                    origins
                        .into_iter()
                        .map(|(id, origin)| {
                            (id, Point::new(origin.x + delta.x, origin.y + delta.y))
                        })
                        .collect(),
                ))
            });
        }
        Some(DragMode::Control {
            node: id,
            index,
            slot,
        }) => {
            let dx = point.x - start.x;
            if dx == 0.0 && !state.edit.is_active() {
                return;
            }
            if let NodeControlValue::Number { value, step, .. } = slot.value {
                state
                    .gesture
                    .change(|| state.change_raw(id, index, Some(value + dx * step), 0.0));
            }
        }
        Some(DragMode::Link { .. }) => state.press.update_payload(|payload| {
            if let DragMode::Link { to, .. } = payload {
                *to = point;
            }
        }),
        Some(DragMode::Marquee { .. }) => state.press.update_payload(|payload| {
            if let DragMode::Marquee { to, .. } = payload {
                *to = point;
            }
        }),
        None => {}
    }
}
/// The press decides what the editor's drag does; the drag's sources (the
/// canvas and its regions) then follow the pointer until it ends. A press
/// that starts no drag is consumed here, so the drag does not see it.
pub(crate) fn pointer(state: State, event: PointerEvent) -> Handled {
    if state.blocked.get() {
        return Handled::No;
    }
    if state.zoom.is_gesturing().get_untracked() {
        return Handled::No;
    }
    if event.phase().press().map(|press| press.button) == Some(MouseButton::Middle) {
        return if state.busy() {
            Handled::Yes
        } else {
            Handled::No
        };
    }
    if matches!(
        event.phase(),
        PointerPhase::Down(ButtonPress {
            button: MouseButton::Primary,
            ..
        })
    ) && state.zoom.space_armed().get_untracked()
    {
        return if state.busy() {
            Handled::Yes
        } else {
            Handled::No
        };
    }
    let point = state.point(event.pos());
    let target = event
        .region()
        .and_then(|id| state.scene.get().ok()?.target(id).copied());
    match event.phase() {
        PointerPhase::Down(ButtonPress {
            button: MouseButton::Primary,
            ..
        }) => {
            if state.busy() {
                return Handled::Yes;
            }
            let mut drag = None;
            let mut pending = None;
            match target {
                Some(Target::Item(id)) => {
                    state.active.set(None);
                    state.mode.set(NodeMode::Idle);
                    match selection::press(
                        &state.selected.get(),
                        id,
                        event.modifiers(),
                        &kivact::interaction::ListPolicy::default(),
                    ) {
                        selection::Press::Pending(id) => pending = Some(id),
                        selection::Press::Select(gesture) => state.select(id, gesture),
                    }
                    drag = Some(DragMode::Items {
                        origins: state
                            .data()
                            .get()
                            .iter()
                            .filter(|n| state.selected.get().contains(&n.id))
                            .map(|n| (n.id, n.position))
                            .collect(),
                    });
                }
                Some(Target::Control(id, index, _)) => {
                    state.active.set(Some((id, index)));
                    if let Some(control) = state.control(id, index)
                        && matches!(control.value, NodeControlValue::Number { .. })
                    {
                        drag = Some(DragMode::Control {
                            node: id,
                            index,
                            slot: control,
                        });
                    }
                }
                Some(Target::Socket(id, true, index)) => {
                    drag = Some(DragMode::Link {
                        from: id,
                        output: index,
                        to: point,
                        rewire: None,
                        released: None,
                    });
                }
                Some(Target::Socket(id, false, index)) => {
                    if let Some(link) = state
                        .links
                        .get()
                        .iter()
                        .find(|l| l.to == id && l.input == index)
                        .copied()
                    {
                        drag = Some(DragMode::Link {
                            from: link.from,
                            output: link.output,
                            to: point,
                            rewire: Some(link),
                            released: None,
                        });
                    }
                }
                Some(target @ Target::Link(..)) => state.invoke(target, false),
                None => {
                    let link = state.scene.get().ok().and_then(|scene| {
                        geometry::hit_link(&state.data().get(), &state.links.get(), point)
                            .filter(|l| scene.live(&Target::link(*l)))
                    });
                    if let Some(link) = link {
                        state.invoke(Target::link(link), false);
                    } else {
                        drag = Some(DragMode::Marquee {
                            from: point,
                            to: point,
                        });
                        state.selected_link.set(None);
                    }
                }
            }
            let Some(drag) = drag else {
                return Handled::Yes;
            };
            state.press.begin(&event, target, drag, pending);
            Handled::No
        }
        PointerPhase::Up(_) => {
            // A link ends on the socket under the release.
            let released = match target {
                Some(Target::Socket(id, false, index)) => Some((id, index)),
                _ => None,
            };
            state.press.update_payload(|payload| {
                if let DragMode::Link { released: end, .. } = payload {
                    *end = released;
                }
            });
            Handled::No
        }
        _ => Handled::No,
    }
}
/// The editor's drag: the primary gesture the press chose, one edit from its
/// first change. Space and the middle button pan the view instead. The
/// returned drag has the callbacks its sources need; `state.gesture` is the
/// same session.
pub(crate) fn drag(state: State) -> Gesture<()> {
    state.press.install(
        state.gesture,
        move || matches!(state.drag_state(), Some(DragMode::Marquee { .. })),
        (
            move || state.selected.get(),
            move |ids| state.selected.set(ids),
            move |id| state.select(id, ListGesture::Replace),
        ),
        (
            move |pos| moved(state, state.point(pos)),
            move |rect| {
                let region = Rect::from_points(
                    state.point(Point::new(rect.x, rect.y)),
                    state.point(Point::new(rect.right(), rect.bottom())),
                );
                state
                    .data()
                    .get()
                    .iter()
                    .filter(|node| region.intersects(geometry::rect(node)))
                    .map(|node| node.id)
                    .collect()
            },
            move |target| {
                if let Target::Control(id, index, _) = target {
                    state.open(id, index);
                }
            },
            move |finished| {
                if finished {
                    finish_fields(state);
                } else {
                    state.reset_fields();
                }
            },
        ),
    )
}
fn finish_fields(state: State) {
    let point = state.point(state.press.current());
    if let Some(DragMode::Link {
        from,
        output,
        rewire,
        released,
        ..
    }) = state.drag_state()
    {
        let input = match released {
            Some(socket) => Some(socket),
            None => state.data().with(|nodes| {
                nodes.iter().find_map(|node| {
                    (0..node.inputs.len())
                        .find(|index| {
                            let socket = geometry::socket(node, false, *index);
                            (point.x - socket.x).abs() < 15.0 && (point.y - socket.y).abs() < 15.0
                        })
                        .map(|index| (node.id, index))
                })
            }),
        };
        let add = input.map(|(to, input)| NodeLink::new(from, output, to, input));
        state.write_links(rewire, add);
    }
}
pub(crate) fn key(state: State, event: KeyEvent) -> Handled {
    if event.key() == Key::Escape {
        if state.zoom.is_gesturing().get_untracked() {
            return Handled::No;
        }
        // The drag's sources take Escape during a drag; a control's edit
        // from the parameter popup ends here.
        if state.edit.is_active() {
            state.cancel();
            return Handled::Yes;
        }
        state.cancel();
        state.choose.clear();
        state.selected_link.set(None);
        return Handled::Yes;
    }
    if state.zoom.is_gesturing().get_untracked() {
        if event.key() == Key::Tab {
            return Handled::No;
        }
        return if matches!(event.key(), Key::Char('+' | '-' | '0') | Key::Space) {
            Handled::No
        } else {
            Handled::Yes
        };
    }
    if state.busy() {
        return if event.key() == Key::Tab {
            Handled::No
        } else {
            Handled::Yes
        };
    }
    match event.key() {
        Key::Delete | Key::Backspace => {
            if let Some(link) = state.selected_link.get_untracked() {
                state.links_edit(Some(link), None);
                state.selected_link.set(None);
            } else {
                let selected = state.selected.get_untracked();
                let removed = state
                    .data()
                    .get_untracked()
                    .into_iter()
                    .filter(|n| selected.contains(&n.id))
                    .collect::<Vec<_>>();
                if !removed.is_empty() {
                    let links = state
                        .links
                        .get_untracked()
                        .into_iter()
                        .filter(|l| selected.contains(&l.from) || selected.contains(&l.to))
                        .collect();
                    state.edit.once((), || {
                        state.request(NodeRequest::Delete {
                            nodes: selected,
                            links,
                        })
                    });
                }
                state.choose.clear();
            }
        }
        _ if selection::select_all(&event) => state.choose.select_all(),
        Key::Char('m') => {
            let selected = state.selected.get_untracked();
            let old = state
                .data()
                .get_untracked()
                .iter()
                .filter(|n| selected.contains(&n.id))
                .map(|n| (n.id, n.muted))
                .collect::<Vec<_>>();
            if !old.is_empty() {
                state
                    .edit
                    .once((), || state.request(NodeRequest::ToggleMute(selected)));
            }
        }
        Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown => {
            if let Some((id, index)) = state.active.get_untracked() {
                state.change(
                    id,
                    index,
                    None,
                    if matches!(event.key(), Key::ArrowLeft | Key::ArrowDown) {
                        -1.0
                    } else {
                        1.0
                    },
                );
            } else {
                let selected = state.selected.get_untracked();
                let positions = state
                    .data()
                    .get_untracked()
                    .iter()
                    .filter(|n| selected.contains(&n.id))
                    .map(|n| (n.id, n.position))
                    .collect::<Vec<_>>();
                let step = if event.modifiers().shift { 10.0 } else { 1.0 };
                if !positions.is_empty() {
                    let next = positions
                        .into_iter()
                        .map(|(id, mut point)| {
                            match event.key() {
                                Key::ArrowLeft => point.x -= step,
                                Key::ArrowRight => point.x += step,
                                Key::ArrowUp => point.y -= step,
                                _ => point.y += step,
                            }
                            (id, point)
                        })
                        .collect();
                    state
                        .edit
                        .once((), || state.request(NodeRequest::Move(next)));
                }
            }
        }
        Key::Enter => {
            if let Some(target @ Target::Socket(..)) = event
                .region()
                .and_then(|region| state.scene.get_untracked().ok()?.target(region).copied())
            {
                state.invoke(target, false);
            } else if let Some((id, index)) = state.active.get_untracked() {
                state.open(id, index);
            } else {
                return Handled::No;
            }
        }
        _ => return Handled::No,
    }
    Handled::Yes
}
