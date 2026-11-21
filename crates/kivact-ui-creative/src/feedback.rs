//! Role: Popup buttons, split buttons and the data of tool windows in the Creative kit.
use kivact::component;
use kivact::custom::Caller;
use kivact::geometry::{Point, Rect};
use kivact::i18n::message;
use kivact::interaction::MenuEntry;
use kivact::state::{Callback, IntoValue, Signal, Value, signal};
use kivact::style::{Align, Icon, Overflow, style};
use kivact::ui::ButtonVariant;
use kivact::view::{Element, Role, ViewFn, icon, row};

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
/// `on_press` runs the primary command. `on_activate` receives the key of
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
