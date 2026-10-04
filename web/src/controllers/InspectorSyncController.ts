/**
 * InspectorSyncController
 *
 * Encapsulates the Inspector sidebar controls:
 * - Slider scrubbing state machine (pointerdown, pointermove, shift fine-drag, double-click reset)
 * - Perceptual Mired non-linear temperature mapping
 * - Workspace tabs (Light, Color, Detail, Emulsion, Info)
 * - Tone profile, Emulsion archetype, Print media, Enlarger filter selects
 * - Tone curve channel switching and point synchronization
 * - IPTC metadata text inputs and sidecar persistence
 * - Emulsion workspace conditional visibility
 */

import type { ToneCurveWidget } from '../ToneCurveWidget';
import { formatExposure } from '../utils/formatters';
import type {
  ColorProfileMode,
  EditSettingsPayload,
  EmulsionArchetype,
  PrintMediaMode,
  RemjetBackingMode,
} from '../WasmBridge';

export const CALIBRATED_GRAIN_DEFAULTS: Record<
  string,
  { size: number; amount: number }
> = {
  AshigaraChrome50: { size: 0.55, amount: 0.45 },
  RochesterK14Chrome64: { size: 0.7, amount: 0.6 },
  LiguriaCinema80: { size: 0.75, amount: 0.65 },
  LeverkusenPortrait160: { size: 0.8, amount: 0.7 },
  CambridgeDiffusion100: { size: 0.85, amount: 0.5 },
  AshigaraPortrait400: { size: 0.95, amount: 0.75 },
  RochesterPortrait400: { size: 1.0, amount: 0.8 },
  BaodingSilver100: { size: 1.05, amount: 1.0 },
  RochesterCinema500: { size: 1.25, amount: 1.1 },
  MobberleySilver400: { size: 1.35, amount: 1.2 },
};

export interface ActiveSliderScrub {
  id: string;
  prop?: keyof EditSettingsPayload;
  input: HTMLInputElement;
  label: HTMLElement;
  format: (v: number) => string;
  isTemp?: boolean;
  initialValue: number;
  initialInputValue: string;
  startClientX: number;
  startInputVal: number;
  min: number;
  max: number;
}

export interface InspectorDelegate {
  getEdits: () => EditSettingsPayload;
  renderCanvas: () => void;
  autoSaveSidecar: () => void;
  isCurrentPhotoRaw: () => boolean;
  getDefaultEdits: () => EditSettingsPayload;
  getCurveWidget: () => ToneCurveWidget;
  getRating: () => number;
  getStatus: () => string;
  getActiveIndex: () => number;
  onResetAllEdits?: () => void;
}

export class InspectorSyncController {
  private delegate: InspectorDelegate;
  private activeScrub: ActiveSliderScrub | null = null;
  private isBound: boolean = false;

  private boundPointerUp: () => void;
  private boundPointerCancel: () => void;

  constructor(delegate: InspectorDelegate) {
    this.delegate = delegate;
    this.boundPointerUp = () => this.handleGlobalPointerUp();
    this.boundPointerCancel = () => this.handleGlobalPointerCancel();
  }

  public hasActiveScrub(): boolean {
    return this.activeScrub !== null;
  }

  public rollbackActiveScrub(): void {
    if (!this.activeScrub) return;
    const scrub = this.activeScrub;
    const edits = this.delegate.getEdits();
    scrub.input.value = scrub.initialInputValue;

    if (scrub.isTemp) {
      edits.temperature = scrub.initialValue;
      scrub.label.innerText = `${Math.round(scrub.initialValue)} K`;
    } else if (scrub.prop) {
      (edits[scrub.prop] as number) = scrub.initialValue;
      scrub.label.innerText = scrub.format(scrub.initialValue);
      if ((scrub.prop as string).startsWith('split_toning_')) {
        this.updateSplitToningHint();
      }
    }
    this.delegate.renderCanvas();
    scrub.input.blur();
    this.activeScrub = null;
  }

  public commitActiveScrub(): void {
    if (this.activeScrub) {
      this.delegate.autoSaveSidecar();
      this.activeScrub = null;
    }
  }

