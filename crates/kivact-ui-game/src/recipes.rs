//! Role: Game appearance recipes for standard UI controls.
use crate::choice::{Boxed, ChoiceCard, Navigation};
use crate::choice_style;
use crate::tokens::Tokens;
use kivact::custom::PartState;
use kivact::draw::{LineCap, LineJoin, Path, Shadow, Stroke, TextFormat, Transform};
use kivact::geometry::{Point, Rect};
use kivact::interaction::Tone;
use kivact::interaction::{
    ListItemState, MenuItemState, PopupState, TabListState, TabState, ToastState,
};
use kivact::style::{
    Align, Color, ColorToken, Cursor, Direction, Justify, Position, Style, Theme, pct, style,
};
use kivact::ui::Look;
use kivact::ui::{
    AxisState, ButtonState, ButtonVariant, CheckableState, ControlSize, DialogState,
    TextInputState, ToggleButtonState, UiRecipes,
};
use kivact::view::CheckedState;
use kivact::view::Orientation;
use std::sync::OnceLock;

/// A Game look of a standard button beyond its [`ButtonVariant`]:
/// `kivact::ui::button(..).variant(ButtonTone::Dark)`. Another theme
/// draws the button's plain variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonTone {
    /// The navy console button.
    Dark,
}

impl kivact::ui::Variant for ButtonTone {}

/// Draw a standard button slanted, the console's parallelogram:
/// `kivact::ui::button(..).variant(Slant)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Slant;

impl kivact::ui::Variant for Slant {}

/// The skew of a slanted button's root.
fn skew(slant: bool) -> f64 {
    if slant { -12.0 } else { 0.0 }
}

/// The Game recipe table, built once and shared, so two themes of the kit
/// compare their recipes by identity and installing an unchanged theme
/// never restyles.
pub(crate) fn recipes() -> UiRecipes {
    static TABLE: OnceLock<UiRecipes> = OnceLock::new();
    TABLE.get_or_init(build).clone()
}

fn build() -> UiRecipes {
    UiRecipes::default()
        .field(crate::recipes_field::field)
        .skeleton(|_| crate::recipes_feedback::skeleton)
        .empty_state(crate::recipes_feedback::empty_state)
        .badge(|_| crate::recipes_indicators::badge)
        .badge_label(|_| crate::recipes_indicators::badge_label)
        .avatar(|_| crate::recipes_indicators::avatar)
        .avatar_status(|_| crate::recipes_indicators::avatar_status)
        .avatar_group(|base| move |t, s| base(t, s).align_items(Align::Center))
        .avatar_group_overflow(|_| crate::recipes_indicators::avatar_overflow)
        .tree(crate::recipes_collections::tree)
        .table(crate::recipes_collections::table)
        .item_list(crate::recipes_collections::item_list)
        .dropzone(crate::recipes_files::dropzone)
        .toolbar(crate::recipes_groups::toolbar)
        .pagination(crate::recipes_navigation::pagination)
        .breadcrumb(crate::recipes_navigation::breadcrumb)
        .stepper(crate::recipes_navigation::stepper)
        .navigation_menu(crate::recipes_navigation::navigation_menu)
        .button_group(crate::recipes_groups::button_group)
        .toggle_group(crate::recipes_groups::toggle_group)
        .input_group(crate::recipes_groups::input_group)
        .button(|_| button)
        .button_content(|_| button_content)
        .button_busy(|_| button_busy)
        .toggle(|_| toggle)
        .checkbox(|_| checkbox)
        .switch(|_| switch)
        .text_input(|_| text_input)
        .text_area(|_| text_area)
        .select(|select| {
            select
                .trigger(|_| select_trigger)
                .surface(|_| select_surface)
                .option(|_| select_option)
                .check(|_| select_check)
                .indicator(|_| select_indicator)
        })
        .menu(|menu| {
            menu.trigger(|_| menu_trigger)
                .surface(|_| menu_surface)
                .item(|_| menu_item)
                .shortcut(|_| menu_shortcut)
                .separator(|_| menu_separator)
        })
        .dialog(|dialog| {
            dialog
                .surface(|_| dialog_surface)
                .title(|_| dialog_title)
                .description(|_| dialog_description)
                .header(|_| dialog_header)
                .body(|_| dialog_body)
                .footer(|_| dialog_footer)
                .scrim(|_| dialog_scrim)
                .close(|_| dialog_close)
        })
        .tooltip(|tooltip| tooltip.surface(|_| tooltip_surface))
        .tabs(|tabs| tabs.list(|_| tabs_list).tab(|_| tab))
        .radio_group(|radio| {
            radio
                .group(|_| radio_group)
                .item(|_| radio_item)
                .indicator(|_| radio_indicator)
                .description(|_| radio_description)
        })
        .slider(|slider| {
            slider
                .track(|_| slider_track)
                .range(|_| slider_range)
                .thumb(|_| slider_thumb)
        })
        .toast(|toast| {
            toast
                .region(|_| toast_region)
                .toast(|_| toast_card)
                .title(|_| toast_title)
                .body(|_| toast_body)
                .icon(|_| toast_icon)
                .close(|_| toast_close)
        })
        .splitter(|splitter| splitter.handle(|_| splitter_handle))
        .list(crate::recipes_collections::list)
        .data_table(crate::recipes_collections::data_table)
        .progress(crate::recipes_small::progress)
        .disclosure(crate::recipes_small::disclosure)
        .command_palette(crate::recipes_palette::command_palette)
        .tag_input(crate::recipes_input::tag_input)
        .number_input(crate::recipes_input::number_input)
}

