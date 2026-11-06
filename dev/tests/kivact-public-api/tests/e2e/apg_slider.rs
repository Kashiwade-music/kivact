//! Role: Verify APG orientation semantics on scalar and themed slider thumbs.
use crate::support::mount;
use kivact::interaction::{RangeThumb, use_range_scalar, use_scalar};
use kivact::prelude::*;
use kivact::ui::range_slider;

#[test]
fn slider_vertical_thumbs_expose_orientation() {
    let (screen, ()) = mount(|| {
        let scalar = use_scalar(signal(40.0), 0.0..=100.0);
        // Build the thumb first: its semantic orientation follows attachment
        // of the track, even when no track existed at thumb construction.
        let thumb = node().part(scalar.thumb()).name("Raw volume");
        let range = use_range_scalar(signal(20.0), signal(80.0), 0.0..=100.0);
        let low = node()
            .part(range.thumb(RangeThumb::Low))
            .name("Raw minimum");
        let high = node()
            .part(range.thumb(RangeThumb::High))
            .name("Raw maximum");
        let horizontal = use_scalar(signal(40.0), 0.0..=100.0);
        (
            (),
            row((
                node()
                    .part(scalar.track(Orientation::Vertical))
                    .style(style().size(24, 160))
                    .child(thumb),
                node()
                    .part(range.track(Orientation::Vertical))
                    .style(style().size(24, 160))
                    .child((low, high)),
                slider("Themed volume", signal(40.0))
                    .orientation(Orientation::Vertical)
                    .style(style().height(160)),
                range_slider("Themed range", signal(20.0), signal(80.0))
                    .orientation(Orientation::Vertical)
                    .style(style().height(160)),
                node()
                    .part(horizontal.track(Orientation::Horizontal))
                    .style(style().size(160, 24))
                    .child(node().part(horizontal.thumb()).name("Horizontal volume")),
            )),
        )
    });
    for name in [
        "Raw volume",
        "Raw minimum",
        "Raw maximum",
        "Themed volume",
        "Minimum",
        "Maximum",
    ] {
        assert_eq!(
            screen.get_by_role(Role::Slider, name).orientation(),
            Orientation::Vertical,
            "{name}"
        );
    }
    assert_eq!(
        screen
            .get_by_role(Role::Slider, "Horizontal volume")
            .orientation(),
        Orientation::Horizontal
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
