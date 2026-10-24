#!/usr/bin/env python3
"""
Automated Model Context Protocol (MCP) Server Test Suite

Asserts MCP protocol negotiation, tool discovery, and tool execution parity.
"Pure capture. Honest color. Zero digital plastic."
"""

import unittest
import subprocess
import json
import os
import sys

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
MCP_SERVER = os.path.join(SCRIPT_DIR, "firstproof_mcp.py")

class TestFirstProofMCP(unittest.TestCase):
    def setUp(self):
        env = os.environ.copy()
        env["FIRSTPROOF_SOCKET"] = "/tmp/firstproof-test-isolated.sock"
        self.proc = subprocess.Popen(
            [sys.executable, MCP_SERVER],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            env=env
        )

    def tearDown(self):
        if self.proc.poll() is None:
            self.proc.terminate()
            try:
                self.proc.wait(timeout=2.0)
            except Exception:
                pass
        if self.proc.stdin:
            self.proc.stdin.close()
        if self.proc.stdout:
            self.proc.stdout.close()
        if self.proc.stderr:
            self.proc.stderr.close()

    def _rpc_call(self, msg: dict) -> dict:
        self.proc.stdin.write(json.dumps(msg) + "\n")
        self.proc.stdin.flush()
        line = self.proc.stdout.readline()
        self.assertTrue(line, "No response from MCP server")
        return json.loads(line.strip())

    def test_initialize_handshake(self):
        req = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }
        resp = self._rpc_call(req)
        self.assertEqual(resp.get("jsonrpc"), "2.0")
        self.assertEqual(resp.get("id"), 1)
        res = resp.get("result", {})
        self.assertEqual(res.get("protocolVersion"), "2024-11-05")
        self.assertEqual(res.get("serverInfo", {}).get("name"), "firstproof-mcp")

    def test_tools_list_schema(self):
        req = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }
        resp = self._rpc_call(req)
        tools = resp.get("result", {}).get("tools", [])
        self.assertEqual(len(tools), 12, f"Expected 12 tools, got {len(tools)}")

        tool_names = {t["name"] for t in tools}
        expected_tools = {
            "firstproof_get_state",
            "firstproof_open_detail_view",
            "firstproof_sample_viewport",
            "firstproof_capture_canvas",
            "firstproof_apply_edit_delta",
            "firstproof_set_emulsion",
            "firstproof_set_print_media",
            "firstproof_reset_workspace",
            "firstproof_inspect_metadata",
            "firstproof_develop_photo",
            "firstproof_convert_dng",
            "firstproof_audit_parity",
        }
        self.assertEqual(tool_names, expected_tools)

        for t in tools:
            self.assertIn("name", t)
            self.assertIn("description", t)
            self.assertIn("inputSchema", t)
            self.assertEqual(t["inputSchema"].get("type"), "object")

    def test_tool_call_apply_edit_delta(self):
        req = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "firstproof_apply_edit_delta",
                "arguments": {
                    "exposure": 0.85,
                    "contrast": 25.0
                }
            }
        }
        resp = self._rpc_call(req)
        content = resp.get("result", {}).get("content", [])
        self.assertTrue(len(content) > 0)
        payload = json.loads(content[0]["text"])
        self.assertEqual(payload.get("status"), "ok")
        self.assertAlmostEqual(payload["settings"]["exposure"], 0.85, places=3)
        self.assertAlmostEqual(payload["settings"]["contrast"], 25.0, places=3)

    def test_tool_call_set_emulsion(self):
        req = {
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "firstproof_set_emulsion",
                "arguments": {
                    "archetype": "rochester-portrait-400",
                    "format": "120"
                }
            }
        }
        resp = self._rpc_call(req)
        content = resp.get("result", {}).get("content", [])
        self.assertTrue(len(content) > 0)
        payload = json.loads(content[0]["text"])
        self.assertEqual(payload.get("status"), "ok")
        self.assertEqual(payload["settings"]["emulsion_archetype"], "RochesterPortrait400")
        self.assertEqual(payload["settings"]["emulsion_format"], "Format120")

    def test_tool_call_unknown_tool(self):
        req = {
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "firstproof_nonexistent_tool",
                "arguments": {}
            }
        }
        resp = self._rpc_call(req)
        content = resp.get("result", {}).get("content", [])
        payload = json.loads(content[0]["text"])
        self.assertIn("error", payload)

if __name__ == "__main__":
    unittest.main()