/// A toggle: the secondary button, in the accent fill while pressed.
fn toggle(t: &Theme, look: &Look<ToggleButtonState>) -> Style {
    let s = button_style(t, ButtonVariant::Secondary, None, look.size, false, false);
    if look.pressed {
        s.background(t.colors().accent)
            .color(t.colors().accent_text)
    } else {
        s
    }
}

fn button(t: &Theme, state: &Look<ButtonState>) -> Style {
    button_style(
        t,
        state
            .variant::<ButtonVariant>()
            .copied()
            .unwrap_or_default(),
        state.variant::<ButtonTone>().copied(),
        state.size,
        state.busy,
        state.variant::<Slant>().is_some(),
    )
}
/// The look of a button of `variant` (or of the kit's `tone`) and `size`,
/// slanted or not, busy or not: what a segment of the kit's radio group
/// draws too.
pub(crate) fn button_style(
    t: &Theme,
    variant: ButtonVariant,
    tone: Option<ButtonTone>,
    size: ControlSize,
    loading: bool,
    slant: bool,
) -> Style {
    let p = t.ext::<Tokens>();
    let skew = skew(slant);
    let (bg, fg, hover, shadow) = match (tone, variant) {
        (Some(ButtonTone::Dark), _) => (
            p.navy,
            t.colors().accent_text,
            p.sidebar_alt,
            kivact::style::rgb(0x000000),
        ),
        (None, ButtonVariant::Primary) => (
            t.colors().accent,
            t.colors().accent_text,
            p.primary_hover,
            p.primary_strong,
        ),
        (None, ButtonVariant::Secondary | ButtonVariant::Outline) => (
            t.colors().surface,
            t.colors().accent,
            p.primary_weak,
            p.primary_weak,
        ),
        (None, ButtonVariant::Ghost | ButtonVariant::Link) => (
            kivact::style::Color::TRANSPARENT,
            p.text_2,
            p.primary_weak,
            kivact::style::Color::TRANSPARENT,
        ),
        (None, ButtonVariant::Danger) => (
            t.colors().danger,
            t.colors().accent_text,
            t.colors().danger,
            t.colors().danger,
        ),
    };
    let mut s = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(bg)
        .color(fg)
        .font_size(13)
        .weight(700)
        .text_wrap(false)
        .transform(Transform::new().skew_x(skew))
        .shadow(Shadow::new(0, 2, 0, shadow))
        .hover(style().background(hover))
        .pressed(style().transform(Transform::new().skew_x(skew).translate_y(2)));
    if matches!(variant, ButtonVariant::Secondary | ButtonVariant::Outline) {
        s = s.border(1.5, t.colors().accent);
    }

    let h = p.control_height;
    s = s.merge(&match size {
        ControlSize::ExtraSmall => style().height(h - 12.).padding_x(8).font_size(11),
        ControlSize::Small => style().height(h - 6.).padding_x(12).font_size(12),
        ControlSize::Medium => style().height(h).padding_x(18).font_size(13),
        ControlSize::Large => style().height(h + 8.).padding_x(28).font_size(14),
    });
    if loading {
        s = s.disabled(style().opacity(1.));
    }
    if variant == ButtonVariant::Primary {
        s = s.paint(|d, s| {
            d.fill_rect(
                Rect::new(s.width - 10., 5., 5., 5.),
                d.theme().ext::<Tokens>().yellow,
            )
        });
    }
    s
}
fn button_content(_: &Theme, state: &Look<ButtonState>) -> Style {
    style()
        .direction(Direction::Row)
        .gap(8)
        .align_items(Align::Center)
        .opacity(if state.busy { 0. } else { 1. })
        .transform(Transform::new().skew_x(-skew(state.variant::<Slant>().is_some())))
}
/// The arc the button turns while busy.
fn button_busy(_: &Theme, state: &Look<ButtonState>) -> Style {
    let variant = state
        .variant::<ButtonVariant>()
        .copied()
        .unwrap_or_default();
    style()
        .position(Position::Absolute)
        .left(pct(50) - 7)
        .top(pct(50) - 7)
        .size(14, 14)
        .paint(move |d, _| {
            let color = match variant {
                ButtonVariant::Secondary | ButtonVariant::Outline => d.theme().colors().accent,
                ButtonVariant::Ghost | ButtonVariant::Link => d.theme().ext::<Tokens>().text_2,
                _ => d.theme().colors().accent_text,
            };
            d.stroke_path(
                &Path::new().arc(Point::new(7., 7.), 6., 0., 270., false),
                &Stroke::new(2.),
                color,
            );
        })
}
fn text_input(t: &Theme, _: &Look<TextInputState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .height(p.control_height)
        .min_height(p.control_height)
        .padding_x(12)
        .padding_y(0)
        .radius(0)
        .border(1, p.line_strong)
        .background(t.colors().surface)
        .color(t.colors().text)
        .hover(style().border_color(p.text_3))
        .focus(
            style()
                .border_color(t.colors().accent)
                .ring(3, p.primary_weak),
        )
        .disabled(style().background(p.surface_3).color(p.text_3).opacity(1.))
}
fn text_area(t: &Theme, _: &Look<TextInputState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .min_height(88)
        .padding_x(12)
        .padding_y(8)
        .radius(0)
        .line_height(1.6)
        .border(1, p.line_strong)
        .focus(
            style()
                .border_color(t.colors().accent)
                .ring(3, p.primary_weak),
        )
}
fn checkbox(_: &Theme, state: &Look<CheckableState>) -> Style {
    let state = **state;
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .min_height(24)
        .padding_left(25)
        .paint(move |d, size| {
            d.layer(
                kivact::draw::Layer::new()
                    .transform(Transform::new().translate_y((size.height - 16.) / 2.)),
                |d| {
                    let c = state.checked.is_checked();
                    let mixed = state.checked == CheckedState::Indeterminate;
                    let p = d.theme().ext::<Tokens>();
                    let primary = d.theme().colors().accent;

                    d.fill_rect(
                        Rect::new(0., 0., 16., 16.),
                        if c || mixed {
                            primary
                        } else {
                            d.theme().colors().surface
                        },
                    );
                    d.stroke_rect(
                        Rect::new(0.75, 0.75, 14.5, 14.5),
                        1.5,
                        if c || mixed { primary } else { p.line_strong },
                    );
                    if mixed {
                        d.line(
                            Point::new(4., 8.),
                            Point::new(12., 8.),
                            2.,
                            ColorToken::AccentText,
                        );
                    } else if c {
                        d.stroke_path(
                            &Path::new()
                                .move_to(Point::new(3., 8.))
                                .line_to(Point::new(6., 11.))
                                .line_to(Point::new(13., 4.)),
                            &Stroke::new(2.).cap(LineCap::Round).join(LineJoin::Round),
                            ColorToken::AccentText,
                        );
                    }
                },
            );
        })
}
fn switch(_: &Theme, state: &Look<CheckableState>) -> Style {
    let state = **state;
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .min_height(24)
        .padding_left(49)
        .paint(move |d, size| {
            d.layer(
                kivact::draw::Layer::new()
                    .transform(Transform::new().translate_y((size.height - 20.) / 2.)),
                |d| {
                    let c = state.checked.is_checked();
                    let p = d.theme().ext::<Tokens>();
                    let primary = d.theme().colors().accent;

                    d.fill_rect(
                        Rect::new(0., 0., 40., 20.),
                        if c { primary } else { p.line_strong },
                    );
                    d.fill_rect(
                        Rect::new(if c { 23. } else { 3. }, 3., 14., 14.),
                        kivact::style::rgb(0xffffff),
                    );
                    d.text(
                        &state.text.map(|text| text.get()).unwrap_or_default(),
                        Point::new(if c { 5. } else { 20. }, 6.),
                        TextFormat {
                            size: 7.5,
                            weight: 700,
                            color: kivact::style::rgb(0xffffff),
                            ..Default::default()
                        },
                    );
                },
            );
        })
}

