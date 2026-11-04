//! Role: Preview an image attachment with upload, open, and remove affordances.
use crate::{IconSize, icon, tokens, visually_hidden};
use kivact::control::{ImageFit, Node, NodeBuilder, image, node};
use kivact::i18n::{message, message_with};
use kivact::motion::{Animated, Easing, Repeat, Transition};
use kivact::primitive::{NodeExt, part};
use kivact::style::{Align, Cursor, Direction, Justify, Position, TransitionProp, pct, style};
use kivact::ui::{ButtonVariant, ControlSize, button, tooltip};
use kivact::view::{
    IntoText, IntoValue, IntoView, LiveRegion, ReadingDirection, Role, View, current_direction,
    node_ref, show,
};
use kivact::{Asset, Value, signal};

const PLACEHOLDER: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M21 19V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2M8.5 13.5l2.5 3 3.5-4.5 4.5 6H5l3.5-5.5z"/></svg>"#;
const SPINNER: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 14 14"><circle cx="7" cy="7" r="5" fill="none" stroke="white" opacity=".3" stroke-width="2"/><path d="M7 2a5 5 0 0 1 3.5355 8.5355" fill="none" stroke="white" stroke-width="2" stroke-linecap="round"/></svg>"#;
const SIDE: f64 = 64.0;

/// When the remove button is visible.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RemoveReveal {
    #[default]
    Hover,
    Always,
}

/// A preview of an image attachment.
pub struct Thumbnail {
    root: Node,
    source: Option<Asset>,
    alt: Option<Value<String>>,
    label: Option<Value<String>>,
    press: Option<Box<dyn Fn()>>,
    remove: Option<Box<dyn Fn()>>,
    loading: Value<bool>,
    disabled: Value<bool>,
    reveal: RemoveReveal,
}

/// Create an image attachment preview.
pub fn thumbnail() -> Thumbnail {
    Thumbnail {
        root: node().role(Role::Group),
        source: None,
        alt: None,
        label: None,
        press: None,
        remove: None,
        loading: false.into_value(),
        disabled: false.into_value(),
        reveal: RemoveReveal::default(),
    }
}

impl Thumbnail {
    /// Set the preview image.
    pub fn src(mut self, source: impl Into<Asset>) -> Self {
        self.source = Some(source.into());
        self
    }
    /// Describe the image itself.
    pub fn alt(mut self, alt: impl IntoText) -> Self {
        self.alt = Some(alt.into_text_value());
        self
    }
    /// Set the filename or another short preview label.
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = Some(label.into_text_value());
        self
    }
    /// Open the image when pressed.
    pub fn on_press(mut self, action: impl Fn() + 'static) -> Self {
        self.press = Some(Box::new(action));
        self
    }
    /// Remove the attachment when pressed.
    pub fn on_remove(mut self, action: impl Fn() + 'static) -> Self {
        self.remove = Some(Box::new(action));
        self
    }
    /// Show upload progress.
    pub fn loading(mut self, loading: impl IntoValue<bool>) -> Self {
        self.loading = loading.into_value();
        self
    }
    /// Make the preview and its controls unavailable.
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = disabled.into_value();
        self
    }
    /// Set when the remove control is shown.
    pub fn show_remove_on(mut self, reveal: RemoveReveal) -> Self {
        self.reveal = reveal;
        self
    }
}