  // --- Perceptual Mired Non-Linear Mapping ---
  // Human color perception of blackbody temperature is linear in Mireds (10^6 / K), not linear Kelvin.
  // 2000 K  -> 500 Mireds (pos = 0.0)
  // 5500 K  -> 181.8 Mireds (pos = 0.707)
  // 20000 K -> 50 Mireds (pos = 1.0)
  public kelvinToMiredPos(kelvin: number): number {
    const k = Math.min(Math.max(kelvin, 2000.0), 20000.0);
    const mired = 1_000_000.0 / k;
    const pos = (500.0 - mired) / 450.0;
    return Math.min(Math.max(pos, 0.0), 1.0);
  }

  public miredPosToKelvin(pos: number): number {
    const p = Math.min(Math.max(pos, 0.0), 1.0);
    const mired = 500.0 - p * 450.0;
    const rawK = 1_000_000.0 / mired;
    const steppedK =
      rawK < 10000.0
        ? Math.round(rawK / 50.0) * 50.0
        : Math.round(rawK / 200.0) * 200.0;
    return Math.min(Math.max(steppedK, 2000.0), 20000.0);
  }

  public bind(): void {
    if (this.isBound) return;

    // 1. Standard Sliders
    this.bindStandardSliders();

    // 2. Temperature Slider (Mired)
    this.bindTemperatureSlider();

    // 3. Dropdown Selects
    this.bindSelects();

    // 4. Tone Curve Channel Controls
    this.bindCurveControls();

    // 5. Workspace Tab Buttons
    this.bindTabButtons();

    // 6. Metadata Inputs
    this.bindMetadataInputs();

    // 7. Global pointer release / cancellation
    window.addEventListener('pointerup', this.boundPointerUp);
    window.addEventListener('pointercancel', this.boundPointerCancel);

    this.isBound = true;
  }

  public unbind(): void {
    if (!this.isBound) return;
    window.removeEventListener('pointerup', this.boundPointerUp);
    window.removeEventListener('pointercancel', this.boundPointerCancel);
    this.isBound = false;
  }

  public selectInspectorTab(tab: string): void {
    const tabBtns = document.querySelectorAll('.seg-btn');
    tabBtns.forEach((b) => {
      const isMatch = b.getAttribute('data-tab') === tab;
      b.classList.toggle('active', isMatch);
      b.setAttribute('aria-selected', isMatch ? 'true' : 'false');
    });

    const panels = document.querySelectorAll('.workspace-panel');
    panels.forEach((p) => {
      p.classList.toggle('hidden', p.id !== `tab-${tab}`);
    });
  }

  public updateSplitToningHint(): void {
    const edits = this.delegate.getEdits();
    const hasSplitToning =
      edits.split_toning_shadow_sat > 0 || edits.split_toning_highlight_sat > 0;
    const hintEl = document.getElementById('hint-split-toning');
    if (hintEl) {
      hintEl.innerText = hasSplitToning ? 'Active' : 'Off';
      hintEl.classList.toggle('active', hasSplitToning);
    }
  }

  public updateEmulsionWorkspaceVisibility(): void {
    const edits = this.delegate.getEdits();

    // 1. Push / Pull exposure slider only visible when an emulsion archetype is selected
    const hasEmulsion = edits.emulsion_archetype !== 'None';
    const rowEmulsionExp = document.getElementById('row-emulsion-exp');
    if (rowEmulsionExp) {
      rowEmulsionExp.classList.toggle('hidden', !hasEmulsion);
    }

    // 2. Carbon Rem-Jet backing only visible when RochesterCinema500 is selected
    const hasRemjet = edits.emulsion_archetype === 'RochesterCinema500';
    const rowRemjet = document.getElementById('row-remjet-backing');
    if (rowRemjet) {
      rowRemjet.classList.toggle('hidden', !hasRemjet);
    }

    // 3. Grain sliders only visible when grain resynthesis mode is enabled
    const hasGrain =
      edits.emulsion_grain_mode && edits.emulsion_grain_mode !== 'Off';
    const rowGrainScale = document.getElementById('row-grain-scale');
    if (rowGrainScale) {
      rowGrainScale.classList.toggle('hidden', !hasGrain);
    }
    const rowGrainSize = document.getElementById('row-grain-size');
    if (rowGrainSize) {
      rowGrainSize.classList.toggle('hidden', !hasGrain);
    }
    const rowGrainAmount = document.getElementById('row-grain-amount');
    if (rowGrainAmount) {
      rowGrainAmount.classList.toggle('hidden', !hasGrain);
    }

    // 3. Optical enlarger filter only visible when monochrome media/stock is selected
    const isMonochrome =
      edits.print_media === 'BarytaSilverGelatin' ||
      edits.print_media === 'WarmtoneChlorobromide' ||
      edits.print_media === 'PlatinumPalladium' ||
      edits.print_media === 'Cyanotype' ||
      edits.print_media === 'BamakoSilverContact' ||
      edits.emulsion_archetype === 'MobberleySilver400' ||
      edits.emulsion_archetype === 'LiguriaCinema80' ||
      edits.emulsion_archetype === 'BaodingSilver100';

    const rowFilter = document.getElementById('row-enlarger-filter');
    if (rowFilter) {
      rowFilter.classList.toggle('hidden', !isMonochrome);
    }

    // 3. Print media sliders only visible when print media is selected
    const hasPrintMedia = edits.print_media !== 'None';
    const sectionPrintSliders = document.getElementById(
      'section-print-media-sliders',
    );
    if (sectionPrintSliders) {
      sectionPrintSliders.classList.toggle('hidden', !hasPrintMedia);
    }
  }

