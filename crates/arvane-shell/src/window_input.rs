//! Role: One window's input normalization, shared by the native port, the virtual backend and injected input.

use crate::clicks::ClickCounter;
use crate::contacts::Contacts;
use crate::events::ShellEvent;
use crate::input::InjectedInput;
use crate::modifiers::ModifierState;
use crate::mouse::Mouse;
use crate::pipeline::{DropSession, ImeDriver, KeyPipeline, RawInput};
use kivact_types::{ButtonPress, Modifiers, Point, PointerPhase, WheelInput};

/// The width of one wheel line in logical pixels.
const WHEEL_LINE_PX: f64 = 20.0;

/// The input normalization of one window: the same for the native port, the
/// virtual backend and injected input. The IME driver stays with its owner
/// (the native window's `NativeInput`, the virtual window's input record).
pub struct WindowInput {
    mouse: Mouse,
    modifiers: ModifierState,
    clicks: ClickCounter,
    contacts: Contacts,
    keys: KeyPipeline,
    dnd: DropSession,
    /// The clock of the last event `platform` saw.
    last_now_ns: u64,
    /// The double-click settings of the last event `platform` saw.
    last_double_click: (u64, (f64, f64)),
}

impl Default for WindowInput {
    fn default() -> Self {
        Self {
            mouse: Mouse::default(),
            modifiers: ModifierState::default(),
            clicks: ClickCounter::default(),
            contacts: Contacts::default(),
            keys: KeyPipeline::default(),
            dnd: DropSession::default(),
            last_now_ns: 0,
            last_double_click: kivact_types::defaults::DOUBLE_CLICK,
        }
    }
}

impl WindowInput {
    /// The input normalization of a window that observed nothing yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Normalize one injected or raw input into the events to deliver, in order
    /// (each already passed through `platform`, so clicks are counted once).
    /// `double_click` is the platform's double-click time and distance, as
    /// `ClickCounter::count` takes it.
    pub fn normalize(
        &mut self,
        input: InjectedInput,
        now_ns: u64,
        double_click: (u64, (f64, f64)),
        ime: &mut ImeDriver,
    ) -> Vec<ShellEvent> {
        match input {
            InjectedInput::MouseMove(position) => self
                .mouse(PointerPhase::Move, Some(position), now_ns, double_click)
                .into_iter()
                .collect(),
            InjectedInput::MouseDown(button) => self
                .mouse(
                    PointerPhase::Down(ButtonPress { button, clicks: 1 }),
                    None,
                    now_ns,
                    double_click,
                )
                .into_iter()
                .collect(),
            InjectedInput::MouseUp(button) => self
                .mouse(
                    PointerPhase::Up(ButtonPress { button, clicks: 1 }),
                    None,
                    now_ns,
                    double_click,
                )
                .into_iter()
                .collect(),
            InjectedInput::MouseLeave => self
                .mouse(PointerPhase::Leave, None, now_ns, double_click)
                .into_iter()
                .collect(),
            InjectedInput::Wheel { lines_x, lines_y } => {
                self.wheel(lines_x, lines_y, now_ns, double_click)
            }
            InjectedInput::Modifiers(modifiers) => {
                self.modifiers_changed(modifiers).into_iter().collect()
            }
            InjectedInput::Raw(raw) => self.raw(raw, now_ns, double_click, ime),
            InjectedInput::Accessibility { .. } => Vec::new(),
        }
    }

    /// The mouse event of `phase` at `position` (`None`: where the mouse last was), as
    /// `Mouse::event` makes it, already passed through `platform`; `None` while the
    /// mouse has no position.
    pub fn mouse(
        &mut self,
        phase: PointerPhase,
        position: Option<Point>,
        now_ns: u64,
        double_click: (u64, (f64, f64)),
    ) -> Option<ShellEvent> {
        let sample = self.mouse.event(phase, position)?;
        Some(self.platform(ShellEvent::Pointer(sample), now_ns, double_click))
    }

    /// The platform's held modifiers: `ShellEvent::Modifiers` only when they changed
    /// (`ModifierState::update`).
    pub fn modifiers_changed(&mut self, modifiers: Modifiers) -> Option<ShellEvent> {
        self.modifiers.update(modifiers)
    }

