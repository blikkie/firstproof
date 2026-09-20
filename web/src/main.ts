import { WasmBridge, EditSettingsPayload, XmpPayload, PreviewResult } from './WasmBridge';
import { WebGLRenderer } from './WebGLRenderer';
import { FolderIntake, PhotoFileEntry } from './FolderIntake';
import { ToneCurveWidget } from './ToneCurveWidget';
import { ThumbnailCacheStore } from './ThumbnailCacheStore';
import { ParityModal } from './parity/ParityModal';
import {
  formatAperture,
  formatShutterSpeed,
  formatFocalLength,
  formatISO,
  formatExposure,
  formatDimensions
} from './utils/formatters';

class DarkroomApp {
  private bridge!: WasmBridge;
  private renderer!: WebGLRenderer;
  private curveWidget!: ToneCurveWidget;
  private parityModal: ParityModal = new ParityModal();

  private fileEntries: PhotoFileEntry[] = [];
  private activeIdx: number = -1;
  private dirHandle: any = null;

  // Edit Settings state
  private edits: EditSettingsPayload = this.getDefaultEdits();
  private isOriginalPeek: boolean = false;
  private currentMetadata: any = {};
  private currentRating: number = 0;
  private currentStatus: string = 'Unflagged';
  private activeEyedropper: 'wb' | 'white' | 'black' | null = null;
  private selectRequestId: number = 0;
  private selectDebounceTimer: any = null;
  private preloadDebounceTimer: any = null;
  private thumbLoadingSet: Set<number> = new Set();
  private prewarmLoadingSet: Set<number> = new Set();

  private rawWorker: Worker | null = null;
  private workerMsgId: number = 0;
  private workerPendingMap: Map<number, (res: any) => void> = new Map();

  private elements: Record<string, HTMLElement> = {};

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

