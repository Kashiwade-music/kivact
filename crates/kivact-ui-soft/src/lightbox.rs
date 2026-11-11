//! Role: Display images in a modal gallery with captions and zoom.
use crate::recipes_dialog::{body_text, scrim};

use kivact::interaction::use_dialog;
use kivact::motion::Easing;
use kivact::motion::Transition;
use kivact::state::IntoBinding;
use kivact::view::Element;
use kivact::view::Node;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::image;
use kivact::view::node;
use kivact::view::text;

use kivact::geometry::Point;
use kivact::interaction::DialogMode;
use kivact::interaction::{ViewTransform, use_zoom_pan};
use kivact::state::Binding;
use kivact::state::IntoValue;
use kivact::state::Signal;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::Overflow;
use kivact::style::Position;
use kivact::style::Style;
use kivact::style::TransitionProp;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::button;
use kivact::view::Asset;
use kivact::view::Handled;
use kivact::view::IntoView;
use kivact::view::Key;
use kivact::view::LiveRegion;
use kivact::view::Placement;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::node_ref;
use kivact::view::overlay;
use kivact::view::show;

/// Image and descriptive text for one gallery entry.
#[derive(Clone, Debug, PartialEq)]
pub struct LightboxMedia {
    /// Image asset to display.
    pub image: Asset,
    /// Alternative text used to name the displayed image.
    pub alt: String,
    /// Optional visible caption beneath the image.
    pub caption: Option<String>,
}

impl LightboxMedia {
    /// Create an image entry without a caption.
    pub fn image(image: impl Into<Asset>, alt: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            alt: alt.into(),
            caption: None,
        }
    }
}

/// Modal image gallery controlled by open and optional index bindings.
pub struct Lightbox {
    open: Binding<bool>,
    media: Value<Vec<LightboxMedia>>,
    // xtask-allow(state-fields): option: the caller's index binding from Lightbox::index; None keeps the gallery's own index from 0
    index: Option<Binding<usize>>,
    // xtask-allow(state-fields): option: fixed at setup; absent means zoom is disabled
    zoom: bool,
    // xtask-allow(state-fields): option: absent keeps the gallery's own fit and zoom view; LightboxHandle::view passes a binding projected from its state
    view: Option<Binding<LightboxView>>,
}

/// Create a gallery starting at index zero with zoom disabled.
pub fn lightbox(
    open: impl IntoBinding<bool>,
    media: impl IntoValue<Vec<LightboxMedia>>,
) -> Lightbox {
    Lightbox {
        open: open.into_binding(),
        media: media.into_value(),
        index: None,
        zoom: false,
        view: None,
    }
}

impl Lightbox {
    /// Use the caller's zero-based active-index binding.
    pub fn index(mut self, index: impl IntoBinding<usize>) -> Self {
        self.index = Some(index.into_binding());
        self
    }

