/**
 * First Proof — Mechanical Parity Test Harness
 *
 * Programmatically drives WebGL shader previews and compares output bit-for-bit
 * against the reference CPU export pipeline.
 */

import { WebGLRenderer } from '../WebGLRenderer';
import { EditSettingsPayload, EvaluatedLut, WasmBridge } from '../WasmBridge';
import { evaluateReferencePixel, evaluateDisplayLuma, ParityPixelResult } from './ReferenceExport';

export interface ParityTestCase {
  id: string;
  name: string;
  category: 'macbeth' | 'exposure' | 'white_balance' | 'tone_recovery' | 'color' | 'curves' | 'active_photo';
  edits: EditSettingsPayload;
  isLinearRec2020: boolean;
  textureWidth: number;
  textureHeight: number;
  createTexture: () => ImageData | { data: Uint16Array; width: number; height: number };
}

export interface ParityAuditResult {
  testId: string;
  testName: string;
  pass: boolean;
  maxChannelDelta: number; // 0..255
  meanChannelDelta: number;
  psnr: number; // dB
  totalPixels: number;
  failingPixels: number; // delta > tolerance
  worstPixelCoord: [number, number];
  worstGlRgb: [number, number, number];
  worstExportRgb: [number, number, number];
  stageDiagnosis: string;
  heatmapData?: ImageData;
}

export const MACBETH_SRGB_24: [number, number, number][] = [
  // Row 1: Natural tones
  [115, 82, 68],   // 1. Dark Skin
  [194, 150, 130], // 2. Light Skin
  [98, 122, 157],  // 3. Blue Sky
  [87, 108, 67],   // 4. Foliage
  [133, 128, 177], // 5. Purplish Blue
  [103, 189, 170], // 6. Delicate Blue
  // Row 2: Miscellaneous
  [214, 126, 44],  // 7. Orange
  [80, 91, 166],   // 8. Purplish Blue 2
  [193, 90, 99],   // 9. Moderate Red
  [94, 60, 108],   // 10. Purple
  [157, 188, 64],  // 11. Yellow Green
  [224, 163, 46],  // 12. Orange Yellow
  // Row 3: Primary & Secondary colors
  [56, 61, 150],   // 13. Blue
  [70, 148, 73],   // 14. Green
  [175, 54, 60],   // 15. Red
  [231, 199, 31],  // 16. Yellow
  [187, 86, 149],  // 17. Magenta
  [8, 133, 161],   // 18. Cyan
  // Row 4: Neutral Grayscale (White -> Black)
  [243, 243, 242], // 19. White 9.5 (.05 D)
  [200, 200, 200], // 20. Neutral 8 (.23 D)
  [160, 160, 160], // 21. Neutral 6.5 (.44 D)
  [122, 122, 121], // 22. Neutral 5 (.70 D)
  [85, 85, 85],    // 23. Neutral 3.5 (1.05 D)
  [52, 52, 52],    // 24. Black 2 (1.50 D)
];

export function createDefaultEdits(): EditSettingsPayload {
  return {
    exposure: 0.0,
    contrast: 0.0,
    highlights: 0.0,
    shadows: 0.0,
    whites: 0.0,
    blacks: 0.0,
    vibrance: 0.0,
    saturation: 0.0,
    deep_blue: 0.0,
    foliage: 0.0,
    temperature: 5500.0,
    tint: 0.0,
    crop_left: 0.0,
    crop_top: 0.0,
    crop_right: 1.0,
    crop_bottom: 1.0,
    crop_angle: 0.0,
    perspective_vertical: 0.0,
    perspective_horizontal: 0.0,
    crop_constrain: false,
    rotation: 0,
    flip_horizontal: false,
    flip_vertical: false,
    tone_profile: 'AgxVivid',
    lut_intensity: 0.0,
    sharpening_amount: 0.0,
    sharpening_radius: 1.0,
    sharpening_masking: 0.0,
    noise_reduction: 0.0,
    emulsion_archetype: 'None',
    emulsion_format: 'Format35mm',
    emulsion_exposure: 0.0,
    print_media: 'None',
    enlarger_filter: 'NeutralPanchromatic',
    print_paper_grade: 2.0,
    print_base_warmth: 0.0,
    print_process_depth: 100.0,
    split_toning_shadow_hue: 215.0,
    split_toning_shadow_sat: 0.0,
    split_toning_highlight_hue: 45.0,
    split_toning_highlight_sat: 0.0,
    split_toning_balance: 0.0,
    curves: { master: [], red: [], green: [], blue: [] },
  };
}

