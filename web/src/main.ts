import {
  WasmBridge,
  EditSettingsPayload,
  XmpPayload,
  PreviewResult,
  ColorProfileMode,
  EmulsionArchetype,
  PrintMediaMode,
} from './WasmBridge';
import { WebGLRenderer } from './WebGLRenderer';
import { FolderIntake, PhotoFileEntry } from './FolderIntake';
import { ToneCurveWidget } from './ToneCurveWidget';
import { ThumbnailCacheStore } from './ThumbnailCacheStore';
import { ParityModal } from './parity/ParityModal';
import { CropController, CropRect } from './CropController';
import { ExportModal, ExportExecutionOptions } from './ExportModal';
import { encodeTiff } from './utils/tiffEncoder';
import {
  formatAperture,
  formatShutterSpeed,
  formatFocalLength,
  formatISO,
  formatExposure,
  formatDimensions
} from './utils/formatters';
import { readExifOrientationFromBlob } from './utils/exifReader';

import { decode_exif_orientation } from 'firstproof-wasm';
import type { ExifOrientationRecord } from 'firstproof-wasm';

export function decodeExifOrientation(orientation?: number): {
  rotation: number;
  flipH: boolean;
  flipV: boolean;
  flip_horizontal: boolean;
  flip_vertical: boolean;
} {
  const rec: ExifOrientationRecord = orientation
    ? decode_exif_orientation(orientation)
    : { rotation: 0, flip_horizontal: false, flip_vertical: false };
  return {
    rotation: rec.rotation,
    flipH: rec.flip_horizontal,
    flipV: rec.flip_vertical,
    flip_horizontal: rec.flip_horizontal,
    flip_vertical: rec.flip_vertical,
  };
}

interface ActiveSliderScrub {
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

class DarkroomApp {
  private bridge!: WasmBridge;
  private renderer!: WebGLRenderer;
  private curveWidget!: ToneCurveWidget;
  private parityModal: ParityModal = new ParityModal();
  private exportModal: ExportModal = new ExportModal();
  private cropController!: CropController;
  private exportCanvas: HTMLCanvasElement | null = null;
  private exportRenderer: WebGLRenderer | null = null;

  private getExportRenderer(): { canvas: HTMLCanvasElement; renderer: WebGLRenderer } {
    if (!this.exportCanvas || !this.exportRenderer) {
      this.exportCanvas = document.createElement('canvas');
      this.exportRenderer = new WebGLRenderer(this.exportCanvas);
    }
    return { canvas: this.exportCanvas, renderer: this.exportRenderer };
  }

  private fileEntries: PhotoFileEntry[] = [];
  private activeIdx: number = -1;
  private selectedIndices: Set<number> = new Set();
  private dirHandle: any = null;
  private activeScrub: ActiveSliderScrub | null = null;

  private edits: EditSettingsPayload = this.getDefaultEdits();
  private cardRotations: Map<number, { rotation: number; flipH: boolean; flipV: boolean }> = new Map();
  private isOriginalPeek: boolean = false;
  private isCameraJpegPeek: boolean = false;
  private currentMetadata: any = {};
  private currentRating: number = 0;
  private currentStatus: string = 'Unflagged';
  private activeEyedropper: 'wb' | 'white' | 'black' | null = null;
  private selectRequestId: number = 0;
  private selectDebounceTimer: any = null;
  private preloadDebounceTimer: any = null;
  private thumbLoadingSet: Set<number> = new Set();
  private prewarmLoadingSet: Set<number> = new Set();

  // Zoom & Pan state (100% 1:1 sensor pixel peeping)
  private zoomScale: number = 1.0;
  private panOffset: { x: number; y: number } = { x: 0, y: 0 };
  private isPanning: boolean = false;
  private panStartPos: { x: number; y: number } = { x: 0, y: 0 };
  private panStartOffset: { x: number; y: number } = { x: 0, y: 0 };

  // Crop & Straighten tool state
  private isCropActive: boolean = false;

  private activeLinearData: Uint16Array | null = null;
  private activeLinearW: number = 0;
  private activeLinearH: number = 0;

  private activeSourceImage: CanvasImageSource | null = null;
  private activeSourceW: number = 0;
  private activeSourceH: number = 0;

  private rawWorker: Worker | null = null;
  private workerMsgId: number = 0;
  private workerPendingMap: Map<number, (res: any) => void> = new Map();

  private elements: Record<string, HTMLElement> = {};

  // Perceptual Mired mapping matching macOS TemperatureSliderRow
  // Human color perception of blackbody temperature is linear in Mireds (10^6 / K), not linear Kelvin.
  // 2000 K  -> 500 Mireds (pos = 0.0)
  // 5500 K  -> 181.8 Mireds (pos = 0.707)
  // 20000 K -> 50 Mireds (pos = 1.0)
  private kelvinToMiredPos(kelvin: number): number {
    const k = Math.min(Math.max(kelvin, 2000.0), 20000.0);
    const mired = 1_000_000.0 / k;
    const pos = (500.0 - mired) / 450.0;
    return Math.min(Math.max(pos, 0.0), 1.0);
  }

  private miredPosToKelvin(pos: number): number {
    const p = Math.min(Math.max(pos, 0.0), 1.0);
    const mired = 500.0 - p * 450.0;
    const rawK = 1_000_000.0 / mired;
    // Stepped in 50K increments below 10,000K, 200K increments above
    const steppedK = (rawK < 10000.0)
      ? Math.round(rawK / 50.0) * 50.0
      : Math.round(rawK / 200.0) * 200.0;
    return Math.min(Math.max(steppedK, 2000.0), 20000.0);
  }

  private updateSplitToningHint() {
    const hasSplitToning = (this.edits.split_toning_shadow_sat > 0) || (this.edits.split_toning_highlight_sat > 0);
    const hintEl = document.getElementById('hint-split-toning');
    if (hintEl) {
      hintEl.innerText = hasSplitToning ? 'Active' : 'Off';
      hintEl.classList.toggle('active', hasSplitToning);
    }
  }

  private rollbackActiveScrub() {
    if (!this.activeScrub) return;
    const scrub = this.activeScrub;
    scrub.input.value = scrub.initialInputValue;
    if (scrub.isTemp) {
      this.edits.temperature = scrub.initialValue;
      scrub.label.innerText = `${Math.round(scrub.initialValue)} K`;
    } else if (scrub.prop) {
      (this.edits[scrub.prop] as number) = scrub.initialValue;
      scrub.label.innerText = scrub.format(scrub.initialValue);
      if ((scrub.prop as string).startsWith('split_toning_')) {
        this.updateSplitToningHint();
      }
    }
    this.renderCanvas();
    scrub.input.blur();
    this.activeScrub = null;
  }

