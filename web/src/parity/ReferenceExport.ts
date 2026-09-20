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

export function applyCameraToneCurve(x: number, mode: number): number {
  const xc = Math.min(Math.max(x, 0.0), 1.0);
  const mid = mode === 2 ? 0.185 : mode === 3 ? 0.200 : mode === 4 ? 0.210 : 0.190;
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
  const skyWeight = Math.min(Math.max(1.0 - Math.abs(h - 235.0) / 35.0, 0.0), 1.0);
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
    edits.split_toning_highlight_sat === 0.0
  );
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
  const skyWeight = Math.min(Math.max(1.0 - Math.abs(h - 235.0) / 35.0, 0.0), 1.0);

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
  const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;

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

  const lum = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2];
  const yClamp = Math.min(Math.max(lum, 0.0), 1.0);
  const t = Math.min(Math.max(0.5 - (balance / 100.0) * 0.25, 0.1), 0.9);
  let [dr, dg, db] = [0.0, 0.0, 0.0];

  if (shadowSat > 0.0 && yClamp < t) {
    const uS = (t - yClamp) / t;
    const wS = uS * uS * (3.0 - 2.0 * uS) * (shadowSat / 100.0);
    const tintRgb = hsv2rgb(shadowHue, 1.0, 1.0);
    const tLum = 0.2126 * tintRgb[0] + 0.7152 * tintRgb[1] + 0.0722 * tintRgb[2];
    dr += wS * (tintRgb[0] - tLum);
    dg += wS * (tintRgb[1] - tLum);
    db += wS * (tintRgb[2] - tLum);
  }

  if (highlightSat > 0.0 && yClamp > t) {
    const uH = (yClamp - t) / (1.0 - t);
    const wH = uH * uH * (3.0 - 2.0 * uH) * (highlightSat / 100.0);
    const tintRgb = hsv2rgb(highlightHue, 1.0, 1.0);
    const tLum = 0.2126 * tintRgb[0] + 0.7152 * tintRgb[1] + 0.0722 * tintRgb[2];
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
  },
  curveLut?: { master?: number[]; red?: number[]; green?: number[]; blue?: number[] }
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

  // 4. Tone Recovery
  rec2020 = applyToneRecovery(rec2020, edits.highlights, edits.shadows, edits.whites, edits.blacks);

  // 5. AgX Filmic Tone Mapping Engine OR Camera Matching Profiles
  const contrastGain = edits.contrast >= 0.0 ? 1.0 + edits.contrast / 100.0 : 1.0 + (edits.contrast / 100.0) * 0.60;
  let finalColor: [number, number, number];
  const isAgxVivid = edits.tone_profile === 'AgxVivid' || !edits.tone_profile;
  const isAgxFilmic = edits.tone_profile === 'AgxFilmic';

  const CAM_MODE_MAP: Record<string, number> = {
    CameraStandard: 1,
    CameraPortrait: 2,
    CameraLandscape: 3,
    CameraVivid: 4,
  };

  if (isAgxVivid) {
    finalColor = agxToneMapVivid(rec2020, contrastGain);
  } else if (isAgxFilmic) {
    finalColor = agxToneMap(rec2020, contrastGain);
  } else if (edits.tone_profile in CAM_MODE_MAP) {
    const camMode = CAM_MODE_MAP[edits.tone_profile];
    const cSrgb = matMul(REC2020_TO_LINEAR_SRGB, rec2020);
    const clampedSrgb: [number, number, number] = [
      Math.max(cSrgb[0], 0.0),
      Math.max(cSrgb[1], 0.0),
      Math.max(cSrgb[2], 0.0),
    ];
    const hsv = rgb2hsv(clampedSrgb[0], clampedSrgb[1], clampedSrgb[2]);
    const hsvAdj = applyCameraHueSat(hsv, camMode);
    const rgbAdj = hsv2rgb(hsvAdj[0], hsvAdj[1], hsvAdj[2]);

    const lum = 0.2126 * rgbAdj[0] + 0.7152 * rgbAdj[1] + 0.0722 * rgbAdj[2];
    const lumCurved = applyCameraToneCurve(lum, camMode);
    const lumAdj = Math.min(Math.max((lumCurved - 0.18) * contrastGain + 0.18, 0.0), 1.0);
    const gain = lumAdj / Math.max(lum, 1e-6);

    finalColor = [
      Math.min(Math.max(rgbAdj[0] * gain, 0.0), 1.0),
      Math.min(Math.max(rgbAdj[1] * gain, 0.0), 1.0),
      Math.min(Math.max(rgbAdj[2] * gain, 0.0), 1.0),
    ];
  } else {
    // Fallback
    const cSrgb = matMul(REC2020_TO_LINEAR_SRGB, rec2020);
    finalColor = [Math.min(Math.max(cSrgb[0], 0.0), 1.0), Math.min(Math.max(cSrgb[1], 0.0), 1.0), Math.min(Math.max(cSrgb[2], 0.0), 1.0)];
  }

  // 6. Color & Vibrance
  finalColor = applyColorAdjustments(finalColor, edits.vibrance, edits.saturation, edits.deep_blue, edits.foliage);

  // 7. Split Toning
  finalColor = applySplitToning(
    finalColor,
    edits.split_toning_shadow_hue,
    edits.split_toning_shadow_sat,
    edits.split_toning_highlight_hue,
    edits.split_toning_highlight_sat,
    edits.split_toning_balance
  );

  // 8. Curves
  if (curveLut?.master && curveLut.master.length === 256) {
    const sampleLut = (lut: number[], val: number): number => {
      const idx = Math.min(Math.max(Math.round(val * 255), 0), 255);
      return lut[idx];
    };
    finalColor[0] = sampleLut(curveLut.master, finalColor[0]);
    finalColor[1] = sampleLut(curveLut.master, finalColor[1]);
    finalColor[2] = sampleLut(curveLut.master, finalColor[2]);
  }

  // 9. Display Target Transform matching Shaders.metal and encoders.rs:
  // AgX (modes 0 & 1) is already display-referred. Camera Matching profiles output linear sRGB.
  const outColor: [number, number, number] = (isAgxVivid || isAgxFilmic)
    ? [
        Math.min(Math.max(finalColor[0], 0.0), 1.0),
        Math.min(Math.max(finalColor[1], 0.0), 1.0),
        Math.min(Math.max(finalColor[2], 0.0), 1.0),
      ]
    : [
        linearToSrgbGamma(finalColor[0]),
        linearToSrgbGamma(finalColor[1]),
        linearToSrgbGamma(finalColor[2]),
      ];

  return [
    Math.round(outColor[0] * 255),
    Math.round(outColor[1] * 255),
    Math.round(outColor[2] * 255),
  ];
}
