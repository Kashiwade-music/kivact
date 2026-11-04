//! Role: `ArvaneTestHost`: drive a client's window in process and read its state.

use super::test_frame::TestFrame;
use super::test_options::{TestFonts, TestHostOptions, default_font};
use crate::contract::{HostError, HostEvent, NodeProperties, NodeType};
use crate::host::pump::Host;
use crate::host::{Client, HostWaker};
use arvane::test_display::{VirtualWindow, VirtualWindowOptions};
use arvane::{Renderer, ShellControl, ShellError, ShellEvent, ShellImeText, Window as _};
use kivact_types::render::{PointerSample, SemanticsTree};
use kivact_types::{Cursor, Insets, Point};
use kivact_types::{DrawList, LayoutCheck, LayoutFinding, NodeId, Rect, RegionId, Size, ThemeHint};
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Turns and frame ticks the client was given.
#[derive(Default)]
struct Counts {
    turns: Cell<u64>,
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
    pub(super) host: Host,
    pub(super) window: VirtualWindow,
    control: ShellControl,
    woken: Arc<AtomicBool>,
    counts: Rc<Counts>,
    failure: Option<ShellError>,
}

impl ArvaneTestHost {
    /// Start `client` on the host (its `start` runs here, and the turns its
    /// first revision asks for).
    pub fn new(client: Client, options: TestHostOptions) -> Result<Self, HostError> {
        let error = |e: &dyn std::fmt::Display| HostError::new(e.to_string());
        let engine = match &options.fonts {
            TestFonts::Bundled => arvane::Engine::with_fonts_only(&[default_font()]),
            TestFonts::Only(fonts) => arvane::Engine::with_fonts_only(fonts),
            TestFonts::System => Ok(arvane::Engine::new()),
        }
        .map_err(|e| error(&e))?;
        let mut window = VirtualWindow::open(VirtualWindowOptions {
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
            mode: options.window,
            gpu: options.gpu,
            renderer: None,
            gpu_preference: options.gpu_preference,
        })
        .map_err(|e| error(&e))?;
        let counts = Rc::new(Counts::default());
        let mut host = crate::host::new(
            counted(client, counts.clone()),
            Size {
                width: options.width,
                height: options.height,
            },
            engine,
            Rc::new(RefCell::new(crate::FrameLog::disabled())),
        );
        // Reported light at start in production; a test may start dark.
        host.model.borrow_mut().frame.start_theme(options.theme);
        let woken = Arc::new(AtomicBool::new(false));
        let flag = woken.clone();
        // A wake also unparks the test thread, which may be waiting for a
        // worker (`std::thread::park_timeout`).
        let test_thread = std::thread::current();
        let waker = HostWaker::new(move || {
            flag.store(true, Ordering::SeqCst);
            test_thread.unpark();
        });
        let store = if options.inline_assets {
            arvane::asset::AssetStore::inline()
        } else {
            let waker = waker.clone();
            arvane::asset::AssetStore::new(move || waker.wake())
        };
        let control = host
            .begin(&mut window, waker, store)
            .map_err(|e| error(&e))?;
        Ok(Self {
            host,
            window,
            control,
            woken,
            counts,
            failure: None,
        })
    }

    // ----- input and lifecycle -----

    /// Deliver one shell event at the host's entry, as the shell delivers
    /// the winit events it maps. Input is applied to the host model at
    /// once; the client's turn for it runs at the next [`Self::wake`] (the
    /// host reports itself [`Self::due`]), as in a running App.
    pub fn send(&mut self, event: ShellEvent) {
        let event = self.window.input(event);
        self.deliver(event);
    }

    /// Deliver the mouse event of `phase` as the test window makes it
    /// (`VirtualWindow::mouse`): at `position` (`None`: where the mouse
    /// last was), with the buttons held so far and a `Down`'s or `Up`'s
    /// button. Panics when nothing placed the mouse yet.
    pub fn mouse(
        &mut self,
        phase: kivact_types::PointerPhase,
        position: Option<kivact_types::Point>,
    ) {
        let Some(event) = self.window.mouse(phase, position) else {
            panic!(
                "kivact::test: a mouse button or leave was injected before the mouse was in the window; move it there first (Screen::mouse_move, or an element's click/hover)"
            );
        };
        self.send(event);
    }

