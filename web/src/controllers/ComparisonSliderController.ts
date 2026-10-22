export class ComparisonSliderController {
  private container: HTMLElement | null = null;
  private divider: HTMLElement | null = null;
  private layerBefore: HTMLElement | null = null;
  private isDragging: boolean = false;
  private splitPosition: number = 50; // percentage (0 - 100)
  private unbindFns: Array<() => void> = [];

  constructor() {
    this.init();
  }

  public init(): void {
    this.container = document.getElementById('optical-comparison-viewer');
    this.divider = document.getElementById('comparison-divider');
    this.layerBefore =
      this.container?.querySelector('.comparison-layer-before') || null;

    if (!this.container || !this.divider) {
      return;
    }

    this.bindEvents();
    this.setSplitPosition(50);
  }

  private bindEvents(): void {
    if (!this.container || !this.divider) return;

    const onPointerDown = (e: PointerEvent) => {
      // Primary button / touch only
      if (e.button !== undefined && e.button !== 0) return;
      this.isDragging = true;
      try {
        this.container?.setPointerCapture(e.pointerId);
      } catch {
        // Pointer capture can throw InvalidStateError or NotFoundError on some mobile engines
      }
      this.updateFromPointer(e);
      // Suppress default mobile gesture interception (prevents premature pointercancel on Android Chrome)
      if (e.cancelable) {
        e.preventDefault();
      }
    };

    const onPointerMove = (e: PointerEvent) => {
      if (!this.isDragging) return;
      this.updateFromPointer(e);
      if (e.cancelable) {
        e.preventDefault();
      }
    };

    const onPointerUp = (e: PointerEvent) => {
      if (!this.isDragging) return;
      this.isDragging = false;
      try {
        this.container?.releasePointerCapture(e.pointerId);
      } catch {
        // Ignore if pointer capture was already released
      }
    };

    this.container.addEventListener('pointerdown', onPointerDown);
    // Bind move and up to window so rapid drags outside container boundaries don't freeze or drop
    window.addEventListener('pointermove', onPointerMove);
    window.addEventListener('pointerup', onPointerUp);
    window.addEventListener('pointercancel', onPointerUp);

    this.unbindFns.push(() => {
      this.container?.removeEventListener('pointerdown', onPointerDown);
      window.removeEventListener('pointermove', onPointerMove);
      window.removeEventListener('pointerup', onPointerUp);
      window.removeEventListener('pointercancel', onPointerUp);
    });

    // Keyboard / Button toggles if present
    const btnJpeg = document.getElementById('btn-compare-jpeg');
    const btnSplit = document.getElementById('btn-compare-split');
    const btnFirstProof =
      document.getElementById('btn-compare-firstproof') ||
      document.getElementById('btn-compare-agx');

    const onJpegClick = () => this.setSplitPosition(100);
    const onSplitClick = () => this.setSplitPosition(50);
    const onFirstProofClick = () => this.setSplitPosition(0);

    btnJpeg?.addEventListener('click', onJpegClick);
    btnSplit?.addEventListener('click', onSplitClick);
    btnFirstProof?.addEventListener('click', onFirstProofClick);

    this.unbindFns.push(() => {
      btnJpeg?.removeEventListener('click', onJpegClick);
      btnSplit?.removeEventListener('click', onSplitClick);
      btnFirstProof?.removeEventListener('click', onFirstProofClick);
    });
  }

  public unbind(): void {
    for (const fn of this.unbindFns) {
      fn();
    }
    this.unbindFns = [];
    this.isDragging = false;
  }

  private updateFromPointer(e: PointerEvent): void {
    if (!this.container) return;
    const rect = this.container.getBoundingClientRect();
    if (rect.width <= 0) return;

    const x = e.clientX - rect.left;
    const clampedX = Math.max(0, Math.min(rect.width, x));
    const percentage = (clampedX / rect.width) * 100;
    this.setSplitPosition(percentage);
  }

  public setSplitPosition(percent: number): void {
    this.splitPosition = Math.max(0, Math.min(100, percent));
    if (this.container) {
      this.container.style.setProperty('--split-pos', `${this.splitPosition}%`);
    }

    if (this.layerBefore) {
      // Direct inline style update forces Android Chrome (Blink) to repaint the composited layer,
      // bypassing the Blink bug where CSS variable changes inside clip-path: polygon() fail to invalidate
      this.layerBefore.style.clipPath = `polygon(0 0, ${this.splitPosition}% 0, ${this.splitPosition}% 100%, 0 100%)`;
    }

    // Update active state on toggle buttons
    const btnJpeg = document.getElementById('btn-compare-jpeg');
    const btnSplit = document.getElementById('btn-compare-split');
    const btnFirstProof =
      document.getElementById('btn-compare-firstproof') ||
      document.getElementById('btn-compare-agx');

    btnJpeg?.classList.toggle('active', this.splitPosition >= 99);
    btnSplit?.classList.toggle(
      'active',
      this.splitPosition > 1 && this.splitPosition < 99,
    );
    btnFirstProof?.classList.toggle('active', this.splitPosition <= 1);
  }

  public getSplitPosition(): number {
    return this.splitPosition;
  }
}
