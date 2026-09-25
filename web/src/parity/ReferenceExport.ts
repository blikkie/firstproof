/**
 * First Proof — Reference Mathematical Export Pipeline (CPU Parity Oracle)
 *
 * Implements the exact sensitometric and color science pipeline of firstproof-core
 * in high-precision float64/float32 arithmetic. Used as the mechanical gold standard
 * to audit and drive bit-level parity against WebGL and Metal GPU shaders.
 */

export interface Vec3 {
  r: number;
  g: number;
  b: number;
}

export interface ParityPixelResult {
  glRgb: [number, number, number];
  exportRgb: [number, number, number];
  deltaRgb: [number, number, number]; // gl - export (-255..255)
  maxDelta: number; // max(|dr|, |dg|, |db|)
  meanDelta: number;
  psnr: number;
}

// Matrices matching firstproof-core/src/color/
const LINEAR_SRGB_TO_REC2020 = [
  [0.6274041, 0.3292821, 0.0433138],
  [0.0690972, 0.9195411, 0.0113617],
  [0.0163914, 0.0880133, 0.8955953],
];

const REC2020_TO_LINEAR_SRGB = [
  [1.6604910, -0.5876411, -0.0728499],
  [-0.1245505, 1.1328999, -0.0083494],
  [-0.0181508, -0.1005789, 1.1187297],
];

const AGX_INSET_MATRIX = [
  [1.4096986, -0.4004796, -0.009218898],
  [0.13136263, 0.77151406, 0.09712328],
  [0.16151477, -0.06034984, 0.89883506],
];

const AGX_OUTSET_MATRIX = [
  [1.1271005, -0.1106066, -0.0164939],
  [-0.1413298, 1.1578237, -0.0164939],
  [-0.1413298, -0.1106066, 1.2519364],
];

const AGX_MIN_EV = -15.2;
const AGX_EV_RANGE = 20.2;
const AGX_GAMMA = 2.4;
const AGX_MID_GRAY_NORM = 0.75247525;

export function srgbGammaToLinear(c: number): number {
  const v = Math.min(Math.max(c, 0.0), 1.0);
  return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
}

export function linearToSrgbGamma(c: number): number {
  const v = Math.min(Math.max(c, 0.0), 1.0);
  return v <= 0.0031308 ? v * 12.92 : 1.055 * Math.pow(v, 1.0 / 2.4) - 0.055;
}

export function matMul(m: number[][], v: [number, number, number]): [number, number, number] {
  return [
    m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
    m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
    m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
  ];
}

export function rgb2hsv(r: number, g: number, b: number): [number, number, number] {
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const d = max - min;
  let h = 0;
  const s = max === 0 ? 0 : d / max;
  const v = max;

  if (d > 1e-7) {
    if (max === r) {
      h = (g - b) / d + (g < b ? 6 : 0);
    } else if (max === g) {
      h = (b - r) / d + 2;
    } else {
      h = (r - g) / d + 4;
    }
    h *= 60;
  }
  return [h, s, v];
}

export function hsv2rgb(h: number, s: number, v: number): [number, number, number] {
  const c = v * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = v - c;
  let r = 0, g = 0, b = 0;

  if (h >= 0 && h < 60) {
    r = c; g = x; b = 0;
  } else if (h >= 60 && h < 120) {
    r = x; g = c; b = 0;
  } else if (h >= 120 && h < 180) {
    r = 0; g = c; b = x;
  } else if (h >= 180 && h < 240) {
    r = 0; g = x; b = c;
  } else if (h >= 240 && h < 300) {
    r = x; g = 0; b = c;
  } else {
    r = c; g = 0; b = x;
  }
  return [r + m, g + m, b + m];
}

function smoothstep(edge0: number, edge1: number, x: number): number {
  const t = Math.min(Math.max((x - edge0) / (edge1 - edge0), 0.0), 1.0);
  return t * t * (3.0 - 2.0 * t);
}

export function applyCameraToneCurve(x: number, mode: number): number {
  const xc = Math.min(Math.max(x, 0.0), 1.0);
  // mode 1: Standard, mode 2: Portrait, mode 3: Landscape, mode 4: Vivid
  const mid = mode === 1 ? 0.185 : mode === 2 ? 0.180 : mode === 3 ? 0.200 : mode === 4 ? 0.210 : 0.185;
  if (xc <= 0.18) {
    const t = xc / 0.18;
    return mid * (t * t * (3.0 - 2.0 * t));
  } else {
    const t = (xc - 0.18) / 0.82;
    return mid + (1.0 - mid) * (t * (2.0 - t));
  }
}

export function applyCameraHueSat(hsv: [number, number, number], mode: number): [number, number, number] {
  let [h, s, v] = hsv;
  let skyWeight = Math.min(Math.max(1.0 - Math.abs(h - 235.0) / 35.0, 0.0), 1.0);
  const vSky = smoothstep(0.20, 0.40, v);
  const sSky = smoothstep(0.08, 0.25, s);
  skyWeight *= (vSky * sSky);

  const greenWeight = Math.min(Math.max(1.0 - Math.abs(h - 120.0) / 35.0, 0.0), 1.0);
  const skinWeight = Math.min(Math.max(1.0 - Math.abs(h - 30.0) / 20.0, 0.0), 1.0);

  h -= 18.0 * skyWeight;
  if (h < 0.0) { h += 360.0; }

  if (mode === 1) { // Camera Standard
    s *= (1.0 - 0.20 * skyWeight + 0.10 * greenWeight + 0.05 * skinWeight);
    v *= (1.0 - 0.03 * skyWeight + 0.03 * skinWeight);
  } else if (mode === 2) { // Camera Portrait
    s *= (1.0 - 0.05 * skinWeight - 0.22 * skyWeight);
    v *= (1.0 + 0.04 * skinWeight);
  } else if (mode === 3) { // Camera Landscape
    s *= (1.0 - 0.12 * skyWeight + 0.20 * greenWeight);
    v *= (1.0 - 0.05 * skyWeight - 0.04 * greenWeight);
  } else if (mode === 4) { // Camera Vivid
    s *= (1.20 - 0.15 * skyWeight);
  }
  return [h, Math.min(Math.max(s, 0.0), 1.0), Math.min(Math.max(v, 0.0), 1.0)];
}

