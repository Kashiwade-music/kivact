//! Role: Drive a client test window and query its settled engine model.

use super::test_frame::TestFrame;
use super::test_options::{TestFonts, TestHostOptions, default_font};
use crate::contract::{HostError, HostEvent, NodeProperties, NodeType};
use crate::host::Client;
use crate::host::launch::{AssetMode, Launch};
use crate::model::Model;
use arvane::spi::{Step, StepOutcome};
use arvane::{
    EventTarget, Handler, Renderer, RendererChange, Shell, ShellControl, ShellError, ShellEvent,
    ShellImeText, ShellSetup, ShellWaker, SteppedShell, Windows,
};
use arvane_virtual::{GpuMode, Probe, Script, VirtualBackend, VirtualOptions};
use kivact_types::PointerSample;
use kivact_types::render::SemanticsTree;
use kivact_types::{
    Capabilities, DrawList, LayoutCheck, LayoutFinding, NodeId, Rect, RegionId, Size, ThemeHint,
    WindowId, WindowRequest,
};
use kivact_types::{Cursor, Insets, Modifiers, Point};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

/// Turns and frame ticks the client was given.
#[derive(Default)]
struct Counts {
    // xtask-allow(state-fields): diagnostic statistics; never used to control scheduling.
    turns: Cell<u64>,
    // xtask-allow(state-fields): diagnostic statistics; never used as an event or generation.
    frame_ticks: Cell<u64>,
}

/// Count turns without adding a second client abstraction.
fn counted(client: Client, counts: Rc<Counts>) -> Client {
    let shared = Rc::new(RefCell::new(client));
    let start = shared.clone();
    Client::new(
        move |info, waker, writer| start.borrow_mut().start(info, waker, writer),
        move |events, boundaries, now_ns| {
            counts.turns.set(counts.turns.get() + 1);
            let ticks = events
                .iter()
                .filter(|event| matches!(event, HostEvent::FrameTick { .. }))
                .count() as u64;
            counts.frame_ticks.set(counts.frame_ticks.get() + ticks);
            shared.borrow_mut().turn(events, boundaries, now_ns)
        },
    )
}

/// The engine driven by a test on the calling thread.
pub struct ArvaneTestHost {
    pub(super) host: Rc<RefCell<Launch>>,
    pub(super) probe: Probe,
    script: Script,
    shell: SteppedShell,
    outcome: StepOutcome,
    renderer: Rc<RefCell<BTreeMap<WindowId, RendererState>>>,
    injected: Rc<RefCell<Vec<HostEvent>>>,
    controls: Rc<RefCell<Vec<Control>>>,
    counts: Rc<Counts>,
}

/// A test control the window handler applies at the next event, where the
/// window is at hand.
enum Control {
    /// Make the window's GPU device lost (`arvane::spi::lose_gpu_device`).
    LoseGpuDevice(WindowId),
}

enum RendererState {
    Ready(Renderer),
    Failed(String),
}

struct SharedHost {
    host: Rc<RefCell<Launch>>,
    injected: Rc<RefCell<Vec<HostEvent>>>,
    controls: Rc<RefCell<Vec<Control>>>,
}
impl Handler for SharedHost {
    fn start(
        &mut self,
        windows: &mut Windows<'_>,
        waker: ShellWaker,
    ) -> Result<ShellControl, ShellError> {
        self.host.borrow_mut().start(windows, waker)
    }
    fn event(
        &mut self,
        target: EventTarget,
        event: ShellEvent,
        windows: &mut Windows<'_>,
    ) -> Result<ShellControl, ShellError> {
        for control in self.controls.borrow_mut().drain(..) {
            match control {
                Control::LoseGpuDevice(id) => {
                    if let Some(mut window) = windows.get(id) {
                        arvane::spi::lose_gpu_device(&mut window);
                    }
                }
            }
        }
        let mut launch = self.host.borrow_mut();
        let host = launch.host_mut();
        for injected in self.injected.borrow_mut().drain(..) {
            let control = host.inject(injected, windows)?;
            if control == ShellControl::Exit {
                return Ok(control);
            }
        }
        host.handle(target, event, windows)
    }
    fn close(&mut self, _windows: &mut Windows<'_>) {}
}

fn make_host(
    client: Client,
    options: &TestHostOptions,
    counts: Rc<Counts>,
) -> Result<Rc<RefCell<Launch>>, HostError> {
    let engine = match &options.fonts {
        TestFonts::Bundled => arvane::Engine::with_fonts_only(&[default_font()]),
        TestFonts::Only(fonts) => arvane::Engine::with_fonts_only(fonts),
        TestFonts::System => Ok(arvane::Engine::new()),
    }
    .map_err(|e| HostError::new(e.to_string()))?;
    let mut host = crate::host::new(
        counted(client, counts),
        Size {
            width: options.width,
            height: options.height,
        },
        engine,
        Rc::new(RefCell::new(crate::FrameLog::disabled())),
    );
    // Reported light at start in production; a test may start dark.
    if let Launch::Pending(args) = &mut host {
        args.theme = options.theme;
        args.assets = if options.inline_assets {
            AssetMode::Inline
        } else {
            AssetMode::Worker
        };
    }
    Ok(Rc::new(RefCell::new(host)))
}

impl ArvaneTestHost {
    /// Engine mutation access for existing backend contract E2E tests.
    pub fn writer(&self) -> crate::api::Writer {
        let launch = self.host.borrow();
        let host = launch.host();
        crate::api::Writer::new(&host.app, &host.engine)
    }
    /// Queue platform prerequisites and input for this host.
    pub fn script(&self) -> arvane_virtual::Script {
        self.script.clone()
    }

