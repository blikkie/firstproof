export class ComparisonSliderController {
  private container: HTMLElement | null = null;
  private divider: HTMLElement | null = null;
  private isDragging: boolean = false;
  private splitPosition: number = 50; // percentage (0 - 100)

  constructor() {
    this.init();
  }

  public init(): void {
    this.container = document.getElementById('optical-comparison-viewer');
    this.divider = document.getElementById('comparison-divider');

    if (!this.container || !this.divider) {
      return;
    }

    this.bindEvents();
    this.setSplitPosition(50);
  }

  private bindEvents(): void {
    if (!this.container || !this.divider) return;

    const onPointerDown = (e: PointerEvent) => {
      this.isDragging = true;
      this.container?.setPointerCapture(e.pointerId);
      this.updateFromPointer(e);
    };

    const onPointerMove = (e: PointerEvent) => {
      if (!this.isDragging) return;
      this.updateFromPointer(e);
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
    this.container.addEventListener('pointermove', onPointerMove);
    this.container.addEventListener('pointerup', onPointerUp);
    this.container.addEventListener('pointercancel', onPointerUp);

    // Keyboard / Button toggles if present
    const btnJpeg = document.getElementById('btn-compare-jpeg');
    const btnSplit = document.getElementById('btn-compare-split');
    const btnFirstProof =
      document.getElementById('btn-compare-firstproof') ||
      document.getElementById('btn-compare-agx');

    btnJpeg?.addEventListener('click', () => this.setSplitPosition(100));
    btnSplit?.addEventListener('click', () => this.setSplitPosition(50));
    btnFirstProof?.addEventListener('click', () => this.setSplitPosition(0));
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

    // Update active state on toggle buttons
    const btnJpeg = document.getElementById('btn-compare-jpeg');
    const btnSplit = document.getElementById('btn-compare-split');
    const btnAgx = document.getElementById('btn-compare-agx');

    btnJpeg?.classList.toggle('active', this.splitPosition >= 99);
    btnSplit?.classList.toggle(
      'active',
      this.splitPosition > 1 && this.splitPosition < 99,
    );
    btnAgx?.classList.toggle('active', this.splitPosition <= 1);
  }
}
