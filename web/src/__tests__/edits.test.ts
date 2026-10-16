import { describe, expect, it } from 'vitest';
import { getDefaultEdits, hasModifications } from '../utils/edits';
import {
  formatAdjustmentValue,
  formatAperture,
  formatFocalLength,
  formatISO,
  formatShutterSpeed,
} from '../utils/formatters';

describe('Edit Settings & Modification Invariants', () => {
  describe('getDefaultEdits', () => {
    it('initializes clean defaults without tone modifications', () => {
      const edits = getDefaultEdits(false);

      expect(edits.exposure).toBe(0.0);
      expect(edits.contrast).toBe(0.0);
      expect(edits.temperature).toBe(5500.0);
      expect(edits.tint).toBe(0.0);
      expect(edits.tone_profile).toBe('CameraStandard');
      expect(edits.emulsion_archetype).toBe('None');
      expect(edits.print_media).toBe('None');
      expect(edits.crop_left).toBe(0.0);
      expect(edits.crop_top).toBe(0.0);
      expect(edits.crop_right).toBe(1.0);
      expect(edits.crop_bottom).toBe(1.0);
    });

    it('initializes zero sharpening for pristine capture across RAW and JPEG', () => {
      const rawEdits = getDefaultEdits(true);
      const jpegEdits = getDefaultEdits(false);

      expect(rawEdits.sharpening_amount).toBe(0.0);
      expect(rawEdits.sharpening_masking).toBe(0.0);
      expect(jpegEdits.sharpening_amount).toBe(0.0);
      expect(jpegEdits.sharpening_masking).toBe(0.0);
    });
  });

  describe('hasModifications', () => {
    it('returns false for pristine default edits', () => {
      const edits = getDefaultEdits(false);
      expect(hasModifications(edits, 1, 0, 'Unflagged')).toBe(false);
    });

    it('detects star ratings and flags as modifications', () => {
      const edits = getDefaultEdits(false);

      expect(hasModifications(edits, 1, 1, 'Unflagged')).toBe(true);
      expect(hasModifications(edits, 1, 0, 'Pick')).toBe(true);
      expect(hasModifications(edits, 1, 0, 'Reject')).toBe(true);
    });

    it('detects slider adjustments as modifications', () => {
      const edits = getDefaultEdits(false);

      expect(
        hasModifications({ ...edits, exposure: 0.33 }, 1, 0, 'Unflagged'),
      ).toBe(true);
      expect(
        hasModifications({ ...edits, contrast: 15 }, 1, 0, 'Unflagged'),
      ).toBe(true);
      expect(
        hasModifications({ ...edits, highlights: -20 }, 1, 0, 'Unflagged'),
      ).toBe(true);
      expect(
        hasModifications({ ...edits, shadows: 25 }, 1, 0, 'Unflagged'),
      ).toBe(true);
      expect(
        hasModifications({ ...edits, temperature: 6200 }, 1, 0, 'Unflagged'),
      ).toBe(true);
      expect(hasModifications({ ...edits, tint: 8 }, 1, 0, 'Unflagged')).toBe(
        true,
      );
    });

    it('detects analog emulsion, grain, and print media selection as modifications', () => {
      const edits = getDefaultEdits(false);

      expect(
        hasModifications(
          { ...edits, emulsion_archetype: 'RochesterPortrait400' },
          1,
          0,
          'Unflagged',
        ),
      ).toBe(true);
      expect(
        hasModifications(
          { ...edits, emulsion_grain_mode: 'SynthesizePhysical' },
          1,
          0,
          'Unflagged',
        ),
      ).toBe(true);
      expect(
        hasModifications({ ...edits, grain_scale: 1.5 }, 1, 0, 'Unflagged'),
      ).toBe(true);
      expect(
        hasModifications(
          { ...edits, print_media: 'PlatinumPalladium' },
          1,
          0,
          'Unflagged',
        ),
      ).toBe(true);
      expect(
        hasModifications(
          { ...edits, print_media: 'BamakoSilverContact' },
          1,
          0,
          'Unflagged',
        ),
      ).toBe(true);
    });

    it('detects crop and orientation adjustments as modifications', () => {
      const edits = getDefaultEdits(false);

      expect(
        hasModifications({ ...edits, crop_left: 0.1 }, 1, 0, 'Unflagged'),
      ).toBe(true);
      expect(
        hasModifications({ ...edits, crop_angle: 1.5 }, 1, 0, 'Unflagged'),
      ).toBe(true);
      expect(
        hasModifications({ ...edits, rotation: 90 }, 1, 0, 'Unflagged'),
      ).toBe(true);
    });

    it('detects user metadata input as modifications', () => {
      const edits = getDefaultEdits(false);

      expect(
        hasModifications(edits, 1, 0, 'Unflagged', {
          title: 'Alpine Vista',
        }),
      ).toBe(true);
      expect(
        hasModifications(edits, 1, 0, 'Unflagged', { creator: 'Remco' }),
      ).toBe(true);
    });
  });

  describe('Sensitometric Telemetry Formatters', () => {
    it('formats aperture values with clean f-stop notation', () => {
      expect(formatAperture(2.8)).toBe('f/2.8');
      expect(formatAperture(1.4)).toBe('f/1.4');
      expect(formatAperture(8.0)).toBe('f/8');
      expect(formatAperture(undefined)).toBe('—');
      expect(formatAperture(0)).toBe('—');
    });

    it('formats shutter speeds preserving fractions and decimal seconds', () => {
      expect(formatShutterSpeed('1/250')).toBe('1/250s');
      expect(formatShutterSpeed('1/1000')).toBe('1/1000s');
      expect(formatShutterSpeed(0.5)).toBe('0.5s');
      expect(formatShutterSpeed(2)).toBe('2s');
      expect(formatShutterSpeed(undefined)).toBe('—');
    });

    it('formats focal lengths with millimetre notation', () => {
      expect(formatFocalLength(50)).toBe('50mm');
      expect(formatFocalLength(28)).toBe('28mm');
      expect(formatFocalLength(undefined)).toBe('—');
    });

    it('formats ISO with clean integer representation', () => {
      expect(formatISO(100)).toBe('100');
      expect(formatISO(1600)).toBe('1600');
      expect(formatISO(undefined)).toBe('—');
    });

    it('formats canonical AdjustmentDescriptors with photographic unit precision', () => {
      expect(
        formatAdjustmentValue(
          {
            id: 'exposure',
            name: 'Exposure',
            category: 'Light',
            min: -3,
            max: 3,
            step: 0.05,
            default: 0,
            scale_mode: 'Linear',
            unit: 'EV',
            precision: 2,
          },
          1.25,
        ),
      ).toBe('+1.25 EV');

      expect(
        formatAdjustmentValue(
          {
            id: 'exposure',
            name: 'Exposure',
            category: 'Light',
            min: -3,
            max: 3,
            step: 0.05,
            default: 0,
            scale_mode: 'Linear',
            unit: 'EV',
            precision: 2,
          },
          -0.5,
        ),
      ).toBe('-0.50 EV');

      expect(
        formatAdjustmentValue(
          {
            id: 'exposure',
            name: 'Exposure',
            category: 'Light',
            min: -3,
            max: 3,
            step: 0.05,
            default: 0,
            scale_mode: 'Linear',
            unit: 'EV',
            precision: 2,
          },
          0,
        ),
      ).toBe('0.00 EV');

      expect(
        formatAdjustmentValue(
          {
            id: 'sharpen_radius',
            name: 'Radius',
            category: 'Detail',
            min: 0.5,
            max: 3,
            step: 0.1,
            default: 1,
            scale_mode: 'Linear',
            unit: 'px',
            precision: 1,
          },
          1.5,
        ),
      ).toBe('1.5 px');

      expect(
        formatAdjustmentValue(
          {
            id: 'process_depth',
            name: 'Process Depth',
            category: 'Emulsion',
            min: 0,
            max: 100,
            step: 1,
            default: 100,
            scale_mode: 'Linear',
            unit: '%',
            precision: 0,
          },
          100,
        ),
      ).toBe('100%');

      expect(
        formatAdjustmentValue(
          {
            id: 'temperature',
            name: 'Warmth (Temp)',
            category: 'Color',
            min: 2000,
            max: 20000,
            step: 50,
            default: 5500,
            scale_mode: 'Mired',
            unit: 'K',
            precision: 0,
          },
          5500,
        ),
      ).toBe('5500 K');
    });
  });
});
