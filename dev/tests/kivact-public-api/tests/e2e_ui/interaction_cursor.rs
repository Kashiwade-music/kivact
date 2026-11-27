//! Role: Tests of a canvas whose cursor follows a signal its pointer sets.
//!
//! The cursor style reads a signal that `on_pointer` sets: the turn that
//! handles the pointer shows the cursor it asks for, with no turn between.

use kivact::prelude::*;
use kivact_public_api_tests::support::mount;

#[test]
fn the_cursor_a_pointer_handler_sets_is_shown_in_the_same_turn() {
    let (screen, ()) = mount(|| {
        let cursor = signal(Cursor::Default);
        let surface = canvas(|_draw, _size| {})
            .test_id("surface")
            .on_pointer(move |_| cursor.set(Cursor::Grab))
            .style(move || style().size(200, 200).cursor(cursor.get()));
        ((), surface)
    });
    let rect = screen.get_by_test_id("surface").rect().expect("laid out");
    screen.mouse_move(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    assert_eq!(screen.cursor(), Cursor::Grab);
}
