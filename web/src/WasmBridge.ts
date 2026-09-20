import initWasm, * as wasm from 'firstproof-wasm';

export interface ImageMetadata {
  camera_make?: string;
  camera_model?: string;
  lens_model?: string;
  iso?: number;
  aperture?: number;
  shutter_speed?: string;
  focal_length?: number;
  date_time?: string;
  taken_time?: number;
  orientation?: number;
  raw_width?: number;
  raw_height?: number;
}

export interface PreviewResult {
  preview_offset: number;
  preview_len: number;
  width: number;
  height: number;
  is_jpeg: boolean;
  metadata: ImageMetadata;
}

export interface CurvePoint {
  x: number;
  y: number;
}

export interface ToneCurvesPayload {
  master: CurvePoint[];
  red: CurvePoint[];
  green: CurvePoint[];
  blue: CurvePoint[];
  [key: string]: CurvePoint[];
}

export interface EvaluatedLut {
  master: number[];
  red: number[];
  green: number[];
  blue: number[];
}

export interface EditSettingsPayload {
  exposure: number;
  contrast: number;
  highlights: number;
  shadows: number;
  whites: number;
  blacks: number;
  vibrance: number;
  saturation: number;
  deep_blue: number;
  foliage: number;
  temperature: number;
  tint: number;
  crop_left: number;
  crop_top: number;
  crop_right: number;
  crop_bottom: number;
  crop_angle: number;
  perspective_vertical: number;
  perspective_horizontal: number;
  crop_constrain: boolean;
  rotation: number;
  flip_horizontal: boolean;
  flip_vertical: boolean;
  tone_profile: string;
  lut_name?: string;
  lut_intensity: number;
  sharpening_amount: number;
  sharpening_radius: number;
  sharpening_masking: number;
  noise_reduction: number;
  emulsion_archetype: string;
  emulsion_format: string;
  emulsion_exposure: number;
  curves: ToneCurvesPayload;
  split_toning_shadow_hue: number;
  split_toning_shadow_sat: number;
  split_toning_highlight_hue: number;
  split_toning_highlight_sat: number;
  split_toning_balance: number;
  print_media: string;
  enlarger_filter: string;
  print_paper_grade: number;
  print_base_warmth: number;
  print_process_depth: number;
}

export interface XmpPayload {
  rating?: number;
  status?: string;
  title?: string;
  description?: string;
  creator?: string;
  copyright?: string;
  keywords?: string[];
  edits: EditSettingsPayload;
}

export class WasmBridge {
  private static instance: WasmBridge | null = null;
  private wasmMemory: WebAssembly.Memory | null = null;
  public memoryPtr: number = 0;
  public memoryLen: number = 0;

  private constructor() {}

  public static async init(capacityMb: number = 256): Promise<WasmBridge> {
    if (!WasmBridge.instance) {
      const exports = await initWasm();
      const bridge = new WasmBridge();
      bridge.wasmMemory = (exports as any)?.memory || null;
      bridge.memoryPtr = wasm.init_scratchpad(capacityMb);
      bridge.memoryLen = wasm.get_scratchpad_len();
      console.log(`[WASM] Scratchpad initialized: ${(bridge.memoryLen / (1024 * 1024)).toFixed(1)} MB at 0x${bridge.memoryPtr.toString(16)}`);
      WasmBridge.instance = bridge;
    }
    return WasmBridge.instance;
  }

  public static getInstance(): WasmBridge {
    if (!WasmBridge.instance) {
      throw new Error("WasmBridge not initialized. Call WasmBridge.init() first.");
    }
    return WasmBridge.instance;
  }

  private getWasmMemoryBuffer(): ArrayBuffer | null {
    if (this.wasmMemory && this.wasmMemory.buffer) {
      return this.wasmMemory.buffer;
    }
    return null;
  }

  public decodeImageBytes(bytes: Uint8Array): { preview: PreviewResult; imageSlice: Uint8Array } {
    const liveBuf = this.getWasmMemoryBuffer();
    const ptr = wasm.get_scratchpad_ptr();
    if (liveBuf && ptr + bytes.length <= liveBuf.byteLength) {
      new Uint8Array(liveBuf, ptr, bytes.length).set(bytes);
    } else {
      wasm.copy_into_scratchpad(0, bytes);
    }

    const resVal = wasm.decode_preview_from_scratchpad(bytes.length);
    const preview: PreviewResult = resVal as PreviewResult;

    // Get slice directly from scratchpad memory using typed getter or fallback
    let imageSlice: Uint8Array;
    if (typeof (wasm as any).get_scratchpad_u8_slice === 'function') {
      imageSlice = (wasm as any).get_scratchpad_u8_slice(preview.preview_offset, preview.preview_len);
    } else {
      const freshBuf = this.getWasmMemoryBuffer();
      if (freshBuf && ptr + preview.preview_offset + preview.preview_len <= freshBuf.byteLength) {
        imageSlice = new Uint8Array(freshBuf, ptr + preview.preview_offset, preview.preview_len).slice();
      } else {
        imageSlice = bytes.slice(preview.preview_offset, preview.preview_offset + preview.preview_len);
      }
    }

    return { preview, imageSlice };
  }

