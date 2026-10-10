import type {
  AllCurvesLutRecord,
  AutoAdjustments,
  AutoWhiteBalanceResult,
  ColorProfileMode,
  CurvePointRecord,
  EditSettings,
  EmulsionArchetype,
  EmulsionFormat,
  EmulsionGrainMode,
  EnlargerFilterMode,
  ExifOrientationRecord,
  ExportColorSpace,
  ExportRenderingIntent,
  ImageMetadataRecord,
  PreviewResultRecord,
  PrintMediaMode,
  RawLinearResultRecord,
  RemjetBackingMode,
  ToneCurvesRecord,
  XmpData,
} from 'firstproof-wasm';
import initWasm, * as wasm from 'firstproof-wasm';

export type {
  AllCurvesLutRecord,
  AutoAdjustments,
  AutoWhiteBalanceResult,
  ColorProfileMode,
  CurvePointRecord,
  EditSettings,
  EmulsionArchetype,
  EmulsionFormat,
  EmulsionGrainMode,
  EnlargerFilterMode,
  ExifOrientationRecord,
  ExportColorSpace,
  ExportRenderingIntent,
  ImageMetadataRecord,
  PreviewResultRecord,
  PrintMediaMode,
  RawLinearResultRecord,
  RemjetBackingMode,
  ToneCurvesRecord,
  XmpData,
};

