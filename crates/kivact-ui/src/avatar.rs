//! Role: Display reactive person images with initials and semantic status fallback.

pub(crate) mod initials;
pub(crate) mod look;

pub use look::AvatarShape;

use crate::ControlSize;
use crate::avatar::initials::initials;
use crate::avatar::look::AvatarTheme;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::theme::AvatarState;
use kivact_core::custom::Caller;
use kivact_core::state::{IntoValue, Value};
use kivact_core::style::{Align, Justify, Position, pct, style};
use kivact_core::view::{
    Asset, ImageFit, ImageStatus, IntoView, Role, Variants, View, ViewFn, dynamic, node, node_ref,
    text,
};
use kivact_interaction::Tone;
use kivact_macros::component;

/// The source of the mounted presence label, without an effect-copied string.
#[derive(Clone, Copy)]
enum AvatarStatusSource {
    None,
    Mounted { label: Value<Option<String>> },
}

/// A person's picture, or their initials while no picture is available, with
/// an optional status. The name is the accessible name and, when the tooltip
/// is enabled, the tooltip's text. The name also goes on the initials text of
/// the fallback, a part inside the image. `name_tooltip` decides whether the
/// name shows in a tooltip; without it the theme decides.
#[component]
pub fn avatar(
    name: Value<String>,
    #[prop(default = None::<Asset>)] src: Value<Option<Asset>>,
    #[prop(default = None::<Asset>)] fallback_src: Value<Option<Asset>>,
    fallback_content: Option<ViewFn>,
    #[prop(default = crate::control_size::default_control_size())] size: Value<ControlSize>,
    #[prop(default = AvatarShape::Circle)] shape: Value<AvatarShape>,
    #[prop(default = None::<Tone>)] status: Value<Option<Tone>>,
    status_content: Option<View>,
    name_tooltip: Option<Value<bool>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let mut root = node().role(Role::Image);
    if caller.has_press() {
        root = root.part(kivact_interaction::use_button().control());
    }
    let accessible_name = caller.name().unwrap_or(name);
    let label = accessible_label(accessible_name, status);
    let content = dynamic(
        move || (src.get(), fallback_src.get()),
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
    let image = with_recipe_style(
        node().child(content),
        |r: &AvatarTheme| r.image_recipe(),
        &variants,
        state,
    );
    let status_badge = status_badge(size, shape, status, &variants);
    let status_content = status_content.map(|content| {
        with_recipe_style(
            node().child(content),
            |r: &AvatarTheme| r.status_content_recipe(),
            &variants,
            state,
        )
    });
    let root = with_base_recipe_style(
        root.name(label),
        |r: &AvatarTheme| r.root_recipe(),
        &variants,
        state,
    )
    .base_style(style().position(Position::Relative))
    .child(image)
    .child(status_content)
    .child(status_badge);
    // The theme's setting is the default; a caller's `name_tooltip` wins.
    let enabled = name_tooltip.unwrap_or_else(|| {
        let theme_default = *kivact_core::style::use_theme()
            .get()
            .ext::<AvatarTheme>()
            .name_tooltip_value();
        theme_default.into_value()
    });
    crate::tooltip(name, root).enabled(enabled)
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
            with_base_recipe_style(
                node(),
                |r: &AvatarTheme| r.status_recipe(),
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
