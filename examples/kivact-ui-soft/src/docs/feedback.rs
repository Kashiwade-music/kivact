//! Role: Demonstrate Soft badges, placeholders and status indicators.
use super::specimen::specimen;
use kivact::control::{Node, NodeBuilder, column, row, text};
use kivact::style::{Align, style};
use kivact_ui_soft::{
    BadgeVariant, BannerStatus, SkeletonRadius, SpinnerShade, SpinnerSize, StatusDotVariant,
    TokenColor, badge, banner, skeleton, spinner, status_dot, token,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "badge" => specimen(
            "Semantic and categorical colors",
            row((
                badge("Neutral"),
                badge("Info").variant(BadgeVariant::Info),
                badge("Success").variant(BadgeVariant::Success),
                badge("Warning").variant(BadgeVariant::Warning),
                badge("Error").variant(BadgeVariant::Error),
                badge("Design").variant(BadgeVariant::Purple),
            ))
            .style(style().gap(8).wrap(true)),
            "badge(\"Success\").variant(BadgeVariant::Success)",
        ),
        "skeleton" => specimen(
            "Staggered placeholders",
            row((
                skeleton()
                    .radius(SkeletonRadius::Rounded)
                    .style(style().size(40, 40)),
                column((
                    skeleton().index(1).style(style().size(160, 14)),
                    skeleton().index(2).style(style().size(112, 12)),
                ))
                .style(style().gap(8)),
            ))
            .style(style().align_items(Align::Center).gap(12)),
            "skeleton().radius(SkeletonRadius::Rounded).style(style().size(40, 40))",
        ),
        "status-dot" => specimen(
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
        ),
        "spinner" => specimen(
            "Sizes and shades",
            row((
                spinner().size(SpinnerSize::Sm),
                spinner().size(SpinnerSize::Md).shade(SpinnerShade::Subtle),
                spinner().size(SpinnerSize::Lg).label("Loading"),
            ))
            .style(style().align_items(Align::Center).gap(16)),
            "spinner().size(SpinnerSize::Lg).label(\"Loading\")",
        ),
        "token" => specimen(
            "Categorical tokens",
            row((
                token("Neutral"),
                token("Engineering").color(TokenColor::Blue),
            ))
            .style(style().gap(8)),
            "token(\"Engineering\").color(TokenColor::Blue)",
        ),
        "banner" => specimen(
            "Status messages",
            column((
                banner(BannerStatus::Info, "Workspace updated"),
                banner(BannerStatus::Warning, "Review required"),
            ))
            .style(style().gap(8)),
            "banner(BannerStatus::Warning, \"Review required\")",
        ),
        _ => return None,
    })
}
