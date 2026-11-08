//! Role: Start and advance the virtual backend and expose owned observations.

use crate::clock::Clock;
use crate::script::{Item, Script, VirtualInput, VirtualWindowInput, WindowItem};
use crate::surface::TestSurface;
use crate::window::{Display, VirtualPlatform, VirtualWindow};
pub use crate::window::{GpuMode, MirrorMode, VirtualOptions};
use arvane_shell::{
    Accessibility, InjectedInput, ShellControl, ShellError, ShellEvent, ShellHandler, ShellImeText,
    ShellLoop, ShellWaker, ShellWindow, Step, StepOutcome, SteppedBackend, SteppedLoop,
    WakeReasons,
};
use kivact_types::render::SemanticsTree;
use kivact_types::{
    Capabilities, Cursor, Environment, Modifiers, Point, Rect, RequestId, Size, ThemeHint,
    WindowId, WindowRequest,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Backend whose input and clock are advanced explicitly.
pub struct VirtualBackend {
    options: VirtualOptions,
    platform: Rc<VirtualPlatform>,
    windows: Rc<RefCell<BTreeMap<WindowId, VirtualWindow>>>,
    input: Rc<RefCell<VirtualInput>>,
    woken: Arc<WakePending>,
}
impl VirtualBackend {
    pub fn new(options: VirtualOptions) -> Self {
        let input = Rc::new(RefCell::new(VirtualInput::default()));
        let saved_state = {
            let input = input.borrow();
            *input.saved_state.borrow_mut() = options.restored_state.clone();
            Rc::clone(&input.saved_state)
        };
        let platform = Rc::new(VirtualPlatform {
            saved_state,
            responders: RefCell::new(Vec::new()),
            now: Clock::default(),
            capabilities: options.capabilities,
            clipboard: RefCell::new(String::new()),
            dialogs: RefCell::new(VecDeque::new()),
            opened: RefCell::new(Vec::new()),
            revealed: RefCell::new(Vec::new()),
            fail_next: RefCell::new(None),
        });
        Self {
            options,
            platform,
            windows: Rc::new(RefCell::new(BTreeMap::new())),
            input,
            woken: Arc::new(WakePending::default()),
        }
    }
    pub fn script(&self) -> Script {
        Script(Rc::clone(&self.input))
    }
    pub fn probe(&self) -> Probe {
        Probe {
            platform: Rc::clone(&self.platform),
            windows: Rc::clone(&self.windows),
            input: Rc::clone(&self.input),
            woken: Arc::clone(&self.woken),
        }
    }
}
/// Decide where `request`'s window goes, opening its native mirror when asked.
fn open_display(options: &VirtualOptions, request: &WindowRequest) -> Result<Display, ShellError> {
    if options.gpu == GpuMode::Required && options.mirror == MirrorMode::Off {
        return Err(ShellError::unsupported(
            "GPU presentation requires a mirror window",
        ));
    }
    #[cfg(feature = "mirror")]
    let mirror = options.open_mirror(request)?;
    #[cfg(not(feature = "mirror"))]
    let mirror: Option<Box<dyn crate::window::Mirror>> = {
        let _ = request;
        if options.mirror == MirrorMode::Required || options.gpu == GpuMode::Required {
            return Err(ShellError::unsupported("mirror support is not built in"));
        }
        None
    };
    if options.gpu == GpuMode::Required && mirror.is_none() {
        return Err(ShellError::unsupported(
            "GPU presentation requires a native mirror",
        ));
    }
    Ok(match mirror {
        Some(mirror) => Display::Mirrored(mirror),
        None => Display::Headless(Accessibility::new()),
    })
}
impl SteppedBackend for VirtualBackend {
    fn start(
        self: Box<Self>,
        handler: Box<dyn ShellHandler>,
    ) -> Result<Box<dyn SteppedLoop>, ShellError> {
        let Self {
            options,
            platform,
            windows,
            input,
            woken,
        } = *self;
        let request = options.main.clone();
        let display = open_display(&options, &request)?;
        let main = VirtualWindow::new(&platform, &options, &request, display);
        windows.borrow_mut().insert(request.id, main);
        input
            .borrow_mut()
            .windows
            .insert(request.id, VirtualWindowInput::default());
        let waker = ShellWaker::new({
            let wake = Arc::clone(&woken);
            move |reasons| wake.request(reasons)
        });
        let mut running = Running {
            options,
            platform,
            windows,
            input,
            handler,
            control: ShellControl::Wait,
            state: State::Live,
            woken,
            waker,
        };
        match running.start_handler() {
            Ok(control) => {
                running.control = control;
                if control == ShellControl::Exit {
                    running.finish(State::Exited);
                }
            }
            Err(error) => {
                running.finish(State::Failed(error.clone()));
                return Err(error);
            }
        }
        running.deliver(
            arvane_shell::EventTarget::Window(request.id),
            ShellEvent::AccessibilityActivated,
        );
        Ok(Box::new(running))
    }
}
/// Coalesced wake reasons shared with background workers.
#[derive(Default)]
struct WakePending(Mutex<WakeReasons>);
impl WakePending {
    fn request(&self, reasons: WakeReasons) {
        self.0
            .lock()
            .expect("invariant: wake mailbox is not poisoned")
            .insert(reasons);
    }
    fn take(&self) -> WakeReasons {
        std::mem::take(
            &mut *self
                .0
                .lock()
                .expect("invariant: wake mailbox is not poisoned"),
        )
    }
    fn pending(&self) -> bool {
        !self
            .0
            .lock()
            .expect("invariant: wake mailbox is not poisoned")
            .is_empty()
    }
}
enum State {
    Live,
    Exited,
    Failed(ShellError),
}
/// The running loop: the handler, the shared window table and the input queue.
struct Running {
    options: VirtualOptions,
    platform: Rc<VirtualPlatform>,
    windows: Rc<RefCell<BTreeMap<WindowId, VirtualWindow>>>,
    input: Rc<RefCell<VirtualInput>>,
    handler: Box<dyn ShellHandler>,
    control: ShellControl,
    state: State,
    woken: Arc<WakePending>,
    waker: ShellWaker,
}
impl Running {
    fn finish(&mut self, state: State) {
        if matches!(self.state, State::Live) {
            let mut windows = self.windows.borrow_mut();
            let mut input = self.input.borrow_mut();
            let mut shell = VirtualLoop {
                options: &self.options,
                platform: &self.platform,
                windows: &mut windows,
                input: &mut input,
                waker: &self.waker,
            };
            self.handler.close(&mut shell);
        }
        self.state = state;
    }
    fn start_handler(&mut self) -> Result<ShellControl, ShellError> {
        let mut windows = self.windows.borrow_mut();
        let mut input = self.input.borrow_mut();
        let mut shell = VirtualLoop {
            options: &self.options,
            platform: &self.platform,
            windows: &mut windows,
            input: &mut input,
            waker: &self.waker,
        };
        self.handler.start(&mut shell, self.waker.clone())
    }
    fn deliver(&mut self, target: arvane_shell::EventTarget, event: ShellEvent) {
        if !matches!(self.state, State::Live) {
            return;
        }
        let result = {
            let mut windows = self.windows.borrow_mut();
            let mut input = self.input.borrow_mut();
            let mut shell = VirtualLoop {
                options: &self.options,
                platform: &self.platform,
                windows: &mut windows,
                input: &mut input,
                waker: &self.waker,
            };
            self.handler.event(target, event, &mut shell)
        };
        match result {
            Ok(ShellControl::Exit) => self.finish(State::Exited),
            Ok(control) => self.control = control,
            Err(error) => self.finish(State::Failed(error)),
        }
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        if matches!(self.state, State::Live) {
            self.finish(State::Exited);
        }
    }
}
impl SteppedLoop for Running {
    fn step(&mut self, step: Step) -> StepOutcome {
        if matches!(self.state, State::Live) {
            self.platform.now.set(self.input.borrow().clock.now());
            match step {
                Step::Input => loop {
                    let item = self.input.borrow_mut().queue.pop_front();
                    let Some(item) = item else {
                        break;
                    };
                    let events = {
                        let mut windows = self.windows.borrow_mut();
                        apply(&self.platform, &mut windows, item)
                    };
                    for (target, event) in events {
                        self.deliver(target, event);
                        if !matches!(self.state, State::Live) {
                            break;
                        }
                    }
                    if !matches!(self.state, State::Live) {
                        break;
                    }
                },
                Step::Wake => {
                    let mut reasons = self.woken.take();
                    if reasons.is_empty() {
                        reasons.insert(WakeReasons::CLIENT);
                    }
                    self.deliver(arvane_shell::EventTarget::Loop, ShellEvent::Wake(reasons));
                }
                Step::Redraw => {
                    let ids = {
                        let mut windows = self.windows.borrow_mut();
                        let mut ids = Vec::new();
                        for (id, window) in windows.iter_mut() {
                            if window.redraw.replace(false) {
                                ids.push(*id);
                            }
                        }
                        ids
                    };
                    for id in ids {
                        self.deliver(arvane_shell::EventTarget::Window(id), ShellEvent::Redraw);
                    }
                }
            }
        }
        match &self.state {
            State::Live => StepOutcome::Idle {
                next_deadline: self.control.deadline(),
                redraw_requested: self
                    .windows
                    .borrow()
                    .values()
                    .any(|window| window.redraw.get()),
                woken: self.woken.pending(),
            },
            State::Exited => StepOutcome::Exited,
            State::Failed(error) => StepOutcome::Failed(error.clone()),
        }
    }
}
/// The loop as the handler sees it during one callback.
struct VirtualLoop<'a> {
    options: &'a VirtualOptions,
    platform: &'a Rc<VirtualPlatform>,
    windows: &'a mut BTreeMap<WindowId, VirtualWindow>,
    input: &'a mut VirtualInput,
    waker: &'a ShellWaker,
}
impl ShellLoop for VirtualLoop<'_> {
    fn window(&mut self, id: WindowId) -> Option<&mut dyn ShellWindow> {
        self.windows
            .get_mut(&id)
            .map(|window| window as &mut dyn ShellWindow)
    }
    fn windows(&self) -> Vec<WindowId> {
        self.windows.keys().copied().collect()
    }
    fn open_window(&mut self, request: WindowRequest) -> Result<(), ShellError> {
        if !self.platform.capabilities.multi_window {
            return Err(ShellError::unsupported(
                "this platform opens no window beyond the first",
            ));
        }
        let display = open_display(self.options, &request)?;
        let window = VirtualWindow::new(self.platform, self.options, &request, display);
        self.windows.insert(request.id, window);
        self.input
            .windows
            .insert(request.id, VirtualWindowInput::default());
        self.input.queue.push_back(Item::Window(
            request.id,
            WindowItem::Event(ShellEvent::AccessibilityActivated),
        ));
        Ok(())
    }
    fn close_window(&mut self, id: WindowId) {
        self.windows.remove(&id);
        self.input.windows.remove(&id);
        self.input
            .queue
            .retain(|item| !matches!(item, Item::Window(window, _) if *window == id));
    }
    fn inject(&mut self, id: WindowId, inputs: Vec<InjectedInput>, request: RequestId) {
        let Some(window) = self.windows.get(&id) else {
            return;
        };
        let now = self.input.clock.now();
        let double_click = kivact_types::defaults::DOUBLE_CLICK;
        let mut events: Vec<ShellEvent> = Vec::new();
        {
            let Some(state) = self.input.windows.get_mut(&id) else {
                return;
            };
            for input in inputs {
                match input {
                    InjectedInput::Accessibility { node, request } => {
                        if let Some(event) = window.accessibility().request_action(node, request) {
                            events.push(event);
                        }
                    }
                    input => events.extend(state.input.normalize(
                        input,
                        now,
                        double_click,
                        &mut state.ime,
                    )),
                }
            }
        }
        for event in events {
            self.input
                .queue
                .push_back(Item::Window(id, WindowItem::Event(event)));
        }
        self.input.queue.push_back(Item::Window(
            id,
            WindowItem::Event(ShellEvent::Injected { request }),
        ));
    }
    fn now_ns(&self) -> u64 {
        self.platform.now.now()
    }
    fn waker(&self) -> ShellWaker {
        self.waker.clone()
    }
    fn capabilities(&self) -> Capabilities {
        match self.windows.get(&self.options.main.id) {
            Some(main) => main.capabilities(),
            None => Capabilities {
                gpu: self.options.gpu == GpuMode::Required,
                ..self.options.capabilities
            },
        }
    }
}
/// Apply one queued item, returning the events and their targets in delivery order.
fn apply(
    platform: &VirtualPlatform,
    windows: &mut BTreeMap<WindowId, VirtualWindow>,
    item: Item,
) -> Vec<(arvane_shell::EventTarget, ShellEvent)> {
    let mut delivered = Vec::new();
    match item {
        Item::Respond(responder) => platform.responders.borrow_mut().push(responder),
        Item::Time(ns) => platform.now.set(ns),
        Item::Clipboard(text) => *platform.clipboard.borrow_mut() = text,
        Item::Dialog(paths) => platform.dialogs.borrow_mut().push_back(paths),
        Item::FailEffect(error) => *platform.fail_next.borrow_mut() = Some(error),
        Item::LoopEvent(event) => delivered.push((arvane_shell::EventTarget::Loop, event)),
        Item::Pointer(settings) => {
            for (id, window) in windows.iter_mut() {
                window.env.get_mut().pointer = settings;
                delivered.push((
                    arvane_shell::EventTarget::Window(*id),
                    ShellEvent::EnvironmentChanged,
                ));
            }
        }
        Item::Window(id, item) => {
            let Some(window) = windows.get_mut(&id) else {
                return delivered;
            };
            if let Some(event) = apply_window(window, item) {
                delivered.push((arvane_shell::EventTarget::Window(id), event));
            }
        }
    }
    delivered
}
/// Apply one queued window item and return its boundary event, if any.
fn apply_window(window: &mut VirtualWindow, item: WindowItem) -> Option<ShellEvent> {
    let event = match item {
        WindowItem::Event(event) => {
            if let ShellEvent::KeyboardInset(insets) = &event {
                window.env.get_mut().keyboard_inset = insets.sanitize();
            }
            Some(event)
        }
        WindowItem::Resize(size) => {
            window.env.get_mut().window_size = size;
            Some(ShellEvent::Resized)
        }
        WindowItem::Scale(scale) => {
            window.env.get_mut().scale_factor = scale;
            Some(ShellEvent::Resized)
        }
        WindowItem::Period(ns) => {
            window.env.get_mut().frame_period_ns = ns;
            Some(ShellEvent::EnvironmentChanged)
        }
        WindowItem::Reduced(value) => {
            window.env.get_mut().reduced_motion = value;
            Some(ShellEvent::EnvironmentChanged)
        }
        WindowItem::Dark(value) => {
            window.env.get_mut().theme_hint = theme_hint(value);
            Some(ShellEvent::EnvironmentChanged)
        }
        WindowItem::Locale(locale) => {
            window.env.get_mut().locale = locale;
            Some(ShellEvent::EnvironmentChanged)
        }
        WindowItem::SafeArea(insets) => {
            window.env.get_mut().safe_area = insets;
            Some(ShellEvent::EnvironmentChanged)
        }
        WindowItem::Surface(live) => surface_event(window, live),
        WindowItem::Occluded(value) => {
            window.occluded = value;
            None
        }
        WindowItem::FailPresents(count) => {
            window.failing = count;
            None
        }
        WindowItem::Action(node, request) => window.accessibility().request_action(node, request),
    };
    if matches!(event, Some(ShellEvent::Resized))
        && let Display::Mirrored(real) = &window.display
    {
        let (width, height) = window.physical_size();
        real.resize(width, height, window.scale());
    }
    event
}
/// Map the scripted preference into the canonical theme vocabulary.
fn theme_hint(dark: bool) -> ThemeHint {
    if dark {
        ThemeHint::Dark
    } else {
        ThemeHint::Light
    }
}
/// Change the scripted surface lifecycle and return its boundary event.
fn surface_event(window: &mut VirtualWindow, live: bool) -> Option<ShellEvent> {
    match (live, window.surface) {
        (true, TestSurface::Destroyed { last }) => {
            window.surface = TestSurface::Live {
                generation: last
                    .checked_add(1)
                    .expect("invariant: surface generations fit u64"),
            };
            Some(ShellEvent::SurfaceCreated)
        }
        (false, TestSurface::Live { generation }) => {
            window.surface = TestSurface::Destroyed { last: generation };
            Some(ShellEvent::SurfaceDestroyed)
        }
        _ => None,
    }
}
/// Cloneable read-only observations. Values never borrow backend state.
#[derive(Clone)]
pub struct Probe {
    platform: Rc<VirtualPlatform>,
    windows: Rc<RefCell<BTreeMap<WindowId, VirtualWindow>>>,
    input: Rc<RefCell<VirtualInput>>,
    woken: Arc<WakePending>,
}
impl Probe {
    /// The open window `id`, or `None` before start and after it closed.
    pub fn window(&self, id: WindowId) -> Option<WindowProbe> {
        if self.windows.borrow().contains_key(&id) {
            Some(WindowProbe {
                windows: Rc::clone(&self.windows),
                input: Rc::clone(&self.input),
                window: id,
            })
        } else {
            None
        }
    }
    /// The open windows, ascending by id.
    pub fn windows(&self) -> Vec<WindowId> {
        self.windows.borrow().keys().copied().collect()
    }
    /// Whether a worker has requested a wake since the last wake step.
    pub fn woken(&self) -> bool {
        self.woken.pending()
    }
    pub fn now_ns(&self) -> u64 {
        self.input.borrow().clock.now()
    }
    pub fn clipboard(&self) -> String {
        self.platform.clipboard.borrow().clone()
    }
    pub fn opened_paths(&self) -> Vec<PathBuf> {
        self.platform.opened.borrow().clone()
    }
    pub fn revealed_paths(&self) -> Vec<PathBuf> {
        self.platform.revealed.borrow().clone()
    }
}
/// Cloneable read-only observations of one virtual window.
#[derive(Clone)]
pub struct WindowProbe {
    windows: Rc<RefCell<BTreeMap<WindowId, VirtualWindow>>>,
    input: Rc<RefCell<VirtualInput>>,
    window: WindowId,
}
impl WindowProbe {
    fn with<R>(&self, f: impl FnOnce(&VirtualWindow) -> R) -> R {
        let windows = self.windows.borrow();
        let window = windows
            .get(&self.window)
            .expect("invariant: a WindowProbe names an open window");
        f(window)
    }
    fn input_with<R>(&self, f: impl FnOnce(&VirtualWindowInput) -> R) -> R {
        let input = self.input.borrow();
        let state = input
            .windows
            .get(&self.window)
            .expect("invariant: an open window has input state");
        f(state)
    }
    pub fn ime_purpose(&self) -> kivact_types::InputPurpose {
        self.with(|window| window.ime_purpose.get())
    }
    pub fn mouse_position(&self) -> Option<Point> {
        self.input_with(|state| state.input.mouse_position())
    }
    pub fn modifiers(&self) -> Modifiers {
        self.input_with(|state| state.input.modifiers())
    }
    pub fn redraw_requested(&self) -> bool {
        self.with(|window| window.redraw.get())
    }
    pub fn frame(&self) -> Option<(u32, u32, Arc<[u32]>)> {
        self.with(|window| match &window.shown {
            crate::window::Shown::OnCpuSurface(kept) | crate::window::Shown::Kept(kept) => {
                Some((kept.width, kept.height, kept.pixels.clone()))
            }
            crate::window::Shown::Nothing => None,
        })
    }
    pub fn accessibility(&self) -> Option<SemanticsTree> {
        self.with(|window| window.accessibility().tree())
    }
    pub fn title(&self) -> String {
        self.with(|window| window.title.borrow().clone())
    }
    pub fn requested_sizes(&self) -> Vec<Size> {
        self.with(|window| window.requested_sizes.clone())
    }
    pub fn min_size(&self) -> Option<Size> {
        self.with(|window| window.min_size.get())
    }
    pub fn ime_area(&self) -> Option<Rect> {
        self.with(|window| window.ime.get())
    }
    pub fn cursor(&self) -> Cursor {
        self.with(|window| window.cursor.get())
    }
    pub fn soft_keyboard(&self) -> (bool, u32) {
        self.with(|window| (window.soft_keyboard.0.get(), window.soft_keyboard.1.get()))
    }
    pub fn ime_text(&self) -> Option<ShellImeText> {
        self.with(|window| window.ime_text.borrow().clone())
    }
    pub fn surface_generation(&self) -> Option<u64> {
        self.with(|window| window.surface.generation())
    }
    /// Whether this window is mirrored into a real native window.
    pub fn has_window(&self) -> bool {
        self.with(|window| matches!(window.display, Display::Mirrored(_)))
    }
    pub fn environment(&self) -> Environment {
        self.with(|window| window.environment())
    }
    pub fn size(&self) -> Size {
        self.with(|window| window.size())
    }
    pub fn scale(&self) -> f64 {
        self.with(|window| window.scale())
    }
    pub fn capabilities(&self) -> Capabilities {
        self.with(|window| window.capabilities())
    }
    pub fn sync_mirror(&self) {
        self.with(|window| {
            if let Display::Mirrored(real) = &window.display {
                real.sync();
            }
        });
    }
    pub fn focus_requests(&self) -> u32 {
        self.with(|window| window.focus_requests.get())
    }
}
