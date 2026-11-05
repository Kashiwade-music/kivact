//! Role: Install Game recipes and draw the base form controls.
use crate::Tokens;
use kivact::draw::{LineCap, LineJoin, Path, Shadow, Stroke, TextFormat, Transform};
use kivact::geometry::{Point, Rect};
use kivact::style::{Align, ColorToken, Direction, Justify, Position, Style, Theme, pct, style};
use kivact::ui::{
    ButtonState, ButtonVariant, CheckableState, ControlSize, Look, TextInputState,
    ToggleButtonState, UiRecipes,
};
use kivact::view::CheckedState;

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
    #[expect(
        clippy::disallowed_types,
        reason = "Immutable static recipe identity is shared by all Game themes."
    )]
    static TABLE: std::sync::OnceLock<UiRecipes> = std::sync::OnceLock::new();
    TABLE.get_or_init(build).clone()
}

fn build() -> UiRecipes {
    install_controls(
        UiRecipes::default()
            .field(crate::recipes_field::field)
            .heading(|_| crate::recipes_metrics::heading)
            .stat(crate::recipes_metrics::stat)
            .description_list(crate::recipes_metrics::description_list)
            .skeleton(|_| crate::recipes_indicators::skeleton)
            .empty_state(crate::recipes_feedback::empty_state)
            .alert(crate::recipes_feedback::alert)
            .badge(|_| crate::recipes_badge::badge)
            .badge_label(|_| crate::recipes_badge::badge_label)
            .avatar(|_| crate::recipes_avatar::avatar)
            .avatar_status(|_| crate::recipes_avatar::avatar_status)
            .avatar_group(|base| move |t, s| base(t, s).align_items(Align::Center))
            .avatar_group_overflow(|_| crate::recipes_avatar::avatar_overflow)
            .tree(crate::recipes_items::tree)
            .table(crate::recipes_items::table)
            .item_list(crate::recipes_items::item_list)
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
            .text_area(|_| text_area),
    )
}

fn install_controls(recipes: UiRecipes) -> UiRecipes {
    recipes
        .select(|select| {
            select
                .trigger(|_| crate::recipes_extra::select_trigger)
                .surface(|_| crate::recipes_extra::select_surface)
                .option(|_| crate::recipes_extra::select_option)
                .check(|_| crate::recipes_extra::select_check)
                .indicator(|_| crate::recipes_extra::select_indicator)
        })
        .menu(|menu| {
            menu.trigger(|_| crate::recipes_extra::menu_trigger)
                .surface(|_| crate::recipes_extra::menu_surface)
                .item(|_| crate::recipes_extra::menu_item)
                .shortcut(|_| crate::recipes_extra::menu_shortcut)
                .separator(|_| crate::recipes_extra::menu_separator)
        })
        .dialog(|dialog| {
            dialog
                .surface(|_| crate::recipes_extra::dialog_surface)
                .title(|_| crate::recipes_extra::dialog_title)
                .description(|_| crate::recipes_extra::dialog_description)
                .header(|_| crate::recipes_extra::dialog_header)
                .body(|_| crate::recipes_extra::dialog_body)
                .footer(|_| crate::recipes_extra::dialog_footer)
                .scrim(|_| crate::recipes_extra::dialog_scrim)
                .close(|_| crate::recipes_extra::dialog_close)
        })
        .tooltip(|tooltip| tooltip.surface(|_| crate::recipes_extra::tooltip_surface))
        .tabs(|tabs| {
            tabs.list(|_| crate::recipes_extra::tabs_list)
                .tab(|_| crate::recipes_extra::tab)
        })
        .radio_group(|radio| {
            radio
                .group(|_| crate::recipes_extra::radio_group)
                .item(|_| crate::recipes_extra::radio_item)
                .indicator(|_| crate::recipes_extra::radio_indicator)
                .description(|_| crate::recipes_extra::radio_description)
        })
        .slider(|slider| {
            slider
                .track(|_| crate::recipes_extra::slider_track)
                .range(|_| crate::recipes_extra::slider_range)
                .thumb(|_| crate::recipes_extra::slider_thumb)
        })
        .toast(|toast| {
            toast
                .region(|_| crate::recipes_extra::toast_region)
                .toast(|_| crate::recipes_extra::toast_card)
                .title(|_| crate::recipes_extra::toast_title)
                .body(|_| crate::recipes_extra::toast_body)
                .icon(|_| crate::recipes_extra::toast_icon)
                .close(|_| crate::recipes_extra::toast_close)
        })
        .splitter(|splitter| splitter.handle(|_| crate::recipes_extra::splitter_handle))
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
        (Some(ButtonTone::Dark), _) => {
            (p.navy, t.colors().accent_text, p.sidebar_alt, p.dark_shadow)
        }
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
                    d.fill_rect(Rect::new(if c { 23. } else { 3. }, 3., 14., 14.), p.on_dark);
                    d.text(
                        &state.text.map(|text| text.get()).unwrap_or_default(),
                        Point::new(if c { 5. } else { 20. }, 6.),
                        TextFormat {
                            size: 7.5,
                            weight: 700,
                            color: p.on_dark,
                            ..Default::default()
                        },
                    );
                },
            );
        })
}

/// The outline of a diamond filling the `side` square at `(x, y)`, inset
/// by `inset`.
pub(crate) fn diamond(x: f64, y: f64, side: f64, inset: f64) -> Path {
    let (lo, mid, hi) = (inset, side / 2., side - inset);
    Path::new()
        .move_to(Point::new(x + mid, y + lo))
        .line_to(Point::new(x + hi, y + mid))
        .line_to(Point::new(x + mid, y + hi))
        .line_to(Point::new(x + lo, y + mid))
        .close()
}
