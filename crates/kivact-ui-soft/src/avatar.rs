//! Role: Display a person's photo, initials, or silhouette with optional status.
use crate::avatar_group::AvatarGroupContext;
use crate::avatar_options::{AvatarShape, AvatarSize};
use crate::avatar_status_dot::{AvatarStatusContext, status_metrics};
use crate::tokens;
use kivact::i18n::message_with;
use kivact::state::Callback;
use kivact::state::IntoValue;
use kivact::view::ImageFit;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::image;
use kivact::view::node;
use kivact::view::text;

use kivact::state::IntoText;
use kivact::state::Value;
use kivact::state::provide;
use kivact::state::signal;
use kivact::state::try_use_context;
use kivact::style::Align;
use kivact::style::Cursor;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::Style;
use kivact::style::style;
use kivact::ui::tooltip;
use kivact::view::Asset;
use kivact::view::IntoView;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::use_direction;

const PERSON: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/></svg>"#;

/// Select the avatar tooltip's content.
#[derive(Clone, Copy, Default)]
pub enum AvatarTooltip {
    #[default]
    Name,
    Text(Value<String>),
    Off,
}

impl From<bool> for AvatarTooltip {
    fn from(on: bool) -> Self {
        if on { Self::Name } else { Self::Off }
    }
}

impl From<&str> for AvatarTooltip {
    fn from(text: &str) -> Self {
        Self::Text(text.into_text_value())
    }
}

/// A photo or initials with an optional presence indicator.
pub struct Avatar {
    root: Node,
    name: Option<Value<String>>,
    alt: Option<Value<String>>,
    src: Option<Asset>,
    fallback_src: Option<Asset>,
    size: Value<AvatarSize>,
    shape: Value<AvatarShape>,
    status: Option<View>,
    tooltip: AvatarTooltip,
    press: Option<Box<dyn Fn()>>,
    open: Option<Box<dyn Fn()>>,
}

/// Create an avatar.
pub fn avatar() -> Avatar {
    Avatar {
        root: node(),
        name: None,
        alt: None,
        src: None,
        fallback_src: None,
        size: (AvatarSize::default()).into_value(),
        shape: (AvatarShape::default()).into_value(),
        status: None,
        tooltip: AvatarTooltip::default(),
        press: None,
        open: None,
    }
}

impl Avatar {
    /// Set the name used for initials, accessibility, and the default tooltip.
    pub fn name(mut self, name: impl IntoText) -> Self {
        self.name = Some(name.into_text_value());
        self
    }
    /// Override the accessible name.
    pub fn alt(mut self, alt: impl IntoText) -> Self {
        self.alt = Some(alt.into_text_value());
        self
    }
    /// Set the primary photo.
    pub fn src(mut self, src: impl Into<Asset>) -> Self {
        self.src = Some(src.into());
        self
    }
    /// Set the fallback photo.
    pub fn fallback_src(mut self, src: impl Into<Asset>) -> Self {
        self.fallback_src = Some(src.into());
        self
    }
    /// Set the avatar diameter.
    pub fn size(mut self, size: impl IntoValue<AvatarSize>) -> Self {
        self.size = size.into_value();
        self
    }
    /// Set the avatar silhouette.
    pub fn shape(mut self, shape: impl IntoValue<AvatarShape>) -> Self {
        self.shape = shape.into_value();
        self
    }
    /// Add an overlaid status indicator.
    pub fn status(mut self, status: impl IntoView) -> Self {
        self.status = Some(status.into_view());
        self
    }
    /// Select the tooltip behavior.
    pub fn tooltip(mut self, tooltip: impl Into<AvatarTooltip>) -> Self {
        self.tooltip = tooltip.into();
        self
    }
    /// Make the avatar a button.
    pub fn on_press(mut self, press: impl Into<Callback<()>>) -> Self {
        let press: Callback<()> = press.into();
        self.press = Some(Box::new(move || press.call(())));
        self
    }
    /// Make the avatar a link with an application-supplied action.
    pub fn on_open(mut self, open: impl Into<Callback<()>>) -> Self {
        let open: Callback<()> = open.into();
        self.open = Some(Box::new(move || open.call(())));
        self
    }
}

