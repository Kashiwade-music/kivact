//! Role: Lay out Soft dialog and drawer headings.
use crate::color as c;
use crate::metrics as m;
use crate::motion::FAST;
use crate::recipe_parts::tween;
use crate::type_scale as t;

use kivact::component;
use kivact::state::Callback;
use kivact::state::Value;
use kivact::style::Align;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::button;
use kivact::view::Element;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::row;
use kivact::view::text;

/// Negative inset applied around the trailing header controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeCompensation {
    /// Compensate the inline end by eight pixels.
    Inline,
    /// Compensate the top and bottom by eight pixels.
    Block,
    /// Apply both inline and block compensation.
    All,
}

/// Dialog heading with optional subtitle, side content, divider and close action.
///
/// `on_close` adds a close button that calls the application callback.
#[component]
pub fn dialog_header(
    title: Value<String>,
    subtitle: Option<Value<String>>,
    header_start: Option<View>,
    header_end: Option<View>,
    header_end_compensation: Option<EdgeCompensation>,
    #[prop(default = false)] header_divider: Value<bool>,
    on_close: Option<Callback<()>>,
) -> impl Element {
    heading(
        title,
        subtitle,
        header_start,
        header_end,
        header_end_compensation,
        header_divider,
        on_close,
    )
}

/// Drawer heading: the same layout as a dialog heading. The enclosing drawer supplies its own name.
#[component]
pub fn drawer_header(
    title: Value<String>,
    subtitle: Option<Value<String>>,
    header_start: Option<View>,
    header_end: Option<View>,
    header_end_compensation: Option<EdgeCompensation>,
    #[prop(default = false)] header_divider: Value<bool>,
    on_close: Option<Callback<()>>,
) -> impl Element {
    heading(
        title,
        subtitle,
        header_start,
        header_end,
        header_end_compensation,
        header_divider,
        on_close,
    )
}

fn heading(
    title: Value<String>,
    subtitle: Option<Value<String>>,
    start: Option<View>,
    end: Option<View>,
    compensation: Option<EdgeCompensation>,
    divider: Value<bool>,
    close: Option<Callback<()>>,
) -> Node {
    let close = close.map(|close| {
        let label = kivact::i18n::message("soft.dialog.close", "Close");
        button("")
            .variant(ButtonVariant::Ghost)
            .name(label)
            .icon(icon("x").style(style().size(16, 16)))
            .style(|| tween(style().size(32, 32).padding(0).border_radius(10), FAST))
            .on_press(move || close.call(()))
    });
    let compensation = compensation.or_else(|| close.is_some().then_some(EdgeCompensation::All));
    let end = row((end, close)).style({
        let mut end = style().gap(8).align_items(Align::Center).flex_shrink(0);
        if matches!(
            compensation,
            Some(EdgeCompensation::Block | EdgeCompensation::All)
        ) {
            end = end.margin_top(-8).margin_bottom(-8);
        }
        if matches!(
            compensation,
            Some(EdgeCompensation::Inline | EdgeCompensation::All)
        ) {
            end = end.margin_right(-8);
        }
        end
    });
    let heading = text(title)
        .with_node(move |node| node.role(Role::Heading))
        .style(|| {
            style()
                .font_size(t::SIZE_XL)
                .line_height(1.4)
                .font_weight(t::WEIGHT_SEMIBOLD)
                .color(c::TEXT_PRIMARY)
        })
        .into_view();
    let subtitle = subtitle.map(|subtitle| {
        text(subtitle)
            .style(|| {
                style()
                    .font_size(t::SIZE_SM)
                    .line_height(20.0 / 14.0)
                    .color(c::TEXT_SECONDARY)
            })
            .into_view()
    });
    let title = column((heading, subtitle)).style(
        style()
            .flex_grow(1)
            .min_width(0)
            .margin_top(-6)
            .margin_bottom(-6),
    );
    let start = start.map(|start| row(start).style(style().gap(8).flex_shrink(0)));
    row((start, title, end)).style(move || {
        let base = style().gap(12).align_items(Align::Start);
        if divider.get() {
            base.border_bottom(m::BORDER_WIDTH, c::BORDER)
        } else {
            base
        }
    })
}
