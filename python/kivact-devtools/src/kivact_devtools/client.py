"""Connect to a running Kivact app and drive it from Python (design 4.7.3)."""
from __future__ import annotations

import base64
import os
from collections.abc import Sequence
from importlib import metadata
from pathlib import Path
from typing import Literal

from ._discovery import list_targets, read_target, target_from_env
from ._generated import (
    AddCommentParams,
    AppInfo,
    AuditReport,
    ClientInfo,
    ClientKind,
    ElementInfo,
    ErrorCode,
    FrameInfo,
    GetNodeParams,
    GetTreeParams,
    HighlightParams,
    InitializeParams,
    InjectInputParams,
    InputStepClick,
    InputStepKey,
    InputStepText,
    LayoutLint,
    ListTurnsParams,
    Methods,
    NodeDetail,
    OverrideScopeCallSite,
    OverrideScopeNode,
    PROTOCOL_VERSION,
    RecordingSettings,
    ReviewItem,
    Role,
    RoleQuery,
    ScreenshotParams,
    SetStyleOverrideParams,
    ShowTableParams,
    SourceLocation,
    StyleOverride,
    TestIdQuery,
    TextQuery,
    TreeSnapshot,
    TurnGraph,
    WindowInfo,
    WindowParams,
)
from ._rpc import RpcConnection
from .errors import DevtoolsError, MultipleFound, NotFound


def connect(*, target: str | os.PathLike[str] | None = None, name: str | None = None) -> App:
    """Connect to a target: by discovery file, by app name, or by environment/one running target."""
    if target is not None and name is not None:
        raise ValueError("pass target or name, not both")
    if target is not None:
        path = Path(target)
    elif name is not None:
        path = _target_by_name(name)
    else:
        from_env = target_from_env()
        if from_env is not None:
            path = from_env
        else:
            found = list_targets()
            if len(found) == 1:
                path = found[0][0]
            elif not found:
                raise NotFound(int(ErrorCode.NO_MATCH), "no running target")
            else:
                listed = ", ".join(f"{file.app_name} (pid {file.pid})" for _, file in found)
                raise MultipleFound(
                    int(ErrorCode.MULTIPLE_MATCHES),
                    f"several running targets: {listed}; pass target= or name=",
                )
    file = read_target(path)
    if file.protocol != PROTOCOL_VERSION:
        raise DevtoolsError(
            int(ErrorCode.VERSION_MISMATCH),
            f"protocol version differs (target v{file.protocol}, client v{PROTOCOL_VERSION})",
        )
    try:
        connection = RpcConnection(file.port)
    except OSError as error:
        raise DevtoolsError(int(ErrorCode.INTERNAL_ERROR), f"cannot connect: {error}") from error
    try:
        result = Methods(connection.call).initialize(
            InitializeParams(
                protocol=PROTOCOL_VERSION,
                token=file.token,
                client=ClientInfo(
                    name="kivact-devtools-python",
                    version=_package_version(),
                    kind=ClientKind.PYTHON,
                ),
            )
        )
    except Exception:
        connection.close()
        raise
    if result.protocol != PROTOCOL_VERSION:
        connection.close()
        raise DevtoolsError(
            int(ErrorCode.VERSION_MISMATCH),
            f"protocol version differs (target v{result.protocol}, client v{PROTOCOL_VERSION})",
        )
    return App(connection, result.app)


def _target_by_name(name: str) -> Path:
    matches = [(path, file) for path, file in list_targets() if file.app_name == name]
    if not matches:
        raise NotFound(int(ErrorCode.NO_MATCH), f"no target named {name!r}")
    if len(matches) > 1:
        listed = ", ".join(f"{file.app_name} (pid {file.pid})" for _, file in matches)
        raise MultipleFound(
            int(ErrorCode.MULTIPLE_MATCHES),
            f"several targets named {name!r}: {listed}; pass target= or name=",
        )
    return matches[0][0]


def _package_version() -> str:
    try:
        return metadata.version("kivact-devtools")
    except metadata.PackageNotFoundError:
        return "0.0.0"