impl NodeBuilder for Thumbnail {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Thumbnail {
    fn into_view(self) -> View {
        let Self {
            root,
            source,
            alt,
            label,
            press,
            remove,
            loading,
            disabled,
            reveal,
        } = self;
        let fallback = message("soft.thumbnail.fallback_name", "Thumbnail");
        let accessible =
            move || match (label.map(|value| value.get()), alt.map(|value| value.get())) {
                (Some(label), Some(alt)) if !label.is_empty() && !alt.is_empty() => {
                    format!("{label} — {alt}")
                }
                (Some(label), _) if !label.is_empty() => label,
                (_, Some(alt)) if !alt.is_empty() => alt,
                _ => fallback.get(),
            };
        let reference = node_ref();
        let hovered = reference.hovered();
        let focus_within = reference.focus_within();
        let rtl = current_direction() == ReadingDirection::Rtl;
        let has_source = source.is_some();
        let media = thumbnail_media(source, alt, loading);
        let open = if let Some(action) = press {
            let label = message_with("soft.thumbnail.open", "Open {name}", move || {
                [("name", accessible())]
            });
            let interaction = part(|input| input).on_press(action);
            let state = interaction.state();
            node()
                .role(Role::Button)
                .name(label)
                .focusable(true)
                .focusable_when_disabled(true)
                .disabled(move || loading.get() || disabled.get())
                .part(interaction)
                .style(|| {
                    let p = tokens();
                    style().size(SIDE, SIDE).cursor(Cursor::Pointer).transition(
                        TransitionProp::Paint,
                        Transition::tween(p.motion.fast as f64, p.motion.ease_standard),
                    )
                })
                .child(media)
                .child(node().hit_testable(false).style(move || {
                    let p = tokens();
                    let input = state.get();
                    let color = if input.pressed {
                        p.colors.overlay_pressed
                    } else if input.hovered {
                        p.colors.overlay_hover
                    } else {
                        kivact::style::Color::TRANSPARENT
                    };
                    fill().background(color).transition(
                        TransitionProp::Paint,
                        Transition::tween(p.motion.fast as f64, p.motion.ease_standard),
                    )
                }))
        } else {
            media
        };
        let remove = remove.map(|remove| {
            let label = message_with("soft.thumbnail.remove", "Remove {name}", move || {
                [("name", accessible())]
            });
            node()
                .style(move || {
                    let p = tokens();
                    let look = style()
                        .position(Position::Absolute)
                        .top(p.spacing.s1)
                        .opacity(
                            if reveal == RemoveReveal::Always || hovered.get() || focus_within.get()
                            {
                                1.0
                            } else {
                                0.0
                            },
                        )
                        .transition(
                            TransitionProp::Paint,
                            Transition::tween(p.motion.fast as f64, p.motion.ease_standard),
                        );
                    if rtl {
                        look.left(p.spacing.s1)
                    } else {
                        look.right(p.spacing.s1)
                    }
                })
                .child(
                    button(label)
                        .variant(ButtonVariant::Secondary)
                        .size(ControlSize::Small)
                        .hide_label(true)
                        .icon(icon("x").size(IconSize::Xs))
                        .disabled(disabled)
                        .on_press(remove)
                        .style(|| {
                            let p = tokens();
                            style()
                                .size(p.sizes.icon_md, p.sizes.icon_md)
                                .min_width(p.sizes.icon_md)
                                .min_height(p.sizes.icon_md)
                                .padding(0)
                                .radius(p.radius.inner)
                                .background(p.colors.overlay_scrim)
                                .color(p.colors.on_dark)
                        }),
                )
        });
        let loading_message = message("soft.spinner.loading", "Loading");
        let announce = visually_hidden(move || {
            if has_source && loading.get() {
                loading_message.get()
            } else {
                String::new()
            }
        })
        .live(LiveRegion::Polite);
        let frame = node()
            .node_ref(reference)
            .style(|| {
                let p = tokens();
                style()
                    .position(Position::Relative)
                    .size(SIDE, SIDE)
                    .radius(p.radius.element)
                    .clip(true)
                    .background(p.colors.neutral)
            })
            .child(open)
            .child(remove);
        let root = root
            .name(accessible)
            .base_style(move || {
                style()
                    .position(Position::Relative)
                    .width(SIDE)
                    .shrink(0)
                    .opacity(if disabled.get() {
                        tokens().sizes.disabled_opacity
                    } else {
                        1.0
                    })
            })
            .child(frame)
            .child(announce);
        if let Some(label) = label {
            tooltip(label).attach(root)
        } else {
            root.into_view()
        }
    }
}

fn fill() -> kivact::style::Style {
    style()
        .position(Position::Absolute)
        .top(0)
        .left(0)
        .size(pct(100), pct(100))
}

fn thumbnail_media(
    source: Option<Asset>,
    alt: Option<Value<String>>,
    loading: Value<bool>,
) -> Node {
    let mut media = node()
        .style(
            style()
                .size(SIDE, SIDE)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .direction(Direction::Row),
        )
        .child(image(Asset::from(PLACEHOLDER)).monochrome(true).style(|| {
            let p = tokens();
            style()
                .size(p.sizes.icon_lg, p.sizes.icon_lg)
                .color(p.colors.icon_secondary)
        }));
    if let Some(source) = source {
        let failed = signal(false);
        let mut photo = image(source)
            .fit(ImageFit::Cover)
            .style(fill())
            .hidden(failed)
            .on_image_error(move |_| failed.set(true));
        if let Some(alt) = alt {
            photo = photo.name(alt);
        }
        media = media
            .child(photo)
            .child(node().hit_testable(false).style(move || {
                let p = tokens();
                fill()
                    .radius(p.radius.element)
                    .border(p.sizes.border_width, p.colors.border)
            }))
            .child(show(loading, upload_overlay));
    } else {
        media = media.child(show(loading, || {
            let p = tokens();
            let pulse = Animated::from(
                0.25f64,
                0.925f64,
                Transition::tween(p.motion.medium_max as f64, Easing::Linear)
                    .delay(1000)
                    .alternate()
                    .repeat(Repeat::Forever),
            );
            node()
                .hit_testable(false)
                .style(move || {
                    let p = tokens();
                    fill()
                        .background(p.colors.bg_surface)
                        .color(p.colors.skeleton)
                        .direction(Direction::Column)
                        .radius(p.radius.element)
                        .clip(true)
                        .align_items(Align::Stretch)
                })
                .child(node().style(move || {
                    fill().background(tokens().colors.skeleton).opacity(
                        if kivact::platform::reduced_motion() {
                            0.25
                        } else {
                            0.25 + ((pulse.get() - 0.25) / 0.075).floor() * 0.075
                        },
                    )
                }))
        }));
    }
    media
}

fn upload_overlay() -> Node {
    let duration = if kivact::platform::reduced_motion() {
        3000
    } else {
        tokens().motion.slow_min
    };
    let turn = Animated::from(
        0.0f64,
        360.0,
        Transition::tween(duration as f64, Easing::Linear)
            .repeat(Repeat::Forever)
            .decorative(false),
    );
    node()
        .hit_testable(false)
        .style(|| {
            fill()
                .background(tokens().colors.overlay_scrim)
                .direction(Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
        })
        .child(image(Asset::from(SPINNER)).style(move || style().size(14, 14).rotate(turn.get())))
}
