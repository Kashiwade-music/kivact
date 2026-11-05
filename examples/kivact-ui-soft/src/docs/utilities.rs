//! Role: Demonstrate scoped themes, localization and native link callbacks.
use super::specimen::specimen;
use kivact::i18n::Messages;
use kivact::i18n::provide_messages;
use kivact::state::signal;
use kivact::style::provide_theme;
use kivact::style::{ThemeMode, rgb, style};
use kivact::ui::{badge, card};
use kivact::view::{Node, column, row, text};
use kivact_ui_soft::{MediaMode, link, media_theme, syntax_theme, theme};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "theme" => specimen(
            "Light and dark subtrees",
            row((
                provide_theme(theme(ThemeMode::Light), || card(badge("Light"))),
                provide_theme(theme(ThemeMode::Dark), || card(badge("Dark"))),
            ))
            .style(style().gap(12)),
            "provide_theme(theme(ThemeMode::Dark), content)",
        ),
        "internationalization-provider" => specimen(
            "Override the loading label",
            provide_messages(
                Messages::new().with("soft.spinner.loading", "Chargement"),
                kivact::ui::spinner,
            ),
            "provide_messages(Messages::new().with(\"soft.spinner.loading\", \"Chargement\"), spinner)",
        ),
        "link-provider" => {
            let selected = signal(false);
            specimen(
                "Route a native link through an application callback",
                column((
                    link("Project notes").on_press(move || selected.set(true)),
                    text(move || {
                        if selected.get() {
                            "Project notes selected"
                        } else {
                            "Choose a document"
                        }
                    }),
                ))
                .style(style().gap(8)),
                "link(\"Project notes\").on_press(open_notes)",
            )
        }
        "media-theme" => specimen(
            "Contrast on a dark preview",
            column(media_theme(MediaMode::Dark, {
                kivact_ui_soft::text("Mountain landscape")
            }))
            .style(style().background(rgb(0x18232d)).padding(24)),
            "media_theme(MediaMode::Dark, || text(\"Mountain landscape\"))",
        ),
        "syntax-theme" => specimen(
            "Scoped keyword color",
            syntax_theme(
                kivact_ui_soft::SyntaxColors {
                    keyword: rgb(0x315db0),
                    ..kivact::style::use_tokens::<kivact_ui_soft::Tokens>()
                        .colors
                        .syntax
                },
                kivact::ui::code_block("fn main() {}").language("rust"),
            ),
            "syntax_theme(colors, || code_block(source).language(\"rust\"))",
        ),
        _ => return None,
    })
}