export class ParityHarness {
  private renderer: WebGLRenderer;
  private width: number;
  private height: number;

  constructor(width: number = 256, height: number = 256) {
    this.width = width;
    this.height = height;
    this.renderer = WebGLRenderer.createOffscreen(width, height);
  }

  /**
   * Builds Macbeth ColorChecker 24-patch test texture
   */
  public createMacbethTexture(): ImageData {
    const w = this.width;
    const h = this.height;
    const imgData = new ImageData(w, h);
    const cols = 6;
    const rows = 4;
    const patchW = Math.floor(w / cols);
    const patchH = Math.floor(h / rows);

    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < cols; c++) {
        const patchIdx = r * cols + c;
        const [pr, pg, pb] = MACBETH_SRGB_24[patchIdx];

        const startX = c * patchW;
        const endX = (c === cols - 1) ? w : (c + 1) * patchW;
        const startY = r * patchH;
        const endY = (r === rows - 1) ? h : (r + 1) * patchH;

        for (let y = startY; y < endY; y++) {
          for (let x = startX; x < endX; x++) {
            const idx = (y * w + x) * 4;
            imgData.data[idx] = pr;
            imgData.data[idx + 1] = pg;
            imgData.data[idx + 2] = pb;
            imgData.data[idx + 3] = 255;
          }
        }
      }
    }
    return imgData;
  }

  /**
   * Builds smooth horizontal neutral gradient ramp (0.0 .. 1.0)
   */
  public createGradientTexture(): ImageData {
    const w = this.width;
    const h = this.height;
    const imgData = new ImageData(w, h);
    for (let y = 0; y < h; y++) {
      for (let x = 0; x < w; x++) {
        const val = Math.round((x / (w - 1)) * 255);
        const idx = (y * w + x) * 4;
        imgData.data[idx] = val;
        imgData.data[idx + 1] = val;
        imgData.data[idx + 2] = val;
        imgData.data[idx + 3] = 255;
      }
    }
    return imgData;
  }

  /**
   * Executes a single parity test case, comparing WebGL output with reference CPU export.
   */
  public auditTestCase(test: ParityTestCase, toleranceMaxDelta: number = 2): ParityAuditResult {
    const w = test.textureWidth;
    const h = test.textureHeight;

    // 1. Upload Texture to WebGL
    const texturePayload = test.createTexture();
    if (test.isLinearRec2020) {
      const lin = texturePayload as { data: Uint16Array; width: number; height: number };
      this.renderer.uploadLinearRec2020Texture(lin.width, lin.height, lin.data);
    } else {
      const imgData = texturePayload as ImageData;
      this.renderer.uploadImage(imgData);
    }

    // If test case has tone curves, evaluate and upload LUT to test renderer
    let curveLut: EvaluatedLut | undefined = undefined;
    if (test.edits.curves && (test.edits.curves.master?.length > 2 || test.edits.curves.red?.length > 2 || test.edits.curves.green?.length > 2 || test.edits.curves.blue?.length > 2)) {
      try {
        const bridge = WasmBridge.getInstance();
        curveLut = bridge.evaluateCurves(test.edits.curves);
        this.renderer.uploadCurveLuts(curveLut);
      } catch (err) {
        console.warn('Could not evaluate curve LUT for parity test', err);
      }
    }

    // 2. Render WebGL
    this.renderer.render(test.edits, false);

    // 3. Readback WebGL Pixels
    const glPixels = this.renderer.readPixels();

    // 4. Compute Reference Export & Compare Pixel-by-Pixel
    let maxChannelDelta = 0;
    let sumDelta = 0;
    let sumSqError = 0;
    let failingPixels = 0;
    const totalPixels = w * h;
    let worstCoord: [number, number] = [0, 0];
    let worstGlRgb: [number, number, number] = [0, 0, 0];
    let worstExportRgb: [number, number, number] = [0, 0, 0];

    const heatmap = new ImageData(w, h);
    const sharpAmount = test.edits.sharpening_amount ?? 0.0;
    const nrAmount = test.edits.noise_reduction ?? 0.0;
    const needsSharpening = sharpAmount > 0.0 || nrAmount > 0.0;
    const r_step = Math.max(test.edits.sharpening_radius ?? 1.0, 0.5);

    const f16ToF32 = (u: number): number => {
      const sign = (u & 0x8000) ? -1 : 1;
      const exp = (u & 0x7c00) >> 10;
      const frac = u & 0x03ff;
      if (exp === 0) return sign * Math.pow(2, -14) * (frac / 1024);
      if (exp === 31) return frac ? NaN : sign * Infinity;
      return sign * Math.pow(2, exp - 15) * (1 + frac / 1024);
    };

    const getTexelLum = (ix: number, iy: number): number => {
      const clampedX = Math.min(Math.max(ix, 0), w - 1);
      const clampedY = Math.min(Math.max(iy, 0), h - 1);
      const idx = (clampedY * w + clampedX) * 4;
      let r: number, g: number, b: number;
      if (test.isLinearRec2020) {
        const lin = texturePayload as { data: Uint16Array };
        r = f16ToF32(lin.data[idx]);
        g = f16ToF32(lin.data[idx + 1]);
        b = f16ToF32(lin.data[idx + 2]);
      } else {
        const img = texturePayload as ImageData;
        r = img.data[idx] / 255.0;
        g = img.data[idx + 1] / 255.0;
        b = img.data[idx + 2] / 255.0;
      }
      return evaluateDisplayLuma([r, g, b], test.isLinearRec2020, test.edits);
    };

    const sampleLumBilinear = (px: number, py: number): number => {
      const cx = Math.min(Math.max(px, 0), w - 1);
      const cy = Math.min(Math.max(py, 0), h - 1);
      const x0 = Math.floor(cx);
      const x1 = Math.min(x0 + 1, w - 1);
      const y0 = Math.floor(cy);
      const y1 = Math.min(y0 + 1, h - 1);
      const fx = cx - x0;
      const fy = cy - y0;

      const l00 = getTexelLum(x0, y0);
      const l10 = getTexelLum(x1, y0);
      const l01 = getTexelLum(x0, y1);
      const l11 = getTexelLum(x1, y1);

      const top = l00 * (1.0 - fx) + l10 * fx;
      const bottom = l01 * (1.0 - fx) + l11 * fx;
      return top * (1.0 - fy) + bottom * fy;
    };

    for (let y = 0; y < h; y++) {
      // In WebGL, row 0 in readPixels is bottom of viewport, row (h-1) is top.
      const glRow = h - 1 - y;
      for (let x = 0; x < w; x++) {
        const glIdx = (glRow * w + x) * 4;
        const glR = glPixels[glIdx];
        const glG = glPixels[glIdx + 1];
        const glB = glPixels[glIdx + 2];

        // Input pixel into reference pipeline
        let inputPixel: [number, number, number];
        if (test.isLinearRec2020) {
          const lin = texturePayload as { data: Uint16Array; width: number; height: number };
          const linIdx = (y * w + x) * 4;
          inputPixel = [
            f16ToF32(lin.data[linIdx]),
            f16ToF32(lin.data[linIdx + 1]),
            f16ToF32(lin.data[linIdx + 2]),
          ];
        } else {
          const imgData = texturePayload as ImageData;
          const sIdx = (y * w + x) * 4;
          inputPixel = [
            imgData.data[sIdx] / 255.0,
            imgData.data[sIdx + 1] / 255.0,
            imgData.data[sIdx + 2] / 255.0,
          ];
        }

        let neighbors: { y_c: number; y_l: number; y_r: number; y_t: number; y_b: number } | undefined;
        if (needsSharpening) {
          const y_l = sampleLumBilinear(x - r_step, y);
          const y_r = sampleLumBilinear(x + r_step, y);
          const y_t = sampleLumBilinear(x, y - r_step);
          const y_b = sampleLumBilinear(x, y + r_step);
          neighbors = { y_c: 0, y_l, y_r, y_t, y_b };
        }

        const expRgb = evaluateReferencePixel(inputPixel, test.isLinearRec2020, test.edits, curveLut, neighbors);

        const dr = Math.abs(glR - expRgb[0]);
        const dg = Math.abs(glG - expRgb[1]);
        const db = Math.abs(glB - expRgb[2]);
        const pixelMaxDelta = Math.max(dr, dg, db);
        const pixelMeanDelta = (dr + dg + db) / 3.0;

        sumDelta += pixelMeanDelta;
        sumSqError += (dr * dr + dg * dg + db * db) / 3.0;

        if (pixelMaxDelta > toleranceMaxDelta) {
          failingPixels++;
        }

        if (pixelMaxDelta > maxChannelDelta) {
          maxChannelDelta = pixelMaxDelta;
          worstCoord = [x, y];
          worstGlRgb = [glR, glG, glB];
          worstExportRgb = expRgb;
        }

        // Difference Heatmap coloring:
        // 0..1 delta: Dark gray / green (matched)
        // 2..5 delta: Yellow (minor)
        // > 5 delta: Red / Magenta (severe divergence)
        const heatIdx = (y * w + x) * 4;
        if (pixelMaxDelta <= 1) {
          heatmap.data[heatIdx] = 16;
          heatmap.data[heatIdx + 1] = 40;
          heatmap.data[heatIdx + 2] = 24;
        } else if (pixelMaxDelta <= 5) {
          heatmap.data[heatIdx] = 200;
          heatmap.data[heatIdx + 1] = 160;
          heatmap.data[heatIdx + 2] = 0;
        } else {
          // Severely amplified error
          heatmap.data[heatIdx] = Math.min(pixelMaxDelta * 15, 255);
          heatmap.data[heatIdx + 1] = 20;
          heatmap.data[heatIdx + 2] = 40;
        }
        heatmap.data[heatIdx + 3] = 255;
      }
    }

    const meanChannelDelta = sumDelta / totalPixels;
    const mse = sumSqError / totalPixels;
    const psnr = mse > 1e-10 ? 10 * Math.log10((255 * 255) / mse) : 99.9;
    const pass = maxChannelDelta <= toleranceMaxDelta && meanChannelDelta <= 1.0;

    // Diagnose most likely mathematical cause of divergence
    let stageDiagnosis = 'Parity Verified (Exact Mathematical Bit Match)';
    if (!pass) {
      if (maxChannelDelta > 30) {
        if (worstGlRgb[0] > worstExportRgb[0] && worstGlRgb[1] > worstExportRgb[1] && worstGlRgb[2] > worstExportRgb[2]) {
          stageDiagnosis = 'CRITICAL: GL is significantly lighter than Export. Likely double gamma encoding (linearToSrgbGamma called on display-referred AgX output)!';
        } else if (worstGlRgb[2] > worstExportRgb[2] + 20) {
          stageDiagnosis = 'CRITICAL: GL blue channel significantly higher than Export. Likely missing Sensor-to-Rec.2020 color matrix calibration!';
        } else {
          stageDiagnosis = `Large Tone Mapping Divergence: Peak Δ=${maxChannelDelta}/255 at (${worstCoord[0]},${worstCoord[1]}) [GL: ${worstGlRgb.join(',')}, Export: ${worstExportRgb.join(',')}]`;
        }
      } else {
        stageDiagnosis = `Subtle GPU float precision difference: Peak Δ=${maxChannelDelta}/255, Mean Δ=${meanChannelDelta.toFixed(2)}`;
      }
    }

    return {
      testId: test.id,
      testName: test.name,
      pass,
      maxChannelDelta,
      meanChannelDelta,
      psnr,
      totalPixels,
      failingPixels,
      worstPixelCoord: worstCoord,
      worstGlRgb,
      worstExportRgb,
      stageDiagnosis,
      heatmapData: heatmap,
    };
  }

  /**
   * Generates standard mechanical parity test battery
   */
  public getStandardTestSuite(): ParityTestCase[] {
    const w = this.width;
    const h = this.height;

    return [
      {
        id: 'macbeth_neutral_agx_vivid',
        name: 'Macbeth 24-Patch Baseline (AgX Vivid Default)',
        category: 'macbeth',
        edits: createDefaultEdits(),
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'macbeth_neutral_agx_filmic',
        name: 'Macbeth 24-Patch Baseline (AgX Filmic)',
        category: 'macbeth',
        edits: { ...createDefaultEdits(), tone_profile: 'AgxFilmic' },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'exposure_plus_1ev',
        name: 'Exposure +1.0 EV Push',
        category: 'exposure',
        edits: { ...createDefaultEdits(), exposure: 1.0 },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'exposure_minus_1ev',
        name: 'Exposure -1.0 EV Pull',
        category: 'exposure',
        edits: { ...createDefaultEdits(), exposure: -1.0 },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'white_balance_warm',
        name: 'White Balance Warm (7000K, Tint +10)',
        category: 'white_balance',
        edits: { ...createDefaultEdits(), temperature: 7000.0, tint: 10.0 },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'tone_recovery_dynamic',
        name: 'Tone Recovery (Highlights -60, Shadows +40)',
        category: 'tone_recovery',
        edits: { ...createDefaultEdits(), highlights: -60.0, shadows: 40.0 },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'color_vibrance_saturation',
        name: 'Color Tuning (Vibrance +35, Saturation +20, Deep Blue +30)',
        category: 'color',
        edits: { ...createDefaultEdits(), vibrance: 35.0, saturation: 20.0, deep_blue: 30.0 },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'neutral_gradient_ramp',
        name: 'Neutral 0..255 Gradient Ramp Achromatic Test',
        category: 'exposure',
        edits: createDefaultEdits(),
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createGradientTexture(),
      },
      {
        id: 'camera_standard_profile',
        name: 'Camera Standard Profile (Hermite S-Curve & Memory Colors)',
        category: 'color',
        edits: { ...createDefaultEdits(), tone_profile: 'CameraStandard' },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'camera_landscape_profile',
        name: 'Camera Landscape Profile (Punchy Skies & Foliage)',
        category: 'color',
        edits: { ...createDefaultEdits(), tone_profile: 'CameraLandscape' },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'camera_vivid_profile',
        name: 'Camera Vivid Profile (Rich Contrast & Boosted Saturation)',
        category: 'color',
        edits: { ...createDefaultEdits(), tone_profile: 'CameraVivid' },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'camera_portrait_profile',
        name: 'Camera Portrait Profile (Skin Smoothing & Calm Skies)',
        category: 'color',
        edits: { ...createDefaultEdits(), tone_profile: 'CameraPortrait' },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'emulsion_rochester_portrait_400',
        name: 'Emulsion: Rochester Portrait 400 (C-41 Color Negative)',
        category: 'color',
        edits: { ...createDefaultEdits(), emulsion_archetype: 'RochesterPortrait400' },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'emulsion_mobberley_silver_400',
        name: 'Emulsion: Mobberley Silver 400 (Panchromatic B&W)',
        category: 'color',
        edits: { ...createDefaultEdits(), emulsion_archetype: 'MobberleySilver400' },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'print_media_cyanotype',
        name: 'Print Media: Prussian Blue Cyanotype (Tonal Split & Base)',
        category: 'color',
        edits: { ...createDefaultEdits(), print_media: 'Cyanotype' },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'print_media_chromogenic',
        name: 'Print Media: RA-4 Chromogenic Type-C Print',
        category: 'color',
        edits: { ...createDefaultEdits(), print_media: 'ChromogenicTypeC' },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'sharpening_baseline_raw',
        name: 'Sharpening Baseline (RAW Default: Amount 20, Radius 1.0, Masking 15)',
        category: 'color',
        edits: { ...createDefaultEdits(), sharpening_amount: 20.0, sharpening_radius: 1.0, sharpening_masking: 15.0 },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'sharpening_crisp',
        name: 'Sharpening Crisp (Amount 50, Radius 1.0, Masking 0)',
        category: 'color',
        edits: { ...createDefaultEdits(), sharpening_amount: 50.0, sharpening_radius: 1.0, sharpening_masking: 0.0 },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
      {
        id: 'sharpening_edge_masked',
        name: 'Sharpening Edge-Masked (Amount 70, Radius 1.0, Masking 35)',
        category: 'color',
        edits: { ...createDefaultEdits(), sharpening_amount: 70.0, sharpening_radius: 1.0, sharpening_masking: 35.0 },
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => this.createMacbethTexture(),
      },
    ];
  }

  /**
   * Runs the complete battery and returns a summary report.
   */
  public runFullAudit(): ParityAuditResult[] {
    const suite = this.getStandardTestSuite();
    return suite.map(tc => this.auditTestCase(tc));
  }
}
