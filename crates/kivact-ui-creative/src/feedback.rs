//! Role: Popup buttons, split buttons and the data of tool windows in the Creative kit.
use std::rc::Rc;

use kivact::component;
use kivact::custom::Caller;
use kivact::geometry::{Point, Rect};
use kivact::i18n::{message, message_with};
use kivact::interaction::{
    DialogMode, DismissOn, DragCoordinates, DragUpdate, FocusReturn, FocusReturnTarget, MenuEntry,
    ReasonSet, use_dialog, use_drag, use_popup,
};
use kivact::platform;
use kivact::state::{Callback, IntoValue, Signal, Value, on_cleanup, on_mount, signal};
use kivact::style::{Align, Icon, Overflow, Position, Style, style};
use kivact::ui::ButtonVariant;
use kivact::view::{
    A11yAction, ButtonPress, Element, Handled, IntoView, MouseButton, Node, NodeRef, Placement,
    PointerPhase, Role, View, ViewFn, column, icon, node, node_ref, overlay, row, show, text,
};

use crate::recipe_parts::glass_style;

/// A button that opens a nonmodal popover below it, and the popover.
///
/// This is the core [`kivact::interaction::DialogApi`] as a `DialogMode::Popover`
/// in Creative's look:
/// a `Role::Dialog` named like the button, closed by Escape, an outside press,
/// assistive technology's Dismiss and a Close button under the content (named
/// by the message `dialog.close`). The button toggles it, and after Escape,
/// Dismiss or Close the focus goes back to the button.
#[component]
pub fn popup_button(label: Value<String>, content: ViewFn) -> impl IntoView {
    let open = signal(false);
    let close = message("dialog.close", "Close");
    kivact::ui::dialog(label, open)
        .closable(false)
        .mode(kivact::ui::DialogKind::Popover)
        .content(content)
        .footer(move || {
            kivact::ui::button(close)
                .variant(kivact::ui::ButtonVariant::Secondary)
                .on_press(move || open.set(false))
        })
        .attach(kivact::ui::button(label).variant(kivact::ui::ButtonVariant::Secondary))
}

/// A primary command joined to a separately accessible options menu.
///
/// `on_press` runs the primary command on the primary button part, not on the
/// menu button beside it. `on_activate` receives the key of
/// the chosen menu entry; it is a no-op by default. The whole control is
/// disabled when the application disables it.
#[component]
pub fn split_button<K: kivact::state::ItemKey>(
    label: Value<String>,
    items: Value<Vec<MenuEntry<K>>>,
    on_press: Option<Callback<()>>,
    on_activate: Option<Callback<K>>,
    #[caller] caller: Caller,
) -> impl Element {
    let disabled = caller.disabled().unwrap_or_else(|| false.into_value());
    let primary = kivact::ui::button(label)
        .variant(ButtonVariant::Primary)
        .with_node(move |node| node.style(style().border_radius(0)));
    let primary = match on_press {
        Some(press) => primary.on_press(press),
        None => primary,
    };
    let choose = on_activate.unwrap_or_else(|| kivact::state::callback(|_: K| ()));
    row((
        primary.with_node(move |node| node.disabled(disabled)),
        kivact::ui::menu("", items)
            .on_activate(move |key| {
                if disabled.get() {
                    Ok(())
                } else {
                    choose.try_call(key)
                }
            })
            .offset(0.)
            .attach(
                kivact::ui::button(message("creative.split_button.options", "Command options"))
                    .with_node(move |node| node.disabled(disabled))
                    .variant(ButtonVariant::Primary)
                    .hide_label(true)
                    .icon(icon("chevron-down"))
                    .with_node(move |node| {
                        node.style(style().width(26).border_radius(0).padding_x(5))
                    }),
            ),
    ))
    .role(Role::Group)
    .style(
        style()
            .align_self(Align::Start)
            .gap(1)
            .border_radius(7)
            .overflow(Overflow::Hidden),
    )
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
    open: Signal<bool>,
}

impl ToolWindowApi {
    /// Hide the window: its `open` value becomes `false`.
    pub fn hide(self) {
        self.open.set(false);
    }
}

