//! Role: Display reactive person images with initials and semantic status fallback.
use crate::avatar_initials::initials;
use crate::variants::Variants;
use crate::{AvatarShape, AvatarState, ControlSize, Variant};
use kivact_core::state::{Callback, IntoText, IntoValue, Value};
use kivact_core::style::{Position, pct, style};
use kivact_core::view::{
    Asset, ImageFit, ImageStatus, IntoView, Node, NodeBuilder, Role, View, dynamic, node, node_ref,
    text,
};
use kivact_interaction::Tone;
/// The source of the mounted presence label, without an effect-copied string.
#[derive(Clone, Copy)]
enum AvatarStatusSource {
    None,
    Mounted { label: Value<Option<String>> },
}
// xtask-allow(state-fields): Builder inputs and optional content are independent configuration; image state belongs to NodeRef.
pub struct Avatar {
    root: Node,
    name: Value<String>,
    src: Value<Option<Asset>>,
    fallback: Value<Option<Asset>>,
    size: Value<ControlSize>,
    shape: Value<AvatarShape>,
    status: Value<Option<Tone>>,
    status_content: Option<View>,
    variants: Variants,
}
pub fn avatar(name: impl IntoText) -> Avatar {
    let name = name.into_text_value();
    Avatar {
        root: node().role(Role::Image),
        name,
        src: None::<Asset>.into_value(),
        fallback: None::<Asset>.into_value(),
        size: crate::values::default_control_size().into_value(),
        shape: AvatarShape::Circle.into_value(),
        status: None::<Tone>.into_value(),
        status_content: None,
        variants: Variants::default(),
    }
}
impl Avatar {
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
    pub fn variant(self, variant: impl Variant) -> Self {
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
fn picture(asset: Asset, fallback: Option<Asset>, name: Value<String>) -> View {
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
        .style(style().size(pct(100), pct(100)))
        .child(photo)
        .child(kivact_core::view::show(
            move || status.get() != ImageStatus::Ready,
            move || match fallback.clone() {
                Some(asset) => picture(asset, None, name),
                None => text(move || initials(&name.get())).into_view(),
            },
        ))
        .into_view()
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
        let label = move || match source() {
            AvatarStatusSource::None => accessible_name.get(),
            AvatarStatusSource::Mounted { label } => label.get().map_or_else(
                || accessible_name.get(),
                |label| format!("{}, {label}", accessible_name.get()),
            ),
        };
        let content = dynamic(
            move || (src.get(), fallback.get()),
            move |(asset, fallback)| match asset.or_else(|| fallback.clone()) {
                Some(asset) => picture(asset, fallback, name),
                None => text(move || initials(&name.get())).into_view(),
            },
        );
        crate::layer::with_base_recipe_style(
            self.root.name(label),
            |r| r.avatar_recipe(),
            &self.variants,
            move || AvatarState {
                size: size.get(),
                shape: shape.get(),
                status: status.get(),
            },
        )
        .base_style(style().position(Position::Relative))
        .child(content)
        .child(self.status_content)
        .child(kivact_core::view::show(
            move || status.get().is_some(),
            move || {
                crate::layer::with_base_recipe_style(
                    node(),
                    |r| r.avatar_status_recipe(),
                    &crate::variants::Variants::default(),
                    move || AvatarState {
                        size: size.get(),
                        shape: shape.get(),
                        status: status.get(),
                    },
                )
            },
        ))
        .into_view()
    }
}
