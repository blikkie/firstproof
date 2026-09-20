/**
 * WebWorker for off-main-thread WASM RAW decoding & image parsing.
 * Keeps the main UI thread at a locked 60 FPS during heavy culling bursts.
 * Includes performance.mark / performance.measure telemetry metrics.
 */

import initWasm, {
  init_scratchpad,
  get_scratchpad_ptr,
  copy_into_scratchpad,
  decode_preview_from_scratchpad,
  decode_raw_linear_from_scratchpad,
  get_scratchpad_u8_slice,
  get_scratchpad_u16_slice,
} from 'firstproof-wasm';

let wasmInitialized = false;
let workerWasmMemory: WebAssembly.Memory | null = null;

interface WorkerDecodeMessage {
  id: number;
  arrayBuffer: ArrayBuffer;
  fileName: string;
}

interface WorkerResponseMessage {
  id: number;
  success: boolean;
  preview?: any;
  imageSlice?: Uint8Array;
  linearData?: Uint16Array;
  linearW?: number;
  linearH?: number;
  error?: string;
  metrics?: {
    decodeMs: number;
    inputBytes: number;
  };
}

self.onmessage = async (e: MessageEvent<WorkerDecodeMessage>) => {
  const { id, arrayBuffer, fileName } = e.data;

  const markStart = `firstproof:decode:start:${id}`;
  const markEnd = `firstproof:decode:end:${id}`;
  const measureName = `firstproof:decode:${fileName}:${id}`;

  performance.mark(markStart);
  const startTime = performance.now();

  try {
    if (!wasmInitialized) {
      const exports = await initWasm();
      workerWasmMemory = (exports as any)?.memory || (initWasm as any)?.memory || null;
      init_scratchpad(256); // 256MB scratchpad buffer for 50MP+ RAW files
      wasmInitialized = true;
    }

    const inputBytes = new Uint8Array(arrayBuffer);
    copy_into_scratchpad(0, inputBytes);

    const preview = decode_preview_from_scratchpad(inputBytes.length);
    let imageSlice: Uint8Array;
    if (typeof get_scratchpad_u8_slice === 'function') {
      imageSlice = get_scratchpad_u8_slice(preview.preview_offset, preview.preview_len);
    } else {
      const ptr = get_scratchpad_ptr();
      const wasmMem = workerWasmMemory?.buffer;
      if (wasmMem && ptr + preview.preview_offset + preview.preview_len <= wasmMem.byteLength) {
        imageSlice = new Uint8Array(wasmMem, ptr + preview.preview_offset, preview.preview_len).slice();
      } else {
        imageSlice = inputBytes.slice(preview.preview_offset, preview.preview_offset + preview.preview_len);
      }
    }

    // Off-main-thread 16-bit Linear Rec.2020 sensor frame decode for RAW files
    const ext = fileName.split('.').pop()?.toLowerCase() || '';
    const isRawFile = ['cr2', 'cr3', 'nef', 'arw', 'dng', 'rw2', 'orf', 'pef', 'raf'].includes(ext);

    let linearData: Uint16Array | undefined;
    let linearW = 0;
    let linearH = 0;

    if (isRawFile) {
      try {
        const rawLin = decode_raw_linear_from_scratchpad(inputBytes.length);
        if (rawLin && rawLin.linear_len > 0) {
          linearW = rawLin.width;
          linearH = rawLin.height;
          if (typeof get_scratchpad_u16_slice === 'function') {
            linearData = get_scratchpad_u16_slice(rawLin.linear_offset, rawLin.linear_len / 2);
          }
        }
      } catch (rawErr) {
        console.warn(`[RawDecodeWorker] Off-thread RAW linear decode fallback for ${fileName}:`, rawErr);
      }
    }

    performance.mark(markEnd);
    performance.measure(measureName, markStart, markEnd);

    const decodeMs = performance.now() - startTime;
    console.debug(`⚡ [RawDecodeWorker] ${fileName} decoded in ${decodeMs.toFixed(2)}ms (${(inputBytes.length / 1024 / 1024).toFixed(1)} MB)`);

    const response: WorkerResponseMessage = {
      id,
      success: true,
      preview,
      imageSlice,
      linearData,
      linearW,
      linearH,
      metrics: {
        decodeMs,
        inputBytes: inputBytes.length
      }
    };

    // Transfer buffers back to main thread zero-copy
    const transferables: Transferable[] = [imageSlice.buffer];
    if (linearData && linearData.buffer) {
      transferables.push(linearData.buffer);
    }
    (self as any).postMessage(response, transferables);

  } catch (err: any) {
    performance.mark(markEnd);
    const decodeMs = performance.now() - startTime;

    const response: WorkerResponseMessage = {
      id,
      success: false,
      error: err?.toString() || 'Worker RAW decoding failed',
      metrics: {
        decodeMs,
        inputBytes: arrayBuffer.byteLength
      }
    };

    (self as any).postMessage(response);
  }
};