    /// Start `client` on the host (its `start` runs here, and the turns its
    /// first revision asks for).
    pub fn new(client: Client, options: TestHostOptions) -> Result<Self, HostError> {
        let error = |e: &dyn std::fmt::Display| HostError::new(e.to_string());
        let default_capabilities = VirtualOptions::default().capabilities;
        let backend = VirtualBackend::new(VirtualOptions {
            main: WindowRequest::main(
                options.title.clone(),
                Size {
                    width: options.width,
                    height: options.height,
                },
            ),
            scale: options.scale,
            frame_period_ns: options.frame_period_ns,
            reduced_motion: options.reduced_motion,
            dark: options.theme == ThemeHint::Dark,
            locale: options.locale.clone(),
            mirror: options.mirror,
            gpu: options.gpu,
            capabilities: Capabilities {
                multi_window: !options.single_window,
                window_placement: !options.single_window,
                ..default_capabilities
            },
            restored_state: options.restored_state.clone(),
        });
        let script = backend.script();
        let probe = backend.probe();
        let counts = Rc::new(Counts::default());
        let host = make_host(client, &options, counts.clone())?;
        let renderer = Rc::new(RefCell::new(BTreeMap::new()));
        let observed = renderer.clone();
        let injected = Rc::new(RefCell::new(Vec::new()));
        let controls = Rc::new(RefCell::new(Vec::new()));
        let shell = Shell::start_stepped(
            Box::new(backend),
            ShellSetup {
                prefer_gpu: options.gpu == GpuMode::Required,
                gpu_preference: kivact_types::GpuPreference {
                    // A GPU test draws its first frames on the GPU.
                    start: if options.gpu == GpuMode::Required {
                        kivact_types::gpu::GpuStart::AtLaunch
                    } else {
                        options.gpu_preference.start
                    },
                    ..options.gpu_preference
                },
                observer: Some(Rc::new(RefCell::new(
                    move |window, change: &RendererChange| {
                        let state = match change {
                            RendererChange::Start { .. } => RendererState::Ready(Renderer::Cpu),
                            RendererChange::Gpu { .. } => RendererState::Ready(Renderer::Gpu),
                            RendererChange::Cpu { reason } => RendererState::Failed(reason.clone()),
                            RendererChange::ExternalFrameMissed { .. }
                            | RendererChange::ShaderFailed { .. }
                            | RendererChange::LimitExceeded { .. } => return,
                        };
                        observed.borrow_mut().insert(window, state);
                    },
                ))),
            },
            SharedHost {
                host: host.clone(),
                injected: injected.clone(),
                controls: controls.clone(),
            },
        )
        .map_err(|e| error(&e))?;
        let mut driver = Self {
            host,
            probe,
            script,
            shell,
            outcome: StepOutcome::Idle {
                next_deadline: None,
                redraw_requested: false,
                woken: false,
            },
            renderer,
            injected,
            controls,
            counts,
        };
        driver.step(Step::Wake);
        driver.wait_for_gpu(options.gpu)?;
        if let Some(failure) = driver.failure() {
            return Err(HostError::new(failure.to_string()));
        }
        Ok(driver)
    }

