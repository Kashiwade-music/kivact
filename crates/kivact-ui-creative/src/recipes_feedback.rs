//! Role: Creative appearance of the standard toasts, alerts, empty states, cards and the indicators: badges, avatars, skeletons, spinners and key caps.
use crate::color;
use crate::fonts::mono_family;
use crate::recipe_parts::{ToneExt, button_style};
use crate::size::ROW;
use crate::variants::NoticeVariant;
use kivact::custom::PartState;
use kivact::interaction::ToastState;
use kivact::style::{Position, Style, Theme, style};
use kivact::ui::theme::{
    AlertState, AlertTheme, AvatarState, AvatarTheme, BadgeState, BadgeTheme, CardState, CardTheme,
    EmptyStateState, EmptyStateTheme, KbdKeyState, KbdTheme, Look, Selectable, SkeletonState,
    SkeletonTheme, SpinnerState, SpinnerTheme, ToastTheme,
};

/// The theme of a toast: a glass card edged in its tone's color.
pub(crate) fn toast() -> ToastTheme {
    ToastTheme::default()
        .region(toast_region)
        .toast(toast_card)
        .title(toast_title)
        .body(toast_body)
        .icon(toast_icon)
        .close(toast_close)
}

/// The stack at the bottom right of the window, over scrolling content.
fn toast_region(_t: &Theme, _s: &Look<PartState>, _base: Style) -> Style {
    style()
        .position(Position::Fixed)
        .z_index(100)
        .max_width(kivact::style::pct(100) - 16.0)
        .align_items(kivact::style::Align::End)
        .gap(8)
        .bottom(16)
        .right(16)
}

fn toast_card(_t: &Theme, s: &Look<ToastState>, _base: Style) -> Style {
    let color = s.tone.color();
    style()
        .flex_direction(kivact::style::FlexDirection::Row)
        .width(340)
        .max_width(kivact::style::pct(100))
        .padding(12)
        .gap(10)
        .border_radius(9)
        .align_items(kivact::style::Align::Start)
        .color(color::TEXT)
        .background(color::OVERLAY_SURFACE)
        .backdrop_blur(22)
        .border(1, color.with_alpha(0.22))
        .shadow(kivact::style::Shadow::new(
            0.0,
            12.0,
            30.0,
            color::MENU_SHADOW,
        ))
}

fn toast_title(_t: &Theme, _s: &Look<ToastState>, _base: Style) -> Style {
    style().font_weight(600)
}

fn toast_body(_t: &Theme, _s: &Look<ToastState>, _base: Style) -> Style {
    style().font_size(11.5)
}

fn toast_icon(_t: &Theme, _s: &Look<ToastState>, _base: Style) -> Style {
    style().size(16, 16).flex_shrink(0)
}

/// The square close button, like a toolbar action.
fn toast_close(t: &Theme, _s: &Look<ToastState>, _base: Style) -> Style {
    button_style(t).width(ROW).padding(0)
}

/// The theme of an alert: a tinted panel; a notice variant has room for its
/// icon and dismiss button.
pub(crate) fn alert() -> AlertTheme {
    AlertTheme::default()
        .root(alert_root)
        .title(alert_title)
        .description(alert_description)
        .icon(alert_icon)
        .dismiss(alert_dismiss)
        .actions(alert_actions)
        .content(alert_content)
}

fn alert_root(_t: &Theme, s: &Look<AlertState>, base: Style) -> Style {
    let color = s.tone.color();
    let base = base
        .padding(12)
        .gap(3)
        .border_radius(9)
        .color(color::TEXT)
        .background(color.with_alpha(0.08))
        .border(1, color.with_alpha(0.22));
    if s.variant::<NoticeVariant>().is_some() {
        base.position(Position::Relative)
            .padding_left(38)
            .padding_right(44)
    } else {
        base
    }
}

fn alert_title(_t: &Theme, _s: &Look<AlertState>, base: Style) -> Style {
    base.font_weight(600).font_size(12.5)
}

fn alert_description(_t: &Theme, _s: &Look<AlertState>, base: Style) -> Style {
    base.font_size(11.5).color(color::TEXT)
}

fn alert_icon(_t: &Theme, s: &Look<AlertState>, base: Style) -> Style {
    let base = base.size(16, 16);
    if s.variant::<NoticeVariant>().is_some() {
        base.position(Position::Absolute).left(12).top(12)
    } else {
        base
    }
}

fn alert_dismiss(t: &Theme, s: &Look<AlertState>, base: Style) -> Style {
    if s.variant::<NoticeVariant>().is_some() {
        button_style(t)
            .position(Position::Absolute)
            .right(12)
            .top(12)
            .size(ROW, ROW)
            .padding(0)
    } else {
        base
    }
}

fn alert_actions(_t: &Theme, _s: &Look<AlertState>, base: Style) -> Style {
    base.gap(6)
}

fn alert_content(_t: &Theme, _s: &Look<AlertState>, base: Style) -> Style {
    base.gap(6).font_size(11.5)
}

