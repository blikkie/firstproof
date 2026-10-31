import type {
  EmulsionArchetype,
  EmulsionFormat,
  EmulsionGrainMode,
  EnlargerFilterMode,
  PrintMediaMode,
  RemjetBackingMode,
  ToneCurvesRecord,
} from './WasmBridge';

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
  lut_name?: string;
  lut_intensity: number;
  sharpening_amount: number;
  sharpening_radius: number;
  sharpening_masking: number;
  noise_reduction: number;
  emulsion_grain_mode: EmulsionGrainMode;
  grain_scale: number;
  grain_size: number;
  grain_amount: number;
  emulsion_archetype: EmulsionArchetype;
  emulsion_format: EmulsionFormat;
  emulsion_exposure: number;
  remjet_backing: RemjetBackingMode;
  curves: ToneCurvesRecord;
  split_toning_shadow_hue: number;
  split_toning_shadow_sat: number;
  split_toning_highlight_hue: number;
  split_toning_highlight_sat: number;
  split_toning_balance: number;
  print_media: PrintMediaMode;
  enlarger_filter: EnlargerFilterMode;
  print_paper_grade: number;
  print_base_warmth: number;
  print_process_depth: number;
  defringe_amount: number;
  lens_correction_enabled: boolean;
  tca_correction_enabled: boolean;
  vignetting_correction_enabled: boolean;
}
