//! Role: Turns winit window events into shell events for one native window.

use crate::port::winit::key_code::{map_key, physical_code};
use crate::port::winit::windows::NativeWindow;
use arvane_shell::pipeline::{DropSession, RawDrop, RawIme, RawKey};
use arvane_shell::{ShellEvent, ShellWindow};
use kivact_types::{Point, PointerPhase, WheelInput};
use winit::event::{ElementState, Ime, MouseScrollDelta, WindowEvent};

/// The events one winit window event delivers, in order.
pub(super) fn window_events(native: &mut NativeWindow, event: WindowEvent) -> Vec<ShellEvent> {
    native
        .window
        .accessibility
        .process_event(&native.window.window, &event);
    let scale = native.window.window.scale_factor();
    // The IME's character positions follow a scale change at once, even
    // when the text it answers from stays the same.
    native.window.input.scale.set(scale);
    let now = native.window.now_ns();
    let double_click = native.window.double_click();
    let (mouse, height) = (native.window.input.mouse.get(), native.window.size().height);
    let mut events = Vec::new();
    if let Some(before) = preceding_event(native, &event, now, double_click) {
        events.push(before);
    }
    events.extend(preceding_contacts(native, &event, scale, now, double_click));
    events.extend(input_event(
        native,
        event,
        scale,
        mouse,
        height,
        now,
        double_click,
    ));
    events
}

/// Contacts begin, move and end several at a time: a Down of a contact
/// whose end was lost cancels it first, and losing the focus cancels every
/// contact and, with a mouse button held, the mouse's gesture.
fn preceding_contacts(
    native: &mut NativeWindow,
    event: &WindowEvent,
    scale: f64,
    now: u64,
    double_click: (u64, (f64, f64)),
) -> Vec<ShellEvent> {
    match event {
        WindowEvent::Touch(touch) => {
            let notes = native.window.input.contacts.borrow();
            let events = native
                .input
                .contacts_mut()
                .deliver(super::contacts::map(*touch, scale, &notes));
            drop(notes);
            events
                .into_iter()
                .map(|event| native.input.platform(event, now, double_click))
                .collect()
        }
        WindowEvent::Focused(false) => native.input.focus_lost(now, double_click),
        _ => Vec::new(),
    }
}

/// The mouse and modifier events the window normalizes itself.
fn input_event(
    native: &mut NativeWindow,
    event: WindowEvent,
    scale: f64,
    mouse: kivact_types::PointerSettings,
    height: f64,
    now: u64,
    double_click: (u64, (f64, f64)),
) -> Vec<ShellEvent> {
    let mut events = Vec::new();
    match event {
        WindowEvent::CursorMoved { position, .. } => {
            let at = logical((position.x, position.y), scale);
            if matches!(native.input.dnd_mut(), DropSession::Hovering { .. }) {
                if let Some(event) = native
                    .input
                    .dnd_mut()
                    .input(RawDrop::Move(at))
                    .into_iter()
                    .next()
                {
                    events.push(native.input.platform(event, now, double_click));
                }
            } else if let Some(event) =
                native
                    .input
                    .mouse(PointerPhase::Move, Some(at), now, double_click)
            {
                events.push(event);
            }
        }
        WindowEvent::CursorLeft { .. } => {
            if let Some(event) = native
                .input
                .mouse(PointerPhase::Leave, None, now, double_click)
            {
                events.push(event);
            }
        }
        WindowEvent::MouseInput { state, button, .. } => {
            if let Some(button) = mouse_button(button) {
                let press = kivact_types::ButtonPress { button, clicks: 1 };
                let phase = if state == ElementState::Pressed {
                    PointerPhase::Down(press)
                } else {
                    PointerPhase::Up(press)
                };
                if let Some(event) = native.input.mouse(phase, None, now, double_click) {
                    events.push(event);
                }
            }
        }
        WindowEvent::ModifiersChanged(modifiers) => {
            let state = modifiers.state();
            let modifiers = kivact_types::Modifiers {
                shift: state.shift_key(),
                ctrl: state.control_key(),
                alt: state.alt_key(),
                meta: state.super_key(),
            };
            if let Some(event) = native.input.modifiers_changed(modifiers) {
                events.push(event);
            }
        }
        WindowEvent::Focused(false) => {
            events.push(
                native
                    .input
                    .platform(ShellEvent::Focused(false), now, double_click),
            );
        }
        event => events.extend(map_window_event(
            native,
            event,
            scale,
            mouse,
            height,
            now,
            double_click,
        )),
    }
    events
}