    /// Enable zoom and pan interaction; disabled by default and fixed at setup.
    pub fn zoom(mut self, zoom: bool) -> Self {
        self.zoom = zoom;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum LightboxView {
    Fit,
    Zoomed { pan: Point },
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum LightboxState {
    Closed,
    Open { index: usize, view: LightboxView },
}

/// Shared gallery handle owning its open, selected-image and zoom state.
#[derive(Clone, Copy)]
pub struct LightboxHandle {
    state: Signal<LightboxState>,
    media: Value<Vec<LightboxMedia>>,
}

/// Create a closed gallery handle over a read-only media collection.
pub fn use_lightbox(media: impl IntoValue<Vec<LightboxMedia>>) -> LightboxHandle {
    LightboxHandle {
        state: signal(LightboxState::Closed),
        media: media.into_value(),
    }
}

impl LightboxHandle {
    /// Show the zero-based gallery entry in fit-to-view mode.
    pub fn show(self, index: usize) {
        self.state.set(LightboxState::Open {
            index,
            view: LightboxView::Fit,
        });
    }

    /// Hide this gallery.
    pub fn hide(self) {
        self.state.set(LightboxState::Closed);
    }

    /// Observe whether this gallery is open.
    pub fn is_open(self) -> Value<bool> {
        (move || matches!(self.state.get(), LightboxState::Open { .. })).into_value()
    }

    /// Create a gallery view connected to this handle's existing state.
    pub fn view(self) -> Lightbox {
        let open = binding(
            move || matches!(self.state.get(), LightboxState::Open { .. }),
            move |open| {
                if open {
                    self.show(0);
                } else {
                    self.hide();
                }
            },
        );
        let index = binding(
            move || match self.state.get() {
                LightboxState::Closed => 0,
                LightboxState::Open { index, .. } => index,
            },
            move |index| {
                self.state.update(|state| {
                    if let LightboxState::Open { index: current, .. } = state {
                        *current = index;
                    }
                });
            },
        );
        let mut lightbox = lightbox(open, self.media).index(index);
        lightbox.view = Some(binding(
            move || match self.state.get() {
                LightboxState::Closed => LightboxView::Fit,
                LightboxState::Open { view, .. } => view,
            },
            move |view| {
                self.state.update(|state| {
                    if let LightboxState::Open { view: current, .. } = state {
                        *current = view;
                    }
                })
            },
        ));
        lightbox
    }

    /// Return an activation callback that opens the given zero-based entry.
    pub fn trigger(self, index: usize) -> impl Fn() + 'static {
        move || self.show(index)
    }
}

impl IntoView for Lightbox {
    fn into_view(self) -> View {
        let open = self.open;
        let media = self.media;
        let index = self.index.unwrap_or_else(|| signal(0).into_binding());
        let active = move || index.get().min(media.get().len().saturating_sub(1));
        let selected = move || media.get().get(active()).cloned();
        let rtl = kivact::view::use_direction().get() == kivact::view::ReadingDirection::Rtl;
        let fallback = kivact::i18n::message("soft.lightbox.media_viewer", "Media viewer");
        let label = move || {
            selected()
                .map(|item| item.alt)
                .filter(|alt| !alt.is_empty())
                .unwrap_or_else(|| fallback.get())
        };
        let close_ref = node_ref();
        let behavior = use_dialog(open, DialogMode::Modal).initial_focus(close_ref);
        show(open, move || {
            let view = self
                .view
                .unwrap_or_else(|| signal(LightboxView::Fit).into_binding());
            let zoomed = move || matches!(view.get(), LightboxView::Zoomed { .. });
            let transform = gallery_transform(view);
            let zoom_behavior = use_zoom_pan(transform).scale_range(1.0, 2.0);
            let zoom = self.zoom;
            let step = move |delta: isize| {
                let next = active().saturating_add_signed(delta);
                if next < media.get().len() && next != active() {
                    index.set(next);
                    view.set(LightboxView::Fit);
                }
            };
            let set_zoom = move |on| {
                view.set(if on {
                    LightboxView::Zoomed {
                        pan: Point::new(0.0, 0.0),
                    }
                } else {
                    LightboxView::Fit
                });
            };
            let content = kivact::view::dynamic(selected, move |item| {
                item.map(|item| {
                    gallery_image(item, transform, zoomed, set_zoom, zoom, zoom_behavior)
                })
            });
            let close = icon_button("x", "soft.lightbox.close", "Close")
                .node_ref(close_ref)
                .on_press(move || open.set(false))
                .style(control_position().top(12).right(12));
            let zoom_control = zoom.then(|| {
                icon_button("search", "soft.lightbox.zoom", "Zoom")
                    .with_node(move |node| node.selected(zoomed))
                    .on_press(move || set_zoom(!zoomed()))
                    .style(control_position().top(12).right(52))
            });
            let controls = gallery_paging(media, active, step);
            let backdrop = node()
                .with_node(move |node| node.role(Role::Generic))
                .style(|| {
                    scrim(&kivact::style::use_tokens::<crate::Tokens>())
                        .position(Position::Absolute)
                })
                .on_press(move || open.set(false));
            let frame = column((backdrop, content, close, zoom_control, controls))
                .style(|| {
                    let viewport = kivact::platform::window_size().get();
                    style().size(viewport.width, viewport.height)
                })
                .on_key(move |event| gallery_key(event, zoom, zoomed, set_zoom, view, step, rtl));
            (overlay(node_ref(), Placement::Center, frame)
                .viewport_margin(0.0)
                .with_node(move |node| node.part(behavior.content()))
                .name(label),)
        })
        .into_view()
    }
}

fn icon_button(
    icon_name: &'static str,
    key: &'static str,
    fallback: &'static str,
) -> kivact::ui::Button {
    button(kivact::i18n::message(key, fallback))
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .icon(icon(icon_name).style(style().size(16, 16)))
}

fn control_position() -> Style {
    style()
        .position(Position::Absolute)
        .size(32, 32)
        .padding(0)
        .color(kivact::style::use_tokens::<crate::Tokens>().colors.on_dark)
}

fn gallery_transform(view: Binding<LightboxView>) -> Binding<ViewTransform> {
    binding(
        move || match view.get() {
            LightboxView::Fit => ViewTransform {
                scale: 1.0,
                offset: Point::new(0.0, 0.0),
            },
            LightboxView::Zoomed { pan } => ViewTransform {
                scale: 2.0,
                offset: pan,
            },
        },
        move |transform: ViewTransform| {
            view.set(if transform.scale <= 1.0 {
                LightboxView::Fit
            } else {
                LightboxView::Zoomed {
                    pan: transform.offset,
                }
            })
        },
    )
}

fn gallery_image(
    item: LightboxMedia,
    transform: Binding<ViewTransform>,
    zoomed: impl Fn() -> bool + Copy + 'static,
    set_zoom: impl Fn(bool) + Copy + 'static,
    zoom: bool,
    zoom_behavior: kivact::interaction::ZoomPanApi,
) -> Node {
    let photo_ref = node_ref();
    let status = photo_ref.image_status();
    let failed = move || status.get() == kivact::view::ImageStatus::Failed;
    let photo = image(item.image)
        .name(item.alt)
        .node_ref(photo_ref)
        .hidden(failed)
        .on_image_error(|_| Handled::Yes)
        .style(move || {
            let Point { x, y } = transform.get().offset;
            let scale = if zoomed() { 2.0 } else { 1.0 };
            style()
                .width(pct(100))
                .min_height(0)
                .grow(1)
                .shrink(1)
                .translate(x, y)
                .scale_xy(scale, scale)
                .transition(TransitionProp::All, Transition::tween(200, Easing::EaseOut))
        });
    let photo = if zoom {
        photo
            .with_node(move |node| node.part(zoom_behavior.viewport()))
            .on_pointer(move |event| {
                if let kivact::view::PointerPhase::Up(press) = event.phase()
                    && press.clicks == 2
                    && press.button == kivact::view::MouseButton::Primary
                {
                    set_zoom(!zoomed());
                }
                Handled::No
            })
    } else {
        photo
    };
    let caption = text(item.caption.unwrap_or_default()).style(
        style()
            .font_size(17)
            .line_height(24.0 / 17.0)
            .text_align(kivact::style::TextAlign::Center)
            .padding_top(8)
            .padding_x(12)
            .max_width(600)
            .color(kivact::style::use_tokens::<crate::Tokens>().colors.on_dark),
    );
    let error = kivact::i18n::message("soft.lightbox.image_unavailable", "Image unavailable");
    let fallback = text(move || if failed() { error.get() } else { String::new() })
        .with_node(move |node| node.role(Role::Status))
        .live_region(LiveRegion::Polite)
        .style(move || {
            body_text(&kivact::style::use_tokens::<crate::Tokens>())
                .color(kivact::style::use_tokens::<crate::Tokens>().colors.on_dark)
                .position(Position::Absolute)
        });
    column((photo, fallback, caption)).style(
        style()
            .width(pct(100))
            .height(pct(100))
            .padding(48)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .overflow(Overflow::Hidden),
    )
}

fn gallery_key(
    event: kivact::view::KeyEvent,
    zoom: bool,
    zoomed: impl Fn() -> bool,
    set_zoom: impl Fn(bool),
    view: Binding<LightboxView>,
    step: impl Fn(isize),
    rtl: bool,
) -> Handled {
    let (dx, dy) = match event.key() {
        Key::Char('+' | '=') if zoom => {
            set_zoom(true);
            return Handled::Yes;
        }
        Key::Char('-') if zoom => {
            set_zoom(false);
            return Handled::Yes;
        }
        Key::ArrowLeft => (-1, 0),
        Key::ArrowRight => (1, 0),
        Key::ArrowUp => (0, -1),
        Key::ArrowDown => (0, 1),
        _ => return Handled::No,
    };
    if zoomed() {
        if let LightboxView::Zoomed { mut pan } = view.get() {
            pan.x -= dx as f64 * 50.0;
            pan.y -= dy as f64 * 50.0;
            view.set(LightboxView::Zoomed { pan });
        }
    } else if dx != 0 {
        step(if rtl { -dx } else { dx });
    } else {
        return Handled::No;
    }
    Handled::Yes
}

fn gallery_paging(
    media: Value<Vec<LightboxMedia>>,
    active: impl Fn() -> usize + Copy + 'static,
    step: impl Fn(isize) + Copy + 'static,
) -> impl IntoView {
    show(
        move || media.get().len() > 1,
        move || {
            let previous = icon_button("chevron-left", "soft.lightbox.previous", "Previous")
                .disabled(move || active() == 0)
                .on_press(move || step(-1))
                .style(control_position().top(pct(50)).left(12).margin_top(-16));
            let next = icon_button("chevron-right", "soft.lightbox.next", "Next")
                .disabled(move || active() + 1 >= media.get().len())
                .on_press(move || step(1))
                .style(control_position().top(pct(50)).right(12).margin_top(-16));
            let counter =
                text(move || format!("{} / {}", active() + 1, media.get().len())).style(|| {
                    body_text(&kivact::style::use_tokens::<crate::Tokens>())
                        .color(kivact::style::use_tokens::<crate::Tokens>().colors.on_dark)
                        .position(Position::Absolute)
                        .top(12)
                        .left(12)
                });
            (previous, next, counter)
        },
    )
}