  private getSceneLuminances(): Float32Array {
    const canvas = document.getElementById('darkroom-canvas') as HTMLCanvasElement;
    const gl = canvas?.getContext('webgl2') as WebGL2RenderingContext;
    if (!gl) return new Float32Array(0);

    const sampleW = Math.min(100, canvas.width);
    const sampleH = Math.min(100, canvas.height);
    const startX = Math.max(0, Math.floor((canvas.width - sampleW) / 2));
    const startY = Math.max(0, Math.floor((canvas.height - sampleH) / 2));
    const pixels = new Uint8Array(sampleW * sampleH * 4);
    gl.readPixels(startX, startY, sampleW, sampleH, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    const lums = new Float32Array(sampleW * sampleH);
    for (let i = 0; i < sampleW * sampleH; i++) {
      const r = pixels[i * 4] / 255.0;
      const g = pixels[i * 4 + 1] / 255.0;
      const b = pixels[i * 4 + 2] / 255.0;
      lums[i] = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    }
    return lums;
  }

  public applyAutoTone() {
    if (this.activeIdx < 0) return;

    let adj: { exposure: number; contrast: number } | null = null;
    if (this.activeLinearData && this.activeLinearW > 0 && this.activeLinearH > 0) {
      adj = this.bridge.computeAutoAdjustments(this.activeLinearData, this.activeLinearW, this.activeLinearH);
    }

    if (!adj) {
      const lums = this.getSceneLuminances();
      if (lums.length > 0) {
        adj = this.bridge.computeAutoAdjustmentsFromLuminances(lums);
      }
    }

    if (adj) {
      this.edits.exposure = adj.exposure;
      this.edits.contrast = adj.contrast;
      this.syncSliderUI();
      this.renderCanvas();
      this.autoSaveSidecar();
      console.log(`⚡ [Auto Tone] Conservative Auto Tone applied: Exposure = ${adj.exposure > 0 ? '+' : ''}${adj.exposure.toFixed(2)} EV, Contrast = ${adj.contrast}`);
    }
  }

  public applyAutoWhiteBalance() {
    if (this.activeIdx < 0) return;

    let wb: { temperature: number; tint: number } | null = null;
    if (this.activeLinearData && this.activeLinearW > 0 && this.activeLinearH > 0) {
      wb = this.bridge.computeAutoWhiteBalance(this.activeLinearData, this.activeLinearW, this.activeLinearH);
    }

    if (!wb) {
      const canvas = document.getElementById('darkroom-canvas') as HTMLCanvasElement;
      const gl = canvas?.getContext('webgl2') as WebGL2RenderingContext;
      if (gl) {
        const readW = Math.min(100, canvas.width);
        const readH = Math.min(100, canvas.height);
        const pixels = new Uint8Array(readW * readH * 4);
        gl.readPixels(0, 0, readW, readH, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        let rSum = 0, gSum = 0, bSum = 0, count = 0;
        for (let i = 0; i < readW * readH; i++) {
          const r = pixels[i * 4] / 255.0;
          const g = pixels[i * 4 + 1] / 255.0;
          const b = pixels[i * 4 + 2] / 255.0;
          const y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
          if (y >= 0.05 && y <= 0.85) {
            rSum += r; gSum += g; bSum += b; count++;
          }
        }
        if (count > 20) {
          wb = this.bridge.calculateWhiteBalanceFromSample(rSum / count, gSum / count, bSum / count);
        }
      }
    }

    if (wb) {
      this.edits.temperature = wb.temperature;
      this.edits.tint = wb.tint;
      this.syncSliderUI();
      this.renderCanvas();
      this.autoSaveSidecar();
      console.log(`⚡ [Auto WB] Applied: Temp = ${wb.temperature} K, Tint = ${wb.tint}`);
    }
  }

  private initWorker() {
    try {
      this.rawWorker = new Worker(new URL('./RawDecodeWorker.ts', import.meta.url), { type: 'module' });
      this.rawWorker.onmessage = (e: MessageEvent) => {
        const { id } = e.data;
        const resolve = this.workerPendingMap.get(id);
        if (resolve) {
          this.workerPendingMap.delete(id);
          resolve(e.data);
        }
      };
      console.log('⚡ [App] Off-main-thread RawDecodeWorker active');
    } catch (err) {
      console.warn('[App] WebWorker pool initialization fallback:', err);
    }
  }

  private async decodeInWorker(arrayBuffer: ArrayBuffer, fileName: string, fullResolution: boolean = false): Promise<{ preview: PreviewResult; imageSlice: Uint8Array; linearData?: Uint16Array; linearW?: number; linearH?: number }> {
    if (!this.rawWorker) {
      const bytes = new Uint8Array(arrayBuffer);
      const res = this.bridge.decodeImageBytes(bytes);
      const ext = fileName.split('.').pop()?.toLowerCase() || '';
      const isRawFile = ['cr2', 'cr3', 'nef', 'arw', 'dng', 'rw2', 'orf', 'pef', 'raf'].includes(ext);
      let linearData: Uint16Array | undefined;
      let linearW: number | undefined;
      let linearH: number | undefined;
      if (isRawFile) {
        try {
          const rawLin = fullResolution ? this.bridge.decodeRawFullLinear(bytes) : this.bridge.decodeRawLinear(bytes);
          linearData = rawLin.linearData;
          linearW = rawLin.res.width;
          linearH = rawLin.res.height;
        } catch (e) {
          console.warn('[main] Fallback decodeRawLinear failed:', e);
        }
      }
      return { preview: res.preview, imageSlice: res.imageSlice, linearData, linearW, linearH };
    }

    const id = ++this.workerMsgId;
    return new Promise((resolve, reject) => {
      this.workerPendingMap.set(id, (res) => {
        if (res.success && res.preview && res.imageSlice) {
          resolve({
            preview: res.preview,
            imageSlice: res.imageSlice,
            linearData: res.linearData,
            linearW: res.linearW,
            linearH: res.linearH,
          });
        } else {
          try {
            const bytes = new Uint8Array(arrayBuffer);
            resolve(this.bridge.decodeImageBytes(bytes));
          } catch (e) {
            reject(new Error(res.error || 'Worker decode failed'));
          }
        }
      });
      const workerBuffer = arrayBuffer.slice(0);
      this.rawWorker!.postMessage({ id, arrayBuffer: workerBuffer, fileName, fullResolution }, [workerBuffer]);
    });
  }

  public async init() {
    console.log('[App] Starting First Proof Web Darkroom Engine...');
    this.cacheElements();

    // 1. Init WASM (256MB scratchpad capacity for 50MP+ 16-bit linear RAW frames)
    try {
      this.bridge = await WasmBridge.init(256);
      this.initWorker();
    } catch (err) {
      console.error('[App] Failed to initialize WASM scratchpad:', err);
      alert('Failed to initialize WebAssembly core engine. Check browser console.');
      return;
    }

    // 2. Init WebGL Renderer
    const canvas = document.getElementById('darkroom-canvas') as HTMLCanvasElement;
    canvas.width = 1920;
    canvas.height = 1080;
    this.renderer = new WebGLRenderer(canvas);
    this.parityModal.init(
      this.renderer,
      () => this.edits,
      () => {
        if (this.activeIdx < 0) return null;
        if (this.activeLinearData && this.activeLinearW > 0 && this.activeLinearH > 0) {
          return {
            isLinearRec2020: true,
            width: this.activeLinearW,
            height: this.activeLinearH,
            linearData: this.activeLinearData,
          };
        } else if (this.activeSourceImage && this.activeSourceW > 0 && this.activeSourceH > 0) {
          return {
            isLinearRec2020: false,
            width: this.activeSourceW,
            height: this.activeSourceH,
            imageSource: this.activeSourceImage,
          };
        }
        return null;
      }
    );

    // 3. Init Export Modal (macOS ExportSheetView Parity)
    this.exportModal.init((photos, options, onProgress) => this.handleExport(photos, options, onProgress));

    // 4. Init Tone Curve Widget
    const curveCanvas = document.getElementById('curve-canvas') as HTMLCanvasElement;
    this.curveWidget = new ToneCurveWidget(curveCanvas, () => this.onCurveChange());
    const initialLut = this.bridge.evaluateCurves(this.edits.curves);
    this.renderer.updateToneCurvesLut(initialLut);

    // 4. Init Interactive Crop Controller
    this.cropController = new CropController('crop-overlay', {
      onCropUpdate: (crop) => {
        this.edits.crop_left = crop.left;
        this.edits.crop_top = crop.top;
        this.edits.crop_right = crop.right;
        this.edits.crop_bottom = crop.bottom;
        this.renderCanvas();
      },
      onStraightenUpdate: (angle) => {
        this.edits.crop_angle = angle;
        const straightenSlider = document.getElementById('slider-straighten') as HTMLInputElement;
        const straightenVal = document.getElementById('val-straighten');
        if (straightenSlider) straightenSlider.value = `${angle}`;
        if (straightenVal) straightenVal.textContent = `${angle > 0 ? '+' : ''}${angle.toFixed(1)}°`;
        this.renderCanvas();
      },
      onStraightenEnd: () => {
        this.autoSaveSidecar();
      },
      onDone: () => {
        this.toggleCropMode(false);
      },
      onCancel: () => {
        this.toggleCropMode(false);
      }
    });

    // 5. Bind UI Controls & Shortcuts
    this.bindEvents();
    this.updateUI();

    // 5. Restore recent photo shoot folder memory from IndexedDB across browser restarts
    FolderIntake.getRecentDirHandle().then((recent) => {
      if (recent) {
        const emptyActions = document.querySelector('.empty-actions');
        if (emptyActions && !document.getElementById('btn-reopen-recent')) {
          const btnRecent = document.createElement('button');
          btnRecent.id = 'btn-reopen-recent';
          btnRecent.className = 'btn btn-secondary btn-lg';
          btnRecent.style.marginLeft = '0.6rem';
          btnRecent.innerHTML = `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg> Re-open ${recent.name}`;
          btnRecent.addEventListener('click', async () => {
            const entries = await FolderIntake.processDirectoryHandle(recent.dirHandle);
            if (entries.length > 0) {
              this.fileEntries = entries;
              this.dirHandle = recent.dirHandle;
              this.onPhotosLoaded();
            }
          });
          emptyActions.appendChild(btnRecent);
        }
      }
    });

    console.log('[App] Ready for photographic intake.');
  }

  public isCurrentPhotoRaw(): boolean {
    if (this.activeIdx < 0 || this.activeIdx >= this.fileEntries.length) return false;
    const name = this.fileEntries[this.activeIdx].name;
    const ext = name.split('.').pop()?.toLowerCase() || '';
    return ['cr2', 'cr3', 'nef', 'arw', 'dng', 'rw2', 'orf', 'pef', 'raf'].includes(ext);
  }

  private getDefaultEdits(isRaw: boolean = false): EditSettingsPayload {
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
      curves: {
        master: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
        red: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
        green: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
        blue: [{ x: 0, y: 0 }, { x: 1, y: 1 }]
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
      print_process_depth: 100.0
    };
  }

  private cacheElements() {
    const ids = [
      'btn-open-folder', 'file-input-fallback', 'btn-original-peek', 'btn-camera-jpeg-peek', 'btn-export-proof', 'btn-parity-audit', 'btn-shortcuts',
      'empty-state', 'btn-empty-open', 'developing-spinner', 'filmstrip-count', 'filmstrip-scroll',
      'meta-filename', 'meta-dimensions', 'meta-camera', 'meta-iso', 'meta-aperture', 'meta-shutter',
      'meta-format-badge', 'meta-full-camera', 'meta-lens', 'meta-focal-length', 'meta-exposure-summary',
      'meta-sensor-res', 'meta-date-taken', 'meta-sidecar-status', 'btn-reset-all',
      'btn-pick', 'btn-reject', 'modal-shortcuts', 'btn-close-modal', 'select-tone-profile',
      'select-wb-preset', 'btn-wb-eyedropper', 'btn-white-point-picker', 'btn-black-point-picker',
      'select-emulsion', 'select-print-media', 'select-enlarger-filter', 'row-enlarger-filter', 'row-emulsion-exp', 'section-print-media-sliders', 'btn-reset-curve', 'btn-auto-tone',
      'meta-input-title', 'meta-input-caption', 'meta-input-creator', 'meta-input-keywords',
      'btn-crop-tool', 'btn-rotate-ccw', 'btn-rotate-cw', 'zoom-badge', 'peek-badge'
    ];
    for (const id of ids) {
      const el = document.getElementById(id);
      if (el) this.elements[id] = el;
    }
  }

  private toggleEyedropper(mode: 'wb' | 'white' | 'black') {
    const wrapper = document.getElementById('canvas-wrapper');
    if (this.activeEyedropper === mode) {
      this.activeEyedropper = null;
      if (wrapper) wrapper.style.cursor = 'default';
      this.elements['btn-wb-eyedropper']?.classList.remove('active');
      this.elements['btn-white-point-picker']?.classList.remove('active');
      this.elements['btn-black-point-picker']?.classList.remove('active');
    } else {
      this.activeEyedropper = mode;
      if (wrapper) wrapper.style.cursor = 'crosshair';
      this.elements['btn-wb-eyedropper']?.classList.toggle('active', mode === 'wb');
      this.elements['btn-white-point-picker']?.classList.toggle('active', mode === 'white');
      this.elements['btn-black-point-picker']?.classList.toggle('active', mode === 'black');
    }
  }

  private bindEvents() {
    // Open Folder Intake Buttons
    const handleOpenFolder = async () => {
      if ('showDirectoryPicker' in window) {
        try {
          const res = await FolderIntake.openDirectoryPicker();
          if (res.entries.length > 0) {
            this.fileEntries = res.entries;
            this.dirHandle = res.dirHandle;
            this.onPhotosLoaded();
            return;
          }
          // Do not show secondary fallback picker if user explicitly cancelled Chrome dialog
          if (res.cancelled) {
            return;
          }
        } catch (err) {
          console.warn('[FolderIntake] openDirectoryPicker error:', err);
        }
      }

      // Fallback for Firefox/Safari or if showDirectoryPicker yields 0 files without cancellation
      const fallbackEl = this.elements['file-input-fallback'] as HTMLInputElement;
      if (fallbackEl) {
        fallbackEl.value = '';
        fallbackEl.click();
      }
    };

    this.elements['btn-open-folder'].addEventListener('click', handleOpenFolder);
    this.elements['btn-empty-open'].addEventListener('click', handleOpenFolder);

    (this.elements['file-input-fallback'] as HTMLInputElement).addEventListener('change', (e: Event) => {
      const target = e.target as HTMLInputElement;
      if (target.files && target.files.length > 0) {
        this.fileEntries = FolderIntake.processFileList(target.files);
        this.onPhotosLoaded();
      }
    });

    // Original Peek \
    const btnPeek = this.elements['btn-original-peek'];
    btnPeek.addEventListener('mousedown', () => { this.isOriginalPeek = true; this.renderCanvas(); });
    btnPeek.addEventListener('mouseup', () => { this.isOriginalPeek = false; this.renderCanvas(); });
    btnPeek.addEventListener('mouseleave', () => { this.isOriginalPeek = false; this.renderCanvas(); });

    // Camera JPEG Peek J
    const btnCameraJpeg = this.elements['btn-camera-jpeg-peek'];
    if (btnCameraJpeg) {
      btnCameraJpeg.addEventListener('mousedown', () => { this.isCameraJpegPeek = true; this.renderCanvas(); });
      btnCameraJpeg.addEventListener('mouseup', () => { this.isCameraJpegPeek = false; this.renderCanvas(); });
      btnCameraJpeg.addEventListener('mouseleave', () => { this.isCameraJpegPeek = false; this.renderCanvas(); });
    }

    // Export Photo Modal (Cmd+E / Toolbar Button)
    this.elements['btn-export-proof'].addEventListener('click', () => this.openExportModal());

    // Mechanical Parity Audit Modal
    this.elements['btn-parity-audit']?.addEventListener('click', () => this.parityModal.show());

    // Shortcuts Modal
    const modal = this.elements['modal-shortcuts'] as HTMLDialogElement;
    this.elements['btn-shortcuts'].addEventListener('click', () => modal.showModal());
    this.elements['btn-close-modal'].addEventListener('click', () => modal.close());

    // Rating & Flag Toolbar
    const starBtns = document.querySelectorAll('.star-btn');
    starBtns.forEach(btn => {
      btn.addEventListener('click', (e) => {
        const star = parseInt((e.target as HTMLElement).getAttribute('data-star') || '0', 10);
        this.currentRating = (this.currentRating === star) ? 0 : star;
        this.syncRatingFlagUI();
        this.autoSaveSidecar();
      });
    });

    this.elements['btn-pick'].addEventListener('click', () => {
      this.currentStatus = (this.currentStatus === 'Pick') ? 'Unflagged' : 'Pick';
      this.syncRatingFlagUI();
      this.autoSaveSidecar();
    });

    this.elements['btn-reject'].addEventListener('click', () => {
      this.currentStatus = (this.currentStatus === 'Reject') ? 'Unflagged' : 'Reject';
      this.syncRatingFlagUI();
      this.autoSaveSidecar();
    });

    // Eyedroppers & Pickers
    this.elements['btn-wb-eyedropper']?.addEventListener('click', () => this.toggleEyedropper('wb'));
    this.elements['btn-white-point-picker']?.addEventListener('click', () => this.toggleEyedropper('white'));
    this.elements['btn-black-point-picker']?.addEventListener('click', () => this.toggleEyedropper('black'));

    const canvas = document.getElementById('darkroom-canvas') as HTMLCanvasElement;
    canvas.addEventListener('click', (e: MouseEvent) => {
      if (!this.activeEyedropper) return;
      const rect = canvas.getBoundingClientRect();
      const x = Math.floor((e.clientX - rect.left) * (canvas.width / rect.width));
      const y = Math.floor((e.clientY - rect.top) * (canvas.height / rect.height));

      const gl = (canvas as any).getContext('webgl2') as WebGL2RenderingContext;
      if (gl) {
        const pixels = new Uint8Array(4);
        const glY = canvas.height - y - 1;
        gl.readPixels(x, glY, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        const [r, g, b] = [pixels[0] / 255, pixels[1] / 255, pixels[2] / 255];

        if (this.activeEyedropper === 'wb') {
          if (r > 0 && g > 0 && b > 0) {
            const curTemp = this.edits.temperature;
            const curTint = this.edits.tint;
            const targetTemp = Math.min(Math.max(Math.round(curTemp * Math.pow(b / r, 0.35)), 2000), 15000);
            const targetTint = Math.min(Math.max(Math.round(curTint + (g - (r + b) / 2) * 80), -150), 150);

            this.edits.temperature = targetTemp;
            this.edits.tint = targetTint;
            if (this.elements['select-wb-preset']) {
              (this.elements['select-wb-preset'] as HTMLSelectElement).value = 'Custom';
            }
          }
        } else if (this.activeEyedropper === 'white') {
          const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
          const shift = Math.round(Math.min(Math.max((1.0 - lum) * 100, -100), 100));
          this.edits.whites = shift;
        } else if (this.activeEyedropper === 'black') {
          const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
          const shift = Math.round(Math.min(Math.max(-lum * 100, -100), 100));
          this.edits.blacks = shift;
        }

        this.syncSliderUI();
        this.renderCanvas();
        this.autoSaveSidecar();
        this.toggleEyedropper(this.activeEyedropper);
      }
    });

    // Header Crop & Rotation Tools
    this.elements['btn-crop-tool']?.addEventListener('click', () => this.toggleCropMode());
    this.elements['btn-rotate-ccw']?.addEventListener('click', () => this.rotateImage(-90));
    this.elements['btn-rotate-cw']?.addEventListener('click', () => this.rotateImage(90));
    this.elements['zoom-badge']?.addEventListener('click', () => this.togglePixelPeep());

    // Canvas Double-Click to Pixel Peep (100% 1:1 sensor view)
    canvas.addEventListener('dblclick', (e: MouseEvent) => {
      if (this.isCropActive) return;
      this.togglePixelPeep(e.clientX, e.clientY);
    });

    // Canvas Drag to Pan Viewport when zoomed
    canvas.addEventListener('mousedown', (e: MouseEvent) => {
      if (this.activeEyedropper) return;
      if (this.isCropActive) return;
      if (this.zoomScale > 1.05 && e.button === 0) {
        this.isPanning = true;
        this.panStartPos = { x: e.clientX, y: e.clientY };
        this.panStartOffset = { ...this.panOffset };
        canvas.style.cursor = 'grabbing';
        e.preventDefault();
      }
    });

    window.addEventListener('mousemove', (e: MouseEvent) => {
      if (this.isPanning) {
        const wrapper = document.getElementById('canvas-wrapper') as HTMLElement;
        const rect = wrapper ? wrapper.getBoundingClientRect() : canvas.getBoundingClientRect();
        if (rect.width > 0 && rect.height > 0) {
          const dx = ((e.clientX - this.panStartPos.x) / (rect.width / 2));
          const dy = -((e.clientY - this.panStartPos.y) / (rect.height / 2));
          this.panOffset.x = this.panStartOffset.x + dx;
          this.panOffset.y = this.panStartOffset.y + dy;
          this.clampPanOffset();
          this.renderCanvas();
        }
      }
    });

    window.addEventListener('mouseup', () => {
      if (this.isPanning) {
        this.isPanning = false;
        canvas.style.cursor = this.zoomScale > 1.05 ? 'grab' : 'default';
      }
    });

    window.addEventListener('resize', () => {
      this.updateCanvasDimensions();
      this.renderCanvas();
      if (this.isCropActive) {
        this.updateCropOverlayPosition();
      }
    });

    // Auto Tone Button (Light & Exposure section)
    this.elements['btn-auto-tone']?.addEventListener('click', () => this.applyAutoTone());

    // Light Source Presets (Identical parity with Swift AppViewModel+Editor.swift)
    this.elements['select-wb-preset']?.addEventListener('change', (e) => {
      const preset = (e.target as HTMLSelectElement).value;
      switch (preset) {
        case 'AsShot':
          this.edits.temperature = 5500.0;
          this.edits.tint = 0.0;
          break;
        case 'Auto':
          this.applyAutoWhiteBalance();
          break;
        case 'Daylight':
          this.edits.temperature = 5500.0;
          this.edits.tint = 10.0;
          break;
        case 'Cloudy':
          this.edits.temperature = 6500.0;
          this.edits.tint = 10.0;
          break;
        case 'Shade':
          this.edits.temperature = 7500.0;
          this.edits.tint = 10.0;
          break;
        case 'Tungsten':
          this.edits.temperature = 2850.0;
          this.edits.tint = 0.0;
          break;
        case 'Fluorescent':
          this.edits.temperature = 3800.0;
          this.edits.tint = 45.0;
          break;
        case 'SodiumVapor':
          this.edits.temperature = 2200.0;
          this.edits.tint = 160.0;
          break;
        case 'Flash':
          this.edits.temperature = 5500.0;
          this.edits.tint = 0.0;
          break;
      }
      this.syncSliderUI();
      this.renderCanvas();
      this.autoSaveSidecar();
    });

    // Sliders & Selects plumbing with double-click reset, Escape rollback, and Shift fine-scrubbing
    const bindSlider = (id: string, prop: keyof EditSettingsPayload, format: (val: number) => string, defaultVal: number | (() => number) = 0) => {
      const input = document.getElementById(`slider-${id}`) as HTMLInputElement;
      const label = document.getElementById(`val-${id}`) as HTMLElement;
      const rowLabel = input?.closest('.slider-row')?.querySelector('label');
      if (input && label) {
        input.addEventListener('pointerdown', (e: PointerEvent) => {
          this.activeScrub = {
            id,
            prop,
            input,
            label,
            format,
            isTemp: false,
            initialValue: this.edits[prop] as number,
            initialInputValue: input.value,
            startClientX: e.clientX,
            startInputVal: parseFloat(input.value),
            min: parseFloat(input.min || '-100'),
            max: parseFloat(input.max || '100'),
          };
        });

        input.addEventListener('pointermove', (e: PointerEvent) => {
          if (this.activeScrub && this.activeScrub.input === input && e.buttons === 1 && e.shiftKey) {
            const rect = input.getBoundingClientRect();
            const width = rect.width || 150;
            const deltaX = e.clientX - this.activeScrub.startClientX;
            const range = this.activeScrub.max - this.activeScrub.min;
            const fineVal = Math.min(
              this.activeScrub.max,
              Math.max(
                this.activeScrub.min,
                this.activeScrub.startInputVal + (deltaX / width) * range * 0.1
              )
            );
            input.value = fineVal.toString();
            (this.edits[prop] as number) = fineVal;
            label.innerText = format(fineVal);
            if ((prop as string).startsWith('split_toning_')) {
              this.updateSplitToningHint();
            }
            this.renderCanvas();
          }
        });

        input.addEventListener('input', (e: Event) => {
          if ((e as any).shiftKey) return;
          const val = parseFloat(input.value);
          (this.edits[prop] as number) = val;
          label.innerText = format(val);
          if ((prop as string).startsWith('split_toning_')) {
            this.updateSplitToningHint();
          }
          this.renderCanvas();
        });
        input.addEventListener('change', () => this.autoSaveSidecar());

        const resetFn = () => {
          const resolvedDefault = typeof defaultVal === 'function' ? defaultVal() : defaultVal;
          input.value = `${resolvedDefault}`;
          (this.edits[prop] as number) = resolvedDefault;
          label.innerText = format(resolvedDefault);
          if ((prop as string).startsWith('split_toning_')) {
            this.updateSplitToningHint();
          }
          this.renderCanvas();
          this.autoSaveSidecar();
        };

        input.addEventListener('dblclick', resetFn);
        rowLabel?.addEventListener('dblclick', resetFn);
      }
    };

    bindSlider('exposure', 'exposure', v => formatExposure(v), 0.0);
    bindSlider('contrast', 'contrast', v => `${Math.round(v)}`, 0.0);
    bindSlider('highlights', 'highlights', v => `${Math.round(v)}`, 0.0);
    bindSlider('shadows', 'shadows', v => `${Math.round(v)}`, 0.0);
    bindSlider('whites', 'whites', v => `${Math.round(v)}`, 0.0);
    bindSlider('blacks', 'blacks', v => `${Math.round(v)}`, 0.0);

    // Temperature Slider with Perceptual Mired Non-Linear Mapping
    const tempInput = document.getElementById('slider-temp') as HTMLInputElement;
    const tempLabel = document.getElementById('val-temp') as HTMLElement;
    const tempRowLabel = tempInput?.closest('.slider-row')?.querySelector('label');
    if (tempInput && tempLabel) {
      tempInput.addEventListener('pointerdown', (e: PointerEvent) => {
        this.activeScrub = {
          id: 'temp',
          input: tempInput,
          label: tempLabel,
          format: v => `${Math.round(v)} K`,
          isTemp: true,
          initialValue: this.edits.temperature,
          initialInputValue: tempInput.value,
          startClientX: e.clientX,
          startInputVal: parseFloat(tempInput.value),
          min: 0.0,
          max: 1.0,
        };
      });

      tempInput.addEventListener('pointermove', (e: PointerEvent) => {
        if (this.activeScrub && this.activeScrub.input === tempInput && e.buttons === 1 && e.shiftKey) {
          const rect = tempInput.getBoundingClientRect();
          const width = rect.width || 150;
          const deltaX = e.clientX - this.activeScrub.startClientX;
          const finePos = Math.min(
            1.0,
            Math.max(
              0.0,
              this.activeScrub.startInputVal + (deltaX / width) * 0.1
            )
          );
          tempInput.value = finePos.toFixed(4);
          const kelvin = this.miredPosToKelvin(finePos);
          this.edits.temperature = kelvin;
          tempLabel.innerText = `${Math.round(kelvin)} K`;
          if (this.elements['select-wb-preset']) {
            (this.elements['select-wb-preset'] as HTMLSelectElement).value = 'Custom';
          }
          this.renderCanvas();
        }
      });

      tempInput.addEventListener('input', (e: Event) => {
        if ((e as any).shiftKey) return;
        const pos = parseFloat(tempInput.value);
        const kelvin = this.miredPosToKelvin(pos);
        this.edits.temperature = kelvin;
        tempLabel.innerText = `${Math.round(kelvin)} K`;
        if (this.elements['select-wb-preset']) {
          (this.elements['select-wb-preset'] as HTMLSelectElement).value = 'Custom';
        }
        this.renderCanvas();
      });
      tempInput.addEventListener('change', () => this.autoSaveSidecar());

      const resetTemp = () => {
        this.edits.temperature = 5500.0;
        tempInput.value = this.kelvinToMiredPos(5500.0).toFixed(4);
        tempLabel.innerText = '5500 K';
        if (this.elements['select-wb-preset']) {
          (this.elements['select-wb-preset'] as HTMLSelectElement).value = 'Daylight';
        }
        this.renderCanvas();
        this.autoSaveSidecar();
      };

      tempInput.addEventListener('dblclick', resetTemp);
      tempRowLabel?.addEventListener('dblclick', resetTemp);
    }

    bindSlider('tint', 'tint', v => `${Math.round(v)}`, 0.0);
    bindSlider('vibrance', 'vibrance', v => `${Math.round(v)}`, 0.0);
    bindSlider('saturation', 'saturation', v => `${Math.round(v)}`, 0.0);
    bindSlider('deep-blue', 'deep_blue', v => `${Math.round(v)}`, 0.0);
    bindSlider('foliage', 'foliage', v => `${Math.round(v)}`, 0.0);
    bindSlider('emulsion-exp', 'emulsion_exposure', v => formatExposure(v), 0.0);
    bindSlider('paper-grade', 'print_paper_grade', v => `${v.toFixed(1)}`, 0.0);
    bindSlider('base-warmth', 'print_base_warmth', v => `${Math.round(v)}`, 0.0);
    bindSlider('process-depth', 'print_process_depth', v => `${Math.round(v)}%`, 100.0);
    bindSlider('sharpen-amount', 'sharpening_amount', v => `${Math.round(v)}`, () => this.isCurrentPhotoRaw() ? 20.0 : 0.0);
    bindSlider('sharpen-radius', 'sharpening_radius', v => `${v.toFixed(1)} px`, 1.0);
    bindSlider('sharpen-masking', 'sharpening_masking', v => `${Math.round(v)}`, () => this.isCurrentPhotoRaw() ? 15.0 : 0.0);
    bindSlider('noise-reduction', 'noise_reduction', v => `${Math.round(v)}`, 0.0);
    bindSlider('shadow-hue', 'split_toning_shadow_hue', v => `${Math.round(v)}°`, 215.0);
    bindSlider('shadow-sat', 'split_toning_shadow_sat', v => `${Math.round(v)}`, 0.0);
    bindSlider('highlight-hue', 'split_toning_highlight_hue', v => `${Math.round(v)}°`, 45.0);
    bindSlider('highlight-sat', 'split_toning_highlight_sat', v => `${Math.round(v)}`, 0.0);
    bindSlider('split-balance', 'split_toning_balance', v => `${Math.round(v)}`, 0.0);

    // Tone Profile Select
    this.elements['select-tone-profile']?.addEventListener('change', (e) => {
      this.edits.tone_profile = (e.target as HTMLSelectElement).value as ColorProfileMode;
      this.renderCanvas();
      this.autoSaveSidecar();
    });

    // Emulsion Select
    this.elements['select-emulsion']?.addEventListener('change', (e) => {
      this.edits.emulsion_archetype = (e.target as HTMLSelectElement).value as EmulsionArchetype;
      this.updateEmulsionWorkspaceVisibility();
      this.renderCanvas();
      this.autoSaveSidecar();
    });

    // Print Media Select
    this.elements['select-print-media']?.addEventListener('change', (e) => {
      this.edits.print_media = (e.target as HTMLSelectElement).value as PrintMediaMode;
      this.updateEmulsionWorkspaceVisibility();
      this.renderCanvas();
      this.autoSaveSidecar();
    });

    // Enlarger Filter Select
    this.elements['select-enlarger-filter']?.addEventListener('change', (e) => {
      this.edits.enlarger_filter = (e.target as HTMLSelectElement).value as any;
      this.renderCanvas();
      this.autoSaveSidecar();
    });

    // Curve Channel Buttons
    const channelBtns = document.querySelectorAll('.channel-btn');
    channelBtns.forEach(btn => {
      btn.addEventListener('click', (e) => {
        channelBtns.forEach(b => b.classList.remove('active'));
        const target = e.target as HTMLElement;
        target.classList.add('active');
        const channel = target.getAttribute('data-channel') as any;
        this.curveWidget.setChannel(channel);
      });
    });

    this.elements['btn-reset-curve'].addEventListener('click', () => {
      this.curveWidget.resetCurrentChannel();
    });

    // Segmented Workspace Tab Bar
    const tabBtns = document.querySelectorAll('.seg-btn');
    tabBtns.forEach(btn => {
      btn.addEventListener('click', () => {
        const tab = btn.getAttribute('data-tab');
        if (tab) this.selectInspectorTab(tab);
      });
    });

    // Reset All Edits Button
    this.elements['btn-reset-all']?.addEventListener('click', () => {
      this.resetAllEdits();
    });

    // Metadata text inputs auto-save
    const metaInputs = ['meta-input-title', 'meta-input-caption', 'meta-input-creator', 'meta-input-keywords'];
    for (const id of metaInputs) {
      if (this.elements[id]) {
        this.elements[id].addEventListener('change', () => this.autoSaveSidecar());
      }
    }

    // Global pointer release & cancellation for slider scrubbing
    window.addEventListener('pointerup', () => {
      if (this.activeScrub) {
        this.autoSaveSidecar();
        this.activeScrub = null;
      }
    });

    window.addEventListener('pointercancel', () => {
      if (this.activeScrub) {
        this.rollbackActiveScrub();
      }
    });

    // Keyboard Shortcuts filter with FIRST-RESPONDER TEXT FIELD SHIELDING
    window.addEventListener('keydown', (e: KeyboardEvent) => {
      // Escape key rolls back active scrubbing slider, active rotation, or exits active crop mode
      if (e.key === 'Escape') {
        if (this.cropController?.isRotating) {
          this.cropController.cancelRotation();
          e.preventDefault();
          e.stopPropagation();
          return;
        }
        if (this.activeScrub) {
          this.rollbackActiveScrub();
          e.preventDefault();
          e.stopPropagation();
          return;
        }
        if (this.isCropActive) {
          this.toggleCropMode(false);
          e.preventDefault();
          e.stopPropagation();
          return;
        }
      }

      const activeEl = document.activeElement;
      const isTextField = activeEl && (
        (activeEl.tagName === 'INPUT' && (activeEl as HTMLInputElement).type !== 'range') ||
        activeEl.tagName === 'TEXTAREA' ||
        (activeEl as HTMLElement).isContentEditable
      );
      if (isTextField) {
        return; // Guardrail: Never intercept single-key shortcuts while user is typing in metadata fields!
      }

      // Cmd+1..5 / Ctrl+1..5: Workspace Tab Navigation
      if ((e.metaKey || e.ctrlKey) && !e.shiftKey) {
        if (e.key === '1') { e.preventDefault(); this.selectInspectorTab('light'); return; }
        if (e.key === '2') { e.preventDefault(); this.selectInspectorTab('color'); return; }
        if (e.key === '3') { e.preventDefault(); this.selectInspectorTab('detail'); return; }
        if (e.key === '4') { e.preventDefault(); this.selectInspectorTab('emulsion'); return; }
        if (e.key === '5') { e.preventDefault(); this.selectInspectorTab('info'); return; }
      }

      // Cmd+Shift+R / Ctrl+Shift+R: Reset All Edits
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === 'r' || e.key === 'R')) {
        e.preventDefault();
        this.resetAllEdits();
        return;
      }

      // Cmd+U / Ctrl+U: Auto Tone calculation
      if ((e.metaKey || e.ctrlKey) && (e.key === 'u' || e.key === 'U')) {
        e.preventDefault();
        this.applyAutoTone();
        return;
      }

      // Cmd+E / Ctrl+E: Export Photo... (macOS Parity)
      if ((e.metaKey || e.ctrlKey) && !e.shiftKey && (e.key === 'e' || e.key === 'E')) {
        e.preventDefault();
        this.openExportModal();
        return;
      }

      switch (e.key) {
        case '1': case '2': case '3': case '4': case '5':
          this.currentRating = parseInt(e.key, 10);
          this.syncRatingFlagUI();
          this.autoSaveSidecar();
          break;
        case '0':
          this.currentRating = 0;
          this.syncRatingFlagUI();
          this.autoSaveSidecar();
          break;
        case 'p': case 'P':
          this.currentStatus = (this.currentStatus === 'Pick') ? 'Unflagged' : 'Pick';
          this.syncRatingFlagUI();
          this.autoSaveSidecar();
          break;
        case 'x': case 'X':
          if (this.isCropActive) {
            this.cropController.flipCropOrientation();
            return;
          }
          this.currentStatus = (this.currentStatus === 'Reject') ? 'Unflagged' : 'Reject';
          this.syncRatingFlagUI();
          this.autoSaveSidecar();
          break;
        case 'u': case 'U':
          this.currentStatus = 'Unflagged';
          this.syncRatingFlagUI();
          this.autoSaveSidecar();
          break;
        case 'Enter':
          if (this.isCropActive) {
            this.toggleCropMode(false);
          }
          break;
        case 'c': case 'C':
          this.toggleCropMode();
          break;
        case 'z': case 'Z':
          if (!this.isCropActive) {
            this.togglePixelPeep();
          }
          break;
        case '[':
          this.rotateImage(-90);
          break;
        case ']':
          this.rotateImage(90);
          break;
        case 'w': case 'W':
          this.toggleEyedropper('wb');
          break;
        case '\\':
          this.isOriginalPeek = true;
          this.renderCanvas();
          break;
        case 'j':
        case 'J':
          if (!e.repeat) {
            this.isCameraJpegPeek = true;
            this.renderCanvas();
          }
          break;
        case 'ArrowLeft':
        case 'ArrowUp':
          if (this.activeIdx > 0) {
            e.preventDefault();
            this.selectPhoto(this.activeIdx - 1, true);
          }
          break;
        case 'ArrowRight':
        case 'ArrowDown':
          if (this.activeIdx < this.fileEntries.length - 1) {
            e.preventDefault();
            this.selectPhoto(this.activeIdx + 1, true);
          }
          break;
      }
    });

    window.addEventListener('keyup', (e: KeyboardEvent) => {
      if (e.key === '\\') {
        this.isOriginalPeek = false;
        this.renderCanvas();
      } else if (e.key === 'j' || e.key === 'J') {
        this.isCameraJpegPeek = false;
        this.renderCanvas();
      }
    });

    window.addEventListener('blur', () => {
      if (this.isOriginalPeek || this.isCameraJpegPeek) {
        this.isOriginalPeek = false;
        this.isCameraJpegPeek = false;
        this.renderCanvas();
      }
    });
  }

  private onPhotosLoaded() {
    this.elements['empty-state'].classList.add('hidden');
    this.elements['filmstrip-count'].innerText = `${this.fileEntries.length} Photos Loaded`;

    if (this.dirHandle) {
      FolderIntake.saveRecentDirHandle(this.dirHandle);
    }

    // Revoke previous session Blob URLs to avoid memory leaks across folder intake
    for (const [, oldUrl] of this.thumbCache) {
      URL.revokeObjectURL(oldUrl);
    }
    this.thumbCache.clear();
    this.cardRotations.clear();

    // Render Filmstrip Cards
    const filmstripScroll = this.elements['filmstrip-scroll'];
    filmstripScroll.innerHTML = '';

    this.fileEntries.forEach((entry, idx) => {
      const card = document.createElement('div');
      card.className = `thumb-card ${idx === 0 ? 'active' : ''}`;
      card.setAttribute('data-idx', `${idx}`);
      card.id = `thumb-card-${idx}`;

      const iconContainer = document.createElement('div');
      iconContainer.className = 'thumb-icon-placeholder';
      iconContainer.innerHTML = `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/></svg>`;

      const title = document.createElement('div');
      title.className = 'thumb-title';
      title.innerText = entry.name;

      card.appendChild(iconContainer);
      card.appendChild(title);

      card.addEventListener('click', (e: MouseEvent) => {
        if (e.metaKey || e.ctrlKey) {
          if (this.selectedIndices.has(idx)) {
            if (this.selectedIndices.size > 1) {
              this.selectedIndices.delete(idx);
            }
          } else {
            this.selectedIndices.add(idx);
          }
          this.updateCardSelectionClasses();
          this.selectPhoto(idx, false, true);
        } else if (e.shiftKey && this.activeIdx >= 0) {
          const start = Math.min(this.activeIdx, idx);
          const end = Math.max(this.activeIdx, idx);
          this.selectedIndices.clear();
          for (let i = start; i <= end; i++) {
            this.selectedIndices.add(i);
          }
          this.updateCardSelectionClasses();
          this.selectPhoto(idx, false, true);
        } else {
          this.selectedIndices.clear();
          this.selectedIndices.add(idx);
          this.updateCardSelectionClasses();
          this.selectPhoto(idx, false, false);
        }
      });
      filmstripScroll.appendChild(card);

      // Fast background EXIF orientation scan from raw byte slice (sub-0.1ms per image)
      readExifOrientationFromBlob(entry.file).then(async (orientation) => {
        if (this.fileEntries[idx] !== entry) return;
        const exifOri = decodeExifOrientation(orientation);
        let rot = exifOri.rotation;
        let flipH = exifOri.flipH;
        let flipV = exifOri.flipV;

        const sidecarXml = await FolderIntake.readSidecarFromDisk(entry);
        if (this.fileEntries[idx] !== entry) return;
        if (sidecarXml) {
          try {
            const xmp = this.bridge.parseXmp(sidecarXml);
            if (xmp.edits?.rotation !== undefined && xmp.edits?.rotation !== null) {
              rot = xmp.edits.rotation;
              flipH = xmp.edits.flip_horizontal ?? false;
              flipV = xmp.edits.flip_vertical ?? false;
            }
          } catch (_) {}
        }
        this.updateThumbnailCardOrientation(idx, rot, flipH, flipV);
      });
    });

    // Eagerly begin loading thumbnails for the initial visible batch of cards
    const initialBatch = Math.min(this.fileEntries.length, 12);
    for (let i = 0; i < initialBatch; i++) {
      this.loadThumbnailForCard(i);
    }

    if (this.fileEntries.length > 0) {
      let initialIdx = 0;
      try {
        const savedName = localStorage.getItem('fp_recent_photo_name');
        if (savedName) {
          const foundIdx = this.fileEntries.findIndex(e => e.name === savedName);
          if (foundIdx >= 0) {
            initialIdx = foundIdx;
          }
        }
      } catch (_) {}
      this.selectPhoto(initialIdx, true);
    }
  }

  private centerActiveThumb(smooth: boolean = true) {
    const cardEl = document.getElementById(`thumb-card-${this.activeIdx}`);
    const container = this.elements['filmstrip-scroll'] as HTMLElement;
    if (cardEl && container) {
      const containerWidth = container.clientWidth;
      const cardOffsetLeft = cardEl.offsetLeft;
      const cardWidth = cardEl.offsetWidth;
      const targetScrollLeft = cardOffsetLeft - (containerWidth / 2) + (cardWidth / 2);

      container.scrollTo({
        left: Math.max(0, targetScrollLeft),
        behavior: smooth ? 'smooth' : 'auto'
      });
    }
  }

  private thumbCache: Map<number, string> = new Map(); // idx -> Blob URL
  private prewarmedCache: Map<number, {
    preview: PreviewResult;
    imageSlice: Uint8Array;
    imgBitmap: ImageBitmap;
    linearData?: Uint16Array;
    linearW?: number;
    linearH?: number;
  }> = new Map();

  private cacheThumbnail(idx: number, blobUrl: string) {
    this.thumbCache.set(idx, blobUrl);
  }

  private async prewarmPhoto(idx: number) {
    if (idx < 0 || idx >= this.fileEntries.length || this.prewarmedCache.has(idx) || this.prewarmLoadingSet.has(idx)) return;
    this.prewarmLoadingSet.add(idx);
    try {
      const entry = this.fileEntries[idx];
      const arrayBuffer = await entry.file.arrayBuffer();
      const res = await this.decodeInWorker(arrayBuffer, entry.name);
      const blob = new Blob([new Uint8Array(res.imageSlice)], { type: 'image/jpeg' });
      let imgBitmap: ImageBitmap;
      try {
        imgBitmap = await createImageBitmap(blob, { resizeWidth: 2560 });
      } catch (_) {
        imgBitmap = await createImageBitmap(blob);
      }

      if (this.prewarmedCache.size >= 4) {
        const firstKey = this.prewarmedCache.keys().next().value;
        if (firstKey !== undefined) {
          const old = this.prewarmedCache.get(firstKey);
          old?.imgBitmap.close();
          this.prewarmedCache.delete(firstKey);
        }
      }
      this.prewarmedCache.set(idx, {
        preview: res.preview,
        imageSlice: res.imageSlice,
        imgBitmap,
        linearData: res.linearData,
        linearW: res.linearW,
        linearH: res.linearH,
      });
    } catch (_) {
      // Ignore background pre-warming errors
    } finally {
      this.prewarmLoadingSet.delete(idx);
    }
  }

  private updateThumbnailCardOrientation(idx: number, rotation: number = 0, flipH: boolean = false, flipV: boolean = false) {
    this.cardRotations.set(idx, { rotation, flipH, flipV });
    const cardEl = document.getElementById(`thumb-card-${idx}`);
    if (!cardEl) return;
    const img = cardEl.querySelector('.thumb-img') as HTMLImageElement | null;
    if (!img) return;

    const rot = ((rotation % 360) + 360) % 360;
    const transforms: string[] = [];
    if (rot !== 0) transforms.push(`rotate(${rot}deg)`);
    if (rot === 90 || rot === 270) transforms.push('scale(0.6667)');
    if (flipH) transforms.push('scaleX(-1)');
    if (flipV) transforms.push('scaleY(-1)');

    img.style.transform = transforms.length > 0 ? transforms.join(' ') : 'none';
  }

  private async loadThumbnailForCard(idx: number) {
    if (idx < 0 || idx >= this.fileEntries.length) return;
    if (this.thumbCache.has(idx) || this.thumbLoadingSet.has(idx)) return;
    const cardEl = document.getElementById(`thumb-card-${idx}`);
    if (!cardEl || cardEl.querySelector('img')) return;

    this.thumbLoadingSet.add(idx);
    try {
      let previewUrl = this.thumbCache.get(idx);
      let cardRot = 0;
      let cardFlipH = false;
      let cardFlipV = false;

      const entry = this.fileEntries[idx];
      const cacheKey = `${entry.name}_${entry.file.size}_${entry.file.lastModified}`;

      if (!previewUrl) {
        const cached = await ThumbnailCacheStore.get(cacheKey);
        if (cached) {
          if (cached instanceof Blob) {
            previewUrl = URL.createObjectURL(cached);
          } else if ((cached as any).blob) {
            previewUrl = URL.createObjectURL((cached as any).blob);
            cardRot = (cached as any).rotation ?? 0;
            cardFlipH = (cached as any).flipH ?? false;
            cardFlipV = (cached as any).flipV ?? false;
          }
          if (previewUrl) {
            this.cacheThumbnail(idx, previewUrl);
          }
        }
      }

      if (!previewUrl) {
        const arrayBuffer = await entry.file.arrayBuffer();
        const bytes = new Uint8Array(arrayBuffer);
        const { imageSlice, preview } = this.bridge.decodeImageBytes(bytes);
        const previewBlob = new Blob([new Uint8Array(imageSlice)], { type: 'image/jpeg' });
        previewUrl = URL.createObjectURL(previewBlob);
        this.cacheThumbnail(idx, previewUrl);

        const orientation = preview.metadata?.orientation ?? 1;
        const exifOri = decodeExifOrientation(orientation);
        cardRot = exifOri.rotation;
        cardFlipH = exifOri.flipH;
        cardFlipV = exifOri.flipV;

        // Check if adjacent XMP sidecar has custom rotation
        const sidecarXml = await FolderIntake.readSidecarFromDisk(entry);
        if (sidecarXml) {
          try {
            const xmp = this.bridge.parseXmp(sidecarXml);
            if (xmp.edits?.rotation !== undefined && xmp.edits?.rotation !== null) {
              cardRot = xmp.edits.rotation;
              cardFlipH = xmp.edits.flip_horizontal ?? false;
              cardFlipV = xmp.edits.flip_vertical ?? false;
            }
          } catch (_) {}
        }

        await ThumbnailCacheStore.set(cacheKey, {
          blob: previewBlob,
          rotation: cardRot,
          flipH: cardFlipH,
          flipV: cardFlipV,
        });
      } else if (!this.cardRotations.has(idx)) {
        // If we only had a cached Blob without rotation, read fast EXIF orientation and sidecar
        try {
          const orientation = await readExifOrientationFromBlob(entry.file);
          const exifOri = decodeExifOrientation(orientation);
          cardRot = exifOri.rotation;
          cardFlipH = exifOri.flipH;
          cardFlipV = exifOri.flipV;
        } catch (_) {}

        const sidecarXml = await FolderIntake.readSidecarFromDisk(entry);
        if (sidecarXml) {
          try {
            const xmp = this.bridge.parseXmp(sidecarXml);
            if (xmp.edits?.rotation !== undefined && xmp.edits?.rotation !== null) {
              cardRot = xmp.edits.rotation;
              cardFlipH = xmp.edits.flip_horizontal ?? false;
              cardFlipV = xmp.edits.flip_vertical ?? false;
            }
          } catch (_) {}
        }
      }

      const existingImg = cardEl.querySelector('img') as HTMLImageElement | null;
      if (existingImg) {
        if (existingImg.src !== previewUrl) existingImg.src = previewUrl;
      } else {
        const img = document.createElement('img');
        img.className = 'thumb-img';
        img.src = previewUrl;
        const placeholder = cardEl.querySelector('.thumb-icon-placeholder');
        if (placeholder) placeholder.remove();
        cardEl.insertBefore(img, cardEl.firstChild);
      }

      const knownRot = this.cardRotations.get(idx);
      if (knownRot) {
        this.updateThumbnailCardOrientation(idx, knownRot.rotation, knownRot.flipH, knownRot.flipV);
      } else {
        this.updateThumbnailCardOrientation(idx, cardRot, cardFlipH, cardFlipV);
      }
    } catch (err) {
      console.warn(`[Thumbnail] Could not extract preview thumbnail for ${this.fileEntries[idx].name}:`, err);
    } finally {
      this.thumbLoadingSet.delete(idx);
    }
  }

  private preloadAdjacentThumbnails(centerIdx: number, requestId: number) {
    if (this.preloadDebounceTimer) {
      clearTimeout(this.preloadDebounceTimer);
    }
    // Schedule background outward loading after user stays on a frame for 100ms
    this.preloadDebounceTimer = setTimeout(async () => {
      if (requestId !== this.selectRequestId) return;

      // Radially load thumbnail cards outward up to 12 frames in both directions (covers 4K/Ultrawide viewports)
      for (let dist = 0; dist <= 12; dist++) {
        const offsets = dist === 0 ? [0] : [dist, -dist];
        for (const offset of offsets) {
          if (requestId !== this.selectRequestId) return;
          const idx = centerIdx + offset;
          if (idx >= 0 && idx < this.fileEntries.length) {
            await this.loadThumbnailForCard(idx);
          }
        }
      }

      // Prewarm high-res WebGL bitmaps for immediate neighbor frames
      for (const offset of [1, -1, 2, -2]) {
        if (requestId !== this.selectRequestId) return;
        const idx = centerIdx + offset;
        if (idx >= 0 && idx < this.fileEntries.length) {
          await this.prewarmPhoto(idx);
        }
      }
    }, 100);
  }

  private updateCardSelectionClasses() {
    const cards = document.querySelectorAll('.thumb-card');
    cards.forEach((c, i) => {
      c.classList.toggle('active', i === this.activeIdx);
      c.classList.toggle('selected', this.selectedIndices.has(i));
    });
  }

  public getSelectedPhotos(): PhotoFileEntry[] {
    if (this.selectedIndices.size > 1) {
      const sorted = Array.from(this.selectedIndices).sort((a, b) => a - b);
      return sorted.map(i => this.fileEntries[i]).filter(Boolean);
    }
    if (this.activeIdx >= 0 && this.activeIdx < this.fileEntries.length) {
      return [this.fileEntries[this.activeIdx]];
    }
    return [];
  }

  private async selectPhoto(idx: number, centerFilmstrip: boolean = false, keepMultiSelection: boolean = false) {
    if (idx < 0 || idx >= this.fileEntries.length) return;

    const requestId = ++this.selectRequestId;
    this.activeIdx = idx;
    this.activeScrub = null;

    if (!keepMultiSelection) {
      this.selectedIndices.clear();
      this.selectedIndices.add(idx);
    }

    // 1. INSTANT UI updates (0ms delay!)
    this.updateCardSelectionClasses();

    if (centerFilmstrip) {
      this.centerActiveThumb(true);
    }

    const entry = this.fileEntries[idx];
    this.elements['meta-filename'].innerText = entry.name;
    try {
      localStorage.setItem('fp_recent_photo_name', entry.name);
    } catch (_) {}

    if (this.selectDebounceTimer) {
      clearTimeout(this.selectDebounceTimer);
      this.selectDebounceTimer = null;
    }

    const isPrewarmedHit = this.prewarmedCache.has(idx);
    const delay = isPrewarmedHit ? 0 : 35; // 35ms debounce for un-cached heavy WASM extractions

    this.selectDebounceTimer = setTimeout(async () => {
      if (requestId !== this.selectRequestId) return; // Dropped by newer selection!

      this.elements['developing-spinner'].classList.remove('hidden');

      try {
        const t0 = performance.now();
        let preview: PreviewResult;
        let imageSlice: Uint8Array;
        let imgBitmap: ImageBitmap;
        let t1 = t0, t2 = t0, t3 = t0;

        let fileBytes: Uint8Array | null = null;
        let workerLinearData: Uint16Array | null = null;
        let workerLinearW = 0;
        let workerLinearH = 0;

        const prewarmed = this.prewarmedCache.get(idx);
        if (prewarmed) {
          if (requestId !== this.selectRequestId) return;
          preview = prewarmed.preview;
          imageSlice = prewarmed.imageSlice;
          imgBitmap = prewarmed.imgBitmap;
          workerLinearData = prewarmed.linearData || null;
          workerLinearW = prewarmed.linearW || 0;
          workerLinearH = prewarmed.linearH || 0;
          t1 = performance.now();
          t2 = t1;
          t3 = t1;
        } else {
          // 1. Read single selected photo file bytes into memory
          const arrayBuffer = await entry.file.arrayBuffer();
          if (requestId !== this.selectRequestId) return; // Dropped while reading file!
          fileBytes = new Uint8Array(arrayBuffer);

          t1 = performance.now();

          // 2. WASM preview extraction & metadata off-main-thread via RawDecodeWorker pool
          const res = await this.decodeInWorker(arrayBuffer, entry.name);
          if (requestId !== this.selectRequestId) return; // Dropped after WASM extraction!

          preview = res.preview;
          imageSlice = res.imageSlice;
          workerLinearData = res.linearData || null;
          workerLinearW = res.linearW || 0;
          workerLinearH = res.linearH || 0;
          t2 = performance.now();

          // 3. Create ImageBitmap for WebGL upload
          const blob = new Blob([new Uint8Array(imageSlice)], { type: 'image/jpeg' });
          try {
            imgBitmap = await createImageBitmap(blob, { resizeQuality: 'high' });
          } catch (decodeErr) {
            console.warn(`[App] createImageBitmap failed for ${entry.name}, creating fallback canvas:`, decodeErr);
            const fallbackCanvas = document.createElement('canvas');
            fallbackCanvas.width = (preview.width && preview.width > 0) ? preview.width : 1920;
            fallbackCanvas.height = (preview.height && preview.height > 0) ? preview.height : 1080;
            const ctx = fallbackCanvas.getContext('2d');
            if (ctx) {
              ctx.fillStyle = '#111318';
              ctx.fillRect(0, 0, fallbackCanvas.width, fallbackCanvas.height);
              ctx.fillStyle = '#f1f5f9';
              ctx.font = 'bold 24px sans-serif';
              ctx.fillText(entry.name, 40, 60);
            }
            imgBitmap = await createImageBitmap(fallbackCanvas);
          }
          if (requestId !== this.selectRequestId) return; // Dropped after bitmap creation!
          t3 = performance.now();
        }

        console.log(`[DNG Analysis] ${entry.name}: Extracted embedded preview dimensions ${preview.width}×${preview.height} (offset 0x${preview.preview_offset.toString(16)}, ${(preview.preview_len / 1024).toFixed(1)} KB)`);
        if (preview.width <= 640 || preview.height <= 480) {
          console.warn(`⚠️ [DNG Warning] ${entry.name} contains only a ${preview.width}×${preview.height} thumbnail (Minimum Preview mode). No full-size JPEG was embedded when converting/saving this DNG.`);
        }

        if (requestId !== this.selectRequestId) return;

        this.currentMetadata = preview.metadata;

        // Update EXIF metadata header (permanent across all tabs)
        this.elements['meta-dimensions'].innerText = formatDimensions(
          preview.width,
          preview.height,
          preview.metadata.raw_width,
          preview.metadata.raw_height
        );
        this.elements['meta-camera'].innerText = `${preview.metadata.camera_model || 'Bayer Raw'}`;
        this.elements['meta-iso'].innerText = formatISO(preview.metadata.iso);
        this.elements['meta-aperture'].innerText = formatAperture(preview.metadata.aperture);
        this.elements['meta-shutter'].innerText = formatShutterSpeed(preview.metadata.shutter_speed);

        // Update format badge & rich hardware metadata for Info tab
        const extBadge = entry.name.split('.').pop()?.toUpperCase() || 'RAW';
        if (this.elements['meta-format-badge']) {
          this.elements['meta-format-badge'].innerText = extBadge;
        }

        const make = preview.metadata.camera_make || '';
        const model = preview.metadata.camera_model || '';
        const fullCamera = `${make} ${model}`.trim() || preview.metadata.camera_model || 'Bayer Sensor';
        if (this.elements['meta-full-camera']) {
          this.elements['meta-full-camera'].innerText = fullCamera;
        }

        if (this.elements['meta-lens']) {
          this.elements['meta-lens'].innerText = preview.metadata.lens_model || '—';
        }

        if (this.elements['meta-focal-length']) {
          this.elements['meta-focal-length'].innerText = formatFocalLength(preview.metadata.focal_length);
        }

        if (this.elements['meta-exposure-summary']) {
          this.elements['meta-exposure-summary'].innerText = `${formatShutterSpeed(preview.metadata.shutter_speed)} • ${formatAperture(preview.metadata.aperture)} • ISO ${formatISO(preview.metadata.iso)}`;
        }

        if (this.elements['meta-sensor-res']) {
          this.elements['meta-sensor-res'].innerText = formatDimensions(
            preview.width,
            preview.height,
            preview.metadata.raw_width,
            preview.metadata.raw_height
          );
        }

        if (this.elements['meta-date-taken']) {
          let dateStr = '—';
          if (preview.metadata.taken_time && preview.metadata.taken_time > 0) {
            dateStr = new Date(preview.metadata.taken_time * 1000).toLocaleString(undefined, {
              dateStyle: 'medium',
              timeStyle: 'short',
            });
          }
          this.elements['meta-date-taken'].innerText = dateStr;
        }

        const orientation = preview.metadata?.orientation ?? 1;
        const exifOri = decodeExifOrientation(orientation);

        // Update filmstrip thumbnail
        const cardEl = document.getElementById(`thumb-card-${idx}`);
        if (cardEl) {
          let previewUrl = this.thumbCache.get(idx);
          if (!previewUrl) {
            const previewBlob = new Blob([new Uint8Array(imageSlice)], { type: 'image/jpeg' });
            previewUrl = URL.createObjectURL(previewBlob);
            this.cacheThumbnail(idx, previewUrl);
            const cacheKey = `${entry.name}_${entry.file.size}_${entry.file.lastModified}`;
            ThumbnailCacheStore.set(cacheKey, {
              blob: previewBlob,
              rotation: exifOri.rotation,
              flipH: exifOri.flipH,
              flipV: exifOri.flipV,
            });
          }

          const existingImg = cardEl.querySelector('img') as HTMLImageElement | null;
          if (existingImg) {
            if (existingImg.src !== previewUrl) existingImg.src = previewUrl;
          } else {
            const img = document.createElement('img');
            img.className = 'thumb-img';
            img.src = previewUrl;
            const placeholder = cardEl.querySelector('.thumb-icon-placeholder');
            if (placeholder) placeholder.remove();
            cardEl.insertBefore(img, cardEl.firstChild);
          }
        }

        // Check for adjacent .xmp sidecar on disk, or restore from localStorage session backup
        let sidecarXml = await FolderIntake.readSidecarFromDisk(entry);
        if (!sidecarXml) {
          try {
            sidecarXml = localStorage.getItem(`fp_xmp_${entry.name}`);
          } catch (_) {}
        }
        if (requestId !== this.selectRequestId) return; // Dropped after sidecar read!

        if (this.elements['meta-sidecar-status']) {
          this.elements['meta-sidecar-status'].innerText = sidecarXml ? '.xmp Synced' : 'Session RAM';
          this.elements['meta-sidecar-status'].style.color = sidecarXml ? '#4ade80' : 'var(--text-muted)';
        }

        const ext = entry.name.split('.').pop()?.toLowerCase() || '';
        const isRawFile = ['cr2', 'cr3', 'nef', 'arw', 'dng', 'rw2', 'orf', 'pef', 'raf'].includes(ext);

        if (sidecarXml) {
          const xmp = this.bridge.parseXmp(sidecarXml);
          this.edits = { ...this.getDefaultEdits(isRawFile), ...xmp.edits };
          if (this.edits.rotation === undefined || this.edits.rotation === null) {
            this.edits.rotation = exifOri.rotation;
            this.edits.flip_horizontal = exifOri.flipH;
            this.edits.flip_vertical = exifOri.flipV;
          }
          this.currentRating = xmp.rating || 0;
          this.currentStatus = xmp.status || 'Unflagged';
          (this.elements['meta-input-title'] as HTMLInputElement).value = xmp.title || '';
          (this.elements['meta-input-caption'] as HTMLTextAreaElement).value = xmp.description || '';
          (this.elements['meta-input-creator'] as HTMLInputElement).value = xmp.creator || '';
          (this.elements['meta-input-keywords'] as HTMLInputElement).value = (xmp.keywords || []).join(', ');
        } else {
          this.edits = this.getDefaultEdits(isRawFile);
          this.edits.rotation = exifOri.rotation;
          this.edits.flip_horizontal = exifOri.flipH;
          this.edits.flip_vertical = exifOri.flipV;
          this.currentRating = 0;
          this.currentStatus = 'Unflagged';
        }

        // Apply updated orientation to thumbnail card in filmstrip
        this.updateThumbnailCardOrientation(idx, this.edits.rotation, this.edits.flip_horizontal, this.edits.flip_vertical);

        if (this.isCropActive) {
          this.cropController.hide();
          this.isCropActive = false;
          this.elements['btn-crop-tool']?.classList.remove('active');
        }
        this.zoomScale = 1.0;
        this.panOffset = { x: 0, y: 0 };
        this.updateZoomUI();

        this.syncSliderUI();
        this.syncRatingFlagUI();

        // Check if raw negative and extract uncompressed 16-bit Linear Rec.2020 sensor frame

        let linear16Data: Uint16Array | null = workerLinearData;
        let linearW = workerLinearW;
        let linearH = workerLinearH;

        if (isRawFile && (!linear16Data || linearW === 0 || linearH === 0)) {
          try {
            const isDetached = fileBytes ? ((fileBytes.buffer as any).detached === true || fileBytes.byteLength === 0) : true;
            const rawBytes = (!isDetached && fileBytes) ? fileBytes : new Uint8Array(await entry.file.arrayBuffer());
            const rawLin = this.bridge.decodeRawLinear(rawBytes);
            linear16Data = rawLin.linearData;
            linearW = rawLin.res.width;
            linearH = rawLin.res.height;
            console.log(`⚡ [RAW Success (Main Fallback)] Decoded uncompressed 16-bit linear RAW frame for ${entry.name}: ${linearW}×${linearH} (${(linear16Data.byteLength / 1024 / 1024).toFixed(1)} MB)`);
          } catch (linErr) {
            console.error(`[App] Linear Rec.2020 RAW decode failed for ${entry.name}:`, linErr);
          }
        } else if (linear16Data && linearW > 0 && linearH > 0) {
          console.log(`⚡ [RAW Worker Success] Received 16-bit linear RAW frame off-thread for ${entry.name}: ${linearW}×${linearH} (${(linear16Data.byteLength / 1024 / 1024).toFixed(1)} MB)`);
        }

        // Upload to WebGL & render (preserving exact photographic aspect ratio up to 3840px for 4K displays)
        const canvas = document.getElementById('darkroom-canvas') as HTMLCanvasElement;
        let imgW = 0;
        let imgH = 0;
        if (linear16Data && linearW > 0 && linearH > 0) {
          this.activeLinearData = linear16Data;
          this.activeLinearW = linearW;
          this.activeLinearH = linearH;
          this.activeSourceImage = null;
          this.activeSourceW = 0;
          this.activeSourceH = 0;
          imgW = linearW;
          imgH = linearH;
          this.renderer.uploadLinearRec2020Texture(linearW, linearH, linear16Data);
        } else {
          this.activeLinearData = null;
          this.activeLinearW = 0;
          this.activeLinearH = 0;
          this.activeSourceImage = imgBitmap;
          this.activeSourceW = imgBitmap.width;
          this.activeSourceH = imgBitmap.height;
          imgW = imgBitmap.width;
          imgH = imgBitmap.height;
          this.renderer.uploadImage(imgBitmap);
        }

        if (imgBitmap) {
          this.renderer.uploadCameraJpeg(imgBitmap);
        }

        this.updateCanvasDimensions();
        this.onCurveChange(); // evaluate LUTs and render canvas
        const t4 = performance.now();

        const mb = (entry.file.size / (1024 * 1024)).toFixed(1);
        if (isPrewarmedHit) {
          console.log(`⚡ [Prewarm Hit] ${entry.name} (${mb} MB): served instantly from background lookahead cache in ${(t4-t0).toFixed(1)}ms!`);
        } else {
          console.log(`⚡ [Perf] ${entry.name} (${mb} MB): readDisk=${(t1-t0).toFixed(1)}ms | wasmExtract=${(t2-t1).toFixed(1)}ms | bitmapDecode=${(t3-t2).toFixed(1)}ms | webglRender=${(t4-t3).toFixed(1)}ms | TOTAL=${(t4-t0).toFixed(1)}ms`);
        }

        // 6. Schedule background lookahead pre-warming for adjacent frames ONLY after settling on the active photo
        this.preloadAdjacentThumbnails(idx, requestId);

      } catch (err) {
        if (requestId === this.selectRequestId) {
          console.error(`[App] Error developing photo ${entry.name}:`, err);
        }
      } finally {
        if (requestId === this.selectRequestId) {
          this.elements['developing-spinner'].classList.add('hidden');
        }
      }
    }, delay);
  }

  private onCurveChange() {
    this.edits.curves = this.curveWidget.getPoints();
    const lut = this.bridge.evaluateCurves(this.edits.curves);
    this.renderer.updateToneCurvesLut(lut);
    this.renderCanvas();
  }

  public selectInspectorTab(tab: string) {
    const tabBtns = document.querySelectorAll('.seg-btn');
    tabBtns.forEach(b => {
      const isMatch = b.getAttribute('data-tab') === tab;
      b.classList.toggle('active', isMatch);
      b.setAttribute('aria-selected', isMatch ? 'true' : 'false');
    });

    const panels = document.querySelectorAll('.workspace-panel');
    panels.forEach(p => {
      p.classList.toggle('hidden', p.id !== `tab-${tab}`);
    });
  }

  public resetAllEdits() {
    const orientation = this.currentMetadata?.orientation ?? 1;
    const exifOri = decodeExifOrientation(orientation);
    const isRaw = this.isCurrentPhotoRaw();
    this.edits = this.getDefaultEdits(isRaw);
    this.edits.rotation = exifOri.rotation;
    this.edits.flip_horizontal = exifOri.flipH;
    this.edits.flip_vertical = exifOri.flipV;
    if (this.activeIdx >= 0) {
      this.updateThumbnailCardOrientation(this.activeIdx, exifOri.rotation, exifOri.flipH, exifOri.flipV);
    }
    if (this.isCropActive) {
      this.cropController.resetCrop();
    }
    this.zoomScale = 1.0;
    this.panOffset = { x: 0, y: 0 };
    this.updateZoomUI();
    this.curveWidget.resetAllChannels();
    this.syncSliderUI();
    this.updateCanvasDimensions();
    this.renderCanvas();
    this.autoSaveSidecar();
    console.log(`⚡ [App] Reset all edits to as-shot defaults (isRaw: ${isRaw}, sharpening: ${this.edits.sharpening_amount})`);
  }

  private updateCanvasDimensions() {
    const canvas = document.getElementById('darkroom-canvas') as HTMLCanvasElement;
    const wrapper = document.getElementById('canvas-wrapper') as HTMLElement;
    if (!canvas || !wrapper) return;
    const dpr = window.devicePixelRatio || 1;
    const w = Math.max(1, Math.round(wrapper.clientWidth * dpr));
    const h = Math.max(1, Math.round(wrapper.clientHeight * dpr));
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w;
      canvas.height = h;
    }
  }

