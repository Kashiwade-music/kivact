//! Role: Queue ordered platform input and environment or service prerequisites.

use crate::clock::Clock;
use arvane_shell::pipeline::{ImeDriver, RawInput};
use arvane_shell::{EventTarget, ShellError, ShellEvent, WindowInput};
use kivact_types::{A11yRequest, Insets, Point, PointerPhase, PointerSample, Size, WindowId};
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;
use std::rc::Rc;

pub(crate) type Responder =
    Box<dyn FnMut(&kivact_types::PlatformRequest) -> Option<kivact_types::PlatformResult>>;

/// One queued loop or window operation.
pub(crate) enum Item {
    Respond(Responder),
    Time(u64),
    Clipboard(String),
    Dialog(Option<Vec<PathBuf>>),
    FailEffect(ShellError),
    /// Delivered with `EventTarget::Loop`: SaveState, Suspended, Resumed, MemoryWarning.
    LoopEvent(ShellEvent),
    /// Sets every open window's `env.pointer`, then `EnvironmentChanged` to each (ascending id).
    Pointer(kivact_types::PointerSettings),
    Window(WindowId, WindowItem),
}

/// One queued operation on a single window.
pub(crate) enum WindowItem {
    Event(ShellEvent),
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
    Action(u64, A11yRequest),
}

/// The input normalization of one virtual window and its IME driver.
#[derive(Default)]
pub(crate) struct VirtualWindowInput {
    pub(crate) input: WindowInput,
    pub(crate) ime: ImeDriver,
}

/// The loop's input queue, clock and the input state of every open window.
#[derive(Default)]
pub(crate) struct VirtualInput {
    pub(crate) saved_state: Rc<RefCell<BTreeMap<String, Vec<u8>>>>,
    pub(crate) queue: VecDeque<Item>,
    pub(crate) clock: Clock,
    pub(crate) windows: BTreeMap<WindowId, VirtualWindowInput>,
}

/// Cloneable virtual platform controls and saved application state.
#[derive(Clone)]
pub struct Script(pub(crate) Rc<RefCell<VirtualInput>>);