fn select_trigger(t: &Theme, s: &Look<kivact::ui::SelectTriggerState>) -> Style {
    // The heights of the buttons of the same size.
    let h = t.ext::<Tokens>().control_height;
    trigger(t).height(match s.size {
        ControlSize::ExtraSmall => h - 12.,
        ControlSize::Small => h - 6.,
        ControlSize::Medium => h,
        ControlSize::Large => h + 8.,
    })
}
fn menu_trigger(t: &Theme, _: &Look<PopupState>) -> Style {
    trigger(t)
}
fn trigger(t: &Theme) -> Style {
    style()
        .direction(Direction::Row)
        .height(t.ext::<Tokens>().control_height)
        .padding_x(12)
        .gap(8)
        .align_items(Align::Center)
        .background(t.colors().surface)
        .border(1, t.ext::<Tokens>().line_strong)
        .focus(
            style()
                .border_color(t.colors().accent)
                .ring(3, t.ext::<Tokens>().primary_weak),
        )
}
fn select_surface(_: &Theme, _: &Look<PopupState>) -> Style {
    style()
        .padding_y(4)
        .min_width(220)
        .max_height(248)
        .background(ColorToken::Surface)
        .border(1, ColorToken::Border)
}
fn select_option(_: &Theme, _: &Look<ListItemState>) -> Style {
    style()
        .direction(Direction::Row)
        .padding_left(12)
        .padding_right(26)
        .height(32)
        .align_items(Align::Center)
        .hover(style().background(ColorToken::SurfaceAlt))
        .selected(
            style()
                .background(ColorToken::Selection)
                .color(ColorToken::Accent),
        )
        .focus(
            style()
                .ring(2, ColorToken::Focus)
                .background(ColorToken::Accent)
                .color(ColorToken::AccentText),
        )
}
/// The `check` of the selected option, in the row's color.
fn select_check(_: &Theme, _: &Look<ListItemState>) -> Style {
    style()
        .position(Position::Absolute)
        .right(10)
        .top(pct(50) - 8)
        .size(16, 16)
}
/// The box of the trigger's `chevron-down`, turned over while open.
fn select_indicator(_: &Theme, state: &Look<PopupState>) -> Style {
    let s = style().size(16, 16).shrink(0).color(ColorToken::Text);
    if state.open { s.rotate(180) } else { s }
}

