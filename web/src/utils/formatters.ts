/**
 * Pure photographic formatters for EXIF floats, shutter speeds, apertures, and sensor resolutions.
 * Zero magic numbers, zero plastic rounding artifacts.
 */

const STANDARD_FSTOPS = [
  1.0, 1.1, 1.2, 1.4, 1.6, 1.8, 2.0, 2.2, 2.5, 2.8, 3.2, 3.5, 4.0, 4.5, 5.0, 5.6,
  6.3, 7.1, 8.0, 9.0, 10, 11, 13, 14, 16, 18, 20, 22, 25, 29, 32, 45, 64
];

/**
 * Format camera aperture floats into standard photographic f-stops (e.g., f/2.8 instead of f/2.799999952316284).
 */
export function formatAperture(val?: number | null): string {
  if (val === undefined || val === null || isNaN(val) || val <= 0) {
    return '—';
  }
  // Find closest standard f-stop within 3% tolerance
  for (const fstop of STANDARD_FSTOPS) {
    if (Math.abs(val - fstop) / fstop < 0.03) {
      return `f/${fstop}`;
    }
  }
  // Fall back to 1 decimal place clean float
  const rounded = Math.round(val * 10) / 10;
  return `f/${rounded}`;
}

/**
 * Format shutter speed into photographic fractions or seconds (e.g. 1/250s, 1/1000s, 2.5s).
 */
export function formatShutterSpeed(shutter?: string | number | null): string {
  if (shutter === undefined || shutter === null || shutter === '') {
    return '—';
  }
  if (typeof shutter === 'string') {
    if (shutter.endsWith('s')) return shutter;
    if (shutter.includes('/')) return `${shutter}s`;
    const num = parseFloat(shutter);
    if (isNaN(num)) return `${shutter}s`;
    shutter = num;
  }
  if (shutter >= 0.4) {
    return `${Math.round(shutter * 10) / 10}s`;
  }
  const fracDenominator = Math.round(1 / shutter);
  return `1/${fracDenominator}s`;
}

/**
 * Format focal length into millimeters (e.g., 50mm, 85mm).
 */
export function formatFocalLength(val?: number | null): string {
  if (val === undefined || val === null || isNaN(val) || val <= 0) {
    return '—';
  }
  return `${Math.round(val)}mm`;
}

/**
 * Format camera ISO rating (e.g., ISO 400).
 */
export function formatISO(val?: number | null): string {
  if (val === undefined || val === null || isNaN(val) || val <= 0) {
    return '—';
  }
  return `${Math.round(val)}`;
}

/**
 * Format exposure EV value into signed string (+1.25 EV, -0.50 EV).
 */
export function formatExposure(ev: number): string {
  const sign = ev > 0 ? '+' : '';
  return `${sign}${ev.toFixed(2)} EV`;
}

/**
 * Format resolution dimensions prioritizing full raw sensor resolution (raw_width × raw_height)
 * over small embedded JPEG preview slice dimensions (e.g. 5184 × 3456 instead of 1024 × 683).
 */
export function formatDimensions(
  previewW: number,
  previewH: number,
  rawW?: number | null,
  rawH?: number | null
): string {
  const w = (rawW && rawW >= previewW) ? rawW : previewW;
  const h = (rawH && rawH >= previewH) ? rawH : previewH;
  return `${w} × ${h}`;
}
