//! Role: The Native Layout page of the game catalog: Windows, trees, inspectors and status bars.

use crate::support::common::{choice_options, line, list, segmented, suppress_search_shortcut};
use crate::support::frame::native_window;
use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::Node;
use kivact_ui_game as g;

/// The page: its one demo, with its explanation.
pub(crate) fn page() -> Page {
    Page::new(
        "native-layout",
        "Native Layout",
        "Console",
        include_str!("native_layout.rs"),
    )
    .description("Windows, trees, inspectors and status bars")
    .example(Example::new(
        "window",
        "Window",
        "A native window with its menu, toolbar, document, inspector and status bar.",
        demo,
    ))
}

// example: window
fn demo() -> impl IntoView {
    layout()
}
// end example

// xtask-allow(state): legacy: notify, opacity; bound only to the inspector's notify switch; the opacity slider has its own signal, so notifications on or off pair with any opacity
pub(crate) fn layout() -> impl kivact::view::Element {
    let document = signal("OPS-2042.md".to_owned());
    let document_title = signal("Section 3 periodic inspection".to_owned());
    let selection = binding(
        memo(move || Some(document.get())),
        move |id: Option<String>| {
            if let Some(id) = id {
                document_title.set(id.clone());
                document.set(id);
            }
        },
    );
    let nodes = workspace_nodes();
    let files = workspace_files(nodes, selection);
    let title = signal("Section 3 periodic inspection".to_owned());
    let priority = signal(Some("medium".to_owned()));
    let notify = signal(true);
    let opacity = signal(90_f64);
    let document_body = document_preview(document, document_title);
    let inspector = document_inspector(title, priority, notify, opacity);
    g::hud(column((
        row((
            g::mono("LAY-01").style(style().color(color::ACCENT)),
            text("Desktop app window").style(style().font_weight(700)),
            spacer(),
            g::mono("3-pane").style(style().color(color::TEXT_MUTED)),
        ))
        .style(
            style()
                .height(36)
                .padding_x(14)
                .gap(10)
                .align_items(Align::Center)
                .background(color::SURFACE_ALT)
                .border_bottom(1, color::BORDER),
        ),
        frame(document, document_title, files, document_body, inspector),
    )))
    .test_id("LAY-01")
}

fn inspector_row(label: &str, control: impl IntoView) -> Node {
    row((
        text(label.to_owned()).style(style().width(84).font_size(11.5).color(color::TEXT_MUTED)),
        node()
            .child(control)
            .style(style().flex_grow(1).min_width(0)),
    ))
    .on_key(suppress_search_shortcut)
    .style(style().align_items(Align::Center).gap(6).width(pct(100)))
}

fn workspace_nodes() -> Vec<kivact::interaction::TreeNode<String>> {
    vec![
        kivact::interaction::TreeNode::new("operations".to_owned(), None, "operations"),
        kivact::interaction::TreeNode::new(
            "OPS-2041.md".to_owned(),
            Some("operations".to_owned()),
            "OPS-2041.md",
        ),
        kivact::interaction::TreeNode::new(
            "OPS-2042.md".to_owned(),
            Some("operations".to_owned()),
            "OPS-2042.md",
        ),
        kivact::interaction::TreeNode::new(
            "OPS-2043.md".to_owned(),
            Some("operations".to_owned()),
            "OPS-2043.md",
        ),
        kivact::interaction::TreeNode::new("inventory".to_owned(), None, "inventory"),
        kivact::interaction::TreeNode::new(
            "stock.db".to_owned(),
            Some("inventory".to_owned()),
            "stock.db",
        ),
        kivact::interaction::TreeNode::new("archive".to_owned(), None, "archive"),
        kivact::interaction::TreeNode::new(
            "2025_summary.md".to_owned(),
            Some("archive".to_owned()),
            "2025_summary.md",
        ),
    ]
}

fn workspace_files(
    nodes: Vec<kivact::interaction::TreeNode<String>>,
    selection: kivact::state::Signal<Option<String>>,
) -> impl IntoView {
    let expanded = signal(vec!["operations".to_owned()]);
    kivact::ui::tree(
        "Workspace files",
        nodes,
        binding(selection, move |key: Option<String>| {
            if key
                .as_deref()
                .is_none_or(|id| !["operations", "inventory", "archive"].contains(&id))
            {
                selection.set(key);
            }
        }),
    )
    .expanded(expanded)
    .on_activate(move |key| {
        if ["operations", "inventory", "archive"].contains(&key.as_str()) {
            expanded.update(|keys| {
                if keys.contains(&key) {
                    keys.retain(|k| k != &key);
                } else {
                    keys.push(key);
                }
            });
        }
    })
    .item(|item, _| {
        let key = item.get_untracked().key;
        let folder = ["operations", "inventory", "archive"].contains(&key.as_str());
        let marker = match key.as_str() {
            "OPS-2041.md" => "M",
            "OPS-2043.md" => "A",
            _ => "",
        };
        row((
            kivact::view::icon(if folder { "folder" } else { "file-text" }).style(
                style().size(14, 14).color(if folder {
                    color::WARNING
                } else {
                    color::TEXT_MUTED
                }),
            ),
            text(key).style(style().flex_grow(1).font_size(12.5)),
            g::mono(marker),
        ))
        .style(style().flex_grow(1).gap(6).align_items(Align::Center))
    })
}