  public rotateImage(deltaDeg: number) {
    if (this.activeIdx < 0) return;
    const curRot = this.edits.rotation ?? 0;
    let newRot = (curRot + deltaDeg) % 360;
    if (newRot < 0) newRot += 360;
    this.edits.rotation = newRot;
    this.updateThumbnailCardOrientation(this.activeIdx, newRot, this.edits.flip_horizontal, this.edits.flip_vertical);
    this.updateCanvasDimensions();
    this.renderCanvas();
    this.autoSaveSidecar();
    if (this.isCropActive) {
      this.updateCropOverlayPosition();
    }
    console.log(`⚡ [App] Rotated photo to ${newRot}°`);
  }

  private getZoomScale100(): number {
    const wrapper = document.getElementById('canvas-wrapper') as HTMLElement;
    const isOrthogonal = (this.edits.rotation === 90 || this.edits.rotation === 270);
    const sensorW = isOrthogonal
      ? (this.currentMetadata?.raw_height || this.activeLinearH || this.activeSourceH || 1920)
      : (this.currentMetadata?.raw_width || this.activeLinearW || this.activeSourceW || 1920);
    const sensorH = isOrthogonal
      ? (this.currentMetadata?.raw_width || this.activeLinearW || this.activeSourceW || 1920)
      : (this.currentMetadata?.raw_height || this.activeLinearH || this.activeSourceH || 1080);
    const imgAspect = sensorW / Math.max(sensorH, 1);

    const viewW = wrapper?.clientWidth || 800;
    const viewH = wrapper?.clientHeight || 600;
    const viewAspect = viewW / Math.max(viewH, 1);

    let baseQuadW: number;
    if (viewAspect > imgAspect) {
      baseQuadW = viewH * imgAspect;
    } else {
      baseQuadW = viewW;
    }

    const scale = sensorW / Math.max(baseQuadW, 1);
    return Math.max(1.5, Math.min(10.0, scale));
  }

