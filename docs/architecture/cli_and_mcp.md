# First Proof Headless Command Bus, CLI & MCP Architecture

> *"Pure capture. Honest color. Zero digital plastic. Print & web first."*

First Proof provides a unified, cross-platform command architecture connecting three execution domains:
1. **Interactive macOS App (`FirstProofApp`)**: High-performance AppKit & Metal viewport listening on a local Unix domain socket (`/tmp/firstproof-debug.sock`).
2. **Headless Studio Automation CLI (`firstproof-cli`)**: Rust command-line binary supporting flag-based commands, batch processing, and piped JSON-RPC / canonical command streams.
3. **Web Darkroom (`web` / `firstproof-wasm`)**: WebAssembly darkroom playground consuming identical commands through `execute_darkroom_command`.
4. **Model Context Protocol Bridge (`firstproof_mcp.py`)**: Production-grade MCP server enabling automated darkroom operations, diagnostic telemetry, and mathematical parity assertions.

---

## 1. Canonical Command Schema (`DarkroomCommand`)

The authoritative command schema is defined in Rust in `firstproof-engine::command::DarkroomCommand`. All commands, parameters, and deltas are strongly typed and serialize to standard JSON.

### Command Catalog

| Command | Tag (`command`) | Parameters | Description |
| :--- | :--- | :--- | :--- |
| **Get State** | `get_state` | *None* | Queries selected photo, canvas mode, lens calibration, and current `EditSettings`. |
| **Select Photo** | `select_photo` | `path: String` | Switches selection in filmstrip and enters detail view. |
| **Set Canvas Mode** | `set_canvas_mode` | `mode: "grid" \| "detail"` | Toggles grid browser vs full-screen darkroom canvas. |
| **Sample Pixel** | `sample_pixel` | `points: [[u, v], ...]` | Samples exact RGB values at normalized $[0.0 .. 1.0]$ coordinates. |
| **Capture Canvas** | `capture_canvas` | `output_path: String` | Captures live Metal viewport snapshot to a PNG file. |
| **Set Exposure** | `set_exposure` | `ev: f32` | Sets exposure in EV ($-5.0$ to $+5.0$). |
| **Set Temperature** | `set_temperature` | `kelvin: f64` | Sets white balance Kelvin ($2000\text{ K}$ to $20000\text{ K}$). |
| **Set Tint** | `set_tint` | `tint: f32` | Sets white balance green/magenta tint ($-100.0$ to $+100.0$). |
| **Set Tone Profile** | `set_tone_profile` | `profile: ColorProfileMode` | Sets tone profile (`agx-vivid`, `agx-filmic`, `camera-standard`, etc.). |
| **Set Emulsion** | `set_emulsion` | `archetype: String, format?: String, exposure?: f32` | Selects chemical film stock archetype and format gate. |
| **Set Print Media** | `set_print_media` | `media: String, paper_grade?: f32, base_warmth?: f32` | Selects physical print media archetype and paper grading. |
| **Apply Edit Delta** | `apply_edit_delta` | `delta: EditDelta` | Applies non-destructive adjustments to active settings. |
| **Reset Workspace** | `reset_workspace` | `category?: "light" \| "color" \| "detail" \| "emulsion"` | Resets adjustments for a specific tab or entire photo. |
| **Inspect Metadata** | `inspect_metadata` | `path?: String` | Extracts EXIF, CFA Bayer layout, and camera calibration. |
| **Develop Photo** | `develop_photo` | `input: String, output?: String, fast?: bool` | Develops photo to 16-bit TIFF or JPEG via AgX color science. |
| **Convert to DNG** | `convert_to_dng` | `input: String, output?: String` | Converts proprietary RAW to archival DNG with verification. |
| **Audit Parity** | `audit_parity` | `photo_path?: String` | Runs dual-engine mathematical parity audit. |

---

## 2. Headless CLI Integration (`firstproof-cli command`)

The CLI exposes the `command` (or `cmd`) subcommand. It can execute commands locally in headless mode or forward them across the local socket to a running GUI instance.

### Single Command Execution

```bash
# Headless inspection
firstproof-cli command '{"command":"inspect_metadata","path":"test.cr2"}'

# Forward to running FirstProofApp GUI
firstproof-cli command --connect '{"command":"select_photo","path":"/Volumes/Photos/Capture01.cr2"}'

# Adjust exposure in live app
firstproof-cli command --connect '{"command":"set_exposure","ev":0.75}'
```

### Piped Stream / Batch Automation

```bash
# Stream multiple commands via standard input
cat << 'EOF' | firstproof-cli command --connect
{"command": "select_photo", "path": "/Photos/Frame01.cr2"}
{"command": "set_tone_profile", "profile": "agx-vivid"}
{"command": "set_exposure", "ev": 0.35}
{"command": "capture_canvas", "output_path": "/tmp/frame01_render.png"}
EOF
```

---

## 3. Unix Domain Socket Protocol (`/tmp/firstproof-debug.sock`)

When `FirstProofApp` starts with IPC enabled (active by default in DEBUG builds or via `--enable-debug-ipc` / `FIRSTPROOF_DEBUG_IPC=1`), it opens a Unix domain stream socket at `/tmp/firstproof-debug.sock`.

### Wire Protocol
- **Transport**: Unix Domain Stream Socket (`AF_UNIX`, `SOCK_STREAM`).
- **Framing**: Line-delimited UTF-8 JSON (`\n`).
- **Concurrency**: Accepts non-blocking client connections, serializes main-thread state access via `MainActor.assumeIsolated`, and returns atomic JSON responses before closing the connection.
- **Envelope Compatibility**: Accepts both canonical tagged commands (`{"command": "...", ...}`) and legacy JSON-RPC payloads (`{"method": "...", "params": {...}}`).

---

## 4. Model Context Protocol (MCP) Bridge

The MCP bridge server (`tools/mcp/firstproof_mcp.py`) enables AI coding agents and automated studio assistants to control First Proof via standard MCP stdio JSON-RPC 2.0.

### Tool Registry

1. `firstproof_get_state`: Introspect current photo selection, edits, and canvas mode.
2. `firstproof_open_detail_view`: Navigate to photo and switch to detail inspection.
3. `firstproof_sample_viewport`: Probe exact RGB pixel values from the Metal viewport.
4. `firstproof_capture_canvas`: Capture high-resolution viewport screenshots.
5. `firstproof_apply_edit_delta`: Adjust exposure, contrast, highlights, shadows, white balance, etc.
6. `firstproof_set_emulsion`: Select curated physical film stocks.
7. `firstproof_set_print_media`: Configure darkroom paper archetypes and contrast grades.
8. `firstproof_reset_workspace`: Revert adjustments cleanly.
9. `firstproof_inspect_metadata`: Extract camera sensor and EXIF metadata.
10. `firstproof_develop_photo`: Run headless archival exports.
11. `firstproof_convert_dng`: Convert RAWs to archival DNGs with hash verification.
12. `firstproof_audit_parity`: Assert bit-level GPU vs CPU color parity.

### Claude Desktop / Cursor / Antigravity MCP Configuration

Add the following to your MCP client configuration (`mcp_config.json`):

```json
{
  "mcpServers": {
    "firstproof": {
      "command": "python3",
      "args": ["tools/mcp/firstproof_mcp.py"],
      "cwd": "/path/to/FirstProof"
    }
  }
}
```

When FirstProofApp is running on the macOS desktop, MCP tools manipulate the live viewport in real time. If the app is closed, tools automatically route through the headless `firstproof-cli` engine without failing.
