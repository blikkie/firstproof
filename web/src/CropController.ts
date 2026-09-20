/**
 * Interactive Crop Controller for First Proof Web Darkroom
 * 
 * Implements a stationary full-canvas gesture container strictly adhering to
 * First Proof Desktop Gesture Architecture & Hit Testing invariants:
 * - Visual handles are purely declarative render elements with pointer-events: none.
 * - Single gesture surface resolves hit targets at startLocation.
 * - Aspect ratio constraints (Original, Free, 1:1, 3:2, 4:3, 16:9).
 * - Continuous smooth updates with 60fps SVG rendering.
 */

export interface CropRect {
  left: number;   // 0.0 .. 1.0 (normalized)
  top: number;    // 0.0 .. 1.0
  right: number;  // 0.0 .. 1.0
  bottom: number; // 0.0 .. 1.0
}

export type AspectRatioMode = 'original' | 'free' | '1:1' | '3:2' | '4:3' | '16:9';

type DragHandle = 'nw' | 'ne' | 'sw' | 'se' | 'n' | 's' | 'e' | 'w' | 'move' | 'create' | null;

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

  private crop: CropRect = { left: 0.0, top: 0.0, right: 1.0, bottom: 1.0 };
  private imageAspect: number = 1.5; // width / height
  private aspectMode: AspectRatioMode = 'original';
  private isFlippedAspect: boolean = false;

  private activeHandle: DragHandle = null;
  private dragStartPos: { x: number; y: number } = { x: 0, y: 0 };
  private dragStartCrop: CropRect = { left: 0, top: 0, right: 1, bottom: 1 };

  private onCropUpdateCallback: ((crop: CropRect) => void) | null = null;
  private onDoneCallback: (() => void) | null = null;

  constructor(
    overlayId: string = 'crop-overlay',
    callbacks?: {
      onCropUpdate?: (crop: CropRect) => void;
      onDone?: () => void;
    }
  ) {
    this.overlayEl = document.getElementById(overlayId) as HTMLElement;
    this.toolbarEl = document.getElementById('crop-toolbar') as HTMLElement;
    this.gestureSurface = document.getElementById('crop-gesture-surface') as HTMLElement;
    this.maskCutout = document.getElementById('crop-mask-cutout') as unknown as SVGRectElement;
    this.boxBorder = document.getElementById('crop-box-border') as unknown as SVGRectElement;
    this.gridH1 = document.getElementById('crop-grid-h1') as unknown as SVGLineElement;
    this.gridH2 = document.getElementById('crop-grid-h2') as unknown as SVGLineElement;
    this.gridV1 = document.getElementById('crop-grid-v1') as unknown as SVGLineElement;
    this.gridV2 = document.getElementById('crop-grid-v2') as unknown as SVGLineElement;

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

    this.aspectSelect = document.getElementById('select-crop-aspect') as HTMLSelectElement;
    this.btnFlip = document.getElementById('btn-crop-flip') as HTMLButtonElement;
    this.btnReset = document.getElementById('btn-crop-reset') as HTMLButtonElement;
    this.btnDone = document.getElementById('btn-crop-done') as HTMLButtonElement;

    if (callbacks?.onCropUpdate) this.onCropUpdateCallback = callbacks.onCropUpdate;
    if (callbacks?.onDone) this.onDoneCallback = callbacks.onDone;

    this.initEvents();
  }

  private initEvents() {
    this.gestureSurface?.addEventListener('pointerdown', this.onPointerDown.bind(this));
    window.addEventListener('pointermove', this.onPointerMove.bind(this));
    window.addEventListener('pointerup', this.onPointerUp.bind(this));
    window.addEventListener('pointercancel', this.onPointerUp.bind(this));

    this.aspectSelect?.addEventListener('change', () => {
      this.aspectMode = this.aspectSelect.value as AspectRatioMode;
      this.applyAspectConstraint();
      this.render();
      if (this.onCropUpdateCallback) this.onCropUpdateCallback({ ...this.crop });
    });

    this.btnFlip?.addEventListener('click', () => {
      this.flipCropOrientation();
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

    const optOriginal = this.aspectSelect.querySelector('option[value="original"]');
    const opt32 = this.aspectSelect.querySelector('option[value="3:2"]');
    const opt43 = this.aspectSelect.querySelector('option[value="4:3"]');
    const opt169 = this.aspectSelect.querySelector('option[value="16:9"]');

    if (optOriginal) optOriginal.textContent = portrait ? 'Original (Portrait)' : 'Original Aspect';
    if (opt32) opt32.textContent = portrait ? '2:3 (35mm)' : '3:2 (35mm)';
    if (opt43) opt43.textContent = portrait ? '3:4 (Medium Format)' : '4:3 (Medium Format)';
    if (opt169) opt169.textContent = portrait ? '9:16 (Vertical)' : '16:9 Widescreen';
  }

  public show(crop: CropRect, imageAspect: number) {
    this.crop = {
      left: Math.max(0, Math.min(0.95, crop.left ?? 0.0)),
      top: Math.max(0, Math.min(0.95, crop.top ?? 0.0)),
      right: Math.max(0.05, Math.min(1.0, crop.right ?? 1.0)),
      bottom: Math.max(0.05, Math.min(1.0, crop.bottom ?? 1.0)),
    };
    this.imageAspect = imageAspect > 0 ? imageAspect : 1.5;
    this.isFlippedAspect = false;
    this.updateAspectOptionLabels();
    this.overlayEl.classList.remove('hidden');
    this.toolbarEl?.classList.remove('hidden');
    this.render();
  }

  public hide() {
    this.overlayEl.classList.add('hidden');
    this.toolbarEl?.classList.add('hidden');
  }

  public flipCropOrientation() {
    this.isFlippedAspect = !this.isFlippedAspect;
    this.updateAspectOptionLabels();

    const currentW = this.crop.right - this.crop.left;
    const currentH = this.crop.bottom - this.crop.top;
    const centerX = (this.crop.left + this.crop.right) / 2.0;
    const centerY = (this.crop.top + this.crop.bottom) / 2.0;

    const currentPhotoAspect = (currentW / Math.max(currentH, 0.001)) * this.imageAspect;
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

    this.crop = { left: newLeft, top: newTop, right: newRight, bottom: newBottom };
    this.render();
    if (this.onCropUpdateCallback) {
      this.onCropUpdateCallback({ ...this.crop });
    }
  }

  public resetCrop() {
    this.isFlippedAspect = false;
    this.updateAspectOptionLabels();
    this.crop = { left: 0.0, top: 0.0, right: 1.0, bottom: 1.0 };
    if (this.aspectMode !== 'free') {
      this.applyAspectConstraint();
    }
    this.render();
    if (this.onCropUpdateCallback) this.onCropUpdateCallback({ ...this.crop });
  }

  public getCrop(): CropRect {
    return { ...this.crop };
  }

  public updatePosition(overlayRect: { left: number; top: number; width: number; height: number }) {
    this.overlayEl.style.left = `${overlayRect.left}px`;
    this.overlayEl.style.top = `${overlayRect.top}px`;
    this.overlayEl.style.width = `${overlayRect.width}px`;
    this.overlayEl.style.height = `${overlayRect.height}px`;
    this.render();
  }

  private resolveHitTarget(clientX: number, clientY: number): DragHandle {
    const rect = this.gestureSurface.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return null;

    const px = clientX - rect.left;
    const py = clientY - rect.top;

    const xL = this.crop.left * rect.width;
    const xR = this.crop.right * rect.width;
    const yT = this.crop.top * rect.height;
    const yB = this.crop.bottom * rect.height;

    const cornerRadius = 24; // Pixel hit radius for corners
    const edgeRadius = 14;   // Pixel hit radius for edges

    // Check corners
    if (Math.hypot(px - xL, py - yT) <= cornerRadius) return 'nw';
    if (Math.hypot(px - xR, py - yT) <= cornerRadius) return 'ne';
    if (Math.hypot(px - xL, py - yB) <= cornerRadius) return 'sw';
    if (Math.hypot(px - xR, py - yB) <= cornerRadius) return 'se';

    // Check edges
    if (Math.abs(py - yT) <= edgeRadius && px >= xL - edgeRadius && px <= xR + edgeRadius) return 'n';
    if (Math.abs(py - yB) <= edgeRadius && px >= xL - edgeRadius && px <= xR + edgeRadius) return 's';
    if (Math.abs(px - xL) <= edgeRadius && py >= yT - edgeRadius && py <= yB + edgeRadius) return 'w';
    if (Math.abs(px - xR) <= edgeRadius && py >= yT - edgeRadius && py <= yB + edgeRadius) return 'e';

    // Check inside
    if (px > xL && px < xR && py > yT && py < yB) return 'move';

    return 'create';
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
      default:
        this.gestureSurface.style.cursor = 'crosshair';
        break;
    }
  }

  private onPointerDown(e: PointerEvent) {
    if (e.button !== 0) return; // Primary button only
    const handle = this.resolveHitTarget(e.clientX, e.clientY);
    this.activeHandle = handle;
    this.dragStartPos = { x: e.clientX, y: e.clientY };
    this.dragStartCrop = { ...this.crop };

    if (handle === 'create') {
      const rect = this.gestureSurface.getBoundingClientRect();
      const normX = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      const normY = Math.max(0, Math.min(1, (e.clientY - rect.top) / rect.height));
      this.dragStartCrop = { left: normX, top: normY, right: normX, bottom: normY };
      this.crop = { ...this.dragStartCrop };
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
    if (rect.width <= 0 || rect.height <= 0) return;

    const dx = (e.clientX - this.dragStartPos.x) / rect.width;
    const dy = (e.clientY - this.dragStartPos.y) / rect.height;

    const minSize = 0.05;
    let newCrop = { ...this.dragStartCrop };

    if (this.activeHandle === 'move') {
      const w = this.dragStartCrop.right - this.dragStartCrop.left;
      const h = this.dragStartCrop.bottom - this.dragStartCrop.top;
      let left = Math.max(0, Math.min(1 - w, this.dragStartCrop.left + dx));
      let top = Math.max(0, Math.min(1 - h, this.dragStartCrop.top + dy));
      newCrop = { left, top, right: left + w, bottom: top + h };
    } else if (this.activeHandle === 'create') {
      const curX = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      const curY = Math.max(0, Math.min(1, (e.clientY - rect.top) / rect.height));
      newCrop = {
        left: Math.min(this.dragStartCrop.left, curX),
        top: Math.min(this.dragStartCrop.top, curY),
        right: Math.max(this.dragStartCrop.left, curX),
        bottom: Math.max(this.dragStartCrop.top, curY),
      };
      if (newCrop.right - newCrop.left < minSize) newCrop.right = newCrop.left + minSize;
      if (newCrop.bottom - newCrop.top < minSize) newCrop.bottom = newCrop.top + minSize;
    } else {
      if (this.activeHandle.includes('w')) {
        newCrop.left = Math.max(0, Math.min(this.dragStartCrop.right - minSize, this.dragStartCrop.left + dx));
      }
      if (this.activeHandle.includes('e')) {
        newCrop.right = Math.min(1, Math.max(this.dragStartCrop.left + minSize, this.dragStartCrop.right + dx));
      }
      if (this.activeHandle.includes('n')) {
        newCrop.top = Math.max(0, Math.min(this.dragStartCrop.bottom - minSize, this.dragStartCrop.top + dy));
      }
      if (this.activeHandle.includes('s')) {
        newCrop.bottom = Math.min(1, Math.max(this.dragStartCrop.top + minSize, this.dragStartCrop.bottom + dy));
      }
    }

    this.crop = newCrop;
    if (this.aspectMode !== 'free') {
      this.applyAspectConstraint(this.activeHandle);
    }

    this.render();
    if (this.onCropUpdateCallback) {
      this.onCropUpdateCallback({ ...this.crop });
    }
  }

  private onPointerUp(e: PointerEvent) {
    if (this.activeHandle) {
      this.activeHandle = null;
      try {
        (e.target as HTMLElement).releasePointerCapture(e.pointerId);
      } catch (_) {}
      const handle = this.resolveHitTarget(e.clientX, e.clientY);
      this.updateCursor(handle);
    }
  }

  private getTargetAspectRatio(): number {
    const isPortrait = this.imageAspect < 1.0;
    let portrait = isPortrait;
    if (this.isFlippedAspect) portrait = !portrait;

    switch (this.aspectMode) {
      case 'original':
        return this.isFlippedAspect ? (1.0 / this.imageAspect) : this.imageAspect;
      case '1:1':
        return 1.0;
      case '3:2':
        return portrait ? (2.0 / 3.0) : (3.0 / 2.0);
      case '4:3':
        return portrait ? (3.0 / 4.0) : (4.0 / 3.0);
      case '16:9':
        return portrait ? (9.0 / 16.0) : (16.0 / 9.0);
      default:
        return this.isFlippedAspect ? (1.0 / this.imageAspect) : this.imageAspect;
    }
  }

  private applyAspectConstraint(handle: DragHandle = null) {
    if (this.aspectMode === 'free') return;
    const targetRatio = this.getTargetAspectRatio(); // (w / h) in physical aspect
    // In normalized coords: (w * imageAspect) / h = targetRatio  =>  w / h = targetRatio / imageAspect
    const normRatio = targetRatio / this.imageAspect;

    let w = this.crop.right - this.crop.left;
    let h = this.crop.bottom - this.crop.top;

    if (handle === 'n' || handle === 's') {
      w = h * normRatio;
      if (w > 1.0) {
        w = 1.0;
        h = w / normRatio;
      }
      const midX = (this.crop.left + this.crop.right) / 2;
      let left = Math.max(0, midX - w / 2);
      if (left + w > 1.0) left = Math.max(0, 1.0 - w);
      this.crop.left = left;
      this.crop.right = Math.min(1.0, left + w);
      this.crop.top = Math.max(0, Math.min(this.crop.top, 1.0 - h));
      this.crop.bottom = Math.min(1.0, this.crop.top + h);
    } else {
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
  }

  public render() {
    const rect = this.gestureSurface.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return;

    const w = rect.width;
    const h = rect.height;

    const x = this.crop.left * w;
    const y = this.crop.top * h;
    const boxW = Math.max(4, (this.crop.right - this.crop.left) * w);
    const boxH = Math.max(4, (this.crop.bottom - this.crop.top) * h);

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
    this.handleElements.nw.setAttribute('d', `M ${x} ${y + arm} L ${x} ${y} L ${x + arm} ${y}`);
    this.handleElements.ne.setAttribute('d', `M ${x + boxW - arm} ${y} L ${x + boxW} ${y} L ${x + boxW} ${y + arm}`);
    this.handleElements.sw.setAttribute('d', `M ${x} ${y + boxH - arm} L ${x} ${y + boxH} L ${x + arm} ${y + boxH}`);
    this.handleElements.se.setAttribute('d', `M ${x + boxW - arm} ${y + boxH} L ${x + boxW} ${y + boxH} L ${x + boxW} ${y + boxH - arm}`);

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
