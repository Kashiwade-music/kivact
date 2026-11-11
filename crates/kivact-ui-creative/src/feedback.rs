//! Role: Banners, notices and toasts that give feedback in the Creative kit.
use kivact::interaction::{DialogMode, MenuEntry, Tone};
use kivact::state::IntoBinding;
use kivact::ui::ButtonVariant;

use kivact::draw::Shadow;
use kivact::geometry::{Point, Rect};
use kivact::i18n::{message, message_with};
use kivact::interaction::{DragCoordinates, DragUpdate, use_drag};
use kivact::platform;
use kivact::view::{Element, Node, column, icon, node, row, text};

use kivact::state::{Binding, Value, signal};
use kivact::state::{IntoText, IntoValue};
use kivact::style::use_theme;
use kivact::style::{Align, Color, Icon, Position, style};
use kivact::view::{
    A11yAction, ButtonPress, Handled, HandlerResult, IntoView, MouseButton, NodeRef, Placement,
    PointerPhase, Role, View, node_ref, overlay, show,
};

/// The colors a Creative theme gives a semantic [`Tone`].
pub(crate) trait ToneExt {
    /// The tone's color.
    fn color(self, p: &crate::Tokens) -> Color;
}
impl ToneExt for Tone {
    fn color(self, p: &crate::Tokens) -> Color {
        match self {
            Self::Neutral => p.muted,
            Self::Info => p.z,
            Self::Success => p.success,
            Self::Warning => p.warning,
            Self::Danger => p.x,
            Self::Accent => p.accent_high,
        }
    }
}

/// The Creative inline notice appearance of a standard `kivact::ui::alert`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoticeVariant;
impl kivact::ui::Variant for NoticeVariant {}

pub(crate) fn glass_style(t: &kivact::style::Theme) -> kivact::style::Style {
    style()
        .padding(6)
        .gap(1)
        .min_width(220)
        .radius(12)
        .color(t.ext::<crate::tokens::Tokens>().text)
        .background(crate::color::overlay_surface())
        .backdrop_blur(22)
        .border(1, crate::color::overlay_border())
        .shadow(Shadow::new(0.0, 18.0, 45.0, crate::color::window_shadow()))
}

/// A button that opens a nonmodal popover below it, and the popover.
///
/// This is the core [`kivact::interaction::DialogApi`] as a `DialogMode::Popover`
/// in Creative's look:
/// a `Role::Dialog` named like the button, closed by Escape, an outside press,
/// assistive technology's Dismiss and a Close button under the content (named
/// by the message `dialog.close`). The button toggles it, and after Escape,
/// Dismiss or Close the focus goes back to the button.
#[track_caller]
pub fn popup_button<V: IntoView>(
    label: impl IntoText,
    content: impl kivact::view::IntoViewFn<V>,
) -> View {
    let label = label.into_text_value();
    let open = signal(false);
    let close = message("dialog.close", "Close");
    kivact::ui::dialog(label, open)
        .closable(false)
        .mode(DialogMode::Popover)
        .content(content)
        .footer(move || {
            kivact::ui::button(close)
                .variant(kivact::ui::ButtonVariant::Secondary)
                .on_press(move || open.set(false))
        })
        .attach(kivact::ui::button(label).variant(kivact::ui::ButtonVariant::Secondary))
}

