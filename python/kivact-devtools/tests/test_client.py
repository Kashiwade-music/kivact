"""Tests of the hand-written DevTools client against a fake agent."""
from __future__ import annotations

import json
import socketserver
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from kivact_devtools import (
    App,
    DevtoolsError,
    Element,
    MultipleFound,
    NotFound,
    Role,
    Timeout,
    connect,
)
from kivact_devtools import _discovery
from kivact_devtools._generated import (
    AddCommentParams,
    ErrorCode,
    GetNodeParams,
    GetTreeParams,
    HighlightParams,
    InitializeParams,
    InjectInputParams,
    ListTurnsParams,
    RecordingSettings,
    RoleQuery,
    ScreenshotParams,
    SetStyleOverrideParams,
    ShowTableParams,
    TestIdQuery,
    TextQuery,
    WindowParams,
)
from kivact_devtools._rpc import RpcConnection

PARAM_CLASSES: dict[str, type] = {
    "initialize": InitializeParams,
    "get_by_role": RoleQuery,
    "query_by_role": RoleQuery,
    "find_by_role": RoleQuery,
    "all_by_role": RoleQuery,
    "get_by_text": TextQuery,
    "query_by_text": TextQuery,
    "find_by_text": TextQuery,
    "all_by_text": TextQuery,
    "get_by_test_id": TestIdQuery,
    "query_by_test_id": TestIdQuery,
    "find_by_test_id": TestIdQuery,
    "all_by_test_id": TestIdQuery,
    "get_tree": GetTreeParams,
    "get_node": GetNodeParams,
    "highlight": HighlightParams,
    "screenshot": ScreenshotParams,
    "set_style_override": SetStyleOverrideParams,
    "add_comment": AddCommentParams,
    "show_table": ShowTableParams,
    "list_turns": ListTurnsParams,
    "start_recording": RecordingSettings,
    "inject_input": InjectInputParams,
    "lint_layout": WindowParams,
    "audit_a11y": WindowParams,
}

ELEMENT_JSON: dict[str, Any] = {
    "id": 2,
    "window": 1,
    "kind": "box",
    "role": "button",
    "name": "Increment",
    "hidden": False,
    "leaving": False,
    "update_count": 0,
}

RECT_JSON: dict[str, Any] = {"x": 0.0, "y": 0.0, "width": 10.0, "height": 10.0}

NODE_DETAIL_JSON: dict[str, Any] = {
    "node": ELEMENT_JSON,
    "bounds": {
        "margin": RECT_JSON,
        "border": RECT_JSON,
        "padding": RECT_JSON,
        "content": RECT_JSON,
    },
    "semantics": {"focusable": True},
}

PNG_BASE64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg=="

NODE_METHODS = {
    "get_by_role",
    "query_by_role",
    "find_by_role",
    "get_by_text",
    "query_by_text",
    "find_by_text",
    "get_by_test_id",
    "query_by_test_id",
    "find_by_test_id",
}

NODES_METHODS = {"all_by_role", "all_by_text", "all_by_test_id"}


class FakeAgent(socketserver.ThreadingTCPServer):
    """A fake DevTools agent: one JSON-RPC request per line, fixed answers."""

    allow_reuse_address = True
    daemon_threads = True

    def __init__(self) -> None:
        super().__init__(("127.0.0.1", 0), FakeHandler)
        self.failures: list[str] = []
        self.requests: list[tuple[str, dict[str, Any]]] = []
        self.initialize_protocol = 1
        self.error: tuple[str, int, str] | None = None
        self.close_on: str | None = None

    def start(self) -> None:
        threading.Thread(target=self.serve_forever, daemon=True).start()

    def answer(self, method: str) -> dict[str, Any]:
        if method == "initialize":
            return {
                "protocol": self.initialize_protocol,
                "app": {
                    "name": "Counter",
                    "pid": 123,
                    "kivact_version": "0.1.0",
                    "build": "debug",
                    "exe": "counter.exe",
                    "cwd": "",
                    "workspace_root": "",
                },
            }
        if method == "list_windows":
            return {
                "windows": [
                    {
                        "id": 1,
                        "title": "Main",
                        "bounds": {"x": 0.0, "y": 0.0, "width": 800.0, "height": 600.0},
                        "scale": 1.0,
                        "focused": True,
                        "mode": "normal",
                    }
                ]
            }
        if method in NODE_METHODS:
            return {"node": ELEMENT_JSON}
        if method in NODES_METHODS:
            return {"nodes": [ELEMENT_JSON]}
        if method == "get_tree":
            return {"window": 1, "root": 1, "revision": 1}
        if method == "get_node":
            return NODE_DETAIL_JSON
        if method in ("lint_layout", "audit_a11y"):
            return {"window": 1}
        if method == "screenshot":
            return {"png_base64": PNG_BASE64, "width": 1, "height": 1}
        if method == "set_style_override":
            return {"id": 1, "scope": {"kind": "node", "node": 2}, "property": "padding", "value": "12"}
        if method == "add_comment":
            return {"id": "C1", "window_title": "Main", "window": 1, "target": {"kind": "window"}}
        if method == "list_turns":
            return {
                "turns": [
                    {
                        "turn": 3,
                        "started_ns": 0,
                        "duration_ns": 1000,
                        "wasted": {"equal_writes": 0, "equal_memos": 0, "unchanged_props": 0},
                        "truncated": False,
                    }
                ]
            }
        if method == "export_review":
            return {"json": "x/review.json", "markdown": "x/review.md"}
        return {}