    fn wait_for_gpu(&mut self, mode: GpuMode) -> Result<(), HostError> {
        if mode != GpuMode::Required {
            return Ok(());
        }
        let started = std::time::Instant::now();
        let main = WindowId::MAIN;
        while self.renderer(main) != Some(Renderer::Gpu)
            && self.failure().is_none()
            && !matches!(
                self.renderer.borrow().get(&main),
                Some(RendererState::Failed(_))
            )
            && started.elapsed() < std::time::Duration::from_secs(5)
        {
            std::thread::park_timeout(std::time::Duration::from_millis(5));
            self.step(Step::Wake);
        }
        if let Some(failure) = self.failure() {
            return Err(HostError::new(failure.to_string()));
        }
        match self.renderer.borrow().get(&main) {
            Some(RendererState::Ready(Renderer::Gpu)) => Ok(()),
            Some(RendererState::Failed(reason)) => Err(HostError::new(reason.clone())),
            _ => Err(HostError::new(
                "GPU renderer did not start within five seconds",
            )),
        }
    }

    fn step(&mut self, step: Step) {
        self.outcome = self.shell.step(step);
    }

    // ----- input and lifecycle -----

    /// The virtual platform's current mouse position inside window `window`.
    pub fn mouse_position(&self, window: WindowId) -> Option<Point> {
        self.probe
            .window(window)
            .and_then(|probe| probe.mouse_position())
    }

    /// Modifiers currently reported by the virtual platform for `window`.
    pub fn modifiers(&self, window: WindowId) -> Modifiers {
        self.probe
            .window(window)
            .map_or(Modifiers::default(), |probe| probe.modifiers())
    }

    /// Normalize raw platform input through the shared virtual script.
    pub fn raw_input(&mut self, window: WindowId, input: arvane_virtual::raw::RawInput) {
        self.script.window(window).raw(input);
        self.step(Step::Input);
    }

    /// Deliver one shell event of `window` at the host's entry, as the shell
    /// delivers the winit events it maps. Input is applied to the host model at
    /// once; the client's turn for it runs at the next [`Self::wake`] (the
    /// host reports itself [`Self::due`]), as in a running App.
    pub fn send(&mut self, window: WindowId, event: ShellEvent) {
        match event {
            ShellEvent::Focused(focused) => self.script.window(window).focus(focused),
            ShellEvent::Accessibility { node, request } => self
                .script
                .window(window)
                .accessibility_action(node, request),
            event if EventTarget::of(&event, window) == EventTarget::Loop => {
                self.script.input(event)
            }
            event => self.script.window(window).input(event),
        }
        self.step(Step::Input);
    }

    /// Deliver the mouse event of `phase` as window `window` makes it
    /// (`Script::mouse`): at `position` (`None`: where the mouse
    /// last was), with the buttons held so far and a `Down`'s or `Up`'s
    /// button. Events without a known mouse position are ignored.
    pub fn mouse(
        &mut self,
        window: WindowId,
        phase: kivact_types::PointerPhase,
        position: Option<kivact_types::Point>,
    ) {
        self.script.window(window).mouse(phase, position);
        self.step(Step::Input);
    }
    /// A touch or pen contact as the platform reports it (`contact.id` is
    /// the platform's id of the contact), delivered as a native window
    /// delivers its contacts: the window gives it a pointer id of its own
    /// (never the mouse's), counts its taps and drops what it would not
    /// report (a move or an end of a contact that never began).
    pub fn contact(&mut self, window: WindowId, contact: PointerSample) {
        self.script.window(window).contact(contact);
        self.step(Step::Input);
    }
    /// Terminal shell failure.
    pub fn failure(&self) -> Option<&ShellError> {
        if let StepOutcome::Failed(error) = &self.outcome {
            Some(error)
        } else {
            None
        }
    }
    /// Whether the shell exited.
    pub fn exited(&self) -> bool {
        matches!(self.outcome, StepOutcome::Exited)
    }
    // ----- clock and scheduling -----

    /// The virtual clock (ns since the host started).
    pub fn now_ns(&self) -> u64 {
        self.probe.now_ns()
    }

