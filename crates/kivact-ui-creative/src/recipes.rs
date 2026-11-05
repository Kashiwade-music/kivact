//! Role: Creative appearance for the shared standard UI components.
//!
//! `radio_group`, `slider`, `toast_region` and `splitter` of `kivact::ui`
//! take the kit's look from `recipes_controls`.
use crate::ColorName;
use kivact::custom::PartState;
use kivact::draw::{LineCap, LineJoin, Path, Shadow, Stroke};
use kivact::geometry::{Point, Rect};
use kivact::interaction::DialogMode;
use kivact::interaction::{ListItemState, MenuItemState, PopupState, TabListState, TabState};
use kivact::style::{Align, Direction, Justify, Position, Style, Theme, pct, style};
use kivact::ui::Look;
use kivact::ui::{
    ButtonState, ButtonVariant, CheckableState, ControlSize, DialogState, SelectTriggerState,
    TextInputState, ToggleButtonState, UiRecipes,
};
use kivact::view::CheckedState;

use crate::tokens::Tokens;

/// The kit's recipe table, built once and shared, so two themes of the kit
/// compare their recipes by identity and installing the kit's theme again
/// never restyles.
pub(crate) fn recipes() -> UiRecipes {
    #[expect(
        clippy::disallowed_types,
        reason = "Process-wide immutable recipe table, initialized once so theme equality retains recipe identity."
    )]
    static RECIPES: std::sync::OnceLock<UiRecipes> = std::sync::OnceLock::new();
    RECIPES.get_or_init(build).clone()
}

