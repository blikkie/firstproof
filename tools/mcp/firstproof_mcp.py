#!/usr/bin/env python3
"""
First Proof MCP Server (Model Context Protocol)

Provides diagnostic introspection, live viewport pixel sampling, canvas capture,
automated dual-engine parity verification, non-destructive darkroom edits,
archival DNG conversion, and headless develop workflows for First Proof.

Conforms to standard MCP stdio JSON-RPC 2.0 transport.
"Pure capture. Honest color. Zero digital plastic."
"""

import sys
import os
import json
import socket
import subprocess

SOCKET_PATH = "/tmp/firstproof-debug.sock"
WORKSPACE_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CLI_BIN = os.path.join(WORKSPACE_ROOT, "target", "debug", "firstproof-cli")
INSPECT_BIN = os.path.join(WORKSPACE_ROOT, "target", "debug", "inspect")

def send_socket_command(cmd: dict, timeout: float = 3.0) -> dict:
    """Send JSON command to running FirstProofApp instance via Unix domain socket."""
    if not os.path.exists(SOCKET_PATH):
        return {"status": "error", "message": f"FirstProofApp socket not found at {SOCKET_PATH}. Is the app running?"}
    try:
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.settimeout(timeout)
        s.connect(SOCKET_PATH)
        s.sendall(json.dumps(cmd).encode("utf-8") + b"\n")
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

def run_cli_command(cmd: dict) -> dict:
    """Execute command via firstproof-cli headless command runner."""
    json_str = json.dumps(cmd)
    bin_path = CLI_BIN
    if os.path.exists(bin_path):
        base_cmd = [bin_path, "command", json_str]
    else:
        base_cmd = [
            "cargo", "run", "--manifest-path",
            os.path.join(WORKSPACE_ROOT, "firstproof-core", "Cargo.toml"),
            "--bin", "firstproof-cli", "--", "command", json_str
        ]
    try:
        proc = subprocess.run(base_cmd, capture_output=True, text=True, check=True)
        return json.loads(proc.stdout.strip())
    except subprocess.CalledProcessError as e:
        return {"status": "error", "message": f"CLI command failed: {e.stderr or e.stdout}"}
    except Exception as e:
        return {"status": "error", "message": str(e)}

