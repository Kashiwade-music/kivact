//! Role: Install Astryx Neutral's recipes of the metrics group: the heading, the stat and the description list.
use crate::color as c;
use crate::radius as r;
use crate::recipe_parts::text;
use crate::type_scale as t;
use crate::variants::{
    HeadingDisplay, LabelPosition, MetadataColumns, MetadataLabelWidth, MetadataOrientation,
};
use kivact::style::{Align, Display, FlexDirection, Length, Style, Theme, Track, color, style};
use kivact::ui::Trend;
use kivact::ui::theme::{
    DescriptionListTheme, HeadingState, HeadingTheme, Look, StatState, StatTheme,
};
use kivact::view::Orientation;

/// The metadata layout that a description list and its pairs read from their variants.
#[derive(Clone, Copy, Debug)]
struct Metadata {
    columns: MetadataColumns,
    position: Option<LabelPosition>,
    width: Option<f64>,
    orientation: Orientation,
}

impl Metadata {
    fn of(look: &Look<()>) -> Self {
        Self {
            columns: look.variant::<MetadataColumns>().unwrap_or_default(),
            position: look.variant::<LabelPosition>(),
            width: look.variant::<MetadataLabelWidth>().map(|w| w.0),
            orientation: look
                .variant::<MetadataOrientation>()
                .map_or(Orientation::Vertical, |o| o.0),
        }
    }

    fn horizontal(self) -> bool {
        self.orientation == Orientation::Horizontal
    }

    fn position(self) -> LabelPosition {
        if self.horizontal() {
            LabelPosition::Top
        } else {
            self.position.unwrap_or(match self.columns {
                MetadataColumns::Single | MetadataColumns::Count(0 | 1) => LabelPosition::Start,
                _ => LabelPosition::Top,
            })
        }
    }

    fn aligned(self) -> bool {
        !self.horizontal()
            && self.position() == LabelPosition::Start
            && matches!(
                self.columns,
                MetadataColumns::Single | MetadataColumns::Count(0 | 1)
            )
    }
}

/// The heading: the display role when the variant names one, else the role of its level.
fn heading(_theme: &Theme, look: &Look<HeadingState>, _base: Style) -> Style {
    let role = match look.variant::<HeadingDisplay>() {
        Some(HeadingDisplay::D1) => t::DISPLAY1,
        Some(HeadingDisplay::D2) => t::DISPLAY2,
        Some(HeadingDisplay::D3) => t::DISPLAY3,
        None => match u32::from(look.level) {
            1 => t::HEADING1,
            2 => t::HEADING2,
            3 => t::HEADING3,
            4 => t::HEADING4,
            5 => t::HEADING5,
            _ => t::HEADING6,
        },
    };
    text(style(), role, c::TEXT_PRIMARY)
}

/// The stat card: the surface, the border and the element radius.
fn stat_root(_theme: &Theme, _look: &Look<StatState>, base: Style) -> Style {
    base.background(c::BG_SURFACE)
        .border_color(color::BORDER)
        .border_radius(r::RADIUS_ELEMENT)
}

/// The stat's label: the secondary text color.
fn stat_label(_theme: &Theme, _look: &Look<StatState>, base: Style) -> Style {
    base.color(c::TEXT_SECONDARY)
}

/// The stat's value: the primary text color.
fn stat_value(_theme: &Theme, _look: &Look<StatState>, base: Style) -> Style {
    base.color(c::TEXT_PRIMARY)
}

/// The stat's delta: the success color when rising, the error color when falling.
fn stat_delta(_theme: &Theme, look: &Look<StatState>, base: Style) -> Style {
    base.color(match look.trend {
        Trend::Up => c::SUCCESS,
        Trend::Down => c::ERROR,
        Trend::Neutral => c::TEXT_SECONDARY,
    })
}

/// The description list: a row when horizontal, else a grid of the chosen columns or a label grid.
fn description_root(_theme: &Theme, look: &Look<()>, _base: Style) -> Style {
    let v = Metadata::of(look);
    if v.horizontal() {
        return style()
            .flex_direction(FlexDirection::Row)
            .flex_wrap(true)
            .gap(16);
    }
    let base = style()
        .display(Display::Grid)
        .row_gap(8)
        .column_gap(16)
        .min_width(0);
    if v.aligned() {
        let term = v
            .width
            .map_or(Track::from(Length::MaxContent), |w| Track::from(w.max(0.0)));
        base.grid_template_columns((term, Track::minmax(0, Track::fr(1))))
    } else {
        let tracks = match v.columns {
            MetadataColumns::Single => Track::fr(1),
            MetadataColumns::Count(n) => {
                Track::repeat(u16::from(n.max(1)), Track::minmax(0, Track::fr(1)))
            }
            MetadataColumns::Multi => Track::repeat_auto_fill(Track::minmax(280, Track::fr(1))),
        };
        base.grid_template_columns(tracks)
    }
}

/// One pair: its contents when aligned in the grid, else a row or a column by the label position.
fn description_item(_theme: &Theme, look: &Look<()>, _base: Style) -> Style {
    let v = Metadata::of(look);
    if v.aligned() {
        return style().display(Display::Contents);
    }
    style()
        .flex_direction(if v.position() == LabelPosition::Start {
            FlexDirection::Row
        } else {
            FlexDirection::Column
        })
        .align_items(Align::Start)
        .gap(if v.position() == LabelPosition::Start {
            16
        } else {
            2
        })
        .min_width(0)
}

/// A term: the secondary text, fixed to the label width when it sits beside its detail.
fn description_term(_theme: &Theme, look: &Look<()>, _base: Style) -> Style {
    let v = Metadata::of(look);
    let base = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(4)
        .flex_shrink(0.0)
        .font_size(14)
        .line_height(20.0 / 14.0)
        .font_weight(500)
        .color(c::TEXT_SECONDARY);
    if v.position() == LabelPosition::Start && !v.aligned() {
        v.width.map_or(base.clone(), |w| base.width(w.max(0.0)))
    } else {
        base
    }
}

/// A detail: the primary text, growing to fill its pair.
fn description_detail(_theme: &Theme, _look: &Look<()>, _base: Style) -> Style {
    style().min_width(0).flex_grow(1.0).color(c::TEXT_PRIMARY)
}

/// Install the kit's recipes of the metrics group.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(HeadingTheme::default().root(heading))
        .with(
            StatTheme::default()
                .root(stat_root)
                .label(stat_label)
                .value(stat_value)
                .delta(stat_delta),
        )
        .with(
            DescriptionListTheme::default()
                .root(description_root)
                .item(description_item)
                .term(description_term)
                .detail(description_detail),
        )
}