class FakeHandler(socketserver.StreamRequestHandler):
    def handle(self) -> None:
        server = self.server
        if not isinstance(server, FakeAgent):
            return
        for raw in self.rfile:
            message = json.loads(raw)
            method = str(message.get("method", ""))
            params = message.get("params") or {}
            server.requests.append((method, params))
            params_class = PARAM_CLASSES.get(method)
            if params_class is not None:
                try:
                    params_class.from_json(params)
                except (KeyError, TypeError):
                    server.failures.append(method)
                    self._write(message, error={"code": int(ErrorCode.INVALID_PARAMS), "message": "invalid params"})
                    continue
            if server.close_on == method:
                return
            if server.error is not None and server.error[0] == method:
                _, code, text = server.error
                self._write(message, error={"code": code, "message": text})
                continue
            self._write(message, result=server.answer(method))

    def _write(
        self,
        message: dict[str, Any],
        *,
        result: dict[str, Any] | None = None,
        error: dict[str, Any] | None = None,
    ) -> None:
        reply: dict[str, Any] = {"jsonrpc": "2.0", "id": message.get("id")}
        if error is not None:
            reply["error"] = error
        else:
            reply["result"] = result
        self.wfile.write((json.dumps(reply) + "\n").encode("utf-8"))
        self.wfile.flush()


