//! Role: Verify Window Splitter collapse/restore and primary-pane relations.
use crate::support::mount;
use kivact::interaction::{ScalarRange, use_splitter};
use kivact::prelude::*;
use kivact::view::EditPhase;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn window_splitter_enter_collapses_and_restores_the_previous_position() {
    let phases = Rc::new(RefCell::new(Vec::new()));
    let seen = phases.clone();
    let (screen, value) = mount(move || {
        let value = signal(40.0);
        let divider = node().name("Primary").style(style().size(8, 100)).part(
            use_splitter(
                value,
                ScalarRange::new(0.0, 100.0).unwrap(),
                Orientation::Horizontal,
            )
            .reset_to(Some(25.0))
            .on_edit(move |phase| seen.borrow_mut().push(phase))
            .handle(),
        );
        (value, divider)
    });
    screen.get_by_role(Role::Splitter, "Primary").focus();
    screen.press_key(Key::Enter);
    assert_eq!(
        value.get_untracked(),
        0.0,
        "Enter collapses instead of resetting"
    );
    screen.press_key(Key::Enter);
    assert_eq!(
        value.get_untracked(),
        40.0,
        "Enter restores the previous size"
    );
    screen.press_key(Key::ArrowRight);
    assert_eq!(value.get_untracked(), 41.0);
    screen.press_key(Key::Enter);
    screen.press_key(Key::Enter);
    assert_eq!(
        value.get_untracked(),
        41.0,
        "a new collapse records the latest size"
    );
    assert_eq!(
        *phases.borrow(),
        vec![
            EditPhase::Begin,
            EditPhase::Commit,
            EditPhase::Begin,
            EditPhase::Commit,
            EditPhase::Begin,
            EditPhase::Commit,
            EditPhase::Begin,
            EditPhase::Commit,
            EditPhase::Begin,
            EditPhase::Commit
        ]
    );
    screen.press_key(Key::Home);
    assert_eq!(value.get_untracked(), 0.0);
    screen.press_key(Key::Enter);
    assert_eq!(
        value.get_untracked(),
        41.0,
        "Home collapse also restores the previous position"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn window_splitter_controls_the_primary_pane() {
    let (screen, ()) = mount(|| {
        let size = signal(120.0);
        let sizes = signal(vec![0.4, 0.6]);
        (
            (),
            column((
                kivact::ui::splitter("Files", size, ScalarRange::new(0.0, 300.0).unwrap())
                    .start(text("File contents"))
                    .end(text("Editor"))
                    .style(style().size(320, 100)),
                kivact::ui::split(sizes, (text("First pane"), text("Second pane")))
                    .style(style().size(320, 100)),
            )),
        )
    });
    let splitter = screen.get_by_role(Role::Splitter, "Files");
    let panes = splitter.controls();
    assert_eq!(panes.len(), 1);
    assert_eq!(panes[0].name(), "Files");
    assert_eq!(
        panes[0].get_by_text("File contents").text(),
        "File contents"
    );
    let grouped = screen.get_by_role(Role::Splitter, "Resize").controls();
    assert_eq!(
        grouped.len(),
        1,
        "fractional split controls its preceding pane"
    );
    assert_eq!(grouped[0].get_by_text("First pane").text(), "First pane");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
