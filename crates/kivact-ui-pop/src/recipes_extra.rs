//! Role: Pop recipes for compound standard controls and button decoration.
use crate::choice::{choice_list, choice_option, select_list, select_option};
use crate::tokens::Hue;
use crate::tokens::Tokens;
use kivact::custom::PartState;
use kivact::draw::{LineCap, LineJoin, Path, Shadow, Stroke, Transform};
use kivact::geometry::Point;
use kivact::interaction::{DialogMode, Tone};
use kivact::interaction::{
    ListItemState, MenuItemState, PopupState, TabListState, TabState, ToastState,
};
use kivact::style::{
    Align, Color, ColorToken, Cursor, Direction, Justify, Position, Style, Theme, pct, style,
};
use kivact::ui::Look;
use kivact::ui::RadioGroupVariant;
use kivact::ui::{
    AxisState, ButtonState, CheckableState, DialogRecipes, DialogState, MenuRecipes,
    RadioGroupRecipes, SelectRecipes, SliderRecipes, TabsRecipes, ToastRecipes,
};
use kivact::view::Orientation;
pub(crate) fn button_content(_: &Theme, s: &Look<ButtonState>) -> Style {
    let base = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(8);
    if s.variant::<crate::button::Slant>().is_some() {
        base.transform(Transform::new().skew_x(12))
    } else {
        base
    }
}
/// The spinner the button turns while busy.
pub(crate) fn button_busy(t: &Theme, _: &Look<ButtonState>) -> Style {
    let phase = 0.0;
    let color = t.ext::<Tokens>().blue;
    let background = t.ext::<Tokens>().surface_active;
    style().size(16, 16).shrink(0).paint(move |d, size| {
        let arc = |fraction: f64| {
            let mut path = Path::new();
            for i in 0..=64 {
                let a = -std::f64::consts::FRAC_PI_2
                    + std::f64::consts::TAU * (phase + fraction * i as f64 / 64.0);
                let p = Point::new(
                    size.width / 2.0 + 7.0 * a.cos(),
                    size.height / 2.0 + 7.0 * a.sin(),
                );
                path = if i == 0 {
                    path.move_to(p)
                } else {
                    path.line_to(p)
                };
            }
            path
        };
        let stroke = Stroke::new(2.0).cap(LineCap::Round).join(LineJoin::Round);
        d.stroke_path(&arc(1.0), &stroke, background);
        d.stroke_path(&arc(0.25), &stroke, color);
    })
}
fn chrome(t: &Theme) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .background(p.surface)
        .border(1, p.line)
        .radius(10)
        .color(p.text)
        .shadow(Shadow::new(0, 10, 15, ColorToken::Shadow))
}
pub(crate) fn select(r: SelectRecipes) -> SelectRecipes {
    r.trigger(|_| select_trigger)
        .surface(|_| |t: &Theme, _: &Look<PopupState>| select_list(t).max_height(250))
        .option(|_| |t: &Theme, _: &Look<ListItemState>| select_option(t))
        .indicator(|_| select_indicator)
}
fn select_trigger(t: &Theme, s: &Look<kivact::ui::SelectTriggerState>) -> Style {
    let p = t.ext::<Tokens>();
    // The heights of the buttons of the same size.
    let height = match s.size {
        kivact::ui::ControlSize::ExtraSmall => 26,
        kivact::ui::ControlSize::Small => 32,
        kivact::ui::ControlSize::Medium => 36,
        kivact::ui::ControlSize::Large => 40,
    };
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .gap(8)
        .height(height)
        .padding_x(12)
        .border(1, p.line)
        .radius(8)
        .background(p.surface)
        .color(p.text)
}
/// The 16 px box of a mark icon, in the text color.
fn mark(t: &Theme) -> Style {
    style().size(16, 16).shrink(0).color(t.ext::<Tokens>().text)
}
/// The trigger's chevron, the theme's `selector` icon.
fn select_indicator(t: &Theme, _: &Look<PopupState>) -> Style {
    mark(t)
}
pub(crate) fn menu(r: MenuRecipes) -> MenuRecipes {
    r.trigger(|_| menu_trigger)
        .surface(|_| {
            |t: &Theme, _: &Look<PopupState>| {
                chrome(t)
                    .direction(Direction::Column)
                    .min_width(192)
                    .padding(4)
                    .radius(8)
            }
        })
        .item(|_| menu_item)
        .separator(|_| {
            |_: &Theme, _: &Look<PartState>| {
                style().height(1).margin_y(4).background(ColorToken::Border)
            }
        })
        .shortcut(|_| {
            |_: &Theme, _: &Look<MenuItemState>| style().font_size(12).color(ColorToken::TextMuted)
        })
        .check(|_| menu_check)
        .submenu(|_| menu_submenu)
}
fn menu_item(_: &Theme, _: &Look<MenuItemState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(8)
        .height(32)
        .padding_x(8)
        .radius(4)
        .hover(style().background(ColorToken::SurfaceAlt))
        .focus(style().background(ColorToken::SurfaceAlt))
        .selected(style().background(ColorToken::SurfaceAlt))
}
/// The box of a check item's `check` icon.
fn menu_check(t: &Theme, _: &Look<MenuItemState>) -> Style {
    mark(t)
}
/// The box of a submenu's `chevron-right` icon.
fn menu_submenu(t: &Theme, _: &Look<MenuItemState>) -> Style {
    mark(t)
}
pub(crate) fn dialog(r: DialogRecipes) -> DialogRecipes {
    r.surface(|_| dialog_surface)
        .title(|_| |_: &Theme, _: &Look<DialogState>| style().font_size(18).weight(600))
        .description(|_| {
            |_: &Theme, _: &Look<DialogState>| style().font_size(14).color(ColorToken::TextMuted)
        })
        // The title over the description, clear of the 36 px close button.
        .header(|_| |_: &Theme, _: &Look<DialogState>| style().gap(6).padding_right(44))
        .body(|_| |_: &Theme, _: &Look<DialogState>| style().gap(8))
        .footer(|_| dialog_footer)
        .scrim(|_| {
            |_: &Theme, _: &Look<DialogState>| style().background(crate::tokens::overlay_scrim())
        })
        .close(|_| dialog_close)
}
fn dialog_surface(t: &Theme, s: &Look<DialogState>) -> Style {
    // A modal and a drawer grow in from 96%.
    let enter = |s: Style| {
        s.enter(style().scale(0.96)).transition(
            kivact::style::TransitionProp::All,
            kivact::motion::Transition::tween(200, kivact::motion::Easing::EaseOut),
        )
    };
    match s.kind {
        DialogMode::Drawer(_) => enter(
            chrome(t)
                .width(400)
                .max_width(pct(100))
                .padding(24)
                .gap(16)
                .radius(0),
        ),
        DialogMode::Popover => chrome(t).width(288).padding(16).gap(6),
        _ => enter(chrome(t).width(480).max_width(pct(95)).padding(24).gap(16)),
    }
}
fn dialog_footer(_: &Theme, s: &Look<DialogState>) -> Style {
    let base = style().gap(8).justify_content(Justify::End);
    if matches!(s.kind, DialogMode::Drawer(_)) {
        base.padding_top(16).border_top(1, ColorToken::Border)
    } else {
        base
    }
}
/// The close button around the theme's `x` icon, muted until hovered.
fn dialog_close(t: &Theme, s: &Look<DialogState>) -> Style {
    let tokens = t.ext::<Tokens>();
    let inset = match s.kind {
        DialogMode::Popover => 8,
        _ => 18,
    };
    style()
        .position(kivact::style::Position::Absolute)
        .top(inset)
        .right(inset)
        .size(36, 36)
        .radius(8)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(tokens.text_muted)
        .pressed(style().opacity(0.7))
        .hover(
            style()
                .background(ColorToken::SurfaceAlt)
                .color(tokens.text),
        )
}
pub(crate) fn tooltip(t: &Theme, _: &Look<PopupState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .padding_x(10)
        .padding_y(6)
        .max_width(320)
        .font_size(12)
        .weight(500)
        .background(p.text)
        .color(p.surface)
        .radius(6)
}
pub(crate) fn tabs(r: TabsRecipes) -> TabsRecipes {
    r.list(|_| |t: &Theme, _: &Look<TabListState>| choice_list(t, false).direction(Direction::Row))
        .tab(|_| |t: &Theme, _: &Look<TabState>| choice_option(t, false))
        .panel(|_| |_: &Theme, _: &Look<TabState>| style().padding_top(16))
}

