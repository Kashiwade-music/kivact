//! Role: Finite graph axes, curves and hit regions in domain coordinates.
use crate::editors::graph::gesture::{DragKind, Settings, State, Target};
use crate::editors::graph::layout::{Space, clipped};
use crate::editors::graph::model::Interpolation;

use crate::editors::shared::lane::{GRAPH_LANE_WIDTH, graph_lane_y};
use crate::editors::shared::metrics::{RULER, SIDEBAR};
use crate::editors::shared::model::Lane;
use crate::editors::shared::text::label;
use kivact::Value;
use kivact::draw::{Draw, Layer, Path, Point, Rect, Size, Stroke};

fn axes(draw: &mut Draw, space: Space, settings: &Settings) {
    let p = draw.theme().get::<crate::tokens::Tokens>();
    for vertical in [false, true] {
        let range = if vertical {
            space.view.value
        } else {
            space.view.time
        };
        let pixels = if vertical {
            space.plot.height
        } else {
            space.plot.width
        };
        let interval = settings.ruler.ticks.interval(pixels / (range.1 - range.0));
        let first = (range.0 / interval).ceil() * interval;
        // Bounded index iteration avoids non-progressing float addition at extreme ranges.
        for index in 0..256 {
            let value = first + f64::from(index) * interval;
            if !value.is_finite() || value > range.1 {
                break;
            }
            if vertical {
                let y = space.point(space.view.time.0, value).y;
                draw.line(
                    Point::new(SIDEBAR, y),
                    Point::new(SIDEBAR + space.plot.width, y),
                    1.0,
                    p.hair,
                );
                label(
                    draw,
                    &(settings.value_format)(value),
                    SIDEBAR - 68.0,
                    y - 12.0,
                    10.0,
                    p.text,
                );
            } else {
                let x = space.point(value, space.view.value.0).x;
                draw.line(
                    Point::new(x, RULER),
                    Point::new(x, RULER + space.plot.height),
                    1.0,
                    p.hair,
                );
                label(
                    draw,
                    &(settings.ruler.format)(value),
                    x + 3.0,
                    3.0,
                    10.0,
                    p.text,
                );
            }
        }
    }
}
pub(crate) fn paint(
    draw: &mut Draw,
    size: Size,
    state: State,
    settings: &Settings,
    lanes: Value<Vec<Lane>>,
) {
    let space = Space::new(state.view.get(), size);
    let p = draw.theme().get::<crate::tokens::Tokens>();
    axes(draw, space, settings);
    let Ok(scene) = state.scene.get() else { return };
    let keys = state.visible.get();
    let selected = state.selected.get();
    for (index, lane) in lanes.get().iter().enumerate() {
        let y = graph_lane_y(index);
        draw.layer(
            Layer::new().clip_rect(Rect::new(
                0.0,
                y,
                (GRAPH_LANE_WIDTH - settings.cells_width).max(0.0),
                crate::editors::shared::metrics::KEY_LANE,
            )),
            |draw| label(draw, &lane.label, 8.0, y, 11.0, p.text),
        );
        if let Some(id) = scene.id(&Target::Lane(lane.id)) {
            draw.region(
                id,
                Rect::new(
                    0.0,
                    y,
                    GRAPH_LANE_WIDTH,
                    crate::editors::shared::metrics::KEY_LANE,
                ),
            );
        }
    }
    draw.layer(Layer::new().clip_rect(space.plot), |draw| {
        for lane in lanes.get() {
            let curve: Vec<_> = keys.iter().filter(|key| key.curve == lane.id).collect();
            for segment in curve.windows(2) {
                let a = &segment[0].key;
                let b = &segment[1].key;
                let from = space.point(a.time, a.value);
                let to = space.point(b.time, b.value);
                let path = Path::new().move_to(from);
                let path = match a.interpolation {
                    Interpolation::Constant => {
                        path.line_to(space.point(b.time, a.value)).line_to(to)
                    }
                    Interpolation::Linear => path.line_to(to),
                    Interpolation::Bezier => path.cubic_to(
                        space.point(a.time + a.out_tangent.time, a.value + a.out_tangent.value),
                        space.point(b.time + b.in_tangent.time, b.value + b.in_tangent.value),
                        to,
                    ),
                };
                draw.stroke_path(&path, &Stroke::new(1.5), segment[0].color);
            }
        }
        for key in &keys {
            if !selected.contains(&key.key.id) {
                continue;
            }
            for incoming in [true, false] {
                let Some(id) = scene.id(&Target::Tangent(key.key.id, incoming)) else {
                    continue;
                };
                let tangent = if incoming {
                    key.key.in_tangent
                } else {
                    key.key.out_tangent
                };
                let point = space.point(key.key.time + tangent.time, key.key.value + tangent.value);
                draw.line(space.point(key.key.time, key.key.value), point, 1.0, p.text);
                let rect = Rect::new(point.x - 4.0, point.y - 4.0, 8.0, 8.0);
                draw.stroke_rect(rect, 1.0, p.accent);
                if let Some(rect) = clipped(rect, space.plot) {
                    draw.region(id, rect);
                }
            }
        }
        for key in &keys {
            let rect = space.key(key);
            draw.fill_rect(
                rect,
                if selected.contains(&key.key.id) {
                    p.accent
                } else {
                    key.color
                },
            );
            if let Some(id) = scene.id(&Target::Key(key.key.id))
                && let Some(rect) = clipped(rect, space.plot)
            {
                draw.region(id, rect);
            }
        }
        let x = space.point(state.playhead.get(), 0.0).x;
        draw.line(
            Point::new(x, RULER),
            Point::new(x, RULER + space.plot.height),
            1.0,
            p.text_high,
        );
        if let Some(gesture) = state.drag_state()
            && let DragKind::Marquee(from, to) = gesture.kind
        {
            draw.stroke_rect(Rect::from_points(from, to), 1.0, p.accent);
        }
    });
    if let Some(id) = scene.id(&Target::Playhead) {
        draw.region(id, Rect::new(SIDEBAR, 0.0, space.plot.width, RULER));
    }
}
