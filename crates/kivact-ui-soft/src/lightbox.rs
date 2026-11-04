//! Role: Display images in a modal gallery with captions and zoom.
use crate::recipes_dialog::{body_text, scrim};
use crate::theme_access::tokens;
use kivact::interaction::use_dialog;
use kivact::motion::Easing;
use kivact::motion::Transition;
use kivact::state::IntoBinding;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::image;
use kivact::view::node;
use kivact::view::text;

use kivact::interaction::DialogMode;
use kivact::state::Binding;
use kivact::state::IntoValue;
use kivact::state::Signal;
use kivact::state::Value;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::Overflow;
use kivact::style::Position;
use kivact::style::Style;
use kivact::style::TransitionProp;
use kivact::style::pct;
use kivact::style::rgb;
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

#[derive(Clone, Debug, PartialEq)]
pub struct LightboxMedia {
    pub image: Asset,
    pub alt: String,
    pub caption: Option<String>,
}

impl LightboxMedia {
    pub fn image(image: impl Into<Asset>, alt: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            alt: alt.into(),
            caption: None,
        }
    }
}

pub struct Lightbox {
    open: Binding<bool>,
    media: Value<Vec<LightboxMedia>>,
    index: Option<Binding<usize>>,
    zoom: bool,
}

pub fn lightbox(
    open: impl IntoBinding<bool>,
    media: impl IntoValue<Vec<LightboxMedia>>,
) -> Lightbox {
    Lightbox {
        open: open.into_binding(),
        media: media.into_value(),
        index: None,
        zoom: false,
    }
}

impl Lightbox {
    pub fn index(mut self, index: impl IntoBinding<usize>) -> Self {
        self.index = Some(index.into_binding());
        self
    }

    pub fn zoom(mut self, zoom: bool) -> Self {
        self.zoom = zoom;
        self
    }
}

#[derive(Clone, Copy)]
pub struct LightboxHandle {
    open: Signal<bool>,
    index: Signal<usize>,
    media: Value<Vec<LightboxMedia>>,
}

pub fn use_lightbox(media: impl IntoValue<Vec<LightboxMedia>>) -> LightboxHandle {
    LightboxHandle {
        open: signal(false),
        index: signal(0),
        media: media.into_value(),
    }
}

impl LightboxHandle {
    pub fn open(self, index: usize) {
        self.index.set(index);
        self.open.set(true);
    }

    pub fn close(self) {
        self.open.set(false);
    }

    pub fn is_open(self) -> Value<bool> {
        self.open.into_value()
    }

    pub fn view(self) -> Lightbox {
        lightbox(self.open, self.media).index(self.index)
    }

    pub fn trigger(self, index: usize) -> impl Fn() + 'static {
        move || self.open(index)
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
            let zoomed = signal(false);
            let pan = signal((0.0f64, 0.0f64));
            let zoom = self.zoom;
            let step = move |delta: isize| {
                let next = active().saturating_add_signed(delta);
                if next < media.get().len() && next != active() {
                    index.set(next);
                    zoomed.set(false);
                    pan.set((0.0, 0.0));
                }
            };
            let set_zoom = move |on| {
                zoomed.set(on);
                if !on {
                    pan.set((0.0, 0.0));
                }
            };
            let content = kivact::view::dynamic(selected, move |item| {
                item.map(|item| {
                    let failed = signal(false);
                    let photo = image(item.image)
                        .name(item.alt)
                        .hidden(failed)
                        .on_image_error(move |_| failed.set(true))
                        .style(move || {
                            let (x, y) = pan.get();
                            let scale = if zoomed.get() { 2.0 } else { 1.0 };
                            style()
                                .width(pct(100))
                                .min_height(0)
                                .grow(1)
                                .shrink(1)
                                .translate(x, y)
                                .scale_xy(scale, scale)
                                .transition(
                                    TransitionProp::Paint,
                                    Transition::tween(200, Easing::EaseOut),
                                )
                        });
                    let photo = if zoom {
                        let drag_start = kivact::var((0.0f64, 0.0f64));
                        let drag = kivact::interaction::use_drag::<()>(
                            kivact::interaction::DragCoordinates::Window,
                        )
                        .on_start(move |_| drag_start.set(pan.get()))
                        .on_update(
                            move |event: kivact::interaction::DragUpdate<()>| {
                                if zoomed.get() {
                                    let (x, y) = drag_start.get();
                                    pan.set((x + event.delta.x, y + event.delta.y));
                                }
                            },
                        );
                        photo.part(drag.source(())).on_pointer(move |event| {
                            if let kivact::view::PointerPhase::Up(press) = event.phase()
                                && press.clicks == 2
                                && press.button == kivact::view::MouseButton::Primary
                            {
                                set_zoom(!zoomed.get());
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
                            .color(rgb(0xffffff)),
                    );
                    let error = kivact::i18n::message(
                        "soft.lightbox.image_unavailable",
                        "Image unavailable",
                    );
                    let fallback = text(move || {
                        if failed.get() {
                            error.get()
                        } else {
                            String::new()
                        }
                    })
                    .role(Role::Status)
                    .live_region(LiveRegion::Polite)
                    .style(move || {
                        body_text(&tokens())
                            .color(rgb(0xffffff))
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
                })
            });
            let close = icon_button("x", "soft.lightbox.close", "Close")
                .node_ref(close_ref)
                .on_press(move || open.set(false))
                .style(control_position().top(12).right(12));
            let zoom_control = zoom.then(|| {
                icon_button("search", "soft.lightbox.zoom", "Zoom")
                    .with_node(move |node| node.selected(zoomed))
                    .on_press(move || set_zoom(!zoomed.get()))
                    .style(control_position().top(12).right(52))
            });
            let controls = show(
                move || media.get().len() > 1,
                move || {
                    let previous =
                        icon_button("chevron-left", "soft.lightbox.previous", "Previous")
                            .disabled(move || active() == 0)
                            .on_press(move || step(-1))
                            .style(control_position().top(pct(50)).left(12).margin_top(-16));
                    let next = icon_button("chevron-right", "soft.lightbox.next", "Next")
                        .disabled(move || active() + 1 >= media.get().len())
                        .on_press(move || step(1))
                        .style(control_position().top(pct(50)).right(12).margin_top(-16));
                    let counter = text(move || format!("{} / {}", active() + 1, media.get().len()))
                        .style(|| {
                            body_text(&tokens())
                                .color(rgb(0xffffff))
                                .position(Position::Absolute)
                                .top(12)
                                .left(12)
                        });
                    (previous, next, counter)
                },
            );
            let backdrop = node()
                .role(Role::Generic)
                .style(|| scrim(&tokens()).position(Position::Absolute))
                .on_press(move || open.set(false));
            let frame = column((backdrop, content, close, zoom_control, controls))
                .style(|| {
                    let viewport = kivact::platform::window_size().get();
                    style().size(viewport.width, viewport.height)
                })
                .on_key(move |event| {
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
                    if zoomed.get() {
                        pan.update(|(x, y)| {
                            *x -= dx as f64 * 50.0;
                            *y -= dy as f64 * 50.0;
                        });
                    } else if dx != 0 {
                        step(if rtl { -dx } else { dx });
                    } else {
                        return Handled::No;
                    }
                    Handled::Yes
                });
            (overlay(node_ref(), Placement::Center, frame)
                .viewport_margin(0.0)
                .part(behavior.content())
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
        .color(rgb(0xffffff))
}