    /// Move the virtual clock to `ns` (never backwards). Nothing runs until
    /// the next [`Self::wake`].
    pub fn set_now(&mut self, ns: u64) {
        self.script.set_now(ns);
    }

    /// The work counts of the last frame presented by window `window`, or
    /// `None` while the host has not started or the window has presented none.
    pub fn frame_stats(&self, window: WindowId) -> Option<kivact_types::render::FrameStats> {
        match &*self.host.borrow() {
            Launch::Running(host) => host.windows.get(&window).and_then(|w| w.last_stats),
            _ => None,
        }
    }

    /// Make window `window`'s GPU device lost, as a device failure does (a test
    /// control; nothing happens on a CPU-only window).
    pub fn lose_gpu_device(&mut self, window: WindowId) {
        self.controls
            .borrow_mut()
            .push(Control::LoseGpuDevice(window));
        self.step(Step::Wake);
    }

    /// The display refresh period frame ticks are paced at (ns) for `window`.
    pub fn frame_period_ns(&self, window: WindowId) -> u64 {
        self.probe
            .window(window)
            .map(|probe| probe.environment().frame_period_ns)
            .expect("invariant: the asked window is open")
    }

    /// When the host next wants to run (virtual ns): queued events at once,
    /// requested frames and the client's timers.
    pub fn next_deadline(&self) -> Option<u64> {
        match self.outcome {
            StepOutcome::Idle { next_deadline, .. } => next_deadline,
            _ => None,
        }
    }

    /// Whether the host wants to run now.
    pub fn due(&self) -> bool {
        if !self.script.is_empty() {
            return true;
        }
        self.next_deadline().is_some_and(|t| t <= self.now_ns())
    }

    /// Whether a worker thread (a blocking job, the asset store) asked for
    /// a wake since the last [`Self::wake`]. A wake unparks the thread that
    /// created the driver, so it can wait for one with
    /// `std::thread::park_timeout`.
    pub fn woken(&self) -> bool {
        matches!(self.outcome, StepOutcome::Idle { .. }) && self.probe.woken()
    }

    /// What the shell does on a wake or a reached deadline: frame ticks due
    /// at the virtual time, the asset store's results and the client's
    /// turn.
    pub fn wake(&mut self) {
        if !self.script.is_empty() {
            self.step(Step::Input);
        }
        self.step(Step::Wake);
    }

    // ----- frames -----

    /// Whether the host asked the window for a redraw that has not been
    /// drawn yet ([`Self::redraw`]).
    pub fn redraw_requested(&self) -> bool {
        matches!(
            self.outcome,
            StepOutcome::Idle {
                redraw_requested: true,
                ..
            }
        )
    }

    /// Render and present a frame if the host asked for one (what the
    /// OS's paint message does). Returns whether it did.
    pub fn redraw(&mut self) -> bool {
        let requested = self.redraw_requested();
        self.step(Step::Redraw);
        requested
    }
    /// The rasterizer that drew window `window`'s last presented frame.
    pub fn renderer(&self, window: WindowId) -> Option<Renderer> {
        match self.renderer.borrow().get(&window) {
            None => None,
            Some(RendererState::Ready(renderer)) => Some(*renderer),
            Some(RendererState::Failed(_)) => Some(Renderer::Cpu),
        }
    }

    /// The last presented frame of window `window`.
    pub fn frame(&self, window: WindowId) -> Option<TestFrame> {
        self.probe
            .window(window)?
            .frame()
            .map(|(width, height, pixels)| TestFrame {
                width,
                height,
                pixels,
            })
    }

    // ----- environment -----

    /// Resize window `window`'s content to `width` × `height` logical pixels.
    pub fn resize(&mut self, window: WindowId, width: f64, height: f64) {
        self.script.window(window).resize(Size { width, height });
        self.step(Step::Input);
    }

    /// Change window `window`'s scale (physical pixels per logical pixel).
    pub fn set_scale(&mut self, window: WindowId, scale: f64) {
        self.script.window(window).set_scale(scale);
        self.step(Step::Input);
    }

    /// Change the system theme of `window` and report it (`EnvChanged` when
    /// it changed).
    /// Update pointer preferences as a platform environment change.
    pub fn set_pointer_settings(&mut self, settings: kivact_types::PointerSettings) {
        self.script.pointer_settings(settings);
        self.step(Step::Input);
    }

