import { CropController } from './CropController';
import { CanvasViewportController } from './controllers/CanvasViewportController';
import { FilmstripController } from './controllers/FilmstripController';
import { InspectorSyncController } from './controllers/InspectorSyncController';
import { KeyboardShortcutController } from './controllers/KeyboardShortcutController';
import { type ExportExecutionOptions, ExportModal } from './ExportModal';
import { FolderIntake, type PhotoFileEntry } from './FolderIntake';
import { ParityModal } from './parity/ParityModal';
import {
  PhotoLoaderService,
  type PhotoRecord,
} from './services/PhotoLoaderService';
import { ToneCurveWidget } from './ToneCurveWidget';
import { isDebugMode } from './utils/debug';
import { getDefaultEdits } from './utils/edits';
import { decodeExifOrientation } from './utils/exifReader';
import { updateMetadataHeaderElements } from './utils/metadataHeader';
import {
  type EditSettingsPayload,
  type ImageMetadata,
  WasmBridge,
  type XmpPayload,
} from './WasmBridge';
import { WebGLRenderer } from './WebGLRenderer';

class DarkroomApp {
  private bridge!: WasmBridge;
  private renderer!: WebGLRenderer;
  private loaderService!: PhotoLoaderService;
  private curveWidget!: ToneCurveWidget;
  private parityModal: ParityModal = new ParityModal();
  private exportModal: ExportModal = new ExportModal();
  private cropController!: CropController;

  // Controllers
  private viewportController!: CanvasViewportController;
  private keyboardController!: KeyboardShortcutController;
  private inspectorController!: InspectorSyncController;
  private filmstripController!: FilmstripController;

  // Active Photo State
  private edits: EditSettingsPayload = getDefaultEdits();
  private isOriginalPeek: boolean = false;
  private isCropActive: boolean = false;
  private currentMetadata: Partial<ImageMetadata> = {};
  private currentRating: number = 0;
  private currentStatus: string = 'Unflagged';

  // Selection Pipeline & Active Frame Buffers
  private selectRequestId: number = 0;
  private selectDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  private activeLinearData: Uint16Array | null = null;
  private activeLinearW: number = 0;
  private activeLinearH: number = 0;
  private activeSourceImage: CanvasImageSource | null = null;
  private activeSourceW: number = 0;
  private activeSourceH: number = 0;

  private elements: Record<string, HTMLElement> = {};