export function isNeutralEdits(edits: {
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
  tone_profile?: string;
  split_toning_shadow_sat: number;
  split_toning_highlight_sat: number;
  sharpening_amount?: number;
  noise_reduction?: number;
  emulsion_archetype?: string;
  print_media?: string;
}): boolean {
  return (
    (edits.tone_profile === 'AgxVivid' || !edits.tone_profile) &&
    edits.exposure === 0.0 &&
    edits.contrast === 0.0 &&
    edits.highlights === 0.0 &&
    edits.shadows === 0.0 &&
    edits.whites === 0.0 &&
    edits.blacks === 0.0 &&
    Math.abs(edits.temperature - 5500.0) < 1.0 &&
    edits.tint === 0.0 &&
    edits.vibrance === 0.0 &&
    edits.saturation === 0.0 &&
    edits.deep_blue === 0.0 &&
    edits.foliage === 0.0 &&
    edits.split_toning_shadow_sat === 0.0 &&
    edits.split_toning_highlight_sat === 0.0 &&
    (edits.sharpening_amount === undefined || edits.sharpening_amount === 0.0) &&
    (edits.noise_reduction === undefined || edits.noise_reduction === 0.0) &&
    (!edits.emulsion_archetype || edits.emulsion_archetype === 'None') &&
    (!edits.print_media || edits.print_media === 'None')
  );
}

// Detail, Sharpening & Noise Reduction (matching firstproof-core pipeline.rs & Detail.metal)
export function applyDetailAndSharpeningRgb(
  r: number, g: number, b: number,
  y_c: number,
  y_l: number, y_r: number, y_t: number, y_b: number,
  amount: number,
  masking: number,
  noiseReduction: number
): [number, number, number] {
  if (amount <= 0.0 && noiseReduction <= 0.0) {
    return [r, g, b];
  }

  // Unsharp mask: Laplacian high-frequency detail
  const y_neighbors = 0.25 * (y_l + y_r + y_t + y_b);
  const delta_y = y_c - y_neighbors;

  // Sobel gradient magnitude
  const gx = (y_r - y_l) * 0.5;
  const gy = (y_b - y_t) * 0.5;
  const grad_mag = Math.sqrt(gx * gx + gy * gy);

  let w_edge = 1.0;
  if (masking > 0.0) {
    const m = Math.min(Math.max(masking / 100.0, 0.0), 1.0);
    const threshold = Math.pow(m, 1.5) * 0.08;
    const t = Math.min(Math.max(grad_mag / Math.max(threshold, 1e-4), 0.0), 1.0);
    w_edge = t * t * (3.0 - 2.0 * t);
  }

  const sharp_gain = (amount / 100.0) * 1.5;
  const sharp_term = sharp_gain * w_edge * delta_y;
  const nr_term = (noiseReduction / 100.0) * (1.0 - w_edge) * delta_y;
  const delta_adj = sharp_term - nr_term;

  return [
    Math.min(Math.max(r + delta_adj, 0.0), 1.0),
    Math.min(Math.max(g + delta_adj, 0.0), 1.0),
    Math.min(Math.max(b + delta_adj, 0.0), 1.0),
  ];
}

export function evaluateDisplayLuma(
  input: [number, number, number],
  isLinearRec2020: boolean,
  edits: {
    exposure: number;
    contrast: number;
    highlights: number;
    shadows: number;
    whites: number;
    blacks: number;
    temperature: number;
    tint: number;
    tone_profile?: string;
    emulsion_archetype?: string;
    emulsion_exposure?: number;
  }
): number {
  let rec2020: [number, number, number];
  if (isLinearRec2020) {
    rec2020 = [input[0], input[1], input[2]];
  } else {
    const srgbLin: [number, number, number] = [
      srgbGammaToLinear(input[0]),
      srgbGammaToLinear(input[1]),
      srgbGammaToLinear(input[2]),
    ];
    rec2020 = matMul(LINEAR_SRGB_TO_REC2020, srgbLin);
  }

  if (Math.abs(edits.temperature - 5500.0) >= 1.0 || edits.tint !== 0.0) {
    const tempNorm = Math.min(Math.max(edits.temperature / 5500.0, 0.25), 4.0);
    let rScale = Math.pow(tempNorm, 0.75);
    let bScale = Math.pow(1.0 / tempNorm, 0.75);
    const tintFactor = edits.tint * 0.002;
    const gScale = Math.max(1.0 - tintFactor, 0.01);
    rScale = Math.max(rScale + tintFactor * 0.5, 0.01);
    bScale = Math.max(bScale + tintFactor * 0.5, 0.01);
    rec2020[0] *= rScale;
    rec2020[1] *= gScale;
    rec2020[2] *= bScale;
  }

  if (edits.exposure !== 0.0) {
    const gain = Math.pow(2.0, edits.exposure);
    rec2020[0] *= gain;
    rec2020[1] *= gain;
    rec2020[2] *= gain;
  }

  rec2020 = applyToneRecovery(rec2020, edits.highlights, edits.shadows, edits.whites, edits.blacks);

  if (edits.emulsion_archetype && edits.emulsion_archetype !== 'None') {
    rec2020 = applyRecordingEmulsion(rec2020, edits.emulsion_archetype, edits.emulsion_exposure ?? 0.0);
  }

  const contrastGain = edits.contrast >= 0.0 ? 1.0 + edits.contrast / 100.0 : 1.0 + (edits.contrast / 100.0) * 0.60;
  const isAgxVivid = edits.tone_profile === 'AgxVivid' || !edits.tone_profile;
  const isAgxFilmic = edits.tone_profile === 'AgxFilmic';

  const CAM_MODE_MAP: Record<string, number> = {
    CameraStandard: 1,
    CameraPortrait: 2,
    CameraLandscape: 3,
    CameraVivid: 4,
  };

  let srgbOut: [number, number, number];
  if (isAgxVivid) {
    srgbOut = agxToneMapVivid(rec2020, contrastGain);
  } else if (isAgxFilmic) {
    srgbOut = agxToneMap(rec2020, contrastGain);
  } else if (edits.tone_profile && edits.tone_profile in CAM_MODE_MAP) {
    const camMode = CAM_MODE_MAP[edits.tone_profile];
    const hsv = rgb2hsv(rec2020[0], rec2020[1], rec2020[2]);
    const hsvAdj = applyCameraHueSat(hsv, camMode);
    const rgbAdj = hsv2rgb(hsvAdj[0], hsvAdj[1], hsvAdj[2]);
    const lum = 0.2627 * rgbAdj[0] + 0.6780 * rgbAdj[1] + 0.0593 * rgbAdj[2];
    const lumCurved = applyCameraToneCurve(lum, camMode);
    return Math.min(Math.max((lumCurved - 0.18) * contrastGain + 0.18, 0.0), 1.0);
  } else {
    const cSrgb = matMul(REC2020_TO_LINEAR_SRGB, rec2020);
    srgbOut = [Math.min(Math.max(cSrgb[0], 0.0), 1.0), Math.min(Math.max(cSrgb[1], 0.0), 1.0), Math.min(Math.max(cSrgb[2], 0.0), 1.0)];
  }

  return 0.2126 * Math.min(Math.max(srgbOut[0], 0.0), 1.0) +
         0.7152 * Math.min(Math.max(srgbOut[1], 0.0), 1.0) +
         0.0722 * Math.min(Math.max(srgbOut[2], 0.0), 1.0);
}

