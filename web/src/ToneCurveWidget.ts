import { CurvePoint, ToneCurvesPayload, WasmBridge } from './WasmBridge';

export class ToneCurveWidget {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private channel: 'master' | 'red' | 'green' | 'blue' = 'master';
  private points: Record<string, CurvePoint[]> = {
    master: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
    red: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
    green: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
    blue: [{ x: 0, y: 0 }, { x: 1, y: 1 }]
  };
  private onChange: () => void;
  private draggingIdx: number = -1;

  constructor(canvas: HTMLCanvasElement, onChange: () => void) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d')!;
    this.onChange = onChange;
    this.bindEvents();
    this.render();
  }

  public setChannel(ch: 'master' | 'red' | 'green' | 'blue') {
    this.channel = ch;
    this.render();
  }

  public getPoints(): ToneCurvesPayload {
    return this.points as unknown as ToneCurvesPayload;
  }

  public setPoints(points: ToneCurvesPayload) {
    if (points.master) this.points.master = [...points.master];
    if (points.red) this.points.red = [...points.red];
    if (points.green) this.points.green = [...points.green];
    if (points.blue) this.points.blue = [...points.blue];
    this.render();
  }

  public resetCurrentChannel() {
    this.points[this.channel] = [{ x: 0, y: 0 }, { x: 1, y: 1 }];
    this.render();
    this.onChange();
  }

  private bindEvents() {
    this.canvas.addEventListener('mousedown', (e) => {
      const rect = this.canvas.getBoundingClientRect();
      const mx = (e.clientX - rect.left) / rect.width;
      const my = 1.0 - (e.clientY - rect.top) / rect.height;

      const pts = this.points[this.channel];
      // Check existing point match (hit radius ~0.08)
      let found = -1;
      for (let i = 0; i < pts.length; i++) {
        const dx = pts[i].x - mx;
        const dy = pts[i].y - my;
        if (Math.sqrt(dx * dx + dy * dy) < 0.08) {
          found = i;
          break;
        }
      }

      if (found >= 0) {
        this.draggingIdx = found;
      } else {
        // Add new point
        pts.push({ x: mx, y: my });
        pts.sort((a, b) => a.x - b.x);
        this.draggingIdx = pts.findIndex(p => Math.abs(p.x - mx) < 0.001);
        this.onChange();
      }
      this.render();
    });

    window.addEventListener('mousemove', (e) => {
      if (this.draggingIdx < 0) return;
      const rect = this.canvas.getBoundingClientRect();
      let mx = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      let my = Math.max(0, Math.min(1, 1.0 - (e.clientY - rect.top) / rect.height));

      const pts = this.points[this.channel];
      // Pin first and last x coordinate
      if (this.draggingIdx === 0) mx = 0;
      if (this.draggingIdx === pts.length - 1) mx = 1;

      pts[this.draggingIdx].x = mx;
      pts[this.draggingIdx].y = my;
      pts.sort((a, b) => a.x - b.x);
      this.draggingIdx = pts.findIndex(p => p.y === my);

      this.render();
      this.onChange();
    });

    window.addEventListener('mouseup', () => {
      if (this.draggingIdx >= 0) {
        this.draggingIdx = -1;
        this.render();
      }
    });
  }

  public render() {
    const w = this.canvas.width;
    const h = this.canvas.height;
    const ctx = this.ctx;

    ctx.clearRect(0, 0, w, h);

    // Grid lines
    ctx.strokeStyle = '#1e2433';
    ctx.lineWidth = 1;
    for (let i = 1; i < 4; i++) {
      const p = (i / 4) * w;
      ctx.beginPath(); ctx.moveTo(p, 0); ctx.lineTo(p, h); ctx.stroke();
      ctx.beginPath(); ctx.moveTo(0, p); ctx.lineTo(w, p); ctx.stroke();
    }
    // Diagonal reference
    ctx.strokeStyle = '#2a3245';
    ctx.beginPath(); ctx.moveTo(0, h); ctx.lineTo(w, 0); ctx.stroke();

    // Draw active channel curve
    const pts = this.points[this.channel];
    const colors: Record<string, string> = {
      master: '#f59e0b',
      red: '#ef4444',
      green: '#10b981',
      blue: '#3b82f6'
    };
    const strokeColor = colors[this.channel];

    ctx.strokeStyle = strokeColor;
    ctx.lineWidth = 2;
    ctx.beginPath();

    // Draw active channel curve using exact WASM monotone cubic spline
    let samples: number[] | null = null;
    try {
      const bridge = WasmBridge.getInstance();
      const lut = bridge.evaluateCurves(this.getPoints());
      samples = lut[this.channel];
    } catch (_) {}

    if (samples && samples.length === 256) {
      for (let i = 0; i < 256; i++) {
        const cx = (i / 255.0) * w;
        const cy = (1.0 - samples[i]) * h;
        if (i === 0) ctx.moveTo(cx, cy);
        else ctx.lineTo(cx, cy);
      }
    } else {
      for (let i = 0; i < pts.length; i++) {
        const cx = pts[i].x * w;
        const cy = (1.0 - pts[i].y) * h;
        if (i === 0) ctx.moveTo(cx, cy);
        else ctx.lineTo(cx, cy);
      }
    }
    ctx.stroke();

    // Control point dots
    for (let i = 0; i < pts.length; i++) {
      const cx = pts[i].x * w;
      const cy = (1.0 - pts[i].y) * h;
      ctx.fillStyle = (i === this.draggingIdx) ? '#ffffff' : strokeColor;
      ctx.beginPath();
      ctx.arc(cx, cy, 4.5, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = '#000000';
      ctx.lineWidth = 1;
      ctx.stroke();
    }
  }
}