  public syncSliderUI(): void {
    const edits = this.delegate.getEdits();

    const setVal = (
      id: string,
      val: number | undefined,
      format: (v: number) => string,
    ) => {
      const input = document.getElementById(`slider-${id}`) as HTMLInputElement;
      const label = document.getElementById(`val-${id}`) as HTMLElement;
      if (input && label && val !== undefined) {
        input.value = `${val}`;
        label.innerText = format(val);
      }
    };

    setVal('exposure', edits.exposure, (v) => formatExposure(v));
    setVal('contrast', edits.contrast, (v) => `${Math.round(v)}`);
    setVal('highlights', edits.highlights, (v) => `${Math.round(v)}`);
    setVal('shadows', edits.shadows, (v) => `${Math.round(v)}`);
    setVal('whites', edits.whites, (v) => `${Math.round(v)}`);
    setVal('blacks', edits.blacks, (v) => `${Math.round(v)}`);

    const tempInput = document.getElementById(
      'slider-temp',
    ) as HTMLInputElement;
    const tempLabel = document.getElementById('val-temp') as HTMLElement;
    if (tempInput && tempLabel && edits.temperature !== undefined) {
      tempInput.value = this.kelvinToMiredPos(edits.temperature).toFixed(4);
      tempLabel.innerText = `${Math.round(edits.temperature)} K`;
    }

    setVal('tint', edits.tint, (v) => `${Math.round(v)}`);
    setVal('vibrance', edits.vibrance, (v) => `${Math.round(v)}`);
    setVal('saturation', edits.saturation, (v) => `${Math.round(v)}`);
    setVal('deep-blue', edits.deep_blue, (v) => `${Math.round(v)}`);
    setVal('foliage', edits.foliage, (v) => `${Math.round(v)}`);
    setVal('emulsion-exp', edits.emulsion_exposure, (v) => formatExposure(v));
    setVal('grain-scale', edits.grain_scale, (v) => `${v.toFixed(2)}x`);
    setVal('grain-size', edits.grain_size, (v) => `${v.toFixed(2)}x`);
    setVal('grain-amount', edits.grain_amount, (v) => `${v.toFixed(2)}x`);
    setVal('paper-grade', edits.print_paper_grade, (v) => `${v.toFixed(1)}`);
    setVal('base-warmth', edits.print_base_warmth, (v) => `${Math.round(v)}`);
    setVal(
      'process-depth',
      edits.print_process_depth,
      (v) => `${Math.round(v)}%`,
    );
    setVal(
      'sharpen-amount',
      edits.sharpening_amount,
      (v) => `${Math.round(v)}`,
    );
    setVal(
      'sharpen-radius',
      edits.sharpening_radius,
      (v) => `${v.toFixed(1)} px`,
    );
    setVal(
      'sharpen-masking',
      edits.sharpening_masking,
      (v) => `${Math.round(v)}`,
    );
    setVal('noise-reduction', edits.noise_reduction, (v) => `${Math.round(v)}`);

    setVal(
      'shadow-hue',
      edits.split_toning_shadow_hue,
      (v) => `${Math.round(v)}°`,
    );
    setVal(
      'shadow-sat',
      edits.split_toning_shadow_sat,
      (v) => `${Math.round(v)}`,
    );
    setVal(
      'highlight-hue',
      edits.split_toning_highlight_hue,
      (v) => `${Math.round(v)}°`,
    );
    setVal(
      'highlight-sat',
      edits.split_toning_highlight_sat,
      (v) => `${Math.round(v)}`,
    );
    setVal(
      'split-balance',
      edits.split_toning_balance,
      (v) => `${Math.round(v)}`,
    );

    this.updateSplitToningHint();

    const selectTone = document.getElementById(
      'select-tone-profile',
    ) as HTMLSelectElement | null;
    if (selectTone && edits.tone_profile) {
      selectTone.value = edits.tone_profile;
    }
    const selectEmulsion = document.getElementById(
      'select-emulsion',
    ) as HTMLSelectElement | null;
    if (selectEmulsion && edits.emulsion_archetype) {
      selectEmulsion.value = edits.emulsion_archetype;
    }
    const selectFormat = document.getElementById(
      'select-emulsion-format',
    ) as HTMLSelectElement | null;
    if (selectFormat && edits.emulsion_format) {
      selectFormat.value = edits.emulsion_format;
    }
    const selectRemjet = document.getElementById(
      'select-remjet-backing',
    ) as HTMLSelectElement | null;
    if (selectRemjet && edits.remjet_backing) {
      selectRemjet.value = edits.remjet_backing;
    }
    const selectGrainMode = document.getElementById(
      'select-emulsion-grain-mode',
    ) as HTMLSelectElement | null;
    if (selectGrainMode && edits.emulsion_grain_mode) {
      selectGrainMode.value = edits.emulsion_grain_mode;
    }
    const selectPrint = document.getElementById(
      'select-print-media',
    ) as HTMLSelectElement | null;
    if (selectPrint && edits.print_media) {
      selectPrint.value = edits.print_media;
    }
    const selectFilter = document.getElementById(
      'select-enlarger-filter',
    ) as HTMLSelectElement | null;
    if (selectFilter) {
      selectFilter.value = edits.enlarger_filter ?? 'NeutralPanchromatic';
    }

    this.updateEmulsionWorkspaceVisibility();

    if (edits.curves) {
      const widget = this.delegate.getCurveWidget();
      if (widget) {
        widget.setPoints(edits.curves);
      }
    }
  }

