//! Role: Verify accessible metrics, headings and term-detail associations.
use crate::support::mount;
use kivact::prelude::*;
use kivact::ui::{self, ProgressVariant, Trend};

#[derive(Debug, PartialEq)]
struct CustomLook;
impl ui::Variant for CustomLook {}

#[test]
fn progress_ring_reports_its_value() {
    let (screen, value) = mount(|| {
        let value = signal(Some(0.7));
        (
            value,
            column((
                ui::progress(value)
                    .variant(CustomLook)
                    .variant(ProgressVariant::Ring)
                    .name("Upload"),
                ui::progress(Some(1.5))
                    .variant(ProgressVariant::Gauge)
                    .name("Capacity"),
                ui::progress(None::<f64>)
                    .variant(ProgressVariant::Ring)
                    .name("Preparing"),
            ))
            .style(style().width(240).gap(12)),
        )
    });
    let ring = screen.get_by_role(Role::ProgressBar, "Upload");
    assert_eq!(ring.value_range(), (Some(0.0), Some(1.0), Some(0.7)));
    assert_eq!(ring.value().as_deref(), Some("70%"));
    ring.get_by_text("70%");
    assert_eq!(
        screen
            .get_by_role(Role::ProgressBar, "Capacity")
            .value_range()
            .2,
        Some(1.0)
    );
    assert_eq!(
        screen
            .get_by_role(Role::ProgressBar, "Preparing")
            .value_range(),
        (None, None, None)
    );
    value.set(Some(0.9));
    screen.run_until_idle();
    let ring = screen.get_by_role(Role::ProgressBar, "Upload");
    assert_eq!(ring.value_range().2, Some(0.9));
    ring.get_by_text("90%");
    screen.assert_layout_clean();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn stat_exposes_label_and_value() {
    let (screen, (label, value, delta, trend)) = mount(|| {
        let label = signal("Revenue".to_owned());
        let value = signal("$120".to_owned());
        let delta = signal("+12%".to_owned());
        let trend = signal(Trend::Up);
        (
            (label, value, delta, trend),
            column((
                ui::stat(label, value)
                    .delta(delta)
                    .trend(trend)
                    .icon(text("Metric icon"))
                    .chart(text("Chart samples"))
                    .variant(CustomLook),
                ui::stat("Formatted", "42 ms")
                    .label_content(text("Latency label"))
                    .value_content(text("42 milliseconds")),
            )),
        )
    });
    let stat = screen.get_by_role(Role::Group, "Revenue");
    assert_eq!(stat.value().as_deref(), Some("$120"));
    stat.get_by_text("Revenue");
    stat.get_by_text("$120");
    stat.get_by_text("+12%");
    stat.get_by_text("Metric icon");
    stat.get_by_text("Chart samples");
    let formatted = screen.get_by_role(Role::Group, "Formatted");
    assert_eq!(formatted.value().as_deref(), Some("42 ms"));
    formatted.get_by_text("Latency label");
    formatted.get_by_text("42 milliseconds");
    label.set("Costs".to_owned());
    value.set("$90".to_owned());
    delta.set("-25%".to_owned());
    trend.set(Trend::Down);
    screen.run_until_idle();
    let stat = screen.get_by_role(Role::Group, "Costs");
    assert_eq!(stat.value().as_deref(), Some("$90"));
    stat.get_by_text("-25%");
    screen.assert_layout_clean();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn heading_has_its_level() {
    let (screen, title) = mount(|| {
        let title = signal("Overview".to_owned());
        (
            title,
            column((
                ui::heading(title, 2).variant(CustomLook),
                ui::heading("First", 0),
                ui::heading("Last", 9),
            )),
        )
    });
    assert_eq!(
        screen.get_by_role(Role::Heading, "Overview").level(),
        Some(2)
    );
    assert_eq!(screen.get_by_role(Role::Heading, "First").level(), Some(1));
    assert_eq!(screen.get_by_role(Role::Heading, "Last").level(), Some(6));
    title.set("Details".to_owned());
    screen.run_until_idle();
    assert_eq!(
        screen.get_by_role(Role::Heading, "Details").level(),
        Some(2)
    );
    screen.assert_layout_clean();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn description_list_pairs_terms_and_details() {
    let (screen, term) = mount(|| {
        let term = signal("Owner".to_owned());
        (
            term,
            ui::description_list((
                ui::description(term, text("Ada")).variant(CustomLook),
                ui::description("Action", button("Open profile"))
                    .term_content(text("Profile actions")),
            ))
            .name("Project")
            .variant(CustomLook),
        )
    });
    let list = screen.get_by_role(Role::List, "Project");
    let pair = list.get_by_role(Role::ListItem, "Owner");
    let detail = pair.get_by_role(Role::Group, "Owner");
    detail.get_by_text("Ada");
    let labels = detail.labelled_by();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].text(), "Owner");
    list.get_by_role(Role::ListItem, "Action")
        .get_by_role(Role::Button, "Open profile");
    let actions = list
        .get_by_role(Role::ListItem, "Action")
        .all_by_role(Role::Group, "Action")
        .into_iter()
        .find(|group| group.query_by_role(Role::Button, "Open profile").is_some())
        .expect("detail group contains the profile action");
    let action_labels = actions.labelled_by();
    assert_eq!(action_labels.len(), 1);
    assert_eq!(action_labels[0].name(), "Action");
    assert_eq!(
        action_labels[0].get_by_text("Profile actions").text(),
        "Profile actions",
    );
    term.set("Maintainer".to_owned());
    screen.run_until_idle();
    let detail = screen
        .get_by_role(Role::ListItem, "Maintainer")
        .get_by_role(Role::Group, "Maintainer");
    assert_eq!(detail.labelled_by()[0].text(), "Maintainer");
    detail.get_by_text("Ada");
    screen.assert_layout_clean();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
