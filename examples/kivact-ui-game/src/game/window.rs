//! Role: Application-owned editor chrome and commands for the catalog.
use super::common::segmented;
use kivact::prelude::*;
use kivact_ui_game as g;

const INITIAL_SOURCE: &str = "# checklist.yaml\ntargets: PB-01..PB-12";

/// What the editor chrome's run commands set; the status bar's activity word comes from it.
#[derive(Clone, Copy, PartialEq)]
enum Run {
    Idle,
    Running,
    Paused,
}
impl Run {
    /// The status bar's activity word.
    fn activity(self) -> &'static str {
        match self {
            Run::Running => "RUNNING",
            Run::Idle | Run::Paused => "IDLE",
        }
    }
}

pub(super) fn frame(
    document: Signal<String>,
    title: Signal<String>,
    explorer: impl IntoView,
    preview: impl IntoView,
    inspector: impl IntoView,
) -> impl IntoView {
    let mode = signal(Some("preview"));
    let status = signal("Ready".to_owned());
    let run = signal(Run::Idle);
    let content = signal(INITIAL_SOURCE.to_owned());
    // xtask-allow(state-fields): independent: the explorer width, written only by the native_window divider within explorer_range; it is layout only, so any width pairs with any editor mode, run state or About dialog
    let width = signal(240_f64);
    let saved = signal(content.get_untracked());
    // xtask-allow(state-fields): independent: the About dialog's open binding, set by the Help menu's help command and cleared by the dialog; it never touches mode, run or the explorer width, so it opens over any editor state
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
        "run" | "pause" => run_command(key, run, status),
        "help" => help.set(true),
        _ => (),
    };
    let menu = window_menu(command);
    let tools = window_tools(command, mode);
    let document_pane = document_pane(mode, content, preview);
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
        .status_bar(window_status(status, content, saved, run)),
        kivact::ui::dialog("About this example", help)
            .content(|| text("Application-owned menus and editor state.")),
    ))
}

fn window_menu(command: impl Fn(&'static str) + Copy + 'static) -> impl IntoView {
    row([
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
    .style(style().gap(4))
}

fn window_tools(
    command: impl Fn(&'static str) + Copy + 'static,
    mode: Signal<Option<&'static str>>,
) -> impl IntoView {
    kivact::ui::toolbar("Console tools")
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
        )
        .test_id("game-window-toolbar")
}

fn document_pane(
    mode: Signal<Option<&'static str>>,
    content: Signal<String>,
    preview: impl IntoView,
) -> Node {
    column((
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
    .style(style().grow(1).min_width(0))
}

/// Record a run command: the activity it sets and the notice it leaves in
/// the status bar, where later commands such as Save replace the notice.
fn run_command(key: &str, run: Signal<Run>, status: Signal<String>) {
    let (next, notice) = if key == "run" {
        (Run::Running, "Running")
    } else {
        (Run::Paused, "Paused")
    };
    run.set(next);
    status.set(notice.to_owned());
}

fn window_status(
    status: Signal<String>,
    content: Signal<String>,
    saved: Signal<String>,
    run: Signal<Run>,
) -> Node {
    row((
        g::mono(move || status.get()).test_id("game-window-status"),
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
        g::mono(move || run.get().activity().to_owned()),
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
}
