//! Role: The standard dialog and select drawn in the Lyra look by the kit theme.

use super::assert_clean;
use kivact::prelude::*;

fn page(child: impl IntoView) -> Node {
    node().style(style().padding(16)).child(child)
}

#[test]
fn the_standard_dialog_puts_the_description_under_the_title_and_focuses_the_content() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        let open = signal(false);
        (
            (),
            page((
                kivact::ui::button("Edit profile").on_press(move || open.set(true)),
                kivact::ui::dialog("Edit profile", open)
                    .description("Make changes to your profile here.")
                    .content(|| kivact::ui::text_input("Name", signal(String::new())))
                    .footer(|| kivact::ui::button("Save changes")),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Edit profile").click();
    assert!(
        screen.get_by_role(Role::TextBox, "Name").is_focused(),
        "the content's first control, not the close button"
    );
    let rect = |e: kivact::test::Element<'_>| e.rect().expect("laid out");
    let title = rect(screen.get_by_role(Role::Heading, "Edit profile"));
    let description = rect(screen.get_by_text("Make changes to your profile here."));
    // `gap-1` between the title and the description.
    assert_eq!(description.y - (title.y + title.height), 4.0);
    let panel = rect(screen.get_by_role(Role::Dialog, "Edit profile"));
    let close = rect(screen.get_by_role(Role::Button, "Close"));
    // `absolute top-2 right-2`.
    assert_eq!(close.y - panel.y, 8.0);
    assert_eq!(panel.x + panel.width - (close.x + close.width), 8.0);
    assert_clean(&screen);
}

#[test]
fn the_select_list_is_at_least_as_wide_as_its_trigger() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        let fruit = signal(None::<u32>);
        (
            (),
            page(
                kivact::ui::select(
                    "Fruit",
                    vec![
                        kivact::ui::ListItem::new(1, "Fig"),
                        kivact::ui::ListItem::new(2, "Kiwi"),
                    ],
                    fruit,
                )
                .placeholder("Select a fruit")
                .style(style().width(300)),
            ),
        )
    });
    let trigger = screen.get_by_role(Role::ComboBox, "Fruit");
    trigger.click();
    let list = screen.get_by_role(Role::ListBox, "Fruit");
    assert_eq!(list.rect().expect("laid out").width, 300.0);
    assert_clean(&screen);
}
