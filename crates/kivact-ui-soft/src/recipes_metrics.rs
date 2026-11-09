//! Role: Apply Soft typography and metric and description appearances.
use crate::{TextKind, Tokens};
use kivact::style::{Style, Theme};
use kivact::ui::{HeadingState, Look, Variant};

/// Display typography independent of the heading's semantic level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadingDisplay {
    /// Use the largest display heading typography.
    D1,
    /// Use the medium display heading typography.
    D2,
    /// Use the smallest display heading typography.
    D3,
}
impl Variant for HeadingDisplay {}

pub(super) fn heading(t: &Theme, look: &Look<HeadingState>) -> Style {
    let kind = match look.variant::<HeadingDisplay>() {
        Some(HeadingDisplay::D1) => TextKind::Display1,
        Some(HeadingDisplay::D2) => TextKind::Display2,
        Some(HeadingDisplay::D3) => TextKind::Display3,
        None => TextKind::Body,
    };
    crate::text::type_style(
        &t.ext::<Tokens>(),
        kind,
        Some(u32::from(look.level)),
        None,
        None,
        None,
    )
}

/// Column arrangement for standard description lists.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MetadataColumns {
    #[default]
    /// Arrange description pairs in one column.
    Single,
    /// Arrange description pairs in the standard multi-column grid.
    Multi,
    /// Arrange description pairs in the specified number of columns.
    Count(u8),
}
/// Place terms beside or above their details.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelPosition {
    /// Place terms at the logical start beside their details.
    Start,
    /// Place terms above their details.
    Top,
}

/// Reactive appearance shared explicitly by a description list and its pairs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MetadataLook(Option<kivact::state::Value<MetadataAppearance>>);

#[derive(Clone, Copy, Debug, PartialEq)]
struct MetadataAppearance {
    columns: MetadataColumns,
    // xtask-allow(state-fields): option: absent means the component's default
    position: Option<LabelPosition>,
    // xtask-allow(state-fields): option: absent means the component's default
    width: Option<f64>,
    orientation: kivact::view::Orientation,
}
impl Default for MetadataAppearance {
    fn default() -> Self {
        Self {
            columns: MetadataColumns::Single,
            position: None,
            width: None,
            orientation: kivact::view::Orientation::Vertical,
        }
    }
}
impl Variant for MetadataLook {}
impl MetadataLook {
    fn get(self) -> MetadataAppearance {
        self.0
            .map_or_else(MetadataAppearance::default, |value| value.get())
    }

    /// Choose one, a fixed count, or automatically fitted columns.
    pub fn columns(self, value: impl kivact::state::IntoValue<MetadataColumns>) -> Self {
        use kivact::state::IntoValue;
        let value = value.into_value();
        Self(Some(
            (move || MetadataAppearance {
                columns: value.get(),
                ..self.get()
            })
            .into_value(),
        ))
    }
    /// Override the automatic beside/above term placement.
    pub fn label_position(self, value: impl kivact::state::IntoValue<LabelPosition>) -> Self {
        use kivact::state::IntoValue;
        let value = value.into_value();
        Self(Some(
            (move || MetadataAppearance {
                position: Some(value.get()),
                ..self.get()
            })
            .into_value(),
        ))
    }
    /// Fix the beside-label column width instead of its intrinsic maximum.
    pub fn label_width(self, value: impl kivact::state::IntoValue<f64>) -> Self {
        use kivact::state::IntoValue;
        let value = value.into_value();
        Self(Some(
            (move || MetadataAppearance {
                width: Some(value.get().max(0.0)),
                ..self.get()
            })
            .into_value(),
        ))
    }
    /// Arrange pairs in a wrapping row or the selected vertical grid.
    pub fn orientation(
        self,
        value: impl kivact::state::IntoValue<kivact::view::Orientation>,
    ) -> Self {
        use kivact::state::IntoValue;
        let value = value.into_value();
        Self(Some(
            (move || MetadataAppearance {
                orientation: value.get(),
                ..self.get()
            })
            .into_value(),
        ))
    }
}
impl MetadataAppearance {
    fn horizontal(self) -> bool {
        self.orientation == kivact::view::Orientation::Horizontal
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
kivact::state::constant_value!(MetadataAppearance);
kivact::state::constant_value!(MetadataColumns, LabelPosition);

pub(super) fn descriptions(
    r: kivact::ui::DescriptionListRecipes,
) -> kivact::ui::DescriptionListRecipes {
    r.root(|_| description_root)
        .item(|_| description_item)
        .term(|_| description_term)
        .detail(|_| description_detail)
}
fn description_root(_: &Theme, s: &Look<()>) -> Style {
    use kivact::style::{Display, Length, Track, style};
    let v = s
        .variant::<MetadataLook>()
        .copied()
        .unwrap_or_default()
        .get();
    if v.horizontal() {
        return style()
            .direction(kivact::style::Direction::Row)
            .wrap(true)
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
fn description_item(_: &Theme, s: &Look<()>) -> Style {
    use kivact::style::{Direction, Display, style};
    let v = s
        .variant::<MetadataLook>()
        .copied()
        .unwrap_or_default()
        .get();
    if v.aligned() {
        return style().display(Display::Contents);
    }
    style()
        .direction(if v.position() == LabelPosition::Start {
            Direction::Row
        } else {
            Direction::Column
        })
        .align_items(kivact::style::Align::Start)
        .gap(if v.position() == LabelPosition::Start {
            16
        } else {
            2
        })
        .min_width(0)
}
fn description_term(t: &Theme, s: &Look<()>) -> Style {
    use kivact::style::{Direction, style};
    let v = s
        .variant::<MetadataLook>()
        .copied()
        .unwrap_or_default()
        .get();
    let base = style()
        .direction(Direction::Row)
        .align_items(kivact::style::Align::Center)
        .gap(4)
        .shrink(0)
        .font_size(14)
        .line_height(20.0 / 14.0)
        .weight(500)
        .color(t.ext::<Tokens>().colors.text_secondary);
    if v.position() == LabelPosition::Start && !v.aligned() {
        v.width.map_or(base.clone(), |w| base.width(w.max(0.0)))
    } else {
        base
    }
}
fn description_detail(t: &Theme, _: &Look<()>) -> Style {
    kivact::style::style()
        .min_width(0)
        .grow(1)
        .color(t.ext::<Tokens>().colors.text_primary)
}

pub(super) fn stat(r: kivact::ui::StatRecipes) -> kivact::ui::StatRecipes {
    r.root(|prev| {
        move |t: &Theme, s: &Look<kivact::ui::StatState>| {
            let p = t.ext::<Tokens>();
            prev(t, s)
                .background(p.colors.bg_surface)
                .border_color(p.colors.border)
                .radius(p.radius.element)
        }
    })
    .label(|prev| {
        move |t: &Theme, s: &Look<kivact::ui::StatState>| {
            prev(t, s).color(t.ext::<Tokens>().colors.text_secondary)
        }
    })
    .value(|prev| {
        move |t: &Theme, s: &Look<kivact::ui::StatState>| {
            prev(t, s).color(t.ext::<Tokens>().colors.text_primary)
        }
    })
    .delta(|prev| {
        move |t: &Theme, s: &Look<kivact::ui::StatState>| {
            let p = t.ext::<Tokens>();
            prev(t, s).color(match s.trend {
                kivact::ui::Trend::Up => p.colors.success,
                kivact::ui::Trend::Down => p.colors.error,
                kivact::ui::Trend::Neutral => p.colors.text_secondary,
            })
        }
    })
}