    pub fn set_theme(&mut self, window: WindowId, theme: ThemeHint) {
        self.script
            .window(window)
            .set_dark(theme == ThemeHint::Dark);
        self.step(Step::Input);
    }

    /// Change window `window`'s system locale (`None`: the system names none)
    /// and report it (`EnvChanged` when it changed).
    pub fn set_locale(&mut self, window: WindowId, locale: Option<&str>) {
        self.script.window(window).set_locale(locale);
        self.step(Step::Input);
    }

    /// Move window `window` to a display refreshing every `period_ns` ns, as
    /// the shell reports a move to another monitor (`EnvChanged` when the
    /// period changes).
    pub fn set_display_period(&mut self, window: WindowId, period_ns: u64) {
        self.script.window(window).set_frame_period(period_ns);
        self.step(Step::Input);
    }

    /// Change window `window`'s reduced-motion setting and report it
    /// (`EnvChanged` when it changed).
    pub fn set_reduced_motion(&mut self, window: WindowId, on: bool) {
        self.script.window(window).set_reduced_motion(on);
        self.step(Step::Input);
    }

    /// Change window `window`'s safe area, as a phone's system UI does
    /// (`EnvChanged` when it changed).
    pub fn set_safe_area(&mut self, window: WindowId, insets: Insets) {
        self.script.window(window).set_safe_area(insets);
        self.step(Step::Input);
    }

    /// Lose (`false`) or regain window `window`'s surface, as when the
    /// platform destroys it and gives the window a new one (the next surface
    /// generation).
    pub fn set_surface(&mut self, window: WindowId, active: bool) {
        if active {
            self.script.window(window).create_surface();
        } else {
            self.script.window(window).destroy_surface();
        }
        self.step(Step::Input);
    }

    /// The platform suspends the application (sends it to the background:
    /// Android, iOS), destroying the window's surface with it when
    /// `destroys_surface` (Android).
    pub fn suspend(&mut self, destroys_surface: bool) {
        self.send(WindowId::MAIN, ShellEvent::Suspended);
        if destroys_surface {
            self.script.window(WindowId::MAIN).destroy_surface();
            self.step(Step::Input);
        }
    }

    /// The suspended application comes back to the foreground, with a new
    /// surface if its surface was destroyed.
    pub fn resume(&mut self) {
        let main = WindowId::MAIN;
        let none = self
            .probe
            .window(main)
            .is_some_and(|probe| probe.surface_generation().is_none());
        if none {
            self.script.window(main).create_surface();
            self.step(Step::Input);
        }
        self.send(main, ShellEvent::Resumed);
    }

    /// The platform warns that memory is short (`MemoryWarning`).
    pub fn memory_warning(&mut self) {
        self.send(WindowId::MAIN, ShellEvent::MemoryWarning);
    }

    /// The generation of window `window`'s surface (`None`: it has none).
    pub fn surface_generation(&self, window: WindowId) -> Option<u64> {
        self.probe
            .window(window)
            .and_then(|probe| probe.surface_generation())
    }

    /// The user asks to close window `window` (`CloseRequested`).
    pub fn request_close(&mut self, window: WindowId) {
        self.send(window, ShellEvent::CloseRequested);
    }

    /// Queue any host event (failures, stale input ..) and run the turn.
    pub fn inject(&mut self, event: HostEvent) {
        if self.failure().is_some() || self.exited() {
            return;
        }
        self.injected.borrow_mut().push(event);
        self.step(Step::Wake);
    }
    // ----- platform services -----

    /// The in-memory clipboard.
    pub fn clipboard(&self) -> String {
        self.probe.clipboard()
    }

    /// Put `text` on the in-memory clipboard.
    pub fn set_clipboard(&mut self, text: &str) {
        self.script.set_clipboard(text);
        self.step(Step::Input);
    }

    /// Script the answer of the next file, save or folder dialog (`None`:
    /// cancelled). An unscripted dialog is cancelled.
    pub fn push_file_dialog(&mut self, path: Option<PathBuf>) {
        self.script.push_file_dialog(path);
        self.step(Step::Input);
    }

    /// Script the next multiple-file picker response; `None` cancels.
    pub fn push_files_dialog(&mut self, paths: Option<Vec<PathBuf>>) {
        self.script.push_files_dialog(paths);
        self.step(Step::Input);
    }