// Aliases for backward compatibility with existing web darkroom modules
export type EditSettingsPayload = EditSettings;
export type XmpPayload = XmpData;
export type ImageMetadata = ImageMetadataRecord;
export type PreviewResult = PreviewResultRecord;
export type ToneCurvesPayload = ToneCurvesRecord;
export type CurvePoint = CurvePointRecord;
export type EvaluatedLut = AllCurvesLutRecord;

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
      console.log(
        `[WASM] Scratchpad initialized: ${(bridge.memoryLen / (1024 * 1024)).toFixed(1)} MB at 0x${bridge.memoryPtr.toString(16)}`,
      );
      WasmBridge.instance = bridge;
    }
    return WasmBridge.instance;
  }

  public static getInstance(): WasmBridge {
    if (!WasmBridge.instance) {
      throw new Error(
        'WasmBridge not initialized. Call WasmBridge.init() first.',
      );
    }
    return WasmBridge.instance;
  }

  private getWasmMemoryBuffer(): ArrayBuffer | null {
    return this.wasmMemory?.buffer ?? null;
  }

  public decodeImageBytes(bytes: Uint8Array): {
    preview: PreviewResult;
    imageSlice: Uint8Array;
  } {
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
      imageSlice = (wasm as any).get_scratchpad_u8_slice(
        preview.preview_offset,
        preview.preview_len,
      );
    } else {
      const freshBuf = this.getWasmMemoryBuffer();
      if (
        freshBuf &&
        ptr + preview.preview_offset + preview.preview_len <=
          freshBuf.byteLength
      ) {
        imageSlice = new Uint8Array(
          freshBuf,
          ptr + preview.preview_offset,
          preview.preview_len,
        ).slice();
      } else {
        imageSlice = bytes.slice(
          preview.preview_offset,
          preview.preview_offset + preview.preview_len,
        );
      }
    }

    return { preview, imageSlice };
  }

  public decodeRawLinear(bytes: Uint8Array): {
    res: any;
    linearData: Uint16Array;
  } {
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
      linearData = (wasm as any).get_scratchpad_u16_slice(
        res.linear_offset,
        res.linear_len / 2,
      );
    } else {
      const freshBuf = this.getWasmMemoryBuffer();
      const absOffset = ptr + res.linear_offset;
      if (freshBuf && absOffset + res.linear_len <= freshBuf.byteLength) {
        linearData = new Uint16Array(
          freshBuf,
          absOffset,
          res.linear_len / 2,
        ).slice();
      } else {
        throw new Error(`WASM memory buffer bounds error`);
      }
    }

    return { res, linearData };
  }

  public decodeRawFullLinear(bytes: Uint8Array): {
    res: any;
    linearData: Uint16Array;
  } {
    const liveBuf = this.getWasmMemoryBuffer();
    const ptr = wasm.get_scratchpad_ptr();
    if (liveBuf && ptr + bytes.length <= liveBuf.byteLength) {
      new Uint8Array(liveBuf, ptr, bytes.length).set(bytes);
    } else {
      wasm.copy_into_scratchpad(0, bytes);
    }

    const resVal = (wasm as any).decode_raw_full_linear_from_scratchpad(
      bytes.length,
    );
    const res = resVal as any;

    let linearData: Uint16Array;
    if (typeof (wasm as any).get_scratchpad_u16_slice === 'function') {
      linearData = (wasm as any).get_scratchpad_u16_slice(
        res.linear_offset,
        res.linear_len / 2,
      );
    } else {
      const freshBuf = this.getWasmMemoryBuffer();
      const absOffset = ptr + res.linear_offset;
      if (freshBuf && absOffset + res.linear_len <= freshBuf.byteLength) {
        linearData = new Uint16Array(
          freshBuf,
          absOffset,
          res.linear_len / 2,
        ).slice();
      } else {
        throw new Error(`WASM memory buffer bounds error`);
      }
    }

    return { res, linearData };
  }

  public developRawArchival(
    bytes: Uint8Array,
    edits: EditSettingsPayload,
    quality: 'Standard' | 'Archival' = 'Standard',
  ): {
    width: number;
    height: number;
    linearData: Uint16Array;
  } {
    const liveBuf = this.getWasmMemoryBuffer();
    const ptr = wasm.get_scratchpad_ptr();
    if (liveBuf && ptr + bytes.length <= liveBuf.byteLength) {
      new Uint8Array(liveBuf, ptr, bytes.length).set(bytes);
    } else {
      wasm.copy_into_scratchpad(0, bytes);
    }

    const editsJson = JSON.stringify(edits);
    const qualityCode = quality === 'Archival' ? 1 : 0;
    const resVal = (wasm as any).develop_raw_archival_from_scratchpad(
      bytes.length,
      editsJson,
      qualityCode,
    );
    const res = resVal as {
      linear_offset: number;
      linear_len: number;
      width: number;
      height: number;
    };

    let linearData: Uint16Array;
    if (typeof (wasm as any).get_scratchpad_u16_slice === 'function') {
      linearData = (wasm as any).get_scratchpad_u16_slice(
        res.linear_offset,
        res.linear_len / 2,
      );
    } else {
      const freshBuf = this.getWasmMemoryBuffer();
      const absOffset = ptr + res.linear_offset;
      if (freshBuf && absOffset + res.linear_len <= freshBuf.byteLength) {
        linearData = new Uint16Array(
          freshBuf,
          absOffset,
          res.linear_len / 2,
        ).slice();
      } else {
        throw new Error(`WASM memory buffer bounds error`);
      }
    }

    return { width: res.width, height: res.height, linearData };
  }

  public exportRaw(
    bytes: Uint8Array,
    edits: EditSettingsPayload,
    format: 'JPEG' | 'TIFF',
    quality: number,
    sizeOption: 'Original' | '2048' | '3840' | 'Custom',
    customDimension: number,
    demosaicQuality: 'Standard' | 'Archival' = 'Standard',
  ): Uint8Array {
    const liveBuf = this.getWasmMemoryBuffer();
    const ptr = wasm.get_scratchpad_ptr();
    if (liveBuf && ptr + bytes.length <= liveBuf.byteLength) {
      new Uint8Array(liveBuf, ptr, bytes.length).set(bytes);
    } else {
      wasm.copy_into_scratchpad(0, bytes);
    }

    const editsJson = JSON.stringify(edits);
    const qualityCode = demosaicQuality === 'Archival' ? 1 : 0;
    const resVal = (wasm as any).export_raw_from_scratchpad(
      bytes.length,
      editsJson,
      format,
      quality,
      sizeOption,
      customDimension,
      qualityCode,
    );
    const res = resVal as {
      output_offset: number;
      output_len: number;
      width: number;
      height: number;
      format: string;
    };

    let outputBytes: Uint8Array;
    if (typeof (wasm as any).get_scratchpad_u8_slice === 'function') {
      outputBytes = (wasm as any).get_scratchpad_u8_slice(
        res.output_offset,
        res.output_len,
      );
    } else {
      const freshBuf = this.getWasmMemoryBuffer();
      const absOffset = ptr + res.output_offset;
      if (freshBuf && absOffset + res.output_len <= freshBuf.byteLength) {
        outputBytes = new Uint8Array(
          freshBuf,
          absOffset,
          res.output_len,
        ).slice();
      } else {
        throw new Error(`WASM scratchpad buffer bounds error`);
      }
    }

    return outputBytes;
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

  public computeAutoAdjustments(
    linearData: Uint16Array,
    width: number,
    height: number,
  ): { exposure: number; contrast: number } | null {
    if (typeof (wasm as any).compute_auto_adjustments_from_u16 === 'function') {
      const res = (wasm as any).compute_auto_adjustments_from_u16(
        linearData,
        width,
        height,
      );
      return res as { exposure: number; contrast: number };
    }
    return null;
  }

  public computeAutoAdjustmentsFromLuminances(
    lums: Float32Array,
  ): { exposure: number; contrast: number } | null {
    if (
      typeof (wasm as any).compute_auto_adjustments_from_luminances ===
      'function'
    ) {
      const res = (wasm as any).compute_auto_adjustments_from_luminances(lums);
      return res as { exposure: number; contrast: number };
    }
    return null;
  }

  public computeAutoWhiteBalance(
    linearData: Uint16Array,
    width: number,
    height: number,
  ): { temperature: number; tint: number } | null {
    if (
      typeof (wasm as any).compute_auto_white_balance_from_u16 === 'function'
    ) {
      const res = (wasm as any).compute_auto_white_balance_from_u16(
        linearData,
        width,
        height,
      );
      return res as { temperature: number; tint: number };
    }
    return null;
  }

  public calculateWhiteBalanceFromSample(
    r: number,
    g: number,
    b: number,
  ): { temperature: number; tint: number } | null {
    if (
      typeof (wasm as any).calculate_white_balance_from_sample === 'function'
    ) {
      const res = (wasm as any).calculate_white_balance_from_sample(r, g, b);
      return res as { temperature: number; tint: number };
    }
    return null;
  }

  public decodeExifOrientation(orientation: number): ExifOrientationRecord {
    return wasm.decode_exif_orientation(orientation) as ExifOrientationRecord;
  }

  public extractMetadata(bytes: Uint8Array): ImageMetadataRecord {
    return wasm.extract_metadata_from_bytes(bytes) as ImageMetadataRecord;
  }

  public evaluateReferenceFrame(
    inputData: Uint8Array,
    isLinearRec2020: boolean,
    width: number,
    height: number,
    edits: EditSettingsPayload,
    curvesLut?: EvaluatedLut | null,
  ): Uint8Array {
    const editsJson = JSON.stringify(edits);
    const curvesJson = curvesLut ? JSON.stringify(curvesLut) : null;
    return wasm.evaluate_reference_frame(
      inputData,
      isLinearRec2020,
      width,
      height,
      editsJson,
      curvesJson,
    );
  }

  public evaluateReferencePixel(
    r: number,
    g: number,
    b: number,
    isLinearRec2020: boolean,
    edits: EditSettingsPayload,
    curvesLut?: EvaluatedLut | null,
  ): [number, number, number] {
    const editsJson = JSON.stringify(edits);
    const curvesJson = curvesLut ? JSON.stringify(curvesLut) : null;
    const res = wasm.evaluate_reference_pixel(
      r,
      g,
      b,
      isLinearRec2020,
      editsJson,
      curvesJson,
    );
    return [res[0], res[1], res[2]];
  }

  public evaluateDisplayLuma(
    r: number,
    g: number,
    b: number,
    isLinearRec2020: boolean,
    edits: EditSettingsPayload,
  ): number {
    const editsJson = JSON.stringify(edits);
    return wasm.evaluate_display_luma(r, g, b, isLinearRec2020, editsJson);
  }
}
