//! Role: Normalizes X11 input, focus, geometry and XEmbed notifications.

use super::keyboard::Keyboard;
use super::window::Window;
use arvane_shell::{ClickCounter, Mouse, ShellError, ShellEvent, ShellWindow};
use kivact_types::{MouseButton, Point, PointerPhase};
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{ConnectionExt, InputFocus};

pub(super) struct Input {
    mouse: Mouse,
    clicks: ClickCounter,
    pub(super) keyboard: Keyboard,
}

impl Input {
    pub(super) fn new(keyboard: Keyboard) -> Self {
        Self {
            mouse: Mouse::default(),
            clicks: ClickCounter::default(),
            keyboard,
        }
    }
    pub(super) fn translate(
        &mut self,
        event: Event,
        window: &mut Window,
    ) -> Result<Vec<ShellEvent>, ShellError> {
        let mut out = window.dnd.event(&window.native, &event, window.scale)?;
        let filtered = window.ime.borrow_mut().filter(&event);
        out.extend(self.drain_ime(window));
        if filtered {
            return Ok(out);
        }

        let scale = window.scale;
        let point = |x: i16, y: i16| Some(Point::new(x as f32 / scale, y as f32 / scale));
        match event {
            Event::Expose(_) => window.redraw.set(true),
            Event::ConfigureNotify(e) if e.window == window.native.id.get() => {
                // Only confirmed native geometry changes notify the handler.
                if (u32::from(e.width), u32::from(e.height)) != (window.width, window.height) {
                    window.resized(e.width.into(), e.height.into());
                    out.push(ShellEvent::Resized);
                }
            }
            Event::FocusIn(_) => out.push(ShellEvent::Focused(true)),
            Event::FocusOut(_) => self.blur(&mut out),
            Event::MotionNotify(e) => {
                self.sync_modifiers(e.state.into(), &mut out);
                out.push(ShellEvent::Pointer(self.mouse.event(
                    PointerPhase::Move,
                    point(e.event_x, e.event_y),
                    None,
                )));
            }
            Event::EnterNotify(e) => {
                self.sync_modifiers(e.state.into(), &mut out);
                out.push(ShellEvent::Pointer(self.mouse.event(
                    PointerPhase::Move,
                    point(e.event_x, e.event_y),
                    None,
                )));
            }
            Event::LeaveNotify(_) => out.push(ShellEvent::Pointer(self.mouse.event(
                PointerPhase::Leave,
                None,
                None,
            ))),
            Event::ButtonPress(e) => {
                self.sync_modifiers(e.state.into(), &mut out);
                if (4..=7).contains(&e.detail) {
                    let (x, y) = match e.detail {
                        4 => (0.0, 1.0),
                        5 => (0.0, -1.0),
                        6 => (1.0, 0.0),
                        _ => (-1.0, 0.0),
                    };
                    let previous = self.mouse.event(PointerPhase::Move, None, None).position;
                    let event =
                        self.mouse
                            .event(PointerPhase::Move, point(e.event_x, e.event_y), None);
                    if event.position != previous {
                        out.push(ShellEvent::Pointer(event));
                    }
                    let settings = kivact_types::protocol::MouseSettings::default();
                    let (x, y) = crate::wheel::lines(
                        x,
                        y,
                        u16::from(e.state) & 1 != 0,
                        window.size().height,
                        settings.wheel_lines,
                        settings.wheel_chars,
                    );
                    out.push(ShellEvent::Wheel { x, y });
                } else if let Some(button) = button(e.detail) {
                    // Wheel buttons must not move keyboard focus; only real buttons do.
                    window
                        .native
                        .connection
                        .set_input_focus(InputFocus::PARENT, window.native.id.get(), e.time)
                        .map_err(super::error::platform)?;
                    window
                        .native
                        .connection
                        .flush()
                        .map_err(super::error::platform)?;
                    out.push(ShellEvent::Pointer(self.mouse.event(
                        PointerPhase::Down,
                        point(e.event_x, e.event_y),
                        Some(button),
                    )));
                }
            }
            Event::ButtonRelease(e) => {
                self.sync_modifiers(e.state.into(), &mut out);
                if let Some(button) = button(e.detail) {
                    out.push(ShellEvent::Pointer(self.mouse.event(
                        PointerPhase::Up,
                        point(e.event_x, e.event_y),
                        Some(button),
                    )));
                }
            }
            Event::KeyPress(e) => {
                if !window.ime.borrow_mut().forward(&e, true) {
                    out.extend(self.keyboard.key(e.detail, e.state.into(), true));
                }
            }
            Event::KeyRelease(e) => {
                if !window.ime.borrow_mut().forward(&e, false) {
                    out.extend(self.keyboard.key(e.detail, e.state.into(), false));
                }
            }
            Event::MappingNotify(_) | Event::XkbNewKeyboardNotify(_) | Event::XkbMapNotify(_) => {
                self.keyboard.reload(&window.native.connection)
            }
            Event::ClientMessage(e) if e.type_ == window.native.xembed && e.format == 32 => {
                match e.data.as_data32()[1] {
                    // Only XEMBED_FOCUS_IN moves focus; WINDOW_ACTIVATE (1) does not.
                    4 => out.push(ShellEvent::Focused(true)),
                    2 | 5 => self.blur(&mut out),
                    _ => {}
                }
            }
            _ => {}
        }
        for event in &out {
            if let ShellEvent::Focused(focused) = event {
                window.accessibility.focus(*focused);
                window.ime.borrow_mut().focus(*focused);
            }
        }
        let now = window.now_ns();
        for event in &mut out {
            self.clicks
                .count(event, || now, || arvane_shell::DEFAULT_DOUBLE_CLICK);
        }
        Ok(out)
    }
    pub(super) fn drain_ime(&mut self, window: &Window) -> Vec<ShellEvent> {
        let (mut events, returned, cancelled) = window.ime.borrow_mut().drain();
        // Reconcile old releases before a new context's returned press can
        // reuse the same physical detail. These keys never go back to XIM.
        for key in cancelled {
            events.extend(
                self.keyboard
                    .release_if_pressed(key.detail, key.state.into()),
            );
        }
        for key in returned {
            events.extend(self.keyboard.key(
                key.detail,
                key.state.into(),
                key.response_type & 0x7f == x11rb::protocol::xproto::KEY_PRESS_EVENT,
            ));
        }
        events
    }
    // Pointer events carry the modifier state; report a change before the
    // pointer event so a modifier changed while another client had focus is seen.
    fn sync_modifiers(&mut self, mask: u16, out: &mut Vec<ShellEvent>) {
        if let Some(event) = self.keyboard.sync_modifiers(mask) {
            out.push(event);
        }
    }
    fn blur(&mut self, out: &mut Vec<ShellEvent>) {
        if let Some(cancel) = self.mouse.cancel() {
            out.push(ShellEvent::Pointer(cancel));
        }
        self.keyboard.focus_lost();
        out.push(ShellEvent::Modifiers(kivact_types::Modifiers::default()));
        out.push(ShellEvent::Focused(false));
    }
}

fn button(detail: u8) -> Option<MouseButton> {
    Some(match detail {
        1 => MouseButton::Primary,
        2 => MouseButton::Middle,
        3 => MouseButton::Secondary,
        8 => MouseButton::Back,
        9 => MouseButton::Forward,
        _ => return None,
    })
}
