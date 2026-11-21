//! Role: Install Astryx Neutral's stepper recipes: the progress list, its items and connectors, the indicators, the labels and the collapsed controls.
use crate::color as c;
use crate::metrics as m;
use crate::motion::FAST;
use crate::radius as r;
use crate::recipe_parts::{ring, text, tween};
use crate::space as s;
use crate::type_scale as t;
use crate::variants::{StepCollapsedVariant, StepDensity, StepIndicatorPosition};
use kivact::draw::{Path, Stroke};
use kivact::geometry::Point;
use kivact::style::{
    Align, Color, FlexDirection, Justify, Position, Style, Theme, Visibility, pct, style,
};
use kivact::ui::theme::{Look, StepperItemState, StepperState, StepperTheme};
use kivact::ui::{StepIndicatorKind, StepStatus};
use kivact::view::{Orientation, ReadingDirection, use_direction};

/// The flow of a row of steps: reversed in a right-to-left reading direction.
fn logical_row() -> FlexDirection {
    if use_direction() == ReadingDirection::Rtl {
        FlexDirection::RowReverse
    } else {
        FlexDirection::Row
    }
}

/// Whether the indicators sit on the connector track: only in the expanded presentation of that position.
fn on_track(look: &Look<StepperItemState>) -> bool {
    look.variant::<StepIndicatorPosition>().unwrap_or_default() == StepIndicatorPosition::OnTrack
        && !look.collapsed
}

/// The density of a step: its spacing and padding step.
fn density(look: &Look<StepperItemState>) -> StepDensity {
    look.variant::<StepDensity>().unwrap_or_default()
}

/// A hidden, zero-size, absolutely positioned part of a collapsed step.
fn hidden() -> Style {
    style()
        .visibility(Visibility::Hidden)
        .position(Position::Absolute)
        .size(0, 0)
}

/// The root of a stepper: its items with the spacing step between them.
fn root(_: &Theme, _: &Look<StepperState>, _base: Style) -> Style {
    style().gap(s::SPACE_S1).padding(0).min_width(0)
}

/// One step: a relative column that grows along a horizontal list, and takes the focus ring.
fn item(_: &Theme, look: &Look<StepperItemState>, _base: Style) -> Style {
    if look.collapsed
        && look.variant::<StepCollapsedVariant>().unwrap_or_default()
            == StepCollapsedVariant::HiddenLabel
    {
        return hidden();
    }
    let base = style()
        .position(Position::Relative)
        .min_width(0)
        .gap(2)
        .border_radius(r::RADIUS_ELEMENT)
        .focus_visible(ring(c::ACCENT));
    if look.orientation == Orientation::Horizontal {
        base.flex_grow(1.0).flex_basis(0.0)
    } else {
        base
    }
}

/// The layout of a step: its indicator and its content, in the flow of the step's track.
fn layout(_: &Theme, look: &Look<StepperItemState>, _base: Style) -> Style {
    let rtl = use_direction() == ReadingDirection::Rtl;
    let on_track = on_track(look);
    let vertical = look.orientation == Orientation::Vertical;
    let horizontal = look.orientation == Orientation::Horizontal;
    let inset = if !on_track && vertical { 6.0 } else { 0.0 };
    style()
        .flex_direction(if on_track && horizontal {
            FlexDirection::Column
        } else {
            logical_row()
        })
        .align_items(if on_track && vertical {
            Align::Stretch
        } else {
            Align::Center
        })
        .justify_content(if look.collapsed {
            Justify::Center
        } else {
            Justify::Start
        })
        .gap(8)
        .min_width(0)
        .padding_top(if !on_track && horizontal && !look.collapsed {
            6.0
        } else {
            0.0
        })
        .padding_left(if rtl { 0.0 } else { inset })
        .padding_right(if rtl { inset } else { 0.0 })
}

/// The track of a step: the connector line that joins the steps, or nothing when it has no indicator.
fn track(_: &Theme, look: &Look<StepperItemState>, _base: Style) -> Style {
    if look.indicator == StepIndicatorKind::None && !on_track(look) {
        return style().size(0, 0).position(Position::Absolute);
    }
    let base = style()
        .flex_direction(
            if on_track(look) && look.orientation == Orientation::Vertical {
                FlexDirection::Column
            } else {
                logical_row()
            },
        )
        .align_items(Align::Center)
        .gap(0)
        .flex_shrink(0.0);
    if on_track(look) && look.orientation == Orientation::Horizontal {
        base.width(pct(100))
    } else {
        base
    }
}

