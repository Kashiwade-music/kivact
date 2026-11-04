//! Role: Shared logical-time geometry and painting.
use super::gesture::{Drag, State, Target};
use super::layout::{item_rect, lane_rect};
use crate::editors::shared::metrics::{RULER, SIDEBAR};
use crate::editors::shared::model::Lane;
use crate::editors::shared::ruler::Ruler;
use crate::editors::shared::text::label;
use crate::editors::time::model::TimeItem;
use kivact::draw::{Draw, Layer, Point, Rect, Size, TextStyle};
pub(crate) fn paint_item<T: TimeItem>(draw: &mut Draw, item: &T, rect: Rect, selected: bool) {
    let p = draw.theme().get::<crate::tokens::Tokens>();
    let color = if selected { p.accent } else { item.color() }.with_alpha(if item.muted() {
        0.28
    } else {
        1.0
    });
    if item.duration() > 0.0 {
        draw.rounded_rect(rect, 5.0, color.with_alpha(0.3));
        draw.stroke_rect(rect, 1.0, color);
        label(draw, item.label(), rect.x + 6.0, rect.y + 4.0, 10.0, p.text);
    } else {
        item.glyph().paint(
            draw,
            rect.center(),
            5.0,
            if selected { p.accent } else { p.text },
        );
    }
}

pub(crate) fn paint_grid(
    draw: &mut Draw,
    size: Size,
    ruler: &Ruler,
    zoom: f64,
    pan: Point,
    lanes: &[Lane],
    layout: (f64, f64),
) {
    let (height, cells_width) = layout;
    let p = draw.theme().get::<crate::tokens::Tokens>();
    draw.fill_rect(Rect::new(0.0, 0.0, size.width, RULER), p.header);
    draw.fill_rect(
        Rect::new(0.0, RULER, SIDEBAR, (size.height - RULER).max(0.0)),
        p.header,
    );
    ruler.draw(draw, size, SIDEBAR, zoom, pan, RULER);
    for (index, lane) in lanes.iter().enumerate() {
        let rect = lane_rect(index, height);
        let y = rect.y;
        let text_height = draw
            .measure_text(
                &lane.label,
                &TextStyle {
                    size: 11.5,
                    ..Default::default()
                },
            )
            .size
            .height;
        draw.layer(
            Layer::new().clip_rect(Rect::new(
                rect.x,
                rect.y,
                (rect.width - cells_width).max(0.0),
                rect.height,
            )),
            |draw| {
                label(
                    draw,
                    &lane.label,
                    12.0,
                    y + (height - text_height) * 0.5,
                    11.5,
                    p.text,
                );
            },
        );
        draw.line(
            Point::new(0.0, y + height),
            Point::new(size.width, y + height),
            1.0,
            p.hair,
        );
    }
    draw.line(
        Point::new(SIDEBAR, 0.0),
        Point::new(SIDEBAR, size.height),
        1.0,
        p.hair_strong,
    );
}

/// Draw the current domain state and the typed regions it exposes.
pub(crate) fn paint<T: TimeItem>(
    draw: &mut Draw,
    size: Size,
    state: State<T>,
    ruler: &Ruler,
    cells_width: f64,
) {
    let scale = state.scale;
    let bounds = state.bounds;
    let offset = state.offset;
    let visible = state.visible;
    let lane_height = state.lane_height;
    let scene = state.scene;
    let data = state.data;
    let selection = state.selection;
    let frame = state.playhead;

    let z = scale.get().clamp(bounds.0, bounds.1);
    let offset = offset.get();
    let lanes = visible.get();
    paint_grid(
        draw,
        size,
        ruler,
        z,
        offset,
        &lanes,
        (lane_height, cells_width),
    );
    if let Ok(scene) = scene.get() {
        for (index, lane) in lanes.iter().enumerate() {
            if let Some(id) = scene.id(&Target::Lane(lane.id)) {
                draw.region(id, lane_rect(index, lane_height));
            }
        }
        for item in data.get() {
            if let Some(rect) = item_rect(&item, &lanes, lane_height, z, offset) {
                let on = selection.with(|s| s.contains(&item.id()));
                paint_item(draw, &item, rect, on);
                if let Some(id) = scene.id(&Target::Item(item.id())) {
                    draw.region(id, rect);
                }
                if on && item.duration() > 0.0 {
                    for right in [false, true] {
                        if let Some(id) = scene.id(&Target::Trim(item.id(), right)) {
                            draw.region(
                                id,
                                Rect::new(
                                    if right {
                                        rect.x + rect.width - 3.0
                                    } else {
                                        rect.x - 3.0
                                    },
                                    rect.y,
                                    12.0,
                                    rect.height,
                                ),
                            );
                        }
                    }
                }
            }
        }
        let x = SIDEBAR + (frame.get() * z) + offset.x;
        draw.line(
            Point::new(x, RULER),
            Point::new(x, size.height),
            1.0,
            draw.theme().get::<crate::tokens::Tokens>().text_high,
        );
        ruler.badge(draw, x, frame.get());
        if let Some(id) = scene.id(&Target::Playhead) {
            draw.region(
                id,
                Rect::new(SIDEBAR, 0.0, (size.width - SIDEBAR).max(0.0), RULER),
            );
        }
    }
    if let Some(Drag::Marquee(from, to)) = state.drag_state() {
        draw.stroke_rect(
            Rect::from_points(from, to),
            1.0,
            draw.theme().get::<crate::tokens::Tokens>().accent,
        );
    }
}