  public async init() {
    console.log('[App] Starting First Proof Web Darkroom Engine...');
    this.cacheElements();

    // 1. Init WASM (256MB scratchpad capacity for 50MP+ 16-bit linear RAW frames)
    try {
      this.bridge = await WasmBridge.init(256);
      this.loaderService = new PhotoLoaderService(this.bridge);
    } catch (err) {
      console.error('[App] Failed to initialize WASM scratchpad:', err);
      alert(
        'Failed to initialize WebAssembly core engine. Check browser console.',
      );
      return;
    }

    // 2. Init WebGL Renderer
    const canvas = document.getElementById(
      'darkroom-canvas',
    ) as HTMLCanvasElement;
    canvas.width = 1920;
    canvas.height = 1080;
    this.renderer = new WebGLRenderer(canvas);

    // Parity Modal (Mechanical Parity Audit)
    this.parityModal.init(
      this.renderer,
      () => this.edits,
      () => {
        if (this.filmstripController.getActiveIndex() < 0) return null;
        if (
          this.activeLinearData &&
          this.activeLinearW > 0 &&
          this.activeLinearH > 0
        ) {
          return {
            isLinearRec2020: true,
            width: this.activeLinearW,
            height: this.activeLinearH,
            linearData: this.activeLinearData,
          };
        } else if (
          this.activeSourceImage &&
          this.activeSourceW > 0 &&
          this.activeSourceH > 0
        ) {
          return {
            isLinearRec2020: false,
            width: this.activeSourceW,
            height: this.activeSourceH,
            imageSource: this.activeSourceImage,
          };
        }
        return null;
      },
    );

    // 3. Init Export Modal
    this.exportModal.init((photos, options, onProgress) =>
      this.handleExport(photos, options, onProgress),
    );

    // 4. Init Tone Curve Widget
    const curveCanvas = document.getElementById(
      'curve-canvas',
    ) as HTMLCanvasElement;
    this.curveWidget = new ToneCurveWidget(curveCanvas, () =>
      this.onCurveChange(),
    );
    const initialLut = this.bridge.evaluateCurves(this.edits.curves);
    this.renderer.updateToneCurvesLut(initialLut);

    // 5. Init Interactive Crop Controller
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
        const straightenSlider = document.getElementById(
          'slider-straighten',
        ) as HTMLInputElement;
        const straightenVal = document.getElementById('val-straighten');
        if (straightenSlider) straightenSlider.value = `${angle}`;
        if (straightenVal)
          straightenVal.textContent = `${angle > 0 ? '+' : ''}${angle.toFixed(1)}°`;
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
      },
    });

    // 6. Init Viewport & Navigation Controllers
    if (!isDebugMode()) {
      const btnParity = document.getElementById('btn-parity-audit');
      if (btnParity) btnParity.style.display = 'none';
    }

    const canvasWrapper = document.getElementById(
      'canvas-wrapper',
    ) as HTMLElement;
    const zoomBadge = document.getElementById('zoom-badge');
    this.viewportController = new CanvasViewportController(
      canvas,
      canvasWrapper,
      zoomBadge,
      {
        onRender: () => this.renderCanvas(),
        onUpdateCropOverlay: () => {
          if (this.isCropActive) this.updateCropOverlayPosition();
        },
        onSampleEyedropper: (mode, r, g, b) =>
          this.handleSampleEyedropper(mode, r, g, b),
        isCropActive: () => this.isCropActive,
        getEdits: () => this.edits,
        getImageDimensions: () => ({
          width: this.activeLinearW || this.activeSourceW || 1920,
          height: this.activeLinearH || this.activeSourceH || 1080,
          rawWidth: this.currentMetadata?.raw_width,
          rawHeight: this.currentMetadata?.raw_height,
        }),
      },
    );
    this.viewportController.bind();

    // 7. Init Filmstrip Controller
    const filmstripScroll = document.getElementById(
      'filmstrip-scroll',
    ) as HTMLElement;
    const filmstripCount = document.getElementById('filmstrip-count');
    const emptyState = document.getElementById('empty-state');

    this.filmstripController = new FilmstripController(
      filmstripScroll,
      filmstripCount,
      emptyState,
      this.bridge,
      this.loaderService,
      {
        onSelectPhoto: (idx, centerFilmstrip, keepMultiSelection) => {
          this.selectPhoto(idx, centerFilmstrip, keepMultiSelection);
        },
      },
    );

    // 8. Init Keyboard Shortcut Controller
    this.keyboardController = new KeyboardShortcutController({
      onNavigate: (dir) => {
        if (dir === 1) this.filmstripController.selectNext();
        else this.filmstripController.selectPrevious();
      },
      onRotate: (deltaDeg) => this.rotateImage(deltaDeg),
      onToggleCrop: () => this.toggleCropMode(),
      onPeekOriginal: (active: boolean) => {
        this.isOriginalPeek = active;
        this.renderCanvas();
      },
      onTogglePixelPeep: () => this.viewportController.togglePixelPeep(),
      onToggleEyedropper: (mode) =>
        this.viewportController.toggleEyedropper(mode),
      onEnter: () => {
        if (this.isCropActive) this.toggleCropMode(false);
      },
      isCropActive: () => this.isCropActive,
      onEscape: () => {
        if (this.cropController?.isRotating) {
          this.cropController.cancelRotation();
          return;
        }
        if (this.inspectorController?.hasActiveScrub()) {
          this.inspectorController.rollbackActiveScrub();
          return;
        }
        if (this.isCropActive) {
          this.toggleCropMode(false);
          return;
        }
        if (this.viewportController.getActiveEyedropper()) {
          this.viewportController.clearEyedropper();
          return;
        }
      },
      onSelectTab: (tab) => this.selectInspectorTab(tab),
      onResetAllEdits: () => this.resetAllEdits(),
      onApplyAutoTone: () => this.applyAutoTone(),
      onOpenExportModal: () => this.openExportModal(),
      onSetRating: (rating) => {
        this.currentRating = rating;
        this.syncRatingFlagUI();
        this.autoSaveSidecar();
      },
      onTogglePick: () => {
        this.currentStatus =
          this.currentStatus === 'Pick' ? 'Unflagged' : 'Pick';
        this.syncRatingFlagUI();
        this.autoSaveSidecar();
      },
      onToggleReject: () => {
        this.currentStatus =
          this.currentStatus === 'Reject' ? 'Unflagged' : 'Reject';
        this.syncRatingFlagUI();
        this.autoSaveSidecar();
      },
      onUnflag: () => {
        this.currentStatus = 'Unflagged';
        this.syncRatingFlagUI();
        this.autoSaveSidecar();
      },
    });
    this.keyboardController.bind();

    // 9. Init Inspector Sync Controller
    this.inspectorController = new InspectorSyncController({
      getEdits: () => this.edits,
      renderCanvas: () => this.renderCanvas(),
      autoSaveSidecar: () => this.autoSaveSidecar(),
      isCurrentPhotoRaw: () => this.isCurrentPhotoRaw(),
      getDefaultEdits: () => getDefaultEdits(this.isCurrentPhotoRaw()),
      getCurveWidget: () => this.curveWidget,
      getRating: () => this.currentRating,
      getStatus: () => this.currentStatus,
      getActiveIndex: () => this.filmstripController.getActiveIndex(),
      onResetAllEdits: () => this.resetAllEdits(),
    });
    this.inspectorController.bind();

    // 10. Bind UI Controls
    this.bindEvents();
    this.syncRatingFlagUI();

    // 10b. Check for full folder access (File System Access API) for sidecar persistence
    const hasFullFolderAccess = 'showDirectoryPicker' in window;
    const accessHint = document.getElementById('browser-folder-access-hint');
    if (accessHint) {
      accessHint.classList.toggle('hidden', hasFullFolderAccess);
    }
    const storageBadge = document.getElementById('browser-storage-badge');
    if (storageBadge) {
      storageBadge.classList.toggle('hidden', hasFullFolderAccess);
    }

    // 11. Restore recent shoot folder memory from IndexedDB across browser restarts
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
            const entries = await FolderIntake.processDirectoryHandle(
              recent.dirHandle,
            );
            if (entries.length > 0) {
              FolderIntake.saveRecentDirHandle(recent.dirHandle);
              let initialName: string | null = null;
              try {
                initialName = localStorage.getItem('fp_recent_photo_name');
              } catch (_) {}
              this.filmstripController.loadPhotos(entries, initialName);
            }
          });
          emptyActions.appendChild(btnRecent);
        }
      }
    });

    console.log('[App] Ready for photographic intake.');
  }

  public isCurrentPhotoRaw(): boolean {
    const entry = this.filmstripController.getActiveEntry();
    if (!entry) return false;
    const ext = entry.name.split('.').pop()?.toLowerCase() || '';
    return [
      'cr2',
      'cr3',
      'nef',
      'arw',
      'dng',
      'rw2',
      'orf',
      'pef',
      'raf',
    ].includes(ext);
  }

  private cacheElements() {
    const ids = [
      'btn-open-folder',
      'file-input-fallback',
      'btn-original-peek',
      'btn-export-proof',
      'btn-parity-audit',
      'btn-shortcuts',
      'empty-state',
      'btn-empty-open',
      'developing-spinner',
      'filmstrip-count',
      'filmstrip-scroll',
      'meta-filename',
      'meta-dimensions',
      'meta-camera',
      'meta-iso',
      'meta-aperture',
      'meta-shutter',
      'meta-format-badge',
      'meta-full-camera',
      'meta-lens',
      'meta-focal-length',
      'meta-exposure-summary',
      'meta-sensor-res',
      'meta-date-taken',
      'meta-sidecar-status',
      'btn-reset-all',
      'btn-pick',
      'btn-reject',
      'modal-shortcuts',
      'btn-close-modal',
      'select-tone-profile',
      'select-wb-preset',
      'btn-wb-eyedropper',
      'btn-white-point-picker',
      'btn-black-point-picker',
      'select-emulsion',
      'select-emulsion-grain-mode',
      'select-print-media',
      'select-enlarger-filter',
      'row-enlarger-filter',
      'row-emulsion-exp',
      'row-grain-scale',
      'section-print-media-sliders',
      'btn-reset-curve',
      'btn-auto-tone',
      'meta-input-title',
      'meta-input-caption',
      'meta-input-creator',
      'meta-input-keywords',
      'btn-crop-tool',
      'btn-rotate-ccw',
      'btn-rotate-cw',
      'zoom-badge',
      'peek-badge',
    ];
    for (const id of ids) {
      const el = document.getElementById(id);
      if (el) this.elements[id] = el;
    }
  }

  private handleSampleEyedropper(
    mode: 'wb' | 'white' | 'black',
    r: number,
    g: number,
    b: number,
  ) {
    if (mode === 'wb') {
      if (r > 0 && g > 0 && b > 0) {
        const curTemp = this.edits.temperature;
        const curTint = this.edits.tint;
        const targetTemp = Math.min(
          Math.max(Math.round(curTemp * (b / r) ** 0.35), 2000),
          15000,
        );
        const targetTint = Math.min(
          Math.max(Math.round(curTint + (g - (r + b) / 2) * 80), -150),
          150,
        );

        this.edits.temperature = targetTemp;
        this.edits.tint = targetTint;
        if (this.elements['select-wb-preset']) {
          (this.elements['select-wb-preset'] as HTMLSelectElement).value =
            'Custom';
        }
      }
    } else if (mode === 'white') {
      const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
      const shift = Math.round(
        Math.min(Math.max((1.0 - lum) * 100, -100), 100),
      );
      this.edits.whites = shift;
    } else if (mode === 'black') {
      const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
      const shift = Math.round(Math.min(Math.max(-lum * 100, -100), 100));
      this.edits.blacks = shift;
    }

    this.inspectorController.syncSliderUI();
    this.renderCanvas();
    this.autoSaveSidecar();
  }

  private bindEvents() {
    const handleOpenFolder = async () => {
      if ('showDirectoryPicker' in window) {
        try {
          const res = await FolderIntake.openDirectoryPicker();
          if (res.entries.length > 0) {
            if (res.dirHandle) {
              FolderIntake.saveRecentDirHandle(res.dirHandle);
            }
            this.filmstripController.loadPhotos(res.entries);
            return;
          }
          if (res.cancelled) return;
        } catch (err) {
          console.warn('[FolderIntake] openDirectoryPicker error:', err);
        }
      }

      const fallbackEl = this.elements[
        'file-input-fallback'
      ] as HTMLInputElement;
      if (fallbackEl) {
        fallbackEl.value = '';
        fallbackEl.click();
      }
    };

    this.elements['btn-open-folder'].addEventListener(
      'click',
      handleOpenFolder,
    );
    this.elements['btn-empty-open'].addEventListener('click', handleOpenFolder);

    (this.elements['file-input-fallback'] as HTMLInputElement).addEventListener(
      'change',
      (e: Event) => {
        const target = e.target as HTMLInputElement;
        if (target.files && target.files.length > 0) {
          const entries = FolderIntake.processFileList(target.files);
          this.filmstripController.loadPhotos(entries);
        }
      },
    );

    // Original Peek \
    const btnPeek = this.elements['btn-original-peek'];
    btnPeek.addEventListener('mousedown', () => {
      this.isOriginalPeek = true;
      this.renderCanvas();
    });
    btnPeek.addEventListener('mouseup', () => {
      this.isOriginalPeek = false;
      this.renderCanvas();
    });
    btnPeek.addEventListener('mouseleave', () => {
      this.isOriginalPeek = false;
      this.renderCanvas();
    });

    // Export Photo Modal
    this.elements['btn-export-proof'].addEventListener('click', () =>
      this.openExportModal(),
    );

    // Mechanical Parity Audit Modal
    this.elements['btn-parity-audit']?.addEventListener('click', () =>
      this.parityModal.show(),
    );

    // Shortcuts Modal
    const modal = this.elements['modal-shortcuts'] as HTMLDialogElement;
    this.elements['btn-shortcuts'].addEventListener('click', () =>
      modal.showModal(),
    );
    this.elements['btn-close-modal'].addEventListener('click', () =>
      modal.close(),
    );

    // Rating & Flag Toolbar
    const starBtns = document.querySelectorAll('.star-btn');
    starBtns.forEach((btn) => {
      btn.addEventListener('click', (e) => {
        const star = parseInt(
          (e.target as HTMLElement).getAttribute('data-star') || '0',
          10,
        );
        this.currentRating = this.currentRating === star ? 0 : star;
        this.syncRatingFlagUI();
        this.autoSaveSidecar();
      });
    });

    this.elements['btn-pick'].addEventListener('click', () => {
      this.currentStatus = this.currentStatus === 'Pick' ? 'Unflagged' : 'Pick';
      this.syncRatingFlagUI();
      this.autoSaveSidecar();
    });

    this.elements['btn-reject'].addEventListener('click', () => {
      this.currentStatus =
        this.currentStatus === 'Reject' ? 'Unflagged' : 'Reject';
      this.syncRatingFlagUI();
      this.autoSaveSidecar();
    });

    // Header Crop & Rotation Tools
    this.elements['btn-crop-tool']?.addEventListener('click', () =>
      this.toggleCropMode(),
    );
    this.elements['btn-rotate-ccw']?.addEventListener('click', () =>
      this.rotateImage(-90),
    );
    this.elements['btn-rotate-cw']?.addEventListener('click', () =>
      this.rotateImage(90),
    );

    // Auto Tone Button
    this.elements['btn-auto-tone']?.addEventListener('click', () =>
      this.applyAutoTone(),
    );

    // Light Source Presets
    this.elements['select-wb-preset']?.addEventListener('change', (e) => {
      const preset = (e.target as HTMLSelectElement).value;
      if (preset === 'AsShot') {
        this.edits.temperature = 5500.0;
        this.edits.tint = 0.0;
      } else if (preset === 'Auto') {
        this.applyAutoWhiteBalance();
      } else {
        const values = this.bridge.getWhiteBalancePresetValues(preset);
        if (values) {
          this.edits.temperature = values[0];
          this.edits.tint = values[1];
        }
      }
      this.inspectorController.syncSliderUI();
      this.renderCanvas();
      this.autoSaveSidecar();
    });
  }

  private async selectPhoto(
    idx: number,
    centerFilmstrip: boolean = false,
    keepMultiSelection: boolean = false,
  ) {
    const entries = this.filmstripController.getEntries();
    if (idx < 0 || idx >= entries.length) return;

    // Save previous active photo edits and auto-save sidecar
    const prevIdx = this.filmstripController.getActiveIndex();
    if (prevIdx >= 0 && prevIdx < entries.length && prevIdx !== idx) {
      const prevEntry = entries[prevIdx];
      await this.saveActivePhotoState(prevEntry);
    }

    const requestId = ++this.selectRequestId;
    this.filmstripController.selectIndex(
      idx,
      centerFilmstrip,
      keepMultiSelection,
    );
    this.inspectorController.rollbackActiveScrub();

    const entry = entries[idx];
    if (this.elements['meta-filename']) {
      this.elements['meta-filename'].innerText = entry.name;
    }
    try {
      localStorage.setItem('fp_recent_photo_name', entry.name);
    } catch (_) {}

    // Reset crop / zoom tool state immediately
    if (this.isCropActive) {
      this.cropController.hide();
      this.isCropActive = false;
      this.elements['btn-crop-tool']?.classList.remove('active');
    }
    this.viewportController.resetZoom();

    // Stage 0: Up-Front Edit & Metadata Staging
    const ext = entry.name.split('.').pop()?.toLowerCase() || '';
    const isRawFile = [
      'cr2',
      'cr3',
      'nef',
      'arw',
      'dng',
      'rw2',
      'orf',
      'pef',
      'raf',
    ].includes(ext);

    const knownRot = this.loaderService.getCachedExif(entry.name);
    const defaultRot = knownRot?.rotation ?? 0;
    const defaultFlipH = knownRot?.flipH ?? false;
    const defaultFlipV = knownRot?.flipV ?? false;

    const record = await this.loaderService.loadPhotoRecord(
      entry,
      isRawFile,
      defaultRot,
      defaultFlipH,
      defaultFlipV,
    );

    if (requestId !== this.selectRequestId) return;

    this.edits = { ...record.edits };
    this.currentRating = record.rating;
    this.currentStatus = record.status;
    this.setMetadataFormFields(record);

    this.inspectorController.syncSliderUI();
    this.syncRatingFlagUI();
    if (this.edits.rotation !== undefined) {
      this.filmstripController.updateThumbnailCardOrientation(
        idx,
        this.edits.rotation,
        this.edits.flip_horizontal,
        this.edits.flip_vertical,
      );
    }
    this.updateSidecarStatusBadge(record.hasSidecarOnDisk);

    // Check High-Resolution Intermediate Texture Cache
    const cachedIntermediate = this.loaderService.getIntermediate(
      entry.name,
      this.edits,
    );
    if (cachedIntermediate) {
      this.elements['developing-spinner']?.classList.add('hidden');
      this.activeLinearData = null;
      this.activeLinearW = 0;
      this.activeLinearH = 0;
      this.activeSourceImage = null;
      this.activeSourceW = cachedIntermediate.width;
      this.activeSourceH = cachedIntermediate.height;

      this.renderer.uploadIntermediate(cachedIntermediate.bitmap);
      this.viewportController.updateCanvasDimensions();
      this.renderCanvas();
    } else {
      this.renderer.clearCanvas();
      this.elements['developing-spinner']?.classList.remove('hidden');
    }

    if (this.selectDebounceTimer) {
      clearTimeout(this.selectDebounceTimer);
      this.selectDebounceTimer = null;
    }

    const isPrewarmedHit = this.loaderService.hasPrewarmed(idx);
    const delay = isPrewarmedHit || cachedIntermediate ? 0 : 35;

    this.selectDebounceTimer = setTimeout(async () => {
      if (requestId !== this.selectRequestId) return;
      if (!cachedIntermediate) {
        this.elements['developing-spinner']?.classList.remove('hidden');
      }

      try {
        const frame = await this.loaderService.loadPhotoFrame(
          entry,
          isRawFile,
          idx,
        );
        if (requestId !== this.selectRequestId) return;

        this.currentMetadata = frame.preview.metadata;
        const baselineSharpness =
          frame.preview.metadata?.baseline_sharpness ?? 1.0;
        this.renderer.setBaselineSharpness(baselineSharpness);
        updateMetadataHeaderElements(this.elements, frame.preview, entry.name);

        const orientation = frame.preview.metadata?.orientation ?? 1;
        const exifOri = decodeExifOrientation(orientation);
        this.loaderService.setCachedExif(entry.name, {
          rotation: exifOri.rotation,
          flipH: exifOri.flipH,
          flipV: exifOri.flipV,
        });

        if (frame.linearData && frame.linearW > 0 && frame.linearH > 0) {
          this.activeLinearData = frame.linearData;
          this.activeLinearW = frame.linearW;
          this.activeLinearH = frame.linearH;
          this.activeSourceImage = null;
          this.activeSourceW = 0;
          this.activeSourceH = 0;
          this.renderer.uploadLinearRec2020Texture(
            frame.linearW,
            frame.linearH,
            frame.linearData,
          );
        } else {
          this.activeLinearData = null;
          this.activeLinearW = 0;
          this.activeLinearH = 0;
          this.activeSourceImage = frame.imgBitmap;
          this.activeSourceW = frame.imgBitmap ? frame.imgBitmap.width : 1920;
          this.activeSourceH = frame.imgBitmap ? frame.imgBitmap.height : 1080;
          if (frame.imgBitmap) {
            this.renderer.uploadImage(frame.imgBitmap);
          }
        }

        this.viewportController.updateCanvasDimensions();
        this.onCurveChange();

        this.loaderService.scheduleIntermediateSnapshot(
          entry.name,
          this.edits,
          this.renderer,
          this.viewportController.getZoom() === 1.0 &&
            !this.isCropActive &&
            !this.isOriginalPeek,
        );

        // Prewarm adjacent frames
        if (idx > 0) this.loaderService.prewarmPhoto(idx - 1, entries[idx - 1]);
        if (idx < entries.length - 1)
          this.loaderService.prewarmPhoto(idx + 1, entries[idx + 1]);
      } catch (err) {
        if (requestId === this.selectRequestId) {
          console.error(`[App] Error developing photo ${entry.name}:`, err);
        }
      } finally {
        if (requestId === this.selectRequestId) {
          this.elements['developing-spinner']?.classList.add('hidden');
        }
      }
    }, delay);
  }

  private setMetadataFormFields(xmp: Partial<XmpPayload>) {
    if (this.elements['meta-input-title']) {
      (this.elements['meta-input-title'] as HTMLInputElement).value =
        xmp.title || '';
    }
    if (this.elements['meta-input-caption']) {
      (this.elements['meta-input-caption'] as HTMLTextAreaElement).value =
        xmp.description || '';
    }
    if (this.elements['meta-input-creator']) {
      (this.elements['meta-input-creator'] as HTMLInputElement).value =
        xmp.creator || '';
    }
    if (this.elements['meta-input-keywords']) {
      (this.elements['meta-input-keywords'] as HTMLInputElement).value = (
        xmp.keywords || []
      ).join(', ');
    }
  }

  private updateSidecarStatusBadge(synced: boolean) {
    const el = this.elements['meta-sidecar-status'];
    if (el) {
      el.innerText = synced ? '.xmp Synced' : 'Session RAM';
      el.style.color = synced ? '#4ade80' : 'var(--text-muted)';
    }
  }

  private onCurveChange() {
    this.edits.curves = this.curveWidget.getPoints();
    const lut = this.bridge.evaluateCurves(this.edits.curves);
    this.renderer.updateToneCurvesLut(lut);
    this.renderCanvas();
  }

  public selectInspectorTab(tab: string) {
    this.inspectorController.selectInspectorTab(tab);
  }

  public resetAllEdits() {
    const orientation = this.currentMetadata?.orientation ?? 1;
    const exifOri = decodeExifOrientation(orientation);
    const isRaw = this.isCurrentPhotoRaw();
    this.edits = getDefaultEdits(isRaw);
    this.edits.rotation = exifOri.rotation;
    this.edits.flip_horizontal = exifOri.flipH;
    this.edits.flip_vertical = exifOri.flipV;

    const idx = this.filmstripController.getActiveIndex();
    if (idx >= 0) {
      this.filmstripController.updateThumbnailCardOrientation(
        idx,
        exifOri.rotation,
        exifOri.flipH,
        exifOri.flipV,
      );
    }
    if (this.isCropActive) {
      this.cropController.resetCrop();
    }
    this.viewportController.resetZoom();
    this.curveWidget.resetAllChannels();
    this.inspectorController.syncSliderUI();
    this.viewportController.updateCanvasDimensions();
    this.renderCanvas();
    this.autoSaveSidecar();
    console.log(
      `⚡ [App] Reset all edits to as-shot defaults (isRaw: ${isRaw})`,
    );
  }

  public rotateImage(deltaDeg: number) {
    const idx = this.filmstripController.getActiveIndex();
    if (idx < 0) return;
    const curRot = this.edits.rotation ?? 0;
    let newRot = (curRot + deltaDeg) % 360;
    if (newRot < 0) newRot += 360;
    this.edits.rotation = newRot;
    this.filmstripController.updateThumbnailCardOrientation(
      idx,
      newRot,
      this.edits.flip_horizontal,
      this.edits.flip_vertical,
    );
    this.viewportController.updateCanvasDimensions();
    this.renderCanvas();
    this.autoSaveSidecar();
    if (this.isCropActive) {
      this.updateCropOverlayPosition();
    }
  }

  public toggleCropMode(force?: boolean) {
    const idx = this.filmstripController.getActiveIndex();
    if (idx < 0) return;
    const nextState = force !== undefined ? force : !this.isCropActive;
    if (this.isCropActive === nextState) return;
    this.isCropActive = nextState;

    const btnCrop = this.elements['btn-crop-tool'];
    btnCrop?.classList.toggle('active', this.isCropActive);

    if (this.isCropActive) {
      this.viewportController.resetZoom();
      this.viewportController.updateCanvasDimensions();
      this.renderCanvas();

      requestAnimationFrame(() => {
        this.updateCropOverlayPosition();
        const isOrthogonal =
          this.edits.rotation === 90 || this.edits.rotation === 270;
        const w = isOrthogonal
          ? this.activeLinearH || this.activeSourceH || 1
          : this.activeLinearW || this.activeSourceW || 1;
        const h = isOrthogonal
          ? this.activeLinearW || this.activeSourceW || 1
          : this.activeLinearH || this.activeSourceH || 1;
        const aspect = w / h;
        this.cropController.show(
          {
            left: this.edits.crop_left ?? 0.0,
            top: this.edits.crop_top ?? 0.0,
            right: this.edits.crop_right ?? 1.0,
            bottom: this.edits.crop_bottom ?? 1.0,
          },
          aspect,
          this.edits.crop_angle ?? 0.0,
        );
      });
    } else {
      this.cropController.hide();
      this.viewportController.updateCanvasDimensions();
      this.renderCanvas();
      this.autoSaveSidecar();
    }
  }

  private updateCropOverlayPosition() {
    const wrapper = document.getElementById('canvas-wrapper') as HTMLElement;
    if (!wrapper) return;

    const imgW = this.activeLinearW || this.activeSourceW || 1920;
    const imgH = this.activeLinearH || this.activeSourceH || 1080;
    const isOrthogonal =
      this.edits.rotation === 90 || this.edits.rotation === 270;
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

    this.cropController.setImageRect(
      { x: left, y: top, width: renderW, height: renderH },
      imgAspect,
    );
  }

  private renderCanvas() {
    if (this.filmstripController.getActiveIndex() >= 0) {
      this.renderer.render(
        this.edits,
        this.isOriginalPeek,
        this.viewportController.getZoom(),
        this.viewportController.getPan(),
        this.isCropActive,
      );
      this.updatePeekBadge();
    }
  }

  private updatePeekBadge() {
    const badge = this.elements['peek-badge'];
    if (badge) {
      badge.classList.toggle('visible', this.isOriginalPeek);
    }
  }

  private syncRatingFlagUI() {
    const idx = this.filmstripController.getActiveIndex();
    this.inspectorController.syncRatingFlagUI(
      this.currentRating,
      this.currentStatus,
      idx,
    );
    this.filmstripController.updateRatingFlagBadge(
      idx,
      this.currentRating,
      this.currentStatus,
    );
  }

  private getActivePhotoRecord(): PhotoRecord {
    const title =
      (this.elements['meta-input-title'] as HTMLInputElement)?.value.trim() ||
      '';
    const description =
      (
        this.elements['meta-input-caption'] as HTMLTextAreaElement
      )?.value.trim() || '';
    const creator =
      (this.elements['meta-input-creator'] as HTMLInputElement)?.value.trim() ||
      '';
    const kwStr = (this.elements['meta-input-keywords'] as HTMLInputElement)
      ?.value;
    const keywords = kwStr
      ? kwStr
          .split(',')
          .map((s) => s.trim())
          .filter(Boolean)
      : [];

    const activeEntry = this.filmstripController.getActiveEntry();
    const existing = activeEntry
      ? this.loaderService.getPhotoRecord(activeEntry.name)
      : undefined;

    return {
      edits: { ...this.edits },
      rating: this.currentRating,
      status: this.currentStatus,
      title,
      description,
      creator,
      keywords,
      hasSidecarOnDisk: existing?.hasSidecarOnDisk ?? false,
    };
  }

  private async saveActivePhotoState(entry: PhotoFileEntry) {
    const record = this.getActivePhotoRecord();
    const orientation = this.currentMetadata?.orientation ?? 1;
    const saved = await this.loaderService.autoSaveSidecar(
      entry,
      record,
      orientation,
    );
    if (saved) {
      this.updateSidecarStatusBadge(true);
    }
  }

  private async autoSaveSidecar() {
    const entry = this.filmstripController.getActiveEntry();
    if (!entry) return;

    this.loaderService.scheduleIntermediateSnapshot(
      entry.name,
      this.edits,
      this.renderer,
      this.viewportController.getZoom() === 1.0 &&
        !this.isCropActive &&
        !this.isOriginalPeek,
    );

    await this.saveActivePhotoState(entry);
  }

  private openExportModal() {
    const photos = this.filmstripController.getSelectedPhotos();
    if (photos.length === 0) return;
    this.exportModal.open(photos);
  }

  private async handleExport(
    photos: PhotoFileEntry[],
    options: ExportExecutionOptions,
    onProgress: (status: string, current: number, total: number) => void,
  ): Promise<{ success: boolean; message: string }> {
    const activeEntry = this.filmstripController.getActiveEntry();
    return this.loaderService.handleExport(
      photos,
      activeEntry?.name ?? null,
      this.edits,
      (isRaw) => getDefaultEdits(isRaw),
      options,
      onProgress,
    );
  }

  private getSceneLuminances(): Float32Array {
    const canvas = document.getElementById(
      'darkroom-canvas',
    ) as HTMLCanvasElement;
    const gl = canvas?.getContext('webgl2') as WebGL2RenderingContext;
    if (!gl) return new Float32Array(0);

    const sampleW = Math.min(100, canvas.width);
    const sampleH = Math.min(100, canvas.height);
    const startX = Math.max(0, Math.floor((canvas.width - sampleW) / 2));
    const startY = Math.max(0, Math.floor((canvas.height - sampleH) / 2));
    const pixels = new Uint8Array(sampleW * sampleH * 4);
    gl.readPixels(
      startX,
      startY,
      sampleW,
      sampleH,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      pixels,
    );

    const lums = new Float32Array(sampleW * sampleH);
    for (let i = 0; i < lums.length; i++) {
      const r = pixels[i * 4] / 255.0;
      const g = pixels[i * 4 + 1] / 255.0;
      const b = pixels[i * 4 + 2] / 255.0;
      lums[i] = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    }
    return lums;
  }

  public applyAutoTone() {
    if (this.filmstripController.getActiveIndex() < 0) return;

    let adj: { exposure: number; contrast: number } | null = null;
    if (
      this.activeLinearData &&
      this.activeLinearW > 0 &&
      this.activeLinearH > 0
    ) {
      adj = this.bridge.computeAutoAdjustments(
        this.activeLinearData,
        this.activeLinearW,
        this.activeLinearH,
      );
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
      this.inspectorController.syncSliderUI();
      this.renderCanvas();
      this.autoSaveSidecar();
    }
  }

  public applyAutoWhiteBalance() {
    if (this.filmstripController.getActiveIndex() < 0) return;

    let wb: { temperature: number; tint: number } | null = null;
    if (
      this.activeLinearData &&
      this.activeLinearW > 0 &&
      this.activeLinearH > 0
    ) {
      wb = this.bridge.computeAutoWhiteBalance(
        this.activeLinearData,
        this.activeLinearW,
        this.activeLinearH,
      );
    }

    if (!wb) {
      const canvas = document.getElementById(
        'darkroom-canvas',
      ) as HTMLCanvasElement;
      const gl = canvas?.getContext('webgl2') as WebGL2RenderingContext;
      if (gl) {
        const readW = Math.min(100, canvas.width);
        const readH = Math.min(100, canvas.height);
        const pixels = new Uint8Array(readW * readH * 4);
        gl.readPixels(0, 0, readW, readH, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        let rSum = 0,
          gSum = 0,
          bSum = 0,
          count = 0;
        for (let i = 0; i < readW * readH; i++) {
          const r = pixels[i * 4] / 255.0;
          const g = pixels[i * 4 + 1] / 255.0;
          const b = pixels[i * 4 + 2] / 255.0;
          const y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
          if (y >= 0.05 && y <= 0.85) {
            rSum += r;
            gSum += g;
            bSum += b;
            count++;
          }
        }
        if (count > 20) {
          wb = this.bridge.calculateWhiteBalanceFromSample(
            rSum / count,
            gSum / count,
            bSum / count,
          );
        }
      }
    }

    if (wb) {
      this.edits.temperature = wb.temperature;
      this.edits.tint = wb.tint;
      this.inspectorController.syncSliderUI();
      this.renderCanvas();
      this.autoSaveSidecar();
    }
  }
}

// Instantiate and start app when DOM is ready
window.addEventListener('DOMContentLoaded', () => {
  const app = new DarkroomApp();
  app.init();
});