    /// Deliver an event the window already counted at the host's entry.
    fn deliver(&mut self, event: ShellEvent) {
        if self.failure.is_some() {
            return;
        }
        match self.host.handle(event, &mut self.window) {
            Ok(control) => self.control = control,
            Err(e) => self.failure = Some(e),
        }
    }

    /// A touch or pen contact as the platform reports it (`contact.id` is
    /// the platform's id of the contact), delivered as a native window
    /// delivers its contacts: the window gives it a pointer id of its own
    /// (never the mouse's), counts its taps and drops what it would not
    /// report (a move or an end of a contact that never began).
    pub fn contact(&mut self, contact: PointerSample) {
        for event in self.window.contact(contact) {
            self.deliver(event);
        }
    }

    /// A callback error the shell would have stopped on; events sent after
    /// it are ignored.
    pub fn failure(&self) -> Option<&ShellError> {
        self.failure.as_ref()
    }

    /// Whether the App exited (it accepted a close request).
    pub fn exited(&self) -> bool {
        self.control == ShellControl::Exit
    }

    // ----- clock and scheduling -----

    /// The virtual clock (ns since the host started).
    pub fn now_ns(&self) -> u64 {
        self.window.now_ns()
    }

    /// Move the virtual clock to `ns` (never backwards). Nothing runs until
    /// the next [`Self::wake`].
    pub fn set_now(&mut self, ns: u64) {
        self.window.set_now(ns);
    }

    /// The display refresh period frame ticks are paced at (ns).
    pub fn frame_period_ns(&self) -> u64 {
        self.host.model.borrow().frame.period_ns()
    }

    /// When the host next wants to run (virtual ns): queued events at once,
    /// requested frames and the client's timers.
    pub fn next_deadline(&self) -> Option<u64> {
        self.control.deadline()
    }

    /// Whether the host wants to run now.
    pub fn due(&self) -> bool {
        self.control
            .deadline()
            .is_some_and(|t| t <= self.window.now_ns())
    }

    /// Whether a worker thread (a blocking job, the asset store) asked for
    /// a wake since the last [`Self::wake`]. A wake unparks the thread that
    /// created the driver, so it can wait for one with
    /// `std::thread::park_timeout`.
    pub fn woken(&self) -> bool {
        self.woken.load(Ordering::SeqCst)
    }

    /// What the shell does on a wake or a reached deadline: frame ticks due
    /// at the virtual time, the asset store's results and the client's
    /// turn.
    pub fn wake(&mut self) {
        self.woken.store(false, Ordering::SeqCst);
        self.send(ShellEvent::Wake);
    }

    // ----- frames -----

    /// Whether the host asked the window for a redraw that has not been
    /// drawn yet ([`Self::redraw`]).
    pub fn redraw_requested(&self) -> bool {
        self.window.redraw_requested()
    }

    /// Render and present a frame if the host asked for one (what the
    /// OS's paint message does). Returns whether it did.
    pub fn redraw(&mut self) -> bool {
        if !self.window.take_redraw() {
            return false;
        }
        self.send(ShellEvent::Redraw);
        true
    }

    /// The rasterizer that drew the last presented frame.
    pub fn renderer(&self) -> Option<Renderer> {
        self.window.renderer()
    }