fn menu_surface(t: &Theme, _: &Look<PopupState>) -> Style {
    style()
        .direction(Direction::Column)
        .min_width(220)
        .padding(4)
        .background(t.colors().surface)
        .border(1, t.ext::<Tokens>().line_strong)
        .shadow(Shadow::new(0, 14, 36, t.colors().shadow))
}
fn menu_item(t: &Theme, _: &Look<MenuItemState>) -> Style {
    let hot = style()
        .background(t.colors().accent)
        .color(t.colors().accent_text);
    style()
        .direction(Direction::Row)
        .height(32)
        .padding_x(10)
        .gap(10)
        .align_items(Align::Center)
        .color(t.colors().text)
        .hover(hot.clone())
        .focus(hot)
}
fn menu_shortcut(t: &Theme, _: &Look<MenuItemState>) -> Style {
    style()
        .font_family(t.ext::<Tokens>().mono_family.as_str())
        .font_size(10.5)
        .weight(600)
        .color(ColorToken::TextMuted)
}
fn menu_separator(_: &Theme, _: &Look<kivact::custom::PartState>) -> Style {
    style().height(1).margin_y(4).background(ColorToken::Border)
}
fn dialog_surface(t: &Theme, s: &Look<DialogState>) -> Style {
    use kivact::geometry::Edge;
    use kivact::interaction::DialogMode::*;
    use kivact::motion::{Easing, Transition};
    use kivact::style::TransitionProp;
    let ease = Easing::CubicBezier(0.2, 0.8, 0.2, 1.);
    match s.kind {
        // A drawer slides in from its edge.
        Drawer(side) => style()
            .width(420)
            .max_width(pct(100))
            .background(t.colors().surface)
            .border_left(1, t.ext::<Tokens>().line_strong)
            .enter(style().opacity(0.).translate(
                match side {
                    Edge::Left => -40.,
                    Edge::Right => 40.,
                    _ => 0.,
                },
                match side {
                    Edge::Top => -40.,
                    Edge::Bottom => 40.,
                    _ => 0.,
                },
            ))
            .transition(TransitionProp::Paint, Transition::tween(240, ease)),
        Popover => style()
            .width(280)
            .padding(14)
            .gap(8)
            .background(ColorToken::Surface)
            .border(1, ColorToken::Border),
        // A modal rises a little as it fades in.
        _ => style()
            .width(520)
            .max_width(pct(100) - 32)
            .background(t.colors().surface)
            .border(1, t.ext::<Tokens>().line_strong)
            .shadow(Shadow::new(0, 14, 36, t.colors().shadow))
            .enter(style().opacity(0.).scale(0.985).translate(0, 10))
            .transition(TransitionProp::Paint, Transition::tween(220, ease)),
    }
}
fn dialog_header(_: &Theme, s: &Look<DialogState>) -> Style {
    if s.kind == kivact::interaction::DialogMode::Popover {
        style().align_items(Align::Center)
    } else {
        // The title over the description, clear of the 32 px close button.
        style()
            .padding_left(20)
            .padding_right(56)
            .padding_top(14)
            .padding_bottom(12)
            .gap(8)
            .border_top(4, ColorToken::Accent)
            .border_bottom(1, ColorToken::Border)
    }
}
fn dialog_title(t: &Theme, s: &Look<DialogState>) -> Style {
    if s.kind == kivact::interaction::DialogMode::Popover {
        style().weight(700)
    } else {
        style()
            .font_family(t.ext::<Tokens>().display_family.as_str())
            .font_size(22)
            .weight(700)
            .italic(true)
            .line_height(1.)
    }
}
fn dialog_description(_: &Theme, _: &Look<DialogState>) -> Style {
    style().font_size(13).color(ColorToken::TextMuted)
}
fn dialog_body(_: &Theme, s: &Look<DialogState>) -> Style {
    if s.kind == kivact::interaction::DialogMode::Popover {
        style().gap(6)
    } else {
        style().padding_x(20).padding_y(18)
    }
}
fn dialog_footer(_: &Theme, _: &Look<DialogState>) -> Style {
    style()
        .shrink(0)
        .padding_x(18)
        .padding_y(14)
        .gap(10)
        .justify_content(Justify::End)
        .align_items(Align::Center)
        .background(ColorToken::SurfaceAlt)
        .border_top(1, ColorToken::Border)
}
fn dialog_scrim(_: &Theme, _: &Look<DialogState>) -> Style {
    style().background(ColorToken::Overlay).backdrop_blur(2)
}
/// The close button around the theme's `x` icon.
fn dialog_close(_: &Theme, _: &Look<DialogState>) -> Style {
    style()
        .size(32, 32)
        .position(Position::Absolute)
        .top(16)
        .right(12)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(ColorToken::TextMuted)
        .hover(
            style()
                .background(ColorToken::SurfaceAlt)
                .border(1, ColorToken::Border)
                .color(ColorToken::Text),
        )
}
fn tooltip_surface(t: &Theme, _: &Look<PopupState>) -> Style {
    style()
        .padding_x(9)
        .padding_y(5)
        .max_width(320)
        .background(t.ext::<Tokens>().navy)
        .color(kivact::style::rgb(0xffffff))
        .border_bottom(2, t.ext::<Tokens>().cyan)
        .font_size(11)
}
fn tabs_list(_: &Theme, state: &Look<TabListState>) -> Style {
    if state.variant::<Boxed>().is_some() {
        return choice_style::boxed_list();
    }
    if state.variant::<Navigation>().is_some() {
        return choice_style::navigation_list();
    }
    style()
        .direction(Direction::Row)
        .width(pct(100))
        .padding(0)
        .background(kivact::style::Color::TRANSPARENT)
        .border(0, ColorToken::Border)
        .border_bottom(1, ColorToken::Border)
        .gap(8)
}
fn tab(_: &Theme, state: &Look<TabState>) -> Style {
    if state.variant::<Boxed>().is_some() {
        return choice_style::boxed_tab();
    }
    if state.variant::<Navigation>().is_some() {
        return choice_style::navigation_item();
    }
    style()
        .direction(Direction::Row)
        .gap(7)
        .height(38)
        .padding_x(16)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .border_bottom(3, kivact::style::Color::TRANSPARENT)
        .color(ColorToken::TextMuted)
        .selected(
            style()
                .border_color(ColorToken::Accent)
                .color(ColorToken::Accent)
                .weight(700),
        )
        .hover(style().background(ColorToken::Selection))
}

