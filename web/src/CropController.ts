/**
 * Interactive Crop Controller for First Proof Web Darkroom
 *
 * Implements a stationary full-canvas gesture container strictly adhering to
 * First Proof Desktop Gesture Architecture & Hit Testing invariants:
 * - Visual handles are purely declarative render elements with pointer-events: none.
 * - Single stationary gesture surface covering the full viewport container.
 * - Outside-crop drag initiates straighten rotation with live floating HUD readout.
 * - Corner and edge aspect-ratio constrained dragging matching native macOS CropOverlayView.swift.
 * - Aspect ratio presets (Original, Free, 1:1, 3:2, 4:3, 16:9) with automatic portrait inversion.
 * - Continuous smooth updates with 60fps SVG rendering.
 */

export interface CropRect {
  left: number; // 0.0 .. 1.0 (normalized)
  top: number; // 0.0 .. 1.0
  right: number; // 0.0 .. 1.0
  bottom: number; // 0.0 .. 1.0
}

export type AspectRatioMode =
  | 'original'
  | 'free'
  | '1:1'
  | '3:2'
  | '4:3'
  | '16:9';

type DragHandle =
  | 'nw'
  | 'ne'
  | 'sw'
  | 'se'
  | 'n'
  | 's'
  | 'e'
  | 'w'
  | 'move'
  | 'rotate'
  | null;

export class CropController {
  private overlayEl: HTMLElement;
  private toolbarEl: HTMLElement | null = null;
  private gestureSurface: HTMLElement;
  private maskCutout: SVGRectElement;
  private boxBorder: SVGRectElement;
  private gridH1: SVGLineElement;
  private gridH2: SVGLineElement;
  private gridV1: SVGLineElement;
  private gridV2: SVGLineElement;
  private handleElements: Record<string, SVGElement>;
  private aspectSelect: HTMLSelectElement;
  private btnFlip: HTMLButtonElement | null = null;
  private btnReset: HTMLButtonElement;
  private btnDone: HTMLButtonElement;
  private straightenHudEl: HTMLElement | null = null;
  private straightenHudValueEl: HTMLElement | null = null;
  private toolbarStraightenSlider: HTMLInputElement | null = null;
  private toolbarStraightenVal: HTMLElement | null = null;

  private crop: CropRect = { left: 0.0, top: 0.0, right: 1.0, bottom: 1.0 };
  private imageAspect: number = 1.5; // width / height
  private aspectMode: AspectRatioMode = 'original';
  private isFlippedAspect: boolean = false;
  private imageRect: { x: number; y: number; width: number; height: number } = {
    x: 0,
    y: 0,
    width: 800,
    height: 600,
  };
  private straightenAngle: number = 0.0;
  private dragStartStraighten: number = 0.0;
  public isRotating: boolean = false;

  private activeHandle: DragHandle = null;
  private dragStartPos: { x: number; y: number } = { x: 0, y: 0 };
  private dragStartCrop: CropRect = { left: 0, top: 0, right: 1, bottom: 1 };

  private onCropUpdateCallback: ((crop: CropRect) => void) | null = null;
  private onStraightenUpdateCallback: ((angleDegrees: number) => void) | null =
    null;
  private onStraightenEndCallback: (() => void) | null = null;
  private onDoneCallback: (() => void) | null = null;
  private onCancelCallback: (() => void) | null = null;

  constructor(
    overlayId: string = 'crop-overlay',
    callbacks?: {
      onCropUpdate?: (crop: CropRect) => void;
      onStraightenUpdate?: (angleDegrees: number) => void;
      onStraightenEnd?: () => void;
      onDone?: () => void;
      onCancel?: () => void;
    },
  ) {
    this.overlayEl = document.getElementById(overlayId) as HTMLElement;
    this.toolbarEl = document.getElementById('crop-toolbar') as HTMLElement;
    this.gestureSurface = document.getElementById(
      'crop-gesture-surface',
    ) as HTMLElement;
    this.maskCutout = document.getElementById(
      'crop-mask-cutout',
    ) as unknown as SVGRectElement;
    this.boxBorder = document.getElementById(
      'crop-box-border',
    ) as unknown as SVGRectElement;
    this.gridH1 = document.getElementById(
      'crop-grid-h1',
    ) as unknown as SVGLineElement;
    this.gridH2 = document.getElementById(
      'crop-grid-h2',
    ) as unknown as SVGLineElement;
    this.gridV1 = document.getElementById(
      'crop-grid-v1',
    ) as unknown as SVGLineElement;
    this.gridV2 = document.getElementById(
      'crop-grid-v2',
    ) as unknown as SVGLineElement;

    this.handleElements = {
      nw: document.getElementById('handle-tl') as unknown as SVGElement,
      ne: document.getElementById('handle-tr') as unknown as SVGElement,
      sw: document.getElementById('handle-bl') as unknown as SVGElement,
      se: document.getElementById('handle-br') as unknown as SVGElement,
      n: document.getElementById('handle-t') as unknown as SVGElement,
      s: document.getElementById('handle-b') as unknown as SVGElement,
      w: document.getElementById('handle-l') as unknown as SVGElement,
      e: document.getElementById('handle-r') as unknown as SVGElement,
    };

    this.aspectSelect = document.getElementById(
      'select-crop-aspect',
    ) as HTMLSelectElement;
    this.btnFlip = document.getElementById(
      'btn-crop-flip',
    ) as HTMLButtonElement;
    this.btnReset = document.getElementById(
      'btn-crop-reset',
    ) as HTMLButtonElement;
    this.btnDone = document.getElementById(
      'btn-crop-done',
    ) as HTMLButtonElement;
    this.straightenHudEl = document.getElementById('straighten-hud');
    this.straightenHudValueEl = document.getElementById('straighten-hud-value');
    this.toolbarStraightenSlider = document.getElementById(
      'crop-toolbar-straighten-slider',
    ) as HTMLInputElement | null;
    this.toolbarStraightenVal = document.getElementById(
      'crop-toolbar-straighten-val',
    );

    if (callbacks?.onCropUpdate)
      this.onCropUpdateCallback = callbacks.onCropUpdate;
    if (callbacks?.onStraightenUpdate)
      this.onStraightenUpdateCallback = callbacks.onStraightenUpdate;
    if (callbacks?.onStraightenEnd)
      this.onStraightenEndCallback = callbacks.onStraightenEnd;
    if (callbacks?.onDone) this.onDoneCallback = callbacks.onDone;
    if (callbacks?.onCancel) this.onCancelCallback = callbacks.onCancel;

    this.initEvents();
  }

