//! Role: Table consumers share keyboard movement, sorting and row selection.
use kivact::prelude::*;
use kivact::test::RenderOptions;

#[test]
fn table_stage3_keyboard_sort_and_selection() {
    let (screen, selected) = kivact::test::render_with_handle(
        RenderOptions {
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let selected = signal(Vec::<u64>::new());
            let view = super::lines(
                &["Name"],
                signal(vec![super::line(1, ["Beta"]), super::line(2, ["Alpha"])]),
                selected,
            );
            (selected, view)
        },
    );
    screen.get_by_role(Role::ColumnHeader, "Name").focus();
    screen.press_key(Key::Enter);
    let alpha = screen.get_by_role(Role::Row, "Alpha");
    let beta = screen.get_by_role(Role::Row, "Beta");
    assert!(
        alpha.rect().expect("table cell laid out").y < beta.rect().expect("table cell laid out").y
    );
    alpha.click();
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Row, "Beta").is_focused());
    screen.press_key(Key::ArrowUp);
    assert_eq!(selected.get_untracked(), vec![2]);
    screen.get_by_role(Role::ColumnHeader, "Name").focus();
    screen.press_key(Key::Enter);
    assert!(
        screen
            .get_by_role(Role::Row, "Beta")
            .rect()
            .expect("table cell laid out")
            .y
            < screen
                .get_by_role(Role::Row, "Alpha")
                .rect()
                .expect("table cell laid out")
                .y
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn table_resize_escape_restores_the_keyboard_adjusted_width() {
    use kivact::view::MouseButton;
    let (screen, widths) = kivact::test::render_with_handle(
        RenderOptions {
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let widths = signal(vec![150.0, 120.0]);
            (
                widths,
                super::lines(
                    &["Name", "Value"],
                    signal(vec![super::line(1, ["Layer", "On"])]),
                    signal(Vec::new()),
                )
                .resizable(widths),
            )
        },
    );
    let resize = screen.get_by_role(Role::Splitter, "Resize Name");
    resize.press_key(Key::ArrowRight);
    assert_eq!(widths.get_untracked(), vec![160.0, 120.0]);
    let rect = resize.rect().expect("resize handle layout");
    let (x, y) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 30.0, y);
    assert_eq!(widths.get_untracked(), vec![190.0, 120.0]);
    screen.press_key(Key::Escape);
    assert_eq!(widths.get_untracked(), vec![160.0, 120.0]);
    screen.mouse_move(x + 40.0, y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(widths.get_untracked(), vec![160.0, 120.0]);
    assert_eq!(
        screen
            .get_by_role(Role::ColumnHeader, "Name")
            .rect()
            .expect("header layout")
            .width,
        160.0
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
