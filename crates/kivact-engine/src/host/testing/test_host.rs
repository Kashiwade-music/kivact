//! Role: Drive a client test window and query its settled engine model.

use super::test_frame::TestFrame;
use super::test_options::{TestFonts, TestHostOptions, default_font};
use crate::contract::{HostError, HostEvent, NodeProperties, NodeType};
use crate::host::Client;
use crate::host::launch::{AssetMode, Launch};
use arvane::spi::{Step, StepOutcome};
use arvane::{
    Handler, Renderer, RendererChange, Shell, ShellControl, ShellError, ShellEvent, ShellImeText,
    ShellSetup, ShellWaker, SteppedShell, Window,
};
use arvane_virtual::{GpuMode, Probe, Script, VirtualBackend, VirtualOptions};
use kivact_types::PointerSample;
use kivact_types::render::SemanticsTree;
use kivact_types::{Cursor, Insets, Modifiers, Point};
use kivact_types::{DrawList, LayoutCheck, LayoutFinding, NodeId, Rect, RegionId, Size, ThemeHint};
use std::cell::{Cell, RefCell};
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
    renderer: Rc<RefCell<RendererState>>,
    injected: Rc<RefCell<Vec<HostEvent>>>,
    counts: Rc<Counts>,
}
enum RendererState {
    Unreported,
    Ready(Renderer),
    Failed(String),
}