    /// Stamp modifiers on any other platform event (key, wheel, touch, file, IME,
    /// Redraw, ...) and count its clicks, returning it to deliver. Call it once per event:
    /// what `mouse`, `normalize` and `focus_lost` return has passed through it already.
    pub fn platform(
        &mut self,
        mut event: ShellEvent,
        now_ns: u64,
        double_click: (u64, (f64, f64)),
    ) -> ShellEvent {
        self.last_now_ns = now_ns;
        self.last_double_click = double_click;
        match &mut event {
            ShellEvent::Pointer(sample) => {
                sample.modifiers = self.modifiers.current();
                self.mouse.observe(*sample);
                self.clicks.count(&mut event, || now_ns, || double_click);
            }
            ShellEvent::Modifiers(modifiers) => {
                let _ = self.modifiers.update(*modifiers);
            }
            _ => {}
        }
        event
    }

    /// The window lost focus: release keys, contacts and the mouse buttons, and
    /// return the release events to deliver.
    pub fn focus_lost(&mut self) -> Vec<ShellEvent> {
        let mut events = Vec::new();
        if self.modifiers.current() != Modifiers::default() {
            events.extend(self.modifiers.update(Modifiers::default()));
        }
        self.keys.focus_lost();
        let (now_ns, double_click) = (self.last_now_ns, self.last_double_click);
        events.extend(
            self.contacts
                .cancel()
                .into_iter()
                .map(|event| self.platform(event, now_ns, double_click)),
        );
        if let Some(sample) = self.mouse.cancel() {
            events.push(self.platform(ShellEvent::Pointer(sample), now_ns, double_click));
        }
        events
    }

    /// The mouse position inside the window (`Mouse::window_position`); `None` outside it.
    pub fn mouse_position(&self) -> Option<Point> {
        self.mouse.window_position()
    }

    /// The last mouse position inside or outside the window (`Mouse::position`).
    pub fn last_mouse_position(&self) -> Option<Point> {
        self.mouse.position()
    }

    /// The held modifier keys.
    pub fn modifiers(&self) -> Modifiers {
        self.modifiers.current()
    }

    /// The drop session the native port feeds platform drag events into.
    pub fn dnd_mut(&mut self) -> &mut DropSession {
        &mut self.dnd
    }

    /// The contact allocator the native port feeds touch events into.
    pub fn contacts_mut(&mut self) -> &mut Contacts {
        &mut self.contacts
    }

    /// The key pipeline the native port feeds keyboard events into.
    pub fn keys_mut(&mut self) -> &mut KeyPipeline {
        &mut self.keys
    }

    /// The wheel event of `lines_x` and `lines_y` at the last mouse position, if any.
    fn wheel(
        &mut self,
        lines_x: f64,
        lines_y: f64,
        now_ns: u64,
        double_click: (u64, (f64, f64)),
    ) -> Vec<ShellEvent> {
        let Some(position) = self.last_mouse_position() else {
            return Vec::new();
        };
        let event = ShellEvent::Wheel(WheelInput {
            delta: Point {
                x: lines_x * WHEEL_LINE_PX,
                y: lines_y * WHEEL_LINE_PX,
            },
            position,
            modifiers: self.modifiers(),
        });
        vec![self.platform(event, now_ns, double_click)]
    }

    /// The events one raw key, IME or file-drop input makes, passed through `platform`.
    fn raw(
        &mut self,
        raw: RawInput,
        now_ns: u64,
        double_click: (u64, (f64, f64)),
        ime: &mut ImeDriver,
    ) -> Vec<ShellEvent> {
        let events = match raw {
            RawInput::Key(key) => {
                let modifiers = self.modifiers();
                let pointer = self.mouse_position();
                vec![self.keys.key(key, modifiers, pointer)]
            }
            RawInput::Ime(input) => ime.input(input),
            RawInput::Drop(drop) => {
                let mut events = self.dnd.input(drop);
                events.extend(self.dnd.take_dropped());
                events
            }
        };
        events
            .into_iter()
            .map(|event| self.platform(event, now_ns, double_click))
            .collect()
    }
}
