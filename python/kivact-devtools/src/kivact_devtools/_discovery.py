"""Discovery of running DevTools targets (design 4.2.2)."""
from __future__ import annotations

import json
import os
import socket
import tempfile
from collections.abc import Mapping
from pathlib import Path

from ._generated import DiscoveryFile


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


def list_targets() -> list[tuple[Path, DiscoveryFile]]:
    """Every discovery file whose port answers; unreadable and refused files are skipped."""
    directory = discovery_dir(os.environ, Path(tempfile.gettempdir()))
    targets: list[tuple[Path, DiscoveryFile]] = []
    for path in sorted(directory.glob("*.json")):
        try:
            file = read_target(path)
        except (OSError, ValueError, KeyError, TypeError):
            continue
        try:
            with socket.create_connection(("127.0.0.1", file.port), timeout=0.1):
                pass
        except OSError:
            continue
        targets.append((path, file))
    return targets


def target_from_env() -> Path | None:
    """The discovery file named by `KIVACT_DEVTOOLS_TARGET`."""
    value = os.environ.get("KIVACT_DEVTOOLS_TARGET")
    return Path(value) if value else None