function agxSigmoid(x: number, power: number): number {
  return x / Math.pow(1.0 + Math.pow(x, power), 1.0 / power);
}

function agxScaledSigmoid(x: number, scale: number, slope: number, power: number, tx: number, ty: number): number {
  return scale * agxSigmoid((slope * (x - tx)) / scale, power) + ty;
}

export function agxApplyCurveChannel(x: number): number {
  if (x < 0.6060606) {
    return Math.min(Math.max(agxScaledSigmoid(x, -1.0359, 2.3843, 1.5, 0.6060606, 0.43446), 0.0), 1.0);
  } else {
    return Math.min(Math.max(agxScaledSigmoid(x, 1.3475, 2.3843, 1.5, 0.6060606, 0.43446), 0.0), 1.0);
  }
}

export function agxToneMap(rec2020: [number, number, number], contrastGain: number): [number, number, number] {
  const minC = Math.min(rec2020[0], rec2020[1], rec2020[2]);
  const cComp: [number, number, number] = minC < 0 ? [rec2020[0] - minC, rec2020[1] - minC, rec2020[2] - minC] : [...rec2020];

  const inset = matMul(AGX_INSET_MATRIX, cComp);
  const process = (v: number): number => {
    const xRel = Math.max(v / 0.18, 1e-6);
    const logEncoded = (Math.log2(xRel) - AGX_MIN_EV) / AGX_EV_RANGE;
    let norm = Math.min(Math.max(logEncoded, 0.0), 1.0);
    if (Math.abs(contrastGain - 1.0) > 0.001) {
      norm = Math.min(Math.max(AGX_MID_GRAY_NORM + (norm - AGX_MID_GRAY_NORM) * contrastGain, 0.0), 1.0);
    }
    const curved = agxApplyCurveChannel(norm);
    return Math.pow(Math.max(curved, 0.0), AGX_GAMMA);
  };

  const linearized: [number, number, number] = [process(inset[0]), process(inset[1]), process(inset[2])];
  const outset = matMul(AGX_OUTSET_MATRIX, linearized);
  return [Math.max(outset[0], 0.0), Math.max(outset[1], 0.0), Math.max(outset[2], 0.0)];
}

export function agxToneMapVivid(rec2020: [number, number, number], contrastGain: number): [number, number, number] {
  const base = agxToneMap(rec2020, contrastGain * 1.15);
  let [h, s, v] = rgb2hsv(base[0], base[1], base[2]);

  const skinWeight = Math.min(Math.max(1.0 - Math.abs(h - 25.0) / 20.0, 0.0), 1.0);
  let skyWeight = Math.min(Math.max(1.0 - Math.abs(h - 235.0) / 35.0, 0.0), 1.0);
  const vSky = smoothstep(0.20, 0.40, v);
  const sSky = smoothstep(0.08, 0.25, s);
  skyWeight *= (vSky * sSky);

  h -= 18.0 * skyWeight;
  h -= 3.0 * skinWeight;
  if (h < 0.0) h += 360.0;
  if (h >= 360.0) h -= 360.0;

  s = Math.min(Math.max(s * (1.18 - 0.20 * skinWeight - 0.15 * skyWeight), 0.0), 1.0);
  const rgb = hsv2rgb(h, s, v);
  return [Math.min(Math.max(rgb[0], 0.0), 1.0), Math.min(Math.max(rgb[1], 0.0), 1.0), Math.min(Math.max(rgb[2], 0.0), 1.0)];
}

export function applyToneRecovery(
  color: [number, number, number],
  highlights: number,
  shadows: number,
  whites: number,
  blacks: number
): [number, number, number] {
  if (highlights === 0 && shadows === 0 && whites === 0 && blacks === 0) return color;
  const y = 0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2];
  if (y <= 1e-6) return color;

  const evBase = Math.log2(Math.max(y, 1e-5) / 0.18);
  let deltaEv = 0.0;

  if (blacks !== 0) {
    const u = Math.min(Math.max((-evBase - 1.0) / 3.0, 0.0), 1.0);
    const wBlacks = u * u * (3.0 - 2.0 * u);
    deltaEv += (blacks / 100.0) * 0.90 * wBlacks;
  }
  if (shadows !== 0) {
    const u = Math.min(Math.max((evBase + 4.0) / 4.0, 0.0), 1.0);
    const wShadows = Math.sin(Math.PI * u);
    deltaEv += (shadows / 100.0) * 1.20 * wShadows;
  }
  if (highlights !== 0) {
    const u = Math.min(Math.max(evBase / 4.0, 0.0), 1.0);
    const wHighlights = Math.sin(Math.PI * u);
    deltaEv += (highlights / 100.0) * 1.30 * wHighlights;
  }
  if (whites !== 0) {
    const u = Math.min(Math.max((evBase - 1.0) / 3.0, 0.0), 1.0);
    const wWhites = u * u * (3.0 - 2.0 * u);
    deltaEv += (whites / 100.0) * 0.90 * wWhites;
  }

  const gain = Math.pow(2.0, deltaEv);
  return [color[0] * gain, color[1] * gain, color[2] * gain];
}