  public syncRatingFlagUI(
    rating: number,
    status: string,
    activeIdx: number,
  ): void {
    const starBtns = document.querySelectorAll('.star-btn');
    starBtns.forEach((btn, idx) => {
      btn.classList.toggle('active', idx < rating);
    });

    const btnPick = document.getElementById('btn-pick');
    const btnReject = document.getElementById('btn-reject');
    if (btnPick) btnPick.classList.toggle('active', status === 'Pick');
    if (btnReject) btnReject.classList.toggle('active', status === 'Reject');

    if (activeIdx >= 0) {
      const cardEl = document.getElementById(`thumb-card-${activeIdx}`);
      if (cardEl) {
        let ratingBadge = cardEl.querySelector(
          '.thumb-badge-rating',
        ) as HTMLElement;
        if (rating > 0) {
          if (!ratingBadge) {
            ratingBadge = document.createElement('div');
            ratingBadge.className = 'thumb-badge-rating';
            cardEl.appendChild(ratingBadge);
          }
          ratingBadge.innerText = '★'.repeat(rating);
        } else if (ratingBadge) {
          ratingBadge.remove();
        }

        let flagBadge = cardEl.querySelector(
          '.thumb-badge-flag',
        ) as HTMLElement;
        if (status === 'Pick' || status === 'Reject') {
          if (!flagBadge) {
            flagBadge = document.createElement('div');
            flagBadge.className = 'thumb-badge-flag';
            cardEl.appendChild(flagBadge);
          }
          flagBadge.className = `thumb-badge-flag ${status.toLowerCase()}`;
          flagBadge.innerText = status.toUpperCase();
        } else if (flagBadge) {
          flagBadge.remove();
        }
      }
    }
  }

  // --- Private Binding Helpers ---