  public togglePixelPeep(clientX?: number, clientY?: number) {
    if (this.activeIdx < 0 || this.isCropActive) return;
    const wrapper = document.getElementById('canvas-wrapper') as HTMLElement;
    if (!wrapper) return;

    if (this.zoomScale > 1.05) {
      // Return to fit
      this.zoomScale = 1.0;
      this.panOffset = { x: 0, y: 0 };
    } else {
      // Zoom 100% 1:1 sensor view
      const targetScale = this.getZoomScale100();
      this.zoomScale = targetScale;

      if (clientX !== undefined && clientY !== undefined) {
        const rect = wrapper.getBoundingClientRect();
        const ndcX = ((clientX - rect.left) / rect.width) * 2.0 - 1.0;
        const ndcY = -(((clientY - rect.top) / rect.height) * 2.0 - 1.0);
        this.panOffset = {
          x: ndcX * (1.0 - targetScale),
          y: ndcY * (1.0 - targetScale)
        };
        this.clampPanOffset();
      } else {
        this.panOffset = { x: 0, y: 0 };
      }
    }

    this.updateZoomUI();
    this.renderCanvas();
  }

  private clampPanOffset() {
    const wrapper = document.getElementById('canvas-wrapper') as HTMLElement;
    const isOrthogonal = (this.edits.rotation === 90 || this.edits.rotation === 270);
    const origW = isOrthogonal ? (this.activeLinearH || this.activeSourceH || 1920) : (this.activeLinearW || this.activeSourceW || 1920);
    const origH = isOrthogonal ? (this.activeLinearW || this.activeSourceW || 1920) : (this.activeLinearH || this.activeSourceH || 1080);
    const cL = this.isCropActive ? 0.0 : (this.edits.crop_left ?? 0.0);
    const cT = this.isCropActive ? 0.0 : (this.edits.crop_top ?? 0.0);
    const cR = this.isCropActive ? 1.0 : (this.edits.crop_right ?? 1.0);
    const cB = this.isCropActive ? 1.0 : (this.edits.crop_bottom ?? 1.0);
    const dispW = Math.max(1, origW * (cR - cL));
    const dispH = Math.max(1, origH * (cB - cT));
    const imgAspect = dispW / dispH;

    const viewW = wrapper?.clientWidth || 800;
    const viewH = wrapper?.clientHeight || 600;
    const viewAspect = viewW / Math.max(viewH, 1);

    let fitScaleX = 1.0;
    let fitScaleY = 1.0;
    if (viewAspect > imgAspect) {
      fitScaleY = 1.0;
      fitScaleX = (viewH * imgAspect) / Math.max(viewW, 1);
    } else {
      fitScaleX = 1.0;
      fitScaleY = (viewW / imgAspect) / Math.max(viewH, 1);
    }

    const effW = fitScaleX * this.zoomScale;
    const effH = fitScaleY * this.zoomScale;
    const maxPanX = Math.max(0, effW - 1.0);
    const maxPanY = Math.max(0, effH - 1.0);

    this.panOffset.x = Math.max(-maxPanX, Math.min(maxPanX, this.panOffset.x));
    this.panOffset.y = Math.max(-maxPanY, Math.min(maxPanY, this.panOffset.y));
  }