/// A button labelled `label` that opens a tool window with `options`, whose
/// `content` gets the [`ToolWindowApi`] to close it. The window's title is
/// [`ToolWindowOptions::title`], or else the label.
///
/// `ToolWindowOptions::default()` is a 520-pixel nonmodal centered window
/// with a draggable and collapsible title bar.
#[component]
pub fn tool_window_button(
    label: Value<String>,
    options: ToolWindowOptions,
    content: Rc<dyn Fn(ToolWindowApi) -> View>,
) -> impl Element {
    let title = options.title.clone().map_or(label, IntoValue::into_value);
    let open = signal(false);
    let anchor = node_ref();
    let opener = kivact::ui::button(label)
        .variant(ButtonVariant::Secondary)
        .on_press(move || open.set(true))
        .with_node(move |node| node.node_ref(anchor))
        .with_node(move |node| node.expanded(open));
    column((opener, window_layer(anchor, open, options, title, content))).style(style().gap(0))
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
#[component]
pub fn tool_window(
    anchor: NodeRef,
    open: Signal<bool>,
    options: ToolWindowOptions,
    content: Rc<dyn Fn(ToolWindowApi) -> View>,
) -> impl Element {
    let title = options.title.clone().map_or_else(
        || message("creative.window.untitled", "Window"),
        IntoValue::into_value,
    );
    column((window_layer(anchor, open, options, title, content),))
}

/// The window of `open`: a popup surface at the centre of the window for a
/// nonmodal window, or a modal dialog with its backdrop.
fn window_layer(
    anchor: NodeRef,
    open: Signal<bool>,
    options: ToolWindowOptions,
    title: Value<String>,
    content: Rc<dyn Fn(ToolWindowApi) -> View>,
) -> View {
    close_removed_window(open);
    let reference = node_ref();
    let bounds = reference.rect();
    let viewport = node_ref();
    let close = ToolWindowApi { open };
    // The state of the window that is open now; a closed window keeps none.
    let state = signal(ToolWindowState::Closed);
    let blur = options.backdrop_blur;
    let modal = options.modal;
    let window = move || {
        // Made again at each opening, so a window starts expanded and in place.
        on_mount(move || {
            state.set(ToolWindowState::Open {
                collapsed: false,
                offset: Point::new(0.0, 0.0),
            });
        });
        on_cleanup(move || state.set(ToolWindowState::Closed));
        window_frame(
            options.clone(),
            title,
            state,
            bounds,
            reference,
            close,
            content.clone(),
        )
    };
    let layer = if modal {
        let dialog = use_dialog(open, DialogMode::Modal).dismiss(DismissOn {
            escape: true,
            outside: false,
        });
        show(dialog.is_open(), move || {
            let frame = overlay(node_ref(), Placement::Center, window())
                .with_node(move |node| node.part(dialog.content()));
            column((window_backdrop(blur), frame)).style(style().gap(0))
        })
        .into_view()
    } else {
        // The window is placed on the viewport, so it opens at its centre. The
        // window closes by writing `open` (its close button, `hide`), which
        // the popup accepts as an unmount, so the focus returns to the opener
        // on Unmount as well as on Escape and the other dismissals.
        use_popup(open)
            .anchor(viewport)
            .focus_return(FocusReturn {
                target: FocusReturnTarget::Anchor(anchor),
                on: ReasonSet::DISMISS
                    .union(ReasonSet::PROGRAMMATIC)
                    .union(ReasonSet::UNMOUNT),
            })
            .surface(Placement::Center, window)
    };
    column((window_viewport(viewport), layer))
        .style(style().gap(0))
        .into_view()
}

/// The box of the whole viewport, fixed at its corner: the anchor of a
/// nonmodal window, so that `Placement::Center` centres the window in the
/// viewport. It takes no pointer, which reaches the controls under it.
fn window_viewport(anchor: NodeRef) -> Node {
    node().node_ref(anchor).hit_testable(false).style(move || {
        let size = platform::window_size().get();
        style()
            .position(Position::Fixed)
            .top(0)
            .left(0)
            .width(size.width)
            .height(size.height)
    })
}

/// The frame of one opened window: its title bar and body, placed by its
/// initial position and drag offset.
fn window_frame(
    options: ToolWindowOptions,
    title: Value<String>,
    state: Signal<ToolWindowState>,
    bounds: Value<Option<Rect>>,
    reference: NodeRef,
    close: ToolWindowApi,
    content: Rc<dyn Fn(ToolWindowApi) -> View>,
) -> Node {
    let mover = window_mover(state);
    let (width, position) = (options.width, options.position);
    let (body_padding, frame_padding, gap, radius) = (
        options.body_padding,
        options.frame_padding,
        options.gap,
        options.radius,
    );
    let header = window_header(options.clone(), title, state, mover, bounds, close);
    let body = node()
        .hidden(move || state.with(ToolWindowState::collapsed))
        .style(style().padding(body_padding))
        .child(content(close));
    let group = column((header, body))
        .role(Role::Group)
        .name(message_with(
            "creative.window.name",
            "{title} window",
            move || [("title", title.get())],
        ))
        .node_ref(reference)
        .style(style().width(width))
        .style(move || {
            glass_style()
                .padding(frame_padding)
                .gap(gap)
                .border_radius(radius)
                .overflow(Overflow::Hidden)
        });
    node()
        .role(Role::Dialog)
        .name(title)
        .style(move || {
            let viewport = platform::window_size().get();
            let rect = bounds.get().unwrap_or(Rect::new(0.0, 0.0, width, 0.0));
            let centered = WindowPosition::Center.origin(viewport, rect);
            let initial = position.origin(viewport, rect);
            let offset = state.with(ToolWindowState::offset);
            style().translate(
                initial.x - centered.x + offset.x,
                initial.y - centered.y + offset.y,
            )
        })
        .child(group)
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

/// Moves the window by its title bar. The window is a popup centred on its
/// opener, so the clamp takes its origin from the drawn rect at the drag
/// start: that rect is the popup's placed origin plus the offset the drag
/// starts from (`initial`), so the origin is that rect less `initial`.
fn window_mover(
    state: Signal<ToolWindowState>,
) -> kivact::interaction::DragApi<Option<(Point, Rect)>> {
    use_drag::<Option<(Point, Rect)>>(DragCoordinates::Window).on_update(
        move |update: DragUpdate<Option<(Point, Rect)>>| {
            let Some((initial, rect)) = update.payload else {
                return;
            };
            let viewport = platform::window_size().get();
            let base = Point::new(rect.x - initial.x, rect.y - initial.y);
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

fn toggle_collapsed(state: Signal<ToolWindowState>) {
    state.update(|state| {
        if let ToolWindowState::Open { collapsed, .. } = state {
            *collapsed = !*collapsed;
        }
    });
}

fn window_header(
    options: ToolWindowOptions,
    title: Value<String>,
    state: Signal<ToolWindowState>,
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
                .style(style().color(crate::color::ACCENT))
        }),
        text(title).style(style().font_weight(600).flex_grow(1)),
        (!options.modal).then(move || {
            kivact::ui::button(minimize_label)
                .variant(ButtonVariant::Secondary)
                .hide_label(true)
                .icon(icon("chevron-down"))
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
            .icon(icon("x"))
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

fn window_header_style(options: &ToolWindowOptions) -> Style {
    style()
        .height(options.titlebar_height)
        .padding_left(options.titlebar_padding.0)
        .padding_right(options.titlebar_padding.1)
        .gap(8)
        .align_items(Align::Center)
        .background(crate::color::HEADER)
        .cursor(if options.modal {
            kivact::style::Cursor::Default
        } else {
            kivact::style::Cursor::Grab
        })
}

fn window_backdrop(blur: f64) -> Option<Node> {
    (blur > 0.0).then(move || {
        node().style(
            style()
                .position(Position::Fixed)
                .top(0)
                .right(0)
                .bottom(0)
                .left(0)
                .z_index(100)
                .background(crate::color::TRANSPARENT_BACKDROP)
                .backdrop_blur(blur),
        )
    })
}

fn close_removed_window(open: Signal<bool>) {
    let value = open.into_value();
    on_cleanup(move || {
        if value.try_get() == Some(true) {
            open.set(false);
        }
    });
}