    /// The paths the App asked to open with the default handler, in order
    /// (recorded; nothing is started).
    pub fn opened_paths(&self) -> Vec<PathBuf> {
        self.probe.opened_paths()
    }

    /// The paths the App asked to reveal in the file manager, in order
    /// (recorded; nothing is started).
    pub fn revealed_paths(&self) -> Vec<PathBuf> {
        self.probe.revealed_paths()
    }

    /// Make the next platform request (clipboard, dialog, open or reveal)
    /// fail with `reason`. Font registrations are not platform requests
    /// here.
    pub fn fail_next_effect(&mut self, reason: &str) {
        self.script.fail_next_effect(reason);
        self.step(Step::Input);
    }

    /// Make the next platform request answer as on a platform without
    /// the service (`PlatformResult::Unsupported`).
    pub fn fail_next_effect_unsupported(&mut self, reason: &str) {
        self.script.fail_next_effect_unsupported(reason);
        self.step(Step::Input);
    }

    // ----- what the host shows -----

    /// The accessibility tree last handed to window `window`'s platform (what
    /// assistive technology sees). A text input's editing snapshot in it (its
    /// properties' `text`: the runs, selection and caret actually
    /// published) is what a test reads its published selection and caret
    /// from; a password has none.
    pub fn semantics(&self, window: WindowId) -> Option<SemanticsTree> {
        self.probe
            .window(window)
            .and_then(|probe| probe.accessibility())
    }

    /// The node, or the node and scene region, an accessibility id of
    /// window `window` stands for (node ids are Kivact node ids; regions
    /// count down from `u64::MAX`).
    pub fn semantics_target(
        &self,
        window: WindowId,
        id: u64,
    ) -> Option<(NodeId, Option<RegionId>)> {
        self.with_window(window, |model| match model.a11y.region_of(id) {
            Some((node, region)) => Some((node, Some(region))),
            None => model.node(NodeId(id)).map(|_| (NodeId(id), None)),
        })
        .flatten()
    }

    /// The accessibility id of a node or of one of its scene regions in
    /// window `window`.
    pub fn semantics_id(
        &self,
        window: WindowId,
        node: NodeId,
        region: Option<RegionId>,
    ) -> Option<u64> {
        self.with_window(window, |model| match region {
            Some(region) => model.a11y.region_id(node, region),
            None => model.node(node).map(|_| node.0),
        })
        .flatten()
    }

    /// Where the box of `id` is drawn, in logical window coordinates: its
    /// layout box mapped by its transforms (their bounds).
    pub fn rect(&self, id: NodeId) -> Option<Rect> {
        self.with_node(id, |model| model.semantic_bounds(id))
            .flatten()
    }

    /// The layout size of `id` (its box before its transforms).
    pub fn size(&self, id: NodeId) -> Option<Size> {
        self.with_node(id, |model| model.computed_layout(id).map(|l| l.size))
            .flatten()
    }

    /// The rect of scene region `region` of `id`, relative to the node's
    /// untransformed layout box.
    pub fn region_local(&self, id: NodeId, region: RegionId) -> Option<Rect> {
        self.with_node(id, |model| Some(model.node(id)?.region(region)?.rect))
            .flatten()
    }

    /// Where scene region `region` of `id` is drawn, in logical window
    /// coordinates (the bounds of its rect mapped by the node's
    /// transforms).
    pub fn region_rect(&self, id: NodeId, region: RegionId) -> Option<Rect> {
        self.with_node(id, |model| {
            let r = model.node(id)?.region(region)?.rect;
            let bounds = model.rect(id)?;
            let own = Rect {
                x: bounds.x + r.x,
                y: bounds.y + r.y,
                ..r
            };
            Some(model.drawn_rect(id, own))
        })
        .flatten()
    }

    /// The window point drawn at `local`, a point relative to the
    /// untransformed layout box of `id` (mapped by its transforms).
    pub fn window_point(&self, id: NodeId, local: Point) -> Point {
        self.with_node(id, |model| model.local_to_window(id, local))
            .unwrap_or_default()
    }

    /// The node hit at a logical window point of window `window`.
    pub fn hit(&self, window: WindowId, x: f64, y: f64) -> Option<NodeId> {
        self.with_window(window, |model| model.hit(Point { x, y }))
            .flatten()
    }