struct SharedHost {
    host: Rc<RefCell<Launch>>,
    injected: Rc<RefCell<Vec<HostEvent>>>,
}
impl Handler for SharedHost {
    fn start(
        &mut self,
        window: &mut Window<'_>,
        waker: ShellWaker,
    ) -> Result<ShellControl, ShellError> {
        self.host.borrow_mut().start(window, waker)
    }
    fn event(
        &mut self,
        event: ShellEvent,
        window: &mut Window<'_>,
    ) -> Result<ShellControl, ShellError> {
        let mut launch = self.host.borrow_mut();
        let host = launch.host_mut();
        for injected in self.injected.borrow_mut().drain(..) {
            let control = host.inject(injected, window)?;
            if control == ShellControl::Exit {
                return Ok(control);
            }
        }
        host.handle(event, window)
    }
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
        crate::api::Writer::new(&host.model, &host.engine)
    }
    /// Queue platform prerequisites and input for this host.
    pub fn script(&self) -> arvane_virtual::Script {
        self.script.clone()
    }

    /// Start `client` on the host (its `start` runs here, and the turns its
    /// first revision asks for).
    pub fn new(client: Client, options: TestHostOptions) -> Result<Self, HostError> {
        let error = |e: &dyn std::fmt::Display| HostError::new(e.to_string());
        let backend = VirtualBackend::new(VirtualOptions {
            title: options.title.clone(),
            size: Size {
                width: options.width,
                height: options.height,
            },
            scale: options.scale,
            frame_period_ns: options.frame_period_ns,
            reduced_motion: options.reduced_motion,
            dark: options.theme == ThemeHint::Dark,
            locale: options.locale.clone(),
            mirror: options.mirror,
            gpu: options.gpu,
        });
        let script = backend.script();
        let probe = backend.probe();
        let counts = Rc::new(Counts::default());
        let host = make_host(client, &options, counts.clone())?;
        let renderer = Rc::new(RefCell::new(RendererState::Unreported));
        let observed = renderer.clone();
        let injected = Rc::new(RefCell::new(Vec::new()));
        let shell = Shell::start_stepped(
            Box::new(backend),
            ShellSetup {
                prefer_gpu: options.gpu == GpuMode::Required,
                gpu_preference: options.gpu_preference,
                observer: Some(Box::new(move |change| match change {
                    RendererChange::Start { .. } => {
                        *observed.borrow_mut() = RendererState::Ready(Renderer::Cpu)
                    }
                    RendererChange::Gpu { .. } => {
                        *observed.borrow_mut() = RendererState::Ready(Renderer::Gpu)
                    }
                    RendererChange::Cpu { reason } => {
                        *observed.borrow_mut() = RendererState::Failed(reason.clone());
                    }
                    RendererChange::ExternalFrameMissed { .. } => {}
                })),
            },
            SharedHost {
                host: host.clone(),
                injected: injected.clone(),
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
        while self.renderer() != Some(Renderer::Gpu)
            && self.failure().is_none()
            && !matches!(*self.renderer.borrow(), RendererState::Failed(_))
            && started.elapsed() < std::time::Duration::from_secs(5)
        {
            std::thread::park_timeout(std::time::Duration::from_millis(5));
            self.step(Step::Wake);
        }
        if let Some(failure) = self.failure() {
            return Err(HostError::new(failure.to_string()));
        }
        match &*self.renderer.borrow() {
            RendererState::Ready(Renderer::Gpu) => Ok(()),
            RendererState::Failed(reason) => Err(HostError::new(reason.clone())),
            _ => Err(HostError::new(
                "GPU renderer did not start within five seconds",
            )),
        }
    }

    fn step(&mut self, step: Step) {
        self.outcome = self.shell.step(step);
    }

    // ----- input and lifecycle -----

    /// The virtual platform's current mouse position inside the window.
    pub fn mouse_position(&self) -> Option<Point> {
        self.probe.mouse_position()
    }

    /// Modifiers currently reported by the virtual platform.
    pub fn modifiers(&self) -> Modifiers {
        self.probe.modifiers()
    }

    /// Normalize raw platform input through the shared virtual script.
    pub fn raw_input(&mut self, input: arvane_virtual::raw::RawInput) {
        self.script.raw(input);
        self.step(Step::Input);
    }

    /// Deliver one shell event at the host's entry, as the shell delivers
    /// the winit events it maps. Input is applied to the host model at
    /// once; the client's turn for it runs at the next [`Self::wake`] (the
    /// host reports itself [`Self::due`]), as in a running App.
    pub fn send(&mut self, event: ShellEvent) {
        match event {
            ShellEvent::Focused(focused) => self.script.focus(focused),
            ShellEvent::Accessibility { node, request } => {
                self.script.accessibility_action(node, request)
            }
            event => self.script.input(event),
        }
        self.step(Step::Input);
    }

    /// Deliver the mouse event of `phase` as the test window makes it
    /// (`Script::mouse`): at `position` (`None`: where the mouse
    /// last was), with the buttons held so far and a `Down`'s or `Up`'s
    /// button. Events without a known mouse position are ignored.
    pub fn mouse(
        &mut self,
        phase: kivact_types::PointerPhase,
        position: Option<kivact_types::Point>,
    ) {
        self.script.mouse(phase, position);
        self.step(Step::Input);
    }
    /// A touch or pen contact as the platform reports it (`contact.id` is
    /// the platform's id of the contact), delivered as a native window
    /// delivers its contacts: the window gives it a pointer id of its own
    /// (never the mouse's), counts its taps and drops what it would not
    /// report (a move or an end of a contact that never began).
    pub fn contact(&mut self, contact: PointerSample) {
        self.script.contact(contact);
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

    /// The display refresh period frame ticks are paced at (ns).
    pub fn frame_period_ns(&self) -> u64 {
        self.host.borrow().host().model.borrow().frame.period_ns()
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
        self.next_deadline().is_some_and(|t| t <= self.now_ns())
    }

    /// Whether a worker thread (a blocking job, the asset store) asked for
    /// a wake since the last [`Self::wake`]. A wake unparks the thread that
    /// created the driver, so it can wait for one with
    /// `std::thread::park_timeout`.
    pub fn woken(&self) -> bool {
        matches!(self.outcome, StepOutcome::Idle { woken: true, .. })
    }

    /// What the shell does on a wake or a reached deadline: frame ticks due
    /// at the virtual time, the asset store's results and the client's
    /// turn.
    pub fn wake(&mut self) {
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
    /// The rasterizer that drew the last presented frame.
    pub fn renderer(&self) -> Option<Renderer> {
        match *self.renderer.borrow() {
            RendererState::Unreported => None,
            RendererState::Ready(renderer) => Some(renderer),
            RendererState::Failed(_) => Some(Renderer::Cpu),
        }
    }

    /// The last presented frame.
    pub fn frame(&self) -> Option<TestFrame> {
        self.probe.frame().map(|(width, height, pixels)| TestFrame {
            width,
            height,
            pixels,
        })
    }

    // ----- environment -----

    /// Resize the content to `width` × `height` logical pixels.
    pub fn resize(&mut self, width: f64, height: f64) {
        self.script.resize(Size { width, height });
        self.step(Step::Input);
    }

    /// Change the scale (physical pixels per logical pixel).
    pub fn set_scale(&mut self, scale: f64) {
        self.script.set_scale(scale);
        self.step(Step::Input);
    }

    /// Change the system theme and report it (`EnvChanged` when it
    /// changed).
    pub fn set_theme(&mut self, theme: ThemeHint) {
        self.script.set_dark(theme == ThemeHint::Dark);
        self.step(Step::Input);
    }

    /// Change the system locale (`None`: the system names none) and report
    /// it (`EnvChanged` when it changed).
    pub fn set_locale(&mut self, locale: Option<&str>) {
        self.script.set_locale(locale);
        self.step(Step::Input);
    }

    /// Move the window to a display refreshing every `period_ns` ns, as
    /// the shell reports a move to another monitor (`EnvChanged` when the
    /// period changes).
    pub fn set_display_period(&mut self, period_ns: u64) {
        self.script.set_frame_period(period_ns);
        self.step(Step::Input);
    }

    /// Change the reduced-motion setting and report it (`EnvChanged` when
    /// it changed).
    pub fn set_reduced_motion(&mut self, on: bool) {
        self.script.set_reduced_motion(on);
        self.step(Step::Input);
    }

    /// Change the safe area, as a phone's system UI does (`EnvChanged`
    /// when it changed).
    pub fn set_safe_area(&mut self, insets: Insets) {
        self.script.set_safe_area(insets);
        self.step(Step::Input);
    }

    /// Lose (`false`) or regain the window's surface, as when the platform
    /// destroys it and gives the window a new one (the next surface
    /// generation).
    pub fn set_surface(&mut self, active: bool) {
        if active {
            self.script.create_surface();
            self.step(Step::Input);
        } else {
            self.script.destroy_surface();
            self.step(Step::Input);
        }
    }

    /// The platform suspends the application (sends it to the background:
    /// Android, iOS), destroying the window's surface with it when
    /// `destroys_surface` (Android).
    pub fn suspend(&mut self, destroys_surface: bool) {
        self.send(ShellEvent::Suspended);
        if destroys_surface {
            self.script.destroy_surface();
            self.step(Step::Input);
        }
    }

    /// The suspended application comes back to the foreground, with a new
    /// surface if its surface was destroyed.
    pub fn resume(&mut self) {
        if self.probe.surface_generation().is_none() {
            self.script.create_surface();
            self.step(Step::Input);
        }
        self.send(ShellEvent::Resumed);
    }

    /// The platform warns that memory is short (`MemoryWarning`).
    pub fn memory_warning(&mut self) {
        self.send(ShellEvent::MemoryWarning);
    }

    /// The generation of the window's surface (`None`: it has none).
    pub fn surface_generation(&self) -> Option<u64> {
        self.probe.surface_generation()
    }

    /// The user asks to close the window (`CloseRequested`).
    pub fn request_close(&mut self) {
        self.send(ShellEvent::CloseRequested);
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

    /// The accessibility tree last handed to the platform (what assistive
    /// technology sees). A text input's editing snapshot in it (its
    /// properties' `text`: the runs, selection and caret actually
    /// published) is what a test reads its published selection and caret
    /// from; a password has none.
    pub fn semantics(&self) -> Option<SemanticsTree> {
        self.probe.accessibility()
    }

    /// The node, or the node and scene region, an accessibility id stands
    /// for (node ids are Kivact node ids; regions count down from
    /// `u64::MAX`).
    pub fn semantics_target(&self, id: u64) -> Option<(NodeId, Option<RegionId>)> {
        let launch = self.host.borrow();
        let host = launch.host();
        let model = host.model.borrow();
        match model.a11y.region_of(id) {
            Some((node, region)) => Some((node, Some(region))),
            None => model.node(NodeId(id)).map(|_| (NodeId(id), None)),
        }
    }

    /// The accessibility id of a node or of one of its scene regions.
    pub fn semantics_id(&self, node: NodeId, region: Option<RegionId>) -> Option<u64> {
        let launch = self.host.borrow();
        let host = launch.host();
        let model = host.model.borrow();
        match region {
            Some(region) => model.a11y.region_id(node, region),
            None => model.node(node).map(|_| node.0),
        }
    }

    /// Where the box of `id` is drawn, in logical window coordinates: its
    /// layout box mapped by its transforms (their bounds).
    pub fn rect(&self, id: NodeId) -> Option<Rect> {
        self.host.borrow().host().model.borrow().bounds(id)
    }

    /// The layout size of `id` (its box before its transforms).
    pub fn size(&self, id: NodeId) -> Option<Size> {
        let launch = self.host.borrow();
        let host = launch.host();
        let model = host.model.borrow();
        model.computed_layout(id).map(|l| l.size)
    }

    /// The rect of scene region `region` of `id`, relative to the node's
    /// untransformed layout box.
    pub fn region_local(&self, id: NodeId, region: RegionId) -> Option<Rect> {
        let launch = self.host.borrow();
        let host = launch.host();
        let model = host.model.borrow();
        Some(model.node(id)?.region(region)?.rect)
    }

    /// Where scene region `region` of `id` is drawn, in logical window
    /// coordinates (the bounds of its rect mapped by the node's
    /// transforms).
    pub fn region_rect(&self, id: NodeId, region: RegionId) -> Option<Rect> {
        let r = self.region_local(id, region)?;
        let launch = self.host.borrow();
        let host = launch.host();
        let model = host.model.borrow();
        let bounds = model.rect(id)?;
        let own = Rect {
            x: bounds.x + r.x,
            y: bounds.y + r.y,
            ..r
        };
        let t = model.transform_between(id, None);
        Some(crate::model::transform::map_rect(t.as_ref(), own))
    }

    /// The window point drawn at `local`, a point relative to the
    /// untransformed layout box of `id` (mapped by its transforms).
    pub fn window_point(&self, id: NodeId, local: Point) -> Point {
        self.host
            .borrow()
            .host()
            .model
            .borrow()
            .local_to_window(id, local)
    }

    /// The node hit at a logical window point.
    pub fn hit(&self, x: f64, y: f64) -> Option<NodeId> {
        self.host.borrow().host().model.borrow().hit(Point { x, y })
    }

    /// The node with the keyboard focus.
    pub fn focused(&self) -> Option<kivact_types::FocusTarget> {
        self.host.borrow().host().model.borrow().focused()
    }

    /// The node under the pointer.
    pub fn hovered(&self) -> Option<NodeId> {
        self.host.borrow().host().model.borrow().hovered()
    }

    /// The open modal that confines input, if any.
    pub fn modal(&self) -> Option<NodeId> {
        self.host.borrow().host().model.borrow().modal()
    }

    /// The host's tree in preorder (from the root), without the subtrees
    /// leaving after kivact removed them (they are kivact's no more).
    pub fn preorder(&self) -> Vec<NodeId> {
        let launch = self.host.borrow();
        let host = launch.host();
        let model = host.model.borrow();
        let tree = model.tree();
        tree.preorder()
            .into_iter()
            .filter(|id| tree.live(*id))
            .collect()
    }

    /// Nodes the host holds for kivact (reachable or not; not the leaving
    /// ones).
    pub fn node_count(&self) -> usize {
        let launch = self.host.borrow();
        let host = launch.host();
        let model = host.model.borrow();
        let tree = model.tree();
        tree.iter().filter(|(id, _)| tree.live(*id)).count()
    }

    /// The layout defects of the texts and text inputs in the subtree of
    /// `scope` (the whole tree without one), in tree order, with the glyphs'
    /// ink as the vertical extent of a line. Nothing is changed or sent.
    pub fn layout_findings(
        &self,
        check: &LayoutCheck,
        scope: Option<NodeId>,
    ) -> Vec<LayoutFinding> {
        let launch = self.host.borrow();
        let host = launch.host();
        let model = host.model.borrow();
        let mut engine = host.engine.borrow_mut();
        crate::check::layout_findings(&model, &mut engine, check, scope, true)
    }

    /// The kind of node `id`.
    pub fn kind(&self, id: NodeId) -> Option<NodeType> {
        self.host
            .borrow()
            .host()
            .model
            .borrow()
            .node(id)
            .map(|n| n.kind())
    }

    /// The children of node `id`, in order (not the leaving ones).
    pub fn children(&self, id: NodeId) -> Option<Vec<NodeId>> {
        let launch = self.host.borrow();
        let host = launch.host();
        let model = host.model.borrow();
        model.node(id).map(|_| model.tree.logical_children(id))
    }

    /// The props node `id` holds (as the client sent them; a text input's
    /// value is [`Self::input_value`]).
    pub fn properties(&self, id: NodeId) -> Option<NodeProperties> {
        let launch = self.host.borrow();
        let host = launch.host();
        let model = host.model.borrow();
        model.node(id).map(|n| n.properties.clone())
    }

    /// The value of text input `id` as the host edits it.
    pub fn input_value(&self, id: NodeId) -> Option<String> {
        let launch = self.host.borrow();
        let host = launch.host();
        let model = host.model.borrow();
        model.node(id).map(|_| model.input_value(id).to_owned())
    }

    /// The draw and draw-over scenes of node `id`.
    pub fn scenes(&self, id: NodeId) -> Option<[Option<Arc<DrawList>>; 2]> {
        let launch = self.host.borrow();
        let host = launch.host();
        let model = host.model.borrow();
        model.node(id).map(|n| n.scenes().clone())
    }

    /// Whether node `id` is shown (it and its ancestors are not hidden).
    pub fn shown(&self, id: NodeId) -> bool {
        self.host.borrow().host().model.borrow().tree().shown(id)
    }

    /// Whether the focused text input holds an IME composition.
    pub fn composing(&self) -> bool {
        let launch = self.host.borrow();
        let host = launch.host();
        let model = host.model.borrow();
        let host = &model;
        host.focused_input()
            .and_then(|i| host.node(i))
            .and_then(|n| {
                model.editors[n.index.0]
                    .as_deref()
                    .map(|editor| editor.editor.state())
            })
            .is_some_and(kivact_editing::EditState::is_composing)
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

    /// The window title the App set.
    pub fn title(&self) -> String {
        self.probe.title()
    }

    /// Requested logical content sizes in order.
    pub fn requested_sizes(&self) -> Vec<Size> {
        self.probe.requested_sizes()
    }

    /// The minimum window size the App set (`None`: no minimum).
    pub fn min_window_size(&self) -> Option<Size> {
        self.probe.min_size()
    }

    /// The IME caret area (logical) while a text input has the focus.
    pub fn ime_area(&self) -> Option<Rect> {
        self.probe.ime_area()
    }

    /// The cursor the window shows over its content.
    pub fn cursor(&self) -> Cursor {
        self.probe.cursor()
    }

    /// Whether the host asked the window to show the software keyboard,
    /// and how often it asked to show or hide it.
    pub fn soft_keyboard(&self) -> (bool, u32) {
        self.probe.soft_keyboard()
    }

    /// What the host gave the window for the IME's queries about the
    /// focused text input (what a native window answers a reconversion,
    /// document feed or character position request from).
    pub fn ime_text(&self) -> Option<ShellImeText> {
        self.probe.ime_text()
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

    /// Whether frames are mirrored to a real window.
    pub fn has_window(&self) -> bool {
        self.probe.has_window()
    }

    /// Wait until the real window shows everything presented so far.
    pub fn sync_window(&self) {
        self.probe.sync_mirror();
    }
}