  private bindStandardSliders(): void {
    const bindSlider = (
      id: string,
      prop: keyof EditSettingsPayload,
      format: (val: number) => string,
      defaultVal: number | (() => number) = 0,
    ) => {
      const input = document.getElementById(`slider-${id}`) as HTMLInputElement;
      const label = document.getElementById(`val-${id}`) as HTMLElement;
      const rowLabel = input?.closest('.slider-row')?.querySelector('label');
      if (input && label) {
        input.addEventListener('pointerdown', (e: PointerEvent) => {
          const edits = this.delegate.getEdits();
          this.activeScrub = {
            id,
            prop,
            input,
            label,
            format,
            isTemp: false,
            initialValue: edits[prop] as number,
            initialInputValue: input.value,
            startClientX: e.clientX,
            startInputVal: parseFloat(input.value),
            min: parseFloat(input.min || '-100'),
            max: parseFloat(input.max || '100'),
          };
        });

        input.addEventListener('pointermove', (e: PointerEvent) => {
          if (
            this.activeScrub &&
            this.activeScrub.input === input &&
            e.buttons === 1 &&
            e.shiftKey
          ) {
            const rect = input.getBoundingClientRect();
            const width = rect.width || 150;
            const deltaX = e.clientX - this.activeScrub.startClientX;
            const range = this.activeScrub.max - this.activeScrub.min;
            const fineVal = Math.min(
              this.activeScrub.max,
              Math.max(
                this.activeScrub.min,
                this.activeScrub.startInputVal + (deltaX / width) * range * 0.1,
              ),
            );
            input.value = fineVal.toString();
            const edits = this.delegate.getEdits();
            (edits[prop] as number) = fineVal;
            label.innerText = format(fineVal);
            if ((prop as string).startsWith('split_toning_')) {
              this.updateSplitToningHint();
            }
            this.delegate.renderCanvas();
          }
        });

        input.addEventListener('input', (e: Event) => {
          if ((e as any).shiftKey) return;
          const val = parseFloat(input.value);
          const edits = this.delegate.getEdits();
          (edits[prop] as number) = val;
          label.innerText = format(val);
          if ((prop as string).startsWith('split_toning_')) {
            this.updateSplitToningHint();
          }
          this.delegate.renderCanvas();
        });

        input.addEventListener('change', () => this.delegate.autoSaveSidecar());

        const resetFn = () => {
          const resolvedDefault =
            typeof defaultVal === 'function' ? defaultVal() : defaultVal;
          input.value = `${resolvedDefault}`;
          const edits = this.delegate.getEdits();
          (edits[prop] as number) = resolvedDefault;
          label.innerText = format(resolvedDefault);
          if ((prop as string).startsWith('split_toning_')) {
            this.updateSplitToningHint();
          }
          this.delegate.renderCanvas();
          this.delegate.autoSaveSidecar();
        };

        input.addEventListener('dblclick', resetFn);
        rowLabel?.addEventListener('dblclick', resetFn);
      }
    };

    bindSlider('exposure', 'exposure', (v) => formatExposure(v), 0.0);
    bindSlider('contrast', 'contrast', (v) => `${Math.round(v)}`, 0.0);
    bindSlider('highlights', 'highlights', (v) => `${Math.round(v)}`, 0.0);
    bindSlider('shadows', 'shadows', (v) => `${Math.round(v)}`, 0.0);
    bindSlider('whites', 'whites', (v) => `${Math.round(v)}`, 0.0);
    bindSlider('blacks', 'blacks', (v) => `${Math.round(v)}`, 0.0);

    bindSlider('tint', 'tint', (v) => `${Math.round(v)}`, 0.0);
    bindSlider('vibrance', 'vibrance', (v) => `${Math.round(v)}`, 0.0);
    bindSlider('saturation', 'saturation', (v) => `${Math.round(v)}`, 0.0);
    bindSlider('deep-blue', 'deep_blue', (v) => `${Math.round(v)}`, 0.0);
    bindSlider('foliage', 'foliage', (v) => `${Math.round(v)}`, 0.0);
    bindSlider(
      'emulsion-exp',
      'emulsion_exposure',
      (v) => formatExposure(v),
      0.0,
    );
    bindSlider('grain-scale', 'grain_scale', (v) => `${v.toFixed(2)}x`, 1.0);
    bindSlider(
      'grain-size',
      'grain_size',
      (v) => `${v.toFixed(2)}x`,
      () => {
        const edits = this.delegate.getEdits();
        const def = edits.emulsion_archetype
          ? CALIBRATED_GRAIN_DEFAULTS[edits.emulsion_archetype]
          : null;
        return def ? def.size : 1.0;
      },
    );
    bindSlider(
      'grain-amount',
      'grain_amount',
      (v) => `${v.toFixed(2)}x`,
      () => {
        const edits = this.delegate.getEdits();
        const def = edits.emulsion_archetype
          ? CALIBRATED_GRAIN_DEFAULTS[edits.emulsion_archetype]
          : null;
        return def ? def.amount : 1.0;
      },
    );
    bindSlider(
      'paper-grade',
      'print_paper_grade',
      (v) => `${v.toFixed(1)}`,
      0.0,
    );
    bindSlider(
      'base-warmth',
      'print_base_warmth',
      (v) => `${Math.round(v)}`,
      0.0,
    );
    bindSlider(
      'process-depth',
      'print_process_depth',
      (v) => `${Math.round(v)}%`,
      100.0,
    );
    bindSlider(
      'sharpen-amount',
      'sharpening_amount',
      (v) => `${Math.round(v)}`,
      () => (this.delegate.isCurrentPhotoRaw() ? 20.0 : 0.0),
    );
    bindSlider(
      'sharpen-radius',
      'sharpening_radius',
      (v) => `${v.toFixed(1)} px`,
      1.0,
    );
    bindSlider(
      'sharpen-masking',
      'sharpening_masking',
      (v) => `${Math.round(v)}`,
      () => (this.delegate.isCurrentPhotoRaw() ? 15.0 : 0.0),
    );
    bindSlider(
      'noise-reduction',
      'noise_reduction',
      (v) => `${Math.round(v)}`,
      0.0,
    );
    bindSlider(
      'shadow-hue',
      'split_toning_shadow_hue',
      (v) => `${Math.round(v)}°`,
      215.0,
    );
    bindSlider(
      'shadow-sat',
      'split_toning_shadow_sat',
      (v) => `${Math.round(v)}`,
      0.0,
    );
    bindSlider(
      'highlight-hue',
      'split_toning_highlight_hue',
      (v) => `${Math.round(v)}°`,
      45.0,
    );
    bindSlider(
      'highlight-sat',
      'split_toning_highlight_sat',
      (v) => `${Math.round(v)}`,
      0.0,
    );
    bindSlider(
      'split-balance',
      'split_toning_balance',
      (v) => `${Math.round(v)}`,
      0.0,
    );
  }