/// A primary command joined to a separately accessible options menu.
#[track_caller]
pub fn split_button<K: kivact::state::ItemKey>(
    label: impl IntoText,
    items: impl IntoValue<Vec<MenuEntry<K>>>,
) -> SplitButton<K> {
    SplitButton {
        root: row(()).role(Role::Group).style(
            style()
                .align_self(Align::Start)
                .gap(1)
                .radius(7)
                .overflow(kivact::style::Overflow::Hidden),
        ),
        primary: kivact::ui::button(label)
            .variant(ButtonVariant::Primary)
            .with_node(move |node| node.style(style().radius(0))),
        entries: items.into_value(),
        choose: kivact::state::callback(|_| ()),
    }
}
/// Primary action with a menu and an optional selection callback.
pub struct SplitButton<K: 'static> {
    root: Node,
    primary: kivact::ui::Button,
    entries: Value<Vec<MenuEntry<K>>>,
    choose: kivact::state::Callback<K>,
}
impl<K: 'static> SplitButton<K> {
    /// Run the primary command.
    pub fn on_press(mut self, primary: impl Into<kivact::state::Callback<()>>) -> Self {
        self.primary = self.primary.on_press(primary);
        self
    }

    /// Activation callback; no action by default.
    pub fn on_activate<R: HandlerResult>(mut self, choose: impl Fn(K) -> R + 'static) -> Self {
        self.choose = kivact::state::callback(choose);
        self
    }
}
impl<K: kivact::state::ItemKey> Element for SplitButton<K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl<K: kivact::state::ItemKey> IntoView for SplitButton<K> {
    fn into_view(mut self) -> View {
        let disabled = self
            .root
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        let choose = self.choose;
        self.root
            .child((
                self.primary.with_node(move |node| node.disabled(disabled)),
                kivact::ui::menu("", self.entries)
                    .on_activate(move |key| {
                        if disabled.get() {
                            Ok(())
                        } else {
                            choose.try_call(key)
                        }
                    })
                    .offset(0.)
                    .attach(
                        kivact::ui::button(message(
                            "creative.split_button.options",
                            "Command options",
                        ))
                        .with_node(move |node| node.disabled(disabled))
                        .variant(ButtonVariant::Primary)
                        .hide_label(true)
                        .icon(icon("chevron-down"))
                        .with_node(move |node| {
                            node.style(style().width(26).radius(0).padding_x(5))
                        }),
                    ),
            ))
            .into_view()
    }
}

/// Initial placement of a floating tool window in window coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum WindowPosition {
    #[default]
    Center,
    /// Insets from the right and top edges of the application window.
    TopRight { right: f64, top: f64 },
}

impl WindowPosition {
    fn origin(self, viewport: kivact::geometry::Size, bounds: Rect) -> Point {
        match self {
            Self::Center => Point::new(
                (viewport.width - bounds.width) * 0.5,
                (viewport.height - bounds.height) * 0.5,
            ),
            Self::TopRight { right, top } => Point::new(viewport.width - bounds.width - right, top),
        }
    }
}

/// Chrome, placement and input containment for a reusable tool window.
///
/// The default is a 520-pixel nonmodal centered window,
/// with a draggable and collapsible title bar. Modal windows retain their
/// backdrop when it is pressed, and close on Escape or their own commands.
#[derive(Clone, Debug)]
pub struct ToolWindowOptions {
    title: Option<String>,
    width: f64,
    position: WindowPosition,
    modal: bool,
    backdrop_blur: f64,
    frame_padding: f64,
    body_padding: f64,
    gap: f64,
    radius: f64,
    titlebar_height: f64,
    titlebar_padding: (f64, f64),
    icon: Option<Icon>,
}
impl Default for ToolWindowOptions {
    fn default() -> Self {
        Self {
            title: None,
            width: 520.0,
            position: WindowPosition::Center,
            modal: false,
            backdrop_blur: 0.0,
            frame_padding: 6.0,
            body_padding: 18.0,
            gap: 1.0,
            radius: 12.0,
            titlebar_height: 36.0,
            titlebar_padding: (12.0, 12.0),
            icon: None,
        }
    }
}
impl ToolWindowOptions {
    /// Give the window a title independent of its opener label.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    pub fn width(mut self, width: f64) -> Self {
        if width.is_finite() {
            self.width = width.max(80.0);
        }
        self
    }
    pub fn position(mut self, position: WindowPosition) -> Self {
        if let WindowPosition::TopRight { right, top } = position
            && (!right.is_finite() || !top.is_finite())
        {
            return self;
        }
        self.position = position;
        self
    }
    /// Contain input and focus inside the window. A constant: it selects the
    /// dialog kind the window is built with, and an options value is plain
    /// data.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }
    /// Blur the contained application's backdrop while a modal window is open.
    pub fn backdrop_blur(mut self, blur: f64) -> Self {
        self.backdrop_blur = blur.max(0.0);
        self
    }
    pub fn frame_padding(mut self, padding: f64) -> Self {
        self.frame_padding = padding.max(0.0);
        self
    }
    pub fn body_padding(mut self, padding: f64) -> Self {
        self.body_padding = padding.max(0.0);
        self
    }
    pub fn gap(mut self, gap: f64) -> Self {
        self.gap = gap.max(0.0);
        self
    }
    pub fn radius(mut self, radius: f64) -> Self {
        self.radius = radius.max(0.0);
        self
    }
    pub fn titlebar_height(mut self, height: f64) -> Self {
        self.titlebar_height = height.max(24.0);
        self
    }
    /// Set the left and right insets of the title bar.
    pub fn titlebar_padding(mut self, left: f64, right: f64) -> Self {
        self.titlebar_padding = (left.max(0.0), right.max(0.0));
        self
    }
    /// Show an icon before the title: a name in the theme's icon set or an
    /// image.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// What a tool window gives its content: [`hide`](ToolWindowApi::hide) closes