  private initEvents() {
    this.gestureSurface?.addEventListener(
      'pointerdown',
      this.onPointerDown.bind(this),
    );
    window.addEventListener('pointermove', this.onPointerMove.bind(this));
    window.addEventListener('pointerup', this.onPointerUp.bind(this));
    window.addEventListener('pointercancel', this.onPointerUp.bind(this));

    this.aspectSelect?.addEventListener('change', () => {
      this.aspectMode = this.aspectSelect.value as AspectRatioMode;
      this.applyAspectConstraint();
      this.render();
      if (this.onCropUpdateCallback)
        this.onCropUpdateCallback({ ...this.crop });
    });

    this.btnFlip?.addEventListener('click', () => {
      this.flipCropOrientation();
    });

    this.toolbarStraightenSlider?.addEventListener('input', (e) => {
      const val = parseFloat((e.target as HTMLInputElement).value) || 0.0;
      this.straightenAngle = val;
      if (this.toolbarStraightenVal) {
        this.toolbarStraightenVal.textContent = `${val > 0 ? '+' : ''}${val.toFixed(1)}°`;
      }
      if (this.onStraightenUpdateCallback) {
        this.onStraightenUpdateCallback(val);
      }
    });

    this.toolbarStraightenSlider?.addEventListener('change', () => {
      if (this.onStraightenEndCallback) {
        this.onStraightenEndCallback();
      }
    });

    this.toolbarStraightenSlider?.addEventListener('dblclick', () => {
      this.straightenAngle = 0.0;
      if (this.toolbarStraightenSlider)
        this.toolbarStraightenSlider.value = '0';
      if (this.toolbarStraightenVal)
        this.toolbarStraightenVal.textContent = '+0.0°';
      if (this.onStraightenUpdateCallback) {
        this.onStraightenUpdateCallback(0.0);
      }
      if (this.onStraightenEndCallback) {
        this.onStraightenEndCallback();
      }
    });

    this.btnReset?.addEventListener('click', () => {
      this.resetCrop();
    });

    this.btnDone?.addEventListener('click', () => {
      if (this.onDoneCallback) this.onDoneCallback();
    });
  }

  private updateAspectOptionLabels() {
    if (!this.aspectSelect) return;
    const isPortrait = this.imageAspect < 1.0;
    let portrait = isPortrait;
    if (this.isFlippedAspect) portrait = !portrait;

    const optOriginal = this.aspectSelect.querySelector(
      'option[value="original"]',
    );
    const opt32 = this.aspectSelect.querySelector('option[value="3:2"]');
    const opt43 = this.aspectSelect.querySelector('option[value="4:3"]');
    const opt169 = this.aspectSelect.querySelector('option[value="16:9"]');

    if (optOriginal)
      optOriginal.textContent = portrait ? 'Original (Vertical)' : 'Original';
    if (opt32) opt32.textContent = portrait ? '2:3 (35mm)' : '3:2 (35mm)';
    if (opt43) opt43.textContent = portrait ? '3:4' : '4:3';
    if (opt169) opt169.textContent = portrait ? '9:16' : '16:9';
  }

  public show(
    crop: CropRect,
    imageAspect: number,
    straightenAngle: number = 0.0,
  ) {
    this.crop = {
      left: Math.max(0, Math.min(0.95, crop.left ?? 0.0)),
      top: Math.max(0, Math.min(0.95, crop.top ?? 0.0)),
      right: Math.max(0.05, Math.min(1.0, crop.right ?? 1.0)),
      bottom: Math.max(0.05, Math.min(1.0, crop.bottom ?? 1.0)),
    };
    this.imageAspect = imageAspect > 0 ? imageAspect : 1.5;
    this.straightenAngle = straightenAngle;
    if (this.toolbarStraightenSlider) {
      this.toolbarStraightenSlider.value = `${straightenAngle}`;
    }
    if (this.toolbarStraightenVal) {
      this.toolbarStraightenVal.textContent = `${straightenAngle > 0 ? '+' : ''}${straightenAngle.toFixed(1)}°`;
    }
    this.isFlippedAspect = false;
    this.updateAspectOptionLabels();
    this.overlayEl.classList.remove('hidden');
    this.toolbarEl?.classList.remove('hidden');
    this.render();
  }

