/**
 * CanvasViewportController
 *
 * Encapsulates the WebGL darkroom viewport geometry, pan/zoom state machine,
 * cursor-anchored trackpad pinch-to-zoom, 100% 1:1 sensor pixel peeping,
 * and eyedropper color sampling.
 *
 * In accordance with First Proof Desktop Interaction principles:
 * - Pinch-to-zoom is strictly scoped to when the pointer hovers over the photo canvas.
 * - Cursor-anchored scaling ensures the feature under the pointer stays stationary.
 * - Outside the canvas (Inspector, filmstrip), native scrolling operates undisturbed.
 */

import { EditSettingsPayload } from '../WasmBridge';

export interface CanvasViewportDelegate {
  onRender: () => void;
  onUpdateCropOverlay?: () => void;
  onSampleEyedropper?: (mode: 'wb' | 'white' | 'black', r: number, g: number, b: number) => void;
  isCropActive: () => boolean;
  getEdits: () => EditSettingsPayload;
  getImageDimensions: () => {
    width: number;
    height: number;
    rawWidth?: number;
    rawHeight?: number;
  };
}

export class CanvasViewportController {
  private canvas: HTMLCanvasElement;
  private wrapper: HTMLElement;
  private zoomBadge: HTMLElement | null;
  private delegate: CanvasViewportDelegate;

  // Pan & Zoom state
  private zoomScale: number = 1.0;
  private panOffset: { x: number; y: number } = { x: 0, y: 0 };
  private isPanning: boolean = false;
  private panStartPos: { x: number; y: number } = { x: 0, y: 0 };
  private panStartOffset: { x: number; y: number } = { x: 0, y: 0 };

  // Eyedropper state
  private activeEyedropper: 'wb' | 'white' | 'black' | null = null;
  private btnWb: HTMLElement | null = null;
  private btnWhite: HTMLElement | null = null;
  private btnBlack: HTMLElement | null = null;

  // Bound listeners for cleanup
  private boundOnWheel: (e: WheelEvent) => void;
  private boundOnMouseDown: (e: MouseEvent) => void;
  private boundOnMouseMove: (e: MouseEvent) => void;
  private boundOnMouseUp: () => void;
  private boundOnDblClick: (e: MouseEvent) => void;
  private boundOnResize: () => void;
  private boundOnClick: (e: MouseEvent) => void;
  private isBound: boolean = false;

  constructor(
    canvas: HTMLCanvasElement,
    wrapper: HTMLElement,
    zoomBadge: HTMLElement | null,
    delegate: CanvasViewportDelegate
  ) {
    this.canvas = canvas;
    this.wrapper = wrapper;
    this.zoomBadge = zoomBadge;
    this.delegate = delegate;

    this.btnWb = document.getElementById('btn-wb-eyedropper');
    this.btnWhite = document.getElementById('btn-white-point-picker');
    this.btnBlack = document.getElementById('btn-black-point-picker');

    this.boundOnWheel = (e) => this.handleWheel(e);
    this.boundOnMouseDown = (e) => this.handleMouseDown(e);
    this.boundOnMouseMove = (e) => this.handleMouseMove(e);
    this.boundOnMouseUp = () => this.handleMouseUp();
    this.boundOnDblClick = (e) => this.handleDblClick(e);
    this.boundOnResize = () => this.handleResize();
    this.boundOnClick = (e) => this.handleClick(e);
  }

  public bind(): void {
    if (this.isBound) return;

    // Wheel/Pinch listener strictly attached to wrapper with non-passive option to allow preventDefault
    this.wrapper.addEventListener('wheel', this.boundOnWheel, { passive: false });
    this.canvas.addEventListener('mousedown', this.boundOnMouseDown);
    this.canvas.addEventListener('dblclick', this.boundOnDblClick);
    this.canvas.addEventListener('click', this.boundOnClick);
    window.addEventListener('mousemove', this.boundOnMouseMove);
    window.addEventListener('mouseup', this.boundOnMouseUp);
    window.addEventListener('resize', this.boundOnResize);

    if (this.zoomBadge) {
      this.zoomBadge.addEventListener('click', () => this.togglePixelPeep());
    }

    if (this.btnWb) {
      this.btnWb.addEventListener('click', () => this.toggleEyedropper('wb'));
    }
    if (this.btnWhite) {
      this.btnWhite.addEventListener('click', () => this.toggleEyedropper('white'));
    }
    if (this.btnBlack) {
      this.btnBlack.addEventListener('click', () => this.toggleEyedropper('black'));
    }

    this.isBound = true;
  }

  public unbind(): void {
    if (!this.isBound) return;
    this.wrapper.removeEventListener('wheel', this.boundOnWheel);
    this.canvas.removeEventListener('mousedown', this.boundOnMouseDown);
    this.canvas.removeEventListener('dblclick', this.boundOnDblClick);
    this.canvas.removeEventListener('click', this.boundOnClick);
    window.removeEventListener('mousemove', this.boundOnMouseMove);
    window.removeEventListener('mouseup', this.boundOnMouseUp);
    window.removeEventListener('resize', this.boundOnResize);
    this.isBound = false;
  }