  private bindTemperatureSlider(): void {
    const tempInput = document.getElementById(
      'slider-temp',
    ) as HTMLInputElement;
    const tempLabel = document.getElementById('val-temp') as HTMLElement;
    const tempRowLabel = tempInput
      ?.closest('.slider-row')
      ?.querySelector('label');
    if (tempInput && tempLabel) {
      tempInput.addEventListener('pointerdown', (e: PointerEvent) => {
        const edits = this.delegate.getEdits();
        this.activeScrub = {
          id: 'temp',
          input: tempInput,
          label: tempLabel,
          format: (v) => `${Math.round(v)} K`,
          isTemp: true,
          initialValue: edits.temperature,
          initialInputValue: tempInput.value,
          startClientX: e.clientX,
          startInputVal: parseFloat(tempInput.value),
          min: 0.0,
          max: 1.0,
        };
      });

      tempInput.addEventListener('pointermove', (e: PointerEvent) => {
        if (
          this.activeScrub &&
          this.activeScrub.input === tempInput &&
          e.buttons === 1 &&
          e.shiftKey
        ) {
          const rect = tempInput.getBoundingClientRect();
          const width = rect.width || 150;
          const deltaX = e.clientX - this.activeScrub.startClientX;
          const finePos = Math.min(
            1.0,
            Math.max(
              0.0,
              this.activeScrub.startInputVal + (deltaX / width) * 0.1,
            ),
          );
          tempInput.value = finePos.toFixed(4);
          const kelvin = this.miredPosToKelvin(finePos);
          const edits = this.delegate.getEdits();
          edits.temperature = kelvin;
          tempLabel.innerText = `${Math.round(kelvin)} K`;
          const wbSelect = document.getElementById(
            'select-wb-preset',
          ) as HTMLSelectElement | null;
          if (wbSelect) wbSelect.value = 'Custom';
          this.delegate.renderCanvas();
        }
      });

      tempInput.addEventListener('input', (e: Event) => {
        if ((e as any).shiftKey) return;
        const pos = parseFloat(tempInput.value);
        const kelvin = this.miredPosToKelvin(pos);
        const edits = this.delegate.getEdits();
        edits.temperature = kelvin;
        tempLabel.innerText = `${Math.round(kelvin)} K`;
        const wbSelect = document.getElementById(
          'select-wb-preset',
        ) as HTMLSelectElement | null;
        if (wbSelect) wbSelect.value = 'Custom';
        this.delegate.renderCanvas();
      });

      tempInput.addEventListener('change', () =>
        this.delegate.autoSaveSidecar(),
      );

      const resetTemp = () => {
        const edits = this.delegate.getEdits();
        edits.temperature = 5500.0;
        tempInput.value = this.kelvinToMiredPos(5500.0).toFixed(4);
        tempLabel.innerText = '5500 K';
        const wbSelect = document.getElementById(
          'select-wb-preset',
        ) as HTMLSelectElement | null;
        if (wbSelect) wbSelect.value = 'Daylight';
        this.delegate.renderCanvas();
        this.delegate.autoSaveSidecar();
      };

      tempInput.addEventListener('dblclick', resetTemp);
      tempRowLabel?.addEventListener('dblclick', resetTemp);
    }
  }

