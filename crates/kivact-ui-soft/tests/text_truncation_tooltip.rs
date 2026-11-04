//! Role: Verify measured single-line ellipsis and line-clamp tooltips.
use kivact::control::{NodeBuilder, column};
use kivact::signal;
use kivact::style::{ThemeMode, style};
use kivact::test::{RenderOptions, render_with_handle};
use kivact::ui::button;
use kivact::view::{Role, node_ref};
use kivact_ui_soft::{TruncateTooltip, text, theme};
use std::time::Duration;

#[test]
fn text_tooltip_opens_only_while_native_layout_cuts_text() {
    const LONG: &str = "Longer content needs several lines in this narrow box.";
    for max_lines in [1, 2] {
        let (screen, truncated) = render_with_handle(
            RenderOptions::from_env().theme(theme(ThemeMode::Light)),
            move || {
                let content = signal("Short".to_owned());
                let reference = node_ref();
                let truncated = reference.text_truncated();
                let message = text(content)
                    .max_lines(max_lines)
                    .truncate_tooltip(TruncateTooltip::Above)
                    .node_ref(reference)
                    .style(style().width(72));
                (
                    truncated,
                    column((
                        button("Show long text").on_press(move || content.set(LONG.to_owned())),
                        message,
                        button("Restore short text")
                            .on_press(move || content.set("Short".to_owned())),
                    )),
                )
            },
        );
        assert!(!truncated.get_untracked());
        screen.get_by_text("Short").hover();
        screen.advance(Duration::from_secs(1));
        assert!(screen.query_by_role(Role::Tooltip, "Short").is_none());

        screen.get_by_role(Role::Button, "Show long text").click();
        assert!(truncated.get_untracked());
        screen.get_by_text(LONG).hover();
        screen.advance(Duration::from_secs(1));
        screen.get_by_role(Role::Tooltip, LONG);

        screen
            .get_by_role(Role::Button, "Restore short text")
            .click();
        assert!(!truncated.get_untracked());
        screen.get_by_text("Short").hover();
        screen.advance(Duration::from_secs(1));
        assert!(screen.query_by_role(Role::Tooltip, "Short").is_none());
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
