//! Role: Tests for the PageDots primitive component.
//!
//! PageDots, a primitive written outside Kivact against its public API:
//! sized by its page count with its siblings after it, a press on a dot
//! (a hit region) and the arrow keys change the page, and each page is a
//! `Tab` leaf for assistive technology.

use std::cell::RefCell;
use std::rc::Rc;

use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::{Screen, render};
use kivact_external_component::page_dots;

fn mount<H: 'static, V: IntoView>(f: impl FnOnce() -> (H, V) + 'static) -> (Screen, H) {
    let slot: Rc<RefCell<Option<H>>> = Rc::new(RefCell::new(None));
    let s2 = slot.clone();
    let screen = render(move || {
        let (h, v) = f();
        *s2.borrow_mut() = Some(h);
        v
    });
    let h = slot.borrow_mut().take().expect("setup ran");
    (screen, h)
}

#[test]
fn page_dots_measure_by_page_count_and_change_page_by_press_and_keys() {
    let (screen, (count, page)) = mount(|| {
        let count = signal(4usize);
        let page = signal(0usize);
        let view = row((
            page_dots(count, page).name("pages").test_id("dots"),
            node()
                .test_id("after")
                .style(style().size(10, 10).shrink(0.0)),
        ))
        .style(style().align_items(Align::Start));
        ((count, page), view)
    });
    let dots = screen.get_by_test_id("dots");
    let r = dots.rect().expect("laid out");
    assert_eq!((r.width, r.height), (64.0, 16.0));
    assert_eq!(
        screen.get_by_test_id("after").rect().map(|a| a.x),
        Some(r.x + 64.0)
    );
    let list = screen.get_by_role(Role::TabList, "pages");
    assert_eq!(list.value().as_deref(), Some("Page 1 of 4"));
    screen.get_by_role(Role::Tab, "Page 3").click();
    assert_eq!(page.get_untracked(), 2);
    assert!(list.is_focused(), "a press focuses it");
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::ArrowRight);
    assert_eq!(page.get_untracked(), 3, "never past the last page");
    screen.press_key(Key::Home);
    assert_eq!(list.value().as_deref(), Some("Page 1 of 4"));
    count.set(6);
    screen.run_until_idle();
    assert_eq!(dots.rect().map(|r| r.width), Some(96.0));
    assert_eq!(
        screen.get_by_test_id("after").rect().map(|a| a.x),
        Some(r.x + 96.0)
    );
    screen.get_by_role(Role::Tab, "Page 6");
}
