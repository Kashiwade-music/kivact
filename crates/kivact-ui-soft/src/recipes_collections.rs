//! Role: Neutral recipes for standard navigation, split panes and collections.

use crate::Tokens;
use kivact::draw::LineCap;
use kivact::draw::Stroke;
use kivact::geometry::Point;
use kivact::geometry::Rect;
use kivact::interaction::SortOrder;
use kivact::interaction::TabState;
use kivact::interaction::TableHeaderState;
use kivact::motion::Transition;
use kivact::style::Align;
use kivact::style::Cursor;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::Style;
use kivact::style::Theme;
use kivact::style::TransitionProp;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::AxisState;
use kivact::ui::DataTableDensity;
use kivact::ui::DataTableRecipes;
use kivact::ui::DisclosureRecipes;
use kivact::ui::ListRecipes;
use kivact::ui::Look;
use kivact::ui::SplitRecipes;
use kivact::ui::SplitterRecipes;
use kivact::ui::TabsRecipes;
use kivact::ui::TreeRecipes;
use kivact::ui::TreeRowRecipeState;
use kivact::view::Orientation;
use kivact::view::ReadingDirection;
use kivact::view::use_direction;

fn paint_transition(p: &Tokens, duration: u64) -> Transition {
    Transition::tween(duration as f64, p.motion.ease_standard)
}

fn focus(p: &Tokens) -> Style {
    style().ring(p.sizes.focus_ring_width, p.colors.focus_ring)
}

pub(crate) fn tree(recipes: TreeRecipes) -> TreeRecipes {
    recipes
        .root(|_| |_: &Theme, _: &_| style().direction(Direction::Column).gap(2))
        .item(|_| tree_item)
        .marker(|_| tree_marker)
}
fn tree_item(theme: &Theme, row: &Look<TreeRowRecipeState>) -> Style {
    let p = theme.ext::<Tokens>();
    let appearance = row
        .variant::<crate::TreeLook>()
        .copied()
        .unwrap_or_default();
    let rtl = use_direction().get() == ReadingDirection::Rtl;
    let indent = 8 + row.level.saturating_sub(1) * 16;
    let mut look = style()
        .enter_size(Orientation::Vertical)
        .exit_size(Orientation::Vertical)
        .direction(if rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .position(Position::Relative)
        .align_items(Align::Center)
        .min_width(0)
        .gap(8)
        .padding_y(
            appearance
                .density
                .map_or(crate::ItemDensity::Balanced, |density| density.get())
                .padding(),
        )
        .padding_left(if rtl { 8 } else { indent })
        .padding_right(if rtl { indent } else { 8 })
        .radius(p.radius.element)
        .color(p.colors.text_primary)
        .font_size(14)
        .line_height(20.0 / 14.0)
        .hover(style().background(p.colors.overlay_hover))
        .selected(style().background(p.colors.accent_muted))
        .focus_visible(focus(&p));
    if appearance.guides.is_none_or(|guides| guides.get()) && !row.ancestor_last.is_empty() {
        let mut lines: Vec<_> = row
            .ancestor_last
            .iter()
            .copied()
            .skip(1)
            .map(|last| (!last).then_some(true))
            .collect();
        lines.push(Some(!row.is_last));
        let guide = p.colors.border_emphasized;
        look = look.paint(move |draw, size| {
            for (ancestor, extend) in lines.iter().enumerate() {
                if let Some(extend) = extend {
                    let inset = 16.0 + ancestor as f64 * 16.0;
                    let x = if rtl {
                        size.width - inset - 0.5
                    } else {
                        inset - 0.5
                    };
                    let height = if *extend {
                        size.height + 2.0
                    } else {
                        size.height
                    };
                    draw.rounded_rect(Rect::new(x, 0.0, 1.0, height), 0.5, guide);
                }
            }
        });
    }
    look
}
fn tree_marker(theme: &Theme, row: &Look<TreeRowRecipeState>) -> Style {
    let p = theme.ext::<Tokens>();
    let rtl = use_direction().get() == ReadingDirection::Rtl;
    let branch = row.branch.has_children() && !row.custom_marker;
    let angle = if row.branch.is_expanded() {
        if rtl { -90.0 } else { 90.0 }
    } else {
        0.0
    };
    let ink = p.colors.icon_secondary;
    style()
        .size(16, 16)
        .shrink(0)
        .font_size(if row.custom_marker { 14 } else { 0 })
        .color(ink)
        .rotate(angle)
        .transition(
            TransitionProp::Paint,
            Transition::tween(125.0, p.motion.ease_standard),
        )
        .paint(move |draw, _| {
            if branch {
                let x = if rtl {
                    [10.0, 6.0, 10.0]
                } else {
                    [6.0, 10.0, 6.0]
                };
                draw.polyline(
                    [
                        Point::new(x[0], 4.0),
                        Point::new(x[1], 8.0),
                        Point::new(x[2], 12.0),
                    ],
                    &Stroke::new(4.0 / 3.0)
                        .cap(LineCap::Round)
                        .join(kivact::draw::LineJoin::Round),
                    ink,
                );
            }
        })
}

pub(crate) fn tabs(r: TabsRecipes) -> TabsRecipes {
    r.list(|_| {
        |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
            style()
                .direction(if s.orientation == Orientation::Vertical {
                    Direction::Column
                } else {
                    Direction::Row
                })
                .gap(p.spacing.s0_5)
                .min_width(0)
        }
    })
    .tab(|_| tab)
    .indicator(|_| tab_indicator)
}