class App:
    """A connected target app."""

    def __init__(self, connection: RpcConnection, info: AppInfo) -> None:
        self.rpc = Methods(connection.call)
        self.info = info
        self._connection = connection

    def windows(self) -> list[Window]:
        """Every open window."""
        return [Window(self, info) for info in self.rpc.list_windows().windows]

    def window(self, title: str) -> Window:
        """The one window with this exact title."""
        matches = [window for window in self.windows() if window.info.title == title]
        if not matches:
            raise NotFound(int(ErrorCode.WINDOW_NOT_FOUND), f"no window titled {title!r}")
        if len(matches) > 1:
            raise MultipleFound(int(ErrorCode.MULTIPLE_MATCHES), f"{len(matches)} windows titled {title!r}")
        return matches[0]

    def record(self, *, updates: bool = True, frames: bool = True, input: bool = False) -> Recording:
        """A recording of the turns, frames and input made in a `with` block."""
        return Recording(self, RecordingSettings(updates=updates, frames=frames, input=input))

    def clear_highlight(self) -> None:
        """Remove the highlight from the target's screen."""
        self.rpc.highlight(HighlightParams(node=None, layout=False))

    def export_review(self) -> list[Path]:
        """Write `review.json` and `review.md`; returns both paths."""
        result = self.rpc.export_review()
        return [Path(result.json), Path(result.markdown)]

    def show_table(self, title: str, columns: Sequence[str], rows: Sequence[Sequence[str]]) -> None:
        """Show a table in the DevTools app."""
        self.rpc.show_table(
            ShowTableParams(
                title=title,
                columns=list(columns),
                rows=[list(row) for row in rows],
            )
        )

    def close(self) -> None:
        """Close the connection."""
        self._connection.close()

    def __enter__(self) -> App:
        return self

    def __exit__(self, exc_type: object, exc: object, traceback: object) -> None:
        self.close()


class _Queries:
    """The twelve queries, written once for windows and elements."""

    _app: App

    def _scope(self) -> tuple[int, int | None]:
        raise NotImplementedError

    def get_by_role(self, role: Role, name: str | None = None) -> Element:
        window, within = self._scope()
        found = self._app.rpc.get_by_role(
            RoleQuery(window=window, within=within, role=role, name=name, timeout_ms=None)
        )
        return Element(self._app, found.node)

    def query_by_role(self, role: Role, name: str | None = None) -> Element | None:
        window, within = self._scope()
        found = self._app.rpc.query_by_role(
            RoleQuery(window=window, within=within, role=role, name=name, timeout_ms=None)
        )
        return None if found.node is None else Element(self._app, found.node)

    def find_by_role(self, role: Role, name: str | None = None, *, timeout: float = 5.0) -> Element:
        window, within = self._scope()
        found = self._app.rpc.find_by_role(
            RoleQuery(window=window, within=within, role=role, name=name, timeout_ms=int(timeout * 1000))
        )
        return Element(self._app, found.node)

    def all_by_role(self, role: Role, name: str | None = None) -> list[Element]:
        window, within = self._scope()
        found = self._app.rpc.all_by_role(
            RoleQuery(window=window, within=within, role=role, name=name, timeout_ms=None)
        )
        return [Element(self._app, info) for info in found.nodes]

    def get_by_text(self, text: str) -> Element:
        window, within = self._scope()
        found = self._app.rpc.get_by_text(TextQuery(window=window, within=within, text=text, timeout_ms=None))
        return Element(self._app, found.node)

    def query_by_text(self, text: str) -> Element | None:
        window, within = self._scope()
        found = self._app.rpc.query_by_text(TextQuery(window=window, within=within, text=text, timeout_ms=None))
        return None if found.node is None else Element(self._app, found.node)

    def find_by_text(self, text: str, *, timeout: float = 5.0) -> Element:
        window, within = self._scope()
        found = self._app.rpc.find_by_text(
            TextQuery(window=window, within=within, text=text, timeout_ms=int(timeout * 1000))
        )
        return Element(self._app, found.node)

    def all_by_text(self, text: str) -> list[Element]:
        window, within = self._scope()
        found = self._app.rpc.all_by_text(TextQuery(window=window, within=within, text=text, timeout_ms=None))
        return [Element(self._app, info) for info in found.nodes]

    def get_by_test_id(self, id: str) -> Element:
        window, within = self._scope()
        found = self._app.rpc.get_by_test_id(TestIdQuery(window=window, within=within, id=id, timeout_ms=None))
        return Element(self._app, found.node)

    def query_by_test_id(self, id: str) -> Element | None:
        window, within = self._scope()
        found = self._app.rpc.query_by_test_id(TestIdQuery(window=window, within=within, id=id, timeout_ms=None))
        return None if found.node is None else Element(self._app, found.node)

    def find_by_test_id(self, id: str, *, timeout: float = 5.0) -> Element:
        window, within = self._scope()
        found = self._app.rpc.find_by_test_id(
            TestIdQuery(window=window, within=within, id=id, timeout_ms=int(timeout * 1000))
        )
        return Element(self._app, found.node)

    def all_by_test_id(self, id: str) -> list[Element]:
        window, within = self._scope()
        found = self._app.rpc.all_by_test_id(TestIdQuery(window=window, within=within, id=id, timeout_ms=None))
        return [Element(self._app, info) for info in found.nodes]