impl Script {
    /// Saved application bytes to supply as the next launch snapshot.
    pub fn saved_state(&self) -> BTreeMap<String, Vec<u8>> {
        self.0.borrow().saved_state.borrow().clone()
    }
    /// Ask the application to save its current state.
    pub fn save_state(&self) {
        self.push(Item::LoopEvent(ShellEvent::SaveState));
    }
    /// Whether all queued platform prerequisites and inputs have been consumed.
    pub fn is_empty(&self) -> bool {
        self.0.borrow().queue.is_empty()
    }
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
    /// Update the platform pointer preferences and primary device class.
    pub fn pointer_settings(&self, settings: kivact_types::PointerSettings) {
        self.push(Item::Pointer(settings));
    }
    /// Queue one event as the main window does; loop events go to the loop.
    pub fn input(&self, event: ShellEvent) {
        if EventTarget::of(&event, WindowId::MAIN) == EventTarget::Loop {
            self.push(Item::LoopEvent(event));
        } else {
            self.window(WindowId::MAIN).input(event);
        }
    }
    /// The script of one window; items for a closed window are dropped.
    pub fn window(&self, id: WindowId) -> WindowScript {
        WindowScript {
            script: self.clone(),
            window: id,
        }
    }
    /// Advance the virtual clock and record the step in the input queue.
    pub fn set_now(&self, ns: u64) {
        let mut input = self.0.borrow_mut();
        input.clock.set(ns);
        let now = input.clock.now();
        input.queue.push_back(Item::Time(now));
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

/// The script operations of one window.
pub struct WindowScript {
    script: Script,
    window: WindowId,
}

impl WindowScript {
    fn push(&self, item: WindowItem) {
        self.script.push(Item::Window(self.window, item));
    }
    /// Queue one event for this window, normalized like platform input.
    pub fn input(&self, event: ShellEvent) {
        let mut input = self.script.0.borrow_mut();
        let input = &mut *input;
        let now = input.clock.now();
        let Some(state) = input.windows.get_mut(&self.window) else {
            return;
        };
        let mut events = Vec::new();
        if matches!(event, ShellEvent::Focused(false)) {
            events.extend(
                state
                    .input
                    .focus_lost(now, kivact_types::defaults::DOUBLE_CLICK),
            );
        }
        events.push(
            state
                .input
                .platform(event, now, kivact_types::defaults::DOUBLE_CLICK),
        );
        for event in events {
            input
                .queue
                .push_back(Item::Window(self.window, WindowItem::Event(event)));
        }
    }
    /// Queue one raw key, IME or file-drop input.
    pub fn raw(&self, raw: RawInput) {
        let mut input = self.script.0.borrow_mut();
        let input = &mut *input;
        let now = input.clock.now();
        let Some(state) = input.windows.get_mut(&self.window) else {
            return;
        };
        let events = state.input.normalize(
            arvane_shell::InjectedInput::Raw(raw),
            now,
            kivact_types::defaults::DOUBLE_CLICK,
            &mut state.ime,
        );
        for event in events {
            input
                .queue
                .push_back(Item::Window(self.window, WindowItem::Event(event)));
        }
    }
    /// Queue a mouse event of `phase` at `position`.
    pub fn mouse(&self, phase: PointerPhase, position: Option<Point>) {
        let mut input = self.script.0.borrow_mut();
        let input = &mut *input;
        let now = input.clock.now();
        let Some(state) = input.windows.get_mut(&self.window) else {
            return;
        };
        if let Some(event) =
            state
                .input
                .mouse(phase, position, now, kivact_types::defaults::DOUBLE_CLICK)
        {
            input
                .queue
                .push_back(Item::Window(self.window, WindowItem::Event(event)));
        }
    }
    /// Queue one touch or pen contact.
    pub fn contact(&self, contact: PointerSample) {
        let mut input = self.script.0.borrow_mut();
        let input = &mut *input;
        let now = input.clock.now();
        let Some(state) = input.windows.get_mut(&self.window) else {
            return;
        };
        let events: Vec<ShellEvent> = state
            .input
            .contacts_mut()
            .deliver(contact)
            .into_iter()
            .map(|event| {
                state
                    .input
                    .platform(event, now, kivact_types::defaults::DOUBLE_CLICK)
            })
            .collect();
        for event in events {
            input
                .queue
                .push_back(Item::Window(self.window, WindowItem::Event(event)));
        }
    }
    /// Queue a focus change; losing the focus releases the held input first.
    pub fn focus(&self, focused: bool) {
        let mut input = self.script.0.borrow_mut();
        let input = &mut *input;
        let now = input.clock.now();
        let Some(state) = input.windows.get_mut(&self.window) else {
            return;
        };
        let mut events = Vec::new();
        if !focused {
            events.extend(
                state
                    .input
                    .focus_lost(now, kivact_types::defaults::DOUBLE_CLICK),
            );
        }
        events.push(state.input.platform(
            ShellEvent::Focused(focused),
            now,
            kivact_types::defaults::DOUBLE_CLICK,
        ));
        for event in events {
            input
                .queue
                .push_back(Item::Window(self.window, WindowItem::Event(event)));
        }
    }
    /// Queue an assistive-technology request for one node.
    pub fn accessibility_action(&self, node: u64, request: A11yRequest) {
        self.push(WindowItem::Action(node, request));
    }
    pub fn resize(&self, size: Size) {
        self.push(WindowItem::Resize(size));
    }
    pub fn set_scale(&self, scale: f64) {
        self.push(WindowItem::Scale(scale));
    }
    pub fn set_frame_period(&self, ns: u64) {
        self.push(WindowItem::Period(ns.max(1)));
    }
    pub fn set_reduced_motion(&self, value: bool) {
        self.push(WindowItem::Reduced(value));
    }
    pub fn set_dark(&self, value: bool) {
        self.push(WindowItem::Dark(value));
    }
    pub fn set_locale(&self, locale: Option<&str>) {
        self.push(WindowItem::Locale(locale.map(str::to_owned)));
    }
    pub fn set_safe_area(&self, insets: Insets) {
        self.push(WindowItem::SafeArea(insets));
    }
    pub fn set_keyboard_inset(&self, insets: Insets) {
        self.input(ShellEvent::KeyboardInset(insets));
    }
    pub fn create_surface(&self) {
        self.push(WindowItem::Surface(true));
    }
    pub fn destroy_surface(&self) {
        self.push(WindowItem::Surface(false));
    }
    pub fn set_occluded(&self, value: bool) {
        self.push(WindowItem::Occluded(value));
    }
    pub fn fail_presents(&self, count: u32) {
        self.push(WindowItem::FailPresents(count));
    }
}