class ClientTest(unittest.TestCase):
    def setUp(self) -> None:
        self.agent = FakeAgent()
        self.agent.start()
        self.addCleanup(self.agent.server_close)
        self.addCleanup(self.agent.shutdown)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name) / "123-456.json"
        self.path.write_text(self._discovery_json(1), "utf-8")

    def _discovery_json(self, protocol: int) -> str:
        return json.dumps(
            {
                "protocol": protocol,
                "pid": 123,
                "app_name": "Counter",
                "kivact_version": "0.1.0",
                "port": self.agent.server_address[1],
                "token": "secret",
                "started_unix_ms": 0,
                "workspace_root": "",
            }
        )

    def connect_app(self) -> App:
        app = connect(target=self.path)
        self.addCleanup(app.close)
        return app

    def test_connect_uses_the_discovery_file(self) -> None:
        app = self.connect_app()
        self.assertEqual(app.info.name, "Counter")
        self.assertEqual(self.agent.failures, [])

    def test_get_by_role_omits_name(self) -> None:
        app = self.connect_app()
        window = app.window("Main")
        element = window.get_by_role(Role.BUTTON, "Increment")
        self.assertEqual(element.info.name, "Increment")
        window.get_by_role(Role.BUTTON)
        params = [params for method, params in self.agent.requests if method == "get_by_role"]
        self.assertEqual(params[0]["name"], "Increment")
        self.assertNotIn("name", params[1])
        self.assertEqual(self.agent.failures, [])

    def test_error_codes_map_to_exceptions(self) -> None:
        app = self.connect_app()
        window = app.window("Main")
        self.agent.error = ("get_by_role", int(ErrorCode.NO_MATCH), "none")
        with self.assertRaises(NotFound):
            window.get_by_role(Role.BUTTON)
        self.agent.error = ("get_by_role", int(ErrorCode.MULTIPLE_MATCHES), "many")
        with self.assertRaises(MultipleFound):
            window.get_by_role(Role.BUTTON)
        self.agent.error = ("find_by_role", int(ErrorCode.TIMED_OUT), "late")
        with self.assertRaises(Timeout):
            window.find_by_role(Role.BUTTON)
        self.assertEqual(self.agent.failures, [])

    def test_connect_rejects_a_different_protocol(self) -> None:
        self.agent.initialize_protocol = 2
        with self.assertRaises(DevtoolsError) as caught:
            connect(target=self.path)
        self.assertEqual(caught.exception.code, int(ErrorCode.VERSION_MISMATCH))
        self.assertEqual(self.agent.failures, [])

    def test_closed_connection_fails_fast(self) -> None:
        self.agent.close_on = "list_windows"
        rpc = RpcConnection(self.agent.server_address[1], timeout=0.5)
        self.addCleanup(rpc.close)
        started = time.monotonic()
        with self.assertRaises(DevtoolsError) as caught:
            rpc.call("list_windows", {})
        self.assertLess(time.monotonic() - started, 0.5)
        self.assertEqual(caught.exception.code, int(ErrorCode.INTERNAL_ERROR))
        self.assertEqual(self.agent.failures, [])

    def test_recording_collects_turns(self) -> None:
        app = self.connect_app()
        button = app.window("Main").get_by_role(Role.BUTTON, "Increment")
        with app.record() as recording:
            button.click()
        methods = [
            method
            for method, _ in self.agent.requests
            if method in {"list_turns", "start_recording", "inject_input", "stop_recording"}
        ]
        self.assertEqual(
            methods,
            ["list_turns", "start_recording", "inject_input", "stop_recording", "list_turns"],
        )
        last = [params for method, params in self.agent.requests if method == "list_turns"][-1]
        self.assertEqual(last["since"], 3)
        self.assertEqual([turn.turn for turn in recording.turns], [3])
        self.assertEqual(self.agent.failures, [])

    def test_every_public_method_runs(self) -> None:
        app = self.connect_app()
        self.assertEqual(len(app.windows()), 1)
        window = app.window("Main")
        app.record()
        app.clear_highlight()
        self.assertEqual(app.export_review(), [Path("x/review.json"), Path("x/review.md")])
        app.show_table("Buttons", ["name"], [["Increment"]])
        self.assertIsInstance(window.get_by_role(Role.BUTTON), Element)
        self.assertIsInstance(window.query_by_role(Role.BUTTON), Element)
        self.assertIsInstance(window.find_by_role(Role.BUTTON), Element)
        self.assertEqual(len(window.all_by_role(Role.BUTTON)), 1)
        self.assertIsInstance(window.get_by_text("Increment"), Element)
        self.assertIsInstance(window.query_by_text("Increment"), Element)
        self.assertIsInstance(window.find_by_text("Increment"), Element)
        self.assertEqual(len(window.all_by_text("Increment")), 1)
        self.assertIsInstance(window.get_by_test_id("increment"), Element)
        self.assertIsInstance(window.query_by_test_id("increment"), Element)
        self.assertIsInstance(window.find_by_test_id("increment"), Element)
        self.assertEqual(len(window.all_by_test_id("increment")), 1)
        window.tree()
        window.type_text("abc")
        window.press_key("Ctrl+Enter")
        self.assertTrue(window.screenshot().startswith(b"\x89PNG"))
        self.assertEqual(window.add_comment("hello").id, "C1")
        window.lint_layout()
        window.audit_a11y()
        element = window.get_by_role(Role.BUTTON)
        element.detail()
        element.highlight()
        element.click()
        self.assertEqual(element.set_style_override("padding", "12").property, "padding")
        self.assertEqual(element.add_comment("hello").id, "C1")
        self.assertEqual(self.agent.failures, [])


class DiscoveryDirTest(unittest.TestCase):
    def test_discovery_dir_cases(self) -> None:
        temp = Path("temp")
        self.assertEqual(
            _discovery.discovery_dir({"LOCALAPPDATA": "local"}, temp),
            Path("local") / "kivact" / "devtools",
        )
        self.assertEqual(
            _discovery.discovery_dir({"XDG_RUNTIME_DIR": "run"}, temp),
            Path("run") / "kivact" / "devtools",
        )
        self.assertEqual(
            _discovery.discovery_dir({"HOME": "home"}, temp),
            Path("home") / ".cache" / "kivact" / "devtools",
        )
        self.assertEqual(_discovery.discovery_dir({}, temp), temp / "kivact-devtools")
        self.assertEqual(
            _discovery.discovery_dir({"LOCALAPPDATA": ""}, temp),
            temp / "kivact-devtools",
        )


if __name__ == "__main__":
    unittest.main()