    /// The last presented frame.
    pub fn frame(&self) -> Option<TestFrame<'_>> {
        self.window
            .frame()
            .map(|(width, height, pixels)| TestFrame {
                width,
                height,
                pixels,
            })
    }

    // ----- environment -----

    /// Resize the content to `width` × `height` logical pixels.
    pub fn resize(&mut self, width: f64, height: f64) {
        self.window.resize(Size { width, height });
        self.send(ShellEvent::Resized);
    }

    /// Change the scale (physical pixels per logical pixel).
    pub fn set_scale(&mut self, scale: f64) {
        self.window.set_scale(scale);
        self.send(ShellEvent::Resized);
    }

    /// Change the system theme and report it (`EnvChanged` when it
    /// changed).
    pub fn set_theme(&mut self, theme: ThemeHint) {
        self.window.set_dark(theme == ThemeHint::Dark);
        self.send(ShellEvent::EnvironmentChanged);
    }

    /// Change the system locale (`None`: the system names none) and report
    /// it (`EnvChanged` when it changed).
    pub fn set_locale(&mut self, locale: Option<&str>) {
        self.window.set_locale(locale);
        self.send(ShellEvent::EnvironmentChanged);
    }

    /// Move the window to a display refreshing every `period_ns` ns, as
    /// the shell reports a move to another monitor (`EnvChanged` when the
    /// period changes).
    pub fn set_display_period(&mut self, period_ns: u64) {
        self.window.set_frame_period(period_ns);
        self.send(ShellEvent::EnvironmentChanged);
    }

    /// Change the reduced-motion setting and report it (`EnvChanged` when
    /// it changed).
    pub fn set_reduced_motion(&mut self, on: bool) {
        self.window.set_reduced_motion(on);
        self.send(ShellEvent::EnvironmentChanged);
    }

    /// Change the safe area, as a phone's system UI does (`EnvChanged`
    /// when it changed).
    pub fn set_safe_area(&mut self, insets: Insets) {
        self.window.set_safe_area(insets);
        self.send(ShellEvent::EnvironmentChanged);
    }

    /// Lose (`false`) or regain the window's surface, as when the platform
    /// destroys it and gives the window a new one (the next surface
    /// generation).
    pub fn set_surface(&mut self, active: bool) {
        if active {
            self.window.create_surface();
            self.send(ShellEvent::SurfaceCreated);
        } else {
            self.window.destroy_surface();
            self.send(ShellEvent::SurfaceDestroyed);
        }
    }

    /// The platform suspends the application (sends it to the background:
    /// Android, iOS), destroying the window's surface with it when
    /// `destroys_surface` (Android).
    pub fn suspend(&mut self, destroys_surface: bool) {
        self.send(ShellEvent::Suspended);
        if destroys_surface {
            self.window.destroy_surface();
            self.send(ShellEvent::SurfaceDestroyed);
        }
    }

    /// The suspended application comes back to the foreground, with a new
    /// surface if its surface was destroyed.
    pub fn resume(&mut self) {
        if self.window.surface_generation().is_none() {
            self.window.create_surface();
            self.send(ShellEvent::SurfaceCreated);
        }
        self.send(ShellEvent::Resumed);
    }

    /// The platform warns that memory is short (`MemoryWarning`).
    pub fn memory_warning(&mut self) {
        self.send(ShellEvent::MemoryWarning);
    }

    /// The generation of the window's surface (`None`: it has none).
    pub fn surface_generation(&self) -> Option<u64> {
        self.window.surface_generation()
    }

    /// The user asks to close the window (`CloseRequested`).
    pub fn request_close(&mut self) {
        self.send(ShellEvent::CloseRequested);
    }

    /// Queue any host event (failures, stale input ..) and run the turn.
    pub fn inject(&mut self, event: HostEvent) {
        if self.failure.is_some() {
            return;
        }
        match self.host.inject(event, &mut self.window) {
            Ok(control) => self.control = control,
            Err(e) => self.failure = Some(e),
        }
    }

    // ----- platform services -----

    /// The in-memory clipboard.
    pub fn clipboard(&self) -> &str {
        self.window.clipboard()
    }

    /// Put `text` on the in-memory clipboard.
    pub fn set_clipboard(&mut self, text: &str) {
        self.window.set_clipboard(text);
    }

    /// Script the answer of the next file, save or folder dialog (`None`:
    /// cancelled). An unscripted dialog is cancelled.
    pub fn push_file_dialog(&mut self, path: Option<PathBuf>) {
        self.window.push_file_dialog(path);
    }

    /// Script the next multiple-file picker response; `None` cancels.
    pub fn push_files_dialog(&mut self, paths: Option<Vec<PathBuf>>) {
        self.window.push_files_dialog(paths);
    }

    /// The paths the App asked to open with the default handler, in order
    /// (recorded; nothing is started).
    pub fn opened_paths(&self) -> &[PathBuf] {
        self.window.opened_paths()
    }

    /// The paths the App asked to reveal in the file manager, in order
    /// (recorded; nothing is started).
    pub fn revealed_paths(&self) -> &[PathBuf] {
        self.window.revealed_paths()
    }

    /// Make the next platform request (clipboard, dialog, open or reveal)
    /// fail with `reason`. Font registrations are not platform requests
    /// here.
    pub fn fail_next_effect(&mut self, reason: &str) {
        self.window.fail_next_effect(reason);
    }

    /// Make the next platform request answer as on a platform without
    /// the service (`PlatformResult::Unsupported`).
    pub fn fail_next_effect_unsupported(&mut self, reason: &str) {
        self.window.fail_next_effect_unsupported(reason);
    }

    // ----- what the host shows -----

    /// The accessibility tree last handed to the platform (what assistive
    /// technology sees). A text input's editing snapshot in it (its
    /// properties' `text`: the runs, selection and caret actually
    /// published) is what a test reads its published selection and caret
    /// from; a password has none.
    pub fn semantics(&self) -> Option<SemanticsTree> {
        self.window.accessibility().tree()
    }

    /// The node, or the node and scene region, an accessibility id stands
    /// for (node ids are Kivact node ids; regions count down from
    /// `u64::MAX`).
    pub fn semantics_target(&self, id: u64) -> Option<(NodeId, Option<RegionId>)> {
        let model = self.host.model.borrow();
        match model.a11y.region_of(id) {
            Some((node, region)) => Some((node, Some(region))),
            None => model.node(NodeId(id)).map(|_| (NodeId(id), None)),
        }
    }

    /// The accessibility id of a node or of one of its scene regions.
    pub fn semantics_id(&self, node: NodeId, region: Option<RegionId>) -> Option<u64> {
        let model = self.host.model.borrow();
        match region {
            Some(region) => model.a11y.region_id(node, region),
            None => model.node(node).map(|_| node.0),
        }
    }

    /// Where the box of `id` is drawn, in logical window coordinates: its
    /// layout box mapped by its transforms (their bounds).
    pub fn rect(&self, id: NodeId) -> Option<Rect> {
        self.host.model.borrow().bounds(id)
    }

    /// The layout size of `id` (its box before its transforms).
    pub fn size(&self, id: NodeId) -> Option<Size> {
        let model = self.host.model.borrow();
        model.computed_layout(id).map(|l| l.size)
    }

    /// The rect of scene region `region` of `id`, relative to the node's
    /// untransformed layout box.
    pub fn region_local(&self, id: NodeId, region: RegionId) -> Option<Rect> {
        let model = self.host.model.borrow();
        Some(model.node(id)?.region(region)?.rect)
    }

    /// Where scene region `region` of `id` is drawn, in logical window
    /// coordinates (the bounds of its rect mapped by the node's
    /// transforms).
    pub fn region_rect(&self, id: NodeId, region: RegionId) -> Option<Rect> {
        let r = self.region_local(id, region)?;
        let model = self.host.model.borrow();
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
        self.host.model.borrow().local_to_window(id, local)
    }

    /// The node hit at a logical window point.
    pub fn hit(&self, x: f64, y: f64) -> Option<NodeId> {
        self.host.model.borrow().hit(Point { x, y })
    }

    /// The node with the keyboard focus.
    pub fn focused(&self) -> Option<kivact_types::FocusTarget> {
        self.host.model.borrow().focused()
    }

    /// The node under the pointer.
    pub fn hovered(&self) -> Option<NodeId> {
        self.host.model.borrow().hovered()
    }

    /// The open modal that confines input, if any.
    pub fn modal(&self) -> Option<NodeId> {
        self.host.model.borrow().modal()
    }

    /// The host's tree in preorder (from the root), without the subtrees
    /// leaving after kivact removed them (they are kivact's no more).
    pub fn preorder(&self) -> Vec<NodeId> {
        let model = self.host.model.borrow();
        let tree = model.tree();
        tree.preorder()
            .into_iter()
            .filter(|id| tree.live(*id))
            .collect()
    }

    /// Nodes the host holds for kivact (reachable or not; not the leaving
    /// ones).
    pub fn node_count(&self) -> usize {
        let model = self.host.model.borrow();
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
        let model = self.host.model.borrow();
        let mut engine = self.host.engine.borrow_mut();
        crate::check::layout_findings(&model, &mut engine, check, scope, true)
    }

    /// The kind of node `id`.
    pub fn kind(&self, id: NodeId) -> Option<NodeType> {
        self.host.model.borrow().node(id).map(|n| n.kind())
    }

    /// The children of node `id`, in order (not the leaving ones).
    pub fn children(&self, id: NodeId) -> Option<Vec<NodeId>> {
        let model = self.host.model.borrow();
        model.node(id).map(|n| {
            model
                .tree
                .children_ids(n)
                .filter(|c| model.tree.live(*c))
                .collect()
        })
    }

    /// The props node `id` holds (as the client sent them; a text input's
    /// value is [`Self::input_value`]).
    pub fn properties(&self, id: NodeId) -> Option<NodeProperties> {
        let model = self.host.model.borrow();
        model.node(id).map(|n| n.properties.clone())
    }

    /// The value of text input `id` as the host edits it.
    pub fn input_value(&self, id: NodeId) -> Option<String> {
        let model = self.host.model.borrow();
        model.node(id).map(|_| model.input_value(id).to_owned())
    }

    /// The draw and draw-over scenes of node `id`.
    pub fn scenes(&self, id: NodeId) -> Option<[Option<Arc<DrawList>>; 2]> {
        let model = self.host.model.borrow();
        model.node(id).map(|n| n.scenes().clone())
    }

    /// Whether node `id` is shown (it and its ancestors are not hidden).
    pub fn shown(&self, id: NodeId) -> bool {
        self.host.model.borrow().tree().shown(id)
    }

    /// Whether the focused text input holds an IME composition.
    pub fn composing(&self) -> bool {
        let model = self.host.model.borrow();
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
        self.host.engine.borrow_mut().has_font_family(family)
    }

    /// The window title the App set.
    pub fn title(&self) -> String {
        self.window.title()
    }

    /// Requested logical content sizes in order.
    pub fn requested_sizes(&self) -> &[Size] {
        self.window.requested_sizes()
    }

    /// The minimum window size the App set (`None`: no minimum).
    pub fn min_window_size(&self) -> Option<Size> {
        self.window.min_size()
    }

    /// The IME caret area (logical) while a text input has the focus.
    pub fn ime_area(&self) -> Option<Rect> {
        self.window.ime_area()
    }

    /// The cursor the window shows over its content.
    pub fn cursor(&self) -> Cursor {
        self.window.cursor()
    }

    /// Whether the host asked the window to show the software keyboard,
    /// and how often it asked to show or hide it.
    pub fn soft_keyboard(&self) -> (bool, u32) {
        self.window.soft_keyboard()
    }

    /// What the host gave the window for the IME's queries about the
    /// focused text input (what a native window answers a reconversion,
    /// document feed or character position request from).
    pub fn ime_text(&self) -> Option<ShellImeText> {
        self.window.ime_text()
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
        self.window.has_window()
    }

    /// Wait until the real window shows everything presented so far.
    pub fn sync_window(&self) {
        self.window.sync();
    }
}
