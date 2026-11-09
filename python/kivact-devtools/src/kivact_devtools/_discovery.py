"""Discovery of running DevTools targets."""
from __future__ import annotations

import contextlib
import json
import os
import socket
import tempfile
from collections.abc import Mapping
from concurrent.futures import ThreadPoolExecutor
from enum import Enum
from pathlib import Path

from ._generated import DiscoveryFile

# A refused loopback connection takes about 2 s on Windows, so a shorter limit would mistake it for a timeout.
PROBE_TIMEOUT = 3.0


def discovery_dir(env: Mapping[str, str], temp_dir: Path) -> Path:
    """The directory of the discovery files, from the environment and a temp directory."""
    local_app_data = env.get("LOCALAPPDATA")
    if local_app_data:
        return Path(local_app_data) / "kivact" / "devtools"
    runtime_dir = env.get("XDG_RUNTIME_DIR")
    if runtime_dir:
        return Path(runtime_dir) / "kivact" / "devtools"
    home = env.get("HOME")
    if home:
        return Path(home) / ".cache" / "kivact" / "devtools"
    return temp_dir / "kivact-devtools"


def read_target(path: Path) -> DiscoveryFile:
    """Read one discovery file."""
    return DiscoveryFile.from_json(json.loads(path.read_text("utf-8")))


class _Probe(Enum):
    """What a connection attempt to a target's port found."""

    REACHABLE = "reachable"
    REFUSED = "refused"
    UNKNOWN = "unknown"


def _probe(port: int) -> _Probe:
    """Try the port once, waiting at most `PROBE_TIMEOUT` seconds."""
    try:
        with socket.create_connection(("127.0.0.1", port), timeout=PROBE_TIMEOUT):
            return _Probe.REACHABLE
    except ConnectionRefusedError:
        return _Probe.REFUSED
    except (OSError, OverflowError):
        return _Probe.UNKNOWN


def list_targets() -> list[tuple[Path, DiscoveryFile]]:
    """The reachable targets. A file whose port refuses the connection is deleted; unreadable and
    timed-out files are skipped. Every file is probed on its own thread."""
    directory = discovery_dir(os.environ, Path(tempfile.gettempdir()))
    candidates: list[tuple[Path, DiscoveryFile]] = []
    for path in sorted(directory.glob("*.json")):
        try:
            candidates.append((path, read_target(path)))
        except (OSError, ValueError, KeyError, TypeError):
            continue
    with ThreadPoolExecutor(max_workers=max(1, len(candidates))) as pool:
        outcomes = list(pool.map(_probe, (file.port for _, file in candidates)))
    targets: list[tuple[Path, DiscoveryFile]] = []
    for (path, file), outcome in zip(candidates, outcomes):
        if outcome is _Probe.REACHABLE:
            targets.append((path, file))
        elif outcome is _Probe.REFUSED:
            with contextlib.suppress(OSError):
                path.unlink()
    return targets


def target_from_env() -> Path | None:
    """The discovery file named by `KIVACT_DEVTOOLS_TARGET`."""
    value = os.environ.get("KIVACT_DEVTOOLS_TARGET")
    return Path(value) if value else None