  private async decodeInWorker(arrayBuffer: ArrayBuffer, fileName: string): Promise<{ preview: PreviewResult; imageSlice: Uint8Array; linearData?: Uint16Array; linearW?: number; linearH?: number }> {
    if (!this.rawWorker) {
      const bytes = new Uint8Array(arrayBuffer);
      const res = this.bridge.decodeImageBytes(bytes);
      return res;
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
      this.rawWorker!.postMessage({ id, arrayBuffer: workerBuffer, fileName }, [workerBuffer]);
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
    this.parityModal.init(this.renderer, () => this.edits);

    // 3. Init Tone Curve Widget
    const curveCanvas = document.getElementById('curve-canvas') as HTMLCanvasElement;
    this.curveWidget = new ToneCurveWidget(curveCanvas, () => this.onCurveChange());

    // 4. Bind UI Controls & Shortcuts
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

  private getDefaultEdits(): EditSettingsPayload {
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
      sharpening_amount: 0.0,
      sharpening_radius: 1.0,
      sharpening_masking: 0.0,
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
      'btn-open-folder', 'file-input-fallback', 'btn-original-peek', 'btn-export-proof', 'btn-parity-audit', 'btn-shortcuts',
      'empty-state', 'btn-empty-open', 'developing-spinner', 'filmstrip-count', 'filmstrip-scroll',
      'meta-filename', 'meta-dimensions', 'meta-camera', 'meta-iso', 'meta-aperture', 'meta-shutter',
      'btn-pick', 'btn-reject', 'modal-shortcuts', 'btn-close-modal', 'select-tone-profile',
      'select-wb-preset', 'btn-wb-eyedropper', 'btn-white-point-picker', 'btn-black-point-picker',
      'select-emulsion', 'select-print-media', 'btn-reset-curve',
      'meta-input-title', 'meta-input-caption', 'meta-input-creator', 'meta-input-keywords'
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

    // Export Proof
    this.elements['btn-export-proof'].addEventListener('click', () => this.exportProofJpeg());

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

    // Light Source Presets (Identical parity with Swift AppViewModel+Editor.swift)
    this.elements['select-wb-preset']?.addEventListener('change', (e) => {
      const preset = (e.target as HTMLSelectElement).value;
      switch (preset) {
        case 'AsShot':
          this.edits.temperature = 5500.0;
          this.edits.tint = 0.0;
          break;
        case 'Auto':
          // Auto WB approximation
          this.edits.temperature = 5400.0;
          this.edits.tint = 2.0;
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

    // Sliders & Selects plumbing with double-click reset
    const bindSlider = (id: string, prop: keyof EditSettingsPayload, format: (val: number) => string, defaultVal: number = 0) => {
      const input = document.getElementById(`slider-${id}`) as HTMLInputElement;
      const label = document.getElementById(`val-${id}`) as HTMLElement;
      const rowLabel = input?.closest('.slider-row')?.querySelector('label');
      if (input && label) {
        input.addEventListener('input', () => {
          const val = parseFloat(input.value);
          (this.edits[prop] as number) = val;
          label.innerText = format(val);
          this.renderCanvas();
        });
        input.addEventListener('change', () => this.autoSaveSidecar());

        const resetFn = () => {
          input.value = `${defaultVal}`;
          (this.edits[prop] as number) = defaultVal;
          label.innerText = format(defaultVal);
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
    bindSlider('temp', 'temperature', v => `${Math.round(v)} K`, 5500.0);
    bindSlider('tint', 'tint', v => `${Math.round(v)}`, 0.0);
    bindSlider('vibrance', 'vibrance', v => `${Math.round(v)}`, 0.0);
    bindSlider('saturation', 'saturation', v => `${Math.round(v)}`, 0.0);
    bindSlider('deep-blue', 'deep_blue', v => `${Math.round(v)}`, 0.0);
    bindSlider('foliage', 'foliage', v => `${Math.round(v)}`, 0.0);
    bindSlider('emulsion-exp', 'emulsion_exposure', v => formatExposure(v), 0.0);
    bindSlider('paper-grade', 'print_paper_grade', v => `${v.toFixed(1)}`, 0.0);
    bindSlider('base-warmth', 'print_base_warmth', v => `${Math.round(v)}`, 0.0);
    bindSlider('shadow-hue', 'split_toning_shadow_hue', v => `${Math.round(v)}°`, 0);
    bindSlider('shadow-sat', 'split_toning_shadow_sat', v => `${Math.round(v)}`, 0);
    bindSlider('highlight-hue', 'split_toning_highlight_hue', v => `${Math.round(v)}°`, 0);
    bindSlider('highlight-sat', 'split_toning_highlight_sat', v => `${Math.round(v)}`, 0);
    bindSlider('split-balance', 'split_toning_balance', v => `${Math.round(v)}`, 0);

    // Tone Profile Select
    this.elements['select-tone-profile']?.addEventListener('change', (e) => {
      this.edits.tone_profile = (e.target as HTMLSelectElement).value;
      this.renderCanvas();
      this.autoSaveSidecar();
    });

    // Emulsion Select
    this.elements['select-emulsion']?.addEventListener('change', (e) => {
      this.edits.emulsion_archetype = (e.target as HTMLSelectElement).value;
      this.renderCanvas();
      this.autoSaveSidecar();
    });

    // Print Media Select
    this.elements['select-print-media']?.addEventListener('change', (e) => {
      this.edits.print_media = (e.target as HTMLSelectElement).value;
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

    // Metadata text inputs auto-save
    const metaInputs = ['meta-input-title', 'meta-input-caption', 'meta-input-creator', 'meta-input-keywords'];
    for (const id of metaInputs) {
      if (this.elements[id]) {
        this.elements[id].addEventListener('change', () => this.autoSaveSidecar());
      }
    }

    // Keyboard Shortcuts filter with FIRST-RESPONDER TEXT FIELD SHIELDING
    window.addEventListener('keydown', (e: KeyboardEvent) => {
      const activeEl = document.activeElement;
      if (activeEl && (activeEl.tagName === 'INPUT' || activeEl.tagName === 'TEXTAREA' || (activeEl as HTMLElement).isContentEditable)) {
        return; // Guardrail: Never intercept single-key shortcuts while user is typing in metadata fields!
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
          this.currentStatus = (this.currentStatus === 'Reject') ? 'Unflagged' : 'Reject';
          this.syncRatingFlagUI();
          this.autoSaveSidecar();
          break;
        case 'u': case 'U':
          this.currentStatus = 'Unflagged';
          this.syncRatingFlagUI();
          this.autoSaveSidecar();
          break;
        case 'w': case 'W':
          this.toggleEyedropper('wb');
          break;
        case '\\':
          this.isOriginalPeek = true;
          this.renderCanvas();
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

      card.addEventListener('click', () => this.selectPhoto(idx, false));
      filmstripScroll.appendChild(card);
    });

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

  private async loadThumbnailForCard(idx: number) {
    if (idx < 0 || idx >= this.fileEntries.length) return;
    if (this.thumbCache.has(idx) || this.thumbLoadingSet.has(idx)) return;
    const cardEl = document.getElementById(`thumb-card-${idx}`);
    if (!cardEl || cardEl.querySelector('img')) return;

    this.thumbLoadingSet.add(idx);
    try {
      let previewUrl = this.thumbCache.get(idx);
      if (!previewUrl) {
        const entry = this.fileEntries[idx];
        const cacheKey = `${entry.name}_${entry.file.size}_${entry.file.lastModified}`;
        const cachedBlob = await ThumbnailCacheStore.get(cacheKey);
        if (cachedBlob) {
          previewUrl = URL.createObjectURL(cachedBlob);
          this.cacheThumbnail(idx, previewUrl);
        } else {
          const arrayBuffer = await entry.file.arrayBuffer();
          const bytes = new Uint8Array(arrayBuffer);
          const { imageSlice } = this.bridge.decodeImageBytes(bytes);
          const previewBlob = new Blob([new Uint8Array(imageSlice)], { type: 'image/jpeg' });
          previewUrl = URL.createObjectURL(previewBlob);
          this.cacheThumbnail(idx, previewUrl);
          await ThumbnailCacheStore.set(cacheKey, previewBlob);
        }
      }

      const existingImg = cardEl.querySelector('img') as HTMLImageElement | null;
      if (existingImg) {
        existingImg.src = previewUrl;
      } else {
        const img = document.createElement('img');
        img.className = 'thumb-img';
        img.src = previewUrl;
        const placeholder = cardEl.querySelector('.thumb-icon-placeholder');
        if (placeholder) placeholder.remove();
        cardEl.insertBefore(img, cardEl.firstChild);
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

  private async selectPhoto(idx: number, centerFilmstrip: boolean = false) {
    if (idx < 0 || idx >= this.fileEntries.length) return;

    const requestId = ++this.selectRequestId;
    this.activeIdx = idx;

    // 1. INSTANT UI updates (0ms delay!)
    const cards = document.querySelectorAll('.thumb-card');
    cards.forEach((c, i) => c.classList.toggle('active', i === idx));

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

        // Update EXIF metadata header
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

        // Update filmstrip thumbnail
        const cardEl = document.getElementById(`thumb-card-${idx}`);
        if (cardEl) {
          let previewUrl = this.thumbCache.get(idx);
          if (!previewUrl) {
            const previewBlob = new Blob([new Uint8Array(imageSlice)], { type: 'image/jpeg' });
            previewUrl = URL.createObjectURL(previewBlob);
            this.cacheThumbnail(idx, previewUrl);
            const cacheKey = `${entry.name}_${entry.file.size}_${entry.file.lastModified}`;
            ThumbnailCacheStore.set(cacheKey, previewBlob);
          }

          const existingImg = cardEl.querySelector('img') as HTMLImageElement | null;
          if (existingImg) {
            existingImg.src = previewUrl;
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

        if (sidecarXml) {
          const xmp = this.bridge.parseXmp(sidecarXml);
          this.edits = { ...this.getDefaultEdits(), ...xmp.edits };
          this.currentRating = xmp.rating || 0;
          this.currentStatus = xmp.status || 'Unflagged';
          (this.elements['meta-input-title'] as HTMLInputElement).value = xmp.title || '';
          (this.elements['meta-input-caption'] as HTMLTextAreaElement).value = xmp.description || '';
          (this.elements['meta-input-creator'] as HTMLInputElement).value = xmp.creator || '';
          (this.elements['meta-input-keywords'] as HTMLInputElement).value = (xmp.keywords || []).join(', ');
        } else {
          this.edits = this.getDefaultEdits();
          this.currentRating = 0;
          this.currentStatus = 'Unflagged';
        }

        this.syncSliderUI();
        this.syncRatingFlagUI();

        // Check if raw negative and extract uncompressed 16-bit Linear Rec.2020 sensor frame
        const ext = entry.name.split('.').pop()?.toLowerCase() || '';
        const isRawFile = ['cr2', 'cr3', 'nef', 'arw', 'dng', 'rw2', 'orf', 'pef', 'raf'].includes(ext);

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
          imgW = linearW;
          imgH = linearH;
          this.renderer.uploadLinearRec2020Texture(linearW, linearH, linear16Data);
        } else {
          imgW = imgBitmap.width;
          imgH = imgBitmap.height;
          this.renderer.uploadImage(imgBitmap);
        }

        const maxDim = Math.min(Math.max(imgW, imgH), 3840);
        canvas.width = imgW >= imgH ? maxDim : Math.round(maxDim * (imgW / imgH));
        canvas.height = imgH > imgW ? maxDim : Math.round(maxDim * (imgH / imgW));
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

  private renderCanvas() {
    if (this.activeIdx >= 0) {
      this.renderer.render(this.edits, this.isOriginalPeek);
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
    setVal('temp', this.edits.temperature, v => `${Math.round(v)} K`);
    setVal('tint', this.edits.tint, v => `${Math.round(v)}`);
    setVal('vibrance', this.edits.vibrance, v => `${Math.round(v)}`);
    setVal('saturation', this.edits.saturation, v => `${Math.round(v)}`);
    setVal('deep-blue', this.edits.deep_blue, v => `${Math.round(v)}`);
    setVal('foliage', this.edits.foliage, v => `${Math.round(v)}`);
    setVal('emulsion-exp', this.edits.emulsion_exposure, v => `${v > 0 ? '+' : ''}${v.toFixed(2)} EV`);
    setVal('paper-grade', this.edits.print_paper_grade, v => `${v.toFixed(1)}`);
    setVal('base-warmth', this.edits.print_base_warmth, v => `${Math.round(v)}`);

    if (this.elements['select-tone-profile']) {
      (this.elements['select-tone-profile'] as HTMLSelectElement).value = this.edits.tone_profile;
    }
    if (this.elements['select-emulsion']) {
      (this.elements['select-emulsion'] as HTMLSelectElement).value = this.edits.emulsion_archetype;
    }
    if (this.elements['select-print-media']) {
      (this.elements['select-print-media'] as HTMLSelectElement).value = this.edits.print_media;
    }

    if (this.edits.curves) {
      this.curveWidget.setPoints(this.edits.curves);
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

    const d = this.getDefaultEdits();
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
    if (e.tone_profile !== d.tone_profile) return true;
    if (e.emulsion_archetype !== d.emulsion_archetype) return true;
    if (e.print_media !== d.print_media) return true;

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

  private exportProofJpeg() {
    const canvas = document.getElementById('darkroom-canvas') as HTMLCanvasElement;
    if (canvas) {
      const dataUrl = canvas.toDataURL('image/jpeg', 0.92);
      const link = document.createElement('a');
      const filename = this.activeIdx >= 0 ? `${this.fileEntries[this.activeIdx].name}_proof.jpg` : 'firstproof_proof.jpg';
      link.download = filename;
      link.href = dataUrl;
      link.click();
    }
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
