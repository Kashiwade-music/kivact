//! Demonstrate scoped themes, localization and native link callbacks.
use super::labelled;
use kivact::i18n::Messages;
use kivact::i18n::provide_messages;
use kivact::state::signal;
use kivact::style::provide_theme;
use kivact::style::{ThemeMode, rgb, style};
use kivact::ui::{badge, card};
use kivact::view::{Node, column, row, text};
use kivact_ui_soft::{MediaMode, link, media_theme, syntax_theme, theme};

// example: theme
pub(super) fn example_theme() -> Node {
    labelled(
        "Light and dark subtrees",
        row((
            provide_theme(theme(ThemeMode::Light), || card(badge("Light"))),
            provide_theme(theme(ThemeMode::Dark), || card(badge("Dark"))),
        ))
        .style(style().gap(12)),
    )
}
// end example

// example: internationalization-provider
pub(super) fn example_internationalization_provider() -> Node {
    labelled(
        "Override the loading label",
        provide_messages(
            Messages::new().with("soft.spinner.loading", "Chargement"),
            kivact::ui::spinner,
        ),
    )
}
// end example

// example: link-provider
pub(super) fn example_link_provider() -> Node {
    {
        let selected = signal(false);
        labelled(
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
        )
    }
}
// end example

// example: media-theme
pub(super) fn example_media_theme() -> Node {
    labelled(
        "Contrast on a dark preview",
        column(media_theme(MediaMode::Dark, {
            kivact_ui_soft::styled_text("Mountain landscape")
        }))
        .style(style().background(rgb(0x18232d)).padding(24)),
    )
}
// end example

// example: syntax-theme
pub(super) fn example_syntax_theme() -> Node {
    labelled(
        "Scoped keyword color",
        syntax_theme(
            kivact_ui_soft::SyntaxColors {
                keyword: rgb(0x315db0),
                ..kivact::style::use_tokens::<kivact_ui_soft::Tokens>()
                    .colors
                    .syntax
            },
            kivact::ui::code_block("fn main() {}")
                .language("rust")
                .highlight(kivact::ui::highlighter(
                    || (),
                    |_, line, _: &()| {
                        (
                            kivact_ui_soft::code_highlight(
                                &kivact_ui_soft::CodeLanguageInput::from("rust"),
                                line,
                            ),
                            (),
                        )
                    },
                )),
        ),
    )
}
// end example
