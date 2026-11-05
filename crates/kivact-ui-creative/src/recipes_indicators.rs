//! Role: Creative palette and dense typography for canonical indicators and key caps.
use crate::Tokens;
use crate::feedback::ToneExt;
use kivact::style::{Style, Theme};
use kivact::ui::{
    AvatarState, BadgeState, KbdKeyState, Look, SkeletonState, SpinnerState, UiRecipes,
};

pub(crate) fn install(r: UiRecipes) -> UiRecipes {
    r.badge(|prev| {
        move |t: &Theme, s: &Look<BadgeState>| {
            let p = t.ext::<Tokens>();
            let color = s.tone.color(&p);
            prev(t, s)
                .min_height(18)
                .padding_x(6)
                .gap(4)
                .radius(5)
                .font_size(10.5)
                .background(color.with_alpha(0.12))
                .color(color)
                .border(1, color.with_alpha(0.22))
        }
    })
    .badge_label(|prev| {
        move |t: &Theme, s: &Look<BadgeState>| prev(t, s).weight(500).text_wrap(false)
    })
    .avatar(|prev| {
        move |t: &Theme, s: &Look<AvatarState>| {
            let p = t.ext::<Tokens>();
            prev(t, s)
                .background(p.strong)
                .color(p.text_high)
                .font_size(11.5)
                .weight(600)
        }
    })
    .avatar_status(|prev| {
        move |t: &Theme, s: &Look<AvatarState>| {
            let p = t.ext::<Tokens>();
            prev(t, s)
                .background(s.status.unwrap_or_default().color(&p))
                .border(2, p.island)
        }
    })
    .avatar_group(|prev| move |t: &Theme, s: &Look<()>| prev(t, s).gap(0))
    .skeleton(|prev| {
        move |t: &Theme, s: &Look<SkeletonState>| prev(t, s).background(t.ext::<Tokens>().strong)
    })
    .spinner(|prev| {
        move |t: &Theme, s: &Look<SpinnerState>| prev(t, s).color(t.ext::<Tokens>().accent)
    })
    .kbd(|prev| move |t: &Theme, s: &Look<()>| prev(t, s).gap(3))
    .kbd_key(|prev| move |t: &Theme, s: &Look<KbdKeyState>| key_cap(t, prev(t, s)))
}

fn key_cap(t: &Theme, base: Style) -> Style {
    let p = t.ext::<Tokens>();
    base.min_height(18)
        .min_width(18)
        .padding_x(5)
        .radius(4)
        .background(p.field)
        .color(p.muted)
        .border_bottom(2, p.hair)
        .font_family(p.mono_font.as_str())
        .font_size(10.5)
}
