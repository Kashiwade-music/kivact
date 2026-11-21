//! Role: Layout, text and sample helpers that the catalog's pages share.

use kivact::prelude::*;
use kivact::style::{GridPlacement, Track};
use kivact::view::Node;
use kivact_ui_pop as ui;
/// A grid of twelve columns for the page's cards.
pub(crate) fn cards_grid(children: impl IntoView) -> Node {
    grid(children).style(
        style()
            .grid_template_columns(Track::repeat(12, Track::minmax(0, Track::fr(1))))
            .gap(16),
    )
}

/// A grid of `count` equal columns.
pub(crate) fn columns(children: impl IntoView, count: u16) -> Node {
    grid(children).style(
        style()
            .grid_template_columns(Track::repeat(count, Track::fr(1)))
            .gap(16),
    )
}

/// A card titled `title` with `content` under the title.
pub(crate) fn card(title: &str, content: impl IntoView) -> kivact::ui::Card {
    kivact::ui::card((
        row(kivact::ui::heading(title.to_string(), 3)
            .style(style().font_size(15).font_weight(600).line_height(1.2)))
        .style(
            style()
                .align_items(Align::Center)
                .min_height(20)
                .gap(8)
                .margin_bottom(16),
        ),
        content,
    ))
}

/// Half of the twelve-column grid.
pub(crate) fn half<C: Element>(element: C) -> C {
    element.style(style().grid_column(GridPlacement::span(6)))
}

/// The whole twelve-column grid, for a card whose heading is wider than half of it.
pub(crate) fn full<C: Element>(element: C) -> C {
    element.style(style().grid_column(GridPlacement::span(12)))
}

/// A third of the twelve-column grid.
pub(crate) fn third<C: Element>(element: C) -> C {
    element.style(style().grid_column(GridPlacement::span(4)))
}

/// A row of `children` that wraps onto more lines when it runs out of width.
pub(crate) fn wrap(children: impl IntoView) -> Node {
    row(children).style(style().gap(8).flex_wrap(true).align_items(Align::Center))
}

/// A numbered section: the number badge, the heading, the subtitle and the content.
pub(crate) fn section(number: &str, title: &str, subtitle: &str, content: impl IntoView) -> Node {
    column((
        row((
            kivact::ui::badge(number).variant(ui::SectionNumber),
            column((
                kivact::ui::heading(title.to_string(), 2),
                text(subtitle.to_string()).style(style().font_size(13).color(color::TEXT_MUTED)),
            )),
        ))
        .style(style().gap(16).align_items(Align::Center).margin_bottom(16)),
        content,
    ))
}

/// Muted caption text.
pub(crate) fn muted(content: &str) -> Node {
    text(content.to_string()).style(style().color(color::TEXT_MUTED))
}

/// The meaning a notification of a hue has.
pub(crate) fn tone_of(hue: ui::Hue) -> kivact::interaction::Tone {
    use kivact::interaction::Tone;
    match hue {
        ui::Hue::Mint => Tone::Success,
        ui::Hue::Yellow => Tone::Warning,
        ui::Hue::Red => Tone::Danger,
        ui::Hue::Blue | ui::Hue::Pink => Tone::Info,
        ui::Hue::Gray => Tone::Neutral,
    }
}

/// The callback that shows a toast of `hue` with a title and a body.
pub(crate) fn notify() -> Callback<(ui::Hue, String, String)> {
    kivact::state::callback(move |(hue, title, body): (ui::Hue, String, String)| {
        let mut toast = kivact::interaction::Toast::new(title.clone())
            .tone(tone_of(hue))
            .body(body)
            .duration(Some(std::time::Duration::from_millis(4200)));
        if title.contains("Reward") {
            toast = toast.icon_name("gift");
        }
        if title.contains("Mission started") {
            toast = toast.icon_name("sword");
        }
        if title == "Copied" {
            toast = toast
                .icon_name("copy")
                .duration(Some(std::time::Duration::from_millis(2200)));
        }
        kivact::interaction::use_toaster().push(toast);
    })
}

