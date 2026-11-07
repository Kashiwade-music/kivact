//! Role: `Draw::drawable`: a view subtree (a drawable) drawn into a canvas at a point.

use super::recorder::Draw;
use crate::geometry::Point;
use kivact_types as p;
use kivact_types::DrawableId;

/// What `Draw::drawable` accepts; implemented by `kivact::view::Drawable`.
pub trait DrawableSource {
    /// The id of the drawable the source stands for.
    #[doc(hidden)]
    fn drawable_id(&self) -> DrawableId;
}

impl Draw {
    /// Draw a drawable's content with its top-left corner at `at`, under the
    /// open layers and transforms. Drawing the same drawable twice in one
    /// recording draws only the first and reports the second (as region errors).
    pub fn drawable(&mut self, drawable: impl DrawableSource, at: Point) {
        let id = drawable.drawable_id();
        if !self.drawn_drawables.insert(id) {
            self.region_errors
                .push(format!("drawable {} drawn twice", id.0));
            return;
        }
        self.cmds.push(p::DrawCmd::Drawable { id, at });
    }
}