impl NodeBuilder for Avatar {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Avatar {
    fn into_view(self) -> View {
        let Self {
            mut root,
            name,
            alt,
            src,
            fallback_src,
            size,
            shape,
            status,
            tooltip: hint,
            press,
            open,
        } = self;
        let group = try_use_context::<AvatarGroupContext>().map(|value| value.get());
        let (side, shape) = group
            .as_ref()
            .map_or((size.get().side(), shape.get()), |cx| (cx.side, cx.shape));
        let grouped = group.is_some();
        let rtl = use_direction().get() == ReadingDirection::Rtl;
        let status_label = signal(None::<String>);
        let named_status = message_with(
            "soft.avatar.name_with_status",
            "{name}, {status}",
            move || {
                [
                    (
                        "name",
                        meaningful(alt)
                            .or_else(|| meaningful(name))
                            .unwrap_or_default(),
                    ),
                    ("status", status_label.get().unwrap_or_default()),
                ]
            },
        );
        let accessible = move || {
            let name = meaningful(alt).or_else(|| meaningful(name));
            match (name, status_label.get()) {
                (Some(_), Some(_)) => named_status.get(),
                (Some(name), None) | (None, Some(name)) => name,
                (None, None) => String::new(),
            }
        };
        let interactive = open.is_some() || press.is_some();
        let tooltip_label = match hint {
            AvatarTooltip::Name => name.filter(|_| meaningful(name).is_some()),
            AvatarTooltip::Text(value) => Some(value),
            AvatarTooltip::Off => None,
        };
        if interactive {
            let role = if open.is_some() {
                Role::Link
            } else {
                Role::Button
            };
            if let Some(action) = open.or(press) {
                root = root.role(role).on_press(action);
            }
            root = if let Some(group) = group {
                group.register(root)
            } else {
                root.focusable(true)
            };
        } else {
            if meaningful(alt).is_some() || meaningful(name).is_some() || status.is_some() {
                root = root.role(Role::Image);
            }
            root = root.focusable(!grouped && tooltip_label.is_some());
        }
        let content = avatar_content(side, shape, name, fallback_src, src);
        let status = status.map(|status| {
            let dot = status_metrics(side).side;
            let inset = if shape == AvatarShape::Circle {
                side * 0.14645 - dot / 2.0
            } else {
                -dot / 4.0
            };
            let style = style().position(Position::Absolute).bottom(inset);
            node()
                .style(if rtl {
                    style.left(inset)
                } else {
                    style.right(inset)
                })
                .child(provide(
                    AvatarStatusContext {
                        side,
                        label: status_label,
                    },
                    status,
                ))
        });
        let root = root
            .name(accessible)
            .base_style(move || {
                let mut look = style()
                    .position(Position::Relative)
                    .shrink(0)
                    .radius(shape.radius());
                look = if grouped {
                    group_frame(look, side, rtl).size(side + 4.0, side + 4.0)
                } else {
                    look.size(side, side)
                };
                if interactive {
                    look = look.cursor(Cursor::Pointer);
                }
                avatar_focus(look)
            })
            .child(content)
            .child(status);
        if let Some(label) = tooltip_label {
            tooltip(label).attach(root)
        } else {
            root.into_view()
        }
    }
}

pub(crate) fn group_frame(look: Style, side: f64, rtl: bool) -> Style {
    let look = look
        .border(2, tokens().colors.bg_surface)
        .background(tokens().colors.bg_surface);
    let overlap = -(side * 0.25).round();
    if rtl {
        look.margin_right(overlap)
    } else {
        look.margin_left(overlap)
    }
}

pub(crate) fn avatar_focus(look: Style) -> Style {
    let p = tokens();
    look.focus_visible(style().ring_with_offset(
        p.sizes.focus_ring_width,
        p.sizes.focus_ring_offset,
        p.colors.focus_ring,
    ))
}

fn avatar_content(
    side: f64,
    shape: AvatarShape,
    name: Option<Value<String>>,
    fallback: Option<Asset>,
    photo: Option<Asset>,
) -> Node {
    let failed = signal(photo.is_none());
    let picture = move |asset| {
        image(asset).fit(ImageFit::Cover).hit_testable(false).style(
            style()
                .position(Position::Absolute)
                .left(0)
                .top(0)
                .size(side, side),
        )
    };
    node()
        .style(move || {
            let p = tokens();
            style()
                .size(side, side)
                .radius(shape.radius())
                .clip(true)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .direction(Direction::Row)
                .background(p.colors.neutral)
                .color(p.colors.text_secondary)
        })
        .child(kivact::view::show(
            move || meaningful(name).is_some_and(|name| !initials(&name).is_empty()),
            move || {
                text(move || meaningful(name).map_or_else(String::new, |name| initials(&name)))
                    .style(move || {
                        let p = tokens();
                        style()
                            .font_size(side * 0.4)
                            .line_height(1.0)
                            .weight(p.typography.weight_medium)
                            .text_wrap(false)
                    })
            },
        ))
        .child(kivact::view::show(
            move || meaningful(name).is_none_or(|name| initials(&name).is_empty()),
            move || {
                image(Asset::from(PERSON))
                    .monochrome(true)
                    .style(style().size(side * 0.6, side * 0.6))
            },
        ))
        .child(fallback.map(|asset| {
            kivact::view::show(failed, move || {
                picture(asset.clone()).on_image_error(|_| {})
            })
        }))
        .child(photo.map(|asset| {
            picture(asset)
                .hidden(failed)
                .on_image_error(move |_| failed.set(true))
        }))
}

fn meaningful(value: Option<Value<String>>) -> Option<String> {
    value
        .map(|value| value.get())
        .filter(|value| !value.trim().is_empty())
}

fn initials(name: &str) -> String {
    let mut letters = name.split_whitespace().filter_map(initial);
    let Some(first) = letters.next() else {
        return String::new();
    };
    let last = letters.next_back().unwrap_or_default();
    format!("{first}{last}").to_uppercase()
}

fn initial(word: &str) -> Option<&str> {
    let is_flag = |c| matches!(c, '\u{1f1e6}'..='\u{1f1ff}');
    let mut chars = word.char_indices();
    let (start, first) = chars.find(|&(_, c)| {
        c.is_alphanumeric() || matches!(c, '\u{1f000}'..='\u{1faff}' | '\u{2600}'..='\u{27bf}')
    })?;
    let mut end = start + first.len_utf8();
    let mut join = false;
    for (i, c) in chars {
        let extend = matches!(c, '\u{0300}'..='\u{036f}' | '\u{fe0e}'..='\u{fe0f}' | '\u{1f3fb}'..='\u{1f3ff}' | '\u{200d}');
        if !join && !extend && !(is_flag(first) && is_flag(c) && i == start + first.len_utf8()) {
            break;
        }
        join = c == '\u{200d}';
        end = i + c.len_utf8();
    }
    Some(&word[start..end])
}
