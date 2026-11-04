//! Role: Verify the table status lane, accessible markers and shared tooltip behavior.
use kivact::style::{ThemeMode, rgb};
use kivact::test::{RenderOptions, render_with};
use kivact::ui::{DataColumn, RowStatus, Tone, data_table};
use kivact::view::{Key, ReadingDirection, Role, provide_direction};
use std::time::Duration;

#[test]
fn row_status_has_a_named_tooltip_and_a_logical_leading_lane() {
    for direction in [ReadingDirection::Ltr, ReadingDirection::Rtl] {
        let screen = render_with(
            RenderOptions {
                theme: kivact_ui_soft::theme(ThemeMode::Light),
                ..RenderOptions::from_env()
            },
            move || {
                provide_direction(direction, || {
                    data_table(
                        vec!["Sync", "Review", "Unmarked"],
                        |row| *row,
                        vec![DataColumn::new("Job", |row: &&str| (*row).to_owned()).width(140.0)],
                    )
                    .row_status(|row| match *row {
                        "Sync" => Some(RowStatus::new("Failed", Tone::Danger)),
                        "Review" => {
                            Some(RowStatus::new("Waiting", Tone::Neutral).color(rgb(0x00490b)))
                        }
                        _ => None,
                    })
                })
            },
        );
        let lane = screen
            .get_by_role(Role::ColumnHeader, "Row status")
            .accessibility_bounds()
            .unwrap();
        let job = screen
            .get_by_role(Role::ColumnHeader, "Job")
            .accessibility_bounds()
            .unwrap();
        assert!((lane.width - 28.0).abs() < 1.0);
        assert_eq!(lane.x < job.x, direction == ReadingDirection::Ltr);
        let failed = screen.get_by_role(Role::Image, "Failed");
        let waiting = screen.get_by_role(Role::Image, "Waiting");
        assert!((failed.accessibility_bounds().unwrap().width - 12.0).abs() < 1.0);
        assert!((waiting.accessibility_bounds().unwrap().width - 8.0).abs() < 1.0);
        screen.get_by_role(Role::GridCell, "Unmarked");

        failed.hover();
        screen.advance(Duration::from_millis(1000));
        screen.get_by_role(Role::Tooltip, "Failed");
        failed.press_key(Key::Escape);
        assert!(screen.query_by_role(Role::Tooltip, "Failed").is_none());

        for _ in 0..8 {
            if waiting.is_focused() {
                break;
            }
            screen.press_tab(false);
        }
        assert!(waiting.is_focused());
        screen.advance(Duration::from_millis(1000));
        screen.get_by_role(Role::Tooltip, "Waiting");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
