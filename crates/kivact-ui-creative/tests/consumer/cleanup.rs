//! Role: Real-window contracts for optional action builders and reactive collections.
use super::*;

#[test]
fn split_button_propagates_reactive_disabled_to_both_owned_actions() {
    let (screen, (disabled, primary, chosen)) =
        kivact::test::render_with_handle(crate::options(), || {
            let disabled = signal(true);
            let primary = signal(0usize);
            let chosen = signal(0usize);
            (
                (disabled, primary, chosen),
                ui::split_button(
                    "Run",
                    move || primary.update(|n| *n += 1),
                    vec![MenuEntry::item(1u8, "Option")],
                )
                .on_select(move |_| chosen.update(|n| *n += 1))
                .disabled(disabled),
            )
        });
    let run = screen.get_by_role(Role::Button, "Run");
    let options = screen.get_by_role(Role::Button, "Command options");
    assert!(run.is_disabled());
    assert!(options.is_disabled());
    run.force_click();
    options.force_click();
    assert_eq!(primary.get_untracked(), 0);
    assert_eq!(chosen.get_untracked(), 0);
    assert!(screen.query_by_role(Role::MenuItem, "Option").is_none());
    disabled.set(false);
    screen.run_until_idle();
    run.click();
    assert_eq!(primary.get_untracked(), 1);
    options.click();
    screen.get_by_role(Role::MenuItem, "Option").click();
    assert_eq!(chosen.get_untracked(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pie_entries_update_while_open_without_reusing_or_closing_popup_content() {
    let (screen, (entries, state, chosen)) =
        kivact::test::render_with_handle(crate::options(), || {
            let entries = signal(vec![ui::PieEntry::new("Before", 0., "settings")]);
            let state = signal(ui::PieState::open_at(Point::new(320., 240.), false));
            let chosen = signal(None::<usize>);
            (
                (entries, state, chosen),
                ui::pie_menu(entries, state, move |index| chosen.set(Some(index))),
            )
        });
    screen.get_by_role(Role::Button, "Before");
    entries.set(vec![
        ui::PieEntry::new("After", 90., "settings"),
        ui::PieEntry::new("Other", 180., "settings"),
    ]);
    screen.run_until_idle();
    assert!(state.get_untracked().open);
    assert!(screen.query_by_role(Role::Button, "Before").is_none());
    screen.get_by_role(Role::Button, "After");
    screen.get_by_role(Role::Button, "Other").click();
    assert_eq!(chosen.get_untracked(), Some(1));
    assert!(!state.get_untracked().open);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
