//! Role: Style canonical progress steps from actual item and responsive layout facts.
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, Color, Direction, Justify, Position, Style, Theme, pct, style};
use kivact::ui::{
    Look, StepIndicatorKind, StepStatus, StepperItemState, StepperRecipes, StepperState,
};
use kivact::view::Orientation;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepDensity {
    Compact,
    #[default]
    Balanced,
    Spacious,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepIndicatorPosition {
    #[default]
    Separated,
    OnTrack,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepCollapsedVariant {
    #[default]
    WithLabelAndControls,
    WithLabel,
    HiddenLabel,
}

/// Reactive presentation choices; orientation, count and selection stay canonical.
#[derive(Clone, Debug, PartialEq)]
pub struct StepperAppearance {
    pub density: Value<StepDensity>,
    pub position: Value<StepIndicatorPosition>,
    pub collapsed: Value<StepCollapsedVariant>,
    pub connector_gap: Value<f64>,
    pub item_densities: Vec<(usize, StepDensity)>,
}
impl Default for StepperAppearance {
    fn default() -> Self {
        Self {
            density: StepDensity::Balanced.into_value(),
            position: StepIndicatorPosition::Separated.into_value(),
            collapsed: StepCollapsedVariant::WithLabelAndControls.into_value(),
            connector_gap: 0.0.into_value(),
            item_densities: Vec::new(),
        }
    }
}
impl kivact::ui::Variant for StepperAppearance {}
struct Presentation {
    density: StepDensity,
    position: StepIndicatorPosition,
    collapsed: StepCollapsedVariant,
    connector_gap: f64,
    item_densities: Vec<(usize, StepDensity)>,
}
fn appearance<S>(s: &Look<S>) -> Presentation {
    s.variant::<StepperAppearance>().map_or_else(
        || Presentation {
            density: StepDensity::Balanced,
            position: StepIndicatorPosition::Separated,
            collapsed: StepCollapsedVariant::WithLabelAndControls,
            connector_gap: 0.0,
            item_densities: Vec::new(),
        },
        |v| Presentation {
            density: v.density.get(),
            position: v.position.get(),
            collapsed: v.collapsed.get(),
            connector_gap: v.connector_gap.get(),
            item_densities: v.item_densities.clone(),
        },
    )
}

fn logical_row() -> Direction {
    if crate::stack::rtl() {
        Direction::RowReverse
    } else {
        Direction::Row
    }
}
fn on_track(s: &Look<StepperItemState>) -> bool {
    appearance(s).position == StepIndicatorPosition::OnTrack && !s.collapsed
}
fn density(s: &Look<StepperItemState>) -> StepDensity {
    let a = appearance(s);
    a.item_densities
        .iter()
        .find(|(index, _)| *index == s.index)
        .map_or_else(|| a.density, |(_, density)| *density)
}
pub(crate) fn stepper(r: StepperRecipes) -> StepperRecipes {
    r.root(|_| |_: &Theme, _: &Look<StepperState>| style().gap(4).padding(0).min_width(0))
        .item(|_| item)
        .layout(|_| layout)
        .track(|_| track)
        .body(|_| body)
        .leading(|_| |t: &Theme, s: &Look<StepperItemState>| connector(t, s, true))
        .trailing(|_| |t: &Theme, s: &Look<StepperItemState>| connector(t, s, false))
        .progress(|_| progress)
        .indicator(|_| indicator)
        .label(|_| label)
        .description(|_| description)
        .content(|_| content)
        .end_content(|_| |_: &Theme, _: &Look<StepperItemState>| style().shrink(0))
        .previous(|_| |t: &Theme, s: &Look<StepperState>| navigation(t, s, false))
        .next(|_| |t: &Theme, s: &Look<StepperState>| navigation(t, s, true))
}
fn item(theme: &Theme, s: &Look<StepperItemState>) -> Style {
    let t = theme.ext::<crate::Tokens>();
    if s.collapsed && appearance(s).collapsed == StepCollapsedVariant::HiddenLabel {
        return style()
            .visibility(kivact::style::Visibility::Hidden)
            .position(Position::Absolute)
            .size(0, 0);
    }
    let base = style()
        .position(Position::Relative)
        .min_width(0)
        .gap(2)
        .radius(t.radius.element)
        .focus_visible(crate::recipe_style::ring(&t, t.colors.accent));
    if s.orientation == Orientation::Horizontal {
        base.grow(1).basis(0)
    } else {
        base
    }
}
fn layout(_: &Theme, s: &Look<StepperItemState>) -> Style {
    style()
        .direction(if on_track(s) && s.orientation == Orientation::Horizontal {
            Direction::Column
        } else {
            logical_row()
        })
        .align_items(if on_track(s) && s.orientation == Orientation::Vertical {
            Align::Stretch
        } else {
            Align::Center
        })
        .justify_content(if s.collapsed {
            Justify::Center
        } else {
            Justify::Start
        })
        .gap(8)
        .min_width(0)
        .padding_top(
            if !on_track(s) && s.orientation == Orientation::Horizontal && !s.collapsed {
                6.0
            } else {
                0.0
            },
        )
        .padding_left(
            if !on_track(s) && s.orientation == Orientation::Vertical && !crate::stack::rtl() {
                6.0
            } else {
                0.0
            },
        )
        .padding_right(
            if !on_track(s) && s.orientation == Orientation::Vertical && crate::stack::rtl() {
                6.0
            } else {
                0.0
            },
        )
}
fn track(_: &Theme, s: &Look<StepperItemState>) -> Style {
    if s.indicator == StepIndicatorKind::None && !on_track(s) {
        return style().size(0, 0).position(Position::Absolute);
    }
    let a = appearance(s);
    let base = style()
        .direction(if on_track(s) && s.orientation == Orientation::Vertical {
            Direction::Column
        } else {
            logical_row()
        })
        .align_items(Align::Center)
        .gap(a.connector_gap.clamp(0.0, 8.0))
        .shrink(0);
    if on_track(s) && s.orientation == Orientation::Horizontal {
        base.width(pct(100))
    } else {
        base
    }
}
fn body(theme: &Theme, s: &Look<StepperItemState>) -> Style {
    let t = theme.ext::<crate::Tokens>();
    let density = density(s);
    let padding = match density {
        StepDensity::Compact => t.spacing.s1,
        StepDensity::Balanced => t.spacing.s2,
        StepDensity::Spacious => t.spacing.s3,
    };
    let base = style()
        .direction(Direction::Column)
        .min_width(0)
        .grow(1)
        .padding_y(padding)
        .padding_x(if density == StepDensity::Spacious {
            t.spacing.s3
        } else {
            t.spacing.s2
        })
        .align_items(
            if s.collapsed || on_track(s) && s.orientation == Orientation::Horizontal {
                Align::Center
            } else {
                Align::Start
            },
        )
        .radius(t.radius.element);
    if s.interactive && !s.disabled {
        base.hover(style().background(t.colors.overlay_hover))
            .pressed(style().background(t.colors.overlay_pressed))
    } else {
        base
    }
}
fn connector(theme: &Theme, s: &Look<StepperItemState>, leading: bool) -> Style {
    if !on_track(s) {
        return style().size(0, 0);
    }
    let t = theme.ext::<crate::Tokens>();
    let filled = if leading {
        s.index <= s.current
    } else {
        s.index < s.current
    };
    let absent = if leading {
        s.index == 0
    } else {
        s.index + 1 == s.count
    };
    let base = style()
        .grow(1)
        .opacity(if absent { 0.0 } else { 1.0 })
        .background(if filled {
            t.colors.accent
        } else {
            t.colors.border
        });
    if s.orientation == Orientation::Vertical {
        base.width(t.spacing.s1).min_height(t.spacing.s2)
    } else {
        base.height(t.spacing.s1).min_width(t.spacing.s2)
    }
}
fn progress(theme: &Theme, s: &Look<StepperItemState>) -> Style {
    if on_track(s) || s.collapsed {
        return style().size(0, 0);
    }
    let t = theme.ext::<crate::Tokens>();
    let base = style()
        .position(Position::Absolute)
        .top(0)
        .radius(t.radius.full)
        .transition(
            kivact::style::TransitionProp::Paint,
            kivact::motion::Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
        )
        .background(if s.index <= s.current {
            t.colors.accent
        } else {
            t.colors.border
        });
    if s.orientation == Orientation::Horizontal {
        base.left(0).width(pct(100)).height(t.spacing.s1)
    } else if crate::stack::rtl() {
        base.right(0).width(t.spacing.s1).height(pct(100))
    } else {
        base.left(0).width(t.spacing.s1).height(pct(100))
    }
}
fn colors(t: &crate::Tokens, status: StepStatus) -> (Color, Color) {
    let c = t.colors;
    match status {
        StepStatus::Default => (c.accent, c.bg_surface),
        StepStatus::Accent => (c.accent, c.on_accent),
        StepStatus::Success => (c.success, c.on_success),
        StepStatus::Warning => (c.warning, c.on_warning),
        StepStatus::Error => (c.error, c.on_error),
    }
}
fn indicator(theme: &Theme, s: &Look<StepperItemState>) -> Style {
    let t = theme.ext::<crate::Tokens>();
    let (bg, ink) = colors(&t, s.status);
    let reached = s.index <= s.current;
    let base = style()
        .size(16, 16)
        .shrink(0)
        .radius(t.radius.full)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .opacity(if s.disabled {
            t.sizes.disabled_opacity
        } else {
            1.0
        });
    if s.indicator == StepIndicatorKind::Custom {
        return base;
    }
    if s.indicator == StepIndicatorKind::Number
        || s.index > s.current && matches!(s.status, StepStatus::Default | StepStatus::Accent)
    {
        return base
            .font_size(t.typography.size_xs)
            .weight(t.typography.weight_semibold)
            .background(if reached { bg } else { t.colors.bg_muted })
            .color(if reached {
                ink
            } else if s.disabled {
                t.colors.text_disabled
            } else {
                t.colors.text_secondary
            });
    }
    let status_ink = match s.status {
        StepStatus::Success => t.colors.text_success,
        StepStatus::Warning => t.colors.text_warning,
        StepStatus::Error => t.colors.text_error,
        _ => bg,
    };
    auto_indicator(base, bg, ink, s.index == s.current, s.status, status_ink)
}
fn auto_indicator(
    base: Style,
    bg: Color,
    ink: Color,
    active: bool,
    status: StepStatus,
    status_ink: Color,
) -> Style {
    base.font_size(0).paint(move |draw, size| {
        use kivact::draw::{Path, Stroke};
        use kivact::geometry::Point;
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

fn label(theme: &Theme, s: &Look<StepperItemState>) -> Style {
    let t = theme.ext::<crate::Tokens>();
    let font = t.typography.body;
    style()
        .font_size(font.size_px)
        .line_height(font.line_height_px / font.size_px)
        .color(if s.disabled {
            t.colors.text_disabled
        } else if s.index > s.current {
            t.colors.text_secondary
        } else {
            t.colors.text_primary
        })
        .weight(if s.index == s.current {
            t.typography.weight_semibold
        } else {
            font.weight
        })
        .single_line()
}
fn description(theme: &Theme, _: &Look<StepperItemState>) -> Style {
    let t = theme.ext::<crate::Tokens>();
    style()
        .font_size(t.typography.supporting.size_px)
        .line_height(t.spacing.s4 / t.typography.supporting.size_px)
        .color(t.colors.text_secondary)
}
fn content(_: &Theme, _: &Look<StepperItemState>) -> Style {
    style().padding_top(8)
}
fn navigation(theme: &Theme, s: &Look<StepperState>, next: bool) -> Style {
    if appearance(s).collapsed != StepCollapsedVariant::WithLabelAndControls {
        return style()
            .visibility(kivact::style::Visibility::Hidden)
            .position(Position::Absolute)
            .size(0, 0);
    }
    let t = theme.ext::<crate::Tokens>();
    let right = next != crate::stack::rtl();
    let color = t.colors.text_secondary;
    style()
        .size(32, 32)
        .padding(0)
        .font_size(0)
        .background(Color::TRANSPARENT)
        .border(0, Color::TRANSPARENT)
        .focus_visible(crate::recipe_style::ring(&t, t.colors.accent))
        .paint(move |draw, _| {
            use kivact::draw::{Path, Stroke};
            let (a, b) = if right { (13.0, 19.0) } else { (19.0, 13.0) };
            draw.stroke_path(
                &Path::new()
                    .move_to(kivact::geometry::Point::new(a, 10.0))
                    .line_to(kivact::geometry::Point::new(b, 16.0))
                    .line_to(kivact::geometry::Point::new(a, 22.0)),
                &Stroke::new(1.5),
                color,
            );
        })
}
kivact::state::constant_value!(StepDensity, StepIndicatorPosition, StepCollapsedVariant);
