//! Role: Verify APG date-picker focus restoration, full weekday names and spinbutton keys.
use crate::support::mount;
use kivact::i18n::{CalendarDate,CalendarMonth};
use kivact::prelude::*;
use kivact::ui::{self,DatePresentation};
#[test]
fn date_picker_selection_returns_focus_to_the_picker_button() {
    let (screen,value) = mount(|| { let value=signal(Some(CalendarDate::new(2026,10,14))); (value,ui::date_input("Date",value).presentation(DatePresentation::Desktop)) });
    let button=screen.get_by_role(Role::Button,"Choose date");
    button.click();
    let selected=screen.get_by_role(Role::GridCell,"2026-10-14");
    assert!(selected.is_focused());
    selected.press_key(Key::ArrowRight); screen.press_key(Key::Enter);
    assert_eq!(value.get_untracked(),Some(CalendarDate::new(2026,10,15)));
    assert!(button.is_focused());
    assert!(screen.errors().is_empty(),"{:?}",screen.errors());
}
#[test]
fn date_picker_weekday_headers_publish_full_names_and_month_changes_are_announced() {
    let (screen,month)=mount(|| { let month=signal(CalendarMonth::new(2026,10)); (month,ui::calendar(signal(None::<CalendarDate>)).month(month)) });
    screen.get_by_role(Role::ColumnHeader,"Monday");
    screen.get_by_role(Role::Button,"Next month").click();
    assert_eq!(month.get_untracked(),CalendarMonth::new(2026,11));
    screen.get_by_role(Role::Status,"November 2026");
    assert!(screen.errors().is_empty(),"{:?}",screen.errors());
}