/// A winit mouse button as shell vocabulary.
fn mouse_button(button: winit::event::MouseButton) -> Option<kivact_types::MouseButton> {
    match button {
        winit::event::MouseButton::Left => Some(kivact_types::MouseButton::Primary),
        winit::event::MouseButton::Right => Some(kivact_types::MouseButton::Secondary),
        winit::event::MouseButton::Middle => Some(kivact_types::MouseButton::Middle),
        winit::event::MouseButton::Back => Some(kivact_types::MouseButton::Back),
        winit::event::MouseButton::Forward => Some(kivact_types::MouseButton::Forward),
        _ => None,
    }
}

/// Deliver the files dropped in this batch of winit events, after every
/// `DroppedFile` of it.
pub(super) fn deliver_drop(native: &mut NativeWindow) -> Vec<ShellEvent> {
    // Runs before every wait: query the system pointer (an X11 connection)
    // only when a drop is pending.
    if !matches!(native.input.dnd_mut(), DropSession::Dropped { .. }) {
        return Vec::new();
    }
    if let Some(position) = file_position(native) {
        let _ = native.input.dnd_mut().input(RawDrop::Move(position));
    }
    let Some(event) = native.input.dnd_mut().take_dropped() else {
        return Vec::new();
    };
    let now = native.window.now_ns();
    let double_click = native.window.double_click();
    vec![native.input.platform(event, now, double_click)]
}

/// The file drag's position, or where the mouse last was.
pub(super) fn file_position(native: &NativeWindow) -> Option<Point> {
    crate::native_input::file_position(&native.window.window)
        .or_else(|| native.input.last_mouse_position())
}

/// Translate the remaining winit window events, already passed through the
/// window's input normalization.
fn map_window_event(
    native: &mut NativeWindow,
    event: WindowEvent,
    scale: f64,
    mouse: kivact_types::PointerSettings,
    height: f64,
    now: u64,
    double_click: (u64, (f64, f64)),
) -> Vec<ShellEvent> {
    let event = match event {
        WindowEvent::RedrawRequested => Some(ShellEvent::Redraw),
        WindowEvent::CloseRequested => Some(ShellEvent::CloseRequested),
        WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
            Some(ShellEvent::Resized)
        }
        WindowEvent::Focused(focus) => Some(ShellEvent::Focused(focus)),
        WindowEvent::MouseWheel { delta, .. } => wheel_event(native, delta, scale, mouse, height),
        WindowEvent::KeyboardInput { event, .. } => Some(key_event(native, event)),
        WindowEvent::ThemeChanged(_) => Some(ShellEvent::EnvironmentChanged),
        event @ (WindowEvent::HoveredFile(_)
        | WindowEvent::HoveredFileCancelled
        | WindowEvent::DroppedFile(_)) => return file_event(native, event, now, double_click),
        WindowEvent::Ime(ime) => ime_event(native, ime),
        _ => None,
    };
    event
        .map(|event| native.input.platform(event, now, double_click))
        .into_iter()
        .collect()
}

/// The wheel event of `delta` at the last mouse position.
fn wheel_event(
    native: &mut NativeWindow,
    delta: MouseScrollDelta,
    scale: f64,
    mouse: kivact_types::PointerSettings,
    height: f64,
) -> Option<ShellEvent> {
    let (x, y) = match delta {
        MouseScrollDelta::LineDelta(x, y) => crate::port::winit::wheel::lines(
            f64::from(x),
            f64::from(y),
            native.input.modifiers().shift,
            height,
            mouse.wheel_lines,
            mouse.wheel_chars,
        ),
        MouseScrollDelta::PixelDelta(p) => ((-p.x / scale), (-p.y / scale)),
    };
    native.input.last_mouse_position().map(|position| {
        ShellEvent::Wheel(WheelInput {
            delta: Point { x, y },
            position,
            modifiers: native.input.modifiers(),
        })
    })
}

/// The key event of a physical and logical keyboard sample.
fn key_event(native: &mut NativeWindow, event: winit::event::KeyEvent) -> ShellEvent {
    let (modifiers, pointer) = (native.input.modifiers(), native.input.mouse_position());
    native.input.keys_mut().key(
        RawKey {
            key: arvane_shell::pipeline::PlatformKey::Code {
                key: map_key(&event.logical_key),
                code: physical_code(event.physical_key),
            },
            pressed: event.state == ElementState::Pressed,
            repeat: Some(event.repeat),
            text: event.text.map(|s| s.to_string()),
        },
        modifiers,
        pointer,
    )
}

