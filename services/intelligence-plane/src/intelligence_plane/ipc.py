# Intelligence Plane — IPC client for communicating with the Rust core engine.

"""
IPC client for communicating with the Rust core engine over AF_UNIX socket.

Uses the SRS-defined framing: 4-byte big-endian unsigned length header,
UTF-8 JSON payload, with explicit schema version.
"""

from __future__ import annotations

import json
import socket
import struct
from typing import Any

MAX_SIGNAL_FRAME_BYTES = 1_048_576  # 1 MiB
FRAME_HEADER_SIZE = 4


class IpcClient:
    """Client for communicating with the Rust core engine over AF_UNIX."""

    def __init__(self, socket_path: str, timeout: float = 30.0) -> None:
        self.socket_path = socket_path
        self.timeout = timeout
        self._sock: socket.socket | None = None

    def connect(self) -> None:
        """Connect to the Rust core engine socket."""
        self._sock = socket.socket(getattr(socket, "AF_UNIX", socket.AF_INET), socket.SOCK_STREAM)  # type: ignore
        self._sock.settimeout(self.timeout)
        self._sock.connect(self.socket_path)

    def send(self, payload: dict[str, Any]) -> dict[str, Any]:
        """Send a framed message and receive the response."""
        if self._sock is None:
            self.connect()

        json_bytes = json.dumps(payload, ensure_ascii=False).encode("utf-8")

        if len(json_bytes) > MAX_SIGNAL_FRAME_BYTES:
            raise ValueError(
                f"Payload size {len(json_bytes)} exceeds maximum {MAX_SIGNAL_FRAME_BYTES}"
            )

        # 4-byte big-endian length header
        header = struct.pack(">I", len(json_bytes))
        self._sock.sendall(header + json_bytes)  # type: ignore[union-attr]

        # Read response header
        resp_header = self._recv_exact(FRAME_HEADER_SIZE)
        resp_len = struct.unpack(">I", resp_header)[0]

        if resp_len > MAX_SIGNAL_FRAME_BYTES:
            raise ValueError(f"Response size {resp_len} exceeds maximum {MAX_SIGNAL_FRAME_BYTES}")

        resp_bytes = self._recv_exact(resp_len)
        return json.loads(resp_bytes.decode("utf-8"))

    def _recv_exact(self, n: int) -> bytes:
        """Receive exactly n bytes from the socket."""
        data = b""
        while len(data) < n:
            chunk = self._sock.recv(n - len(data))  # type: ignore[union-attr]
            if not chunk:
                raise ConnectionError("Connection closed by peer")
            data += chunk
        return data

    def close(self) -> None:
        """Close the connection."""
        if self._sock is not None:
            self._sock.close()
            self._sock = None