export function applyColorAdjustments(
  rgb: [number, number, number],
  vibrance: number,
  saturation: number,
  deepBlue: number,
  foliage: number
): [number, number, number] {
  if (vibrance === 0 && saturation === 0 && deepBlue === 0 && foliage === 0) return rgb;

  let [r, g, b] = [Math.min(Math.max(rgb[0], 0.0), 1.0), Math.min(Math.max(rgb[1], 0.0), 1.0), Math.min(Math.max(rgb[2], 0.0), 1.0)];
  const lum = 0.2627 * r + 0.6780 * g + 0.0593 * b;

  if (saturation !== 0) {
    const satGain = Math.max(1.0 + saturation / 100.0, 0.0);
    r = lum + (r - lum) * satGain;
    g = lum + (g - lum) * satGain;
    b = lum + (b - lum) * satGain;
  }

  if (vibrance !== 0) {
    const maxC = Math.max(r, g, b);
    const minC = Math.min(r, g, b);
    const chroma = maxC - minC;

    let skinProtect = 1.0;
    const sum = r + g + b;
    if (r > g && g > b && sum > 0.015) {
      const rN = r / sum;
      const gN = g / sum;
      const dR = Math.abs(rN - 0.44);
      const dG = Math.abs(gN - 0.32);
      const skinMetric = Math.min(Math.max(1.0 - (dR * 4.5 + dG * 9.0), 0.0), 1.0);
      skinProtect = 1.0 - 0.6 * skinMetric;
    }

    const vibGain = vibrance > 0 ? (vibrance / 100.0) * (1.0 - chroma) * 0.8 * skinProtect : (vibrance / 100.0) * 0.6;
    r += (r - lum) * vibGain;
    g += (g - lum) * vibGain;
    b += (b - lum) * vibGain;
  }

  if (deepBlue !== 0) {
    const maxWarm = Math.max(r, g);
    const skySignal = Math.max(b - maxWarm, 0.0);
    if (skySignal > 0.0) {
      const x = deepBlue / 100.0;
      const factor = Math.sign(x) * Math.pow(Math.abs(x), 1.6);
      if (factor > 0) {
        r -= factor * skySignal * 0.30;
        g -= factor * skySignal * 0.15;
        b -= factor * skySignal * 0.20;
      } else {
        r -= factor * skySignal * 0.25;
        g -= factor * skySignal * 0.20;
        b -= factor * skySignal * 0.10;
      }
    }
  }

  if (foliage !== 0) {
    const maxRb = Math.max(r, b);
    const greenSignal = Math.max(g - maxRb, 0.0);
    if (greenSignal > 0.0) {
      const x = foliage / 100.0;
      const factor = Math.sign(x) * Math.pow(Math.abs(x), 1.6);
      r -= factor * greenSignal * 0.30;
      b -= factor * greenSignal * 0.15;
      g += factor * greenSignal * 0.10;
    }
  }

  return [Math.min(Math.max(r, 0.0), 1.0), Math.min(Math.max(g, 0.0), 1.0), Math.min(Math.max(b, 0.0), 1.0)];
}

export function applySplitToning(
  rgb: [number, number, number],
  shadowHue: number,
  shadowSat: number,
  highlightHue: number,
  highlightSat: number,
  balance: number
): [number, number, number] {
  if (shadowSat <= 0.0 && highlightSat <= 0.0) return rgb;

  const lum = 0.2627 * rgb[0] + 0.6780 * rgb[1] + 0.0593 * rgb[2];
  const yClamp = Math.min(Math.max(lum, 0.0), 1.0);
  const t = Math.min(Math.max(0.5 - (balance / 100.0) * 0.25, 0.1), 0.9);
  let [dr, dg, db] = [0.0, 0.0, 0.0];

  if (shadowSat > 0.0 && yClamp < t) {
    const uS = (t - yClamp) / t;
    const wS = uS * uS * (3.0 - 2.0 * uS) * (shadowSat / 100.0);
    const tintRgb = hsv2rgb(shadowHue, 1.0, 1.0);
    const tLum = 0.2627 * tintRgb[0] + 0.6780 * tintRgb[1] + 0.0593 * tintRgb[2];
    dr += wS * (tintRgb[0] - tLum);
    dg += wS * (tintRgb[1] - tLum);
    db += wS * (tintRgb[2] - tLum);
  }

  if (highlightSat > 0.0 && yClamp > t) {
    const uH = (yClamp - t) / (1.0 - t);
    const wH = uH * uH * (3.0 - 2.0 * uH) * (highlightSat / 100.0);
    const tintRgb = hsv2rgb(highlightHue, 1.0, 1.0);
    const tLum = 0.2627 * tintRgb[0] + 0.6780 * tintRgb[1] + 0.0593 * tintRgb[2];
    dr += wH * (tintRgb[0] - tLum);
    dg += wH * (tintRgb[1] - tLum);
    db += wH * (tintRgb[2] - tLum);
  }

  return [
    Math.min(Math.max(rgb[0] + dr, 0.0), 1.0),
    Math.min(Math.max(rgb[1] + dg, 0.0), 1.0),
    Math.min(Math.max(rgb[2] + db, 0.0), 1.0),
  ];
}

export const EMULSION_MAP: Record<string, number> = {
  None: 0,
  RochesterPortrait400: 1,
  AshigaraChrome50: 2,
  RochesterK14Chrome64: 3,
  RochesterCinema500: 4,
  RochesterDaylight200: 5,
  MobberleySilver400: 6,
  CambridgeDiffusion100: 7,
};

export const PRINT_MEDIA_MAP: Record<string, number> = {
  None: 0,
  ChromogenicTypeC: 1,
  SilverDyeBleach: 2,
  FineArtCottonRag: 3,
  BarytaSilverGelatin: 4,
  WarmtoneChlorobromide: 5,
  PlatinumPalladium: 6,
  Cyanotype: 7,
  PrussianCyanotype: 7,
};

