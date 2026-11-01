# GUI harness (Windows, SendInput + UI Automation)

Real-window tests for the sample apps, the workbench and the external-component apps. The
scripts drive the Arvane host with `SendInput` and read state through UI
Automation. Each scenario is described at the top of its `scenario-*.ps1`; the testing overview is in [docs/for-kivact-developer/design.md](../design.md).

## Prerequisites

- Windows 10/11 and PowerShell 7 (`pwsh -STA -NoProfile -File`).
  STA is required by the UI Automation and desktop-input helpers. No execution-policy override is used.
- `python` on `PATH` (a local HTTP server for mini-postman, a WAV generator for
  the music player).
- An interactive, unlocked desktop (verified at 96 DPI only) with a Japanese IME installed
  (the IME items type romaji and convert). Do not touch the mouse or keyboard
  while a round runs.
- Debug builds of the six apps:

  ```powershell
  cargo build -p kivact-example-task-board -p kivact-example-mini-postman -p kivact-music-player -p kivact-workbench -p kivact-ext-apps --bins
  ```

  `perf-gui.ps1` needs a release build instead (`--release`).

## Paths and environment variables

All paths are derived by `harness-paths.ps1` (dot-sourced by the entry scripts):

| variable | default | meaning |
|---|---|---|
| `CARGO_TARGET_DIR` | `<repo>\target` | where the app binaries are taken from (`debug\` or `release\`) |
| `KIVACT_GUI_EVIDENCE` | `<repo>\target\gui-evidence` | root of logs, screenshots and summaries |
| `KIVACT_GUI_LOCK` | `%TEMP%\kivact-gui.lock` | lock file that allows only one GUI run at a time on the machine |
| `KIVACT_GUI_APPROVED_UNTIL` | none | explicitly approved desktop-use deadline in ISO 8601 UTC, required by catalog and starter scenarios |

Focus and `SendInput` are global, so only one GUI run may be active per desktop.
The input helpers refuse to send a key unless the foreground window belongs to
a program under the target dir, and a button press or wheel unless the window
under the cursor does (`KIVACT_GUI_GUARD_ROOT`, set by `harness-paths.ps1`):
a run that loses the foreground to another app stops with an error instead of
typing into it.
Every script takes the lock before starting an app (waiting up to 1-2 hours
while it is busy). `rerun-round.ps1` holds it for a whole round and sets
`KIVACT_GUI_LOCK_HELD=1` so the child scripts do not take it again. If a run is
killed, verify that its owned processes have stopped before removing its stale lock file. Worktrees that share one desktop
should use the same `KIVACT_GUI_LOCK` (the default already is).

## Entry scripts

Obtain permission for the target applications and time window before opening
windows or sending input. Build all applications first. The three catalogs are
separate executables, for example `kivact-ui-pop-catalog.exe`; set
`KIVACT_CATALOG_SCENE` to the matching kit for the internal capture scenario.
This variable chooses a test scenario and does not switch the app's kit.

```powershell
# One full round (6 apps plus the workbench icon scenario, about 6 minutes) holding the lock for the whole round.
pwsh -STA -NoProfile -File dev\tools\gui-harness\rerun-round.ps1 -Round 1
# -> target\gui-evidence\round\<app>-r1\ and round1-summary.txt

# The same round without the round-wide lock (each app takes the lock itself).
pwsh -STA -NoProfile -File dev\tools\gui-harness\check-round.ps1 -Round 1 [-Root <dir>]

# One scenario against one exe.
pwsh -STA -NoProfile -File dev\tools\gui-harness\run-gui-locked.ps1 -Exe <exe> -Title "Kivact Workbench" -Out <dir> -Scenario workbench-extra

# Starter search/detail/save route, using a dedicated output file.
pwsh -STA -NoProfile -File dev\tools\gui-harness\run-gui-locked.ps1 -Exe target\debug\kivact-interaction-starter.exe -Title "Kivact Interaction Starter" -Out <dir> -Scenario interaction-starter -AppArguments <absolute-output-file>

# Release app startup times (process start -> window -> first UIA element).
pwsh -STA -NoProfile -File dev\tools\gui-harness\perf-gui.ps1 [-Out <dir>]

# Host benchmark (dev/benchmarks/hostbench on arvane (the GPU, started with the window) and arvane-cpu (ARVANE_RENDERER=cpu); builds a release binary).
pwsh -STA -NoProfile -File dev\tools\gui-harness\hostbench.ps1 [-Runs 3] [-Hosts arvane,arvane-cpu] [-Scenarios text,idle] [-NoBuild]
# -> target\gui-evidence\hostbench\<timestamp>\summary.md, raw.jsonl, frame logs, screenshots
```

`hostbench.ps1` does not drive the app with input. It brings the bench window
(found by its title `hostbench <scenario>`) to the foreground, as in the
recorded tables; when `SetForegroundWindow` is refused it holds Alt for the
call (an Alt press and release on the desktop). It resizes it with
`SetWindowPos` in the `resize` scenario, and attaches no UI Automation client
(with one, the host keeps its accessibility tree up to date).

The summary lists `passed=N failed=M` per app. The IME reconversion items need
the host to answer `WM_IME_REQUEST`/`IMR_RECONVERTSTRING` (the Arvane shell
subclasses the window to do so).

Clicks, wheels and drags first send absolute `SendInput` moves to the target.
`SetCursorPos` alone reports the move lazily, after a button or wheel message
sent right behind it; a real mouse always reports the move first. (The Arvane
shell reads the position a button or wheel message carries and reports a move
first when it differs from the last one.)

The DPI item (`Fin-Dpi`) needs a debug build of the Arvane host: it posts the
registered message `ArvaneDebugDpiChange` (wParam 144, then 96), which the
host's window answers by sending itself the `WM_DPICHANGED` of a monitor of
that DPI (Windows refuses a cross-process `WM_DPICHANGED`, and every monitor
here is 96 DPI). A release build ignores it; that is logged, not counted.

Scripts that contain Japanese text retain UTF-8 with BOM. The main GUI scenario
log is explicitly UTF-16 (Unicode), matching the wrapper readers; app and hostbench
outputs explicitly marked UTF-8 remain UTF-8.

Run real-window commands under the repository window lock as well as the harness GUI lock.
Build the apps first. For example, from Git Bash:

```sh
dev/tools/window-lock.sh --timeout 5400 -- pwsh -STA -NoProfile -File dev/tools/gui-harness/rerun-round.ps1 -Round 1
```

Do not overlap this Windows round with a WSLg real-window suite: their process IDs
belong to different lock domains. No machine execution policy needs to be changed.
