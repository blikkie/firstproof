import type {
  AdjustmentDescriptor,
  AllCurvesLutRecord,
  AutoAdjustments,
  AutoWhiteBalanceResult,
  CropAspectRatioPreset,
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
  ScaleMode,
  ToneCurvesRecord,
  WhiteBalancePreset,
  WorkspaceCategory,
  XmpData,
} from 'firstproof-wasm';
import initWasm, * as wasm from 'firstproof-wasm';
import type { EditSettingsPayload } from './types';

// Aliases for backward compatibility with existing web darkroom modules
export type {
  AdjustmentDescriptor,
  AllCurvesLutRecord,
  AutoAdjustments,
  AutoWhiteBalanceResult,
  CropAspectRatioPreset,
  CurvePointRecord,
  EditSettings,
  EditSettingsPayload,
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
  ScaleMode,
  ToneCurvesRecord,
  WhiteBalancePreset,
  WorkspaceCategory,
  XmpData,
};
export type XmpPayload = XmpData;
export type ImageMetadata = ImageMetadataRecord;
export type PreviewResult = PreviewResultRecord;
export type ToneCurvesPayload = ToneCurvesRecord;
export type CurvePoint = CurvePointRecord;
export type EvaluatedLut = AllCurvesLutRecord;

export interface LensCorrectionCoefficients {
  distortion: number[];
  tca: number[];
  vignetting: number[];
  distortion_model: string;
  has_distortion: boolean;
  has_tca: boolean;
  has_vignetting: boolean;
}

export interface EvHistogramRecord {
  bins_luminance: number[];
  bins_r: number[];
  bins_g: number[];
  bins_b: number[];
  shadow_clip_percent: number;
  sensor_clip_percent: number;
  display_rolloff_percent: number;
  ev_min: number;
  ev_max: number;
  num_bins: number;
}

export interface ArchivalDevelopResult {
  linear_offset: number;
  linear_len: number;
  width: number;
  height: number;
}

export interface ExportResult {
  output_offset: number;
  output_len: number;
  width: number;
  height: number;
  format: string;
}

export interface AutoAdjustmentsResult {
  exposure: number;
  contrast: number;
}

export interface AutoWhiteBalanceResultRecord {
  temperature: number;
  tint: number;
}

export class WasmBridge {
  private static instance: WasmBridge | null = null;
  private wasmMemory: WebAssembly.Memory | null = null;
  public memoryPtr: number = 0;
  public memoryLen: number = 0;

  private constructor() {}