  public hide() {
    this.overlayEl.classList.add('hidden');
    this.toolbarEl?.classList.add('hidden');
    if (this.straightenHudEl) {
      this.straightenHudEl.classList.add('hidden');
    }
    this.isRotating = false;
    this.activeHandle = null;
  }

  public setImageRect(
    rect: { x: number; y: number; width: number; height: number },
    imageAspect?: number,
  ) {
    this.imageRect = { ...rect };
    if (imageAspect && imageAspect > 0) {
      this.imageAspect = imageAspect;
      this.updateAspectOptionLabels();
    }
    this.render();
  }

  public updatePosition(overlayRect: {
    left: number;
    top: number;
    width: number;
    height: number;
  }) {
    this.setImageRect({
      x: overlayRect.left,
      y: overlayRect.top,
      width: overlayRect.width,
      height: overlayRect.height,
    });
  }

  public setStraightenAngle(angle: number) {
    this.straightenAngle = angle;
    if (this.toolbarStraightenSlider) {
      this.toolbarStraightenSlider.value = `${angle}`;
    }
    if (this.toolbarStraightenVal) {
      this.toolbarStraightenVal.textContent = `${angle > 0 ? '+' : ''}${angle.toFixed(1)}°`;
    }
  }

  public cancelRotation(): boolean {
    if (this.isRotating) {
      this.isRotating = false;
      this.activeHandle = null;
      this.straightenAngle = this.dragStartStraighten;
      if (this.straightenHudEl) this.straightenHudEl.classList.add('hidden');
      if (this.toolbarStraightenSlider) {
        this.toolbarStraightenSlider.value = `${this.dragStartStraighten}`;
      }
      if (this.toolbarStraightenVal) {
        const sign =
          this.dragStartStraighten > 0
            ? '+'
            : this.dragStartStraighten < 0
              ? ''
              : '+';
        this.toolbarStraightenVal.textContent = `${sign}${this.dragStartStraighten.toFixed(1)}°`;
      }
      if (this.onStraightenUpdateCallback) {
        this.onStraightenUpdateCallback(this.dragStartStraighten);
      }
      return true;
    }
    return false;
  }

  public cancel(): void {
    if (this.onCancelCallback) {
      this.onCancelCallback();
    }
  }

  public flipCropOrientation() {
    this.isFlippedAspect = !this.isFlippedAspect;
    this.updateAspectOptionLabels();

    const currentW = this.crop.right - this.crop.left;
    const currentH = this.crop.bottom - this.crop.top;
    const centerX = (this.crop.left + this.crop.right) / 2.0;
    const centerY = (this.crop.top + this.crop.bottom) / 2.0;

    const currentPhotoAspect =
      (currentW / Math.max(currentH, 0.001)) * this.imageAspect;
    const newPhotoAspect = 1.0 / Math.max(currentPhotoAspect, 0.01);
    const newNormAspect = newPhotoAspect / this.imageAspect;

    let newW = currentH * newNormAspect;
    let newH = currentH;
    if (newW > 1.0) {
      newW = 1.0;
      newH = newW / Math.max(newNormAspect, 0.001);
    }
    if (newH > 1.0) {
      newH = 1.0;
      newW = newH * newNormAspect;
    }

    const newLeft = Math.max(0, centerX - newW / 2.0);
    const newRight = Math.min(1.0, newLeft + newW);
    const newTop = Math.max(0, centerY - newH / 2.0);
    const newBottom = Math.min(1.0, newTop + newH);

    this.crop = {
      left: newLeft,
      top: newTop,
      right: newRight,
      bottom: newBottom,
    };
    this.render();
    if (this.onCropUpdateCallback) {
      this.onCropUpdateCallback({ ...this.crop });
    }
  }

  public resetCrop() {
    this.isFlippedAspect = false;
    this.straightenAngle = 0.0;
    this.aspectMode = 'original';
    if (this.aspectSelect) this.aspectSelect.value = 'original';
    if (this.toolbarStraightenSlider) this.toolbarStraightenSlider.value = '0';
    if (this.toolbarStraightenVal)
      this.toolbarStraightenVal.textContent = '+0.0°';
    this.updateAspectOptionLabels();
    this.crop = { left: 0.0, top: 0.0, right: 1.0, bottom: 1.0 };
    this.render();
    if (this.onStraightenUpdateCallback) this.onStraightenUpdateCallback(0.0);
    if (this.onStraightenEndCallback) this.onStraightenEndCallback();
    if (this.onCropUpdateCallback) this.onCropUpdateCallback({ ...this.crop });
  }

  public getCrop(): CropRect {
    return { ...this.crop };
  }

