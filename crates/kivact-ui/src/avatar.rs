//! Role: Display reactive person images with initials and semantic status fallback.
use crate::avatar_initials::initials;
use crate::variants::Variants;
use crate::{AvatarShape, AvatarState, ControlSize, Variant};
use kivact_core::state::{Callback, IntoText, IntoValue, Value};
use kivact_core::style::{Align, Justify, Position, pct, style};
use kivact_core::view::{
    Asset, ImageFit, ImageStatus, IntoView, IntoViewFn, Node, NodeBuilder, Role, View, ViewFn,
    dynamic, node, node_ref, text,
};
use kivact_interaction::Tone;
/// The source of the mounted presence label, without an effect-copied string.
#[derive(Clone, Copy)]
enum AvatarStatusSource {
    None,
    Mounted { label: Value<Option<String>> },
}
pub struct Avatar {
    root: Node,
    name: Value<String>,
    src: Value<Option<Asset>>,
    fallback: Value<Option<Asset>>,
    fallback_content: Option<ViewFn>,
    size: Value<ControlSize>,
    shape: Value<AvatarShape>,
    status: Value<Option<Tone>>,
    // xtask-allow(state-fields): builder option - status content
    status_content: Option<View>,
    // xtask-allow(state-fields): builder option - name tooltip
    name_tooltip: Option<Value<bool>>,
    variants: Variants,
}
pub fn avatar(name: impl IntoText) -> Avatar {
    let name = name.into_text_value();
    Avatar {
        root: node().role(Role::Image),
        name,
        src: None::<Asset>.into_value(),
        fallback: None::<Asset>.into_value(),
        fallback_content: None,
        size: crate::values::default_control_size().into_value(),
        shape: AvatarShape::Circle.into_value(),
        status: None::<Tone>.into_value(),
        status_content: None,
        name_tooltip: None,
        variants: Variants::default(),
    }
}
impl Avatar {
    /// Recreate custom fallback content when no image is available.
    pub fn fallback_content<V: IntoView>(mut self, content: impl IntoViewFn<V>) -> Self {
        self.fallback_content = Some(content.into_view_fn());
        self
    }

