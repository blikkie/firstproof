import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ComparisonSliderController } from '../controllers/ComparisonSliderController';
import {
  type InspectorDelegate,
  InspectorSyncController,
} from '../controllers/InspectorSyncController';
import {
  isTextInputActive,
  type KeyboardActions,
  KeyboardShortcutController,
} from '../controllers/KeyboardShortcutController';
import type { ToneCurveWidget } from '../ToneCurveWidget';
import { getDefaultEdits } from '../utils/edits';
import type { EditSettingsPayload } from '../WasmBridge';

describe('Controller & First-Responder Architecture', () => {
  beforeEach(() => {
    document.body.innerHTML = '';
  });

  afterEach(() => {
    document.body.innerHTML = '';
  });

  describe('isTextInputActive First-Responder Detection', () => {
    it('identifies standard text input fields as active', () => {
      const textInput = document.createElement('input');
      textInput.type = 'text';
      document.body.appendChild(textInput);
      textInput.focus();

      expect(isTextInputActive()).toBe(true);
    });

    it('identifies search and email inputs as active text fields', () => {
      const searchInput = document.createElement('input');
      searchInput.type = 'search';
      document.body.appendChild(searchInput);
      searchInput.focus();

      expect(isTextInputActive()).toBe(true);
    });

    it('does NOT shield range slider inputs (sliders should allow keyboard navigation)', () => {
      const rangeInput = document.createElement('input');
      rangeInput.type = 'range';
      document.body.appendChild(rangeInput);
      rangeInput.focus();

      expect(isTextInputActive()).toBe(false);
    });

    it('identifies textarea and contenteditable elements as active', () => {
      const textarea = document.createElement('textarea');
      document.body.appendChild(textarea);
      textarea.focus();

      expect(isTextInputActive()).toBe(true);

      textarea.blur();
      const div = document.createElement('div');
      div.contentEditable = 'true';
      document.body.appendChild(div);
      div.focus();

      expect(isTextInputActive()).toBe(true);
    });

    it('does NOT shield regular buttons or canvas', () => {
      const button = document.createElement('button');
      document.body.appendChild(button);
      button.focus();

      expect(isTextInputActive()).toBe(false);

      const canvas = document.createElement('canvas');
      document.body.appendChild(canvas);
      canvas.focus();

      expect(isTextInputActive()).toBe(false);
    });
  });

  describe('KeyboardShortcutController Bind Lifecycle', () => {
    it('unbind removes all listeners and stops intercepting events', () => {
      const actions: KeyboardActions = {
        onEscape: vi.fn(),
        onSelectTab: vi.fn(),
        onResetAllEdits: vi.fn(),
        onApplyAutoTone: vi.fn(),
        onOpenExportModal: vi.fn(),
        onToggleMetadata: vi.fn(),
        onSetRating: vi.fn(),
        onTogglePick: vi.fn(),
        onToggleReject: vi.fn(),
        onUnflag: vi.fn(),
        onEnter: vi.fn(),
        onToggleCrop: vi.fn(),
        onCropFlipOrientation: vi.fn(),
        onTogglePixelPeep: vi.fn(),
        onRotate: vi.fn(),
        onToggleEyedropper: vi.fn(),
        onPeekOriginal: vi.fn(),
        onNavigate: vi.fn(),
        isCropActive: vi.fn().mockReturnValue(false),
      };

      const controller = new KeyboardShortcutController(actions);
      controller.bind();

      window.dispatchEvent(new KeyboardEvent('keydown', { key: '5' }));
      expect(actions.onSetRating).toHaveBeenCalledWith(5);

      // Unbind and fire again
      controller.unbind();
      window.dispatchEvent(new KeyboardEvent('keydown', { key: '4' }));
      expect(actions.onSetRating).not.toHaveBeenCalledWith(4);
    });
  });

  describe('InspectorSyncController Format Gate Contextualization', () => {
    it('contextualizes format dropdown options and labels for Polaroid vs standard film', () => {
      // Set up DOM elements
      document.body.innerHTML = `
        <div id="row-remjet"></div>
        <div id="row-enlarger-filter"></div>
        <div id="section-print-media-sliders"></div>
        <label id="label-emulsion-format">Film Format Gate</label>
        <select id="select-emulsion-format" class="select-input">
          <option value="Super8">Super 8 (7.3x Magnification)</option>
          <option value="Format16mm">16mm Cinema (3.4x Magnification)</option>
          <option value="Format35mm" selected>35mm Still Frame (1.0x Baseline)</option>
          <option value="Format120">120 Medium Format / 6x7 (0.48x)</option>
          <option value="Format4x5">4x5 Large Format Sheet (0.28x)</option>
        </select>
      `;

      const edits: EditSettingsPayload = {
        ...getDefaultEdits(),
        emulsion_archetype: 'CambridgeDiffusion100',
        emulsion_format: 'Super8',
        print_media: 'None',
      };

      const mockDelegate: InspectorDelegate = {
        getEdits: () => edits,
        renderCanvas: vi.fn(),
        autoSaveSidecar: vi.fn(),
        isCurrentPhotoRaw: () => true,
        getDefaultEdits: () => ({ ...edits }),
        getCurveWidget: () => null as unknown as ToneCurveWidget,
        getRating: () => 0,
        getStatus: () => 'None',
        getActiveIndex: () => 0,
      };

      const controller = new InspectorSyncController(mockDelegate);
      controller.updateEmulsionWorkspaceVisibility();

      const label = document.getElementById('label-emulsion-format')!;
      const select = document.getElementById(
        'select-emulsion-format',
      ) as HTMLSelectElement;

      // 1. Polaroid label
      expect(label.textContent).toBe('Instant Print Format');

      // 2. Only 3 instant format options
      const options = Array.from(select.options).map((o) => o.value);
      expect(options).toEqual(['Format35mm', 'Format120', 'Format4x5']);

      // 3. Super8 coerced to Format35mm
      expect(edits.emulsion_format).toBe('Format35mm');
      expect(select.value).toBe('Format35mm');

      // 4. Switching back to standard film restores all 5 options and Film Format Gate label
      edits.emulsion_archetype = 'RochesterPortrait400';
      controller.updateEmulsionWorkspaceVisibility();

      expect(label.textContent).toBe('Film Format Gate');
      const filmOptions = Array.from(select.options).map((o) => o.value);
      expect(filmOptions).toEqual([
        'Super8',
        'Format16mm',
        'Format35mm',
        'Format120',
        'Format4x5',
      ]);
      expect(select.value).toBe('Format35mm');
    });

    it('auto-engages SeedFromSensor grain when selecting a film archetype and resets to Off on None', () => {
      document.body.innerHTML = `
        <div id="row-remjet"></div>
        <div id="row-enlarger-filter"></div>
        <div id="section-print-media-sliders"></div>
        <label id="label-emulsion-format">Film Format Gate</label>
        <select id="select-emulsion-format" class="select-input">
          <option value="Format35mm" selected>35mm</option>
        </select>
        <select id="select-emulsion" class="select-input">
          <option value="None" selected>None</option>
          <option value="RochesterPortrait400">Rochester Portrait 400</option>
        </select>
        <select id="select-emulsion-grain-mode" class="select-input">
          <option value="Off" selected>Off</option>
          <option value="SeedFromSensor">SeedFromSensor</option>
        </select>
        <input type="range" id="slider-grain-size" value="1.0" />
        <span id="val-grain-size">1.00x</span>
        <input type="range" id="slider-grain-amount" value="1.0" />
        <span id="val-grain-amount">1.00x</span>
      `;

      const edits = {
        ...getDefaultEdits(),
        emulsion_archetype: 'None' as const,
        emulsion_grain_mode: 'Off' as const,
        grain_size: 1.0,
        grain_amount: 1.0,
        print_media: 'None' as const,
      };

      const mockDelegate: InspectorDelegate = {
        getEdits: () => edits,
        renderCanvas: vi.fn(),
        autoSaveSidecar: vi.fn(),
        isCurrentPhotoRaw: () => true,
        getDefaultEdits: () => ({ ...edits }),
        getCurveWidget: () => null as unknown as ToneCurveWidget,
        getRating: () => 0,
        getStatus: () => 'None',
        getActiveIndex: () => 0,
      };

      const controller = new InspectorSyncController(mockDelegate);
      controller.bind();
      controller.updateEmulsionWorkspaceVisibility();

      const selectEmulsion = document.getElementById(
        'select-emulsion',
      ) as HTMLSelectElement;
      const selectGrainMode = document.getElementById(
        'select-emulsion-grain-mode',
      ) as HTMLSelectElement;

      // 1. Initial baseline: clean sensor, grain Off
      expect(edits.emulsion_archetype).toBe('None');
      expect(edits.emulsion_grain_mode).toBe('Off');

      // 2. Select Rochester Portrait 400: "when we simulate a film, default to all of the film"
      selectEmulsion.value = 'RochesterPortrait400';
      selectEmulsion.dispatchEvent(new Event('change'));

      expect(edits.emulsion_archetype).toBe('RochesterPortrait400');
      expect(edits.emulsion_grain_mode).toBe('SeedFromSensor');
      expect(selectGrainMode.value).toBe('SeedFromSensor');
      expect(edits.grain_size).toBe(1.0);
      expect(edits.grain_amount).toBe(0.8);

      // 3. Reverting back to None resets grain mode to Off
      selectEmulsion.value = 'None';
      selectEmulsion.dispatchEvent(new Event('change'));

      expect(edits.emulsion_archetype).toBe('None');
      expect(edits.emulsion_grain_mode).toBe('Off');
      expect(selectGrainMode.value).toBe('Off');
    });
  });

  describe('InspectorSyncController Canonical Schema Hydration', () => {
    it('hydrates slider min, max, step from AdjustmentDescriptor and formats readouts', () => {
      document.body.innerHTML = `
        <div class="slider-row">
          <label>Exposure <span id="val-exposure" class="slider-val">0.00 EV</span></label>
          <input type="range" id="slider-exposure" min="-10" max="10" step="1" value="0.0" />
        </div>
        <div class="slider-row">
          <label>Sharpening Radius <span id="val-sharpen-radius" class="slider-val">1.0 px</span></label>
          <input type="range" id="slider-sharpen-radius" min="0" max="10" step="1" value="1.0" />
        </div>
        <div class="slider-row">
          <label>Process Depth <span id="val-process-depth" class="slider-val">100%</span></label>
          <input type="range" id="slider-process-depth" min="0" max="10" step="10" value="100" />
        </div>
      `;

      const edits = {
        ...getDefaultEdits(),
        exposure: 0.75,
        sharpening_radius: 1.8,
        print_process_depth: 85,
      };

      const mockDelegate: InspectorDelegate = {
        getEdits: () => edits,
        renderCanvas: vi.fn(),
        autoSaveSidecar: vi.fn(),
        isCurrentPhotoRaw: () => true,
        getDefaultEdits: () => ({ ...edits }),
        getCurveWidget: () => null as unknown as ToneCurveWidget,
        getRating: () => 0,
        getStatus: () => 'None',
        getActiveIndex: () => 0,
      };

      const controller = new InspectorSyncController(mockDelegate);

      // Inject mock schema descriptors simulating WASM initialization
      (
        controller as unknown as {
          descriptors: Map<string, unknown>;
        }
      ).descriptors.set('exposure', {
        id: 'exposure',
        name: 'Exposure',
        category: 'Light',
        min: -3.0,
        max: 3.0,
        step: 0.05,
        default: 0.0,
        scale_mode: 'Linear',
        unit: 'EV',
        precision: 2,
      });
      (
        controller as unknown as {
          descriptors: Map<string, unknown>;
        }
      ).descriptors.set('sharpen_radius', {
        id: 'sharpen_radius',
        name: 'Radius',
        category: 'Detail',
        min: 0.5,
        max: 3.0,
        step: 0.1,
        default: 1.0,
        scale_mode: 'Linear',
        unit: 'px',
        precision: 1,
      });
      (
        controller as unknown as {
          descriptors: Map<string, unknown>;
        }
      ).descriptors.set('process_depth', {
        id: 'process_depth',
        name: 'Process Depth',
        category: 'Emulsion',
        min: 0.0,
        max: 100.0,
        step: 1.0,
        default: 100.0,
        scale_mode: 'Linear',
        unit: '%',
        precision: 0,
      });

      controller.bind();

      const expInput = document.getElementById(
        'slider-exposure',
      ) as HTMLInputElement;
      const radInput = document.getElementById(
        'slider-sharpen-radius',
      ) as HTMLInputElement;
      const depthInput = document.getElementById(
        'slider-process-depth',
      ) as HTMLInputElement;

      // 1. Verify input attributes are dynamically hydrated from schema
      expect(expInput.min).toBe('-3');
      expect(expInput.max).toBe('3');
      expect(expInput.step).toBe('0.05');

      expect(radInput.min).toBe('0.5');
      expect(radInput.max).toBe('3');
      expect(radInput.step).toBe('0.1');

      expect(depthInput.min).toBe('0');
      expect(depthInput.max).toBe('100');
      expect(depthInput.step).toBe('1');

      // 2. Sync UI and verify formatting
      controller.syncSliderUI();
      const expVal = document.getElementById('val-exposure')!;
      const radVal = document.getElementById('val-sharpen-radius')!;
      const depthVal = document.getElementById('val-process-depth')!;

      expect(expVal.textContent).toBe('+0.75 EV');
      expect(radVal.textContent).toBe('1.8 px');
      expect(depthVal.textContent).toBe('85%');

      // 3. Double-click reset restores schema default
      expInput.dispatchEvent(new MouseEvent('dblclick'));
      expect(edits.exposure).toBe(0.0);
      expect(expVal.textContent).toBe('0.00 EV');

      radInput.dispatchEvent(new MouseEvent('dblclick'));
      expect(edits.sharpening_radius).toBe(1.0);
      expect(radVal.textContent).toBe('1.0 px');

      depthInput.dispatchEvent(new MouseEvent('dblclick'));
      expect(edits.print_process_depth).toBe(100.0);
      expect(depthVal.textContent).toBe('100%');
    });
  });

  describe('ComparisonSliderController Mobile & Touch Architecture', () => {
    let container: HTMLDivElement;
    let layerBefore: HTMLImageElement;
    let btnJpeg: HTMLButtonElement;
    let btnSplit: HTMLButtonElement;
    let btnFirstProof: HTMLButtonElement;
    let controller: ComparisonSliderController;

    beforeEach(() => {
      document.body.innerHTML = `
        <div id="optical-comparison-viewer" class="comparison-viewer">
          <img class="comparison-layer comparison-layer-after" />
          <img class="comparison-layer comparison-layer-before" />
          <div id="comparison-divider" class="comparison-divider"></div>
        </div>
        <div class="comparison-segmented">
          <button id="btn-compare-jpeg" class="comparison-btn" type="button">In-Camera JPEG</button>
          <button id="btn-compare-split" class="comparison-btn active" type="button">Split (50 / 50)</button>
          <button id="btn-compare-firstproof" class="comparison-btn" type="button">First Proof</button>
        </div>
      `;

      container = document.getElementById(
        'optical-comparison-viewer',
      ) as HTMLDivElement;
      layerBefore = container.querySelector(
        '.comparison-layer-before',
      ) as HTMLImageElement;
      btnJpeg = document.getElementById(
        'btn-compare-jpeg',
      ) as HTMLButtonElement;
      btnSplit = document.getElementById(
        'btn-compare-split',
      ) as HTMLButtonElement;
      btnFirstProof = document.getElementById(
        'btn-compare-firstproof',
      ) as HTMLButtonElement;

      // Mock bounding box for container (1000px wide, left 100)
      vi.spyOn(container, 'getBoundingClientRect').mockReturnValue({
        left: 100,
        top: 0,
        right: 1100,
        bottom: 600,
        width: 1000,
        height: 600,
        x: 100,
        y: 0,
        toJSON: () => {},
      });

      controller = new ComparisonSliderController();
    });

    afterEach(() => {
      controller.unbind();
    });

    it('initializes to 50% split position with CSS variable and direct inline clipPath', () => {
      expect(controller.getSplitPosition()).toBe(50);
      expect(container.style.getPropertyValue('--split-pos')).toBe('50%');
      expect(layerBefore.style.clipPath).toBe(
        'polygon(0 0, 50% 0, 50% 100%, 0 100%)',
      );
      expect(btnSplit.classList.contains('active')).toBe(true);
      expect(btnJpeg.classList.contains('active')).toBe(false);
      expect(btnFirstProof.classList.contains('active')).toBe(false);
    });

    it('updates position and active states when clicking segmented buttons', () => {
      // In-Camera JPEG -> 100%
      btnJpeg.click();
      expect(controller.getSplitPosition()).toBe(100);
      expect(container.style.getPropertyValue('--split-pos')).toBe('100%');
      expect(layerBefore.style.clipPath).toBe(
        'polygon(0 0, 100% 0, 100% 100%, 0 100%)',
      );
      expect(btnJpeg.classList.contains('active')).toBe(true);
      expect(btnSplit.classList.contains('active')).toBe(false);
      expect(btnFirstProof.classList.contains('active')).toBe(false);

      // First Proof -> 0%
      btnFirstProof.click();
      expect(controller.getSplitPosition()).toBe(0);
      expect(container.style.getPropertyValue('--split-pos')).toBe('0%');
      expect(layerBefore.style.clipPath).toBe(
        'polygon(0 0, 0% 0, 0% 100%, 0 100%)',
      );
      expect(btnFirstProof.classList.contains('active')).toBe(true);
      expect(btnJpeg.classList.contains('active')).toBe(false);
      expect(btnSplit.classList.contains('active')).toBe(false);

      // Split -> 50%
      btnSplit.click();
      expect(controller.getSplitPosition()).toBe(50);
      expect(btnSplit.classList.contains('active')).toBe(true);
    });

    it('tracks pointer drag and calls preventDefault to shield mobile Chrome gestures', () => {
      const setPointerCaptureSpy = vi.fn();
      const releasePointerCaptureSpy = vi.fn();
      container.setPointerCapture = setPointerCaptureSpy;
      container.releasePointerCapture = releasePointerCaptureSpy;

      // Pointer down at clientX = 400 (rect.left = 100, width = 1000 => 30%)
      const downEvent = new PointerEvent('pointerdown', {
        clientX: 400,
        clientY: 300,
        pointerId: 1,
        button: 0,
        cancelable: true,
      });
      const preventDefaultSpy = vi.spyOn(downEvent, 'preventDefault');

      container.dispatchEvent(downEvent);

      expect(preventDefaultSpy).toHaveBeenCalled();
      expect(setPointerCaptureSpy).toHaveBeenCalledWith(1);
      expect(controller.getSplitPosition()).toBe(30);
      expect(container.style.getPropertyValue('--split-pos')).toBe('30%');
      expect(layerBefore.style.clipPath).toBe(
        'polygon(0 0, 30% 0, 30% 100%, 0 100%)',
      );

      // Drag to clientX = 850 (rect.left = 100 => 750 / 1000 = 75%)
      const moveEvent = new PointerEvent('pointermove', {
        clientX: 850,
        clientY: 300,
        pointerId: 1,
        cancelable: true,
      });
      window.dispatchEvent(moveEvent);

      expect(controller.getSplitPosition()).toBe(75);
      expect(container.style.getPropertyValue('--split-pos')).toBe('75%');
      expect(layerBefore.style.clipPath).toBe(
        'polygon(0 0, 75% 0, 75% 100%, 0 100%)',
      );

      // Pointer up
      const upEvent = new PointerEvent('pointerup', {
        clientX: 850,
        clientY: 300,
        pointerId: 1,
      });
      window.dispatchEvent(upEvent);

      expect(releasePointerCaptureSpy).toHaveBeenCalledWith(1);

      // Moving after pointer up should not change position
      const moveAfterUp = new PointerEvent('pointermove', {
        clientX: 200,
        clientY: 300,
        pointerId: 1,
      });
      window.dispatchEvent(moveAfterUp);
      expect(controller.getSplitPosition()).toBe(75);
    });

    it('tolerates pointer capture failures and cancellations gracefully', () => {
      // Simulate Android Chrome throwing on setPointerCapture
      container.setPointerCapture = vi.fn().mockImplementation(() => {
        throw new Error('InvalidStateError');
      });

      const downEvent = new PointerEvent('pointerdown', {
        clientX: 500,
        clientY: 300,
        pointerId: 2,
        button: 0,
      });
      expect(() => container.dispatchEvent(downEvent)).not.toThrow();
      expect(controller.getSplitPosition()).toBe(40); // (500 - 100) / 1000 = 40%

      // Simulate pointercancel from Android Chrome gesture detector
      const cancelEvent = new PointerEvent('pointercancel', {
        clientX: 500,
        clientY: 300,
        pointerId: 2,
      });
      expect(() => window.dispatchEvent(cancelEvent)).not.toThrow();

      // Subsequent moves should be ignored
      const moveEvent = new PointerEvent('pointermove', {
        clientX: 900,
        clientY: 300,
        pointerId: 2,
      });
      window.dispatchEvent(moveEvent);
      expect(controller.getSplitPosition()).toBe(40);
    });
  });

  describe('Optics Workspace & Lens Corrections (Distortion, TCA, Vignetting)', () => {
    it('switches to the optics workspace tab and updates panel visibility', () => {
      document.body.innerHTML = `
        <button class="seg-btn" data-tab="develop"></button>
        <button class="seg-btn" data-tab="optics"></button>
        <div class="workspace-panel" id="tab-develop"></div>
        <div class="workspace-panel hidden" id="tab-optics"></div>
      `;
      const edits = getDefaultEdits(true);
      const controller = new InspectorSyncController({
        getEdits: () => edits,
        renderCanvas: vi.fn(),
        autoSaveSidecar: vi.fn(),
        isCurrentPhotoRaw: () => true,
        getDefaultEdits: () => edits,
        getCurveWidget: () => null as unknown as ToneCurveWidget,
        getRating: () => 0,
        getStatus: () => '',
        getActiveIndex: () => 0,
      });

      controller.selectInspectorTab('optics');

      const opticsBtn = document.querySelector('.seg-btn[data-tab="optics"]');
      const opticsPanel = document.getElementById('tab-optics');
      const developPanel = document.getElementById('tab-develop');

      expect(opticsBtn?.classList.contains('active')).toBe(true);
      expect(opticsPanel?.classList.contains('hidden')).toBe(false);
      expect(developPanel?.classList.contains('hidden')).toBe(true);
    });

    it('toggles vignetting correction, mutates edits, and renders canvas', () => {
      document.body.innerHTML = `
        <input type="checkbox" id="toggle-distortion" />
        <input type="checkbox" id="toggle-tca" checked />
        <input type="checkbox" id="toggle-vignetting" />
      `;
      const edits = getDefaultEdits(true);
      const renderCanvas = vi.fn();
      const autoSaveSidecar = vi.fn();
      const controller = new InspectorSyncController({
        getEdits: () => edits,
        renderCanvas,
        autoSaveSidecar,
        isCurrentPhotoRaw: () => true,
        getDefaultEdits: () => edits,
        getCurveWidget: () => null as unknown as ToneCurveWidget,
        getRating: () => 0,
        getStatus: () => '',
        getActiveIndex: () => 0,
      });

      controller.bind();

      const toggleVig = document.getElementById(
        'toggle-vignetting',
      ) as HTMLInputElement;
      expect(toggleVig.checked).toBe(false);

      toggleVig.checked = true;
      toggleVig.dispatchEvent(new Event('change'));

      expect(edits.vignetting_correction_enabled).toBe(true);
      expect(renderCanvas).toHaveBeenCalledTimes(1);
      expect(autoSaveSidecar).toHaveBeenCalledTimes(1);
    });

    it('synchronizes optics toggles and defringe slider from edits in syncSliderUI', () => {
      document.body.innerHTML = `
        <input type="checkbox" id="toggle-distortion" />
        <input type="checkbox" id="toggle-tca" />
        <input type="checkbox" id="toggle-vignetting" />
        <input type="range" id="slider-defringe-amount" />
        <span id="val-defringe-amount"></span>
      `;
      const edits = getDefaultEdits(true);
      edits.lens_correction_enabled = true;
      edits.tca_correction_enabled = false;
      edits.vignetting_correction_enabled = true;
      edits.defringe_amount = 45.0;

      const controller = new InspectorSyncController({
        getEdits: () => edits,
        renderCanvas: vi.fn(),
        autoSaveSidecar: vi.fn(),
        isCurrentPhotoRaw: () => true,
        getDefaultEdits: () => edits,
        getCurveWidget: () => null as unknown as ToneCurveWidget,
        getRating: () => 0,
        getStatus: () => '',
        getActiveIndex: () => 0,
      });

      controller.syncSliderUI();

      const toggleDist = document.getElementById(
        'toggle-distortion',
      ) as HTMLInputElement;
      const toggleTca = document.getElementById(
        'toggle-tca',
      ) as HTMLInputElement;
      const toggleVig = document.getElementById(
        'toggle-vignetting',
      ) as HTMLInputElement;
      const defringeVal = document.getElementById('val-defringe-amount');

      expect(toggleDist.checked).toBe(true);
      expect(toggleTca.checked).toBe(false);
      expect(toggleVig.checked).toBe(true);
      expect(defringeVal?.innerText).toBe('45');
    });

    it('updates lens correction toggles disabled state and telemetry card in updateLensCorrectionUI', () => {
      document.body.innerHTML = `
        <input type="checkbox" id="toggle-distortion" />
        <input type="checkbox" id="toggle-tca" />
        <input type="checkbox" id="toggle-vignetting" />
        <div id="lens-calibration-telemetry" style="display: none;">
          <div id="lens-telemetry-model"></div>
          <div id="lens-telemetry-distortion"></div>
          <div id="lens-telemetry-tca"></div>
          <div id="lens-telemetry-vignetting"></div>
        </div>
      `;
      const edits = getDefaultEdits(true);
      const controller = new InspectorSyncController({
        getEdits: () => edits,
        renderCanvas: vi.fn(),
        autoSaveSidecar: vi.fn(),
        isCurrentPhotoRaw: () => true,
        getDefaultEdits: () => edits,
        getCurveWidget: () => null as unknown as ToneCurveWidget,
        getRating: () => 0,
        getStatus: () => '',
        getActiveIndex: () => 0,
      });

      // 1. Uncalibrated / null
      controller.updateLensCorrectionUI(null);
      const toggleDist = document.getElementById(
        'toggle-distortion',
      ) as HTMLInputElement;
      const toggleTca = document.getElementById(
        'toggle-tca',
      ) as HTMLInputElement;
      const toggleVig = document.getElementById(
        'toggle-vignetting',
      ) as HTMLInputElement;
      const card = document.getElementById(
        'lens-calibration-telemetry',
      ) as HTMLElement;
      expect(toggleDist.disabled).toBe(true);
      expect(toggleTca.disabled).toBe(true);
      expect(toggleVig.disabled).toBe(true);
      expect(card.style.display).toBe('none');

      // 2. Calibrated Lensfun record
      controller.updateLensCorrectionUI(
        {
          distortion: [0.02964, 0, 0, 0],
          tca: [1.0002, 1.00005, 0, 0, 0, 0],
          vignetting: [-0.8, 0.2, 0],
          distortion_model: 'ptlens',
          has_distortion: true,
          has_tca: true,
          has_vignetting: true,
        },
        'Canon EF 24-70mm f/2.8L II USM',
      );
      expect(toggleDist.disabled).toBe(false);
      expect(toggleTca.disabled).toBe(false);
      expect(toggleVig.disabled).toBe(false);
      expect(card.style.display).toBe('block');
      expect(document.getElementById('lens-telemetry-model')?.textContent).toBe(
        'Canon EF 24-70mm f/2.8L II USM',
      );
      expect(
        document.getElementById('lens-telemetry-distortion')?.textContent,
      ).toBe('Distortion: PTLENS');
      expect(document.getElementById('lens-telemetry-tca')?.textContent).toBe(
        'Lateral CA: Calibrated',
      );
      expect(
        document.getElementById('lens-telemetry-vignetting')?.textContent,
      ).toBe('Vignetting: Calibrated');
    });
  });
});
