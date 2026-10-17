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

import type { AdjustmentDescriptor } from 'firstproof-wasm';
import {
  convert_kelvin_to_mired_norm,
  convert_mired_norm_to_kelvin,
  get_all_adjustments,
} from 'firstproof-wasm';
import type { ToneCurveWidget } from '../ToneCurveWidget';
import { formatAdjustmentValue } from '../utils/formatters';
import type {
  ColorProfileMode,
  EditSettingsPayload,
  EmulsionArchetype,
  EmulsionFormat,
  EmulsionGrainMode,
  EnlargerFilterMode,
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
  private descriptors = new Map<string, AdjustmentDescriptor>();
  private sliderFormatters = new Map<string, (val: number) => string>();

  private boundPointerUp: () => void;
  private boundPointerCancel: () => void;

  constructor(delegate: InspectorDelegate) {
    this.delegate = delegate;
    this.initDescriptors();
    this.boundPointerUp = () => this.handleGlobalPointerUp();
    this.boundPointerCancel = () => this.handleGlobalPointerCancel();
  }

  public getDescriptor(id: string): AdjustmentDescriptor | undefined {
    return this.descriptors.get(id);
  }

  private initDescriptors(): void {
    try {
      const all = get_all_adjustments() as AdjustmentDescriptor[];
      if (Array.isArray(all)) {
        for (const desc of all) {
          this.descriptors.set(desc.id, desc);
        }
      }
    } catch {
      // Gracefully handles test environments where WASM runtime is absent
    }
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
    return convert_kelvin_to_mired_norm(kelvin);
  }

  public miredPosToKelvin(pos: number): number {
    return convert_mired_norm_to_kelvin(pos);
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

    // 4. Contextual Film Format Gate vs Instant Print Format for Polaroid
    const isInstant = edits.emulsion_archetype === 'CambridgeDiffusion100';
    const labelFormat = document.getElementById('label-emulsion-format');
    const selectFormat = document.getElementById(
      'select-emulsion-format',
    ) as HTMLSelectElement | null;
    if (labelFormat) {
      labelFormat.textContent = isInstant
        ? 'Instant Print Format'
        : 'Film Format Gate';
    }
    if (selectFormat) {
      const isCurrentlyInstant =
        selectFormat.getAttribute('data-mode') === 'instant';
      if (isInstant && !isCurrentlyInstant) {
        selectFormat.setAttribute('data-mode', 'instant');
        selectFormat.innerHTML = `
          <option value="Format35mm" selected>Standard Frame (SX-70 / 600 / Packfilm)</option>
          <option value="Format120">4x5 Large Format Sheet (Type 59)</option>
          <option value="Format4x5">8x10 Studio Sheet (Type 809)</option>
        `;
        if (
          edits.emulsion_format === 'Super8' ||
          edits.emulsion_format === 'Format16mm' ||
          !edits.emulsion_format
        ) {
          edits.emulsion_format = 'Format35mm';
        }
        selectFormat.value = edits.emulsion_format;
      } else if (!isInstant && isCurrentlyInstant) {
        selectFormat.setAttribute('data-mode', 'film');
        selectFormat.innerHTML = `
          <option value="Super8">Super 8 (7.3x Magnification)</option>
          <option value="Format16mm">16mm Cinema (3.4x Magnification)</option>
          <option value="Format35mm" selected>35mm Still Frame (1.0x Baseline)</option>
          <option value="Format120">120 Medium Format / 6x7 (0.48x)</option>
          <option value="Format4x5">4x5 Large Format Sheet (0.28x)</option>
        `;
        selectFormat.value = edits.emulsion_format || 'Format35mm';
      }
    }
  }

  public syncSliderUI(): void {
    const edits = this.delegate.getEdits();

    const setVal = (
      id: string,
      val: number | undefined,
      overrideFormat?: (v: number) => string,
    ) => {
      const input = document.getElementById(
        `slider-${id}`,
      ) as HTMLInputElement | null;
      const label = document.getElementById(`val-${id}`) as HTMLElement | null;
      if (input && label && val !== undefined) {
        input.value = `${val}`;
        const format = overrideFormat || this.sliderFormatters.get(id);
        label.innerText = format ? format(val) : `${val}`;
      }
    };

    setVal('exposure', edits.exposure);
    setVal('contrast', edits.contrast);
    setVal('highlights', edits.highlights);
    setVal('shadows', edits.shadows);
    setVal('whites', edits.whites);
    setVal('blacks', edits.blacks);

    const tempInput = document.getElementById(
      'slider-temp',
    ) as HTMLInputElement | null;
    const tempLabel = document.getElementById('val-temp') as HTMLElement | null;
    if (tempInput && tempLabel && edits.temperature !== undefined) {
      tempInput.value = this.kelvinToMiredPos(edits.temperature).toFixed(4);
      tempLabel.innerText = `${Math.round(edits.temperature)} K`;
    }

    setVal('tint', edits.tint);
    setVal('vibrance', edits.vibrance);
    setVal('saturation', edits.saturation);
    setVal('deep-blue', edits.deep_blue);
    setVal('foliage', edits.foliage);
    setVal('emulsion-exp', edits.emulsion_exposure);
    setVal('grain-scale', edits.grain_scale);
    setVal('grain-size', edits.grain_size);
    setVal('grain-amount', edits.grain_amount);
    setVal('paper-grade', edits.print_paper_grade);
    setVal('process-depth', edits.print_process_depth);
    setVal('sharpen-amount', edits.sharpening_amount);
    setVal('sharpen-radius', edits.sharpening_radius);
    setVal('sharpen-masking', edits.sharpening_masking);
    setVal('noise-reduction', edits.noise_reduction);

    setVal('shadow-hue', edits.split_toning_shadow_hue);
    setVal('shadow-sat', edits.split_toning_shadow_sat);
    setVal('highlight-hue', edits.split_toning_highlight_hue);
    setVal('highlight-sat', edits.split_toning_highlight_sat);
    setVal('split-balance', edits.split_toning_balance);

    this.updateSplitToningHint();

    const selectTone = document.getElementById(
      'select-tone-profile',
    ) as HTMLSelectElement | null;
    if (selectTone && edits.tone_profile) {
      if (edits.tone_profile.startsWith('Camera')) {
        edits.tone_profile = 'AgxVivid';
      }
      selectTone.value = edits.tone_profile;
    }
    const selectEmulsion = document.getElementById(
      'select-emulsion',
    ) as HTMLSelectElement | null;
    if (selectEmulsion && edits.emulsion_archetype) {
      selectEmulsion.value = edits.emulsion_archetype;
    }

    this.updateEmulsionWorkspaceVisibility();

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
      descriptorId?: string,
      customFormat?: (val: number) => string,
      customDefaultVal?: number | (() => number),
    ) => {
      const input = document.getElementById(
        `slider-${id}`,
      ) as HTMLInputElement | null;
      const label = document.getElementById(`val-${id}`) as HTMLElement | null;
      const rowLabel = input?.closest('.slider-row')?.querySelector('label');

      // 1. Resolve canonical schema descriptor
      const desc = descriptorId
        ? this.descriptors.get(descriptorId)
        : undefined;

      // 2. Hydrate input min/max/step dynamically from schema
      if (input && desc) {
        input.min = `${desc.min}`;
        input.max = `${desc.max}`;
        input.step = `${desc.step}`;
      }

      // 3. Resolve default value and formatting dynamically
      const defaultVal =
        customDefaultVal !== undefined
          ? customDefaultVal
          : desc
            ? desc.default
            : 0.0;

      const format =
        customFormat !== undefined
          ? customFormat
          : desc
            ? (val: number) => formatAdjustmentValue(desc, val)
            : (val: number) => `${Math.round(val)}`;

      this.sliderFormatters.set(id, format);

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
          if ('shiftKey' in e && (e as unknown as MouseEvent).shiftKey) return;
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

    bindSlider('exposure', 'exposure', 'exposure');
    bindSlider('contrast', 'contrast', 'contrast');
    bindSlider('highlights', 'highlights', 'highlights');
    bindSlider('shadows', 'shadows', 'shadows');
    bindSlider('whites', 'whites', 'whites');
    bindSlider('blacks', 'blacks', 'blacks');

    bindSlider('tint', 'tint', 'tint');
    bindSlider('vibrance', 'vibrance', 'vibrance');
    bindSlider('saturation', 'saturation', 'saturation');
    bindSlider('deep-blue', 'deep_blue', 'deep_blue');
    bindSlider('foliage', 'foliage', 'foliage');
    bindSlider('emulsion-exp', 'emulsion_exposure', 'emulsion_exp');

    bindSlider('paper-grade', 'print_paper_grade', 'paper_grade');
    bindSlider('process-depth', 'print_process_depth', 'process_depth');

    bindSlider('sharpen-amount', 'sharpening_amount', 'sharpen_amount');
    bindSlider('sharpen-radius', 'sharpening_radius', 'sharpen_radius');
    bindSlider('sharpen-masking', 'sharpening_masking', 'sharpen_masking');
    bindSlider('noise-reduction', 'noise_reduction', 'noise_reduction');

    // Secondary controls without primary schema descriptors:
    bindSlider(
      'grain-scale',
      'grain_scale',
      undefined,
      (v) => `${v.toFixed(2)}x`,
      1.0,
    );
    bindSlider(
      'grain-size',
      'grain_size',
      undefined,
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
      undefined,
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
      'shadow-hue',
      'split_toning_shadow_hue',
      undefined,
      (v) => `${Math.round(v)}°`,
      215.0,
    );
    bindSlider(
      'shadow-sat',
      'split_toning_shadow_sat',
      undefined,
      (v) => `${Math.round(v)}`,
      0.0,
    );
    bindSlider(
      'highlight-hue',
      'split_toning_highlight_hue',
      undefined,
      (v) => `${Math.round(v)}°`,
      45.0,
    );
    bindSlider(
      'highlight-sat',
      'split_toning_highlight_sat',
      undefined,
      (v) => `${Math.round(v)}`,
      0.0,
    );
    bindSlider(
      'split-balance',
      'split_toning_balance',
      undefined,
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
    this.sliderFormatters.set('temp', (v) => `${Math.round(v)} K`);
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
        if ('shiftKey' in e && (e as unknown as MouseEvent).shiftKey) return;
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

      const tempDesc = this.descriptors.get('temperature');
      const defaultKelvin = tempDesc ? tempDesc.default : 5500.0;
      const resetTemp = () => {
        const edits = this.delegate.getEdits();
        edits.temperature = defaultKelvin;
        tempInput.value = this.kelvinToMiredPos(defaultKelvin).toFixed(4);
        tempLabel.innerText = `${Math.round(defaultKelvin)} K`;
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

      if (archetype && archetype !== 'None') {
        // When simulating a film stock, default to all of the film:
        // Automatically engage SeedFromSensor grain if grain mode was Off or uninitialized.
        if (!edits.emulsion_grain_mode || edits.emulsion_grain_mode === 'Off') {
          edits.emulsion_grain_mode = 'SeedFromSensor';
          const selectGrainMode = document.getElementById(
            'select-emulsion-grain-mode',
          ) as HTMLSelectElement | null;
          if (selectGrainMode) {
            selectGrainMode.value = 'SeedFromSensor';
          }
        }

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
      } else {
        // Reverting to None: reset grain back to Off
        edits.emulsion_grain_mode = 'Off';
        const selectGrainMode = document.getElementById(
          'select-emulsion-grain-mode',
        ) as HTMLSelectElement | null;
        if (selectGrainMode) {
          selectGrainMode.value = 'Off';
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
      edits.emulsion_format = (e.target as HTMLSelectElement)
        .value as EmulsionFormat;
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
      const mode = (e.target as HTMLSelectElement).value as EmulsionGrainMode;
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
      edits.enlarger_filter = (e.target as HTMLSelectElement)
        .value as EnlargerFilterMode;
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
        const channel = target.getAttribute('data-channel') as
          | 'master'
          | 'red'
          | 'green'
          | 'blue';
        const widget = this.delegate.getCurveWidget();
        if (widget && channel) widget.setChannel(channel);
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