  private bindSelects(): void {
    const selectTone = document.getElementById(
      'select-tone-profile',
    ) as HTMLSelectElement | null;
    selectTone?.addEventListener('change', (e) => {
      const edits = this.delegate.getEdits();
      edits.tone_profile = (e.target as HTMLSelectElement)
        .value as ColorProfileMode;
      this.delegate.renderCanvas();
      this.delegate.autoSaveSidecar();
    });

    const selectEmulsion = document.getElementById(
      'select-emulsion',
    ) as HTMLSelectElement | null;
    selectEmulsion?.addEventListener('change', (e) => {
      const edits = this.delegate.getEdits();
      const archetype = (e.target as HTMLSelectElement)
        .value as EmulsionArchetype;
      edits.emulsion_archetype = archetype;

      // Auto-initialize grain_size and grain_amount from calibrated defaults
      const defaults = CALIBRATED_GRAIN_DEFAULTS[archetype];
      if (defaults) {
        edits.grain_size = defaults.size;
        edits.grain_amount = defaults.amount;
        const inputSize = document.getElementById(
          'slider-grain-size',
        ) as HTMLInputElement | null;
        const valSize = document.getElementById('val-grain-size');
        if (inputSize && valSize) {
          inputSize.value = `${defaults.size}`;
          valSize.innerText = `${defaults.size.toFixed(2)}x`;
        }
        const inputAmount = document.getElementById(
          'slider-grain-amount',
        ) as HTMLInputElement | null;
        const valAmount = document.getElementById('val-grain-amount');
        if (inputAmount && valAmount) {
          inputAmount.value = `${defaults.amount}`;
          valAmount.innerText = `${defaults.amount.toFixed(2)}x`;
        }
      }

      this.updateEmulsionWorkspaceVisibility();
      this.delegate.renderCanvas();
      this.delegate.autoSaveSidecar();
    });

    const selectFormat = document.getElementById(
      'select-emulsion-format',
    ) as HTMLSelectElement | null;
    selectFormat?.addEventListener('change', (e) => {
      const edits = this.delegate.getEdits();
      edits.emulsion_format = (e.target as HTMLSelectElement).value as any;
      this.delegate.renderCanvas();
      this.delegate.autoSaveSidecar();
    });

    const selectRemjet = document.getElementById(
      'select-remjet-backing',
    ) as HTMLSelectElement | null;
    selectRemjet?.addEventListener('change', (e) => {
      const edits = this.delegate.getEdits();
      edits.remjet_backing = (e.target as HTMLSelectElement)
        .value as RemjetBackingMode;
      this.delegate.renderCanvas();
      this.delegate.autoSaveSidecar();
    });

    const selectGrainMode = document.getElementById(
      'select-emulsion-grain-mode',
    ) as HTMLSelectElement | null;
    selectGrainMode?.addEventListener('change', (e) => {
      const edits = this.delegate.getEdits();
      const mode = (e.target as HTMLSelectElement).value as any;
      edits.emulsion_grain_mode = mode;
      if (
        mode !== 'Off' &&
        edits.emulsion_archetype &&
        edits.emulsion_archetype !== 'None'
      ) {
        const defaults = CALIBRATED_GRAIN_DEFAULTS[edits.emulsion_archetype];
        if (
          defaults &&
          (edits.grain_size === 1.0 || edits.grain_size === undefined) &&
          (edits.grain_amount === 1.0 || edits.grain_amount === undefined)
        ) {
          edits.grain_size = defaults.size;
          edits.grain_amount = defaults.amount;
          const inputSize = document.getElementById(
            'slider-grain-size',
          ) as HTMLInputElement | null;
          const valSize = document.getElementById('val-grain-size');
          if (inputSize && valSize) {
            inputSize.value = `${defaults.size}`;
            valSize.innerText = `${defaults.size.toFixed(2)}x`;
          }
          const inputAmount = document.getElementById(
            'slider-grain-amount',
          ) as HTMLInputElement | null;
          const valAmount = document.getElementById('val-grain-amount');
          if (inputAmount && valAmount) {
            inputAmount.value = `${defaults.amount}`;
            valAmount.innerText = `${defaults.amount.toFixed(2)}x`;
          }
        }
      }
      this.updateEmulsionWorkspaceVisibility();
      this.delegate.renderCanvas();
      this.delegate.autoSaveSidecar();
    });

    const selectPrint = document.getElementById(
      'select-print-media',
    ) as HTMLSelectElement | null;
    selectPrint?.addEventListener('change', (e) => {
      const edits = this.delegate.getEdits();
      edits.print_media = (e.target as HTMLSelectElement)
        .value as PrintMediaMode;
      this.updateEmulsionWorkspaceVisibility();
      this.delegate.renderCanvas();
      this.delegate.autoSaveSidecar();
    });

    const selectFilter = document.getElementById(
      'select-enlarger-filter',
    ) as HTMLSelectElement | null;
    selectFilter?.addEventListener('change', (e) => {
      const edits = this.delegate.getEdits();
      edits.enlarger_filter = (e.target as HTMLSelectElement).value as any;
      this.delegate.renderCanvas();
      this.delegate.autoSaveSidecar();
    });
  }