/// The body of a step: its label and description in a column, with the density's padding.
fn body(_: &Theme, look: &Look<StepperItemState>, _base: Style) -> Style {
    let density = density(look);
    let padding = match density {
        StepDensity::Compact => s::SPACE_S1,
        StepDensity::Balanced => s::SPACE_S2,
        StepDensity::Spacious => s::SPACE_S3,
    };
    style()
        .flex_direction(FlexDirection::Column)
        .min_width(0)
        .flex_grow(1.0)
        .padding_y(padding)
        .padding_x(if density == StepDensity::Spacious {
            s::SPACE_S3
        } else {
            s::SPACE_S2
        })
        .align_items(
            if look.collapsed || on_track(look) && look.orientation == Orientation::Horizontal {
                Align::Center
            } else {
                Align::Start
            },
        )
        .border_radius(r::RADIUS_ELEMENT)
}

/// A connector of a step, leading or trailing: a line that is filled up to the current step.
fn connector(look: &Look<StepperItemState>, leading: bool) -> Style {
    if !on_track(look) {
        return style().size(0, 0);
    }
    let filled = if leading {
        look.index <= look.current
    } else {
        look.index < look.current
    };
    let absent = if leading {
        look.index == 0
    } else {
        look.index + 1 == look.count
    };
    let base = style()
        .flex_grow(1.0)
        .opacity(if absent { 0.0 } else { 1.0 })
        .background(if filled { c::ACCENT } else { c::BORDER });
    if look.orientation == Orientation::Vertical {
        base.width(s::SPACE_S1).min_height(s::SPACE_S2)
    } else {
        base.height(s::SPACE_S1).min_width(s::SPACE_S2)
    }
}

/// The leading connector of a step.
fn leading(_: &Theme, look: &Look<StepperItemState>, _base: Style) -> Style {
    connector(look, true)
}

/// The trailing connector of a step.
fn trailing(_: &Theme, look: &Look<StepperItemState>, _base: Style) -> Style {
    connector(look, false)
}

/// The progress bar of a step in the collapsed presentation, filled up to the current step.
fn progress(_: &Theme, look: &Look<StepperItemState>, _base: Style) -> Style {
    if on_track(look) || look.collapsed {
        return style().size(0, 0);
    }
    let base = tween(
        style()
            .position(Position::Absolute)
            .top(0)
            .border_radius(r::RADIUS_FULL)
            .background(if look.index <= look.current {
                c::ACCENT
            } else {
                c::BORDER
            }),
        FAST,
    );
    if look.orientation == Orientation::Horizontal {
        base.left(0).width(pct(100)).height(s::SPACE_S1)
    } else if use_direction() == ReadingDirection::Rtl {
        base.right(0).width(s::SPACE_S1).height(pct(100))
    } else {
        base.left(0).width(s::SPACE_S1).height(pct(100))
    }
}

/// The fill and ink of an indicator for its status.
fn status_colors(status: StepStatus) -> (Color, Color) {
    match status {
        StepStatus::Default => (c::ACCENT, c::BG_SURFACE),
        StepStatus::Accent => (c::ACCENT, c::ON_ACCENT),
        StepStatus::Success => (c::SUCCESS, c::ON_SUCCESS),
        StepStatus::Warning => (c::WARNING, c::ON_WARNING),
        StepStatus::Error => (c::ERROR, c::ON_ERROR),
    }
}

/// The indicator of a step: a number, a custom mark, or the automatic status mark.
fn indicator(_: &Theme, look: &Look<StepperItemState>, _base: Style) -> Style {
    let (bg, ink) = status_colors(look.status);
    let reached = look.index <= look.current;
    let base = style()
        .size(16, 16)
        .flex_shrink(0.0)
        .border_radius(r::RADIUS_FULL)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .opacity(if look.disabled {
            m::DISABLED_OPACITY
        } else {
            1.0
        });
    if look.indicator == StepIndicatorKind::Custom {
        return base;
    }
    if look.indicator == StepIndicatorKind::Number
        || look.index > look.current
            && matches!(look.status, StepStatus::Default | StepStatus::Accent)
    {
        return base
            .font_size(t::SIZE_XS)
            .font_weight(t::WEIGHT_SEMIBOLD)
            .background(if reached { bg } else { c::BG_MUTED })
            .color(if reached {
                ink
            } else if look.disabled {
                c::TEXT_DISABLED
            } else {
                c::TEXT_SECONDARY
            });
    }
    let status_ink = match look.status {
        StepStatus::Success => c::TEXT_SUCCESS,
        StepStatus::Warning => c::TEXT_WARNING,
        StepStatus::Error => c::TEXT_ERROR,
        _ => bg,
    };
    auto_indicator(
        base,
        bg,
        ink,
        look.index == look.current,
        look.status,
        status_ink,
    )
}