    pub fn src(mut self, src: impl IntoValue<Option<Asset>>) -> Self {
        self.src = src.into_value();
        self
    }
    pub fn fallback_src(mut self, src: impl IntoValue<Option<Asset>>) -> Self {
        self.fallback = src.into_value();
        self
    }
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }
    pub fn shape(mut self, shape: impl IntoValue<AvatarShape>) -> Self {
        self.shape = shape.into_value();
        self
    }
    pub fn status(mut self, status: impl IntoValue<Option<Tone>>) -> Self {
        self.status = status.into_value();
        self
    }
    pub fn status_content(mut self, content: impl IntoView) -> Self {
        self.status_content = Some(content.into_view());
        self
    }
    /// Show the avatar's name in a tooltip, or not. Without this the theme decides ([`AvatarTokens::name_tooltip`](crate::AvatarTokens::name_tooltip)).
    pub fn name_tooltip(mut self, show: impl IntoValue<bool>) -> Self {
        self.name_tooltip = Some(show.into_value());
        self
    }
    pub fn variant(mut self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
    pub fn on_press(mut self, f: impl Into<Callback<()>>) -> Self {
        let f = f.into();
        self.root = self
            .root
            .part(kivact_interaction::use_button().control())
            .on_press(move || f.call(()));
        self
    }
    pub fn on_open(self, f: impl Into<Callback<()>>) -> Self {
        let mut result = self.on_press(f);
        result.root = result.root.role(Role::Link);
        result
    }
}
impl NodeBuilder for Avatar {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
fn picture(
    asset: Asset,
    fallback: Option<Asset>,
    name: Value<String>,
    content: Option<ViewFn>,
) -> View {
    let reference = node_ref();
    let status = reference.image_status();
    let photo = kivact_core::view::image(asset)
        .node_ref(reference)
        .on_image_error(|_| ())
        .fit(ImageFit::Cover)
        .style(
            style()
                .position(Position::Absolute)
                .left(0)
                .top(0)
                .size(pct(100), pct(100)),
        );
    node()
        .style(
            style()
                .size(pct(100), pct(100))
                .align_items(Align::Center)
                .justify_content(Justify::Center),
        )
        .child(photo)
        .child(kivact_core::view::show(
            move || status.get() != ImageStatus::Ready,
            move || match fallback.clone() {
                Some(asset) => picture(asset, None, name, content.clone()),
                None => fallback_view(name, content.as_ref()),
            },
        ))
        .into_view()
}
fn fallback_view(name: Value<String>, content: Option<&ViewFn>) -> View {
    match content {
        Some(content) => content.call(()),
        None => text(move || initials(&name.get())).into_view(),
    }
}
impl IntoView for Avatar {
    fn into_view(mut self) -> View {
        let name = self.name;
        let accessible_name = self.root.accessible_name().unwrap_or(name);
        let status = self.status;
        let size = self.size;
        let shape = self.shape;
        let src = self.src;
        let fallback = self.fallback;
        let fallback_content = self.fallback_content;
        let label = accessible_label(accessible_name, status);
        let content = dynamic(
            move || (src.get(), fallback.get()),
            move |(asset, fallback)| match (asset, fallback) {
                (Some(asset), fallback) => picture(asset, fallback, name, fallback_content.clone()),
                (None, Some(asset)) => picture(asset, None, name, fallback_content.clone()),
                (None, None) => fallback_view(name, fallback_content.as_ref()),
            },
        );
        let state = move || AvatarState {
            size: size.get(),
            shape: shape.get(),
            status: status.get(),
        };
        let image = crate::layer::with_recipe_style(
            node().child(content),
            |r| r.avatar_image_recipe(),
            &self.variants,
            state,
        );
        let status_badge = status_badge(size, shape, status, &self.variants);
        let status_content = self.status_content.map(|content| {
            crate::layer::with_recipe_style(
                node().child(content),
                |r| r.avatar_status_content_recipe(),
                &self.variants,
                state,
            )
        });
        let root = crate::layer::with_base_recipe_style(
            self.root.name(label),
            |r| r.avatar_recipe(),
            &self.variants,
            state,
        )
        .base_style(style().position(Position::Relative))
        .child(image)
        .child(status_content)
        .child(status_badge);
        match name_tooltip(self.name_tooltip) {
            Some(show) => crate::tooltip(name).enabled(show).attach(root),
            None => root.into_view(),
        }
    }
}

/// The caller's choice, or the theme's default; `None` attaches no tooltip at all.
fn name_tooltip(explicit: Option<Value<bool>>) -> Option<Value<bool>> {
    explicit.or_else(|| {
        let tokens = kivact_core::style::use_theme()
            .get()
            .ext::<crate::UiTokens>();
        tokens.avatar.name_tooltip.then(|| true.into_value())
    })
}

fn accessible_label(accessible_name: Value<String>, status: Value<Option<Tone>>) -> Value<String> {
    let labels = [
        kivact_core::i18n::message("avatar.status.neutral", "Neutral"),
        kivact_core::i18n::message("avatar.status.info", "Information"),
        kivact_core::i18n::message("avatar.status.success", "Available"),
        kivact_core::i18n::message("avatar.status.warning", "Away"),
        kivact_core::i18n::message("avatar.status.danger", "Busy"),
        kivact_core::i18n::message("avatar.status.accent", "Active"),
    ];
    let status_label = (move || {
        status.get().map(|tone| {
            labels[match tone {
                Tone::Neutral => 0,
                Tone::Info => 1,
                Tone::Success => 2,
                Tone::Warning => 3,
                Tone::Danger => 4,
                Tone::Accent => 5,
            }]
            .get()
        })
    })
    .into_value();
    let source = move || {
        if status_label.get().is_some() {
            AvatarStatusSource::Mounted {
                label: status_label,
            }
        } else {
            AvatarStatusSource::None
        }
    };
    (move || match source() {
        AvatarStatusSource::None => accessible_name.get(),
        AvatarStatusSource::Mounted { label } => label.get().map_or_else(
            || accessible_name.get(),
            |label| format!("{}, {label}", accessible_name.get()),
        ),
    })
    .into_value()
}

fn status_badge(
    size: Value<ControlSize>,
    shape: Value<AvatarShape>,
    status: Value<Option<Tone>>,
    variants: &Variants,
) -> View {
    let variants = variants.clone();
    kivact_core::view::show(
        move || status.get().is_some(),
        move || {
            crate::layer::with_base_recipe_style(
                node(),
                |r| r.avatar_status_recipe(),
                &variants,
                move || AvatarState {
                    size: size.get(),
                    shape: shape.get(),
                    status: status.get(),
                },
            )
        },
    )
    .into_view()
}
