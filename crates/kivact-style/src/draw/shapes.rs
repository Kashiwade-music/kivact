//! Role: Rectangle, line, path and shape commands of `Draw`.

use super::recorder::Draw;
use crate::effect::{Shape, basic};
use crate::geometry::{Point, Rect};
use crate::paint::Paint;
use crate::path::Path;
use kivact_types as p;
use kivact_types::{SceneCmd, Stroke};

impl Draw {
    /// Fill a rectangle with the element styles' paint vocabulary.
    pub fn fill_rect(&mut self, rect: Rect, fill: impl Into<Paint>) {
        let fill = self.fill_of(&fill.into());
        self.cmds.push(SceneCmd::Rect { rect, fill });
    }

    /// Stroke a rectangle with the same stroke and paint vocabulary as paths.
    pub fn stroke_rect(&mut self, rect: Rect, stroke: impl Into<Stroke>, paint: impl Into<Paint>) {
        let stroke = stroke.into();
        let paint = paint.into();
        if let Some(color) = paint.solid().filter(|_| plain_stroke(&stroke)) {
            self.cmds.push(SceneCmd::StrokeRect {
                rect,
                width: stroke.width,
                color: self.color(color),
                radius: 0.0,
            });
        } else {
            self.stroke_path(&Path::new().rect(rect), &stroke, paint);
        }
    }

    pub fn rounded_rect(&mut self, rect: Rect, radius: f32, fill: impl Into<Paint>) {
        let fill = self.fill_of(&fill.into());
        self.cmds.push(SceneCmd::RoundedRect { rect, radius, fill });
    }

    /// Stroke a line with the same stroke and paint vocabulary as paths.
    pub fn line(
        &mut self,
        from: Point,
        to: Point,
        stroke: impl Into<Stroke>,
        paint: impl Into<Paint>,
    ) {
        let stroke = stroke.into();
        let paint = paint.into();
        if let Some(color) = paint.solid().filter(|_| plain_stroke(&stroke)) {
            self.cmds.push(SceneCmd::Line {
                from,
                to,
                width: stroke.width,
                color: self.color(color),
            });
        } else {
            self.stroke_path(&Path::new().move_to(from).line_to(to), &stroke, paint);
        }
    }

    pub(super) fn painted_shape(
        &mut self,
        shape: p::DrawingShape,
        paint: Paint,
        stroke: Option<&Stroke>,
    ) {
        self.cmds.push(p::SceneCmd::Shape(Box::new(p::ShapePaint {
            shape,
            paint: self.fill_of(&paint),
            stroke: stroke.cloned(),
        })));
    }

    pub fn fill_path(&mut self, path: &Path, paint: impl Into<Paint>) {
        self.painted_shape(path.wire(), paint.into(), None);
    }

    pub fn stroke_path(&mut self, path: &Path, stroke: &Stroke, paint: impl Into<Paint>) {
        self.painted_shape(path.wire(), paint.into(), Some(stroke));
    }

    /// Percentages in the shape and paint resolve against `rect`.
    pub fn fill_shape(&mut self, rect: Rect, shape: &Shape, paint: impl Into<Paint>) {
        self.painted_shape(basic(rect, shape), paint.into(), None);
    }

    pub fn circle(&mut self, center: Point, radius: impl Into<f64>, paint: impl Into<Paint>) {
        let r = radius.into() as f32;
        self.fill_shape(
            Rect::new(center.x - r, center.y - r, r * 2.0, r * 2.0),
            &Shape::circle(r),
            paint,
        );
    }

    pub fn polyline(
        &mut self,
        points: impl IntoIterator<Item = Point>,
        stroke: &Stroke,
        paint: impl Into<Paint>,
    ) {
        let mut points = points.into_iter();
        let Some(first) = points.next() else { return };
        let path = points.fold(Path::new().move_to(first), |path, p| path.line_to(p));
        self.stroke_path(&path, stroke, paint);
    }
}

pub(super) fn plain_stroke(stroke: &Stroke) -> bool {
    stroke.cap == p::LineCap::Butt
        && stroke.join == p::LineJoin::Miter
        && stroke.miter_limit == 10.0
        && stroke.dash.is_empty()
        && stroke.dash_offset == 0.0
}