  private updateZoomUI() {
    const badge = this.elements['zoom-badge'];
    const canvas = document.getElementById('darkroom-canvas') as HTMLCanvasElement;
    if (badge) {
      if (this.zoomScale <= 1.05) {
        badge.innerText = 'Fit';
        badge.classList.remove('zoomed');
      } else {
        badge.innerText = `${Math.round(this.zoomScale * 100)}%`;
        badge.classList.add('zoomed');
      }
    }
    if (canvas) {
      canvas.style.cursor = this.zoomScale > 1.05 ? 'grab' : 'default';
    }
  }

  public toggleCropMode(force?: boolean) {
    if (this.activeIdx < 0) return;
    const nextState = force !== undefined ? force : !this.isCropActive;
    if (this.isCropActive === nextState) return;
    this.isCropActive = nextState;

    const btnCrop = this.elements['btn-crop-tool'];
    btnCrop?.classList.toggle('active', this.isCropActive);

    if (this.isCropActive) {
      // Reset zoom to fit during crop mode
      this.zoomScale = 1.0;
      this.panOffset = { x: 0, y: 0 };
      this.updateZoomUI();

      this.updateCanvasDimensions();
      this.renderCanvas();

      requestAnimationFrame(() => {
        this.updateCropOverlayPosition();
        const isOrthogonal = (this.edits.rotation === 90 || this.edits.rotation === 270);
        const w = isOrthogonal ? (this.activeLinearH || this.activeSourceH || 1) : (this.activeLinearW || this.activeSourceW || 1);
        const h = isOrthogonal ? (this.activeLinearW || this.activeSourceW || 1) : (this.activeLinearH || this.activeSourceH || 1);
        const aspect = w / h;
        this.cropController.show({
          left: this.edits.crop_left ?? 0.0,
          top: this.edits.crop_top ?? 0.0,
          right: this.edits.crop_right ?? 1.0,
          bottom: this.edits.crop_bottom ?? 1.0,
        }, aspect, this.edits.crop_angle ?? 0.0);
      });
    } else {
      this.cropController.hide();
      this.updateCanvasDimensions();
      this.renderCanvas();
      this.autoSaveSidecar();
    }
  }