def dispatch_command(cmd: dict, prefer_live: bool = True) -> dict:
    """Route command to live AppKit socket if available; fall back to headless CLI runner."""
    if prefer_live and os.path.exists(SOCKET_PATH):
        res = send_socket_command(cmd)
        if res.get("status") == "ok":
            return res
    return run_cli_command(cmd)

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
        "name": "firstproof_apply_edit_delta",
        "description": "Apply non-destructive adjustment deltas (exposure, contrast, highlights, shadows, saturation, temperature, etc.) to the darkroom.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "exposure": {"type": "number", "description": "Exposure in EV (-5.0 to +5.0)"},
                "contrast": {"type": "number", "description": "Contrast (-100 to +100)"},
                "highlights": {"type": "number", "description": "Highlights recovery (-100 to +100)"},
                "shadows": {"type": "number", "description": "Shadows lift (-100 to +100)"},
                "whites": {"type": "number", "description": "Whites clipping (-100 to +100)"},
                "blacks": {"type": "number", "description": "Blacks clipping (-100 to +100)"},
                "saturation": {"type": "number", "description": "Saturation (-100 to +100)"},
                "vibrance": {"type": "number", "description": "Vibrance (-100 to +100)"},
                "temperature": {"type": "number", "description": "White balance Kelvin (2000 to 20000)"},
                "tint": {"type": "number", "description": "White balance green/magenta tint (-100 to +100)"}
            },
            "required": []
        }
    },
    {
        "name": "firstproof_set_emulsion",
        "description": "Select a curated physical film stock emulsion archetype (e.g. Rochester Portrait 400, Ashigara Chrome 50, Mobberley Silver 400).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "archetype": {
                    "type": "string",
                    "description": "Stock name: 'rochester-portrait-400', 'ashigara-chrome-50', 'mobberley-silver-400', 'none'"
                },
                "format": {
                    "type": "string",
                    "description": "Film gate format: '35mm', '120', '4x5', 'super8', '16mm'"
                },
                "exposure": {
                    "type": "number",
                    "description": "Emulsion exposure push/pull in EV"
                }
            },
            "required": ["archetype"]
        }
    },
    {
        "name": "firstproof_set_print_media",
        "description": "Select a physical print media archetype (Baryta, Warmtone Chlorobromide, Platinum/Palladium, Cyanotype) and paper grade.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "media": {
                    "type": "string",
                    "description": "Print medium: 'baryta', 'warmtone', 'platinum', 'cyanotype', 'bamako', 'none'"
                },
                "paper_grade": {
                    "type": "number",
                    "description": "Contrast grade (0.0 to 5.0, default 2.0)"
                },
                "base_warmth": {
                    "type": "number",
                    "description": "Paper base warmth (0.0 to 1.0)"
                }
            },
            "required": ["media"]
        }
    },
    {
        "name": "firstproof_reset_workspace",
        "description": "Reset active edits (all adjustments or a specific workspace tab: 'light', 'color', 'detail', 'emulsion').",
        "inputSchema": {
            "type": "object",
            "properties": {
                "category": {
                    "type": "string",
                    "description": "Workspace tab to reset: 'light', 'color', 'detail', 'emulsion', or omit for all."
                }
            },
            "required": []
        }
    },
    {
        "name": "firstproof_inspect_metadata",
        "description": "Inspect camera EXIF, lens calibration, CFA Bayer layout, and XMP edits for a RAW or image file.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "photo_path": {
                    "type": "string",
                    "description": "Path to RAW or image file. If omitted, inspects currently active photo in AppKit."
                }
            },
            "required": []
        }
    },
    {
        "name": "firstproof_develop_photo",
        "description": "Develop a camera RAW or image file headlessly to 16-bit TIFF or JPEG using First Proof's AgX color science.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "input": {"type": "string", "description": "Source RAW file path"},
                "output": {"type": "string", "description": "Destination TIFF/JPEG path"},
                "fast": {"type": "boolean", "description": "Fast standard demosaicing tier for drafts"}
            },
            "required": ["input"]
        }
    },
    {
        "name": "firstproof_convert_dng",
        "description": "Convert proprietary camera RAW (.CR2, .NEF, .ARW) to archival DNG with automatic AgX preview and bit-for-bit SHA-256 verification.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "input": {"type": "string", "description": "Source RAW file path"},
                "output": {"type": "string", "description": "Target DNG file path"}
            },
            "required": ["input"]
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
        res = send_socket_command({"command": "get_state"})
        return json.dumps(res, indent=2)

    elif name == "firstproof_open_detail_view":
        path = args.get("photo_path")
        res = send_socket_command({"command": "select_photo", "path": path})
        return json.dumps(res, indent=2)

    elif name == "firstproof_sample_viewport":
        pts = args.get("points", [])
        res = send_socket_command({"command": "sample_viewport", "points": pts})
        return json.dumps(res, indent=2)

    elif name == "firstproof_capture_canvas":
        out_path = args.get("output_path", "/tmp/firstproof_viewport.png")
        res = send_socket_command({"command": "capture_canvas", "output_path": out_path})
        return json.dumps(res, indent=2)

    elif name == "firstproof_apply_edit_delta":
        res = dispatch_command({"command": "apply_edit_delta", "delta": args})
        return json.dumps(res, indent=2)

    elif name == "firstproof_set_emulsion":
        res = dispatch_command({
            "command": "set_emulsion",
            "archetype": args.get("archetype"),
            "format": args.get("format"),
            "exposure": args.get("exposure")
        })
        return json.dumps(res, indent=2)

    elif name == "firstproof_set_print_media":
        res = dispatch_command({
            "command": "set_print_media",
            "media": args.get("media"),
            "paper_grade": args.get("paper_grade"),
            "base_warmth": args.get("base_warmth")
        })
        return json.dumps(res, indent=2)

    elif name == "firstproof_reset_workspace":
        res = dispatch_command({
            "command": "reset_workspace",
            "category": args.get("category")
        })
        return json.dumps(res, indent=2)

    elif name == "firstproof_inspect_metadata":
        res = dispatch_command({
            "command": "inspect_metadata",
            "path": args.get("photo_path")
        })
        return json.dumps(res, indent=2)

    elif name == "firstproof_develop_photo":
        res = dispatch_command({
            "command": "develop_photo",
            "input": args.get("input"),
            "output": args.get("output"),
            "fast": args.get("fast")
        }, prefer_live=False)
        return json.dumps(res, indent=2)

    elif name == "firstproof_convert_dng":
        res = dispatch_command({
            "command": "convert_to_dng",
            "input": args.get("input"),
            "output": args.get("output")
        }, prefer_live=False)
        return json.dumps(res, indent=2)

    elif name == "firstproof_audit_parity":
        path = args.get("photo_path")
        if not path:
            state = send_socket_command({"command": "get_state"})
            path = state.get("selectedPhoto")

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
                    "serverInfo": {"name": "firstproof-mcp", "version": "0.2.0"}
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
