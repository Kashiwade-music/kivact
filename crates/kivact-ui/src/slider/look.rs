//! Role: The slider recipe state, recipe table and default looks.

use crate::control_size::ControlSize;
use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::custom::PartState;
use kivact_core::style::{Cursor, Style, Theme, style};
use kivact_core::view::Orientation;
/// The thickness of a track and the side of a thumb.
pub(crate) fn slider_metrics(t: &Theme, size: ControlSize) -> (f64, f64) {
    let thumb = match size {
        ControlSize::ExtraSmall | ControlSize::Small => {
            (t.get(&kivact_core::style::font::SIZE_VAR)).round()
        }
        ControlSize::Medium | ControlSize::Large => {
            (t.get(&kivact_core::style::font::SIZE_VAR) + 4.0).round()
        }
    };
    ((thumb / 3.0).round().max(2.0), thumb)
}

/// The track: a thin rounded bar as tall as the thumb, so the thumb fits
/// it; 160 px long when vertical.
pub(crate) fn slider_track(t: &Theme, state: &Look<AxisState>) -> Style {
    let (bar, thumb) = slider_metrics(t, state.size);
    let s = style().flex_shrink(0).cursor(Cursor::Pointer);
    // The bar is the box's middle third; its painted fill keeps the box's
    // full thumb-sized extent for the pointer.
    let s = s.paint(move |d, size| {
        let r = bar / 2.0;
        let rect = if size.width >= size.height {
            kivact_core::geometry::Rect::new(0.0, (size.height - bar) / 2.0, size.width, bar)
        } else {
            kivact_core::geometry::Rect::new((size.width - bar) / 2.0, 0.0, bar, size.height)
        };
        d.rounded_rect(rect, r, kivact_core::style::color::SURFACE_ALT);
    });
    match state.orientation {
        Orientation::Vertical => s.width(thumb).height(160),
        _ => s.height(thumb).min_width(80),
    }
}

/// The filled part of the track, in the accent.
pub(crate) fn slider_range(t: &Theme, state: &Look<AxisState>) -> Style {
    let (bar, _) = slider_metrics(t, state.size);
    style().paint(move |d, size| {
        let rect = if size.width >= size.height && size.height > 0.0 {
            kivact_core::geometry::Rect::new(0.0, (size.height - bar) / 2.0, size.width, bar)
        } else {
            kivact_core::geometry::Rect::new((size.width - bar) / 2.0, 0.0, bar, size.height)
        };
        d.rounded_rect(rect, bar / 2.0, kivact_core::style::color::ACCENT);
    })
}

/// A round thumb on the surface, ringed in the accent.
pub(crate) fn slider_thumb(t: &Theme, state: &Look<AxisState>) -> Style {
    let (_, thumb) = slider_metrics(t, state.size);
    style()
        .size(thumb, thumb)
        .border_radius(thumb / 2.0)
        .background(kivact_core::style::color::SURFACE)
        .border(2, kivact_core::style::color::ACCENT)
        .shadow(kivact_core::style::Shadow::new(
            0,
            1,
            2,
            kivact_core::style::color::SHADOW,
        ))
}

/// The state of a part laid along an axis, for its recipe: a slider's
/// track, range and thumb, and a splitter's handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AxisState {
    /// The actual input state of the part (the thumb's and the handle's
    /// own; the track's and the range's are the track's).
    pub input: PartState,
    /// The direction the value grows along: a horizontal slider's track is
    /// a row, a vertical one's a column. A splitter's is the direction its
    /// panes are laid out in, so the handle between panes in a row is a
    /// vertical bar.
    pub orientation: Orientation,
    /// The size the caller chose with `.size(..)` (default `Medium`).
    pub size: ControlSize,
}

recipe_table! {
    /// The recipes of a [`slider`](super::slider) and a
    /// [`range_slider`](super::range_slider).
    SliderTheme {
        /// The track the pointer moves along; the component places the
        /// range and the thumbs in it.
        track(track_recipe): AxisState = slider_track,
        /// The filled part of the track, from its start (or the first
        /// thumb) to the thumb.
        range(range_recipe): AxisState = slider_range,
        /// A thumb. Its size is the recipe's; the component places it.
        thumb(thumb_recipe): AxisState = slider_thumb,
    }
}