  private bindCurveControls(): void {
    const channelBtns = document.querySelectorAll('.channel-btn');
    channelBtns.forEach((btn) => {
      btn.addEventListener('click', (e) => {
        channelBtns.forEach((b) => {
          b.classList.remove('active');
        });
        const target = e.target as HTMLElement;
        target.classList.add('active');
        const channel = target.getAttribute('data-channel') as any;
        const widget = this.delegate.getCurveWidget();
        if (widget) widget.setChannel(channel);
      });
    });

    const btnResetCurve = document.getElementById('btn-reset-curve');
    btnResetCurve?.addEventListener('click', () => {
      const widget = this.delegate.getCurveWidget();
      if (widget) widget.resetCurrentChannel();
    });
  }

  private bindTabButtons(): void {
    const tabBtns = document.querySelectorAll('.seg-btn');
    tabBtns.forEach((btn) => {
      btn.addEventListener('click', () => {
        const tab = btn.getAttribute('data-tab');
        if (tab) this.selectInspectorTab(tab);
      });
    });

    const btnResetAll = document.getElementById('btn-reset-all');
    btnResetAll?.addEventListener('click', () => {
      if (this.delegate.onResetAllEdits) {
        this.delegate.onResetAllEdits();
      }
    });
  }

  private bindMetadataInputs(): void {
    const metaInputs = [
      'meta-input-title',
      'meta-input-caption',
      'meta-input-creator',
      'meta-input-keywords',
    ];
    for (const id of metaInputs) {
      const el = document.getElementById(id);
      if (el) {
        el.addEventListener('change', () => this.delegate.autoSaveSidecar());
      }
    }
  }

  private handleGlobalPointerUp(): void {
    if (this.activeScrub) {
      this.delegate.autoSaveSidecar();
      this.activeScrub = null;
    }
  }

  private handleGlobalPointerCancel(): void {
    if (this.activeScrub) {
      this.rollbackActiveScrub();
    }
  }
}