  public decodeRawLinear(bytes: Uint8Array): { res: any; linearData: Uint16Array } {
    const liveBuf = this.getWasmMemoryBuffer();
    const ptr = wasm.get_scratchpad_ptr();
    if (liveBuf && ptr + bytes.length <= liveBuf.byteLength) {
      new Uint8Array(liveBuf, ptr, bytes.length).set(bytes);
    } else {
      wasm.copy_into_scratchpad(0, bytes);
    }

    const resVal = wasm.decode_raw_linear_from_scratchpad(bytes.length);
    const res = resVal as any;

    let linearData: Uint16Array;
    if (typeof (wasm as any).get_scratchpad_u16_slice === 'function') {
      linearData = (wasm as any).get_scratchpad_u16_slice(res.linear_offset, res.linear_len / 2);
    } else {
      const freshBuf = this.getWasmMemoryBuffer();
      const absOffset = ptr + res.linear_offset;
      if (freshBuf && absOffset + res.linear_len <= freshBuf.byteLength) {
        linearData = new Uint16Array(freshBuf, absOffset, res.linear_len / 2).slice();
      } else {
        throw new Error(`WASM memory buffer bounds error`);
      }
    }

    return { res, linearData };
  }

  public decodeRawFullLinear(bytes: Uint8Array): { res: any; linearData: Uint16Array } {
    const liveBuf = this.getWasmMemoryBuffer();
    const ptr = wasm.get_scratchpad_ptr();
    if (liveBuf && ptr + bytes.length <= liveBuf.byteLength) {
      new Uint8Array(liveBuf, ptr, bytes.length).set(bytes);
    } else {
      wasm.copy_into_scratchpad(0, bytes);
    }

    const resVal = (wasm as any).decode_raw_full_linear_from_scratchpad(bytes.length);
    const res = resVal as any;

    let linearData: Uint16Array;
    if (typeof (wasm as any).get_scratchpad_u16_slice === 'function') {
      linearData = (wasm as any).get_scratchpad_u16_slice(res.linear_offset, res.linear_len / 2);
    } else {
      const freshBuf = this.getWasmMemoryBuffer();
      const absOffset = ptr + res.linear_offset;
      if (freshBuf && absOffset + res.linear_len <= freshBuf.byteLength) {
        linearData = new Uint16Array(freshBuf, absOffset, res.linear_len / 2).slice();
      } else {
        throw new Error(`WASM memory buffer bounds error`);
      }
    }

    return { res, linearData };
  }

  public evaluateCurves(curves: ToneCurvesPayload): EvaluatedLut {
    const jsonStr = JSON.stringify(curves);
    const res = wasm.evaluate_curves(jsonStr);
    return res as EvaluatedLut;
  }

  public parseXmp(xml: string): XmpPayload {
    const res = wasm.parse_xmp(xml);
    return res as XmpPayload;
  }

  public generateXmp(xmp: XmpPayload): string {
    const jsonStr = JSON.stringify(xmp);
    return wasm.generate_xmp(jsonStr);
  }

  public computeAutoAdjustments(linearData: Uint16Array, width: number, height: number): { exposure: number; contrast: number } | null {
    if (typeof (wasm as any).compute_auto_adjustments_from_u16 === 'function') {
      const res = (wasm as any).compute_auto_adjustments_from_u16(linearData, width, height);
      return res as { exposure: number; contrast: number };
    }
    return null;
  }

  public computeAutoAdjustmentsFromLuminances(lums: Float32Array): { exposure: number; contrast: number } | null {
    if (typeof (wasm as any).compute_auto_adjustments_from_luminances === 'function') {
      const res = (wasm as any).compute_auto_adjustments_from_luminances(lums);
      return res as { exposure: number; contrast: number };
    }
    return null;
  }

  public computeAutoWhiteBalance(linearData: Uint16Array, width: number, height: number): { temperature: number; tint: number } | null {
    if (typeof (wasm as any).compute_auto_white_balance_from_u16 === 'function') {
      const res = (wasm as any).compute_auto_white_balance_from_u16(linearData, width, height);
      return res as { temperature: number; tint: number };
    }
    return null;
  }

  public calculateWhiteBalanceFromSample(r: number, g: number, b: number): { temperature: number; tint: number } | null {
    if (typeof (wasm as any).calculate_white_balance_from_sample === 'function') {
      const res = (wasm as any).calculate_white_balance_from_sample(r, g, b);
      return res as { temperature: number; tint: number };
    }
    return null;
  }
}

