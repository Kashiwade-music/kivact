# Use DevTools

[日本語](../../jp/how-to-guide/devtools.md)

DevTools is a separate application that connects to a running Kivact app. It
shows the element tree and layout of every window, lets you edit styles and
emulation, records turns and frames so you can see why the screen updated, and
keeps a review of comments and style changes that you can share with another
person or an AI agent. The same protocol is available to MCP tools and to Python
scripts.

## Install the tools

Install the DevTools app and the MCP server from the Kivact repository. Both
binaries go on `PATH`, and a target app uses `PATH` as the fourth and last place
it looks for the DevTools app:

```sh
cargo install --git https://github.com/Kashiwade-music/kivact kivact-devtools kivact-devtools-mcp
```

The Python client installs with pip. The command works outside a checkout of
Kivact:

```sh
pip install "kivact-devtools @ git+https://github.com/Kashiwade-music/kivact#subdirectory=python/kivact-devtools"
```

Only when you edit the Kivact repository itself, use
`pip install -e python/kivact-devtools`.

## Enable DevTools

DevTools lives behind the `devtools` feature of the `kivact` crate. Turn it on
in the app you want to debug:

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }

[features]
devtools = ["kivact/devtools"]
```

Run the app with `cargo run --features devtools`. Leave the feature out of the
release builds you ship: a release build with it enabled still works, but it
prints a warning at startup.

Add `.kivact/review/` and `.kivact/traces/` to `.gitignore`.

## Open DevTools

Press F12 or Ctrl+Shift+I in a running app. Ctrl+Shift+C starts pick mode and
opens DevTools at the same time; the pick starts in the window where you pressed
the keys. If DevTools is already open, the key brings it to the front.

The keys and the editor command come from `.kivact/devtools.json` in the
workspace root:

```json
{
  "$schema": "https://raw.githubusercontent.com/Kashiwade-music/kivact/main/crates/kivact-devtools-protocol/schema/devtools-config.schema.json",
  "keys": {
    "open": ["F12", "Ctrl+Shift+I"],
    "pick": ["Ctrl+Shift+C"]
  },
  "editor": ["code", "--goto", "{file}:{line}:{column}"]
}
```

Write a chord as the modifiers `Ctrl`, `Shift`, `Alt` and `Meta` joined with
`+`, then the key name (`F12`, `Enter`, or a single character). `keys.open` and
`keys.pick` are independent: omit one to keep its default, or set it to `[]` to
disable only those keys. `editor` defaults to
`["code", "--goto", "{file}:{line}:{column}"]`; `{file}` is replaced with the
absolute path from the workspace root. On Windows, when `code` cannot be
started, the app tries `code.cmd` once.

The target app looks for the DevTools app in the `app` key of
`.kivact/devtools.json`, then `KIVACT_DEVTOOLS_APP`, then the directory of the
target executable, then `PATH`. If it finds none, it reports `DevTools app not
found.` Install it with the `cargo install` command above, or set `app` to the
executable's path.

`.kivact/` is created when the first review or trace file is written, not when
the app starts.

## Inspect elements

The Elements panel shows the element tree of the window selected in the toolbar.
Each row shows the role, name, source location and update count (counted since
recording started). Use the pick button (or Ctrl+Shift+C) and click an element
in the target to select it.

- Clicking the source location opens the file in your editor.
- "Copy locator" copies a `kivact::test` expression to the clipboard. It uses
  the test id when there is one, and the role and name otherwise:
  `screen.get_by_test_id("increment")` or
  `screen.get_by_role(Role::Button, "Increment")`. A node in another window is
  prefixed with `screen.window("Settings").`.
- The "Owners" toggle replaces the element tree with the owner tree of the
  window. A row shows the owner's label and creation location; selecting one
  selects its first element.
- The Layout tab has the flex/grid overlay toggle. It draws the child boxes and
  gaps over the target's screen.

## Edit styles

The Styles tab lists the style layers from the highest priority down. A
checkbox turns each override on and off, and the value input applies a new value
on Enter. An override has one of three scopes:

- this node only;
- every node created from the same call site;
- a theme token.

The same tab toggles forced interactive states such as hover and pressed; an
empty list releases them. Turn on "Record changes in review" to record the
changes in the review item.

## Comment and export a review

Add comments to an element from Elements, or to a window from the toolbar. Every
change writes `review.json` (the canonical document, with a schema and a version)
and `review.md` (the same content for people) to
`.kivact/review/<YYYYMMDD-HHMMSS>-<pid>-<port>/`. There is no separate export
step; "Open folder" opens that directory. An item whose node has left the tree
shows "(removed)" and cannot be highlighted, but its comments stay editable.

A `review.json` example:

```json
{
  "schema": "kivact-devtools-review/1",
  "app": {
    "name": "Counter",
    "pid": 12345,
    "kivact_version": "0.1.0",
    "build": "debug",
    "exe": "C:\\dev\\counter\\target\\debug\\counter.exe",
    "cwd": "C:\\dev\\counter",
    "workspace_root": "C:\\dev\\counter"
  },
  "session": "20261108-053200-12345-49152",
  "created": "2026-11-08T05:32:00Z",
  "updated": "2026-11-08T05:34:10Z",
  "environment": {
    "pseudo_locale": "off"
  },
  "items": [
    {
      "id": "C1",
      "window_title": "Counter",
      "window": 1,
      "target": {
        "kind": "node",
        "node": 17,
        "role": "button",
        "name": "Increment",
        "source": { "file": "src/main.rs", "line": 42, "column": 9 },
        "call_chain": [
          {
            "location": { "file": "src/main.rs", "line": 42, "column": 9 },
            "app_code": true
          },
          {
            "location": { "file": "src/main.rs", "line": 12, "column": 5 },
            "app_code": true
          }
        ],
        "bounds": { "x": 24, "y": 96, "width": 96, "height": 32 },
        "states": ["hovered"]
      },
      "style_changes": [
        {
          "scope": { "kind": "node", "node": 17 },
          "property": "padding",
          "after": "12"
        }
      ],
      "image": "images/C1.png",
      "comments": [
        {
          "id": "C1.1",
          "text": "Rename this to \"Add one\".",
          "created": "2026-11-08T05:33:01Z"
        }
      ]
    }
  ]
}
```

The matching `review.md`:

```markdown
# DevTools review: Counter

