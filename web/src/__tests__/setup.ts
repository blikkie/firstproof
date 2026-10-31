import fs from 'node:fs';
import path from 'node:path';
import { initSync } from 'firstproof-wasm';

try {
  const wasmPath = path.resolve(
    __dirname,
    '../../../firstproof-wasm/pkg/firstproof_wasm_bg.wasm',
  );
  if (fs.existsSync(wasmPath)) {
    const wasmBytes = fs.readFileSync(wasmPath);
    initSync({ module: wasmBytes });
  }
} catch (e) {
  console.warn('[Vitest Setup] Failed to init wasm:', e);
}
