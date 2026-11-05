//! Role: Application-owned editor chrome and commands for the catalog.
use super::common::segmented;
use kivact::prelude::*;
use kivact_ui_game as g;

const INITIAL_SOURCE: &str = "# checklist.yaml\ntargets: PB-01..PB-12";

pub(super) fn frame(
    document: Signal<String>,
    title: Signal<String>,
    explorer: impl IntoView,
    preview: impl IntoView,
    inspector: impl IntoView,
) -> impl IntoView {
    let mode = signal(Some("preview"));
    let status = signal("Ready".to_owned());
    let running = signal(false);
    let content = signal(INITIAL_SOURCE.to_owned());
    let width = signal(240_f64);
    let saved = signal(content.get_untracked());
    let help = signal(false);
    let command = move |key: &'static str| match key {
        "new" => {
            document.set("untitled.md".to_owned());
            title.set("Untitled".to_owned());
            content.set(String::new());
            saved.set(String::new());
            mode.set(Some("edit"));
            status.set("New document".to_owned());
        }
        "save" => {
            saved.set(content.get_untracked());
            status.set(format!("Saved {}", document.get_untracked()));
        }
        "clear" => {
            content.set(String::new());
            mode.set(Some("edit"));
            status.set("Editor cleared".to_owned());
        }
        "edit" => mode.set(Some("edit")),
        "preview" => mode.set(Some("preview")),
        "run" => {
            running.set(true);
            status.set("Running".to_owned());
        }
        "pause" => {
            running.set(false);
            status.set("Paused".to_owned());
        }
        "help" => help.set(true),
        _ => (),
    };
    let menu = row([
        (
            "game.window.menu.file",
            "File",
            vec![
                MenuEntry::item("new", message("game.window.new", "New")),
                MenuEntry::item("save", message("game.window.save", "Save")),
            ],
        ),
        (
            "game.window.menu.edit",
            "Edit",
            vec![MenuEntry::item("clear", "Clear editor")],
        ),
        (
            "game.window.menu.view",
            "View",
            vec![
                MenuEntry::item("edit", message("game.window.edit", "Edit")),
                MenuEntry::item("preview", message("game.window.preview", "Preview")),
            ],
        ),
        (
            "game.window.menu.tools",
            "Tools",
            vec![
                MenuEntry::item("run", message("game.window.run", "Run")),
                MenuEntry::item("pause", message("game.window.pause", "Pause")),
            ],
        ),
        (
            "game.window.menu.help",
            "Help",
            vec![MenuEntry::item("help", "About this example")],
        ),
    ]
    .into_iter()
    .map(|(key, label, entries)| {
        kivact::ui::menu(message(key, label), entries)
            .on_activate(command)
            .attach(
                kivact::ui::button(message(key, label))
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .style(style().color(rgb(0xe3ecf6))),
            )
    })
    .collect::<Vec<_>>())
    .style(style().gap(4));
    let tools = kivact::ui::toolbar("Console tools")
        .start((
            g::tool_button(message("game.window.new", "New"))
                .icon(kivact::view::icon("plus"))
                .hide_label(true)
                .on_press(move || command("new")),
            g::tool_button(message("game.window.save", "Save"))
                .icon(kivact::view::icon("device-floppy"))
                .hide_label(true)
                .on_press(move || command("save")),
            g::tool_button(message("game.window.run", "Run"))
                .icon(kivact::view::icon("player-play"))
                .hide_label(true)
                .on_press(move || command("run")),
            g::tool_button(message("game.window.pause", "Pause"))
                .icon(kivact::view::icon("player-pause"))
                .hide_label(true)
                .on_press(move || command("pause")),
            spacer(),
            segmented(
                message("game.window.mode", "Mode"),
                vec![
                    ListItem::new("edit", message("game.window.edit", "Edit")),
                    ListItem::new("preview", message("game.window.preview", "Preview")),
                ],
                mode,
            ),
        ))
        .style(
            style()
                .padding_x(8)
                .padding_y(4)
                .background(ColorToken::SurfaceAlt),
        );
    let document_pane = column((
        show(
            move || mode.get() == Some("edit"),
            move || {
                kivact::ui::text_area("Document source", content)
                    .style(style().width(pct(100)).min_height(220))
            },
        ),
        node()
            .child((
                preview,
                show(
                    move || content.get() != INITIAL_SOURCE,
                    move || g::mono(content).test_id("game-window-preview-source"),
                ),
            ))
            .hidden(move || mode.get() != Some("preview")),
    ))
    .style(style().grow(1).min_width(0));
    column((
        crate::game::frame::native_window(
            move || format!("{} ▪", document.get()),
            width,
            column((
                g::mono(message("game.window.explorer", "EXPLORER")),
                explorer,
            )),
            row((
                document_pane,
                column((
                    g::mono(message("game.window.inspector", "INSPECTOR")),
                    inspector,
                ))
                .style(
                    style()
                        .width(260)
                        .shrink(0)
                        .border_left(1, ColorToken::Border),
                ),
            ))
            .style(style().align_items(Align::Stretch)),
        )
        .explorer_range(
            kivact::interaction::ScalarRange::new(140., 420.)
                .expect("invariant: the catalog explorer range is finite"),
        )
        .menu_bar(menu)
        .toolbar(tools)
        .status_bar({
            row((
                g::mono(status).test_id("game-window-status"),
                g::mono(message("game.window.branch", "main")),
                g::mono(move || {
                    if content.get() == saved.get() {
                        "Saved"
                    } else {
                        "Modified"
                    }
                    .to_owned()
                })
                .test_id("game-window-save-state"),
                spacer(),
                g::mono(move || if running.get() { "RUNNING" } else { "IDLE" }.to_owned()),
                g::mono(message("game.window.status", "UTF-8 · LF · Markdown")),
            ))
            .style(
                style()
                    .height(24)
                    .padding_x(10)
                    .gap(16)
                    .align_items(Align::Center)
                    .background(ColorToken::Accent)
                    .color(ColorToken::AccentText),
            )
        }),
        kivact::ui::dialog("About this example", help)
            .content(|| text("Application-owned menus and editor state.")),
    ))
}