  private resolveHitTarget(clientX: number, clientY: number): DragHandle {
    const rect = this.gestureSurface.getBoundingClientRect();
    if (
      rect.width <= 0 ||
      rect.height <= 0 ||
      this.imageRect.width <= 0 ||
      this.imageRect.height <= 0
    )
      return null;

    const px = clientX - rect.left;
    const py = clientY - rect.top;

    const xL = this.imageRect.x + this.crop.left * this.imageRect.width;
    const xR = this.imageRect.x + this.crop.right * this.imageRect.width;
    const yT = this.imageRect.y + this.crop.top * this.imageRect.height;
    const yB = this.imageRect.y + this.crop.bottom * this.imageRect.height;

    const cornerRadius = 24; // Corner resize handles
    const edgeRadius = 16; // Edge resize handles
    const cornerRotateRadius = 60; // Outer corner rotation zone

    // 1. Check corners (resize)
    if (Math.hypot(px - xL, py - yT) <= cornerRadius) return 'nw';
    if (Math.hypot(px - xR, py - yT) <= cornerRadius) return 'ne';
    if (Math.hypot(px - xL, py - yB) <= cornerRadius) return 'sw';
    if (Math.hypot(px - xR, py - yB) <= cornerRadius) return 'se';

    // 2. Check edges (resize)
    if (
      Math.abs(py - yT) <= edgeRadius &&
      px >= xL - edgeRadius &&
      px <= xR + edgeRadius
    )
      return 'n';
    if (
      Math.abs(py - yB) <= edgeRadius &&
      px >= xL - edgeRadius &&
      px <= xR + edgeRadius
    )
      return 's';
    if (
      Math.abs(px - xL) <= edgeRadius &&
      py >= yT - edgeRadius &&
      py <= yB + edgeRadius
    )
      return 'w';
    if (
      Math.abs(px - xR) <= edgeRadius &&
      py >= yT - edgeRadius &&
      py <= yB + edgeRadius
    )
      return 'e';

    // 3. Check corner rotation zones (outer margin around 4 corners)
    if (Math.hypot(px - xL, py - yT) <= cornerRotateRadius) return 'rotate';
    if (Math.hypot(px - xR, py - yT) <= cornerRotateRadius) return 'rotate';
    if (Math.hypot(px - xL, py - yB) <= cornerRotateRadius) return 'rotate';
    if (Math.hypot(px - xR, py - yB) <= cornerRotateRadius) return 'rotate';

    // 4. Inside crop box -> move (only if not full frame)
    if (px > xL && px < xR && py > yT && py < yB) {
      const isFull =
        this.crop.left <= 0.001 &&
        this.crop.top <= 0.001 &&
        this.crop.right >= 0.999 &&
        this.crop.bottom >= 0.999;
      return isFull ? null : 'move';
    }

    // 5. Safe empty canvas -> null (no accidental rotate trigger)
    return null;
  }

  private updateCursor(handle: DragHandle) {
    switch (handle) {
      case 'nw':
      case 'se':
        this.gestureSurface.style.cursor = 'nwse-resize';
        break;
      case 'ne':
      case 'sw':
        this.gestureSurface.style.cursor = 'nesw-resize';
        break;
      case 'n':
      case 's':
        this.gestureSurface.style.cursor = 'ns-resize';
        break;
      case 'w':
      case 'e':
        this.gestureSurface.style.cursor = 'ew-resize';
        break;
      case 'move':
        this.gestureSurface.style.cursor = 'move';
        break;
      case 'rotate':
        this.gestureSurface.style.cursor =
          "url(\"data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24'><g fill='none' stroke='%23000000' stroke-width='3.5' stroke-linecap='round' stroke-linejoin='round'><path d='M21.5 2v6h-6'/><path d='M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67'/></g><g fill='none' stroke='%23ffffff' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M21.5 2v6h-6'/><path d='M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67'/></g></svg>\") 12 12, crosshair";
        break;
      default:
        this.gestureSurface.style.cursor = 'default';
        break;
    }
  }

  private onPointerDown(e: PointerEvent) {
    if (e.button !== 0) return; // Primary button only
    const handle = this.resolveHitTarget(e.clientX, e.clientY);
    if (!handle) return;

    this.activeHandle = handle;
    this.dragStartPos = { x: e.clientX, y: e.clientY };
    this.dragStartCrop = { ...this.crop };
    this.dragStartStraighten = this.straightenAngle;

    if (handle === 'rotate') {
      this.isRotating = true;
    }

    this.updateCursor(handle);
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
    e.preventDefault();
  }

