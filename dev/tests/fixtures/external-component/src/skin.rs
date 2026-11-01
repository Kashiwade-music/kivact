//! Role: Two alternate looks over the same external components.
//!
//! Two looks over the same components (style/skin invariance): a
//! business-tool skin and a game-console skin.
//!
//! A skin is a typed [`Theme`] (colors, spacing, radius, type, focus ring)
//! plus a handful of *style functions* (`fn(Skin, &Theme) -> Style`:
//! variants are ordinary functions, there is no appearance type). Switching
//! the skin re-resolves paint only: layout sizes that hit regions depend on
//! (row heights, canvas sizes) are fixed by the components, so semantics and
//! hit regions are identical in both skins (tested in `tests/style.rs` and the
//! ext apps). The console skin's hover and pressed states move and scale
//! the controls (CSS `translate` and `scale`).

use kivact::view::View;
use kivact::view::provide_theme;
use kivact::view::try_context;

use kivact::draw::{Shadow, linear_gradient};
use kivact::prelude::*;
use kivact::style::theme::{Palette, Radii, Typography};
use kivact::style::{Align, Color, Justify, Style, rgba};

/// The look of a subtree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skin {
    /// Flat, dense, light: a business tool.
    Business,
    /// Dark, rounded, glowing, larger type: a game-console UI.
    Console,
}

impl Skin {
    pub fn name(self) -> &'static str {
        match self {
            Skin::Business => "Business",
            Skin::Console => "Console",
        }
    }

    pub fn other(self) -> Skin {
        match self {
            Skin::Business => Skin::Console,
            Skin::Console => Skin::Business,
        }
    }

    /// The typed theme of this skin.
    pub fn theme(self) -> Theme {
        match self {
            Skin::Business => {
                let base = Theme::light();
                let radius = Radii {
                    sm: 2.0,
                    md: 4.0,
                    ..*base.radius()
                };
                base.with_radius(radius)
            }
            Skin::Console => {
                let base = Theme::dark();
                let colors = Palette {
                    surface: rgb(0x0b1020),
                    surface_alt: rgb(0x18204a),
                    accent: rgb(0xff2e88),
                    accent_text: rgb(0xffffff),
                    focus: rgb(0xffd400),
                    border: rgb(0x3a4a9a),
                    selection: rgba(0xff2e88, 0.35),
                    ..base.colors().clone()
                };
                let radius = Radii {
                    sm: 8.0,
                    md: 14.0,
                    lg: 20.0,
                };
                let text = Typography {
                    size: 16.0,
                    weight: 500,
                    ..base.text().clone()
                };
                base.with_colors(colors)
                    .with_radius(radius)
                    .with_text(text)
                    .with_focus_ring(4.0)
                    .with_control_height(36.0)
            }
        }
    }
}

/// Context entry: the skin signal of a subtree.
#[derive(Clone, Copy)]
pub struct SkinCtx(pub Signal<Skin>);

/// Provide `skin` (and its theme) to the subtree built by `child`. Setting
/// `skin` switches skin and theme together.
#[track_caller]
pub fn provide_skin<V: IntoView>(skin: Signal<Skin>, child: impl FnOnce() -> V + 'static) -> View {
    provide_theme(
        move || skin.get().theme(),
        move || provide(SkinCtx(skin), child),
    )
}

/// A Copy closure reading the current skin (tracked) as seen from the
/// calling setup. Components without a provided skin get `Skin::Business`.
/// A style reads it inside
/// `.style(move || skin::panel(look(), &current_theme()))`, which also reads
/// the theme.
#[track_caller]
pub fn look() -> impl Fn() -> Skin + Copy + 'static {
    let skin = try_context::<SkinCtx>().map(|c| c.0);
    move || skin.map(|s| s.get()).unwrap_or(Skin::Business)
}

fn glow(color: ColorToken, blur: f32) -> Shadow {
    Shadow::new(0.0, 0.0, blur, Color::token(color))
}

fn drop_shadow() -> Shadow {
    Shadow::new(0.0, 1.0, 2.0, Color::token(ColorToken::Shadow))
}

/// Panel / card surface.
pub fn panel(skin: Skin, t: &Theme) -> Style {
    let base = style().padding(t.space().md).gap(t.space().md);
    match skin {
        Skin::Business => base
            .background(ColorToken::Surface)
            .border(1, ColorToken::Border)
            .radius(t.radius().sm),
        Skin::Console => base
            .background(linear_gradient(180).colors([ColorToken::SurfaceAlt, ColorToken::Surface]))
            .border(1, ColorToken::Border)
            .radius(t.radius().lg)
            .shadow(glow(ColorToken::Accent, 10.0)),
    }
}

/// Input frame (the search box and select border and background).
pub fn field(skin: Skin, t: &Theme) -> Style {
    let base = style()
        .padding_x(t.space().md)
        .gap(t.space().md)
        .min_height(t.control_height())
        .align_items(Align::Center)
        .border(1, ColorToken::Border)
        .background(ColorToken::Surface)
        .radius(t.radius().md);
    match skin {
        Skin::Business => base,
        Skin::Console => base.shadow(glow(ColorToken::Accent, 6.0)),
    }
}

/// Push buttons: layered over the standard `button` style with
/// `button(..).style(move || skin::button(look(), &current_theme()))`.
pub fn button(skin: Skin, t: &Theme) -> Style {
    let base = style()
        .padding_x(t.space().lg)
        .padding_y(t.space().sm)
        .min_height(t.control_height())
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .radius(t.radius().md)
        .color(ColorToken::AccentText)
        .focus(style().ring(t.focus_ring(), ColorToken::Focus))
        .disabled(style().opacity(0.45));
    match skin {
        Skin::Business => base
            .background(ColorToken::Accent)
            .shadow(drop_shadow())
            .hover(style().opacity(0.9))
            .pressed(style().opacity(0.8)),
        Skin::Console => base
            .background(
                linear_gradient(180).colors([Color::from(ColorToken::Accent), rgb(0x7a1cff)]),
            )
            .shadow(glow(ColorToken::Accent, 12.0))
            .hover(style().scale(1.04))
            .pressed(style().translate(0, 1).scale(0.97)),
    }
}

/// Rows of lists: hover, pressed and selected states.
pub fn list_row(skin: Skin, t: &Theme) -> Style {
    let base = style()
        .direction(kivact::style::Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .padding_x(t.space().md)
        .selected(style().background(ColorToken::Selection));
    match skin {
        Skin::Business => base
            .hover(style().background(ColorToken::SurfaceAlt))
            .pressed(style().opacity(0.85)),
        Skin::Console => base
            .radius(t.radius().sm)
            .hover(style().background(ColorToken::SurfaceAlt).translate(4, 0))
            .pressed(style().scale(0.99)),
    }
}

/// Small status chip.
pub fn chip(skin: Skin, t: &Theme, color: ColorToken) -> Style {
    let base = style()
        .padding_x(t.space().sm)
        .radius(t.radius().sm)
        .background(color)
        .color(ColorToken::AccentText);
    match skin {
        Skin::Business => base,
        Skin::Console => base.shadow(glow(color, 6.0)),
    }
}

/// Focusable custom surfaces (lists, canvases): the focus ring.
pub fn focus_ring(t: &Theme) -> Style {
    style().focus(style().ring(t.focus_ring(), ColorToken::Focus))
}