class Window(_Queries):
    """One window of a target app."""

    def __init__(self, app: App, info: WindowInfo) -> None:
        self._app = app
        self.info = info

    def _scope(self) -> tuple[int, int | None]:
        return (self.info.id, None)

    def tree(self) -> TreeSnapshot:
        """The element tree of this window."""
        return self._app.rpc.get_tree(GetTreeParams(window=self.info.id, root=None, depth=None))

    def type_text(self, text: str) -> None:
        """Type text into the focused element."""
        self._app.rpc.inject_input(InjectInputParams(window=self.info.id, steps=[InputStepText(text=text)]))

    def press_key(self, key: str) -> None:
        """Press one key or chord (`"Enter"`, `"Ctrl+Enter"`)."""
        self._app.rpc.inject_input(InjectInputParams(window=self.info.id, steps=[InputStepKey(key=key)]))

    def screenshot(self) -> bytes:
        """A PNG of this window."""
        image = self._app.rpc.screenshot(ScreenshotParams(window=self.info.id, node=None))
        return base64.b64decode(image.png_base64)

    def add_comment(self, text: str, *, screenshot: bool = True) -> ReviewItem:
        """Add a comment about this window to the review."""
        return self._app.rpc.add_comment(
            AddCommentParams(window=self.info.id, node=None, text=text, screenshot=screenshot)
        )

    def lint_layout(self) -> LayoutLint:
        """Layout problems found in this window."""
        return self._app.rpc.lint_layout(WindowParams(window=self.info.id))

    def audit_a11y(self) -> AuditReport:
        """Accessibility problems found in this window."""
        return self._app.rpc.audit_a11y(WindowParams(window=self.info.id))


class Element(_Queries):
    """One element of a target app."""

    def __init__(self, app: App, info: ElementInfo) -> None:
        self._app = app
        self.info = info
        self.source: SourceLocation | None = info.source

    def _scope(self) -> tuple[int, int | None]:
        return (self.info.window, self.info.id)

    def detail(self) -> NodeDetail:
        """Everything known about this element."""
        return self._app.rpc.get_node(GetNodeParams(node=self.info.id))

    def highlight(self) -> None:
        """Highlight this element on the target's screen."""
        self._app.rpc.highlight(HighlightParams(node=self.info.id, layout=False))

    def click(self) -> None:
        """Click the centre of this element."""
        self._app.rpc.inject_input(
            InjectInputParams(window=self.info.window, steps=[InputStepClick(node=self.info.id)])
        )

    def set_style_override(
        self,
        property: str,
        value: str,
        *,
        scope: Literal["node", "call_site"] = "node",
    ) -> StyleOverride:
        """Override one style property of this element or of its call site."""
        if scope == "node":
            selected = OverrideScopeNode(node=self.info.id)
        elif scope == "call_site":
            if self.source is None:
                raise ValueError("this element has no source location")
            selected = OverrideScopeCallSite(location=self.source)
        else:
            raise ValueError(f"unknown scope: {scope!r}")
        return self._app.rpc.set_style_override(
            SetStyleOverrideParams(scope=selected, property=property, value=value)
        )

    def add_comment(self, text: str, *, screenshot: bool = True) -> ReviewItem:
        """Add a comment about this element to the review."""
        return self._app.rpc.add_comment(
            AddCommentParams(window=self.info.window, node=self.info.id, text=text, screenshot=screenshot)
        )


class Recording:
    """The turns and frames recorded in a `with` block."""

    def __init__(self, app: App, settings: RecordingSettings) -> None:
        self._app = app
        self._settings = settings
        self._since: int | None = None
        self.turns: list[TurnGraph] = []
        self.frames: list[FrameInfo] = []

    def __enter__(self) -> Recording:
        latest = self._app.rpc.list_turns(ListTurnsParams(since=None, limit=1, wait_ms=None)).turns
        self._since = latest[-1].turn if latest else None
        self._app.rpc.start_recording(self._settings)
        return self

    def __exit__(self, exc_type: object, exc: object, traceback: object) -> None:
        self._app.rpc.stop_recording()
        self.turns = self._app.rpc.list_turns(
            ListTurnsParams(since=self._since, limit=None, wait_ms=None)
        ).turns
        self.frames = [frame for turn in self.turns for frame in turn.frames]