/// The events of one file-drag protocol message.
fn file_event(
    native: &mut NativeWindow,
    event: WindowEvent,
    now: u64,
    double_click: (u64, (f64, f64)),
) -> Vec<ShellEvent> {
    let mut events = Vec::new();
    let event = match event {
        WindowEvent::HoveredFile(path) => {
            if matches!(native.input.dnd_mut(), DropSession::Dropped { .. }) {
                events.extend(deliver_drop(native));
            }
            let (mut files, last) = match native.input.dnd_mut() {
                DropSession::Hovering { files, at } => (files.clone(), *at),
                _ => (Vec::new(), None),
            };
            if !files.contains(&path) {
                files.push(path);
            }
            crate::native_input::track_files(&native.window.window, true);
            let at = file_position(native).or(last);
            native
                .input
                .dnd_mut()
                .input(RawDrop::Hover { files, at })
                .into_iter()
                .next()
        }
        WindowEvent::HoveredFileCancelled => {
            crate::native_input::track_files(&native.window.window, false);
            if let Some(position) = file_position(native) {
                let _ = native.input.dnd_mut().input(RawDrop::Move(position));
            }
            native
                .input
                .dnd_mut()
                .input(RawDrop::Leave)
                .into_iter()
                .next()
        }
        WindowEvent::DroppedFile(path) => {
            crate::native_input::track_files(&native.window.window, false);
            let (mut files, at) = match native.input.dnd_mut() {
                DropSession::Dropped { files, at } => (files.clone(), *at),
                DropSession::Hovering { at, .. } => (Vec::new(), *at),
                DropSession::Idle => (Vec::new(), None),
            };
            files.push(path);
            let _ = native.input.dnd_mut().input(RawDrop::Hover { files, at });
            let _ = native.input.dnd_mut().input(RawDrop::Drop);
            None
        }
        _ => None,
    };
    if let Some(event) = event {
        events.push(native.input.platform(event, now, double_click));
    }
    events
}

/// The events of one IME protocol message.
fn ime_event(native: &mut NativeWindow, ime: Ime) -> Option<ShellEvent> {
    let raw = match ime {
        Ime::Preedit(text, caret) => Some(RawIme::Preedit { text, caret }),
        Ime::Commit(text) => Some(RawIme::Commit(text)),
        Ime::Disabled => Some(RawIme::End),
        Ime::Enabled => None,
    };
    raw.and_then(|raw| {
        native
            .window
            .input
            .ime
            .borrow_mut()
            .input(raw)
            .into_iter()
            .next()
    })
}

/// Deliver native observations that precede the corresponding winit event.
fn preceding_event(
    native: &mut NativeWindow,
    event: &WindowEvent,
    now: u64,
    double_click: (u64, (f64, f64)),
) -> Option<ShellEvent> {
    // What the window's subclass learned that winit does not report,
    // delivered before the event it precedes.
    let mut before = None;
    match event {
        // A range the IME took for reconversion.
        WindowEvent::Ime(_) => {
            before = native.window.input.reconvert.take().map(|(start, end)| {
                native
                    .window
                    .input
                    .ime
                    .borrow_mut()
                    .input(RawIme::Reconvert {
                        selection: (start, end),
                    })
                    .remove(0)
            });
        }
        // winit reports buttons and wheels without a position: the
        // message's own, when it differs from the last move (a move
        // the platform reported late, after the button).
        WindowEvent::MouseInput { .. } | WindowEvent::MouseWheel { .. } => {
            let size = native.window.window.inner_size();
            let inside = |(x, y): (f64, f64)| {
                x >= 0.0 && y >= 0.0 && x < f64::from(size.width) && y < f64::from(size.height)
            };
            if let Some(at) = native.window.input.pointer.take().filter(|at| inside(*at)) {
                let at = logical(at, native.window.window.scale_factor());
                if native.input.mouse_position() != Some(at) {
                    before = native
                        .input
                        .mouse(PointerPhase::Move, Some(at), now, double_click);
                }
            }
        }
        // winit tells of no change of the locale, and only Windows has a
        // message for it (`WM_SETTINGCHANGE`): coming back to the window
        // is when the user may have changed it elsewhere.
        WindowEvent::Focused(true) => {
            let locale = crate::native_input::locale::system_locale();
            if native.window.input.locale.replace(locale.clone()) != locale {
                before = Some(ShellEvent::EnvironmentChanged);
            }
        }
        // Another monitor may refresh at another rate.
        WindowEvent::Moved(_) | WindowEvent::ScaleFactorChanged { .. } => {
            let monitor = native.window.window.current_monitor();
            if monitor != native.window.monitor {
                native.window.monitor = monitor;
                native.window.read_frame_period();
                before = Some(ShellEvent::EnvironmentChanged);
            }
        }
        _ => {}
    }
    before.map(|event| native.input.platform(event, now, double_click))
}

/// A physical client position in logical pixels.
fn logical((x, y): (f64, f64), scale: f64) -> Point {
    Point {
        x: (x / scale),
        y: (y / scale),
    }
}
