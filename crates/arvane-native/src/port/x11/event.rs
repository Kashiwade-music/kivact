//! Role: Normalizes X11 input, focus, geometry and XEmbed notifications.
use super::keyboard::Keyboard;
use super::window::{ChildRedraw, Window};
use arvane_shell::{ClickCounter, Mouse, ShellError, ShellEvent, ShellWindow};
use kivact_types::{MouseButton, Point, PointerPhase};
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{ConnectionExt, InputFocus};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum WindowFocus {
    Blurred,
    Focused,
}

pub(super) struct Input {
    mouse: Mouse,
    dnd: arvane_shell::pipeline::DropSession,
    clicks: ClickCounter,
    pub(super) keyboard: Keyboard,
    /// The window has the keyboard focus as last reported; reported again only on change.
    focused: WindowFocus,
}

impl Input {
    pub(super) fn new(keyboard: Keyboard) -> Self {
        Self {
            mouse: Mouse::default(),
            dnd: arvane_shell::pipeline::DropSession::default(),
            clicks: ClickCounter::default(),
            keyboard,
            focused: WindowFocus::Blurred,
        }
    }
    pub(super) fn translate(
        &mut self,
        event: Event,
        window: &mut Window,
    ) -> Result<Vec<ShellEvent>, ShellError> {
        let mut out = Vec::new();
        for raw in window.dnd.event(&window.native, &event, window.scale)? {
            // Xdnd supplies paths only after conversion; only its completed drop is delivered.
            let _ = self.dnd.input(raw);
        }
        out.extend(self.dnd.take_dropped());
        let filtered = window.ime.borrow_mut().filter(&event);
        out.extend(self.drain_ime(window));
        if filtered {
            return Ok(out);
        }

        let scale = window.scale;
        let point = |x: i16, y: i16| {
            Some(Point {
                x: x as f64 / scale,
                y: y as f64 / scale,
            })
        };
        match event {
            Event::Expose(_) => window.redraw.set(ChildRedraw::Needed),
            Event::ConfigureNotify(e) if e.window == window.native.id.get() => {
                // Only confirmed native geometry changes notify the handler.
                if (u32::from(e.width), u32::from(e.height)) != (window.width, window.height) {
                    window.resized(e.width.into(), e.height.into());
                    out.push(ShellEvent::Resized);
                }
            }
            Event::FocusIn(_) => self.focus_in(&mut out),
            Event::FocusOut(_) => self.blur(&mut out),
            Event::MotionNotify(e) => {
                self.sync_modifiers(e.state.into(), &mut out);
                out.extend(
                    self.mouse
                        .event(PointerPhase::Move, point(e.event_x, e.event_y))
                        .map(ShellEvent::Pointer),
                );
            }
            Event::EnterNotify(e) => {
                self.sync_modifiers(e.state.into(), &mut out);
                out.extend(
                    self.mouse
                        .event(PointerPhase::Move, point(e.event_x, e.event_y))
                        .map(ShellEvent::Pointer),
                );
            }
            Event::LeaveNotify(_) => out.extend(
                self.mouse
                    .event(PointerPhase::Leave, None)
                    .map(ShellEvent::Pointer),
            ),
            Event::ButtonPress(e) => {
                self.sync_modifiers(e.state.into(), &mut out);
                if (4..=7).contains(&e.detail) {
                    let (x, y) = match e.detail {
                        4 => (0.0, 1.0),
                        5 => (0.0, -1.0),
                        6 => (1.0, 0.0),
                        _ => (-1.0, 0.0),
                    };
                    let previous = self.mouse.position();
                    if let Some(event) = self
                        .mouse
                        .event(PointerPhase::Move, point(e.event_x, e.event_y))
                        && Some(event.position) != previous
                    {
                        out.push(ShellEvent::Pointer(event));
                    }
                    let settings = kivact_types::PointerSettings::default();
                    let (x, y) = crate::port::winit::wheel::lines(
                        x,
                        y,
                        u16::from(e.state) & 1 != 0,
                        window.size().height,
                        settings.wheel_lines,
                        settings.wheel_chars,
                    );
                    if let Some(position) = self.mouse.position() {
                        out.push(ShellEvent::Wheel(kivact_types::WheelInput {
                            delta: kivact_types::Point { x, y },
                            position,
                            modifiers: self.keyboard.modifiers(),
                        }));
                    }
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
                    out.extend(
                        self.mouse
                            .button(button, true, point(e.event_x, e.event_y))
                            .map(ShellEvent::Pointer),
                    );
                }
            }
            Event::ButtonRelease(e) => {
                self.sync_modifiers(e.state.into(), &mut out);
                if let Some(button) = button(e.detail) {
                    out.extend(
                        self.mouse
                            .button(button, false, point(e.event_x, e.event_y))
                            .map(ShellEvent::Pointer),
                    );
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
                    4 => self.focus_in(&mut out),
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
            if let ShellEvent::Pointer(sample) = event {
                sample.modifiers = self.keyboard.modifiers();
            }
            if let ShellEvent::Key(key) = event {
                key.pointer = self.mouse.window_position();
            }
            self.clicks
                .count(event, || now, || kivact_types::defaults::DOUBLE_CLICK);
        }
        Ok(out)
    }
    pub(super) fn drain_ime(&mut self, window: &Window) -> Vec<ShellEvent> {
        let (raw, returned, cancelled) = window.ime.borrow_mut().drain();
        let mut events: Vec<_> = raw
            .into_iter()
            .flat_map(|raw| window.ime_driver.borrow_mut().input(raw))
            .collect();
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
    fn focus_in(&mut self, out: &mut Vec<ShellEvent>) {
        if self.focused == WindowFocus::Blurred {
            self.focused = WindowFocus::Focused;
            out.push(ShellEvent::Focused(true));
        }
    }
    fn blur(&mut self, out: &mut Vec<ShellEvent>) {
        if self.focused == WindowFocus::Blurred {
            return;
        }
        self.focused = WindowFocus::Blurred;
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