    /// The node with the keyboard focus in window `window`.
    pub fn focused(&self, window: WindowId) -> Option<kivact_types::FocusTarget> {
        self.with_window(window, |model| model.focused()).flatten()
    }

    /// The node under the pointer in window `window`.
    pub fn hovered(&self, window: WindowId) -> Option<NodeId> {
        self.with_window(window, |model| model.hovered()).flatten()
    }

    /// The open modal that confines window `window`'s input, if any.
    pub fn modal(&self, window: WindowId) -> Option<NodeId> {
        self.with_window(window, |model| model.modal()).flatten()
    }

    /// Window `window`'s tree in preorder (from the root), without the
    /// subtrees leaving after kivact removed them (they are kivact's no more).
    pub fn preorder(&self, window: WindowId) -> Vec<NodeId> {
        self.with_window(window, |model| {
            let tree = model.tree();
            tree.preorder()
                .into_iter()
                .filter(|id| tree.live(*id))
                .collect()
        })
        .unwrap_or_default()
    }

    /// Nodes the host holds for kivact in the main window's tree (reachable
    /// or not; not the leaving ones).
    pub fn node_count(&self) -> usize {
        self.with_window(WindowId::MAIN, |model| {
            let tree = model.tree();
            tree.iter().filter(|(id, _)| tree.live(*id)).count()
        })
        .unwrap_or_default()
    }

    /// The layout defects of the texts and text inputs in the subtree of
    /// `scope` of window `window` (the whole tree without one), in tree
    /// order, with the glyphs' ink as the vertical extent of a line. Nothing
    /// is changed or sent.
    pub fn layout_findings(
        &self,
        window: WindowId,
        check: &LayoutCheck,
        scope: Option<NodeId>,
    ) -> Vec<LayoutFinding> {
        let launch = self.host.borrow();
        let host = launch.host();
        let mut app = host.app.borrow_mut();
        let Some(model) = app.model_mut(window) else {
            return Vec::new();
        };
        let mut engine = host.engine.borrow_mut();
        crate::check::layout_findings(model, &mut engine, check, scope, true)
    }

    /// The kind of node `id`.
    pub fn kind(&self, id: NodeId) -> Option<NodeType> {
        self.with_node(id, |model| model.node(id).map(|n| n.kind()))
            .flatten()
    }

    /// The children of node `id`, in order (not the leaving ones).
    pub fn children(&self, id: NodeId) -> Option<Vec<NodeId>> {
        self.with_node(id, |model| {
            model.node(id).map(|_| model.tree.logical_children(id))
        })
        .flatten()
    }

    /// The props node `id` holds (as the client sent them; a text input's
    /// value is [`Self::input_value`]).
    pub fn properties(&self, id: NodeId) -> Option<NodeProperties> {
        self.with_node(id, |model| model.node(id).map(|n| n.properties.clone()))
            .flatten()
    }

    /// The value of text input `id` as the host edits it.
    pub fn input_value(&self, id: NodeId) -> Option<String> {
        self.with_node(id, |model| {
            model.node(id).map(|_| model.input_value(id).to_owned())
        })
        .flatten()
    }

    /// The draw and draw-over scenes of node `id`.
    pub fn scenes(&self, id: NodeId) -> Option<[Option<Arc<DrawList>>; 2]> {
        self.with_node(id, |model| model.node(id).map(|n| n.scenes().clone()))
            .flatten()
    }

    /// Whether node `id` is shown (it and its ancestors are not hidden).
    pub fn shown(&self, id: NodeId) -> bool {
        self.with_node(id, |model| model.tree().shown(id))
            .unwrap_or(false)
    }

    /// Whether the focused text input of the main window holds an IME
    /// composition.
    pub fn composing(&self) -> bool {
        self.with_window(WindowId::MAIN, |model| {
            model
                .focused_input()
                .and_then(|i| model.node(i))
                .and_then(|n| {
                    model.editors[n.index.0]
                        .as_deref()
                        .map(|editor| editor.editor.state())
                })
                .is_some_and(kivact_editing::EditState::is_composing)
        })
        .unwrap_or(false)
    }

    /// Whether text can be shaped with the font family `family` (bundled,
    /// registered by the App, or a system font with [`TestFonts::System`]).
    pub fn has_font_family(&mut self, family: &str) -> bool {
        self.host
            .borrow()
            .host()
            .engine
            .borrow_mut()
            .has_font_family(family)
    }