fn tab(t: &Theme, s: &Look<TabState>) -> Style {
    let p = t.ext::<Tokens>();
    let height = p.sizes.control_md;
    let ink = if s.selected {
        p.colors.text_primary
    } else {
        p.colors.text_secondary
    };
    let inset = p.spacing.s3;
    let mut tab = style()
        .position(Position::Relative)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .height(height)
        .shrink(0)
        .padding_x(inset)
        .gap(p.spacing.s1)
        .radius(p.radius.element)
        .font_size(p.typography.label.size_px)
        .line_height(p.typography.label.line_height_px / p.typography.label.size_px)
        .text_wrap(false)
        .color(ink)
        .weight(if s.selected {
            p.typography.weight_semibold
        } else {
            p.typography.weight_normal
        })
        .focus_visible(focus(&p))
        .transition(TransitionProp::Paint, paint_transition(&p, p.motion.fast));
    if !s.input.disabled {
        tab = tab
            .hover(style().background(p.colors.overlay_hover))
            .pressed(style().background(p.colors.overlay_pressed));
    }
    tab
}

fn tab_indicator(t: &Theme, s: &Look<TabState>) -> Style {
    let p = t.ext::<Tokens>();
    let inset = p.spacing.s3;
    let indicator = style()
        .position(Position::Absolute)
        .radius(1)
        .background(p.colors.accent)
        .opacity(if s.selected { 1.0 } else { 0.0 })
        .transition(
            TransitionProp::Paint,
            paint_transition(&p, p.motion.fast).essential(),
        );
    if s.orientation == Orientation::Vertical {
        let indicator = indicator.top(inset).bottom(inset).width(2);
        if use_direction().get() == ReadingDirection::Rtl {
            indicator.right(0)
        } else {
            indicator.left(0)
        }
    } else {
        indicator.left(inset).right(inset).bottom(-1).height(2)
    }
}

pub(crate) fn disclosure(r: DisclosureRecipes) -> DisclosureRecipes {
    r.root(|_| {
        |t: &Theme, s: &Look<kivact::ui::DisclosureRecipeState>| {
            let p = t.ext::<Tokens>();
            let variant = s
                .variant::<crate::DisclosureVariant>()
                .copied()
                .unwrap_or_default();
            let base = style().direction(Direction::Column).width(pct(100));
            if variant.dividers && s.group.is_some_and(|group| group.index > 0) {
                base.border_top(p.sizes.border_width, p.colors.border)
            } else {
                base
            }
        }
    })
    .trigger(|_| {
        |t: &Theme, s: &Look<kivact::ui::DisclosureRecipeState>| {
            let p = t.ext::<Tokens>();
            let variant = s
                .variant::<crate::DisclosureVariant>()
                .copied()
                .unwrap_or_default();
            let start = variant.chevron == crate::ChevronPosition::Start;
            let rtl = use_direction().get() == ReadingDirection::Rtl;
            style()
                .direction(if start != rtl {
                    Direction::RowReverse
                } else {
                    Direction::Row
                })
                .width(pct(100))
                .align_items(Align::Center)
                .justify_content(Justify::SpaceBetween)
                .gap(p.spacing.s2)
                .padding_y(disclosure_padding(&p, variant))
                .font_size(p.typography.large.size_px)
                .line_height(1.0)
                .weight(p.typography.weight_semibold)
                .color(p.colors.text_primary)
                .opacity(if s.input.disabled {
                    p.sizes.disabled_opacity
                } else {
                    1.0
                })
                .pressed(style().background(p.colors.overlay_pressed))
                .focus_visible(focus(&p))
                .transition(TransitionProp::Paint, paint_transition(&p, p.motion.fast))
        }
    })
    .indicator(|_| {
        |t: &Theme, s: &Look<kivact::ui::DisclosureRecipeState>| {
            let p = t.ext::<Tokens>();
            let variant = s
                .variant::<crate::DisclosureVariant>()
                .copied()
                .unwrap_or_default();
            let start = variant.chevron == crate::ChevronPosition::Start;
            let rtl = use_direction().get() == ReadingDirection::Rtl;
            let degrees = if start {
                if s.open { 0 } else { -90 }
            } else if s.open {
                180
            } else {
                0
            };
            style()
                .size(p.typography.large.size_px, p.typography.large.size_px)
                .shrink(0)
                .color(p.colors.icon_secondary)
                .scale_xy(if start && rtl { -1.0 } else { 1.0 }, 1.0)
                .rotate(degrees)
                .transition(TransitionProp::Paint, paint_transition(&p, p.motion.fast))
        }
    })
    .content(|_| {
        |t: &Theme, s: &Look<kivact::ui::DisclosureRecipeState>| {
            let p = t.ext::<Tokens>();
            let variant = s
                .variant::<crate::DisclosureVariant>()
                .copied()
                .unwrap_or_default();
            style()
                .direction(Direction::Column)
                .padding_top(p.spacing.s1)
                .padding_bottom(disclosure_padding(&p, variant))
                .font_size(p.typography.body.size_px)
                .line_height(p.typography.body.line_height_px / p.typography.body.size_px)
                .color(p.colors.text_primary)
        }
    })
}
fn disclosure_padding(p: &Tokens, variant: crate::DisclosureVariant) -> f64 {
    match variant.density.or(variant
        .dividers
        .then_some(crate::DisclosureDensity::Balanced))
    {
        Some(crate::DisclosureDensity::Compact) => p.spacing.s1,
        Some(crate::DisclosureDensity::Balanced) => p.spacing.s2,
        Some(crate::DisclosureDensity::Spacious) => p.spacing.s3,
        None => 0.0,
    }
}

