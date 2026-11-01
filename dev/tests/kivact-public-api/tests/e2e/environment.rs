//! Role: Tests for window environment and platform capabilities.
//!
//! The window's environment and platform as the App sees them (`kivact::platform`): the
//! safe area a phone's system UI covers and what the platform can do.

use std::cell::Cell;
use std::rc::Rc;

use kivact::platform::{self, Insets};
use kivact::prelude::*;
use kivact::style::Position;
use kivact::test::render;

/// The window's environment is one value, but a reader of one field runs
/// again only when that field changed.
#[test]
fn a_change_of_one_environment_field_notifies_only_its_readers() {
    let motion = Rc::new(Cell::new(0u32));
    let safe = Rc::new(Cell::new(0u32));
    let size = Rc::new(Cell::new(0u32));
    let screen = render({
        let (motion, safe, size) = (motion.clone(), safe.clone(), size.clone());
        move || {
            let (motion, safe, size) = (motion.clone(), safe.clone(), size.clone());
            column((
                text(move || {
                    motion.set(motion.get() + 1);
                    format!("motion {}", platform::reduced_motion())
                }),
                text(move || {
                    safe.set(safe.get() + 1);
                    format!("safe {}", platform::safe_area().top)
                }),
                text(move || {
                    size.set(size.get() + 1);
                    format!("size {}", platform::window_size().width)
                }),
            ))
        }
    });
    screen.get_by_text("motion false");
    screen.get_by_text("safe 0");
    let counts = |c: &[&Rc<Cell<u32>>]| c.iter().map(|c| c.get()).collect::<Vec<_>>();
    let start = counts(&[&motion, &safe, &size]);

    // Reduced motion changes: only its reader runs again.
    screen.set_reduced_motion(true);
    screen.get_by_text("motion true");
    assert_eq!(
        counts(&[&motion, &safe, &size]),
        [start[0] + 1, start[1], start[2]]
    );

    // The safe area changes: only its reader runs again.
    screen.set_safe_area(Insets {
        top: 24.0,
        ..Insets::default()
    });
    screen.get_by_text("safe 24");
    assert_eq!(
        counts(&[&motion, &safe, &size]),
        [start[0] + 1, start[1] + 1, start[2]]
    );

    // A field nothing reads (the theme) runs nobody; the same value again
    // changes nothing.
    screen.set_dark(true);
    screen.set_reduced_motion(true);
    assert_eq!(
        counts(&[&motion, &safe, &size]),
        [start[0] + 1, start[1] + 1, start[2]]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_safe_area_starts_at_zero_and_follows_the_platform() {
    let screen = render(|| {
        let safe = move || {
            let s = platform::safe_area();
            format!("safe {} {} {} {}", s.top, s.right, s.bottom, s.left)
        };
        column((
            text(safe).test_id("safe"),
            // Content kept below the status bar: its top follows the inset.
            node().test_id("bar").style(move || {
                style()
                    .position(Position::Absolute)
                    .top(platform::safe_area().top)
                    .left(0)
                    .size(100, 20)
            }),
        ))
    });
    screen.get_by_text("safe 0 0 0 0");
    let bar = screen
        .get_by_test_id("bar")
        .rect()
        .expect("the bar is laid out");
    assert_eq!(bar.y, 0.0);

    // A phone in portrait: a status bar with a notch above, a home
    // indicator below.
    screen.set_safe_area(Insets {
        top: 44.0,
        right: 0.0,
        bottom: 34.0,
        left: 0.0,
    });
    screen.get_by_text("safe 44 0 34 0");
    let bar = screen
        .get_by_test_id("bar")
        .rect()
        .expect("the bar is laid out");
    assert_eq!(bar.y, 44.0, "the bar moved below the status bar");

    // Rotated to landscape: the notch is on the left now.
    screen.set_safe_area(Insets {
        top: 0.0,
        right: 0.0,
        bottom: 21.0,
        left: 44.0,
    });
    screen.get_by_text("safe 0 0 21 44");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// What the platform can do is reported at start and readable by the App;
/// relying on what the platform lacks is noted instead of being silently
/// zero. The test window stands for a platform with touch, pens, a
/// software keyboard, a safe area and memory warnings (it records and
/// injects them); it draws on the CPU.
#[test]
fn the_platform_capabilities_are_reported_to_the_app() {
    let screen = render(|| {
        let caps = platform::capabilities();
        text(format!(
            "touch {} pen {} keyboard {} safe area {} memory {} reconversion {} gpu {}",
            caps.touch,
            caps.pen,
            caps.soft_keyboard,
            caps.safe_area,
            caps.memory_warnings,
            caps.ime_reconversion,
            caps.gpu
        ))
    });
    screen.get_by_text(
        "touch true pen true keyboard true safe area true memory true reconversion true gpu false",
    );
    // The safe area is reported here, so relying on it notes nothing.
    let screen = render(|| text(move || format!("top {}", platform::safe_area().top)));
    screen.get_by_text("top 0");
    let noted = screen
        .trace()
        .iter()
        .any(|turn| turn.notes.iter().any(|note| note.contains("safe area")));
    assert!(!noted, "{:?}", screen.trace());
}