export function applyFilmCharacteristicCurve(val: number, gamma: number, shoulder: number, toe: number): number {
  const v = Math.min(Math.max(val, 0.0), 2.0);
  let base = v;
  if (Math.abs(gamma - 1.0) > 0.001) {
    const u = Math.min(Math.max(v / (v + 0.35), 0.0), 1.0);
    const gMod = (gamma - 1.0) * 0.40;
    base = Math.max(v + gMod * u * (1.0 - u) * (2.0 * u - 1.0), 0.0);
  }
  let afterShoulder = base;
  if (shoulder > 0.0 && base > 0.65) {
    const t = Math.min(Math.max((base - 0.65) / 0.35, 0.0), 1.0);
    const comp = shoulder * 0.12 * t * t * (3.0 - 2.0 * t);
    afterShoulder = base - comp;
  }
  if (toe > 0.0 && afterShoulder < 0.35) {
    const t = Math.min(Math.max(afterShoulder / 0.35, 0.0), 1.0);
    const comp = toe * 0.06 * (1.0 - t) * (1.0 - t);
    return Math.max(afterShoulder - comp, 0.0);
  }
  return afterShoulder;
}

export function applyDirChemicalCoupling(
  cur: [number, number, number],
  c_rg: number, c_rb: number,
  c_gr: number, c_gb: number,
  c_br: number, c_bg: number
): [number, number, number] {
  const eps = 1e-4;
  const d_r = -Math.log2(Math.max(cur[0], eps)) / 3.321928;
  const d_g = -Math.log2(Math.max(cur[1], eps)) / 3.321928;
  const d_b = -Math.log2(Math.max(cur[2], eps)) / 3.321928;

  const c_rr = 1.0 - c_rg - c_rb;
  const c_gg = 1.0 - c_gr - c_gb;
  const c_bb = 1.0 - c_br - c_bg;

  const d_r_out = c_rr * d_r + c_rg * d_g + c_rb * d_b;
  const d_g_out = c_gr * d_r + c_gg * d_g + c_gb * d_b;
  const d_b_out = c_br * d_r + c_bg * d_g + c_bb * d_b;

  return [
    Math.min(Math.max(Math.pow(10.0, -d_r_out), 0.0), 2.0),
    Math.min(Math.max(Math.pow(10.0, -d_g_out), 0.0), 2.0),
    Math.min(Math.max(Math.pow(10.0, -d_b_out), 0.0), 2.0),
  ];
}

export function applyRecordingEmulsion(
  rgb: [number, number, number],
  archetype: string,
  pushPullEv: number
): [number, number, number] {
  const mode = EMULSION_MAP[archetype] ?? 0;
  if (mode === 0) return rgb;

  const expGain = Math.pow(2.0, Math.min(Math.max(pushPullEv, -2.0), 2.0));
  const cur: [number, number, number] = [
    Math.max(rgb[0] * expGain, 0.0),
    Math.max(rgb[1] * expGain, 0.0),
    Math.max(rgb[2] * expGain, 0.0),
  ];

  if (mode === 1) { // Rochester Portrait 400
    const dir = applyDirChemicalCoupling(cur, -0.04, 0.02, -0.08, 0.02, 0.02, -0.05);
    return [
      applyFilmCharacteristicCurve(dir[0], 0.90, 0.38, 0.08),
      applyFilmCharacteristicCurve(dir[1], 0.90, 0.38, 0.08),
      applyFilmCharacteristicCurve(dir[2], 0.90, 0.38, 0.08),
    ];
  } else if (mode === 2) { // Ashigara Chrome 50
    const dir = applyDirChemicalCoupling(cur, -0.15, -0.08, -0.18, -0.08, -0.06, -0.12);
    return [
      applyFilmCharacteristicCurve(dir[0], 1.10, 0.12, 0.14),
      applyFilmCharacteristicCurve(dir[1], 1.12, 0.10, 0.14),
      applyFilmCharacteristicCurve(dir[2], 1.11, 0.12, 0.16),
    ];
  } else if (mode === 3) { // Rochester K-14 Chrome 64
    const dir = applyDirChemicalCoupling(cur, -0.08, 0.06, -0.10, 0.04, 0.08, -0.04);
    return [
      applyFilmCharacteristicCurve(dir[0], 1.05, 0.18, 0.08),
      applyFilmCharacteristicCurve(dir[1], 1.06, 0.16, 0.10),
      applyFilmCharacteristicCurve(dir[2], 1.08, 0.14, 0.14),
    ];
  } else if (mode === 4) { // Rochester Cinema 500T
    const dir = applyDirChemicalCoupling(cur, -0.02, 0.01, -0.03, 0.01, 0.01, -0.02);
    return [
      applyFilmCharacteristicCurve(dir[0], 0.82, 0.45, 0.08),
      applyFilmCharacteristicCurve(dir[1], 0.82, 0.45, 0.08),
      applyFilmCharacteristicCurve(dir[2], 0.82, 0.45, 0.08),
    ];
  } else if (mode === 5) { // Rochester Daylight 200
    const dir = applyDirChemicalCoupling(cur, -0.06, 0.03, -0.07, 0.02, 0.08, -0.10);
    return [
      applyFilmCharacteristicCurve(dir[0], 1.12, 0.20, 0.12),
      applyFilmCharacteristicCurve(dir[1], 1.08, 0.22, 0.12),
      applyFilmCharacteristicCurve(dir[2], 0.98, 0.26, 0.16),
    ];
  } else if (mode === 6) { // Mobberley Silver 400 (B&W)
    const panLum = 0.28 * cur[0] + 0.62 * cur[1] + 0.10 * cur[2];
    const mono = applyFilmCharacteristicCurve(panLum, 1.08, 0.30, 0.25);
    return [mono, mono, mono];
  } else if (mode === 7) { // Cambridge Instant Diffusion 100
    const dir = applyDirChemicalCoupling(cur, -0.03, 0.02, -0.04, 0.03, 0.01, -0.03);
    const curv: [number, number, number] = [
      applyFilmCharacteristicCurve(dir[0], 0.98, 0.35, 0.12),
      applyFilmCharacteristicCurve(dir[1], 0.98, 0.35, 0.12),
      applyFilmCharacteristicCurve(dir[2], 0.98, 0.35, 0.12),
    ];
    const lum = 0.2126 * curv[0] + 0.7152 * curv[1] + 0.0722 * curv[2];
    const baseRefl = 0.025 * Math.max(1.0 - lum, 0.0);
    return [
      Math.min(curv[0] + baseRefl * 1.02, 2.0),
      Math.min(curv[1] + baseRefl * 1.00, 2.0),
      Math.min(curv[2] + baseRefl * 0.94, 2.0),
    ];
  }
  return cur;
}

