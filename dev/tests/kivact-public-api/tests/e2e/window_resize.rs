//! Role: Desktop platform requests use the ordinary recorded window service.
use kivact::{platform, prelude::*, test::render};
#[test]
fn desktop_size_request_is_recorded_in_logical_pixels() {
    let screen = render(|| {
        spawn(platform::request_window_size(420.5, 280));
        text("Resize requested")
    });
    let sizes = screen.requested_sizes();
    assert_eq!(sizes.len(), 1);
    assert_eq!((sizes[0].width, sizes[0].height), (420.5, 280.0));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
