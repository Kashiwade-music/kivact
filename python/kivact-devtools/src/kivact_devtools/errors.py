"""Errors raised by the DevTools client."""
from __future__ import annotations

from ._generated import ErrorCode


class DevtoolsError(Exception):
    """An error answer from the agent, or a closed connection (`ErrorCode.INTERNAL_ERROR`)."""

    def __init__(self, code: int, message: str) -> None:
        super().__init__(f"{code}: {message}")
        self.code = code
        self.message = message


class NotFound(DevtoolsError): """`NODE_NOT_FOUND`, `WINDOW_NOT_FOUND` or `NO_MATCH`."""
class MultipleFound(DevtoolsError): """-32013."""
class Timeout(DevtoolsError): """`TIMED_OUT`, or no answer within the call timeout."""


def error_from_rpc(code: int, message: str) -> DevtoolsError:
    if code in (ErrorCode.NODE_NOT_FOUND, ErrorCode.WINDOW_NOT_FOUND, ErrorCode.NO_MATCH):
        return NotFound(code, message)
    if code == ErrorCode.MULTIPLE_MATCHES:
        return MultipleFound(code, message)
    if code == ErrorCode.TIMED_OUT:
        return Timeout(code, message)
    return DevtoolsError(code, message)