  public static async init(capacityMb: number = 256): Promise<WasmBridge> {
    if (!WasmBridge.instance) {
      const exports = (await initWasm()) as unknown as {
        memory?: WebAssembly.Memory;
      };
      const bridge = new WasmBridge();
      bridge.wasmMemory = exports?.memory || null;
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

  private copyBytesToScratchpad(bytes: Uint8Array): void {
    const liveBuf = this.getWasmMemoryBuffer();
    const ptr = wasm.get_scratchpad_ptr();
    if (liveBuf && ptr + bytes.length <= liveBuf.byteLength) {
      new Uint8Array(liveBuf, ptr, bytes.length).set(bytes);
    } else {
      wasm.copy_into_scratchpad(0, bytes);
    }
  }

  public decodeImageBytes(bytes: Uint8Array): {
    preview: PreviewResult;
    imageSlice: Uint8Array;
  } {
    this.copyBytesToScratchpad(bytes);
    const preview: PreviewResult = wasm.decode_preview_from_scratchpad(
      bytes.length,
    ) as PreviewResult;
    const imageSlice = wasm.get_scratchpad_u8_slice(
      preview.preview_offset,
      preview.preview_len,
    );

    return { preview, imageSlice };
  }

  public decodeRawLinear(bytes: Uint8Array): {
    res: RawLinearResultRecord;
    linearData: Uint16Array;
  } {
    this.copyBytesToScratchpad(bytes);

    const res: RawLinearResultRecord = wasm.decode_raw_linear_from_scratchpad(
      bytes.length,
    ) as RawLinearResultRecord;
    const linearData = wasm.get_scratchpad_u16_slice(
      res.linear_offset,
      res.linear_len / 2,
    );

    return { res, linearData };
  }

  public decodeRawFullLinear(bytes: Uint8Array): {
    res: RawLinearResultRecord;
    linearData: Uint16Array;
  } {
    this.copyBytesToScratchpad(bytes);

    const res: RawLinearResultRecord =
      wasm.decode_raw_full_linear_from_scratchpad(
        bytes.length,
      ) as RawLinearResultRecord;
    const linearData = wasm.get_scratchpad_u16_slice(
      res.linear_offset,
      res.linear_len / 2,
    );

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
    this.copyBytesToScratchpad(bytes);

    const editsJson = JSON.stringify(edits);
    const qualityCode = quality === 'Archival' ? 1 : 0;
    const res = wasm.develop_raw_archival_from_scratchpad(
      bytes.length,
      editsJson,
      qualityCode,
    ) as ArchivalDevelopResult;

    const linearData = wasm.get_scratchpad_u16_slice(
      res.linear_offset,
      res.linear_len / 2,
    );

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
    this.copyBytesToScratchpad(bytes);

    const editsJson = JSON.stringify(edits);
    const qualityCode = demosaicQuality === 'Archival' ? 1 : 0;
    const res = wasm.export_raw_from_scratchpad(
      bytes.length,
      editsJson,
      format,
      quality,
      sizeOption,
      customDimension,
      qualityCode,
    ) as ExportResult;

    return wasm.get_scratchpad_u8_slice(res.output_offset, res.output_len);
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
    const res = wasm.compute_auto_adjustments_from_u16(
      linearData,
      width,
      height,
    ) as AutoAdjustmentsResult | null;
    return res ?? null;
  }

  public computeAutoAdjustmentsFromLuminances(
    lums: Float32Array,
  ): { exposure: number; contrast: number } | null {
    const res = wasm.compute_auto_adjustments_from_luminances(
      lums,
    ) as AutoAdjustmentsResult | null;
    return res ?? null;
  }

  public computeAutoWhiteBalance(
    linearData: Uint16Array,
    width: number,
    height: number,
  ): { temperature: number; tint: number } | null {
    const res = wasm.compute_auto_white_balance_from_u16(
      linearData,
      width,
      height,
    ) as AutoWhiteBalanceResultRecord | null;
    return res ?? null;
  }

  public computeEvHistogram(
    linearData: Uint16Array | Uint8Array,
    width: number,
    height: number,
    edits: EditSettingsPayload,
    lensCorrectionEnabled: boolean = false,
    tcaCorrectionEnabled: boolean = false,
    vignettingCorrectionEnabled: boolean = false,
    vignettingK1: number = 0.0,
    vignettingK2: number = 0.0,
    vignettingK3: number = 0.0,
  ): EvHistogramRecord | null {
    try {
      const byteView =
        linearData instanceof Uint8Array
          ? linearData
          : new Uint8Array(
              linearData.buffer,
              linearData.byteOffset,
              linearData.byteLength,
            );
      const res = wasm.compute_ev_histogram_from_bytes(
        byteView,
        width,
        height,
        edits.exposure,
        edits.contrast,
        edits.highlights,
        edits.shadows,
        edits.whites,
        edits.blacks,
        edits.crop_left,
        edits.crop_top,
        edits.crop_right,
        edits.crop_bottom,
        lensCorrectionEnabled,
        tcaCorrectionEnabled,
        vignettingCorrectionEnabled,
        vignettingK1,
        vignettingK2,
        vignettingK3,
      );
      return (res as EvHistogramRecord) ?? null;
    } catch (err) {
      console.error('[WasmBridge] computeEvHistogram failed:', err);
      return null;
    }
  }

  public calculateWhiteBalanceFromSample(
    r: number,
    g: number,
    b: number,
  ): { temperature: number; tint: number } | null {
    const res = wasm.calculate_white_balance_from_sample(
      r,
      g,
      b,
    ) as AutoWhiteBalanceResult | null;
    return res ?? null;
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

  public calculateTargetCropRatio(
    preset: string,
    imageAspect: number,
    isPortrait: boolean,
  ): number | null {
    const res = wasm.calculate_target_crop_ratio(
      preset,
      imageAspect,
      isPortrait,
    );
    return res !== undefined ? res : null;
  }

  public getWhiteBalancePresetValues(preset: string): [number, number] | null {
    const res = wasm.get_white_balance_preset_values(preset);
    return res ? [res[0], res[1]] : null;
  }

  public detectWhiteBalancePreset(
    temperature: number,
    tint: number,
    asShotTemp: number,
    asShotTint: number,
  ): string {
    return wasm.detect_white_balance_preset(
      temperature,
      tint,
      asShotTemp,
      asShotTint,
    );
  }

  public kelvinToMiredPos(kelvin: number): number {
    return wasm.convert_kelvin_to_mired_norm(kelvin);
  }

  public miredPosToKelvin(pos: number): number {
    return wasm.convert_mired_norm_to_kelvin(pos);
  }

  public getAllAdjustments(): AdjustmentDescriptor[] {
    return (wasm.get_all_adjustments() as AdjustmentDescriptor[]) || [];
  }

  public getAdjustmentDescriptor(id: string): AdjustmentDescriptor | null {
    return (wasm.get_adjustment_descriptor(id) as AdjustmentDescriptor) || null;
  }

  public loadLensfunDatabase(jsonStr: string): boolean {
    return wasm.load_lensfun_database(jsonStr);
  }

  public resolveLensCoefficients(
    cameraMake: string,
    cameraModel: string,
    lensModel: string,
    focalLength: number,
    aperture?: number,
  ): LensCorrectionCoefficients {
    return wasm.resolve_lens_coefficients(
      cameraMake,
      cameraModel,
      lensModel,
      focalLength,
      aperture,
    ) as LensCorrectionCoefficients;
  }
}
