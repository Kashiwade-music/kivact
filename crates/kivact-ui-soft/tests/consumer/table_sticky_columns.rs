//! Role: Verify that pinned DataTable cells remain at the scroll edges.
use kivact::style::ThemeMode;
use kivact::style::style;
use kivact::test::RenderOptions;
use kivact::test::render_with;
use kivact::ui::DataColumn;
use kivact::ui::DataTableDensity;
use kivact::ui::data_table;
use kivact::view::Element;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::provide_direction;

#[test]
fn sticky_columns_follow_both_edges_in_ltr_and_rtl() {
    for direction in [ReadingDirection::Ltr, ReadingDirection::Rtl] {
        let screen = render_with(
            RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)),
            move || {
                provide_direction(direction, || {
                    let columns = [
                        ("A", 52.0),
                        ("B", 68.0),
                        ("Middle", 460.0),
                        ("D", 64.0),
                        ("E", 56.0),
                    ];
                    data_table(
                        vec![1],
                        |key| *key,
                        columns
                            .into_iter()
                            .map(|(name, width)| {
                                DataColumn::new(name, move |key| format!("{name} {key}"))
                                    .width(width)
                            })
                            .collect(),
                    )
                    .sticky_columns(vec![0, 1], vec![3, 4])
                    .density(DataTableDensity::Compact)
                    .style(style().width(400))
                })
            },
        );
        let headers = ["A", "B", "D", "E"].map(|name| screen.get_by_role(Role::ColumnHeader, name));
        let cells =
            ["A 1", "B 1", "D 1", "E 1"].map(|name| screen.get_by_role(Role::GridCell, name));
        let middle = screen.get_by_role(Role::ColumnHeader, "Middle");
        let before = headers
            .each_ref()
            .map(|item| item.accessibility_bounds().unwrap().x);
        let middle_before = middle.accessibility_bounds().unwrap().x;
        let target = if direction == ReadingDirection::Ltr {
            &headers[0]
        } else {
            &headers[3]
        };
        target.scroll(100.0, 0.0);
        let middle_after = middle.accessibility_bounds().unwrap().x;
        assert!(
            middle_after < middle_before - 20.0,
            "{direction:?}: middle column did not scroll"
        );
        for (index, (head, cell)) in headers.iter().zip(&cells).enumerate() {
            let x = head.accessibility_bounds().unwrap().x;
            assert!(
                (x - before[index]).abs() < 2.0,
                "{direction:?}: pinned header moved"
            );
            assert!(
                (x - cell.accessibility_bounds().unwrap().x).abs() < 2.0,
                "{direction:?}: cell drifted from header"
            );
        }
        let after = headers
            .each_ref()
            .map(|item| item.accessibility_bounds().unwrap().x);
        if direction == ReadingDirection::Ltr {
            assert!((after[1] - after[0] - 52.0).abs() < 2.0);
            assert!((after[3] - after[2] - 64.0).abs() < 2.0);
        } else {
            assert!((after[0] - after[1] - 68.0).abs() < 2.0);
            assert!((after[2] - after[3] - 56.0).abs() < 2.0);
        }
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn soft_sticky_paint_uses_neutral_surface_and_edge_shadow() {
    use kivact::style::rgba;
    for (mode, alpha) in [
        (ThemeMode::Light, 26.0 / 255.0),
        (ThemeMode::Dark, 77.0 / 255.0),
    ] {
        let theme = kivact_ui_soft::theme(mode);
        let paint = theme
            .ext::<kivact::ui::UiTokens>()
            .data_table
            .sticky
            .clone();
        let tokens = theme.ext::<kivact_ui_soft::Tokens>();
        assert_eq!(paint.surface, tokens.colors.bg_card);
        assert_eq!(paint.hover, tokens.colors.overlay_hover);
        assert_eq!(paint.selected, tokens.colors.accent_muted);
        assert_eq!(paint.shadow, rgba(0, alpha));
        assert_eq!(paint.shadow_width, 6.0);
        assert!(paint.shadow_transition.is_some());
        assert!(paint.overlay_transition.is_some());
    }
}
