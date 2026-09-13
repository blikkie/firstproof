#!/usr/bin/env python3
"""
First Proof MCP Server (Model Context Protocol)

Provides diagnostic introspection, live viewport pixel sampling, canvas capture,
and automated dual-engine parity verification for First Proof.

Conforms to standard MCP stdio transport.
"""

import sys
import os
import json
import socket
import subprocess

SOCKET_PATH = "/tmp/firstproof-debug.sock"
WORKSPACE_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
INSPECT_BIN = os.path.join(WORKSPACE_ROOT, "target", "debug", "inspect")

def send_socket_command(cmd: dict, timeout: float = 2.0) -> dict:
    if not os.path.exists(SOCKET_PATH):
        return {"status": "error", "message": f"FirstProofApp debug socket not found at {SOCKET_PATH}. Is the app running?"}
    try:
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.settimeout(timeout)
        s.connect(SOCKET_PATH)
        s.sendall(json.dumps(cmd).encode("utf-8"))
        data = b""
        while True:
            chunk = s.recv(65536)
            if not chunk:
                break
            data += chunk
            if b"\n" in chunk:
                break
        s.close()
        return json.loads(data.decode("utf-8"))
    except Exception as e:
        return {"status": "error", "message": str(e)}

TOOLS = [
    {
        "name": "firstproof_get_state",
        "description": "Get current state from FirstProofApp (selected photo path, XMP edits, lens calibration, canvas mode).",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    {
        "name": "firstproof_open_detail_view",
        "description": "Switch FirstProofApp to detail inspection view, optionally selecting a specific photo path.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "photo_path": {
                    "type": "string",
                    "description": "Absolute path to the photo to select and display in detail view. If omitted, opens currently active photo."
                }
            },
            "required": []
        }
    },
    {
        "name": "firstproof_sample_viewport",
        "description": "Sample exact pixel RGB colors from the live Metal viewport at specified normalized UV coordinates [0.0 .. 1.0].",
        "inputSchema": {
            "type": "object",
            "properties": {
                "points": {
                    "type": "array",
                    "items": {
                        "type": "array",
                        "items": {"type": "number"},
                        "minItems": 2,
                        "maxItems": 2
                    },
                    "description": "List of normalized [u, v] coordinates to sample, e.g. [[0.5, 0.5], [0.35, 0.55]]."
                }
            },
            "required": ["points"]
        }
    },
    {
        "name": "firstproof_capture_canvas",
        "description": "Capture the current Metal canvas rendering directly to an image artifact file.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "output_path": {
                    "type": "string",
                    "description": "Destination file path (e.g. /tmp/canvas.png). Default: /tmp/firstproof_viewport.png"
                }
            },
            "required": []
        }
    },
    {
        "name": "firstproof_audit_parity",
        "description": "Run an automated dual-engine mathematical parity audit between simulated GPU viewport math and CPU export pipeline on a RAW file.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "photo_path": {
                    "type": "string",
                    "description": "Absolute path to RAW photo. If omitted, uses the currently selected photo in FirstProofApp."
                }
            },
            "required": []
        }
    }
]

def handle_tool_call(name: str, args: dict) -> str:
    if name == "firstproof_get_state":
        res = send_socket_command({"method": "get_state"})
        return json.dumps(res, indent=2)

    elif name == "firstproof_open_detail_view":
        path = args.get("photo_path")
        res = send_socket_command({"method": "open_detail_view", "params": {"photo_path": path}})
        return json.dumps(res, indent=2)

    elif name == "firstproof_sample_viewport":
        pts = args.get("points", [])
        res = send_socket_command({"method": "sample_viewport", "params": {"points": pts}})
        return json.dumps(res, indent=2)

    elif name == "firstproof_capture_canvas":
        out_path = args.get("output_path", "/tmp/firstproof_viewport.png")
        res = send_socket_command({"method": "capture_canvas", "params": {"output_path": out_path}})
        return json.dumps(res, indent=2)

    elif name == "firstproof_audit_parity":
        path = args.get("photo_path")
        if not path:
            state = send_socket_command({"method": "get_state"})
            path = state.get("selectedPhoto")

        # Run inspect binary
        bin_path = INSPECT_BIN
        if not os.path.exists(bin_path):
            base_cmd = ["cargo", "run", "--manifest-path", os.path.join(WORKSPACE_ROOT, "firstproof-core", "Cargo.toml"), "--bin", "inspect", "--"]
        else:
            base_cmd = [bin_path]

        if path:
            cmd = base_cmd + ["audit-parity", path]
        else:
            cmd = base_cmd + ["audit-synthetic"]

        try:
            proc = subprocess.run(cmd, capture_output=True, text=True, check=True)
            return proc.stdout
        except subprocess.CalledProcessError as e:
            return f"Audit failed with exit code {e.returncode}:\nStdout: {e.stdout}\nStderr: {e.stderr}"

    return json.dumps({"error": f"Unknown tool: {name}"})

def main():
    while True:
        line = sys.stdin.readline()
        if not line:
            break
        line = line.strip()
        if not line:
            continue

        try:
            msg = json.loads(line)
        except Exception:
            continue

        msg_id = msg.get("id")
        method = msg.get("method")

        if method == "initialize":
            resp = {
                "jsonrpc": "2.0",
                "id": msg_id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "firstproof-mcp", "version": "0.1.0"}
                }
            }
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()

        elif method == "notifications/initialized":
            pass

        elif method == "tools/list":
            resp = {
                "jsonrpc": "2.0",
                "id": msg_id,
                "result": {"tools": TOOLS}
            }
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()

        elif method == "tools/call":
            params = msg.get("params", {})
            name = params.get("name")
            arguments = params.get("arguments", {})
            result_text = handle_tool_call(name, arguments)
            resp = {
                "jsonrpc": "2.0",
                "id": msg_id,
                "result": {
                    "content": [{"type": "text", "text": result_text}]
                }
            }
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()

if __name__ == "__main__":
    main()