  public getZoom(): number {
    return this.zoomScale;
  }

  public getPan(): { x: number; y: number } {
    return { ...this.panOffset };
  }

  public isPanningActive(): boolean {
    return this.isPanning;
  }

  public resetZoom(): void {
    this.zoomScale = 1.0;
    this.panOffset = { x: 0, y: 0 };
    this.updateZoomUI();
  }

  public setZoom(scale: number, pan?: { x: number; y: number }): void {
    this.zoomScale = Math.max(1.0, scale);
    if (pan) {
      this.panOffset = { ...pan };
    }
    this.clampPanOffset();
    this.updateZoomUI();
  }

  public getActiveEyedropper(): 'wb' | 'white' | 'black' | null {
    return this.activeEyedropper;
  }

  public clearEyedropper(): void {
    if (!this.activeEyedropper) return;
    this.activeEyedropper = null;
    this.wrapper.style.cursor = 'default';
    this.btnWb?.classList.remove('active');
    this.btnWhite?.classList.remove('active');
    this.btnBlack?.classList.remove('active');
  }

  public toggleEyedropper(mode: 'wb' | 'white' | 'black'): void {
    if (this.activeEyedropper === mode) {
      this.clearEyedropper();
    } else {
      this.activeEyedropper = mode;
      this.wrapper.style.cursor = 'crosshair';
      this.btnWb?.classList.toggle('active', mode === 'wb');
      this.btnWhite?.classList.toggle('active', mode === 'white');
      this.btnBlack?.classList.toggle('active', mode === 'black');
    }
  }