fn menu_trigger(t: &Theme, _: &Look<PopupState>) -> Style {
    let color = t.ext::<Tokens>().text;
    crate::button::button_style(
        t,
        kivact::ui::ButtonVariant::Secondary,
        None,
        kivact::ui::ControlSize::Medium,
        false,
    )
    .padding_right(40)
    .paint(move |d, size| {
        let x = size.width - 32.0;
        let y = (size.height - 16.0) / 2.0;
        let p = Path::new()
            .move_to(Point::new(x + 4.0, y + 6.0))
            .line_to(Point::new(x + 8.0, y + 10.0))
            .line_to(Point::new(x + 12.0, y + 6.0));
        d.stroke_path(
            &p,
            &Stroke::new(4.0 / 3.0)
                .cap(LineCap::Round)
                .join(LineJoin::Round),
            color,
        );
    })
}
/// The recipes of a plain radio group, and of the [`RadioGroupVariant::Segmented`] pill look
/// of one (no indicator, a pill per segment in a tinted track).
pub(crate) fn radio_group(r: RadioGroupRecipes) -> RadioGroupRecipes {
    r.group(|_| radio_group_layout)
        .item(|_| radio_item)
        .indicator(|_| radio_indicator)
        .description(|_| {
            |_: &Theme, _: &Look<CheckableState>| {
                style().font_size(13).color(crate::ColorName::TextMuted)
            }
        })
}
fn radio_group_layout(t: &Theme, orientation: &Look<Orientation>) -> Style {
    if orientation.variant::<RadioGroupVariant>() == Some(&RadioGroupVariant::Segmented) {
        return choice_list(t, true).direction(Direction::Row);
    }
    let s = style().gap(12);
    match **orientation {
        Orientation::Vertical => s.direction(Direction::Column),
        _ => s.direction(Direction::Row).wrap(true),
    }
}
/// A radio row: the indicator beside the label.
fn radio_item(t: &Theme, s: &Look<CheckableState>) -> Style {
    if s.variant::<RadioGroupVariant>() == Some(&RadioGroupVariant::Segmented) {
        return choice_option(t, true);
    }
    crate::choice::check_style(t).weight(500)
}
/// The 16 px ring of an option, the accent around the theme's `dot` while
/// chosen.
fn radio_indicator(t: &Theme, s: &Look<CheckableState>) -> Style {
    if s.variant::<RadioGroupVariant>() == Some(&RadioGroupVariant::Segmented) {
        // A segment shows no indicator: out of the row, with no size.
        return style().position(Position::Absolute).size(0, 0).clip(true);
    }
    let p = t.ext::<Tokens>();
    let on = s.is_checked();
    style()
        .size(16, 16)
        .shrink(0)
        .radius(8)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(p.surface)
        .border(1, if on { p.blue } else { p.line_strong })
        .color(p.blue)
}
/// The height of a slider's track and the side of its thumb.
const TRACK: f64 = 20.0;
const BAR: f64 = 6.0;
pub(crate) fn slider(r: SliderRecipes) -> SliderRecipes {
    r.track(|_| slider_track)
        .range(|_| slider_range)
        .thumb(|_| slider_thumb)
}
/// The bar of a track or its filled range: 6 px across the middle of `size`.
fn bar(size: kivact::geometry::Size) -> kivact::geometry::Rect {
    if size.width >= size.height {
        kivact::geometry::Rect::new(0.0, (size.height - BAR) / 2.0, size.width, BAR)
    } else {
        kivact::geometry::Rect::new((size.width - BAR) / 2.0, 0.0, BAR, size.height)
    }
}
fn slider_track(t: &Theme, s: &Look<AxisState>) -> Style {
    let color = t.ext::<Tokens>().surface_active;
    let base = style()
        .shrink(0)
        .cursor(Cursor::Pointer)
        .paint(move |d, size| d.rounded_rect(bar(size), BAR / 2.0, color));
    match s.orientation {
        Orientation::Vertical => base.width(TRACK).height(160),
        _ => base.height(TRACK).min_width(120),
    }
}
fn slider_range(t: &Theme, _: &Look<AxisState>) -> Style {
    let color = t.ext::<Tokens>().blue;
    style().paint(move |d, size| {
        if size.width > 0.0 && size.height > 0.0 {
            d.rounded_rect(bar(size), BAR / 2.0, color);
        }
    })
}
/// A white 16 px thumb ringed in the accent.
fn slider_thumb(t: &Theme, _: &Look<AxisState>) -> Style {
    style()
        .size(16, 16)
        .radius(8)
        .background(Color::WHITE)
        .border(1, t.ext::<Tokens>().blue)
}
pub(crate) fn toast(r: ToastRecipes) -> ToastRecipes {
    r.region(|_| toast_region)
        .toast(|_| toast_card)
        .title(|_| {
            |_: &Theme, _: &Look<ToastState>| style().weight(500).color(crate::ColorName::Text)
        })
        .body(|_| {
            |_: &Theme, _: &Look<ToastState>| style().font_size(13).color(ColorToken::TextMuted)
        })
        .icon(|_| toast_icon)
        .close(|_| toast_close)
}
/// The stack at the bottom right of the window, above the page.
fn toast_region(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .position(Position::Fixed)
        .z_index(100)
        .max_width(pct(100) - 16.0)
        .align_items(Align::End)
        .gap(8)
        .bottom(16)
        .right(16)
}
/// A card in the hue's color, which its icon takes.
fn toast_card(t: &Theme, s: &Look<ToastState>) -> Style {
    let p = t.ext::<Tokens>();
    let hue = match s.tone {
        Tone::Neutral => Hue::Gray,
        Tone::Info => Hue::Blue,
        Tone::Success => Hue::Mint,
        Tone::Warning => Hue::Yellow,
        Tone::Danger => Hue::Red,
        Tone::Accent => Hue::Pink,
    };
    chrome(t)
        .direction(Direction::Row)
        .width(356)
        .max_width(pct(100))
        .padding_left(16)
        .padding_right(12)
        .padding_y(14)
        .align_items(Align::Start)
        .gap(10)
        .color(p.hue(hue).1)
}
/// The tone's icon: 16 px, in the card's color, level with the title.
fn toast_icon(_: &Theme, _: &Look<ToastState>) -> Style {
    style().size(16, 16).shrink(0).margin_top(2)
}
/// The 28 px close button around the theme's `x` icon.
fn toast_close(t: &Theme, _: &Look<ToastState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .size(28, 28)
        .shrink(0)
        .radius(8)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(p.text_muted)
        .hover(style().background(p.surface_active).color(p.text))
        .pressed(style().opacity(0.7))
}