/// A sample of the type scale: five lines, each in its size and weight.
pub(crate) fn type_samples() -> Vec<Node> {
    [
        ("Display 32", 32, "Barlow Italic 800"),
        (
            "Heading H2 / 20",
            20,
            "Inter + Noto Sans JP 600 · tracking −2%",
        ),
        ("Heading H3 / 15", 15, "600"),
        (
            "Body text 14 — easy to read and soft.",
            14,
            "400 · line-height 1.5",
        ),
        ("Caption 12 — muted", 12, "500"),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (label, size, sub))| {
        column((
            text(label)
                .style(style().font_size(size).font_weight(if i == 0 {
                    800
                } else if i < 3 {
                    600
                } else {
                    400
                }))
                .style(if i == 0 {
                    style()
                        .font_family("Barlow")
                        .font_style(kivact::style::FontStyle::Italic)
                        .color(color::ACCENT)
                } else {
                    style()
                }),
            text(sub).style(style().font_size(12).color(color::TEXT_MUTED)),
        ))
        .style(
            style()
                .gap(2)
                .padding_bottom(12)
                .border_bottom(1, color::BORDER),
        )
    })
    .collect::<Vec<_>>()
}

/// Samples of the corner radii and the shadows.
pub(crate) fn shape_samples() -> Vec<Node> {
    [
        (4, "sm · checkbox"),
        (8, "md · control"),
        (10, "lg · popover"),
        (14, "xl · card"),
    ]
    .into_iter()
    .map(|(radius, hint)| {
        column((
            text(radius.to_string()).style(style().font_size(16).font_weight(600)),
            text(hint).style(style().font_size(12).color(color::TEXT_MUTED)),
        ))
        .style(
            style()
                .height(88)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .background(color::SURFACE_ALT)
                .border(1, color::BORDER)
                .border_radius(f64::from(radius)),
        )
    })
    .chain(
        [
            (1, "shadow-sm", "card"),
            (2, "shadow-md", "menu"),
            (3, "shadow-lg", "dialog / toast"),
        ]
        .into_iter()
        .map(|(n, label, hint)| {
            column((
                text(label).style(style().font_weight(600)),
                text(hint).style(style().font_size(12).color(color::TEXT_MUTED)),
            ))
            .style(
                style()
                    .height(88)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .border(1, color::BORDER)
                    .background(color::SURFACE)
                    .border_radius(14.0)
                    .shadow(Shadow::new(0, n, n * 4, color::SHADOW)),
            )
        }),
    )
    .collect::<Vec<_>>()
}

/// A sample tile whose corner is cut off by a polygon.
pub(crate) fn cut_corner_sample() -> Node {
    column((
        text("cut").style(style().font_weight(600)),
        text("corner accent").style(style().font_size(12).color(color::TEXT_MUTED)),
    ))
    .style(
        style()
            .height(88)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .background(ui::Hue::Blue.accent())
            .border_radius(14.0)
            .clip_path(
                kivact::style::Shape::polygon()
                    .point(0, 0)
                    .point(pct(100) - 20, 0)
                    .point(pct(100), 20)
                    .point(pct(100), pct(100))
                    .point(0, pct(100)),
            ),
    )
}

/// The look of a decorated avatar: its diameter and hue in logical px and
/// degrees, and whether it has a halo ring and an online dot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AvatarLook {
    size: f64,
    hue: f64,
    halo: bool,
    online: bool,
}

impl Default for AvatarLook {
    fn default() -> Self {
        AvatarLook {
            size: 40.0,
            hue: 205.0,
            halo: false,
            online: false,
        }
    }
}

impl AvatarLook {
    /// The diameter in logical px.
    pub(crate) fn size(mut self, size: impl Into<f64>) -> Self {
        self.size = size.into();
        self
    }

    /// The hue of the gradient, in degrees.
    pub(crate) fn hue(mut self, hue: impl Into<f64>) -> Self {
        self.hue = hue.into();
        self
    }

    /// A halo ring around the avatar.
    pub(crate) fn halo(mut self, halo: bool) -> Self {
        self.halo = halo;
        self
    }

    /// An online dot on the avatar.
    pub(crate) fn online(mut self, online: bool) -> Self {
        self.online = online;
        self
    }
}

/// An avatar of `initials` with its look: the gradient, the halo ring and the online dot.
pub(crate) fn decorated_avatar(
    initials: kivact::state::Value<String>,
    look: AvatarLook,
) -> kivact::ui::Avatar {
    kivact::ui::avatar(initials)
        .fallback_content(move || text(initials))
        .variant(ui::AvatarSize(look.size))
        .variant(ui::AvatarHue(look.hue))
        .status_content(
            ui::avatar_decorations()
                .size(look.size)
                .hue(look.hue)
                .halo(look.halo)
                .online(look.online),
        )
}