  /**
   * Computes the 1:1 sensor-to-screen pixel ratio for 100% pixel peeping.
   */
  public getZoomScale100(): number {
    const edits = this.delegate.getEdits();
    const dims = this.delegate.getImageDimensions();
    const isOrthogonal = (edits.rotation === 90 || edits.rotation === 270);
    const sensorW = isOrthogonal
      ? (dims.rawHeight || dims.height || 1920)
      : (dims.rawWidth || dims.width || 1920);
    const sensorH = isOrthogonal
      ? (dims.rawWidth || dims.width || 1920)
      : (dims.rawHeight || dims.height || 1080);
    const imgAspect = sensorW / Math.max(sensorH, 1);

    const viewW = this.wrapper?.clientWidth || 800;
    const viewH = this.wrapper?.clientHeight || 600;
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

  /**
   * Toggles between standard Fit view and 100% 1:1 sensor pixel peeping.
   */
  public togglePixelPeep(clientX?: number, clientY?: number): void {
    if (this.delegate.isCropActive()) return;

    if (this.zoomScale > 1.05) {
      // Return to fit
      this.resetZoom();
    } else {
      // Zoom to 100% 1:1 sensor view
      const targetScale = this.getZoomScale100();
      this.zoomScale = targetScale;

      if (clientX !== undefined && clientY !== undefined) {
        const rect = this.wrapper.getBoundingClientRect();
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
    this.delegate.onRender();
  }

  /**
   * Clamps panOffset so the photo quad does not pan completely off-screen.
   */
  public clampPanOffset(): void {
    const edits = this.delegate.getEdits();
    const dims = this.delegate.getImageDimensions();
    const isOrthogonal = (edits.rotation === 90 || edits.rotation === 270);
    const origW = isOrthogonal ? (dims.height || 1920) : (dims.width || 1920);
    const origH = isOrthogonal ? (dims.width || 1920) : (dims.height || 1080);
    const cropActive = this.delegate.isCropActive();
    const cL = cropActive ? 0.0 : (edits.crop_left ?? 0.0);
    const cT = cropActive ? 0.0 : (edits.crop_top ?? 0.0);
    const cR = cropActive ? 1.0 : (edits.crop_right ?? 1.0);
    const cB = cropActive ? 1.0 : (edits.crop_bottom ?? 1.0);
    const dispW = Math.max(1, origW * (cR - cL));
    const dispH = Math.max(1, origH * (cB - cT));
    const imgAspect = dispW / dispH;

    const viewW = this.wrapper?.clientWidth || 800;
    const viewH = this.wrapper?.clientHeight || 600;
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

  public updateZoomUI(): void {
    if (this.zoomBadge) {
      if (this.zoomScale <= 1.05) {
        this.zoomBadge.innerText = 'Fit';
        this.zoomBadge.classList.remove('zoomed');
      } else {
        this.zoomBadge.innerText = `${Math.round(this.zoomScale * 100)}%`;
        this.zoomBadge.classList.add('zoomed');
      }
    }
    this.canvas.style.cursor = this.zoomScale > 1.05 ? 'grab' : 'default';
  }

  public updateCanvasDimensions(): void {
    const dpr = window.devicePixelRatio || 1;
    const w = Math.round(this.wrapper.clientWidth * dpr);
    const h = Math.round(this.wrapper.clientHeight * dpr);
    if (this.canvas.width !== w || this.canvas.height !== h) {
      this.canvas.width = w;
      this.canvas.height = h;
    }
  }

  // --- Private Event Handlers ---

  private handleWheel(e: WheelEvent): void {
    if (this.delegate.isCropActive()) return;

    // Trackpad pinch sends e.ctrlKey=true. Alt/Meta + wheel also zooms.
    const isPinch = e.ctrlKey;
    const isModifierZoom = e.altKey || e.metaKey;

    if (!isPinch && !isModifierZoom) {
      // Normal wheel over canvas: let native gestures or passive scroll through if needed,
      // or ignore so users don't accidentally zoom when hovering across the canvas.
      return;
    }

    e.preventDefault();
    e.stopPropagation();

    const rect = this.wrapper.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return;

    // Cursor position in NDC [-1, 1] relative to center
    const cursorNdcX = ((e.clientX - rect.left) / rect.width) * 2.0 - 1.0;
    const cursorNdcY = -(((e.clientY - rect.top) / rect.height) * 2.0 - 1.0);

    // Continuous exponential zoom factor
    const zoomFactor = isPinch
      ? Math.exp(-e.deltaY * 0.01)
      : Math.exp(-e.deltaY * 0.002);

    const oldScale = this.zoomScale;
    const target100 = this.getZoomScale100();
    const maxScale = Math.max(4.0, target100 * 2.5);
    const newScale = Math.min(Math.max(oldScale * zoomFactor, 1.0), maxScale);

    if (Math.abs(newScale - oldScale) < 0.001) return;

    if (newScale <= 1.05) {
      this.resetZoom();
    } else {
      // Exact cursor anchoring invariant:
      // P_new = cursorNdc - (cursorNdc - P_old) * (newScale / oldScale)
      const ratio = newScale / oldScale;
      this.panOffset.x = cursorNdcX - (cursorNdcX - this.panOffset.x) * ratio;
      this.panOffset.y = cursorNdcY - (cursorNdcY - this.panOffset.y) * ratio;
      this.zoomScale = newScale;
      this.clampPanOffset();
      this.updateZoomUI();
    }

    this.delegate.onRender();
  }

  private handleMouseDown(e: MouseEvent): void {
    if (this.activeEyedropper) return;
    if (this.delegate.isCropActive()) return;

    if (this.zoomScale > 1.05 && e.button === 0) {
      this.isPanning = true;
      this.panStartPos = { x: e.clientX, y: e.clientY };
      this.panStartOffset = { ...this.panOffset };
      this.canvas.style.cursor = 'grabbing';
      e.preventDefault();
    }
  }

  private handleMouseMove(e: MouseEvent): void {
    if (this.isPanning) {
      const rect = this.wrapper.getBoundingClientRect();
      if (rect.width > 0 && rect.height > 0) {
        const dx = ((e.clientX - this.panStartPos.x) / (rect.width / 2));
        const dy = -((e.clientY - this.panStartPos.y) / (rect.height / 2));
        this.panOffset.x = this.panStartOffset.x + dx;
        this.panOffset.y = this.panStartOffset.y + dy;
        this.clampPanOffset();
        this.delegate.onRender();
      }
    }
  }

  private handleMouseUp(): void {
    if (this.isPanning) {
      this.isPanning = false;
      this.canvas.style.cursor = this.zoomScale > 1.05 ? 'grab' : 'default';
    }
  }

  private handleDblClick(e: MouseEvent): void {
    if (this.delegate.isCropActive()) return;
    this.togglePixelPeep(e.clientX, e.clientY);
  }

  private handleResize(): void {
    this.updateCanvasDimensions();
    this.delegate.onRender();
    if (this.delegate.isCropActive() && this.delegate.onUpdateCropOverlay) {
      this.delegate.onUpdateCropOverlay();
    }
  }

  private handleClick(e: MouseEvent): void {
    if (!this.activeEyedropper) return;
    const mode = this.activeEyedropper;

    const rect = this.canvas.getBoundingClientRect();
    const x = Math.floor((e.clientX - rect.left) * (this.canvas.width / rect.width));
    const y = Math.floor((e.clientY - rect.top) * (this.canvas.height / rect.height));

    const gl = this.canvas.getContext('webgl2') as WebGL2RenderingContext;
    if (gl) {
      const pixels = new Uint8Array(4);
      const glY = this.canvas.height - y - 1;
      gl.readPixels(x, glY, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      const [r, g, b] = [pixels[0] / 255, pixels[1] / 255, pixels[2] / 255];

      if (this.delegate.onSampleEyedropper) {
        this.delegate.onSampleEyedropper(mode, r, g, b);
      }
    }

    this.clearEyedropper();
  }
}
