//! Role: `Draw::border_image`: a nine-slice picture or gradient drawn into a rect.

use super::recorder::Draw;
use crate::geometry::Rect;
use crate::paint::BorderImage;
use kivact_types as p;

impl Draw {
    /// Draw `image` as a nine-slice into `rect`: its corners stay, its edges
    /// and middle stretch or repeat as the image says (CSS `border-image`).
    pub fn border_image(&mut self, rect: Rect, image: &BorderImage) {
        let image = image.wire(&self.theme);
        self.cmds.push(p::DrawCmd::BorderImage {
            rect,
            image: Box::new(image),
        });
    }
}