  private onPointerMove(e: PointerEvent) {
    if (!this.activeHandle) {
      const handle = this.resolveHitTarget(e.clientX, e.clientY);
      this.updateCursor(handle);
      return;
    }

    const rect = this.gestureSurface.getBoundingClientRect();
    if (
      rect.width <= 0 ||
      rect.height <= 0 ||
      this.imageRect.width <= 0 ||
      this.imageRect.height <= 0
    )
      return;

    if (this.activeHandle === 'rotate') {
      const centerX = this.imageRect.x + this.imageRect.width / 2.0;
      const centerY = this.imageRect.y + this.imageRect.height / 2.0;
      const startVector = {
        x: this.dragStartPos.x - rect.left - centerX,
        y: this.dragStartPos.y - rect.top - centerY,
      };
      const currentVector = {
        x: e.clientX - rect.left - centerX,
        y: e.clientY - rect.top - centerY,
      };
      const startAngle = Math.atan2(startVector.y, startVector.x);
      const currentAngle = Math.atan2(currentVector.y, currentVector.x);
      let deltaDegrees = ((currentAngle - startAngle) * 180.0) / Math.PI;
      if (deltaDegrees > 180.0) deltaDegrees -= 360.0;
      if (deltaDegrees < -180.0) deltaDegrees += 360.0;

      const newStraighten = Math.max(
        -45.0,
        Math.min(45.0, this.dragStartStraighten + deltaDegrees),
      );
      this.straightenAngle = newStraighten;

      if (this.straightenHudEl) {
        this.straightenHudEl.classList.remove('hidden');
      }
      if (this.straightenHudValueEl) {
        const sign = newStraighten > 0 ? '+' : newStraighten < 0 ? '' : '+';
        this.straightenHudValueEl.textContent = `Straighten: ${sign}${newStraighten.toFixed(1)}°`;
      }
      if (this.toolbarStraightenSlider) {
        this.toolbarStraightenSlider.value = `${newStraighten}`;
      }
      if (this.toolbarStraightenVal) {
        const sign = newStraighten > 0 ? '+' : newStraighten < 0 ? '' : '+';
        this.toolbarStraightenVal.textContent = `${sign}${newStraighten.toFixed(1)}°`;
      }

      if (this.onStraightenUpdateCallback) {
        this.onStraightenUpdateCallback(newStraighten);
      }
      return;
    }

    const deltaU = (e.clientX - this.dragStartPos.x) / this.imageRect.width;
    const deltaV = (e.clientY - this.dragStartPos.y) / this.imageRect.height;

    if (this.activeHandle === 'move') {
      const w = this.dragStartCrop.right - this.dragStartCrop.left;
      const h = this.dragStartCrop.bottom - this.dragStartCrop.top;
      const left = Math.max(
        0,
        Math.min(1.0 - w, this.dragStartCrop.left + deltaU),
      );
      const top = Math.max(
        0,
        Math.min(1.0 - h, this.dragStartCrop.top + deltaV),
      );
      this.crop = { left, top, right: left + w, bottom: top + h };
    } else if (['n', 's', 'e', 'w'].includes(this.activeHandle)) {
      this.updateEdgeDrag(this.activeHandle, deltaU, deltaV);
    } else if (['nw', 'ne', 'sw', 'se'].includes(this.activeHandle)) {
      this.updateCornerDrag(this.activeHandle, deltaU, deltaV);
    }

    this.render();
    if (this.onCropUpdateCallback) {
      this.onCropUpdateCallback({ ...this.crop });
    }
  }

  private onPointerUp(e: PointerEvent) {
    if (this.activeHandle) {
      const wasRotating = this.activeHandle === 'rotate';
      this.activeHandle = null;
      this.isRotating = false;

      if (wasRotating) {
        if (this.straightenHudEl) {
          this.straightenHudEl.classList.add('hidden');
        }
        if (this.onStraightenEndCallback) {
          this.onStraightenEndCallback();
        }
      }

      try {
        (e.target as HTMLElement).releasePointerCapture(e.pointerId);
      } catch (_) {}
      const handle = this.resolveHitTarget(e.clientX, e.clientY);
      this.updateCursor(handle);
    }
  }

