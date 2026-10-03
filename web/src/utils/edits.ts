import type { EditSettingsPayload } from '../WasmBridge';
import { decodeExifOrientation } from './exifReader';

export function getDefaultEdits(isRaw: boolean = false): EditSettingsPayload {
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
    crop_constrain: true,
    rotation: 0,
    flip_horizontal: false,
    flip_vertical: false,
    tone_profile: 'AgxVivid',
    lut_intensity: 1.0,
    sharpening_amount: isRaw ? 20.0 : 0.0,
    sharpening_radius: 1.0,
    sharpening_masking: isRaw ? 15.0 : 0.0,
    noise_reduction: 0.0,
    emulsion_archetype: 'None',
    emulsion_format: 'Format35mm',
    emulsion_exposure: 0.0,
    emulsion_grain_mode: 'Off',
    grain_scale: 1.0,
    curves: {
      master: [
        { x: 0, y: 0 },
        { x: 1, y: 1 },
      ],
      red: [
        { x: 0, y: 0 },
        { x: 1, y: 1 },
      ],
      green: [
        { x: 0, y: 0 },
        { x: 1, y: 1 },
      ],
      blue: [
        { x: 0, y: 0 },
        { x: 1, y: 1 },
      ],
    },
    split_toning_shadow_hue: 215.0,
    split_toning_shadow_sat: 0.0,
    split_toning_highlight_hue: 45.0,
    split_toning_highlight_sat: 0.0,
    split_toning_balance: 0.0,
    print_media: 'None',
    enlarger_filter: 'NeutralPanchromatic',
    print_paper_grade: 0.0,
    print_base_warmth: 0.0,
    print_process_depth: 100.0,
  };
}

export function hasModifications(
  edits: EditSettingsPayload,
  orientation: number = 1,
  rating: number = 0,
  status: string = 'Unflagged',
  meta?: {
    title?: string;
    description?: string;
    creator?: string;
    keywords?: string;
  },
): boolean {
  if (rating > 0) return true;
  if (status !== 'Unflagged') return true;
  if (
    meta?.title?.trim() ||
    meta?.description?.trim() ||
    meta?.creator?.trim() ||
    meta?.keywords?.trim()
  ) {
    return true;
  }

  if (Math.abs(edits.exposure) > 1e-4) return true;
  if (Math.abs(edits.contrast) > 1e-4) return true;
  if (Math.abs(edits.highlights) > 1e-4) return true;
  if (Math.abs(edits.shadows) > 1e-4) return true;
  if (Math.abs(edits.whites) > 1e-4) return true;
  if (Math.abs(edits.blacks) > 1e-4) return true;
  if (Math.abs(edits.temperature - 5500.0) > 1.0) return true;
  if (Math.abs(edits.tint) > 1e-4) return true;
  if (Math.abs(edits.vibrance) > 1e-4) return true;
  if (Math.abs(edits.saturation) > 1e-4) return true;
  if (Math.abs(edits.deep_blue) > 1e-4) return true;
  if (Math.abs(edits.foliage) > 1e-4) return true;
  if (edits.tone_profile !== 'AgxVivid') return true;
  if (edits.emulsion_archetype !== 'None') return true;
  if (edits.emulsion_grain_mode && edits.emulsion_grain_mode !== 'Off')
    return true;
  if (
    edits.grain_scale !== undefined &&
    Math.abs(edits.grain_scale - 1.0) > 1e-4
  )
    return true;
  if (edits.print_media !== 'None') return true;
  if (Math.abs(edits.split_toning_shadow_sat) > 1e-4) return true;
  if (Math.abs(edits.split_toning_highlight_sat) > 1e-4) return true;
  if (Math.abs(edits.crop_angle ?? 0.0) > 1e-4) return true;
  if (Math.abs((edits.crop_left ?? 0.0) - 0.0) > 1e-4) return true;
  if (Math.abs((edits.crop_top ?? 0.0) - 0.0) > 1e-4) return true;
  if (Math.abs((edits.crop_right ?? 1.0) - 1.0) > 1e-4) return true;
  if (Math.abs((edits.crop_bottom ?? 1.0) - 1.0) > 1e-4) return true;

  const exifOri = decodeExifOrientation(orientation);
  if ((edits.rotation ?? 0) !== exifOri.rotation) return true;

  return false;
}