  private updateCropOverlayPosition() {
    const wrapper = document.getElementById('canvas-wrapper') as HTMLElement;
    if (!wrapper) return;

    let imgW = this.activeLinearW || this.activeSourceW || 1920;
    let imgH = this.activeLinearH || this.activeSourceH || 1080;
    const isOrthogonal = (this.edits.rotation === 90 || this.edits.rotation === 270);
    const dispW = isOrthogonal ? imgH : imgW;
    const dispH = isOrthogonal ? imgW : imgH;
    const imgAspect = dispW / Math.max(dispH, 1);

    const viewW = wrapper.clientWidth;
    const viewH = wrapper.clientHeight;
    const viewAspect = viewW / Math.max(viewH, 1);

    let renderW: number;
    let renderH: number;
    if (viewAspect > imgAspect) {
      renderH = viewH;
      renderW = renderH * imgAspect;
    } else {
      renderW = viewW;
      renderH = renderW / imgAspect;
    }

    const left = (viewW - renderW) / 2.0;
    const top = (viewH - renderH) / 2.0;

    this.cropController.setImageRect({ x: left, y: top, width: renderW, height: renderH }, imgAspect);
  }

  private renderCanvas() {
    if (this.activeIdx >= 0) {
      this.renderer.render(
        this.edits,
        this.isOriginalPeek,
        this.zoomScale,
        this.panOffset,
        this.isCropActive,
        this.isCameraJpegPeek
      );
      this.updatePeekBadge();
    }
  }