  private updateEdgeDrag(handle: DragHandle, deltaU: number, deltaV: number) {
    const W0 = this.dragStartCrop.right - this.dragStartCrop.left;
    const H0 = this.dragStartCrop.bottom - this.dragStartCrop.top;
    const targetRatio = this.getTargetAspectRatio();

    if (this.aspectMode !== 'free' && targetRatio !== null) {
      const normRatio = targetRatio / this.imageAspect;
      const Yc = (this.dragStartCrop.top + this.dragStartCrop.bottom) / 2.0;
      const Xc = (this.dragStartCrop.left + this.dragStartCrop.right) / 2.0;

      switch (handle) {
        case 'e': {
          const maxAllowedW = Math.max(
            0.01,
            Math.min(
              1.0 - this.dragStartCrop.left,
              2.0 * Math.min(Yc, 1.0 - Yc) * normRatio,
            ),
          );
          const minW = Math.min(0.05, maxAllowedW);
          const newW = Math.max(minW, Math.min(maxAllowedW, W0 + deltaU));
          const newH = newW / normRatio;
          this.crop.left = this.dragStartCrop.left;
          this.crop.right = Math.max(
            0.0,
            Math.min(1.0, this.dragStartCrop.left + newW),
          );
          this.crop.top = Math.max(0.0, Math.min(1.0, Yc - newH / 2.0));
          this.crop.bottom = Math.max(0.0, Math.min(1.0, Yc + newH / 2.0));
          break;
        }
        case 'w': {
          const maxAllowedW = Math.max(
            0.01,
            Math.min(
              this.dragStartCrop.right,
              2.0 * Math.min(Yc, 1.0 - Yc) * normRatio,
            ),
          );
          const minW = Math.min(0.05, maxAllowedW);
          const newW = Math.max(minW, Math.min(maxAllowedW, W0 - deltaU));
          const newH = newW / normRatio;
          this.crop.left = Math.max(
            0.0,
            Math.min(1.0, this.dragStartCrop.right - newW),
          );
          this.crop.right = this.dragStartCrop.right;
          this.crop.top = Math.max(0.0, Math.min(1.0, Yc - newH / 2.0));
          this.crop.bottom = Math.max(0.0, Math.min(1.0, Yc + newH / 2.0));
          break;
        }
        case 's': {
          const maxAllowedH = Math.max(
            0.01,
            Math.min(
              1.0 - this.dragStartCrop.top,
              (2.0 * Math.min(Xc, 1.0 - Xc)) / normRatio,
            ),
          );
          const minH = Math.min(0.05, maxAllowedH);
          const newH = Math.max(minH, Math.min(maxAllowedH, H0 + deltaV));
          const newW = newH * normRatio;
          this.crop.top = this.dragStartCrop.top;
          this.crop.bottom = Math.max(
            0.0,
            Math.min(1.0, this.dragStartCrop.top + newH),
          );
          this.crop.left = Math.max(0.0, Math.min(1.0, Xc - newW / 2.0));
          this.crop.right = Math.max(0.0, Math.min(1.0, Xc + newW / 2.0));
          break;
        }
        case 'n': {
          const maxAllowedH = Math.max(
            0.01,
            Math.min(
              this.dragStartCrop.bottom,
              (2.0 * Math.min(Xc, 1.0 - Xc)) / normRatio,
            ),
          );
          const minH = Math.min(0.05, maxAllowedH);
          const newH = Math.max(minH, Math.min(maxAllowedH, H0 - deltaV));
          const newW = newH * normRatio;
          this.crop.top = Math.max(
            0.0,
            Math.min(1.0, this.dragStartCrop.bottom - newH),
          );
          this.crop.bottom = this.dragStartCrop.bottom;
          this.crop.left = Math.max(0.0, Math.min(1.0, Xc - newW / 2.0));
          this.crop.right = Math.max(0.0, Math.min(1.0, Xc + newW / 2.0));
          break;
        }
      }
    } else {
      switch (handle) {
        case 'n':
          this.crop.top = Math.max(
            0.0,
            Math.min(
              this.dragStartCrop.bottom - 0.05,
              this.dragStartCrop.top + deltaV,
            ),
          );
          break;
        case 's':
          this.crop.bottom = Math.min(
            1.0,
            Math.max(
              this.dragStartCrop.top + 0.05,
              this.dragStartCrop.bottom + deltaV,
            ),
          );
          break;
        case 'w':
          this.crop.left = Math.max(
            0.0,
            Math.min(
              this.dragStartCrop.right - 0.05,
              this.dragStartCrop.left + deltaU,
            ),
          );
          break;
        case 'e':
          this.crop.right = Math.min(
            1.0,
            Math.max(
              this.dragStartCrop.left + 0.05,
              this.dragStartCrop.right + deltaU,
            ),
          );
          break;
      }
    }
  }