/// The outline of a diamond filling the `side` square at `(x, y)`, inset
/// by `inset`.
fn diamond(x: f64, y: f64, side: f64, inset: f64) -> Path {
    let (lo, mid, hi) = (inset, side / 2., side - inset);
    Path::new()
        .move_to(Point::new(x + mid, y + lo))
        .line_to(Point::new(x + hi, y + mid))
        .line_to(Point::new(x + mid, y + hi))
        .line_to(Point::new(x + lo, y + mid))
        .close()
}

fn radio_group(_: &Theme, orientation: &Look<Orientation>) -> Style {
    if orientation.variant::<kivact::ui::RadioGroupVariant>()
        == Some(&kivact::ui::RadioGroupVariant::Segmented)
    {
        return choice_style::segment_group();
    }
    if orientation.variant::<ChoiceCard>().is_some() {
        return choice_style::card_group();
    }
    let s = style().gap(10);
    match **orientation {
        Orientation::Vertical => s.direction(Direction::Column),
        _ => s.direction(Direction::Row).wrap(true),
    }
}
fn radio_item(t: &Theme, state: &Look<CheckableState>) -> Style {
    if state.variant::<kivact::ui::RadioGroupVariant>()
        == Some(&kivact::ui::RadioGroupVariant::Segmented)
    {
        // A small slanted secondary button, filled while chosen.
        return button_style(
            t,
            ButtonVariant::Secondary,
            None,
            ControlSize::Small,
            false,
            false,
        )
        .clip_path(
            kivact::draw::Shape::polygon()
                .point(8, 0)
                .point(pct(100), 0)
                .point(pct(100) - 8, pct(100))
                .point(0, pct(100)),
        )
        .selected(choice_style::segment_chosen());
    }
    if state.variant::<ChoiceCard>().is_some() {
        return choice_style::card_item(state.checked.is_checked());
    }
    style()
        .direction(Direction::Row)
        .gap(9)
        .align_items(Align::Center)
        .min_height(24)
        .color(ColorToken::Text)
}
/// The diamond around the theme's `dot`, which is a diamond too.
fn radio_indicator(_: &Theme, state: &Look<CheckableState>) -> Style {
    if state.variant::<kivact::ui::RadioGroupVariant>()
        == Some(&kivact::ui::RadioGroupVariant::Segmented)
        || state.variant::<ChoiceCard>().is_some()
    {
        return choice_style::hidden_indicator();
    }
    let checked = state.checked.is_checked();
    style()
        .size(16, 16)
        .shrink(0)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(ColorToken::Accent)
        .paint(move |d, size| {
            let color = if checked {
                d.theme().colors().accent
            } else {
                d.theme().ext::<Tokens>().line_strong
            };
            d.stroke_path(
                &diamond(0., 0., size.width.min(size.height), 1.),
                &Stroke::new(1.5).cap(LineCap::Round).join(LineJoin::Round),
                color,
            );
        })
}
fn radio_description(t: &Theme, _: &Look<CheckableState>) -> Style {
    style().font_size(11.5).color(t.ext::<Tokens>().text_3)
}

