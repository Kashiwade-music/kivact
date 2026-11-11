//! Role: Tests for the delayed show and hide machine.
//!
//! `use_delayed`: it shows after the open delay and hides after the close
//! delay, `dismiss` keeps it hidden until the condition goes false, and `pulse`
//! shows at once and restarts the close delay.

use std::time::Duration;

use crate::support::mount;
use kivact::interaction::{DelayedApi, Delays, use_delayed};
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::View;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn button(name: &'static str) -> Node {
    node()
        .role(Role::Button)
        .name(name)
        .test_id(name)
        .focusable(true)
        .style(style().size(100, 30).shrink(0))
        .child(text(name))
}

fn card_shown(screen: &Screen) -> bool {
    screen.query_by_text("Card").is_some()
}

/// The card, a button that dismisses it and one that pulses it.
fn page(wanted: Signal<bool>, delays: Delays) -> View {
    let card: DelayedApi = use_delayed(wanted, delays);
    column((
        button("Dismiss").on_press(move || card.hide()),
        button("Pulse").on_press(move || card.pulse()),
        show(card.is_open(), || text("Card")),
    ))
    .into_view()
}

#[test]
fn shows_after_the_open_delay_and_hides_after_the_close_delay() {
    let delays = Delays {
        open: ms(300),
        close: ms(200),
    };
    let (screen, wanted) = mount(move || {
        let wanted = signal(false);
        (wanted, page(wanted, delays))
    });
    assert!(!card_shown(&screen));
    wanted.set(true);
    screen.advance(ms(299));
    assert!(!card_shown(&screen), "not before the open delay");
    screen.advance(ms(1));
    assert!(card_shown(&screen));
    wanted.set(false);
    screen.advance(ms(199));
    assert!(
        card_shown(&screen),
        "still shown while the close delay runs"
    );
    screen.advance(ms(1));
    assert!(!card_shown(&screen));
    // A visit shorter than the open delay shows nothing.
    wanted.set(true);
    screen.advance(ms(100));
    wanted.set(false);
    screen.advance(ms(500));
    assert!(!card_shown(&screen));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn dismiss_keeps_it_hidden_until_wanted_goes_false() {
    let delays = Delays {
        open: Duration::ZERO,
        close: Duration::ZERO,
    };
    let (screen, wanted) = mount(move || {
        let wanted = signal(false);
        (wanted, page(wanted, delays))
    });
    wanted.set(true);
    screen.advance(ms(1));
    assert!(card_shown(&screen));
    screen.get_by_test_id("Dismiss").click();
    assert!(!card_shown(&screen), "dismissed at once");
    screen.advance(ms(500));
    assert!(!card_shown(&screen), "stays hidden while wanted");
    wanted.set(false);
    screen.advance(ms(1));
    wanted.set(true);
    screen.advance(ms(1));
    assert!(card_shown(&screen), "wanted again after going false");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pulse_shows_at_once_and_a_second_pulse_restarts_the_close_delay() {
    let delays = Delays {
        open: Duration::ZERO,
        close: ms(200),
    };
    let (screen, ()) = mount(move || ((), page(signal(false), delays)));
    let pulse = screen.get_by_test_id("Pulse");
    assert!(!card_shown(&screen));
    pulse.click();
    assert!(card_shown(&screen), "shown at once");
    screen.advance(ms(150));
    pulse.click();
    screen.advance(ms(150));
    assert!(card_shown(&screen), "the second pulse restarted the delay");
    screen.advance(ms(50));
    assert!(
        !card_shown(&screen),
        "hidden a close delay after the last pulse"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
