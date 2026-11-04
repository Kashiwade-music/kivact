//! Role: Named status markers in the table's leading accessory lane.
use crate::tooltip;
use kivact_core::i18n::message;
use kivact_core::state::Value;
use kivact_core::style::{Align, Color, Icon, Justify, style, use_theme};
use kivact_core::view::{IntoView, Role, View, dynamic};
use kivact_core::view::{Node, icon, node};
use kivact_interaction::Tone;

// xtask-allow(state-fields): Immutable presentation descriptor; optional icon and color are independent overrides.
/// A status announced by name and shown in the table's leading lane.
#[derive(Clone, Debug, PartialEq)]
pub struct RowStatus {
    label: String,
    tone: Tone,
    icon: Option<Icon>,
    color: Option<Color>,
}

impl RowStatus {
    /// Create a marker in the theme's color for this tone.
    pub fn new(label: impl Into<String>, tone: Tone) -> Self {
        Self {
            label: label.into(),
            tone,
            icon: None,
            color: None,
        }
    }

    /// Use an explicit color instead of the theme's tone.
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Display a named or custom icon instead of the default mark.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

fn lane() -> Node {
    node().style(
        style()
            .width(28)
            .min_width(28)
            .shrink(0)
            .align_items(Align::Center)
            .justify_content(Justify::Center),
    )
}

pub(super) fn header() -> Node {
    let paint = use_theme()
        .get()
        .ext::<crate::UiTokens>()
        .data_table
        .row_status
        .clone();
    lane()
        .role(Role::ColumnHeader)
        .name(message(paint.header_key, "Row status"))
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
            let paint = use_theme()
                .get()
                .ext::<crate::UiTokens>()
                .data_table
                .row_status
                .clone();
            let color = color.unwrap_or(match tone {
                Tone::Neutral => paint.neutral,
                Tone::Info => paint.info,
                Tone::Success => paint.success,
                Tone::Warning => paint.warning,
                Tone::Danger => paint.danger,
                Tone::Accent => paint.accent,
            });
            if has_icon {
                style().size(12, 12).color(color)
            } else {
                style().size(8, 8).radius(4).background(color)
            }
        });
    tooltip(label).attach(mark)
}