fn build() -> UiRecipes {
    let recipes = UiRecipes::default()
        .button(|_| button)
        .button_content(|_| {
            |_: &Theme, _: &Look<ButtonState>| {
                style()
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .gap(6)
            }
        })
        .toggle(|_| toggle_button)
        .checkbox(|_| |t: &Theme, s: &Look<CheckableState>| toggle(t, s, false))
        .switch(|_| |t: &Theme, s: &Look<CheckableState>| toggle(t, s, true))
        .text_input(|_| {
            |t: &Theme, _: &Look<TextInputState>| {
                crate::control_style::field_style(t)
                    .padding_y(0)
                    .min_height(t.ext::<Tokens>().row)
            }
        })
        .text_area(|_| {
            |t: &Theme, _: &Look<TextInputState>| {
                crate::control_style::field_style(t)
                    .height(100)
                    .min_height(t.ext::<Tokens>().row)
            }
        })
        .select(|select| {
            select
                .trigger(|_| select_trigger)
                .surface(|_| {
                    |t: &Theme, _: &Look<PopupState>| dropdown_surface(t).max_height(214.5)
                })
                .option(|_| select_option)
                .check(|_| select_check)
                .indicator(|_| |_: &Theme, _: &Look<PopupState>| mark())
        });
    let recipes = overlays(recipes)
        .toggle_group(|group| {
            group.item(|_| {
                |t: &Theme, s: &Look<kivact::ui::ToggleGroupItemState>| {
                    toggle_button(t, &Look::new(s.toggle))
                }
            })
        })
        .radio_group(crate::recipes_controls::radio_group)
        .slider(crate::recipes_controls::slider)
        .toast(crate::recipes_controls::toast)
        .splitter(crate::recipes_controls::splitter)
        .alert(crate::recipes_feedback::alert)
        .empty_state(crate::recipes_feedback::empty_state)
        .card(crate::recipes_feedback::card)
        .input_group(crate::recipes_feedback::input_group)
        .table(crate::recipes_collections::table)
        .item_list(crate::recipes_collections::item_list)
        .list(crate::recipes_collections::list)
        .data_table(crate::recipes_collections::data_table)
        .progress(crate::recipes_controls::progress)
        .disclosure(crate::recipes_controls::disclosure);
    crate::recipes_metrics::install(crate::recipes_indicators::install(recipes))
}
fn select_trigger(t: &Theme, s: &Look<SelectTriggerState>) -> Style {
    // The heights of the buttons of the same size.
    let field = crate::control_style::field_style(t);
    match s.size {
        ControlSize::ExtraSmall => field.height(16).font_size(10),
        ControlSize::Small => field.height(20).font_size(11),
        ControlSize::Medium => field,
        ControlSize::Large => field.height(30).font_size(13),
    }
}
fn select_option(t: &Theme, _: &Look<ListItemState>) -> Style {
    dropdown_option(t).padding_right(45)
}
/// The `check` of the selected option, in the text color.
fn select_check(t: &Theme, _: &Look<ListItemState>) -> Style {
    style()
        .position(Position::Absolute)
        .right(5)
        .top(pct(50) - 8)
        .size(16, 16)
        .color(t.ext::<Tokens>().text)
}
fn menu_item(t: &Theme, _: &Look<MenuItemState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .direction(Direction::Row)
        .height(27)
        .gap(9)
        .padding_x(7)
        .radius(6)
        .align_items(Align::Center)
        .hover(style().background(kivact::style::ColorToken::Hover))
        .selected(style().background(p.selection))
}
fn dialog_surface(t: &Theme, s: &Look<DialogState>) -> Style {
    match s.kind {
        DialogMode::Drawer(_) => crate::feedback::glass_style(t)
            .padding(18)
            .gap(14)
            .width(400)
            .max_width(pct(100))
            .radius(0),
        _ => crate::feedback::glass_style(t)
            .padding(18)
            .gap(14)
            .min_width(320)
            .max_width(640),
    }
}
fn button(t: &Theme, s: &Look<ButtonState>) -> Style {
    let p = t.ext::<Tokens>();
    let base = crate::controls::button_style(t);
    let base = match s.variant::<ButtonVariant>().copied().unwrap_or_default() {
        ButtonVariant::Primary => base
            .background(p.accent)
            .color(p.accent_ink)
            .weight(600)
            .hover(style().background(p.accent_high).color(p.accent_ink)),
        ButtonVariant::Danger => base.hover(style().background(p.x.with_alpha(0.16)).color(p.x)),
        ButtonVariant::Ghost | ButtonVariant::Link => base
            .background(crate::color::transparent_ink())
            .border(0, p.hair)
            .color(p.muted)
            .hover(
                style()
                    .background(crate::color::menu_hover())
                    .color(p.text_high),
            ),
        _ => base,
    };
    let base = match s.size {
        ControlSize::ExtraSmall => base.height(16).font_size(10).radius(5).padding_x(6),
        ControlSize::Small => base.height(20).font_size(11).radius(6),
        ControlSize::Medium => base.height(p.row),
        ControlSize::Large => base.height(30).font_size(13).radius(9).padding_x(18),
    };
    // A square toolbar action: as wide as the button is high.
    if s.icon_only {
        base.aspect_ratio(1.0).padding(0)
    } else {
        base
    }
}
/// A tool toggle: the secondary button, in the selection colors while pressed.
fn toggle_button(t: &Theme, s: &Look<ToggleButtonState>) -> Style {
    let p = t.ext::<Tokens>();
    let mut state = ButtonState::new(s.input, s.size);
    state.leading_icon = s.leading_icon;
    state.icon_only = s.icon_only;
    let base = button(t, &Look::new(state).with_variant(ButtonVariant::Secondary));
    if s.pressed {
        base.background(p.selection)
            .color(p.accent_high)
            .border(1, p.accent.with_alpha(0.35))
    } else {
        base
    }
}
fn toggle(t: &Theme, s: &CheckableState, switch: bool) -> Style {
    let state = *s;
    crate::control_style::check_style(t)
        .padding_left(if switch { 37 } else { 21 })
        .paint(move |d, size| {
            let p = d.theme().ext::<Tokens>();
            let on = state.checked.is_checked();
            let mixed = state.checked == CheckedState::Indeterminate;
            let y = (size.height - 16.) / 2.;
            if switch {
                d.rounded_rect(
                    Rect::new(0., y, 30., 16.),
                    8.,
                    if on { p.accent } else { p.strong },
                );
                d.rounded_rect(
                    Rect::new(if on { 16. } else { 2. }, y + 2., 12., 12.),
                    6.,
                    if on { p.accent_ink } else { p.muted },
                );
            } else {
                d.rounded_rect(
                    Rect::new(0.5, y + 1.5, 13., 13.),
                    4.,
                    if on || mixed { p.accent } else { p.strong },
                );
                if mixed {
                    d.line(
                        Point::new(3.5, y + 8.),
                        Point::new(10.5, y + 8.),
                        1.7,
                        p.accent_ink,
                    );
                } else if on {
                    d.stroke_path(
                        &Path::new()
                            .move_to(Point::new(3., y + 8.))
                            .line_to(Point::new(6., y + 10.5))
                            .line_to(Point::new(11., y + 5.5)),
                        &Stroke::new(1.7).cap(LineCap::Round).join(LineJoin::Round),
                        p.accent_ink,
                    );
                }
            }
        })
}
/// The 16 px box of a mark icon (a check, a chevron), in the text color.
fn mark() -> Style {
    style().size(16, 16).shrink(0)
}
fn dropdown_surface(t: &Theme) -> Style {
    style()
        .padding(5)
        .min_width(160)
        .radius(12)
        .background(crate::color::overlay_surface())
        .backdrop_blur(22)
        .border(1, crate::color::overlay_border())
        .color(t.ext::<Tokens>().text)
        .shadow(Shadow::new(0., 10., 24., crate::color::menu_shadow()))
}
fn dropdown_option(t: &Theme) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .direction(Direction::Row)
        .height(27)
        .padding_x(9)
        .gap(20)
        .align_items(Align::Center)
        .radius(6)
        .hover(style().background(kivact::style::ColorToken::Hover))
        .selected(style().color(p.accent_high))
}