/// the window, as its close button does.
#[derive(Clone, Copy)]
pub struct ToolWindowApi {
    open: Binding<bool>,
}

impl ToolWindowApi {
    /// Hide the window: its `open` value becomes `false`.
    pub fn hide(self) {
        self.open.set(false);
    }
}

/// A button labelled `label` that opens a tool window with `options`, whose
/// `content` gets the [`ToolWindowApi`] to close it.
///
/// This is [`tool_window`] with the button as its anchor and an open state the
/// button keeps itself. The window's title is [`ToolWindowOptions::title`], or
/// else the label. `ToolWindowOptions::default()` is a 520-pixel nonmodal
/// centered window with a draggable and collapsible title bar.
#[track_caller]
pub fn tool_window_button<V: IntoView>(
    label: impl IntoText,
    options: ToolWindowOptions,
    content: impl Fn(ToolWindowApi) -> V + 'static,
) -> impl kivact::view::Element {
    let label = label.into_text_value();
    let title = options.title.clone().map_or(label, IntoValue::into_value);
    let open = signal(false);
    let anchor = node_ref();
    let opener = kivact::ui::button(label)
        .variant(kivact::ui::ButtonVariant::Secondary)
        .on_press(move || open.set(true))
        .with_node(move |node| node.node_ref(anchor))
        .with_node(move |node| node.expanded(open));
    column((
        opener,
        window_layer(anchor, open.into_binding(), options, title, content),
    ))
    .style(style().gap(0))
}