/// The theme of an empty state: a centered column of muted text.
pub(crate) fn empty_state() -> EmptyStateTheme {
    EmptyStateTheme::default()
        .root(empty_root)
        .title(empty_title)
        .description(empty_description)
        .icon(empty_icon)
        .actions(empty_actions)
}

fn empty_root(_t: &Theme, _s: &Look<EmptyStateState>, base: Style) -> Style {
    base.padding(24).gap(8)
}

fn empty_title(_t: &Theme, _s: &Look<EmptyStateState>, base: Style) -> Style {
    base.font_size(14).font_weight(600).color(color::TEXT_HIGH)
}

fn empty_description(_t: &Theme, _s: &Look<EmptyStateState>, base: Style) -> Style {
    base.font_size(11.5).color(color::MUTED)
}

fn empty_icon(_t: &Theme, _s: &Look<EmptyStateState>, base: Style) -> Style {
    base.color(color::DIM)
}

fn empty_actions(_t: &Theme, _s: &Look<EmptyStateState>, base: Style) -> Style {
    base.gap(6)
}

/// The theme of a card: an island with a header band.
pub(crate) fn card() -> CardTheme {
    CardTheme::default()
        .root(card_root)
        .header(card_header)
        .content(card_content)
        .footer(card_footer)
        .media(card_media)
}

fn card_root(_t: &Theme, s: &Look<CardState>, base: Style) -> Style {
    base.border_radius(9)
        .background(color::ISLAND)
        .color(color::TEXT)
        .border(
            1,
            if s.selection() == Selectable::Selected {
                color::ACCENT
            } else {
                color::HAIR
            },
        )
}

fn card_header(_t: &Theme, _s: &Look<CardState>, base: Style) -> Style {
    base.padding(10).gap(6).background(color::HEADER)
}

fn card_content(_t: &Theme, _s: &Look<CardState>, base: Style) -> Style {
    base.padding(10).gap(6)
}

fn card_footer(_t: &Theme, _s: &Look<CardState>, base: Style) -> Style {
    base.padding(10).gap(6).border_top(1, color::HAIR)
}

fn card_media(_t: &Theme, _s: &Look<CardState>, base: Style) -> Style {
    base.clip(true)
}

/// The theme of a badge: a tinted chip in its tone's color.
pub(crate) fn badge() -> BadgeTheme {
    BadgeTheme::default().root(badge_root).label(badge_label)
}

fn badge_root(_t: &Theme, s: &Look<BadgeState>, base: Style) -> Style {
    let color = s.tone.color();
    base.min_height(18)
        .padding_x(6)
        .gap(4)
        .border_radius(5)
        .font_size(10.5)
        .background(color.with_alpha(0.12))
        .color(color)
        .border(1, color.with_alpha(0.22))
}

fn badge_label(_t: &Theme, _s: &Look<BadgeState>, base: Style) -> Style {
    base.font_weight(500).text_wrap(false)
}

/// The theme of an avatar: a strong disc, with a ring for its status.
pub(crate) fn avatar() -> AvatarTheme {
    AvatarTheme::default()
        .root(avatar_root)
        .status(avatar_status)
}

fn avatar_root(_t: &Theme, _s: &Look<AvatarState>, base: Style) -> Style {
    base.background(color::STRONG)
        .color(color::TEXT_HIGH)
        .font_size(11.5)
        .font_weight(600)
}

fn avatar_status(_t: &Theme, s: &Look<AvatarState>, base: Style) -> Style {
    base.background(s.status.unwrap_or_default().color())
        .border(2, color::ISLAND)
}

/// The theme of a skeleton: a strong placeholder.
pub(crate) fn skeleton() -> SkeletonTheme {
    SkeletonTheme::default().root(skeleton_root)
}

fn skeleton_root(_t: &Theme, _s: &Look<SkeletonState>, base: Style) -> Style {
    base.background(color::STRONG)
}

/// The theme of a spinner: the accent.
pub(crate) fn spinner() -> SpinnerTheme {
    SpinnerTheme::default().root(spinner_root)
}

fn spinner_root(_t: &Theme, _s: &Look<SpinnerState>, base: Style) -> Style {
    base.color(color::ACCENT)
}

/// The theme of a key cap: a field with a bottom edge in the mono family.
pub(crate) fn kbd() -> KbdTheme {
    KbdTheme::default().root(kbd_root).key(kbd_key)
}

fn kbd_root(_t: &Theme, _s: &Look<()>, base: Style) -> Style {
    base.gap(3)
}

fn kbd_key(t: &Theme, _s: &Look<KbdKeyState>, base: Style) -> Style {
    base.min_height(18)
        .min_width(18)
        .padding_x(5)
        .border_radius(4)
        .background(color::FIELD)
        .color(color::MUTED)
        .border_bottom(2, color::HAIR)
        .font_family(mono_family(t))
        .font_size(10.5)
}

/// Install the feedback and indicator recipes into `theme`.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(toast())
        .with(alert())
        .with(empty_state())
        .with(card())
        .with(badge())
        .with(avatar())
        .with(skeleton())
        .with(spinner())
        .with(kbd())
}