/// The automatic mark of an indicator: a filled dot when current, the status glyph, or a check on a filled disc.
fn auto_indicator(
    base: Style,
    bg: Color,
    ink: Color,
    active: bool,
    status: StepStatus,
    status_ink: Color,
) -> Style {
    base.font_size(0.0).paint(move |draw, size| {
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let circle = Path::new().arc(center, 7, 0, 360, false);
        let check = Path::new()
            .move_to(Point::new(4.0, 8.0))
            .line_to(Point::new(7.0, 11.0))
            .line_to(Point::new(12.0, 5.0));
        if active {
            draw.stroke_path(&circle, &Stroke::new(2), bg);
            draw.fill_path(&Path::new().arc(center, 4, 0, 360, false), bg);
        } else if matches!(
            status,
            StepStatus::Success | StepStatus::Warning | StepStatus::Error
        ) {
            if status == StepStatus::Warning {
                draw.stroke_path(
                    &Path::new()
                        .move_to(Point::new(8.0, 1.0))
                        .line_to(Point::new(15.0, 14.0))
                        .line_to(Point::new(1.0, 14.0))
                        .close(),
                    &Stroke::new(1.5),
                    status_ink,
                );
                draw.stroke_path(
                    &Path::new()
                        .move_to(Point::new(8.0, 5.0))
                        .line_to(Point::new(8.0, 9.0)),
                    &Stroke::new(1.5),
                    status_ink,
                );
                draw.fill_path(
                    &Path::new().arc(Point::new(8.0, 11.5), 0.7, 0, 360, false),
                    status_ink,
                );
            } else {
                draw.stroke_path(&circle, &Stroke::new(1.5), status_ink);
                if status == StepStatus::Success {
                    draw.stroke_path(&check, &Stroke::new(1.5), status_ink);
                } else {
                    draw.stroke_path(
                        &Path::new()
                            .move_to(Point::new(5.0, 5.0))
                            .line_to(Point::new(11.0, 11.0))
                            .move_to(Point::new(11.0, 5.0))
                            .line_to(Point::new(5.0, 11.0)),
                        &Stroke::new(1.5),
                        status_ink,
                    );
                }
            }
        } else {
            draw.fill_path(&circle, bg);
            draw.stroke_path(&check, &Stroke::new(1.5), ink);
        }
    })
}

/// The label of a step: the body role, semibold for the current step and muted after it.
fn label(_: &Theme, look: &Look<StepperItemState>, _base: Style) -> Style {
    let color = if look.disabled {
        c::TEXT_DISABLED
    } else if look.index > look.current {
        c::TEXT_SECONDARY
    } else {
        c::TEXT_PRIMARY
    };
    let weight = if look.index == look.current {
        t::WEIGHT_SEMIBOLD
    } else {
        t::BODY.weight
    };
    text(style(), t::BODY, color)
        .font_weight(weight)
        .single_line()
}

/// The description of a step: the supporting role in the secondary color.
fn description(theme: &Theme, _: &Look<StepperItemState>, _base: Style) -> Style {
    text(style(), t::SUPPORTING, c::TEXT_SECONDARY)
        .line_height(theme.get(&s::SPACE_S4_VAR) / t::SUPPORTING.size_px)
}

/// The content of a step: a gap below its label.
fn content(_: &Theme, _: &Look<StepperItemState>, _base: Style) -> Style {
    style().padding_top(8)
}

/// The end content of a step: it keeps its size.
fn end_content(_: &Theme, _: &Look<StepperItemState>, _base: Style) -> Style {
    style().flex_shrink(0.0)
}

/// A collapsed stepper's previous or next control: a chevron, mirrored by the reading direction.
fn navigation(look: &Look<StepperState>, next: bool) -> Style {
    if look.variant::<StepCollapsedVariant>().unwrap_or_default()
        != StepCollapsedVariant::WithLabelAndControls
    {
        return hidden();
    }
    let right = next != (use_direction() == ReadingDirection::Rtl);
    let color = c::TEXT_SECONDARY;
    style()
        .size(32, 32)
        .padding(0)
        .font_size(0.0)
        .background(Color::TRANSPARENT)
        .border(0.0, Color::TRANSPARENT)
        .focus_visible(ring(c::ACCENT))
        .paint(move |draw, _| {
            let (a, b) = if right { (13.0, 19.0) } else { (19.0, 13.0) };
            draw.stroke_path(
                &Path::new()
                    .move_to(Point::new(a, 10.0))
                    .line_to(Point::new(b, 16.0))
                    .line_to(Point::new(a, 22.0)),
                &Stroke::new(1.5),
                color,
            );
        })
}

/// The previous control of a stepper.
fn previous(_: &Theme, look: &Look<StepperState>, _base: Style) -> Style {
    navigation(look, false)
}

/// The next control of a stepper.
fn next(_: &Theme, look: &Look<StepperState>, _base: Style) -> Style {
    navigation(look, true)
}

/// Install the kit's recipes of the stepper.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        StepperTheme::default()
            .root(root)
            .previous(previous)
            .next(next)
            .item(item)
            .layout(layout)
            .track(track)
            .leading(leading)
            .trailing(trailing)
            .progress(progress)
            .body(body)
            .indicator(indicator)
            .label(label)
            .description(description)
            .content(content)
            .end_content(end_content),
    )
}
