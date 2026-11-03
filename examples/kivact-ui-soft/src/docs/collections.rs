//! Role: Demonstrate the tabs, disclosure, list, table and split recipes.
use super::examples::specimen;
use kivact::control::{Node, NodeBuilder, text};
use kivact::interaction::ListActivation;
use kivact::signal;
use kivact::style::style;
use kivact::ui::{DataColumn, ListItem, data_table, disclosure, list, split, tabs};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "tab-list" | "tab" => specimen(
            "Manual activation",
            tabs(
                "Project views",
                vec![
                    ListItem::new("summary", "Summary"),
                    ListItem::new("activity", "Activity"),
                    ListItem::new("settings", "Settings"),
                ],
                signal("summary"),
            )
            .activation(ListActivation::Manual)
            .content(|key| text(format!("Selected view: {key}"))),
            "tabs(\"Project views\", options, selected).activation(ListActivation::Manual)",
        ),
        "collapsible" | "collapsible-group" => specimen(
            "Open by default",
            disclosure("Delivery details", signal(true), || {
                text("Ships in two business days.")
            }),
            "disclosure(\"Delivery details\", signal(true), details)",
        ),
        "list" | "list-item" => specimen(
            "Selectable list",
            list(
                "Workspace",
                vec![
                    ListItem::new("design", "Design"),
                    ListItem::new("docs", "Documentation"),
                    ListItem::new("archive", "Archive").disabled(true),
                ],
                signal(Some("design")),
            )
            .style(style().width(240)),
            "list(\"Workspace\", options, selected)",
        ),
        "table" => specimen(
            "Sortable columns",
            data_table(
                vec![
                    ("Design", "Active"),
                    ("Documentation", "Review"),
                    ("Archive", "Closed"),
                ],
                |row| row.0,
                vec![
                    DataColumn::new("Project", |row: &(&str, &str)| row.0.to_owned()).sortable(),
                    DataColumn::new("Status", |row: &(&str, &str)| row.1.to_owned()),
                ],
            )
            .style(style().width(440)),
            "data_table(rows, |row| row.0, columns)",
        ),
        "resizable" | "resize-handle" => specimen(
            "Split panes",
            split(
                (text("Navigation"), text("Content")),
                signal(vec![140.0, 200.0]),
            )
            .style(style().width(340).height(72)),
            "split((navigation, content), sizes)",
        ),
        _ => return None,
    })
}
