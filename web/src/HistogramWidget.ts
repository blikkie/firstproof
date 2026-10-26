import type { EvHistogramRecord } from './WasmBridge';

export interface HistogramWidgetCallbacks {
  onToggleClippingOverlay?: () => void;
  onToggleCollapse?: (collapsed: boolean) => void;
}

export class HistogramWidget {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D | null;
  private callbacks: HistogramWidgetCallbacks;

  // DOM elements
  private titleEl!: HTMLElement;
  private collapseBtn!: HTMLButtonElement;
  private shadowBadge!: HTMLButtonElement;
  private shadowText!: HTMLElement;
  private agxBadge!: HTMLElement;
  private agxText!: HTMLElement;
  private sensorBadge!: HTMLButtonElement;
  private sensorText!: HTMLElement;
  private graphContainer!: HTMLElement;
  private scaleContainer!: HTMLElement;

  private currentRecord: EvHistogramRecord | null = null;
  private isCollapsed: boolean = false;
  private hoverFrac: number | null = null;

  constructor(
    container: HTMLElement,
    callbacks: HistogramWidgetCallbacks = {},
  ) {
    this.container = container;
    this.callbacks = callbacks;

    this.renderSkeleton();

    this.canvas = this.container.querySelector(
      '.histogram-canvas',
    ) as HTMLCanvasElement;
    this.ctx = this.canvas.getContext('2d');

    this.bindEvents();
    this.draw();
  }

  private renderSkeleton() {
    this.container.classList.add('histogram-card');
    this.container.innerHTML = `
      <div class="histogram-hud-header">
        <!-- Line 1: Title & Interactive Hover Readout + Collapse Toggle -->
        <div class="histogram-header-line1">
          <div class="histogram-title-readout" id="histogram-title-readout">HISTOGRAM (EV)</div>
          <button class="histogram-collapse-btn" id="histogram-collapse-btn" type="button" title="Toggle Histogram (H)">
            <svg class="chevron-icon" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <polyline points="18 15 12 9 6 15"></polyline>
            </svg>
          </button>
        </div>

        <!-- Line 2: Telemetry Badges (Shadow Floor, AgX Roll-Off, Sensor Saturation) -->
        <div class="histogram-header-line2">
          <button class="histogram-badge badge-shadow" id="hist-badge-shadow" type="button" title="Shadow Floor crushed at noise floor (≤ −8.0 EV). Click or press J to toggle clipping overlay.">
            <span class="badge-icon">◀</span>
            <span class="badge-text" id="hist-shadow-val">Floor 0.0%</span>
          </button>
          <div class="histogram-badge badge-agx hidden" id="hist-badge-agx" title="AgX Shoulder: preserved highlight compression (Zone VII+).">
            <span class="badge-tag">AgX</span>
            <span class="badge-text" id="hist-agx-val">0.0%</span>
          </div>
          <button class="histogram-badge badge-sensor" id="hist-badge-sensor" type="button" title="Sensor Saturation: blown raw Bayer photosites (≥ 1.0 capacity). Click or press J to toggle clipping overlay.">
            <span class="badge-text" id="hist-sensor-val">Clip 0.0%</span>
            <span class="badge-icon">▶</span>
          </button>
        </div>
      </div>

      <!-- Graph Canvas Container -->
      <div class="histogram-graph-container" id="histogram-graph-container">
        <canvas class="histogram-canvas" height="64"></canvas>
      </div>

      <!-- 2-Line Scale Labels Below -->
      <div class="histogram-scale-legend" id="histogram-scale-legend">
        <div class="scale-line scale-line-stops">
          <span class="scale-stop left">−8 EV (Floor)</span>
          <span class="scale-stop center accent">0 EV (18%)</span>
          <span class="scale-stop right">+4 EV (Clip)</span>
        </div>
        <div class="scale-line scale-line-zones">
          <span class="scale-zone left">Zone 0 (Dmax)</span>
          <span class="scale-zone center accent">Zone V (Grey)</span>
          <span class="scale-zone right">Zone VIII+ (White)</span>
        </div>
      </div>
    `;

    this.titleEl = this.container.querySelector(
      '#histogram-title-readout',
    ) as HTMLElement;
    this.collapseBtn = this.container.querySelector(
      '#histogram-collapse-btn',
    ) as HTMLButtonElement;
    this.shadowBadge = this.container.querySelector(
      '#hist-badge-shadow',
    ) as HTMLButtonElement;
    this.shadowText = this.container.querySelector(
      '#hist-shadow-val',
    ) as HTMLElement;
    this.agxBadge = this.container.querySelector(
      '#hist-badge-agx',
    ) as HTMLElement;
    this.agxText = this.container.querySelector('#hist-agx-val') as HTMLElement;
    this.sensorBadge = this.container.querySelector(
      '#hist-badge-sensor',
    ) as HTMLButtonElement;
    this.sensorText = this.container.querySelector(
      '#hist-sensor-val',
    ) as HTMLElement;
    this.graphContainer = this.container.querySelector(
      '#histogram-graph-container',
    ) as HTMLElement;
    this.scaleContainer = this.container.querySelector(
      '#histogram-scale-legend',
    ) as HTMLElement;
  }