  private updateCornerDrag(handle: DragHandle, deltaU: number, deltaV: number) {
    const W0 = this.dragStartCrop.right - this.dragStartCrop.left;
    const targetRatio = this.getTargetAspectRatio();

    if (this.aspectMode !== 'free' && targetRatio !== null) {
      const normRatio = targetRatio / this.imageAspect;

      switch (handle) {
        case 'se': {
          const delta = (deltaU + deltaV * normRatio) / 2.0;
          const maxAllowedW = Math.max(
            0.01,
            Math.min(
              1.0 - this.dragStartCrop.left,
              (1.0 - this.dragStartCrop.top) * normRatio,
            ),
          );
          const minW = Math.min(0.05, maxAllowedW);
          const newW = Math.max(minW, Math.min(maxAllowedW, W0 + delta));
          const newH = newW / normRatio;
          this.crop.left = this.dragStartCrop.left;
          this.crop.right = Math.max(
            0.0,
            Math.min(1.0, this.dragStartCrop.left + newW),
          );
          this.crop.top = this.dragStartCrop.top;
          this.crop.bottom = Math.max(
            0.0,
            Math.min(1.0, this.dragStartCrop.top + newH),
          );
          break;
        }
        case 'ne': {
          const delta = (deltaU - deltaV * normRatio) / 2.0;
          const maxAllowedW = Math.max(
            0.01,
            Math.min(
              1.0 - this.dragStartCrop.left,
              this.dragStartCrop.bottom * normRatio,
            ),
          );
          const minW = Math.min(0.05, maxAllowedW);
          const newW = Math.max(minW, Math.min(maxAllowedW, W0 + delta));
          const newH = newW / normRatio;
          this.crop.left = this.dragStartCrop.left;
          this.crop.right = Math.max(
            0.0,
            Math.min(1.0, this.dragStartCrop.left + newW),
          );
          this.crop.top = Math.max(
            0.0,
            Math.min(1.0, this.dragStartCrop.bottom - newH),
          );
          this.crop.bottom = this.dragStartCrop.bottom;
          break;
        }
        case 'sw': {
          const delta = (-deltaU + deltaV * normRatio) / 2.0;
          const maxAllowedW = Math.max(
            0.01,
            Math.min(
              this.dragStartCrop.right,
              (1.0 - this.dragStartCrop.top) * normRatio,
            ),
          );
          const minW = Math.min(0.05, maxAllowedW);
          const newW = Math.max(minW, Math.min(maxAllowedW, W0 + delta));
          const newH = newW / normRatio;
          this.crop.left = Math.max(
            0.0,
            Math.min(1.0, this.dragStartCrop.right - newW),
          );
          this.crop.right = this.dragStartCrop.right;
          this.crop.top = this.dragStartCrop.top;
          this.crop.bottom = Math.max(
            0.0,
            Math.min(1.0, this.dragStartCrop.top + newH),
          );
          break;
        }
        case 'nw': {
          const delta = (-deltaU - deltaV * normRatio) / 2.0;
          const maxAllowedW = Math.max(
            0.01,
            Math.min(
              this.dragStartCrop.right,
              this.dragStartCrop.bottom * normRatio,
            ),
          );
          const minW = Math.min(0.05, maxAllowedW);
          const newW = Math.max(minW, Math.min(maxAllowedW, W0 + delta));
          const newH = newW / normRatio;
          this.crop.left = Math.max(
            0.0,
            Math.min(1.0, this.dragStartCrop.right - newW),
          );
          this.crop.right = this.dragStartCrop.right;
          this.crop.top = Math.max(
            0.0,
            Math.min(1.0, this.dragStartCrop.bottom - newH),
          );
          this.crop.bottom = this.dragStartCrop.bottom;
          break;
        }
      }
    } else {
      switch (handle) {
        case 'nw':
          this.crop.left = Math.max(
            0.0,
            Math.min(
              this.dragStartCrop.right - 0.05,
              this.dragStartCrop.left + deltaU,
            ),
          );
          this.crop.top = Math.max(
            0.0,
            Math.min(
              this.dragStartCrop.bottom - 0.05,
              this.dragStartCrop.top + deltaV,
            ),
          );
          break;
        case 'ne':
          this.crop.right = Math.min(
            1.0,
            Math.max(
              this.dragStartCrop.left + 0.05,
              this.dragStartCrop.right + deltaU,
            ),
          );
          this.crop.top = Math.max(
            0.0,
            Math.min(
              this.dragStartCrop.bottom - 0.05,
              this.dragStartCrop.top + deltaV,
            ),
          );
          break;
        case 'sw':
          this.crop.left = Math.max(
            0.0,
            Math.min(
              this.dragStartCrop.right - 0.05,
              this.dragStartCrop.left + deltaU,
            ),
          );
          this.crop.bottom = Math.min(
            1.0,
            Math.max(
              this.dragStartCrop.top + 0.05,
              this.dragStartCrop.bottom + deltaV,
            ),
          );
          break;
        case 'se':
          this.crop.right = Math.min(
            1.0,
            Math.max(
              this.dragStartCrop.left + 0.05,
              this.dragStartCrop.right + deltaU,
            ),
          );
          this.crop.bottom = Math.min(
            1.0,
            Math.max(
              this.dragStartCrop.top + 0.05,
              this.dragStartCrop.bottom + deltaV,
            ),
          );
          break;
      }
    }
  }

  private getTargetAspectRatio(): number | null {
    if (this.aspectMode === 'free') return null;

    const currentW = this.dragStartCrop.right - this.dragStartCrop.left;
    const currentH = this.dragStartCrop.bottom - this.dragStartCrop.top;
    const isCurrentlyPortrait = currentW * this.imageAspect < currentH;
    let isPortrait =
      this.dragStartCrop.left < 0.01 &&
      this.dragStartCrop.top < 0.01 &&
      this.dragStartCrop.right > 0.99 &&
      this.dragStartCrop.bottom > 0.99
        ? this.imageAspect < 1.0
        : isCurrentlyPortrait;

    if (this.isFlippedAspect) {
      isPortrait = !isPortrait;
    }

    switch (this.aspectMode) {
      case 'original': {
        const isBasePortrait = this.imageAspect < 1.0;
        return isPortrait === isBasePortrait
          ? this.imageAspect
          : 1.0 / this.imageAspect;
      }
      case '1:1':
        return 1.0;
      case '4:3':
        return isPortrait ? 3.0 / 4.0 : 4.0 / 3.0;
      case '3:2':
        return isPortrait ? 2.0 / 3.0 : 3.0 / 2.0;
      case '16:9':
        return isPortrait ? 9.0 / 16.0 : 16.0 / 9.0;
      default:
        return null;
    }
  }