/// The thumb's 14 px run along the track and 18 px across it.
const THUMB: (f64, f64) = (14., 18.);
/// A 4 px bar across the middle of a box, along its longer side, leaving
/// `inset` at both ends.
fn bar(size: kivact::geometry::Size, inset: f64) -> Rect {
    if size.width >= size.height {
        Rect::new(
            inset,
            (size.height - 4.) / 2.,
            (size.width - 2. * inset).max(0.),
            4.,
        )
    } else {
        Rect::new(
            (size.width - 4.) / 2.,
            inset,
            4.,
            (size.height - 2. * inset).max(0.),
        )
    }
}
/// A 28 px track holding the bar between the thumb's centres at the ends.
fn slider_track(_: &Theme, state: &Look<AxisState>) -> Style {
    let s = style().shrink(0).cursor(Cursor::Pointer).paint(|d, size| {
        d.fill_rect(bar(size, THUMB.0 / 2.), d.theme().ext::<Tokens>().surface_3);
    });
    match state.orientation {
        Orientation::Vertical => s.width(28).height(160),
        _ => s.height(28).width(pct(100)).min_width(80),
    }
}
fn slider_range(_: &Theme, _: &Look<AxisState>) -> Style {
    style().paint(|d, size| d.fill_rect(bar(size, 0.), ColorToken::Accent))
}
/// The console's parallelogram thumb on the surface, outlined in the accent.
fn slider_thumb(_: &Theme, state: &Look<AxisState>) -> Style {
    let vertical = state.orientation == Orientation::Vertical;
    let (along, across) = THUMB;
    let s = if vertical {
        style().size(across, along)
    } else {
        style().size(along, across)
    };
    s.paint(move |d, size| {
        let (w, h) = (size.width, size.height);
        let thumb = if vertical {
            Path::new()
                .move_to(Point::new(0., 4.))
                .line_to(Point::new(0., h))
                .line_to(Point::new(w, h - 4.))
                .line_to(Point::new(w, 0.))
                .close()
        } else {
            Path::new()
                .move_to(Point::new(4., 0.))
                .line_to(Point::new(w, 0.))
                .line_to(Point::new(w - 4., h))
                .line_to(Point::new(0., h))
                .close()
        };
        d.fill_path(&thumb.clone(), d.theme().colors().surface);
        d.stroke_path(
            &thumb,
            &Stroke::new(1.5).cap(LineCap::Round).join(LineJoin::Round),
            ColorToken::Accent,
        );
    })
}