pub(crate) fn splitter(r: SplitterRecipes) -> SplitterRecipes {
    r.handle(|_| {
        |t: &Theme, s: &Look<AxisState>| {
            let p = t.ext::<Tokens>();
            let hit = p.spacing.s4;
            let vertical = s.orientation == Orientation::Vertical;
            // Splitter has no grip child, while Split only builds one when its
            // caller opts in. Keep the shared handle visible as a divider.
            let divider = if s.input.pressed {
                p.colors.border_emphasized
            } else {
                p.colors.border
            };
            let line = p.sizes.border_width;
            let handle = style()
                .paint(move |draw, size| {
                    let rect = if vertical {
                        Rect::new(0.0, (size.height - line) / 2.0, size.width, line)
                    } else {
                        Rect::new((size.width - line) / 2.0, 0.0, line, size.height)
                    };
                    draw.fill_rect(rect, divider);
                })
                .direction(Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .shrink(0)
                .z_index(2)
                .focus_visible(focus(&p));
            if vertical {
                handle
                    .width(pct(100))
                    .height(hit)
                    .margin_y(-hit / 2.0)
                    .cursor(if s.input.disabled {
                        Cursor::Default
                    } else {
                        Cursor::NsResize
                    })
            } else {
                handle
                    .height(pct(100))
                    .width(hit)
                    .margin_x(-hit / 2.0)
                    .cursor(if s.input.disabled {
                        Cursor::Default
                    } else {
                        Cursor::EwResize
                    })
            }
        }
    })
}

pub(crate) fn split(r: SplitRecipes) -> SplitRecipes {
    r.grip(|_| {
        |t: &Theme, s: &Look<AxisState>| {
            let p = t.ext::<Tokens>();
            let long = p.spacing.s8;
            let grip = style()
                .shrink(0)
                .radius(p.radius.full)
                .background(if s.input.pressed {
                    p.colors.border_emphasized
                } else {
                    p.colors.border
                })
                .transition(TransitionProp::Paint, paint_transition(&p, p.motion.fast));
            if s.orientation == Orientation::Vertical {
                grip.size(long, 3).translate_y(-5.5)
            } else {
                grip.size(3, long)
                    .translate_x(if use_direction().get() == ReadingDirection::Rtl {
                        5.5
                    } else {
                        -5.5
                    })
            }
        }
    })
}

pub(crate) fn list(r: ListRecipes) -> ListRecipes {
    r.root(|prev| move |t: &Theme, s: &_| prev(t, s).gap(t.ext::<Tokens>().spacing.s0_5))
        .item(|_| {
            |t: &Theme, s: &_| {
                let p = t.ext::<Tokens>();
                let mut item = style()
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .gap(p.spacing.s2)
                    .padding(p.spacing.s2)
                    .radius(p.radius.element)
                    .font_size(p.typography.body.size_px)
                    .line_height(p.typography.body.line_height_px / p.typography.body.size_px)
                    .color(if s.input.disabled {
                        p.colors.text_disabled
                    } else {
                        p.colors.text_primary
                    })
                    .selected(style().background(p.colors.accent_muted))
                    .focus_visible(focus(&p))
                    .transition(
                        TransitionProp::Paint,
                        paint_transition(&p, p.motion.fast_min),
                    );
                if !s.input.disabled {
                    item = item
                        .hover(style().background(p.colors.overlay_hover))
                        .pressed(style().background(p.colors.overlay_pressed));
                }
                item
            }
        })
        .description(|_| {
            |t: &Theme, _: &_| {
                let p = t.ext::<Tokens>();
                style()
                    .font_size(p.typography.supporting.size_px)
                    .line_height(
                        p.typography.supporting.line_height_px / p.typography.supporting.size_px,
                    )
                    .color(p.colors.text_secondary)
            }
        })
}

pub(crate) fn data_table(r: DataTableRecipes) -> DataTableRecipes {
    r.frame(|_| {
        |_: &Theme, _: &_| {
            style()
                .direction(Direction::Column)
                .width(pct(100))
                .min_width(0)
                .shrink(0)
        }
    })
    .toolbar(|prev| {
        move |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
            prev(t, s)
                .gap(p.spacing.s2)
                .padding(0)
                .padding_bottom(p.spacing.s2)
        }
    })
    .header(|_| {
        |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
            table_cell(
                &p,
                s.variant::<DataTableDensity>().copied().unwrap_or_default(),
            )
            .gap(p.spacing.s1)
            .weight(p.typography.weight_semibold)
            .color(p.colors.text_secondary)
            .border_bottom(p.sizes.border_width, p.colors.border)
            .cursor(if s.sortable {
                Cursor::Pointer
            } else {
                Cursor::Default
            })
            .focus_visible(focus(&p))
        }
    })
    .sort(|_| sort_mark)
    .row(|prev| {
        move |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
            prev(t, s)
                .min_height(s.height.unwrap_or(0.0))
                .border_bottom(p.sizes.border_width, p.colors.border)
                .hover(style().background(p.colors.overlay_hover))
                .selected(style().background(p.colors.accent_muted))
                .transition(TransitionProp::Paint, paint_transition(&p, p.motion.fast))
        }
    })
    .group_header(|_| {
        |t: &Theme, _: &_| {
            let p = t.ext::<Tokens>();
            style()
                .direction(Direction::Row)
                .background(p.colors.bg_muted)
                .border_bottom(p.sizes.border_width, p.colors.border)
                .font_size(p.typography.body.size_px)
        }
    })
    .cell(|_| {
        |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
            table_cell(
                &p,
                s.variant::<DataTableDensity>().copied().unwrap_or_default(),
            )
            .justify_content(if s.numeric {
                Justify::End
            } else {
                Justify::Start
            })
            .color(p.colors.text_primary)
            .focus_visible(focus(&p))
        }
    })
    .check(|_| {
        |_: &Theme, _: &_| {
            style()
                .direction(Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .width(36)
                .shrink(0)
        }
    })
}