  private bindEvents() {
    this.collapseBtn.addEventListener('click', () => {
      this.toggleCollapse();
    });

    this.shadowBadge.addEventListener('click', () => {
      this.callbacks.onToggleClippingOverlay?.();
    });

    this.sensorBadge.addEventListener('click', () => {
      this.callbacks.onToggleClippingOverlay?.();
    });

    this.canvas.addEventListener('mousemove', (e) => {
      const rect = this.canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      this.hoverFrac = Math.min(Math.max(x / rect.width, 0), 1);
      this.updateHeader();
      this.draw();
    });

    this.canvas.addEventListener('mouseleave', () => {
      this.hoverFrac = null;
      this.updateHeader();
      this.draw();
    });
  }

  public setRecord(record: EvHistogramRecord | null) {
    this.currentRecord = record;
    this.updateHeader();
    this.draw();
  }

  public toggleCollapse(force?: boolean) {
    this.isCollapsed = force !== undefined ? force : !this.isCollapsed;
    this.graphContainer.classList.toggle('hidden', this.isCollapsed);
    this.scaleContainer.classList.toggle('hidden', this.isCollapsed);

    const svg = this.collapseBtn.querySelector('svg');
    if (svg) {
      svg.innerHTML = this.isCollapsed
        ? '<polyline points="6 9 12 15 18 9"></polyline>'
        : '<polyline points="18 15 12 9 6 15"></polyline>';
    }
    this.collapseBtn.title = this.isCollapsed
      ? 'Show Histogram (H)'
      : 'Hide Histogram (H)';
    this.callbacks.onToggleCollapse?.(this.isCollapsed);
  }

  public getIsCollapsed(): boolean {
    return this.isCollapsed;
  }

  private updateHeader() {
    const shadowClip = this.currentRecord?.shadow_clip_percent ?? 0;
    const sensorClip = this.currentRecord?.sensor_clip_percent ?? 0;
    const rolloff = this.currentRecord?.display_rolloff_percent ?? 0;

    // Badges
    this.shadowText.textContent = `Floor ${shadowClip.toFixed(1)}%`;
    this.shadowBadge.classList.toggle('badge-active', shadowClip > 0.05);

    this.sensorText.textContent = `Clip ${sensorClip.toFixed(1)}%`;
    this.sensorBadge.classList.toggle('badge-active', sensorClip > 0.05);

    if (rolloff > 0.05) {
      this.agxBadge.classList.remove('hidden');
      this.agxText.textContent = `${rolloff.toFixed(1)}%`;
    } else {
      this.agxBadge.classList.add('hidden');
    }

    // Line 1: Title or Interactive Zone Readout
    if (this.hoverFrac !== null) {
      const info = this.zoneReadout(this.hoverFrac);
      const evColorClass = info.ev.startsWith('+')
        ? 'ev-highlight'
        : info.ev.startsWith('-')
          ? 'ev-shadow'
          : 'ev-mid';
      this.titleEl.innerHTML = `
        <span class="zone-name">${info.zone}</span>
        <span class="sep">•</span>
        <span class="zone-ev ${evColorClass}">${info.ev}</span>
        <span class="sep">•</span>
        <span class="zone-desc">${info.description}</span>
      `;
      this.titleEl.title = `${info.zone} (${info.description}): ${info.ev} relative to 18% middle grey (Zone V)`;
    } else {
      this.titleEl.innerHTML = 'HISTOGRAM (EV)';
      this.titleEl.title = 'Sensitometric 12-Zone EV Histogram';
    }
  }

  private zoneReadout(frac: number): {
    zone: string;
    description: string;
    ev: string;
  } {
    const totalStops = 12.0;
    const clampedFrac = Math.min(Math.max(frac, 0.0), 1.0);
    const ev = -8.0 + clampedFrac * totalStops;
    const evString = `${ev >= 0 ? '+' : ''}${ev.toFixed(1)} EV`;

    let zone: string;
    let description: string;
    if (ev < -5.5) {
      zone = 'Zone 0';
      description = 'Sub-Floor / Noise';
    } else if (ev < -4.5) {
      zone = 'Zone 0';
      description = 'Dmax Maximum Black';
    } else if (ev < -3.5) {
      zone = 'Zone I';
      description = 'Deep Shadow';
    } else if (ev < -2.5) {
      zone = 'Zone II';
      description = 'First Texture';
    } else if (ev < -1.5) {
      zone = 'Zone III';
      description = 'Shadow Detail';
    } else if (ev < -0.5) {
      zone = 'Zone IV';
      description = 'Low Midtone';
    } else if (ev <= 0.5) {
      zone = 'Zone V';
      description = '18% Middle Grey';
    } else if (ev <= 1.5) {
      zone = 'Zone VI';
      description = 'Skin Midtone';
    } else if (ev <= 2.5) {
      zone = 'Zone VII';
      description = 'Textured White';
    } else if (ev <= 3.5) {
      zone = 'Zone VIII';
      description = 'High Texture';
    } else if (ev <= 4.0) {
      zone = 'Zone IX';
      description = 'Blank White';
    } else {
      zone = 'Zone X';
      description = 'Sensor Saturation';
    }

    return { zone, description, ev: evString };
  }

