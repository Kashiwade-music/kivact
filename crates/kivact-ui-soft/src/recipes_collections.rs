//! Role: Neutral recipes for standard navigation, split panes and collections.

use crate::Tokens;
use kivact::draw::{LineCap, Point, Rect, Stroke};
use kivact::interaction::{SortOrder, TabState, TableHeaderState, TreeItemState};
use kivact::motion::Transition;
use kivact::style::{
    Align, Cursor, Direction, Justify, Position, Style, Theme, TransitionProp, pct, style,
};
use kivact::ui::{
    AxisState, DataTableDensity, DataTableRecipes, DisclosureRecipes, ListRecipes, Look,
    SplitRecipes, SplitterRecipes, TabsRecipes, TreeRecipes,
};
use kivact::view::{Orientation, ReadingDirection, current_direction};

fn paint_transition(p: &Tokens, duration: u64) -> Transition {
    Transition::tween(duration as f64, p.motion.ease_standard)
}

fn focus(p: &Tokens) -> Style {
    style().ring(p.sizes.focus_ring_width, p.colors.focus_ring)
}

pub(crate) fn tree(recipes: TreeRecipes) -> TreeRecipes {
    recipes
        .root(|_| |_: &Theme, _: &_| style().direction(Direction::Column).gap(2))
        .item(|_| {
            |theme: &Theme, row: &Look<TreeItemState>| {
                let tokens = theme.get::<Tokens>();
                let rtl = current_direction() == ReadingDirection::Rtl;
                let indent = 8 + row.level.saturating_sub(1) * 16;
                style()
                    .direction(if rtl {
                        Direction::RowReverse
                    } else {
                        Direction::Row
                    })
                    .align_items(Align::Center)
                    .min_width(0)
                    .gap(8)
                    .padding_y(crate::ItemDensity::Balanced.padding())
                    .padding_left(if rtl { 8 } else { indent })
                    .padding_right(if rtl { indent } else { 8 })
                    .radius(tokens.radius.element)
                    .color(tokens.colors.text_primary)
                    .hover(style().background(tokens.colors.overlay_hover))
                    .selected(style().background(tokens.colors.accent_muted))
                    .focus_visible(focus(&tokens))
            }
        })
}

pub(crate) fn tabs(r: TabsRecipes) -> TabsRecipes {
    r.list(|_| {
        |t: &Theme, s: &_| {
            let p = t.get::<Tokens>();
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
    let p = t.get::<Tokens>();
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
    let p = t.get::<Tokens>();
    let inset = p.spacing.s3;
    let indicator = style()
        .position(Position::Absolute)
        .radius(1)
        .background(p.colors.accent)
        .opacity(if s.selected { 1.0 } else { 0.0 })
        .transition(
            TransitionProp::Paint,
            paint_transition(&p, p.motion.fast).decorative(false),
        );
    if s.orientation == Orientation::Vertical {
        let indicator = indicator.top(inset).bottom(inset).width(2);
        if current_direction() == ReadingDirection::Rtl {
            indicator.right(0)
        } else {
            indicator.left(0)
        }
    } else {
        indicator.left(inset).right(inset).bottom(-1).height(2)
    }
}

pub(crate) fn disclosure(r: DisclosureRecipes) -> DisclosureRecipes {
    r.root(|_| |_: &Theme, _: &_| style().direction(Direction::Column).width(pct(100)))
        .trigger(|_| {
            |t: &Theme, s: &_| {
                let p = t.get::<Tokens>();
                style()
                    .direction(Direction::Row)
                    .width(pct(100))
                    .align_items(Align::Center)
                    .justify_content(Justify::SpaceBetween)
                    .gap(p.spacing.s2)
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
            |t: &Theme, s: &_| {
                let p = t.get::<Tokens>();
                let size = p.typography.large.size_px;
                style()
                    .size(size, size)
                    .shrink(0)
                    .color(p.colors.icon_secondary)
                    .rotate(if s.open { 180 } else { 0 })
                    .transition(TransitionProp::Paint, paint_transition(&p, p.motion.fast))
            }
        })
        .content(|_| {
            |t: &Theme, _: &_| {
                let p = t.get::<Tokens>();
                style()
                    .direction(Direction::Column)
                    .padding_top(p.spacing.s1)
                    .font_size(p.typography.body.size_px)
                    .line_height(p.typography.body.line_height_px / p.typography.body.size_px)
                    .color(p.colors.text_primary)
            }
        })
}

pub(crate) fn splitter(r: SplitterRecipes) -> SplitterRecipes {
    r.handle(|_| {
        |t: &Theme, s: &Look<AxisState>| {
            let p = t.get::<Tokens>();
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
            let p = t.get::<Tokens>();
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
                    .translate_x(if current_direction() == ReadingDirection::Rtl {
                        5.5
                    } else {
                        -5.5
                    })
            }
        }
    })
}

pub(crate) fn list(r: ListRecipes) -> ListRecipes {
    r.root(|prev| move |t: &Theme, s: &_| prev(t, s).gap(t.get::<Tokens>().spacing.s0_5))
        .item(|_| {
            |t: &Theme, s: &_| {
                let p = t.get::<Tokens>();
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
                let p = t.get::<Tokens>();
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
    // TODO(soft-core: C-29): expose the appendix's density and row index options.
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
            let p = t.get::<Tokens>();
            prev(t, s)
                .gap(p.spacing.s2)
                .padding(0)
                .padding_bottom(p.spacing.s2)
        }
    })
    .header(|_| {
        |t: &Theme, s: &_| {
            let p = t.get::<Tokens>();
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
            let p = t.get::<Tokens>();
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
            let p = t.get::<Tokens>();
            style()
                .direction(Direction::Row)
                .background(p.colors.bg_muted)
                .border_bottom(p.sizes.border_width, p.colors.border)
                .font_size(p.typography.body.size_px)
        }
    })
    .cell(|_| {
        |t: &Theme, s: &_| {
            let p = t.get::<Tokens>();
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
    let p = t.get::<Tokens>();
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
