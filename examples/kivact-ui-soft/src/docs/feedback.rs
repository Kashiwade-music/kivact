//! Role: Demonstrate Soft badges, placeholders and status indicators.
use super::specimen::specimen;
use kivact::style::{Align, style};
use kivact::ui::badge;
use kivact::ui::{ControlSize, skeleton, spinner};
use kivact::view::{Node, NodeBuilder, column, row, text};
use kivact_ui_soft::{
    BadgeVariant, SkeletonRadius, SpinnerShade, SpinnerVariant, StatusDotVariant, TokenColor,
    status_dot, token,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "badge" => example_badge(),
        "skeleton" => example_skeleton(),
        "status-dot" => example_status_dot(),
        "spinner" => example_spinner(),
        "token" => example_token(),
        "banner" => example_banner(),
        _ => return None,
    })
}

fn example_badge() -> Node {
    specimen(
        "Semantic and categorical colors",
        row((
            badge("Neutral"),
            badge("Info").tone(kivact::interaction::Tone::Info),
            badge("Success").tone(kivact::interaction::Tone::Success),
            badge("Warning").tone(kivact::interaction::Tone::Warning),
            badge("Error").tone(kivact::interaction::Tone::Danger),
            badge("Design").variant(BadgeVariant::Purple),
        ))
        .style(style().gap(8).wrap(true)),
        "badge(\"Success\").tone(kivact::interaction::Tone::Success)",
    )
}

fn example_skeleton() -> Node {
    specimen(
        "Staggered placeholders",
        row((
            skeleton()
                .variant(SkeletonRadius::Rounded)
                .style(style().size(40, 40)),
            column((
                skeleton().index(1).style(style().size(160, 14)),
                skeleton().index(2).style(style().size(112, 12)),
            ))
            .style(style().gap(8)),
        ))
        .style(style().align_items(Align::Center).gap(12)),
        "skeleton().variant(SkeletonRadius::Rounded).style(style().size(40, 40))",
    )
}

fn example_status_dot() -> Node {
    specimen(
        "Status and activity",
        column((
            row((
                status_dot(StatusDotVariant::Success, "Online"),
                text("Online"),
            ))
            .style(style().align_items(Align::Center).gap(8)),
            row((
                status_dot(StatusDotVariant::Warning, "Processing").pulsing(true),
                text("Processing"),
            ))
            .style(style().align_items(Align::Center).gap(8)),
            row((
                status_dot(StatusDotVariant::Error, "Offline"),
                text("Offline"),
            ))
            .style(style().align_items(Align::Center).gap(8)),
        ))
        .style(style().gap(8)),
        "status_dot(StatusDotVariant::Warning, \"Processing\").pulsing(true)",
    )
}

fn example_spinner() -> Node {
    specimen(
        "Sizes and shades",
        row((
            spinner().size(ControlSize::ExtraSmall),
            spinner().size(ControlSize::Small).variant(SpinnerVariant {
                shade: SpinnerShade::Subtle,
                ..Default::default()
            }),
            spinner()
                .size(ControlSize::Medium)
                .name("Loading")
                .label_content(text("Loading")),
        ))
        .style(style().align_items(Align::Center).gap(16)),
        "spinner().size(ControlSize::Medium).name(\"Loading\").label_content(text(\"Loading\"))",
    )
}

fn example_token() -> Node {
    specimen(
        "Categorical tokens",
        row((
            token("Neutral"),
            token("Engineering").color(TokenColor::Blue),
        ))
        .style(style().gap(8)),
        "token(\"Engineering\").color(TokenColor::Blue)",
    )
}

fn example_banner() -> Node {
    specimen(
        "Status messages",
        column((
            kivact::ui::alert("Workspace updated")
                .tone(kivact::interaction::Tone::Info)
                .icon(kivact_ui_soft::banner_icon(kivact::interaction::Tone::Info)),
            kivact::ui::alert("Review required")
                .tone(kivact::interaction::Tone::Warning)
                .icon(kivact_ui_soft::banner_icon(
                    kivact::interaction::Tone::Warning,
                )),
        ))
        .style(style().gap(8)),
        "alert(\"Review required\").tone(Tone::Warning).icon(banner_icon(Tone::Warning))",
    )
}