/// A tool window that the application opens and closes through `open`.
///
/// The window has a title bar to drag and, unless it is modal, to collapse,
/// a close button, and `content`, which gets the [`ToolWindowApi`] to close the
/// window. `anchor` is the node the window belongs to, usually the control
/// that opens it. `open` is the only state: the window mounts when it
/// becomes `true`, and it is written `false` by the close button, Escape,
/// Dismiss from assistive technology, an outside press on a nonmodal window,
/// [`ToolWindowApi::hide`] and the unmounting of the window's part itself. A
/// modal window keeps input and focus inside it and ignores an outside press.
/// The title is [`ToolWindowOptions::title`], or the message `window.untitled`
/// ("Window").
///
/// Each time the window opens it starts expanded at its initial position,
/// whatever the earlier use left.
///
/// After Escape, the close button, [`ToolWindowApi::hide`] or Dismiss, a
/// nonmodal window uses the shared dialog to restore its anchor when focus
/// was inside it. An unavailable anchor is harmless. A modal window gives it
/// back to the element that had it when the window opened. An outside press
/// never moves the focus.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_creative as ui;
///
/// fn tool() -> impl IntoView {
///     let open = signal(false);
///     let anchor = node_ref();
///     column((
///         kivact::ui::button("Tool").with_node(move |node| node.node_ref(anchor)).on_press(move || open.set(true)),
///         ui::tool_window(
///             anchor,
///             open,
///             ui::ToolWindowOptions::default().title("Tool"),
///         ).content(|window: ui::ToolWindowApi| {
///                 kivact::ui::button("Done").on_press(move || window.hide())
///             }),
///     ))
/// }
/// ```
#[track_caller]
pub fn tool_window(
    anchor: NodeRef,
    open: impl IntoBinding<bool>,
    options: ToolWindowOptions,
) -> ToolWindow {
    ToolWindow {
        root: column(()),
        anchor,
        open: open.into_binding(),
        options,
        content: kivact::view::ViewFn::from(|_| ()),
    }
}
/// Tool-window description; content is rebuilt on every opening.
pub struct ToolWindow {
    root: Node,
    anchor: NodeRef,
    open: Binding<bool>,
    options: ToolWindowOptions,
    content: kivact::view::ViewFn<ToolWindowApi>,
}
impl ToolWindow {
    /// Content built on opening; empty by default.
    pub fn content<V: IntoView>(mut self, content: impl Fn(ToolWindowApi) -> V + 'static) -> Self {
        self.content = kivact::view::ViewFn::from(content);
        self
    }
}
impl Element for ToolWindow {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for ToolWindow {
    fn into_view(self) -> View {
        let title = self.options.title.clone().map_or_else(
            || message("creative.window.untitled", "Window"),
            IntoValue::into_value,
        );
        let content = self.content;
        self.root
            .child(window_layer(
                self.anchor,
                self.open,
                self.options,
                title,
                move |window| content.call(window),
            ))
            .into_view()
    }
}

#[track_caller]
fn window_layer<V: IntoView>(
    anchor: NodeRef,
    open: Binding<bool>,
    options: ToolWindowOptions,
    title: Value<String>,
    content: impl Fn(ToolWindowApi) -> V + 'static,
) -> Node {
    close_removed_window(open);
    let reference = node_ref();
    let bounds = reference.rect();
    let kind = if options.modal {
        DialogMode::Modal
    } else {
        DialogMode::Popover
    };
    let mut dialog = kivact::interaction::use_dialog(open, kind).anchor(anchor);
    if options.modal {
        dialog = dialog.dismiss(kivact::interaction::DismissOn {
            escape: true,
            outside: false,
        });
    }
    let close = ToolWindowApi { open };
    // The state of the window that is open now; a closed window keeps none.
    let state = signal(ToolWindowState::Closed);
    let layer = show(dialog.is_open(), move || {
        // Made again at each opening, so a window starts expanded and in place.
        kivact::state::on_mount(move || {
            state.set(ToolWindowState::Open {
                collapsed: false,
                offset: Point::new(0.0, 0.0),
            });
        });
        kivact::state::on_cleanup(move || state.set(ToolWindowState::Closed));
        // The offset and box at the press, which the drag moves from.
        let mover = window_mover(state, options.position);
        let header = window_header(options.clone(), title, state, mover, bounds, close);
        let window = overlay(
            node_ref(),
            Placement::Center,
            column((
                header,
                node()
                    .hidden(move || state.with(ToolWindowState::collapsed))
                    .style(style().padding(options.body_padding))
                    .child(content(close)),
            ))
            .role(Role::Group)
            .name(message_with(
                "creative.window.name",
                "{title} window",
                move || [("title", title.get())],
            ))
            .node_ref(reference)
            .style(style().width(options.width))
            .style(move || {
                let t = &use_theme().get();
                glass_style(t)
                    .padding(options.frame_padding)
                    .gap(options.gap)
                    .radius(options.radius)
                    .overflow(kivact::style::Overflow::Hidden)
            }),
        )
        .with_node(move |node| node.role(Role::Dialog))
        .with_node(move |node| node.name(title))
        .with_node(move |node| {
            node.style(move || {
                let viewport = platform::window_size().get();
                let rect = bounds
                    .get()
                    .unwrap_or(Rect::new(0.0, 0.0, options.width, 0.0));
                let centered = WindowPosition::Center.origin(viewport, rect);
                let initial = options.position.origin(viewport, rect);
                let offset = state.with(ToolWindowState::offset);
                style().translate(
                    initial.x - centered.x + offset.x,
                    initial.y - centered.y + offset.y,
                )
            })
        })
        .with_node(move |node| node.part(dialog.content()));
        column((window_backdrop(&options), window)).style(style().gap(0))
    });
    column((layer,)).style(style().gap(0))
}

/// One tool window: closed, or open with its collapsed state and drag offset.
#[derive(Clone, Copy, PartialEq)]
enum ToolWindowState {
    Closed,
    Open { collapsed: bool, offset: Point },
}

impl ToolWindowState {
    fn collapsed(&self) -> bool {
        matches!(
            self,
            Self::Open {
                collapsed: true,
                ..
            }
        )
    }

