/**
 * WebWorker for off-main-thread WASM RAW decoding & image parsing.
 * Keeps the main UI thread at a locked 60 FPS during heavy culling bursts.
 * Includes performance.mark / performance.measure telemetry metrics.
 */

import initWasm, {
  copy_into_scratchpad,
  decode_preview_from_scratchpad,
  decode_raw_full_linear_from_scratchpad,
  decode_raw_linear_from_scratchpad,
  export_raw_from_scratchpad,
  get_scratchpad_u8_slice,
  get_scratchpad_u16_slice,
  init_scratchpad,
  type PreviewResultRecord,
} from 'firstproof-wasm';

interface ExportResultRecord {
  output_offset: number;
  output_len: number;
  width: number;
  height: number;
  format: string;
}

type ExportRawFn = (
  inputLen: number,
  editsJson: string,
  format: string,
  quality: number,
  sizeOption: string,
  customDim: number,
  demosaicQuality: number,
) => ExportResultRecord;

let wasmInitialized = false;

interface WorkerDecodeMessage {
  type?: 'decode' | 'export';
  id: number;
  arrayBuffer: ArrayBuffer;
  fileName: string;
  fullResolution?: boolean;
  editsJson?: string;
  format?: 'JPEG' | 'TIFF';
  quality?: number;
  sizeOption?: string;
  customDimension?: number;
  demosaicQuality?: number;
}

export interface WorkerResponseMessage {
  id: number;
  success: boolean;
  isExport?: boolean;
  outputBytes?: Uint8Array;
  width?: number;
  height?: number;
  format?: string;
  preview?: PreviewResultRecord;
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

interface WorkerScope {
  postMessage(message: WorkerResponseMessage, transfer?: Transferable[]): void;
}

const ctx = self as unknown as WorkerScope;

self.onmessage = async (e: MessageEvent<WorkerDecodeMessage>) => {
  const data = e.data;
  const { id, arrayBuffer, fileName, fullResolution } = data;

  const markStart = `firstproof:decode:start:${id}`;
  const markEnd = `firstproof:decode:end:${id}`;
  const measureName = `firstproof:decode:${fileName}:${id}`;

  performance.mark(markStart);
  const startTime = performance.now();

  try {
    if (!wasmInitialized) {
      await initWasm();
      init_scratchpad(256); // 256MB scratchpad buffer for 50MP+ RAW files
      wasmInitialized = true;
    }

    if (data.type === 'export') {
      const inputBytes = new Uint8Array(arrayBuffer);
      copy_into_scratchpad(0, inputBytes);

      const exportFn = export_raw_from_scratchpad as unknown as ExportRawFn;
      const res = exportFn(
        inputBytes.length,
        data.editsJson || '{}',
        data.format || 'JPEG',
        data.quality ?? 90,
        data.sizeOption || 'Original',
        data.customDimension ?? 2048,
        data.demosaicQuality ?? 0,
      );

      const outputBytes = get_scratchpad_u8_slice(
        res.output_offset,
        res.output_len,
      );

      const response: WorkerResponseMessage = {
        id,
        success: true,
        isExport: true,
        outputBytes,
        width: res.width,
        height: res.height,
        format: res.format,
      };

      ctx.postMessage(response, [outputBytes.buffer]);
      return;
    }

    const inputBytes = new Uint8Array(arrayBuffer);
    copy_into_scratchpad(0, inputBytes);

    const preview = decode_preview_from_scratchpad(inputBytes.length);
    const imageSlice = get_scratchpad_u8_slice(
      preview.preview_offset,
      preview.preview_len,
    );

    // Off-main-thread 16-bit Linear Rec.2020 sensor frame decode for RAW files
    const ext = fileName.split('.').pop()?.toLowerCase() || '';
    const isRawFile = [
      'cr2',
      'cr3',
      'nef',
      'arw',
      'dng',
      'rw2',
      'orf',
      'pef',
      'raf',
      'raw',
    ].includes(ext);

    let linearData: Uint16Array | undefined;
    let linearW = 0;
    let linearH = 0;

    if (isRawFile) {
      try {
        const rawLin = fullResolution
          ? decode_raw_full_linear_from_scratchpad(inputBytes.length)
          : decode_raw_linear_from_scratchpad(inputBytes.length);
        if (rawLin && rawLin.linear_len > 0) {
          linearW = rawLin.width;
          linearH = rawLin.height;
          linearData = get_scratchpad_u16_slice(
            rawLin.linear_offset,
            rawLin.linear_len / 2,
          );
        }
      } catch (rawErr) {
        console.warn(
          `[RawDecodeWorker] Off-thread RAW linear decode fallback for ${fileName}:`,
          rawErr,
        );
      }
    }

    performance.mark(markEnd);
    performance.measure(measureName, markStart, markEnd);

    const decodeMs = performance.now() - startTime;
    console.debug(
      `⚡ [RawDecodeWorker] ${fileName} decoded in ${decodeMs.toFixed(2)}ms (${(inputBytes.length / 1024 / 1024).toFixed(1)} MB)`,
    );

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
        inputBytes: inputBytes.length,
      },
    };

    // Transfer buffers back to main thread zero-copy
    const transferables: Transferable[] = [imageSlice.buffer];
    if (linearData?.buffer) {
      transferables.push(linearData.buffer);
    }
    ctx.postMessage(response, transferables);
  } catch (err: unknown) {
    performance.mark(markEnd);
    const decodeMs = performance.now() - startTime;
    const errMsg =
      err instanceof Error
        ? err.message
        : typeof err === 'string'
          ? err
          : 'Worker RAW decoding failed';

    const response: WorkerResponseMessage = {
      id,
      success: false,
      error: errMsg,
      metrics: {
        decodeMs,
        inputBytes: arrayBuffer.byteLength,
      },
    };

    ctx.postMessage(response);
  }
};