export function applyPaperGrade(val: number, grade: number): number {
  if (Math.abs(grade) < 0.001) return val;
  const alpha = 0.40 * (grade / 2.0);
  return Math.min(Math.max(val + alpha * val * (1.0 - val) * (2.0 * val - 1.0), 0.0), 1.0);
}

export function applyBaseWarmth(rgb: [number, number, number], warmth: number): [number, number, number] {
  if (Math.abs(warmth) < 0.001) return rgb;
  const lum = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2];
  const factor = (warmth / 100.0) * Math.min(Math.max(Math.pow(lum, 0.6), 0.0), 1.0);
  const mult = factor > 0
    ? [1.0 + factor * 0.060, 1.0 + factor * 0.025, 1.0 - factor * 0.045]
    : [1.0 + factor * 0.025, 1.0 + factor * 0.008, 1.0 - factor * 0.045];
  return [
    Math.min(Math.max(rgb[0] * mult[0], 0.0), 1.0),
    Math.min(Math.max(rgb[1] * mult[1], 0.0), 1.0),
    Math.min(Math.max(rgb[2] * mult[2], 0.0), 1.0),
  ];
}

export function applyPrintMedia(
  rgb: [number, number, number],
  printMedia: string,
  grade: number,
  warmth: number,
  depthPercent: number = 100.0,
  filterMode: number = 0
): [number, number, number] {
  const mode = PRINT_MEDIA_MAP[printMedia] ?? 0;
  if (mode === 0) return rgb;

  const depth = Math.min(Math.max(depthPercent / 100.0, 0.0), 1.0);

  const getEnlargerFilterWeights = (fm: number): [number, number, number] => {
    if (fm === 1) return [0.35, 0.60, 0.05];
    if (fm === 2) return [0.55, 0.42, 0.03];
    if (fm === 3) return [0.85, 0.15, 0.00];
    if (fm === 4) return [0.15, 0.80, 0.05];
    if (fm === 5) return [0.00, 0.60, 0.40];
    return [0.2627, 0.6780, 0.0593];
  };

  if (mode === 1) { // ChromogenicTypeC (RA-4)
    const rG = applyPaperGrade(rgb[0], grade);
    const gG = applyPaperGrade(rgb[1], grade);
    const bG = applyPaperGrade(rgb[2], grade);

    const compressShoulder = (c: number): number => {
      if (c > 0.70) {
        const u = Math.min(Math.max((c - 0.70) / 0.30, 0.0), 1.0);
        return c - 0.045 * depth * u * u * (3.0 - 2.0 * u);
      }
      return c;
    };
    const rP = compressShoulder(rG);
    const gP = compressShoulder(gG);
    const bP = compressShoulder(bG);

    const lum = 0.2126 * rP + 0.7152 * gP + 0.0722 * bP;
    const satScale = 0.70 + 0.35 * depth;
    let rOut = Math.min(Math.max(lum + (rP - lum) * satScale, 0.0), 1.0);
    let gOut = Math.min(Math.max(lum + (gP - lum) * satScale, 0.0), 1.0);
    let bOut = Math.min(Math.max(lum + (bP - lum) * satScale, 0.0), 1.0);

    const dyeDensity = 0.04 * Math.max(1.0 - lum, 0.0) * depth;
    rOut = Math.min(Math.max(rOut * (1.0 - dyeDensity * 0.4), 0.0), 1.0);
    gOut = Math.min(Math.max(gOut * (1.0 - dyeDensity * 0.6), 0.0), 1.0);
    bOut = Math.min(Math.max(bOut * (1.0 - dyeDensity * 0.9), 0.0), 1.0);

    return applyBaseWarmth([rOut, gOut, bOut], warmth);
  } else if (mode === 2) { // SilverDyeBleach (Azo Reversal)
    const rG = applyPaperGrade(rgb[0], grade);
    const gG = applyPaperGrade(rgb[1], grade);
    const bG = applyPaperGrade(rgb[2], grade);

    const shadowPow = 1.0 + 0.12 * depth;
    const rP = Math.pow(Math.max(rG, 0.0), shadowPow);
    const gP = Math.pow(Math.max(gG, 0.0), shadowPow);
    const bP = Math.pow(Math.max(bG, 0.0), shadowPow);

    const lum = 0.2126 * rP + 0.7152 * gP + 0.0722 * bP;
    const azoSat = 0.80 + 0.30 * depth;
    const rOut = Math.min(Math.max(lum + (rP - lum) * azoSat, 0.0), 1.0);
    const gOut = Math.min(Math.max(lum + (gP - lum) * azoSat, 0.0), 1.0);
    const bOut = Math.min(Math.max(lum + (bP - lum) * azoSat, 0.0), 1.0);

    return applyBaseWarmth([rOut, gOut, bOut], warmth);
  } else if (mode === 3) { // FineArtCottonRag
    const rG = applyPaperGrade(rgb[0], grade);
    const gG = applyPaperGrade(rgb[1], grade);
    const bG = applyPaperGrade(rgb[2], grade);

    const matteFloor = 0.055 - 0.015 * depth;
    const rP = rG + matteFloor * (1.0 - rG) * (1.0 - rG);
    const gP = gG + matteFloor * (1.0 - gG) * (1.0 - gG);
    const bP = bG + matteFloor * (1.0 - bG) * (1.0 - bG);

    const lum = 0.2126 * rP + 0.7152 * gP + 0.0722 * bP;
    const pigmentSat = 0.75 + 0.25 * depth;
    const rOut = Math.min(Math.max(lum + (rP - lum) * pigmentSat, 0.0), 1.0);
    const gOut = Math.min(Math.max(lum + (gP - lum) * pigmentSat, 0.0), 1.0);
    const bOut = Math.min(Math.max(lum + (bP - lum) * pigmentSat, 0.0), 1.0);

    return applyBaseWarmth([rOut, gOut, bOut], warmth + 15.0);
  } else if (mode === 4) { // BarytaSilverGelatin
    const w = getEnlargerFilterWeights(filterMode);
    const rawLum = Math.min(Math.max(rgb[0] * w[0] + rgb[1] * w[1] + rgb[2] * w[2], 0.0), 1.0);
    const silver = applyPaperGrade(rawLum, grade);

    const dmaxGamma = 1.0 + 0.08 * depth;
    const silverToned = Math.pow(Math.max(silver, 0.0), dmaxGamma);

    return applyBaseWarmth([silverToned, silverToned, silverToned], warmth);
  } else if (mode === 5) { // WarmtoneChlorobromide
    const w = getEnlargerFilterWeights(filterMode);
    const rawLum = Math.min(Math.max(rgb[0] * w[0] + rgb[1] * w[1] + rgb[2] * w[2], 0.0), 1.0);
    const silver = applyPaperGrade(rawLum, grade);

    const shadowPow = 1.0 + 0.10 * depth;
    const silverDeep = Math.pow(Math.max(silver, 0.0), shadowPow);

    const midMask = 4.0 * silver * (1.0 - silver);
    const shadowDensity = Math.max(1.0 - silver, 0.0);

    const rTone = silverDeep + depth * (midMask * 0.030 + shadowDensity * 0.020);
    const gTone = silverDeep + depth * (midMask * 0.012 - shadowDensity * 0.015);
    const bTone = silverDeep + depth * (-midMask * 0.018 + shadowDensity * 0.030);

    const toned: [number, number, number] = [
      Math.min(Math.max(rTone, 0.0), 1.0),
      Math.min(Math.max(gTone, 0.0), 1.0),
      Math.min(Math.max(bTone, 0.0), 1.0),
    ];
    return applyBaseWarmth(toned, warmth);
  } else if (mode === 6) { // PlatinumPalladium
    const w = getEnlargerFilterWeights(filterMode);
    const rawLum = Math.min(Math.max(rgb[0] * w[0] + rgb[1] * w[1] + rgb[2] * w[2], 0.0), 1.0);
    const silver = applyPaperGrade(rawLum, grade);

    const matteFloor = 0.035 * depth;
    const silverBase = matteFloor + (1.0 - matteFloor) * silver;

    const warmLift = Math.max(1.0 - Math.pow(Math.max(silverBase, 0.0), 0.6), 0.0) * depth;
    const rTone = silverBase + warmLift * 0.045;
    const gTone = silverBase + warmLift * 0.028;
    const bTone = silverBase - warmLift * 0.020;

    const toned: [number, number, number] = [
      Math.min(Math.max(rTone, 0.0), 1.0),
      Math.min(Math.max(gTone, 0.0), 1.0),
      Math.min(Math.max(bTone, 0.0), 1.0),
    ];
    return applyBaseWarmth(toned, warmth + 10.0 * depth);
  } else if (mode === 7) { // Cyanotype
    const w = getEnlargerFilterWeights(filterMode);
    const rawLum = Math.min(Math.max(rgb[0] * w[0] + rgb[1] * w[1] + rgb[2] * w[2], 0.0), 1.0);
    const silver = applyPaperGrade(rawLum, grade);

    const density = 1.0 - silver;
    const rP = silver * (1.0 - 0.75 * depth * density);
    const gP = silver * (1.0 - 0.35 * depth * density) + 0.02 * depth * density;
    const bP = silver * (1.0 - 0.05 * depth * density) + 0.07 * depth * density;

    const toned: [number, number, number] = [
      Math.min(Math.max(rP, 0.0), 1.0),
      Math.min(Math.max(gP, 0.0), 1.0),
      Math.min(Math.max(bP, 0.0), 1.0),
    ];
    return applyBaseWarmth(toned, warmth + 8.0 * depth);
  }

  return rgb;
}