/// The color of a toast's tone.
fn tone_color(t: &Theme, tone: Tone) -> Color {
    match tone {
        Tone::Neutral | Tone::Info => t.colors().accent,
        Tone::Success => t.colors().success,
        Tone::Warning => t.colors().warning,
        Tone::Danger => t.colors().danger,
        Tone::Accent => t.ext::<Tokens>().yellow,
    }
}
/// A stack at the window's bottom right, above the page.
fn toast_region(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .position(Position::Fixed)
        .z_index(100)
        .right(16)
        .bottom(16)
        .max_width(pct(100) - 16)
        .align_items(Align::End)
        .gap(8)
}
/// A navy card with its tone's edge and a diamond of its tone before the
/// text. It slides in from the right and back out when it is dismissed.
fn toast_card(t: &Theme, state: &Look<ToastState>) -> Style {
    use kivact::motion::{Easing, Transition};
    use kivact::style::TransitionProp;
    let tone = state.tone;
    style()
        .enter(style().translate_x(40))
        .exit(style().translate_x(40).opacity(0.))
        .transition(
            TransitionProp::Paint,
            Transition::tween(260, Easing::EaseOut),
        )
        .direction(Direction::Row)
        .width(360)
        .padding_left(14)
        .padding_right(12)
        .padding_top(12)
        .padding_bottom(14)
        .gap(12)
        .align_items(Align::Start)
        .background(t.ext::<Tokens>().navy)
        .color(kivact::style::rgb(0xffffff))
        .border_left(4, tone_color(t, tone))
}
/// The diamond of the tone, in the tone's color.
fn toast_icon(t: &Theme, state: &Look<ToastState>) -> Style {
    style()
        .size(18, 18)
        .shrink(0)
        .margin_top(1)
        .color(tone_color(t, state.tone))
}
fn toast_title(_: &Theme, _: &Look<ToastState>) -> Style {
    style()
        .weight(700)
        .font_size(13)
        .color(kivact::style::rgb(0xffffff))
}
fn toast_body(_: &Theme, _: &Look<ToastState>) -> Style {
    style().font_size(12).color(kivact::style::rgb(0xb8c9dc))
}
/// The close button around the kit's `x`.
fn toast_close(_: &Theme, _: &Look<ToastState>) -> Style {
    style()
        .size(22, 22)
        .shrink(0)
        .padding(3)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(kivact::style::rgb(0xb8c9dc))
        .hover(style().color(kivact::style::rgb(0xffffff)))
}

/// A 5 px bar between the panes that lights up under the pointer and while
/// dragged.
fn splitter_handle(_: &Theme, state: &Look<AxisState>) -> Style {
    let s = style()
        .shrink(0)
        .align_self(Align::Stretch)
        .background(ColorToken::Border)
        .hover(style().background(ColorToken::Accent))
        .pressed(style().background(ColorToken::Accent));
    match state.orientation {
        Orientation::Vertical => s.height(5).cursor(Cursor::NsResize),
        _ => s.width(5).cursor(Cursor::EwResize),
    }
}
