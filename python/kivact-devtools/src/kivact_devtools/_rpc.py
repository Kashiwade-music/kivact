"""JSON-RPC over a loopback socket: one request per line, one response per line."""
from __future__ import annotations

import json, queue, socket, threading
from typing import Any

from ._generated import ErrorCode
from .errors import DevtoolsError, Timeout, error_from_rpc

_CLOSED = int(ErrorCode.INTERNAL_ERROR)


class RpcConnection:
    def __init__(self, port: int, *, timeout: float = 30.0) -> None:
        self._sock = socket.create_connection(("127.0.0.1", port), timeout=5.0)
        self._sock.settimeout(None)
        self._file = self._sock.makefile("rwb")
        self._timeout = timeout
        self._lock = threading.Lock()
        self._next_id = 1
        # The waiting calls, or why the connection closed. Never both.
        self._calls: dict[int, queue.Queue[dict[str, Any]]] | DevtoolsError = {}
        threading.Thread(target=self._read_loop, daemon=True).start()

    def call(self, method: str, params: dict[str, Any], *, timeout: float | None = None) -> Any:
        """Send one call and wait for its answer; `timeout` seconds, or the default for this method."""
        limit = self._answer_timeout(method, params) if timeout is None else timeout
        replies: queue.Queue[dict[str, Any]] = queue.Queue(maxsize=1)
        with self._lock:
            if isinstance(self._calls, DevtoolsError):
                raise self._calls
            request_id = self._next_id
            self._next_id += 1
            self._calls[request_id] = replies
            line = json.dumps({"jsonrpc": "2.0", "id": request_id, "method": method, "params": params}) + "\n"
            try:
                self._file.write(line.encode("utf-8"))
                self._file.flush()
            except OSError as error:
                self._fail(DevtoolsError(_CLOSED, f"connection closed: {error}"))
        try:
            reply = replies.get(timeout=limit)
        except queue.Empty:
            self._forget(request_id)
            raise Timeout(int(ErrorCode.TIMED_OUT), f"no answer to {method} in {limit:g} s") from None
        if "error" in reply:
            raise error_from_rpc(int(reply["error"]["code"]), str(reply["error"]["message"]))
        return reply.get("result")

    def _answer_timeout(self, method: str, params: dict[str, Any]) -> float:
        """The default timeout, or the agent's own wait plus 5 s when that is longer (`find_by_*`: timeout_ms; `list_turns`: wait_ms)."""
        if method == "list_turns":
            wait_ms = params.get("wait_ms")
        elif method.startswith("find_by_"):
            wait_ms = params.get("timeout_ms")
        else:
            return self._timeout
        if not isinstance(wait_ms, int):
            return self._timeout
        return max(self._timeout, wait_ms / 1000 + 5.0)

    def close(self) -> None:
        with self._lock:
            self._fail(DevtoolsError(_CLOSED, "connection closed"))
        self._sock.close()

    def _forget(self, request_id: int) -> None:
        """Forget a call that timed out, so a late reply is ignored."""
        with self._lock:
            if isinstance(self._calls, dict):
                self._calls.pop(request_id, None)

    def _fail(self, error: DevtoolsError) -> None:
        """Close: give every waiting call `error`. The caller holds `self._lock`."""
        if isinstance(self._calls, DevtoolsError):
            return
        waiting, self._calls = self._calls, error
        for replies in waiting.values():
            replies.put({"error": {"code": error.code, "message": error.message}})

    def _read_loop(self) -> None:
        reason = "connection closed"
        try:
            for raw in self._file:
                message = json.loads(raw)
                if not isinstance(message, dict):
                    raise ValueError("a message is not a JSON object")
                request_id = message.get("id")
                if request_id is None:
                    continue  # a notification: this client does not subscribe
                with self._lock:
                    replies = self._calls.pop(request_id, None) if isinstance(self._calls, dict) else None
                if replies is not None:
                    replies.put(message)
        except (OSError, ValueError) as error:
            reason = f"connection closed: {error}"
        with self._lock:
            self._fail(DevtoolsError(_CLOSED, reason))
