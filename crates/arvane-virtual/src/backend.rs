//! Role: Start and advance the virtual backend and expose owned observations.

use crate::script::{Input, Item, Script};
use crate::surface::TestSurface;
use crate::window::VirtualShell;
pub use crate::window::{GpuMode, MirrorMode, VirtualOptions};
use arvane_shell::{
    Accessibility, ShellControl, ShellError, ShellEvent, ShellHandler, ShellImeText, ShellWaker,
    ShellWindow, Step, StepOutcome, SteppedBackend, SteppedLoop,
};
use kivact_types::render::SemanticsTree;
use kivact_types::{Capabilities, Cursor, Environment, Modifiers, Point, Rect, Size, ThemeHint};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Backend whose input and clock are advanced explicitly.
pub struct VirtualBackend {
    options: VirtualOptions,
    shell: Rc<RefCell<VirtualShell>>,
    input: Rc<RefCell<Input>>,
}
impl VirtualBackend {
    pub fn new(options: VirtualOptions) -> Self {
        let shell = VirtualShell {
            responders: Vec::new(),
            now: 0,
            env: RefCell::new(Environment {
                window_size: options.size,
                scale_factor: options.scale,
                frame_period_ns: options.frame_period_ns.max(1),
                reduced_motion: options.reduced_motion,
                theme_hint: if options.dark {
                    ThemeHint::Dark
                } else {
                    ThemeHint::Light
                },
                locale: options.locale.clone(),
                ..Environment::default()
            }),
            requested_sizes: Vec::new(),
            clipboard: String::new(),
            dialogs: VecDeque::new(),
            opened: Vec::new(),
            revealed: Vec::new(),
            fail_next: None,
            redraw: Cell::new(false),
            title: RefCell::new(options.title.clone()),
            min_size: Cell::new(None),
            ime: Cell::new(None),
            ime_text: RefCell::new(None),
            cursor: Cell::new(Cursor::Default),
            soft_keyboard: (Cell::new(false), Cell::new(0)),
            frame: None,
            gpu: options.gpu == GpuMode::Required,
            real: None,
            headless: Accessibility::new(),
            occluded: false,
            failing: 0,
            surface: TestSurface::Live {
                generation: NonZeroU64::MIN,
            },
            cpu_frame: Cell::new(false),
        };
        Self {
            options,
            shell: Rc::new(RefCell::new(shell)),
            input: Rc::new(RefCell::new(Input::default())),
        }
    }
    pub fn script(&self) -> Script {
        Script(Rc::clone(&self.input))
    }
    pub fn probe(&self) -> Probe {
        Probe {
            shell: Rc::clone(&self.shell),
            input: Rc::clone(&self.input),
        }
    }
}
impl SteppedBackend for VirtualBackend {
    fn start(
        self: Box<Self>,
        handler: Box<dyn ShellHandler>,
    ) -> Result<Box<dyn SteppedLoop>, ShellError> {
        if self.options.gpu == GpuMode::Required && self.options.mirror == MirrorMode::Off {
            return Err(ShellError::unsupported(
                "GPU presentation requires a mirror window",
            ));
        }
        #[cfg(feature = "mirror")]
        {
            self.shell.borrow_mut().real = self.options.open_mirror()?;
        }
        #[cfg(not(feature = "mirror"))]
        if self.options.mirror == MirrorMode::Required || self.options.gpu == GpuMode::Required {
            return Err(ShellError::unsupported("mirror support is not built in"));
        }
        if self.options.gpu == GpuMode::Required && self.shell.borrow().real.is_none() {
            return Err(ShellError::unsupported(
                "GPU presentation requires a native mirror",
            ));
        }
        let mut running = Running {
            shell: self.shell,
            input: self.input,
            handler,
            control: ShellControl::Wait,
            state: State::Live,
            woken: Arc::new(WakePending::default()),
        };
        let wake = Arc::clone(&running.woken);
        let result = running.handler.start(
            &mut *running.shell.borrow_mut(),
            ShellWaker::new(move || {
                wake.request();
            }),
        );
        match result {
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
        Ok(Box::new(running))
    }
}
/// Latest-wins wake mailbox shared with background workers.
#[derive(Default)]
struct WakePending(AtomicBool);
impl WakePending {
    fn request(&self) {
        self.0.store(true, Ordering::Release);
    }
    fn take(&self) {
        self.0.store(false, Ordering::Release);
    }
    fn pending(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
enum State {
    Live,
    Exited,
    Failed(ShellError),
}
struct Running {
    shell: Rc<RefCell<VirtualShell>>,
    input: Rc<RefCell<Input>>,
    handler: Box<dyn ShellHandler>,
    control: ShellControl,
    state: State,
    woken: Arc<WakePending>,
}
impl Running {
    fn finish(&mut self, state: State) {
        if matches!(self.state, State::Live) {
            self.handler.close(&mut *self.shell.borrow_mut());
        }
        self.state = state;
    }
    fn deliver(&mut self, event: ShellEvent) {
        if !matches!(self.state, State::Live) {
            return;
        }
        let result = self.handler.event(event, &mut *self.shell.borrow_mut());
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
            self.shell.borrow_mut().now = self.input.borrow().clock.now();
            match step {
                Step::Input => loop {
                    let item = self.input.borrow_mut().queue.pop_front();
                    let Some(item) = item else {
                        break;
                    };
                    let event = apply(&mut self.shell.borrow_mut(), item);
                    if let Some(event) = event {
                        self.deliver(event);
                    }
                    if !matches!(self.state, State::Live) {
                        break;
                    }
                },
                Step::Wake => {
                    self.woken.take();
                    self.deliver(ShellEvent::Wake);
                }
                Step::Redraw => {
                    let redraw = self.shell.borrow().redraw.replace(false);
                    if redraw {
                        self.deliver(ShellEvent::Redraw);
                    }
                }
            }
        }
        match &self.state {
            State::Live => StepOutcome::Idle {
                next_deadline: self.control.deadline(),
                redraw_requested: self.shell.borrow().redraw.get(),
                woken: self.woken.pending(),
            },
            State::Exited => StepOutcome::Exited,
            State::Failed(error) => StepOutcome::Failed(error.clone()),
        }
    }
}
fn apply(shell: &mut VirtualShell, item: Item) -> Option<ShellEvent> {
    let event = match item {
        Item::Event(event) => Some(event),
        Item::Time(ns) => {
            shell.now = ns;
            None
        }
        Item::Resize(size) => {
            shell.env.get_mut().window_size = size;
            Some(ShellEvent::Resized)
        }
        Item::Scale(scale) => {
            shell.env.get_mut().scale_factor = scale;
            Some(ShellEvent::Resized)
        }
        Item::Period(ns) => {
            shell.env.get_mut().frame_period_ns = ns;
            Some(ShellEvent::EnvironmentChanged)
        }
        Item::Reduced(value) => {
            shell.env.get_mut().reduced_motion = value;
            Some(ShellEvent::EnvironmentChanged)
        }
        Item::Dark(value) => {
            shell.env.get_mut().theme_hint = if value {
                ThemeHint::Dark
            } else {
                ThemeHint::Light
            };
            Some(ShellEvent::EnvironmentChanged)
        }
        Item::Locale(locale) => {
            shell.env.get_mut().locale = locale;
            Some(ShellEvent::EnvironmentChanged)
        }
        Item::SafeArea(insets) => {
            shell.env.get_mut().safe_area = insets;
            Some(ShellEvent::EnvironmentChanged)
        }
        Item::Surface(live) => match (live, shell.surface) {
            (true, TestSurface::Destroyed { last }) => {
                shell.surface = TestSurface::Live {
                    generation: last
                        .checked_add(1)
                        .expect("invariant: surface generations fit u64"),
                };
                Some(ShellEvent::SurfaceCreated)
            }
            (false, TestSurface::Live { generation }) => {
                shell.surface = TestSurface::Destroyed { last: generation };
                Some(ShellEvent::SurfaceDestroyed)
            }
            _ => None,
        },
        Item::Occluded(value) => {
            shell.occluded = value;
            None
        }
        Item::FailPresents(count) => {
            shell.failing = count;
            None
        }
        Item::Clipboard(text) => {
            shell.clipboard = text;
            None
        }
        Item::Dialog(paths) => {
            shell.dialogs.push_back(paths);
            None
        }
        Item::Respond(responder) => {
            shell.responders.push(responder);
            None
        }
        Item::FailEffect(error) => {
            shell.fail_next = Some(error);
            None
        }
        Item::Action(node, request) => shell.accessibility().request_action(node, request),
    };
    if matches!(event, Some(ShellEvent::Resized)) {
        if let Some(real) = &shell.real {
            let (w, h) = shell.physical_size();
            real.resize(w, h, shell.scale());
        }
    }
    event
}
/// Cloneable read-only observations. Values never borrow backend state.
#[derive(Clone)]
pub struct Probe {
    shell: Rc<RefCell<VirtualShell>>,
    input: Rc<RefCell<Input>>,
}
impl Probe {
    pub fn now_ns(&self) -> u64 {
        self.input.borrow().clock.now()
    }
    pub fn mouse_position(&self) -> Option<Point> {
        self.input.borrow().mouse.window_position()
    }
    pub fn modifiers(&self) -> Modifiers {
        self.input.borrow().modifiers
    }
    pub fn redraw_requested(&self) -> bool {
        self.shell.borrow().redraw.get()
    }
    pub fn frame(&self) -> Option<(u32, u32, Arc<[u32]>)> {
        self.shell.borrow().frame.clone()
    }
    pub fn accessibility(&self) -> Option<SemanticsTree> {
        self.shell.borrow().accessibility().tree()
    }
    pub fn clipboard(&self) -> String {
        self.shell.borrow().clipboard.clone()
    }
    pub fn opened_paths(&self) -> Vec<PathBuf> {
        self.shell.borrow().opened.clone()
    }
    pub fn revealed_paths(&self) -> Vec<PathBuf> {
        self.shell.borrow().revealed.clone()
    }
    pub fn title(&self) -> String {
        self.shell.borrow().title.borrow().clone()
    }
    pub fn requested_sizes(&self) -> Vec<Size> {
        self.shell.borrow().requested_sizes.clone()
    }
    pub fn min_size(&self) -> Option<Size> {
        self.shell.borrow().min_size.get()
    }
    pub fn ime_area(&self) -> Option<Rect> {
        self.shell.borrow().ime.get()
    }
    pub fn cursor(&self) -> Cursor {
        self.shell.borrow().cursor.get()
    }
    pub fn soft_keyboard(&self) -> (bool, u32) {
        let shell = self.shell.borrow();
        (shell.soft_keyboard.0.get(), shell.soft_keyboard.1.get())
    }
    pub fn ime_text(&self) -> Option<ShellImeText> {
        self.shell.borrow().ime_text.borrow().clone()
    }
    pub fn surface_generation(&self) -> Option<u64> {
        self.shell.borrow().surface.generation()
    }
    pub fn has_window(&self) -> bool {
        self.shell.borrow().real.is_some()
    }
    pub fn environment(&self) -> Environment {
        self.shell.borrow().environment()
    }
    pub fn size(&self) -> Size {
        self.shell.borrow().size()
    }
    pub fn scale(&self) -> f64 {
        self.shell.borrow().scale()
    }
    pub fn capabilities(&self) -> Capabilities {
        self.shell.borrow().capabilities()
    }
    pub fn sync_mirror(&self) {
        if let Some(real) = &self.shell.borrow().real {
            real.sync();
        }
    }
}