fn table_cell(p: &Tokens, density: DataTableDensity) -> Style {
    let (inline, block) = match density {
        DataTableDensity::Compact => (p.spacing.s2, p.spacing.s1),
        DataTableDensity::Balanced => (p.spacing.s3, p.spacing.s2),
        DataTableDensity::Spacious => (p.spacing.s4, p.spacing.s3),
    };
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .grow(1)
        .basis(0)
        .shrink(0)
        .min_width(0)
        .padding_x(inline)
        .padding_y(block)
        .font_size(p.typography.body.size_px)
        .line_height(1.5)
}

fn sort_mark(t: &Theme, s: &Look<TableHeaderState>) -> Style {
    let p = t.ext::<Tokens>();
    if !s.sortable {
        return style().size(0, 0).shrink(0);
    }
    let order = s.sort;
    let color = if order.is_some() {
        p.colors.icon_accent
    } else {
        p.colors
            .icon_secondary
            .with_alpha(if s.input.hovered || s.input.focused {
                1.0
            } else {
                0.35
            })
    };
    style()
        .size(p.spacing.s3, p.spacing.s3)
        .shrink(0)
        .paint(move |d, size| {
            let at = |x, y| Point::new(size.width * x, size.height * y);
            let stroke = Stroke::new(size.width * 0.08).cap(LineCap::Round);
            let mut arrow = |x, tip, tail, wing| {
                d.polyline([at(x, tail), at(x, tip)], &stroke, color);
                d.polyline(
                    [at(x - 0.2, wing), at(x, tip), at(x + 0.2, wing)],
                    &stroke,
                    color,
                );
            };
            match order {
                Some(SortOrder::Ascending) => arrow(0.5, 0.15, 0.85, 0.4),
                Some(SortOrder::Descending) => arrow(0.5, 0.85, 0.15, 0.6),
                None => {
                    arrow(0.25, 0.15, 0.85, 0.4);
                    arrow(0.75, 0.85, 0.15, 0.6);
                }
            }
        })
}
