//! Role: Verify native carousel differences from the WAI-ARIA APG carousel pattern.
//! Source: https://www.w3.org/WAI/ARIA/apg/patterns/carousel/
//! Native semantics lack role descriptions, so group names include Carousel/Slide.
//! The focusable group adds arrow navigation; controls retain native button activation.
use crate::support::mount;
use kivact::prelude::*;
use kivact::ui::carousel;
#[test]
fn carousel_named_native_groups_replace_role_descriptions() {
    let (screen, ()) = mount(|| {
        (
            (),
            carousel(signal(0usize), (text("First"), text("Second")))
                .style(style().width(240).height(180)),
        )
    });
    assert!(screen.get_by_role(Role::Group, "Carousel").is_visible());
    assert!(screen.get_by_role(Role::Group, "Slide 1 / 2").is_visible());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
#[test]
fn carousel_group_adds_arrows_and_buttons_keep_focus() {
    let (screen, index) = mount(|| {
        let index = signal(0usize);
        (
            index,
            carousel(index, (text("First"), text("Second"), text("Third")))
                .looping()
                .style(style().width(240).height(180)),
        )
    });
    screen.get_by_role(Role::Group, "Carousel").focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(index.get_untracked(), 1);
    let next = screen.get_by_role(Role::Button, "Next slide");
    next.focus();
    screen.press_key(Key::Enter);
    assert_eq!(index.get_untracked(), 2);
    assert!(next.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