fn document_preview(
    document: kivact::state::Signal<String>,
    document_title: kivact::state::Signal<String>,
) -> Node {
    list((
        g::mono(move || document.get().trim_end_matches(".md").to_uppercase()).style(style().color(color::ACCENT)),
        text(document_title).test_id("game-document-title").style(style().font_size(18).font_weight(900)),
        line((kivact::ui::badge("RUNNING").tone(kivact::interaction::Tone::Info), kivact::ui::badge("Maintenance").tone(kivact::interaction::Tone::Neutral) .variant(g::BadgeVariant::Plain), g::mono("updated 09:38").style(style().color(color::TEXT_MUTED)))),
        text("Inspect 12 switchboards and 4 cooling units. Check the correction status of the 2 findings from the last inspection and record the result in the audit log. Planned work window: 10:00–15:00.")
            .style(style().font_size(12.5).color(color::TEXT_MUTED).line_height(1.75)),
        node().style(style().padding_x(12).padding_y(10).background(rgb(0x0c1f36)).border_left(3, rgb(0x2ec4f0)))
            .child(g::mono("# checklist.yaml\ntargets:\n  - id: \"PB-01..PB-12\"\n    mode: inspect\n  - id: \"CU-01..CU-04\"\n    mode: thermal").style(style().font_size(11.5).line_height(1.7).color(rgb(0xcfe0f2)))),
    )).style(style().padding_y(18).padding_x(22))
}

fn document_inspector(
    title: kivact::state::Signal<String>,
    priority: kivact::state::Signal<Option<String>>,
    notify: kivact::state::Signal<bool>,
    opacity: kivact::state::Signal<f64>,
) -> Node {
    column((
        kivact::ui::disclosure("Basic", signal(true)).content(move || {
            list((
                inspector_row(
                    "Title",
                    kivact::ui::text_input(title)
                        .label("Inspector title")
                        .style(style().height(26).min_height(26).font_size(11.5)),
                ),
                inspector_row(
                    "Priority",
                    kivact::ui::select(
                        choice_options(&[
                            ("high", "P1 high"),
                            ("medium", "P2 medium"),
                            ("low", "P3 low"),
                        ]),
                        priority,
                    )
                    .placeholder("")
                    .name("Inspector priority")
                    .style(style().height(26)),
                ),
                inspector_row(
                    "Notifications",
                    kivact::ui::switch("", notify).name("Inspector notice"),
                ),
            ))
            .style(style().padding(10).gap(8))
        }),
        kivact::ui::disclosure("View", signal(true)).content(move || {
            list((
                inspector_row(
                    "Accent",
                    row((
                        node().style(style().size(16, 16).background(color::ACCENT)),
                        g::mono("--c-primary"),
                    ))
                    .style(style().gap(8).align_items(Align::Center)),
                ),
                inspector_row(
                    "Opacity",
                    kivact::ui::slider(opacity).label("Inspector opacity"),
                ),
            ))
            .style(style().padding(10).gap(8))
        }),
        kivact::ui::disclosure("Details", signal(false)).content(|| {
            kivact::ui::description_list(
                (vec![("TYPE".into(), "Markdown"), ("STATUS".into(), "Saved")])
                    .into_iter()
                    .map(|(term, detail): (String, _)| kivact::ui::description(term, detail))
                    .collect::<Vec<_>>(),
            )
        }),
    ))
}

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

// xtask-allow(state): legacy: width, help; the explorer width, written only by the native_window divider within explorer_range; it is layout only, so any width pairs with any
pub(crate) fn frame(
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
        "run" | "pause" => run_command(key, run, status),
        "help" => help.set(true),
        _ => (),
    };
    let menu = window_menu(command);
    let tools = window_tools(command, mode);
    let document_pane = document_pane(mode, content, preview);
    column((
        native_window(
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
                        .flex_shrink(0)
                        .border_left(1, color::BORDER),
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
            g::tool_button(message("game.window.new", "New")).on_press(move || command("new")),
            g::tool_button(message("game.window.save", "Save")).on_press(move || command("save")),
            g::tool_button(message("game.window.run", "Run")).on_press(move || command("run")),
            g::tool_button(message("game.window.pause", "Pause"))
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
                .background(color::SURFACE_ALT),
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
                kivact::ui::text_area(content)
                    .label("Document source")
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
    .style(style().flex_grow(1).min_width(0))
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
            .background(color::ACCENT)
            .color(color::ACCENT_TEXT),
    )
}