- Session: 20261108-053200-12345-49152 / Updated: 2026-11-08T05:34:10Z
- Build: debug / Kivact 0.1.0
- Emulation: none

## C1 Button "Increment"  (window "Counter")

- Source: src/main.rs:42:9
- Call chain: src/main.rs:42 ← src/main.rs:12
- Query: screen.get_by_role(Role::Button, "Increment")
- State: hovered / Bounds: x 24, y 96, 96×32
- Image: images/C1.png
- Style changes:
  - padding: (unset) → 12  (node #17)
- Comments:
  1. Rename this to "Add one".  (2026-11-08T05:33:01Z)
```

## See why the screen updated

Start recording from the toolbar, then use the app. The Updates panel shows a
timeline with lanes for input, updates and frames. Each turn is colored by its
cause and drawn against the frame budget; select a range to zoom. Below it, the
cause graph of the selected turn goes from the cause to the state changes, the
recalculation, the elements that changed, and the layout, paint and frame bands.
Click a card to highlight the element in the target, or double-click it to open
the editor. A grouped set of nodes expands with "×N". You can switch between
"flash on screen" and "selected element only", and draw a heatmap on the screen.

## Measure performance

The Performance panel draws frame times as bars with a budget line for each
window, a flame chart of the turn stages and frame stages, and whether the GPU
or the CPU drew each frame (including fallbacks). Counters are line charts, and
you can take a snapshot and diff two snapshots to see what was added and removed.
The paint flashing toggle flashes the boxes that were repainted. "Export as
Perfetto" writes numbered files under `.kivact/traces/`.

## Check accessibility

The Accessibility panel shows the accessibility tree of a window, the audit
results as a table (select a row to highlight it in the target), the Tab order
with numbered badges, and the focus history.

## Emulate

The Emulation panel sets the theme (none, Light or Dark), reduced motion, locale,
pseudo-locale, DPI scale and touch. An unset field is not overridden. Theme
emulation only affects apps that choose their theme from
`platform::theme_hint()`, and the panel says so. Pseudo-locale changes only the
strings that go through `message()` and `message_with()`, so text written
directly in `text("...")` stays unchanged.

## Use DevTools from an AI agent (MCP)

`kivact-devtools-mcp` is an MCP server on stdio. Register it with:

```sh
claude mcp add kivact-devtools -- kivact-devtools-mcp
```

Every protocol method except `initialize`, `subscribe` and `show_table` is a
tool with the same name, plus two local tools. `list_targets` lists the running
targets (path, app name, pid, port, version, start time and workspace root); it
never includes the token. `select_target` takes `{ "target": "<path>" }`.
Without a selection the server uses `KIVACT_DEVTOOLS_TARGET`, then the only
running target. With no target or several targets it returns an error that tells
you to call `list_targets`.

The tools cover the same ground as the panels: elements and layout (`get_tree`,
`get_node`, the 12 queries, `get_owners`, `screenshot`, `highlight`,
`start_pick`/`stop_pick`, `lint_layout`), updates and performance (recording,
`list_turns`, `list_frames`, `get_counters`, snapshots, `export_trace`), styles
and emulation (`set_style_override`, `force_state`, `set_emulation`,
`set_overlays`, `set_time_scale`), input (`inject_input`, `list_inputs`), and the
review (`get_review`, `add_comment`, `edit_comment`, `remove_comment`,
`export_review`).

After `inject_input`, call `list_turns` with `wait_ms` to wait for the turn.

## Script DevTools with Python

Install the Python package with the pip command in
[Install the tools](#install-the-tools). In VS Code, Pylance and pyright read the
type hints and `py.typed`; no stubs are needed.

```python
import kivact_devtools as kd

app = kd.connect()                       # the only running target
win = app.window("Counter")              # by title
button = win.get_by_role(kd.Role.BUTTON, "Increment")
buttons = win.all_by_role(kd.Role.BUTTON)
button.highlight()
app.clear_highlight()
button.click()
win.type_text("abc")
win.press_key("Ctrl+Enter")              # the chord syntax of the config file

with app.record() as rec:
    button.click()
for turn in rec.turns:
    print(turn.turn, turn.duration_ns, len(turn.changed_nodes))

app.export_review()                      # [review.json, review.md] paths
```

`connect(target=...)` takes a discovery file path, and `name="Counter"` chooses
by app name. With neither, `connect()` uses `KIVACT_DEVTOOLS_TARGET`, then the
only running target. The `Window` queries have the same
`get_`/`query_`/`find_`/`all_` shapes as the protocol. `app.rpc` holds the
low-level typed methods, which take and return protocol dataclasses
(`app.rpc.get_tree(...)`); the hand-written `App`, `Window`, `Element` and
`Recording` classes do not inherit them.

The Scripts panel lists the `.py` files in `.kivact/devtools/scripts/` and runs
them. It finds the interpreter in the `python` setting, the workspace root's
`.venv`, `python3`, then `python`. It shows standard output and standard error
line by line, and shows tables produced by `show_table`.

## Limits

- In an embedded app (`embed`), `inject_input` is refused with `UNSUPPORTED`
  (-32015).
- Every client shares the same state. When two clients edit at once, the last
  write wins.
- When the last client disconnects, the state DevTools created is put back: pick,
  overlays and highlights, style overrides, forced states, emulation, the time
  scale and recording. The review stays on disk.
- Layer borders are not shown. Use paint flashing and the flex/grid overlay
  instead.
- `focus_window` is best effort. Where it does not work, such as on Wayland,
  the DevTools app does not come to the front.

## See also

- [Find out why something failed or updated](diagnostics.md): errors, warnings
  and the records DevTools reads.
- [Test your app](testing.md): the queries that "Copy locator" writes.
- [Open multiple windows](windows.md): the windows DevTools lists and focuses.
- [Use platform services](platform-services.md): OS features such as file
  dialogs and opening paths.