  private applyAspectConstraint() {
    if (this.aspectMode === 'free') return;
    const targetRatio = this.getTargetAspectRatio();
    if (targetRatio === null) return;
    const normRatio = targetRatio / this.imageAspect;

    let w = this.crop.right - this.crop.left;
    let h = this.crop.bottom - this.crop.top;

    h = w / normRatio;
    if (h > 1.0) {
      h = 1.0;
      w = h * normRatio;
      this.crop.right = Math.min(1.0, this.crop.left + w);
    }
    if (this.crop.top + h > 1.0) {
      h = 1.0 - this.crop.top;
      w = h * normRatio;
      this.crop.right = Math.min(1.0, this.crop.left + w);
    } else {
      this.crop.bottom = this.crop.top + h;
    }
  }

  public render() {
    const rect = this.gestureSurface.getBoundingClientRect();
    if (
      rect.width <= 0 ||
      rect.height <= 0 ||
      this.imageRect.width <= 0 ||
      this.imageRect.height <= 0
    )
      return;

    const x = this.imageRect.x + this.crop.left * this.imageRect.width;
    const y = this.imageRect.y + this.crop.top * this.imageRect.height;
    const boxW = Math.max(
      4,
      (this.crop.right - this.crop.left) * this.imageRect.width,
    );
    const boxH = Math.max(
      4,
      (this.crop.bottom - this.crop.top) * this.imageRect.height,
    );

    // Update mask cutout
    this.maskCutout.setAttribute('x', `${x}`);
    this.maskCutout.setAttribute('y', `${y}`);
    this.maskCutout.setAttribute('width', `${boxW}`);
    this.maskCutout.setAttribute('height', `${boxH}`);

    // Update border
    this.boxBorder.setAttribute('x', `${x}`);
    this.boxBorder.setAttribute('y', `${y}`);
    this.boxBorder.setAttribute('width', `${boxW}`);
    this.boxBorder.setAttribute('height', `${boxH}`);

    // Rule of thirds grid lines
    const h1Y = y + boxH / 3;
    const h2Y = y + (boxH * 2) / 3;
    const v1X = x + boxW / 3;
    const v2X = x + (boxW * 2) / 3;

    this.gridH1.setAttribute('x1', `${x}`);
    this.gridH1.setAttribute('y1', `${h1Y}`);
    this.gridH1.setAttribute('x2', `${x + boxW}`);
    this.gridH1.setAttribute('y2', `${h1Y}`);

    this.gridH2.setAttribute('x1', `${x}`);
    this.gridH2.setAttribute('y1', `${h2Y}`);
    this.gridH2.setAttribute('x2', `${x + boxW}`);
    this.gridH2.setAttribute('y2', `${h2Y}`);

    this.gridV1.setAttribute('x1', `${v1X}`);
    this.gridV1.setAttribute('y1', `${y}`);
    this.gridV1.setAttribute('x2', `${v1X}`);
    this.gridV1.setAttribute('y2', `${y + boxH}`);

    this.gridV2.setAttribute('x1', `${v2X}`);
    this.gridV2.setAttribute('y1', `${y}`);
    this.gridV2.setAttribute('x2', `${v2X}`);
    this.gridV2.setAttribute('y2', `${y + boxH}`);

    // 4 Corner Brackets (18px arm length, 3.5px stroke - Mac App Parity)
    const arm = 18;
    this.handleElements.nw.setAttribute(
      'd',
      `M ${x} ${y + arm} L ${x} ${y} L ${x + arm} ${y}`,
    );
    this.handleElements.ne.setAttribute(
      'd',
      `M ${x + boxW - arm} ${y} L ${x + boxW} ${y} L ${x + boxW} ${y + arm}`,
    );
    this.handleElements.sw.setAttribute(
      'd',
      `M ${x} ${y + boxH - arm} L ${x} ${y + boxH} L ${x + arm} ${y + boxH}`,
    );
    this.handleElements.se.setAttribute(
      'd',
      `M ${x + boxW - arm} ${y + boxH} L ${x + boxW} ${y + boxH} L ${x + boxW} ${y + boxH - arm}`,
    );

    // 4 Edge Capsules (26x5 / 5x26 white pills - Mac App Parity)
    const barLen = 26;
    const barThick = 5;
    this.handleElements.n.setAttribute('x', `${x + boxW / 2 - barLen / 2}`);
    this.handleElements.n.setAttribute('y', `${y - barThick / 2}`);
    this.handleElements.n.setAttribute('width', `${barLen}`);
    this.handleElements.n.setAttribute('height', `${barThick}`);

    this.handleElements.s.setAttribute('x', `${x + boxW / 2 - barLen / 2}`);
    this.handleElements.s.setAttribute('y', `${y + boxH - barThick / 2}`);
    this.handleElements.s.setAttribute('width', `${barLen}`);
    this.handleElements.s.setAttribute('height', `${barThick}`);

    this.handleElements.w.setAttribute('x', `${x - barThick / 2}`);
    this.handleElements.w.setAttribute('y', `${y + boxH / 2 - barLen / 2}`);
    this.handleElements.w.setAttribute('width', `${barThick}`);
    this.handleElements.w.setAttribute('height', `${barLen}`);

    this.handleElements.e.setAttribute('x', `${x + boxW - barThick / 2}`);
    this.handleElements.e.setAttribute('y', `${y + boxH / 2 - barLen / 2}`);
    this.handleElements.e.setAttribute('width', `${barThick}`);
    this.handleElements.e.setAttribute('height', `${barLen}`);
  }
}
