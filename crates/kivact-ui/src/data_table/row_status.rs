//! Role: Named status markers in the table's leading accessory lane.
use crate::tooltip;
use kivact_core::i18n::message;
use kivact_core::state::Value;
use kivact_core::style::{Align, Icon, Justify, style, use_theme};
use kivact_core::view::{IntoView, Role, View, dynamic};
use kivact_core::view::{Node, icon, node};
use kivact_interaction::Tone;

use crate::data_table::types::RowStatus;

fn lane() -> Node {
    node().style(
        style()
            .width(28)
            .min_width(28)
            .flex_shrink(0)
            .align_items(Align::Center)
            .justify_content(Justify::Center),
    )
}

pub(super) fn header() -> Node {
    let header_key = *use_theme()
        .get()
        .ext::<crate::data_table::look::DataTableTheme>()
        .row_status_header_key_value();
    lane()
        .role(Role::ColumnHeader)
        .name(message(header_key, "Row status"))
}

pub(super) fn cell(status: Value<Option<RowStatus>>, role: Role) -> (Node, View) {
    (
        lane().role(role),
        dynamic(status, |status| status.map(marker)).into_view(),
    )
}

fn marker(status: RowStatus) -> View {
    let RowStatus {
        label,
        tone,
        icon: custom_icon,
        color,
    } = status;
    let default_icon = if color.is_some() {
        None
    } else {
        match tone {
            Tone::Success => Some("circle-check"),
            Tone::Warning => Some("triangle-alert"),
            Tone::Danger => Some("circle-x"),
            _ => None,
        }
    };
    let glyph = custom_icon.or_else(|| default_icon.map(Icon::from));
    let has_icon = glyph.is_some();
    let mark = node()
        .role(Role::Image)
        .name(label.clone())
        .focusable(true)
        .child(glyph.map(icon))
        .style(move || {
            let theme = use_theme().get();
            let color = color.unwrap_or(match tone {
                Tone::Neutral => theme.get(&crate::data_table::look::ROW_STATUS_NEUTRAL_VAR),
                Tone::Info => theme.get(&crate::data_table::look::ROW_STATUS_INFO_VAR),
                Tone::Success => theme.get(&crate::data_table::look::ROW_STATUS_SUCCESS_VAR),
                Tone::Warning => theme.get(&crate::data_table::look::ROW_STATUS_WARNING_VAR),
                Tone::Danger => theme.get(&crate::data_table::look::ROW_STATUS_DANGER_VAR),
                Tone::Accent => theme.get(&crate::data_table::look::ROW_STATUS_ACCENT_VAR),
            });
            if has_icon {
                style().size(12, 12).color(color)
            } else {
                style().size(8, 8).border_radius(4).background(color)
            }
        });
    tooltip(label).attach(mark)
}