    fn offset(&self) -> Point {
        match self {
            Self::Closed => Point::new(0.0, 0.0),
            Self::Open { offset, .. } => *offset,
        }
    }
}

fn window_mover(
    state: kivact::state::Signal<ToolWindowState>,
    position: WindowPosition,
) -> kivact::interaction::DragApi<Option<(Point, Rect)>> {
    use_drag::<Option<(Point, Rect)>>(DragCoordinates::Window).on_update(
        move |update: DragUpdate<Option<(Point, Rect)>>| {
            let Some((initial, rect)) = update.payload else {
                return;
            };
            let viewport = platform::window_size().get();
            let base = position.origin(viewport, rect);
            let proposed = Point::new(
                base.x + initial.x + update.delta.x,
                base.y + initial.y + update.delta.y,
            );
            let offset = Point::new(
                proposed.x.clamp(
                    -rect.width + 80.0,
                    (viewport.width - 80.0).max(-rect.width + 80.0),
                ) - base.x,
                proposed.y.clamp(0.0, (viewport.height - 30.0).max(0.0)) - base.y,
            );
            state.update(|state| {
                if let ToolWindowState::Open {
                    offset: current, ..
                } = state
                {
                    *current = offset;
                }
            });
        },
    )
}

fn toggle_collapsed(state: kivact::state::Signal<ToolWindowState>) {
    state.update(|state| {
        if let ToolWindowState::Open { collapsed, .. } = state {
            *collapsed = !*collapsed;
        }
    });
}

fn window_header(
    options: ToolWindowOptions,
    title: Value<String>,
    state: kivact::state::Signal<ToolWindowState>,
    mover: kivact::interaction::DragApi<Option<(Point, Rect)>>,
    bounds: Value<Option<Rect>>,
    close: ToolWindowApi,
) -> Node {
    let (minimize_label, close_label) = (
        message("creative.window.minimize", "Minimize window"),
        message("creative.window.close", "Close window"),
    );
    let window_icon = options.icon.clone();
    let header_options = options.clone();
    let mut header = row((
        options.icon.is_some().then(move || {
            icon(window_icon.clone().unwrap_or(Icon::named("settings")))
                .style(style().color(crate::ColorName::Accent))
        }),
        text(title).style(style().weight(600).grow(1)),
        (!options.modal).then(move || {
            kivact::ui::button(minimize_label)
                .variant(ButtonVariant::Secondary)
                .hide_label(true)
                .icon(kivact::view::icon("chevron-down"))
                .with_node(move |node| {
                    node.on_pointer_default(|event| {
                        if matches!(event.phase(), PointerPhase::Down(_)) {
                            Handled::Yes
                        } else {
                            Handled::No
                        }
                    })
                })
                .on_press(move || toggle_collapsed(state))
        }),
        kivact::ui::button(close_label)
            .variant(ButtonVariant::Secondary)
            .hide_label(true)
            .icon(kivact::view::icon("x"))
            .with_node(move |node| {
                node.on_pointer_default(|event| {
                    if matches!(event.phase(), PointerPhase::Down(_)) {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                })
            })
            .on_press(move || close.hide()),
    ))
    .name(message_with(
        "creative.window.title_bar",
        "{title} title bar",
        move || [("title", title.get())],
    ))
    .style(move || window_header_style(&header_options));
    if !options.modal {
        header = header
            .role(Role::Button)
            .focusable(true)
            .expanded(move || !state.with(ToolWindowState::collapsed))
            .a11y_action(A11yAction::Toggle, move |_| toggle_collapsed(state))
            .on_pointer_default(move |e| {
                if matches!(
                    e.phase(),
                    PointerPhase::Down(ButtonPress {
                        button: MouseButton::Primary,
                        clicks: 2..
                    })
                ) {
                    mover.cancel();
                    toggle_collapsed(state);
                    return Handled::Yes;
                }
                Handled::No
            })
            .part(mover.source_with(move || {
                bounds
                    .get_untracked()
                    .map(|rect| (state.with_untracked(ToolWindowState::offset), rect))
            }));
    }
    header
}

fn window_header_style(options: &ToolWindowOptions) -> kivact::style::Style {
    let t = &use_theme().get();
    style()
        .height(options.titlebar_height)
        .padding_left(options.titlebar_padding.0)
        .padding_right(options.titlebar_padding.1)
        .gap(8)
        .align_items(Align::Center)
        .background(t.ext::<crate::tokens::Tokens>().header)
        .cursor(if options.modal {
            kivact::style::Cursor::Default
        } else {
            kivact::style::Cursor::Grab
        })
}

fn window_backdrop(options: &ToolWindowOptions) -> Option<Node> {
    let modal = options.modal;
    let blur = options.backdrop_blur;
    (modal && blur > 0.0).then(move || {
        node().style(
            style()
                .position(Position::Fixed)
                .top(0)
                .right(0)
                .bottom(0)
                .left(0)
                .z_index(100)
                .background(crate::color::transparent_backdrop())
                .backdrop_blur(blur),
        )
    })
}

fn close_removed_window(open: Binding<bool>) {
    let value = open.into_value();
    kivact::state::on_cleanup(move || {
        if value.try_get() == Some(true) {
            open.set(false);
        }
    });
}