  private draw() {
    if (this.isCollapsed || !this.ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = this.canvas.getBoundingClientRect();
    const w = rect.width > 0 ? rect.width : 240;
    const h = 64;

    if (
      this.canvas.width !== Math.round(w * dpr) ||
      this.canvas.height !== Math.round(h * dpr)
    ) {
      this.canvas.width = Math.round(w * dpr);
      this.canvas.height = Math.round(h * dpr);
    }

    const ctx = this.ctx;
    ctx.save();
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, w, h);

    // 1. Ansel Adams Zone Reference Grid
    this.drawZoneGrid(ctx, w, h);

    // 2. Continuous Histogram Densities (Luminance + RGB traces)
    if (this.currentRecord && this.currentRecord.bins_luminance.length > 1) {
      this.drawHistogramTraces(ctx, w, h, this.currentRecord);
    }

    // 3. Hover Hairline Indicator
    if (this.hoverFrac !== null) {
      const hx = this.hoverFrac * w;
      ctx.beginPath();
      ctx.setLineDash([2, 2]);
      ctx.moveTo(hx, 0);
      ctx.lineTo(hx, h);
      ctx.strokeStyle = 'rgba(255, 255, 255, 0.5)';
      ctx.lineWidth = 1;
      ctx.stroke();
      ctx.setLineDash([]);
    }

    ctx.restore();
  }

  private drawZoneGrid(ctx: CanvasRenderingContext2D, w: number, h: number) {
    const totalStops = 12.0;

    for (let stop = 0; stop <= 12; stop++) {
      const frac = stop / totalStops;
      const x = frac * w;
      const ev = -8 + stop;

      if (ev === 0) {
        // Zone V (0.0 EV / 18% Middle Grey) - Distinct Accent Marker
        ctx.beginPath();
        ctx.setLineDash([3, 2]);
        ctx.moveTo(x, 0);
        ctx.lineTo(x, h);
        ctx.strokeStyle = 'rgba(235, 140, 50, 0.40)'; // accent tone
        ctx.lineWidth = 1;
        ctx.stroke();
        ctx.setLineDash([]);
      } else if (ev === -5 || ev === 3) {
        // Zone 0 (-5 EV, Dmax) and Zone VIII (+3 EV, Textured White)
        ctx.beginPath();
        ctx.moveTo(x, h - 8);
        ctx.lineTo(x, h);
        ctx.strokeStyle = 'rgba(200, 200, 200, 0.28)';
        ctx.lineWidth = 1;
        ctx.stroke();
      } else {
        // Standard 1-stop ticks along bottom
        ctx.beginPath();
        ctx.moveTo(x, h - 4);
        ctx.lineTo(x, h);
        ctx.strokeStyle = 'rgba(200, 200, 200, 0.18)';
        ctx.lineWidth = 1;
        ctx.stroke();
      }
    }
  }

  private drawHistogramTraces(
    ctx: CanvasRenderingContext2D,
    w: number,
    h: number,
    record: EvHistogramRecord,
  ) {
    const n = record.bins_luminance.length;
    if (n <= 1) return;

    const buildPath = (bins: number[]): Path2D => {
      const path = new Path2D();
      path.moveTo(0, h - bins[0] * (h - 2));
      for (let i = 1; i < n; i++) {
        const x = (i / (n - 1)) * w;
        const y = h - bins[i] * (h - 2);
        path.lineTo(x, y);
      }
      return path;
    };

    // 1. Luminance Monochrome Filled Mass
    const fillPath = buildPath(record.bins_luminance);
    fillPath.lineTo(w, h);
    fillPath.lineTo(0, h);
    fillPath.closePath();

    const grad = ctx.createLinearGradient(0, 0, 0, h);
    grad.addColorStop(0, 'rgba(200, 200, 200, 0.32)');
    grad.addColorStop(1, 'rgba(200, 200, 200, 0.04)');
    ctx.fillStyle = grad;
    ctx.fill(fillPath);

    // 2. Muted Desaturated Channel Traces (Quiet Darkroom Tones)
    if (record.bins_r.length === n) {
      ctx.strokeStyle = 'rgba(224, 89, 89, 0.40)';
      ctx.lineWidth = 0.9;
      ctx.stroke(buildPath(record.bins_r));
    }

    if (record.bins_g.length === n) {
      ctx.strokeStyle = 'rgba(89, 191, 102, 0.40)';
      ctx.lineWidth = 0.9;
      ctx.stroke(buildPath(record.bins_g));
    }

    if (record.bins_b.length === n) {
      ctx.strokeStyle = 'rgba(97, 148, 230, 0.40)';
      ctx.lineWidth = 0.9;
      ctx.stroke(buildPath(record.bins_b));
    }

    // 3. Luminance Stroke Outline
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.55)';
    ctx.lineWidth = 1.0;
    ctx.stroke(buildPath(record.bins_luminance));
  }
}