  private updatePeekBadge() {
    const badge = this.elements['peek-badge'];
    if (!badge) return;
    if (this.isCameraJpegPeek) {
      badge.textContent = 'Camera JPEG (J)';
      badge.className = 'peek-badge camera-jpeg';
    } else if (this.isOriginalPeek) {
      badge.textContent = 'Original Negative (\\)';
      badge.className = 'peek-badge original-negative';
    } else {
      badge.className = 'peek-badge hidden';
    }
  }

  private syncSliderUI() {
    const setVal = (id: string, val: number, format: (v: number) => string) => {
      const input = document.getElementById(`slider-${id}`) as HTMLInputElement;
      const label = document.getElementById(`val-${id}`) as HTMLElement;
      if (input && label) {
        input.value = `${val}`;
        label.innerText = format(val);
      }
    };

    setVal('exposure', this.edits.exposure, v => `${v > 0 ? '+' : ''}${v.toFixed(2)} EV`);
    setVal('contrast', this.edits.contrast, v => `${Math.round(v)}`);
    setVal('highlights', this.edits.highlights, v => `${Math.round(v)}`);
    setVal('shadows', this.edits.shadows, v => `${Math.round(v)}`);
    setVal('whites', this.edits.whites, v => `${Math.round(v)}`);
    setVal('blacks', this.edits.blacks, v => `${Math.round(v)}`);
    const tempInput = document.getElementById('slider-temp') as HTMLInputElement;
    const tempLabel = document.getElementById('val-temp') as HTMLElement;
    if (tempInput && tempLabel) {
      tempInput.value = this.kelvinToMiredPos(this.edits.temperature).toFixed(4);
      tempLabel.innerText = `${Math.round(this.edits.temperature)} K`;
    }

    setVal('tint', this.edits.tint, v => `${Math.round(v)}`);
    setVal('vibrance', this.edits.vibrance, v => `${Math.round(v)}`);
    setVal('saturation', this.edits.saturation, v => `${Math.round(v)}`);
    setVal('deep-blue', this.edits.deep_blue, v => `${Math.round(v)}`);
    setVal('foliage', this.edits.foliage, v => `${Math.round(v)}`);
    setVal('emulsion-exp', this.edits.emulsion_exposure, v => `${v > 0 ? '+' : ''}${v.toFixed(2)} EV`);
    setVal('paper-grade', this.edits.print_paper_grade, v => `${v.toFixed(1)}`);
    setVal('base-warmth', this.edits.print_base_warmth, v => `${Math.round(v)}`);
    setVal('process-depth', this.edits.print_process_depth, v => `${Math.round(v)}%`);
    setVal('sharpen-amount', this.edits.sharpening_amount, v => `${Math.round(v)}`);
    setVal('sharpen-radius', this.edits.sharpening_radius, v => `${v.toFixed(1)} px`);
    setVal('sharpen-masking', this.edits.sharpening_masking, v => `${Math.round(v)}`);
    setVal('noise-reduction', this.edits.noise_reduction, v => `${Math.round(v)}`);

    setVal('shadow-hue', this.edits.split_toning_shadow_hue, v => `${Math.round(v)}°`);
    setVal('shadow-sat', this.edits.split_toning_shadow_sat, v => `${Math.round(v)}`);
    setVal('highlight-hue', this.edits.split_toning_highlight_hue, v => `${Math.round(v)}°`);
    setVal('highlight-sat', this.edits.split_toning_highlight_sat, v => `${Math.round(v)}`);
    setVal('split-balance', this.edits.split_toning_balance, v => `${Math.round(v)}`);

    this.updateSplitToningHint();

    if (this.elements['select-tone-profile']) {
      (this.elements['select-tone-profile'] as HTMLSelectElement).value = this.edits.tone_profile;
    }
    if (this.elements['select-emulsion']) {
      (this.elements['select-emulsion'] as HTMLSelectElement).value = this.edits.emulsion_archetype;
    }
    if (this.elements['select-print-media']) {
      (this.elements['select-print-media'] as HTMLSelectElement).value = this.edits.print_media;
    }
    if (this.elements['select-enlarger-filter']) {
      (this.elements['select-enlarger-filter'] as HTMLSelectElement).value = this.edits.enlarger_filter ?? 'NeutralPanchromatic';
    }

    this.updateEmulsionWorkspaceVisibility();

    if (this.edits.curves) {
      this.curveWidget.setPoints(this.edits.curves);
      const lut = this.bridge.evaluateCurves(this.edits.curves);
      this.renderer.updateToneCurvesLut(lut);
    }
  }

  private updateEmulsionWorkspaceVisibility() {
    // 1. Push / Pull exposure slider only visible when an emulsion archetype is selected
    const hasEmulsion = this.edits.emulsion_archetype !== 'None';
    const rowEmulsionExp = this.elements['row-emulsion-exp'];
    if (rowEmulsionExp) {
      rowEmulsionExp.classList.toggle('hidden', !hasEmulsion);
    }

    // 2. Optical enlarger filter only visible when monochrome media/stock is selected
    const isMonochrome =
      this.edits.print_media === 'BarytaSilverGelatin' ||
      this.edits.print_media === 'WarmtoneChlorobromide' ||
      this.edits.print_media === 'PlatinumPalladium' ||
      this.edits.print_media === 'Cyanotype' ||
      this.edits.emulsion_archetype === 'MobberleySilver400';

    const rowFilter = this.elements['row-enlarger-filter'];
    if (rowFilter) {
      rowFilter.classList.toggle('hidden', !isMonochrome);
    }

    // 3. Print media sliders (Paper Grade, Base Warmth, Process Depth) only visible when print media is selected
    const hasPrintMedia = this.edits.print_media !== 'None';
    const sectionPrintSliders = this.elements['section-print-media-sliders'];
    if (sectionPrintSliders) {
      sectionPrintSliders.classList.toggle('hidden', !hasPrintMedia);
    }
  }

  private syncRatingFlagUI() {
    const starBtns = document.querySelectorAll('.star-btn');
    starBtns.forEach((btn, idx) => {
      btn.classList.toggle('active', idx < this.currentRating);
    });

    const btnPick = this.elements['btn-pick'];
    const btnReject = this.elements['btn-reject'];

    btnPick.classList.toggle('active', this.currentStatus === 'Pick');
    btnReject.classList.toggle('active', this.currentStatus === 'Reject');

    if (this.activeIdx >= 0) {
      const cardEl = document.getElementById(`thumb-card-${this.activeIdx}`);
      if (cardEl) {
        let ratingBadge = cardEl.querySelector('.thumb-badge-rating') as HTMLElement;
        if (this.currentRating > 0) {
          if (!ratingBadge) {
            ratingBadge = document.createElement('div');
            ratingBadge.className = 'thumb-badge-rating';
            cardEl.appendChild(ratingBadge);
          }
          ratingBadge.innerText = '★'.repeat(this.currentRating);
        } else if (ratingBadge) {
          ratingBadge.remove();
        }

        let flagBadge = cardEl.querySelector('.thumb-badge-flag') as HTMLElement;
        if (this.currentStatus === 'Pick' || this.currentStatus === 'Reject') {
          if (!flagBadge) {
            flagBadge = document.createElement('div');
            flagBadge.className = 'thumb-badge-flag';
            cardEl.appendChild(flagBadge);
          }
          flagBadge.className = `thumb-badge-flag ${this.currentStatus.toLowerCase()}`;
          flagBadge.innerText = this.currentStatus.toUpperCase();
        } else if (flagBadge) {
          flagBadge.remove();
        }
      }
    }
  }

