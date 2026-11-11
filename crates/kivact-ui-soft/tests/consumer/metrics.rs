//! Role: Verify canonical metrics and Soft typography and metadata layouts.
use kivact::prelude::{Element, Role, ThemeMode, column, signal, text};
use kivact::test::{RenderOptions, render_with, render_with_handle};
use kivact::ui;
use kivact_ui_soft::{HeadingDisplay, MetadataColumns, MetadataLook};

#[test]
fn heading_has_its_level() {
    let screen = render_with(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)),
        || {
            column((
                ui::heading("Display", 2).variant(HeadingDisplay::D1),
                ui::heading("Section", 3),
            ))
        },
    );
    assert_eq!(
        screen.get_by_role(Role::Heading, "Display").level(),
        Some(2)
    );
    assert_eq!(
        screen.get_by_role(Role::Heading, "Section").level(),
        Some(3)
    );
    screen.assert_layout_clean();
}

#[test]
fn stat_exposes_label_and_value() {
    let (screen, value) = render_with_handle(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)),
        || {
            let value = signal("42 ms".to_owned());
            (
                value,
                ui::stat("Latency", value)
                    .delta("Improved")
                    .trend(ui::Trend::Down)
                    .icon(text("Clock"))
                    .chart(text("Recent samples")),
            )
        },
    );
    assert_eq!(
        screen
            .get_by_role(Role::Group, "Latency")
            .value()
            .as_deref(),
        Some("42 ms")
    );
    screen.get_by_text("Recent samples");
    value.set("30 ms".to_owned());
    screen.run_until_idle();
    assert_eq!(
        screen
            .get_by_role(Role::Group, "Latency")
            .value()
            .as_deref(),
        Some("30 ms")
    );
    screen.assert_layout_clean();
}

#[test]
fn progress_ring_reports_its_value() {
    let screen = render_with(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Dark)).reduced_motion(),
        || {
            ui::progress(Some(0.6))
                .name("Upload")
                .variant(ui::ProgressVariant::Ring)
        },
    );
    let (_, _, value) = screen
        .get_by_role(Role::ProgressBar, "Upload")
        .value_range();
    assert_eq!(value, Some(0.6));
    screen.assert_layout_clean();
}

#[test]
fn description_list_pairs_terms_and_details() {
    let (screen, columns) = render_with_handle(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)),
        || {
            let columns = signal(MetadataColumns::Single);
            let look = MetadataLook::default().columns(columns);
            (
                columns,
                ui::description_list((
                    ui::description("Owner", text("Ada"))
                        .variant(look)
                        .test_id("owner"),
                    ui::description("Current status", text("Ready"))
                        .variant(look)
                        .test_id("status"),
                ))
                .variant(look)
                .name("Project"),
            )
        },
    );
    let list = screen.get_by_role(Role::List, "Project");
    let pair = list.get_by_role(Role::ListItem, "Owner");
    let bounds = pair.rect().expect("boxless pair derives its child bounds");
    assert!(bounds.width > 0.0 && bounds.height > 0.0);
    assert_eq!(pair.accessibility_bounds(), Some(bounds));
    assert!(screen.accessibility_tree().contains("ListItem \"Owner\""));
    let detail = pair.get_by_role(Role::Group, "Owner");
    assert_eq!(detail.labelled_by()[0].text(), "Owner");
    assert_eq!(
        detail.get_by_text("Ada").rect().unwrap().x,
        list.get_by_role(Role::Group, "Current status")
            .get_by_text("Ready")
            .rect()
            .unwrap()
            .x
    );
    columns.set(MetadataColumns::Count(2));
    screen.run_until_idle();
    assert_eq!(
        list.get_by_role(Role::ListItem, "Owner")
            .get_by_role(Role::Group, "Owner")
            .labelled_by()[0]
            .text(),
        "Owner"
    );
    screen.assert_layout_clean();
}
