//! Role: Queue ordered platform input and environment or service prerequisites.

use crate::clock::Clock;
use arvane_shell::{ClickCounter, Contacts, Mouse, ShellError, ShellEvent};
use kivact_types::{A11yRequest, Insets, Modifiers, Point, PointerPhase, PointerSample, Size};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::rc::Rc;

pub(crate) type Responder =
    Box<dyn FnMut(&kivact_types::PlatformRequest) -> Option<kivact_types::PlatformResult>>;

pub(crate) enum Item {
    Respond(Responder),
    Event(ShellEvent),
    Time(u64),
    Resize(Size),
    Scale(f64),
    Period(u64),
    Reduced(bool),
    Dark(bool),
    Locale(Option<String>),
    SafeArea(Insets),
    Surface(bool),
    Occluded(bool),
    FailPresents(u32),
    Clipboard(String),
    Dialog(Option<Vec<PathBuf>>),
    FailEffect(ShellError),
    Action(u64, A11yRequest),
}
#[derive(Default)]
pub(crate) struct Input {
    pub(crate) queue: VecDeque<Item>,
    pub(crate) clock: Clock,
    pub(crate) mouse: Mouse,
    pub(crate) modifiers: Modifiers,
    clicks: ClickCounter,
    contacts: Contacts,
    keys: arvane_shell::pipeline::KeyPipeline,
    ime: arvane_shell::pipeline::ImeDriver,
    dnd: arvane_shell::pipeline::DropSession,
}
/// Cloneable write-only controls for the virtual platform.
#[derive(Clone)]
pub struct Script(pub(crate) Rc<RefCell<Input>>);
impl Script {
    /// Override requests; the newest responder returning a result wins.
    pub fn respond(
        &self,
        f: impl FnMut(&kivact_types::PlatformRequest) -> Option<kivact_types::PlatformResult> + 'static,
    ) {
        self.push(Item::Respond(Box::new(f)));
    }
    fn push(&self, item: Item) {
        self.0.borrow_mut().queue.push_back(item);
    }
    pub fn input(&self, event: ShellEvent) {
        let mut input = self.0.borrow_mut();
        enqueue(&mut input, event);
    }
    pub fn raw(&self, raw: arvane_shell::pipeline::RawInput) {
        let mut input = self.0.borrow_mut();
        let modifiers = input.modifiers;
        let pointer = input.mouse.window_position();
        let events = match raw {
            arvane_shell::pipeline::RawInput::Key(key) => {
                vec![input.keys.key(key, modifiers, pointer)]
            }
            arvane_shell::pipeline::RawInput::Ime(ime) => input.ime.input(ime),
            arvane_shell::pipeline::RawInput::Drop(drop) => {
                let mut events = input.dnd.input(drop);
                events.extend(input.dnd.take_dropped());
                events
            }
        };
        for event in events {
            enqueue(&mut input, event);
        }
    }
    pub fn mouse(&self, phase: PointerPhase, position: Option<Point>) {
        let mut input = self.0.borrow_mut();
        if let Some(mut event) = input.mouse.event(phase, position) {
            event.modifiers = input.modifiers;
            enqueue(&mut input, ShellEvent::Pointer(event));
        }
    }
    pub fn contact(&self, contact: PointerSample) {
        let mut input = self.0.borrow_mut();
        for event in input.contacts.deliver(contact) {
            enqueue(&mut input, event);
        }
    }
    pub fn focus(&self, focused: bool) {
        let mut input = self.0.borrow_mut();
        if !focused {
            for event in input.contacts.cancel() {
                enqueue(&mut input, event);
            }
            if let Some(cancel) = input.mouse.cancel() {
                enqueue(&mut input, ShellEvent::Pointer(cancel));
            }
        }
        enqueue(&mut input, ShellEvent::Focused(focused));
    }
    pub fn accessibility_action(&self, node: u64, request: A11yRequest) {
        self.push(Item::Action(node, request));
    }
    pub fn set_now(&self, ns: u64) {
        let mut input = self.0.borrow_mut();
        input.clock.set(ns);
        let now = input.clock.now();
        input.queue.push_back(Item::Time(now));
    }
    pub fn resize(&self, size: Size) {
        self.push(Item::Resize(size));
    }
    pub fn set_scale(&self, scale: f64) {
        self.push(Item::Scale(scale));
    }
    pub fn set_frame_period(&self, ns: u64) {
        self.push(Item::Period(ns.max(1)));
    }
    pub fn set_reduced_motion(&self, value: bool) {
        self.push(Item::Reduced(value));
    }
    pub fn set_dark(&self, value: bool) {
        self.push(Item::Dark(value));
    }
    pub fn set_locale(&self, locale: Option<&str>) {
        self.push(Item::Locale(locale.map(str::to_owned)));
    }
    pub fn set_safe_area(&self, insets: Insets) {
        self.push(Item::SafeArea(insets));
    }
    pub fn create_surface(&self) {
        self.push(Item::Surface(true));
    }
    pub fn destroy_surface(&self) {
        self.push(Item::Surface(false));
    }
    pub fn set_occluded(&self, value: bool) {
        self.push(Item::Occluded(value));
    }
    pub fn fail_presents(&self, count: u32) {
        self.push(Item::FailPresents(count));
    }
    pub fn set_clipboard(&self, text: &str) {
        self.push(Item::Clipboard(text.to_owned()));
    }
    pub fn push_file_dialog(&self, path: Option<PathBuf>) {
        self.push(Item::Dialog(path.map(|path| vec![path])));
    }
    pub fn push_files_dialog(&self, paths: Option<Vec<PathBuf>>) {
        self.push(Item::Dialog(paths));
    }
    pub fn fail_next_effect(&self, reason: &str) {
        self.push(Item::FailEffect(ShellError::new(reason)));
    }
    pub fn fail_next_effect_unsupported(&self, reason: &str) {
        self.push(Item::FailEffect(ShellError::unsupported(reason)));
    }
}
fn enqueue(input: &mut Input, mut event: ShellEvent) {
    if let ShellEvent::Modifiers(modifiers) = &event {
        input.modifiers = *modifiers;
    }
    if let ShellEvent::Pointer(pointer) = &event {
        input.mouse.observe(*pointer);
    }
    if matches!(event, ShellEvent::Focused(false)) {
        input.modifiers = Modifiers::default();
        input.keys.focus_lost();
        let _ = input.mouse.cancel();
    }
    let now = input.clock.now();
    input
        .clicks
        .count(&mut event, || now, || kivact_types::defaults::DOUBLE_CLICK);
    input.queue.push_back(Item::Event(event));
}