  private hasModifications(): boolean {
    if (this.currentRating > 0) return true;
    if (this.currentStatus !== 'Unflagged') return true;

    const title = (this.elements['meta-input-title'] as HTMLInputElement)?.value.trim();
    const description = (this.elements['meta-input-caption'] as HTMLTextAreaElement)?.value.trim();
    const creator = (this.elements['meta-input-creator'] as HTMLInputElement)?.value.trim();
    const kwStr = (this.elements['meta-input-keywords'] as HTMLInputElement)?.value.trim();
    if (title || description || creator || kwStr) return true;

    const d = this.getDefaultEdits(this.isCurrentPhotoRaw());
    const e = this.edits;
    if (Math.abs(e.exposure - d.exposure) > 1e-4) return true;
    if (Math.abs(e.contrast - d.contrast) > 1e-4) return true;
    if (Math.abs(e.highlights - d.highlights) > 1e-4) return true;
    if (Math.abs(e.shadows - d.shadows) > 1e-4) return true;
    if (Math.abs(e.whites - d.whites) > 1e-4) return true;
    if (Math.abs(e.blacks - d.blacks) > 1e-4) return true;
    if (Math.abs(e.temperature - d.temperature) > 1.0) return true;
    if (Math.abs(e.tint - d.tint) > 0.5) return true;
    if (Math.abs(e.vibrance - d.vibrance) > 1e-4) return true;
    if (Math.abs(e.saturation - d.saturation) > 1e-4) return true;
    if (Math.abs(e.deep_blue - d.deep_blue) > 1e-4) return true;
    if (Math.abs(e.foliage - d.foliage) > 1e-4) return true;
    if (Math.abs(e.emulsion_exposure - d.emulsion_exposure) > 1e-4) return true;
    if (Math.abs(e.print_paper_grade - d.print_paper_grade) > 1e-4) return true;
    if (Math.abs(e.print_base_warmth - d.print_base_warmth) > 1e-4) return true;
    if (Math.abs(e.sharpening_amount - d.sharpening_amount) > 1e-4) return true;
    if (Math.abs(e.sharpening_radius - d.sharpening_radius) > 1e-4) return true;
    if (Math.abs(e.sharpening_masking - d.sharpening_masking) > 1e-4) return true;
    if (Math.abs(e.noise_reduction - d.noise_reduction) > 1e-4) return true;
    if (e.tone_profile !== d.tone_profile) return true;
    if (e.emulsion_archetype !== d.emulsion_archetype) return true;
    if (e.print_media !== d.print_media) return true;
    if (e.enlarger_filter !== d.enlarger_filter) return true;
    if (Math.abs((e.print_process_depth ?? 100.0) - (d.print_process_depth ?? 100.0)) > 1e-4) return true;
    if (Math.abs((e.crop_left ?? 0.0) - 0.0) > 1e-4) return true;
    if (Math.abs((e.crop_top ?? 0.0) - 0.0) > 1e-4) return true;
    if (Math.abs((e.crop_right ?? 1.0) - 1.0) > 1e-4) return true;
    if (Math.abs((e.crop_bottom ?? 1.0) - 1.0) > 1e-4) return true;
    const orientation = this.currentMetadata?.orientation ?? 1;
    const exifOri = decodeExifOrientation(orientation);
    if ((e.rotation ?? 0) !== exifOri.rotation) return true;

    return false;
  }

  private async autoSaveSidecar() {
    if (this.activeIdx < 0 || this.activeIdx >= this.fileEntries.length) return;
    if (!this.hasModifications()) return; // Touch-Free Ingestion: Never create or write XMP sidecars for untouched photos!

    const entry = this.fileEntries[this.activeIdx];

    const title = (this.elements['meta-input-title'] as HTMLInputElement).value.trim();
    const description = (this.elements['meta-input-caption'] as HTMLTextAreaElement).value.trim();
    const creator = (this.elements['meta-input-creator'] as HTMLInputElement).value.trim();
    const kwStr = (this.elements['meta-input-keywords'] as HTMLInputElement).value;
    const keywords = kwStr ? kwStr.split(',').map(s => s.trim()).filter(Boolean) : [];

    const xmpPayload: XmpPayload = {
      rating: this.currentRating > 0 ? this.currentRating : undefined,
      status: this.currentStatus,
      title: title || undefined,
      description: description || undefined,
      creator: creator || undefined,
      keywords: keywords.length > 0 ? keywords : undefined,
      edits: this.edits
    };

    const xml = this.bridge.generateXmp(xmpPayload);

    // Save to localStorage as instant session backup
    try {
      localStorage.setItem(`fp_xmp_${entry.name}`, xml);
    } catch (_) {}

    // Direct write to disk via File System Access API
    await FolderIntake.writeSidecarToDisk(entry, xml);
  }

  private openExportModal() {
    const photos = this.getSelectedPhotos();
    if (photos.length === 0) return;
    this.exportModal.open(photos);
  }

  private async handleExport(
    photos: PhotoFileEntry[],
    options: ExportExecutionOptions,
    onProgress: (status: string, current: number, total: number) => void
  ): Promise<{ success: boolean; message: string }> {
    if (photos.length === 0) {
      return { success: false, message: 'No photos selected for export.' };
    }

    const { canvas: expCanvas, renderer: expRenderer } = this.getExportRenderer();
    const savedActiveIdx = this.activeIdx;
    const savedEdits = JSON.parse(JSON.stringify(this.edits));

    let successCount = 0;
    let lastOutputPath = '';

    for (let i = 0; i < photos.length; i++) {
      const entry = photos[i];
      const entryIdx = this.fileEntries.indexOf(entry);
      onProgress(`Exporting ${i + 1} of ${photos.length}...`, i + 1, photos.length);

      // Determine edits for this photo
      let photoEdits: EditSettingsPayload;
      if (entryIdx === savedActiveIdx) {
        photoEdits = savedEdits;
      } else {
        // Read XMP sidecar if exists, otherwise default edits
        const sidecarXml = (await FolderIntake.readSidecarFromDisk(entry)) || localStorage.getItem(`fp_xmp_${entry.name}`);
        if (sidecarXml) {
          try {
            const xmp = this.bridge.parseXmp(sidecarXml);
            photoEdits = xmp.edits || this.getDefaultEdits();
          } catch (_) {
            photoEdits = this.getDefaultEdits();
          }
        } else {
          photoEdits = this.getDefaultEdits();
        }
      }

      const fileExt = entry.name.split('.').pop()?.toLowerCase() || '';
      const isRawFile = ['cr2', 'cr3', 'nef', 'arw', 'dng', 'rw2', 'orf', 'pef', 'raf'].includes(fileExt);

      // Load texture for photo: for RAW files, always decode full sensor resolution (1:1 demosaiced CFA)
      let texW = 0;
      let texH = 0;
      if (isRawFile) {
        const arrayBuffer = await entry.file.arrayBuffer();
        const res = await this.decodeInWorker(arrayBuffer, entry.name, true);
        if (res.linearData && res.linearW && res.linearH) {
          expRenderer.uploadLinearRec2020Texture(res.linearW, res.linearH, res.linearData);
          texW = res.linearW;
          texH = res.linearH;
        } else {
          const blob = new Blob([new Uint8Array(res.imageSlice)], { type: 'image/jpeg' });
          const imgBitmap = await createImageBitmap(blob);
          expRenderer.uploadImage(imgBitmap);
          texW = imgBitmap.width;
          texH = imgBitmap.height;
        }
      } else {
        const arrayBuffer = await entry.file.arrayBuffer();
        const res = await this.decodeInWorker(arrayBuffer, entry.name, false);
        const blob = new Blob([new Uint8Array(res.imageSlice)], { type: 'image/jpeg' });
        const imgBitmap = await createImageBitmap(blob);
        expRenderer.uploadImage(imgBitmap);
        texW = imgBitmap.width;
        texH = imgBitmap.height;
      }

      if (texW <= 0) texW = 1920;
      if (texH <= 0) texH = 1080;

      // Update tone curves LUT on export renderer if curves exist
      if (photoEdits.curves && (photoEdits.curves.master?.length > 2 || photoEdits.curves.red?.length > 2 || photoEdits.curves.green?.length > 2 || photoEdits.curves.blue?.length > 2)) {
        try {
          const lut = this.bridge.evaluateCurves(photoEdits.curves);
          expRenderer.updateToneCurvesLut(lut);
        } catch (err) {
          console.warn('[Export] Could not evaluate curve LUT:', err);
        }
      }

      // Calculate rotated and cropped dimensions
      const rot = (photoEdits.rotation ?? 0) % 360;
      const isOrthogonal = (rot === 90 || rot === 270);
      const baseW = isOrthogonal ? texH : texW;
      const baseH = isOrthogonal ? texW : texH;

      const cropL = photoEdits.crop_left ?? 0.0;
      const cropT = photoEdits.crop_top ?? 0.0;
      const cropR = photoEdits.crop_right ?? 1.0;
      const cropB = photoEdits.crop_bottom ?? 1.0;
      const effectiveW = Math.max(1, Math.round(baseW * Math.max(0.01, cropR - cropL)));
      const effectiveH = Math.max(1, Math.round(baseH * Math.max(0.01, cropB - cropT)));

      // Sizing calculation
      let targetW = effectiveW;
      let targetH = effectiveH;
      let maxDim: number | null = null;
      if (options.sizeOption === '2048') maxDim = 2048;
      else if (options.sizeOption === '3840') maxDim = 3840;
      else if (options.sizeOption === 'Custom') maxDim = options.customDimension;

      if (maxDim && (effectiveW > maxDim || effectiveH > maxDim)) {
        if (effectiveW >= effectiveH) {
          targetW = maxDim;
          targetH = Math.max(1, Math.round((effectiveH * maxDim) / effectiveW));
        } else {
          targetH = maxDim;
          targetW = Math.max(1, Math.round((effectiveW * maxDim) / effectiveH));
        }
      }

      // Clamp to WebGL viewport capability
      const maxGl = expRenderer.glContext.getParameter(expRenderer.glContext.MAX_VIEWPORT_DIMS)?.[0] || 8192;
      if (targetW > maxGl || targetH > maxGl) {
        const scale = maxGl / Math.max(targetW, targetH);
        targetW = Math.max(1, Math.round(targetW * scale));
        targetH = Math.max(1, Math.round(targetH * scale));
      }

      // Render frame on dedicated off-screen export canvas
      expCanvas.width = targetW;
      expCanvas.height = targetH;
      expRenderer.render(photoEdits, false, 1.0, { x: 0, y: 0 }, false);

      // Encode image (macOS Parity: ${baseName}_FirstProof.${ext})
      const stem = entry.name.substring(0, entry.name.lastIndexOf('.')) || entry.name;
      const ext = (options.format === 'TIFF') ? 'tiff' : 'jpg';
      const filename = `${stem}_FirstProof.${ext}`;
      lastOutputPath = filename;

      let fileBlob: Blob;
      if (options.format === 'TIFF') {
        const pixels = expRenderer.readRgbaPixels(targetW, targetH);
        const tiffBytes = encodeTiff(targetW, targetH, pixels);
        fileBlob = new Blob([tiffBytes.buffer as ArrayBuffer], { type: 'image/tiff' });
      } else {
        // Standard calibrated sRGB JPEG
        fileBlob = await new Promise<Blob>((resolve, reject) => {
          expCanvas.toBlob(
            (b) => b ? resolve(b) : reject(new Error('Canvas toBlob failed')),
            'image/jpeg',
            options.quality / 100
          );
        });
      }

      // Write to destination directory or trigger browser download
      if (options.destinationDirHandle) {
        try {
          const fileHandle = await options.destinationDirHandle.getFileHandle(filename, { create: true });
          const writable = await fileHandle.createWritable();
          await writable.write(fileBlob);
          await writable.close();
        } catch (writeErr) {
          console.warn('[Export] File system write failed, falling back to download:', writeErr);
          this.triggerDownload(fileBlob, filename);
        }
      } else {
        this.triggerDownload(fileBlob, filename);
      }

      successCount++;
    }

    const isBatch = photos.length > 1;
    const message = isBatch
      ? `Finished exporting ${successCount} of ${photos.length} photos`
      : `Successfully exported to ${lastOutputPath}`;

    return { success: successCount > 0, message };
  }

  private triggerDownload(blob: Blob, filename: string) {
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1500);
  }

  private updateUI() {
    this.syncRatingFlagUI();
  }
}

// Instantiate and start app when DOM is ready
window.addEventListener('DOMContentLoaded', () => {
  const app = new DarkroomApp();
  app.init();
});
