//! Role: Draw the console's radio group, slider and splitter recipes.
use crate::choice_style;
use crate::color::{LINE_STRONG, SURFACE_3, TEXT_3};
use crate::recipes_controls::{button_style, diamond};
use crate::variants::ChoiceCard;
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::geometry::{Point, Rect, Size};
use kivact::style::color::{ACCENT, BORDER, SURFACE, TEXT};
use kivact::style::{Align, Cursor, FlexDirection, Style, Theme, pct, style};
use kivact::ui::theme::{
    AxisState, CheckableState, Look, RadioGroupTheme, SliderTheme, SplitterTheme,
};
use kivact::ui::{ButtonVariant, ControlSize, RadioGroupVariant};
use kivact::view::Orientation;

/// The slider thumb dimensions along and across its axis.
const THUMB: (f64, f64) = (14., 18.);

/// Install the radio group, slider and splitter recipes.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            RadioGroupTheme::default()
                .group(radio_group)
                .item(radio_item)
                .indicator(radio_indicator)
                .description(radio_description),
        )
        .with(
            SliderTheme::default()
                .track(slider_track)
                .range(slider_range)
                .thumb(slider_thumb),
        )
        .with(SplitterTheme::default().handle(splitter_handle))
}

pub(crate) fn radio_group(_: &Theme, orientation: &Look<Orientation>, _base: Style) -> Style {
    if orientation.variant::<RadioGroupVariant>() == Some(RadioGroupVariant::Segmented) {
        return choice_style::segment_group();
    }
    if orientation.variant::<ChoiceCard>().is_some() {
        return choice_style::card_group();
    }
    let s = style().gap(10);
    match **orientation {
        Orientation::Vertical => s.flex_direction(FlexDirection::Column),
        _ => s.flex_direction(FlexDirection::Row).flex_wrap(true),
    }
}

pub(crate) fn radio_item(_: &Theme, state: &Look<CheckableState>, _base: Style) -> Style {
    if state.variant::<RadioGroupVariant>() == Some(RadioGroupVariant::Segmented) {
        // A small slanted secondary button, filled while chosen.
        return button_style(
            ButtonVariant::Secondary,
            None,
            ControlSize::Small,
            false,
            false,
        )
        .clip_path(
            kivact::style::Shape::polygon()
                .point(8, 0)
                .point(pct(100), 0)
                .point(pct(100) - 8, pct(100))
                .point(0, pct(100)),
        )
        .selected(choice_style::segment_chosen());
    }
    if state.variant::<ChoiceCard>().is_some() {
        return choice_style::card_item(state.is_checked());
    }
    style()
        .flex_direction(FlexDirection::Row)
        .gap(9)
        .align_items(Align::Center)
        .min_height(24)
        .color(TEXT)
}

/// The diamond around the theme's `dot`, which is a diamond too.
pub(crate) fn radio_indicator(_: &Theme, state: &Look<CheckableState>, _base: Style) -> Style {
    if state.variant::<RadioGroupVariant>() == Some(RadioGroupVariant::Segmented)
        || state.variant::<ChoiceCard>().is_some()
    {
        return choice_style::hidden_indicator();
    }
    let checked = state.is_checked();
    style()
        .size(16, 16)
        .flex_shrink(0.)
        .align_items(Align::Center)
        .justify_content(kivact::style::Justify::Center)
        .color(ACCENT)
        .paint(move |d, size| {
            let color = if checked { ACCENT } else { LINE_STRONG };
            d.stroke_path(
                &diamond(0., 0., size.width.min(size.height), 1.),
                &Stroke::new(1.5).cap(LineCap::Round).join(LineJoin::Round),
                color,
            );
        })
}

pub(crate) fn radio_description(_: &Theme, _: &Look<CheckableState>, _base: Style) -> Style {
    style().font_size(11.5).color(TEXT_3)
}

/// A 4 px bar across the middle of a box, along its longer side, leaving
/// `inset` at both ends.
fn bar(size: Size, inset: f64) -> Rect {
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
pub(crate) fn slider_track(_: &Theme, state: &Look<AxisState>, _base: Style) -> Style {
    let s = style()
        .flex_shrink(0.)
        .cursor(Cursor::Pointer)
        .paint(|d, size| d.fill_rect(bar(size, THUMB.0 / 2.), SURFACE_3));
    match state.orientation {
        Orientation::Vertical => s.width(28).height(160),
        _ => s.height(28).width(pct(100)).min_width(80),
    }
}

pub(crate) fn slider_range(_: &Theme, _: &Look<AxisState>, _base: Style) -> Style {
    style().paint(|d, size| d.fill_rect(bar(size, 0.), ACCENT))
}

/// The console's parallelogram thumb on the surface, outlined in the accent.
pub(crate) fn slider_thumb(_: &Theme, state: &Look<AxisState>, _base: Style) -> Style {
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
        d.fill_path(&thumb.clone(), SURFACE);
        d.stroke_path(
            &thumb,
            &Stroke::new(1.5).cap(LineCap::Round).join(LineJoin::Round),
            ACCENT,
        );
    })
}

/// A 5 px bar between the panes that lights up under the pointer and while dragged.
pub(crate) fn splitter_handle(_: &Theme, state: &Look<AxisState>, _base: Style) -> Style {
    let s = style()
        .flex_shrink(0.)
        .align_self(Align::Stretch)
        .background(BORDER)
        .hover(style().background(ACCENT))
        .active(style().background(ACCENT));
    match state.orientation {
        Orientation::Vertical => s.height(5).cursor(Cursor::NsResize),
        _ => s.width(5).cursor(Cursor::EwResize),
    }
}
