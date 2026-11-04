//! Role: Shared arrangement layout, geometry, painting and semantic regions.
use super::gesture::State;
use crate::editors::arrangement::a11y::targets::Target;
use crate::editors::arrangement::model::TrackFlag;
use crate::editors::shared::metrics::{ARRANGEMENT_SIDEBAR as LEFT, ARRANGEMENT_TOP, LOOP_RULER};
use crate::editors::shared::model::drawable;
use crate::editors::shared::region_parts::bounded_range;
use kivact::draw::{
    Draw, Layer, LineCap, LineJoin, Path, Point, Rect, RegionId, Size, Stroke, TextStyle,
};
use kivact::style::{Color, ColorToken};

use super::layout::{automation_y, clamp_pan_y, track_layout};
pub(crate) fn paint(d: &mut Draw, size: Size, state: State) {
    let tracks = state.track_data.get();
    let data = state.data.get();
    let scale = state.pixels.get().clamp(state.bounds.0, state.bounds.1);
    let pan = state.offset.get();
    let pan_y = clamp_pan_y(&tracks, size.height, pan.y);
    let selected = state.selection.get();
    let regions = state.regions.get();
    let p = d.theme().get::<crate::tokens::Tokens>();
    d.fill_rect(size.to_rect(), p.canvas);
    let first = ((-pan.x / scale).floor().max(0.0)) as u64;
    let last = ((size.width - LEFT - pan.x) / scale)
        .ceil()
        .max(0.0)
        .min(state.length.get()) as u64;
    for b in first..=last {
        let x = LEFT + b as f64 * scale + pan.x;
        d.line(
            Point::new(x, ARRANGEMENT_TOP),
            Point::new(x, size.height),
            1.0,
            if b % 4 == 0 { p.hair_strong } else { p.hair },
        );
        if b % 4 == 0 {
            label(d, &format!("{}", b / 4 + 1), x + 4.0, 27.0, p.dim);
        }
    }
    d.fill_rect(Rect::new(0.0, 0.0, LEFT, size.height), p.header);
    label(d, &state.texts.tracks(tracks.len()), 8.0, 16.0, p.dim);
    let (lo, hi) = bounded_range(
        state.range.get(),
        state.length.get(),
        state.loop_min_duration,
    );
    let lx = LEFT + lo * scale + pan.x;
    let hx = LEFT + hi * scale + pan.x;
    d.rounded_rect(
        Rect::new(lx, 1.0, (hx - lx).max(1.0), 9.0),
        4.0,
        p.accent
            .with_alpha(if state.cycle.get() { 0.4 } else { 0.12 }),
    );
    ruler_region(
        d,
        regions.id(Target::LoopCreate),
        LEFT,
        size.width,
        size.width,
    );
    ruler_region(
        d,
        regions.id(Target::LoopBody),
        lx + 6.0,
        hx - 6.0,
        size.width,
    );
    for (target, x) in [(Target::LoopStart, lx), (Target::LoopEnd, hx)] {
        ruler_region(d, regions.id(target), x - 5.0, x + 5.0, size.width);
    }
    for marker in state.markers.get() {
        let x = LEFT + marker.beat * scale + pan.x;
        d.rounded_rect(
            Rect::new(x, 13.0, marker.label.len() as f64 * 6.0 + 8.0, 14.0),
            4.0,
            marker.color,
        );
        label(d, &marker.label, x + 3.0, 15.0, p.canvas);
    }
    d.layer(
        Layer::new().clip_rect(Rect::new(
            0.0,
            ARRANGEMENT_TOP,
            size.width,
            (size.height - ARRANGEMENT_TOP).max(0.0),
        )),
        |d| {
            for (i, (track, row)) in tracks.iter().zip(track_layout(&tracks, pan_y)).enumerate() {
                let mut y = row.top;
                let h = row.height;
                d.fill_rect(Rect::new(0.0, y, 3.0, h), track.color);
                label(
                    d,
                    &format!("{}   {}", i + 1, track.name),
                    12.0,
                    y + 8.0,
                    p.text,
                );
                for tag in TrackFlag::ALL {
                    let checked = tag.checked(track);
                    let x = 10.0
                        + TrackFlag::ALL
                            .iter()
                            .position(|candidate| *candidate == tag)
                            .expect("invariant: switch is in ALL") as f64
                            * 24.0;
                    d.rounded_rect(
                        Rect::new(x, y + 29.0, 20.0, 18.0),
                        4.0,
                        if checked {
                            track.color.with_alpha(0.4)
                        } else {
                            Color::token(ColorToken::Hover)
                        },
                    );
                    label(
                        d,
                        &state.texts.switch_letter(tag),
                        x + 6.0,
                        y + 33.0,
                        p.text,
                    );
                    if let Some(id) = regions.id(Target::Track(track.id, tag)) {
                        d.region(RegionId(id), Rect::new(x, y + 29.0, 20.0, 18.0));
                    }
                }
                for clip in data.iter().filter(|c| c.lane == track.id && drawable(c)) {
                    let r = Rect::new(
                        LEFT + clip.start * scale + pan.x,
                        y + 3.0,
                        clip.duration * scale,
                        h - 6.0,
                    );
                    let mut paint = clip.clone();
                    paint.muted |= track.muted;
                    crate::editors::time::paint::paint_item(
                        d,
                        &paint,
                        r,
                        selected.contains(&clip.id),
                    );
                    if let Some(id) = regions.id(Target::Item(clip.id)) {
                        d.region(RegionId(id), r);
                    }
                    if selected.contains(&clip.id) {
                        for (right, x) in [(false, r.x), (true, r.x + r.width - 6.0)] {
                            if let Some(id) = regions.id(Target::Trim(clip.id, right)) {
                                d.fill_rect(
                                    Rect::new(x, r.y + 15.0, 6.0, r.height - 15.0),
                                    p.text_high.with_alpha(0.4),
                                );
                                d.region(RegionId(id), Rect::new(x - 3.0, r.y, 12.0, r.height));
                            }
                        }
                    }
                }
                y += h;
                if let Some((top, height)) = row.automation {
                    if !track.automation_label.is_empty() {
                        label(d, &track.automation_label, 30.0, y + 10.0, p.dim);
                    }

                    let mut path = Path::new();
                    for (n, knot) in track
                        .automation
                        .iter()
                        .filter(|p| p.beat.is_finite() && p.value.is_finite())
                        .enumerate()
                    {
                        let point = Point::new(
                            LEFT + knot.beat * scale + pan.x,
                            automation_y(knot.value, top),
                        );
                        path = if n == 0 {
                            path.move_to(point)
                        } else {
                            path.line_to(point)
                        };
                        d.rounded_rect(
                            Rect::new(point.x - 3.5, point.y - 3.5, 7.0, 7.0),
                            3.5,
                            p.text,
                        );
                        if let Some(id) = regions.id(Target::Automation(track.id, knot.id)) {
                            d.region(
                                RegionId(id),
                                Rect::new(point.x - 7.0, point.y - 7.0, 14.0, 14.0),
                            );
                        }
                    }
                    d.stroke_path(
                        &path,
                        &Stroke::new(2.0).cap(LineCap::Round).join(LineJoin::Round),
                        track.color,
                    );
                    y = top + height;
                }
                d.line(Point::new(0.0, y), Point::new(size.width, y), 1.0, p.hair);
            }
        },
    );
    d.fill_rect(Rect::new(0.0, 0.0, LEFT, ARRANGEMENT_TOP), p.header);
    label(d, &state.texts.tracks(tracks.len()), 8.0, 16.0, p.dim);
    let x = LEFT + state.frame.get() * scale + pan.x;
    label(d, "+", LEFT - 22.0, 16.0, p.text);
    if let Some(id) = regions.id(Target::AddTrack) {
        d.region(RegionId(id), Rect::new(LEFT - 30.0, 8.0, 24.0, 24.0));
    }
    d.line(
        Point::new(x, LOOP_RULER),
        Point::new(x, size.height),
        1.0,
        p.text_high,
    );
    if let Some(id) = regions.id(Target::Playhead) {
        d.region(
            RegionId(id),
            Rect::new(
                LEFT,
                LOOP_RULER,
                (size.width - LEFT).max(0.0),
                ARRANGEMENT_TOP - LOOP_RULER,
            ),
        );
    }
    if let Some((a, b)) = state.marquee() {
        let r = Rect::new(
            a.x.min(b.x),
            a.y.min(b.y),
            (a.x - b.x).abs(),
            (a.y - b.y).abs(),
        );
        d.fill_rect(r, p.accent.with_alpha(0.12));
        d.stroke_rect(r, 1.0, p.accent);
    }
}

// Region geometry must be clipped explicitly, including its hit target.
fn ruler_region(d: &mut Draw, id: Option<u64>, left: f64, right: f64, width: f64) {
    if let Some(id) = id {
        let left = left.max(LEFT);
        let right = right.min(width);
        if right > left {
            d.region(RegionId(id), Rect::new(left, 0.0, right - left, LOOP_RULER));
        }
    }
}

fn label(d: &mut Draw, text: &str, x: f64, y: f64, color: Color) {
    d.text(
        text,
        Point::new(x, y),
        TextStyle {
            size: 10.5,
            color,
            ..Default::default()
        },
    );
}