/// Install floating-surface and tab recipes after basic controls.
fn overlays(recipes: UiRecipes) -> UiRecipes {
    recipes
        .menu(|menu| {
            menu.trigger(|_| |t: &Theme, _: &Look<PopupState>| crate::controls::button_style(t))
                .surface(|_| {
                    |t: &Theme, _: &Look<PopupState>| {
                        crate::feedback::glass_style(t).direction(Direction::Column)
                    }
                })
                .item(|_| menu_item)
                .shortcut(|_| {
                    |t: &Theme, _: &Look<MenuItemState>| {
                        style()
                            .font_family(t.ext::<Tokens>().mono_font.as_str())
                            .font_size(10)
                    }
                })
                .separator(|_| {
                    |_: &Theme, _: &Look<PartState>| {
                        style().height(1).background(ColorName::Hair).margin_y(4)
                    }
                })
                .check(|_| |_: &Theme, _: &Look<MenuItemState>| mark())
                .submenu(|_| |_: &Theme, _: &Look<MenuItemState>| mark())
        })
        .dialog(|dialog| {
            dialog
                .surface(|_| dialog_surface)
                .scrim(|_| {
                    |_: &Theme, _: &Look<DialogState>| {
                        style().background(crate::color::modal_backdrop())
                    }
                })
                // The title over the description, clear of the close button.
                .header(|_| |_: &Theme, _: &Look<DialogState>| style().gap(4).padding_right(28))
                .title(|_| |_: &Theme, _: &Look<DialogState>| style().weight(600))
                .description(|_| |_: &Theme, _: &Look<DialogState>| style().color(ColorName::Muted))
                .body(|_| |_: &Theme, _: &Look<DialogState>| style().gap(8))
                .footer(|_| {
                    |_: &Theme, _: &Look<DialogState>| style().gap(8).justify_content(Justify::End)
                })
        })
        .tooltip(|tooltip| {
            tooltip.surface(|_| {
                |_: &Theme, _: &Look<PopupState>| {
                    style()
                        .padding(10)
                        .max_width(320)
                        .radius(8)
                        .color(ColorName::Text)
                        .background(crate::color::overlay_surface())
                        .backdrop_blur(22)
                        .border(1, ColorName::HairStrong)
                        .shadow(Shadow::new(0., 10., 24., crate::color::dialog_shadow()))
                }
            })
        })
        .tabs(|tabs| {
            tabs.list(|_| |t: &Theme, _: &Look<TabListState>| crate::layout::tab_track_style(t))
                .tab(|_| |t: &Theme, _: &Look<TabState>| crate::layout::tab_option_style(t, false))
                .panel(|_| |_: &Theme, _: &Look<TabState>| style())
        })
}
