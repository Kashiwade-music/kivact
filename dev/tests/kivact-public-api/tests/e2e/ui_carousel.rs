//! Role: Verify carousel selection through native keyboard and scrolling input.
use crate::support::mount;
use kivact::prelude::*;
use kivact::ui::{CarouselMode, carousel};
use std::time::Duration;
#[test]
fn carousel_moves_with_arrows_and_reports_the_index() {
    let (screen, index) = mount(|| {
        let index = signal(0usize);
        (
            index,
            carousel(index, (text("One"), text("Two"), text("Three")))
                .looping()
                .style(style().width(240).height(180)),
        )
    });
    screen.get_by_role(Role::Group, "Carousel").focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(index.get_untracked(), 1);
    screen.press_key(Key::ArrowLeft);
    screen.press_key(Key::ArrowLeft);
    assert_eq!(index.get_untracked(), 2);
    screen.get_by_role(Role::Button, "Next slide").click();
    assert_eq!(index.get_untracked(), 0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
#[test]
fn carousel_scroll_mode_snaps_to_a_slide() {
    let (screen, index) = mount(|| {
        let index = signal(0usize);
        (
            index,
            carousel(index, (text("One"), text("Two"), text("Three")))
                .mode(CarouselMode::Scroll)
                .items_per_view(1.0)
                .style(style().width(240).height(180)),
        )
    });
    screen.get_by_text("One").scroll(180.0, 0.0);
    screen.advance(Duration::from_millis(200));
    assert_eq!(index.get_untracked(), 1);
    screen.get_by_role(Role::Button, "Next slide").click();
    assert_eq!(index.get_untracked(), 2);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