    /// The title the App set for window `window`.
    pub fn title(&self, window: WindowId) -> String {
        self.probe
            .window(window)
            .map(|probe| probe.title())
            .unwrap_or_default()
    }

    /// Requested logical content sizes of window `window`, in order.
    pub fn requested_sizes(&self, window: WindowId) -> Vec<Size> {
        self.probe
            .window(window)
            .map(|probe| probe.requested_sizes())
            .unwrap_or_default()
    }

    /// The minimum window size the App set for `window` (`None`: no minimum).
    pub fn min_window_size(&self, window: WindowId) -> Option<Size> {
        self.probe.window(window).and_then(|probe| probe.min_size())
    }

    /// The IME caret area (logical) of `window` while a text input has the focus.
    pub fn ime_area(&self, window: WindowId) -> Option<Rect> {
        self.probe.window(window).and_then(|probe| probe.ime_area())
    }

    /// The cursor window `window` shows over its content.
    pub fn cursor(&self, window: WindowId) -> Cursor {
        self.probe
            .window(window)
            .map(|probe| probe.cursor())
            .unwrap_or(Cursor::Default)
    }

    /// Whether the host asked window `window` to show the software keyboard,
    /// and how often it asked to show or hide it.
    pub fn soft_keyboard(&self, window: WindowId) -> (bool, u32) {
        self.probe
            .window(window)
            .map(|probe| probe.soft_keyboard())
            .unwrap_or((false, 0))
    }

    /// The focused editor's purpose delivered to window `window`'s platform.
    pub fn ime_purpose(&self, window: WindowId) -> kivact_types::InputPurpose {
        self.probe
            .window(window)
            .map(|probe| probe.ime_purpose())
            .unwrap_or_default()
    }
    /// What the host gave window `window` for IME surrounding-text queries.
    pub fn ime_text(&self, window: WindowId) -> Option<ShellImeText> {
        self.probe.window(window).and_then(|probe| probe.ime_text())
    }

    // ----- counters -----

    /// Client turns run so far.
    pub fn turns(&self) -> u64 {
        self.counts.turns.get()
    }

    /// Frame ticks delivered to the client so far.
    pub fn frame_ticks(&self) -> u64 {
        self.counts.frame_ticks.get()
    }

    // ----- the real window -----

    /// Whether frames of window `window` are mirrored to a real window.
    pub fn has_window(&self, window: WindowId) -> bool {
        self.probe
            .window(window)
            .is_some_and(|probe| probe.has_window())
    }

    /// Wait until the real window shows everything window `window` presented
    /// so far.
    pub fn sync_window(&self, window: WindowId) {
        if let Some(probe) = self.probe.window(window) {
            probe.sync_mirror();
        }
    }

    /// The open windows, ascending by id.
    pub fn windows(&self) -> Vec<WindowId> {
        self.probe.windows()
    }

    /// The title the platform shows for `window`.
    pub fn window_title(&self, window: WindowId) -> Option<String> {
        self.probe.window(window).map(|probe| probe.title())
    }

    /// Physical pixels per logical pixel of the test window `window`, as the
    /// window reports it. `None` when the window is closed or was never
    /// opened.
    pub fn scale(&self, window: WindowId) -> Option<f64> {
        self.probe
            .window(window)
            .map(|probe| probe.environment().scale_factor)
    }

    /// How often the platform asked window `window` to take the focus.
    pub fn focus_requests(&self, window: WindowId) -> u32 {
        self.probe
            .window(window)
            .map_or(0, |probe| probe.focus_requests())
    }

    /// Run `f` on window `window`'s model.
    fn with_window<R>(&self, window: WindowId, f: impl FnOnce(&Model) -> R) -> Option<R> {
        let launch = self.host.borrow();
        let host = launch.host();
        let app = host.app.borrow();
        app.model(window).map(f)
    }

    /// Run `f` on the model that holds `node`.
    fn with_node<R>(&self, node: NodeId, f: impl FnOnce(&Model) -> R) -> Option<R> {
        let launch = self.host.borrow();
        let host = launch.host();
        let app = host.app.borrow();
        let window = app.window_of(node)?;
        app.model(window).map(f)
    }
}
