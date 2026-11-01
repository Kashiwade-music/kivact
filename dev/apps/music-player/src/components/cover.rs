//! Role: Album art of the music player: embedded SVGs cropped to a square.
//!
//! The album art: an embedded SVG for each kind of track, cropped to a
//! square with `ImageFit::Cover` (the art is 3:2, the box 1:1).

use crate::audio::DEMO_TITLE;
use crate::player_state::PlayerCx;
use kivact::Asset;
use kivact::control::ImageFit;
use kivact::prelude::*;

/// Art of the generated demo (an original SVG).
static DEMO_COVER: &[u8] = include_bytes!("../../assets/demo-cover.svg");
/// Art of a loaded WAV file (an original SVG).
static FILE_COVER: &[u8] = include_bytes!("../../assets/file-cover.svg");

pub(super) fn cover() -> impl IntoView {
    let p = context::<PlayerCx>();
    let demo = memo(move || p.clip.with(|c| c.0.title == DEMO_TITLE));
    image(move || Asset::from(if demo.get() { DEMO_COVER } else { FILE_COVER }))
        .fit(ImageFit::Cover)
        .name("Album art")
        .test_id("album-art")
        .style(style().size(56, 56).shrink(0))
}