/**
 * Executes full reference CPU export pipeline on a single pixel.
 */
export function evaluateReferencePixel(
  input: [number, number, number],
  isLinearRec2020: boolean,
  edits: {
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
    tone_profile: string; // 'AgxVivid' | 'AgxFilmic' | etc.
    split_toning_shadow_hue: number;
    split_toning_shadow_sat: number;
    split_toning_highlight_hue: number;
    split_toning_highlight_sat: number;
    split_toning_balance: number;
    sharpening_amount?: number;
    sharpening_radius?: number;
    sharpening_masking?: number;
    noise_reduction?: number;
    emulsion_archetype?: string;
    emulsion_exposure?: number;
    print_media?: string;
    print_paper_grade?: number;
    print_base_warmth?: number;
    print_process_depth?: number;
    enlarger_filter?: string;
  },
  curveLut?: { master?: number[]; red?: number[]; green?: number[]; blue?: number[] },
  neighbors?: { y_c: number; y_l: number; y_r: number; y_t: number; y_b: number }
): [number, number, number] {
  // Neutral JPEG Passthrough
  if (!isLinearRec2020 && isNeutralEdits(edits) && !curveLut?.master) {
    return [
      Math.min(Math.max(Math.round(input[0] * 255), 0), 255),
      Math.min(Math.max(Math.round(input[1] * 255), 0), 255),
      Math.min(Math.max(Math.round(input[2] * 255), 0), 255),
    ];
  }

  // 1. Scene-Referred Working Space
  let rec2020: [number, number, number];
  if (isLinearRec2020) {
    rec2020 = [input[0], input[1], input[2]];
  } else {
    const srgbLin: [number, number, number] = [
      srgbGammaToLinear(input[0]),
      srgbGammaToLinear(input[1]),
      srgbGammaToLinear(input[2]),
    ];
    rec2020 = matMul(LINEAR_SRGB_TO_REC2020, srgbLin);
  }

  // 2. White Balance
  if (Math.abs(edits.temperature - 5500.0) >= 1.0 || edits.tint !== 0.0) {
    const tempNorm = Math.min(Math.max(edits.temperature / 5500.0, 0.25), 4.0);
    let rScale = Math.pow(tempNorm, 0.75);
    let bScale = Math.pow(1.0 / tempNorm, 0.75);
    const tintFactor = edits.tint * 0.002;
    const gScale = Math.max(1.0 - tintFactor, 0.01);
    rScale = Math.max(rScale + tintFactor * 0.5, 0.01);
    bScale = Math.max(bScale + tintFactor * 0.5, 0.01);
    rec2020[0] *= rScale;
    rec2020[1] *= gScale;
    rec2020[2] *= bScale;
  }

  // 3. Exposure Multiplication
  if (edits.exposure !== 0.0) {
    const gain = Math.pow(2.0, edits.exposure);
    rec2020[0] *= gain;
    rec2020[1] *= gain;
    rec2020[2] *= gain;
  }

  // 3b. Tone Recovery
  rec2020 = applyToneRecovery(rec2020, edits.highlights, edits.shadows, edits.whites, edits.blacks);

  // 4. Photochemical Recording Emulsion
  if (edits.emulsion_archetype && edits.emulsion_archetype !== 'None') {
    rec2020 = applyRecordingEmulsion(rec2020, edits.emulsion_archetype, edits.emulsion_exposure ?? 0.0);
  }

  // 5. Tone Curves (Master + R + G + B) in Linear Rec.2020 Working Space
  const sampleLut = (lut: number[], val: number): number => {
    const idx = Math.min(Math.max(Math.round(val * 255), 0), 255);
    return lut[idx];
  };
  if (curveLut?.master && curveLut.master.length === 256) {
    rec2020[0] = sampleLut(curveLut.master, rec2020[0]);
    rec2020[1] = sampleLut(curveLut.master, rec2020[1]);
    rec2020[2] = sampleLut(curveLut.master, rec2020[2]);
  }
  if (curveLut) {
    if (curveLut.red && curveLut.red.length === 256) rec2020[0] = sampleLut(curveLut.red, rec2020[0]);
    if (curveLut.green && curveLut.green.length === 256) rec2020[1] = sampleLut(curveLut.green, rec2020[1]);
    if (curveLut.blue && curveLut.blue.length === 256) rec2020[2] = sampleLut(curveLut.blue, rec2020[2]);
  }

  // 6. Color & Vibrance in Linear Rec.2020
  rec2020 = applyColorAdjustments(rec2020, edits.vibrance, edits.saturation, edits.deep_blue, edits.foliage);

  // 7. Split Toning in Linear Rec.2020
  rec2020 = applySplitToning(
    rec2020,
    edits.split_toning_shadow_hue,
    edits.split_toning_shadow_sat,
    edits.split_toning_highlight_hue,
    edits.split_toning_highlight_sat,
    edits.split_toning_balance
  );

  // 8. Detail, Sharpening & Noise Reduction in Linear Rec.2020
  const sharpAmount = edits.sharpening_amount ?? 0.0;
  const nrAmount = edits.noise_reduction ?? 0.0;
  if (neighbors && (sharpAmount > 0.0 || nrAmount > 0.0)) {
    const y_c = 0.2627 * rec2020[0] + 0.6780 * rec2020[1] + 0.0593 * rec2020[2];
    rec2020 = applyDetailAndSharpeningRgb(
      rec2020[0], rec2020[1], rec2020[2],
      y_c, neighbors.y_l, neighbors.y_r, neighbors.y_t, neighbors.y_b,
      sharpAmount,
      edits.sharpening_masking ?? 0.0,
      nrAmount
    );
  }

  // 8b. Fine-Art Print Media Chemistry in Linear Rec.2020
  if (edits.print_media && edits.print_media !== 'None') {
    const filterMap: Record<string, number> = {
      NeutralPanchromatic: 0, FilterGrade2: 0, YellowK2: 1, Orange16: 2, Red25A: 3, GreenX1: 4, Orthochromatic: 5
    };
    const filterMode = filterMap[edits.enlarger_filter ?? ''] ?? 0;
    rec2020 = applyPrintMedia(
      rec2020,
      edits.print_media,
      edits.print_paper_grade ?? 0.0,
      edits.print_base_warmth ?? 0.0,
      edits.print_process_depth ?? 100.0,
      filterMode
    );
  }

  // 9. TERMINAL STAGE: AgX Filmic Tone Mapping Engine OR Camera Matching Profiles -> Linear sRGB
  const contrastGain = edits.contrast >= 0.0 ? 1.0 + edits.contrast / 100.0 : 1.0 + (edits.contrast / 100.0) * 0.60;
  let srgbLinearOut: [number, number, number];
  const isAgxVivid = edits.tone_profile === 'AgxVivid' || !edits.tone_profile;
  const isAgxFilmic = edits.tone_profile === 'AgxFilmic';

  const CAM_MODE_MAP: Record<string, number> = {
    CameraStandard: 1,
    CameraPortrait: 2,
    CameraLandscape: 3,
    CameraVivid: 4,
  };

  if (isAgxVivid) {
    srgbLinearOut = agxToneMapVivid(rec2020, contrastGain);
  } else if (isAgxFilmic) {
    srgbLinearOut = agxToneMap(rec2020, contrastGain);
  } else if (edits.tone_profile in CAM_MODE_MAP) {
    const camMode = CAM_MODE_MAP[edits.tone_profile];
    const hsv = rgb2hsv(rec2020[0], rec2020[1], rec2020[2]);
    const hsvAdj = applyCameraHueSat(hsv, camMode);
    const rgbAdj = hsv2rgb(hsvAdj[0], hsvAdj[1], hsvAdj[2]);

    const lum = 0.2627 * rgbAdj[0] + 0.6780 * rgbAdj[1] + 0.0593 * rgbAdj[2];
    const lumCurved = applyCameraToneCurve(lum, camMode);
    const lumAdj = Math.min(Math.max((lumCurved - 0.18) * contrastGain + 0.18, 0.0), 1.0);
    const gain = lumAdj / Math.max(lum, 1e-6);

    const curvedRec2020: [number, number, number] = [
      Math.min(Math.max(rgbAdj[0] * gain, 0.0), 1.0),
      Math.min(Math.max(rgbAdj[1] * gain, 0.0), 1.0),
      Math.min(Math.max(rgbAdj[2] * gain, 0.0), 1.0),
    ];
    srgbLinearOut = matMul(REC2020_TO_LINEAR_SRGB, curvedRec2020);
  } else {
    // Fallback
    srgbLinearOut = matMul(REC2020_TO_LINEAR_SRGB, rec2020);
  }

  // 10. Final Display Output: Display Gamma (Terminal Step)
  const outColor: [number, number, number] = [
    linearToSrgbGamma(srgbLinearOut[0]),
    linearToSrgbGamma(srgbLinearOut[1]),
    linearToSrgbGamma(srgbLinearOut[2]),
  ];

  return [
    Math.round(outColor[0] * 255),
    Math.round(outColor[1] * 255),
    Math.round(outColor[2] * 255),
  ];
}
