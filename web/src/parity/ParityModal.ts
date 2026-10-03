/**
 * First Proof — Mechanical Parity Audit Modal UI
 *
 * Interactive inspector that provides live side-by-side comparison, difference heatmaps,
 * and automated test battery execution between the WebGL GPU canvas and Reference CPU Export.
 */

import type { EditSettingsPayload } from '../WasmBridge';
import type { WebGLRenderer } from '../WebGLRenderer';
import {
  type ParityAuditResult,
  ParityHarness,
  type ParityTestCase,
} from './ParityHarness';

export interface ActivePhotoInput {
  isLinearRec2020: boolean;
  width: number;
  height: number;
  linearData?: Uint16Array;
  imageSource?: CanvasImageSource;
}

export class ParityModal {
  private harness: ParityHarness;
  private modalEl: HTMLDialogElement | null = null;
  private results: ParityAuditResult[] = [];
  private activeTestIndex: number = 0;
  private activeRenderer: WebGLRenderer | null = null;
  private getActiveEdits: (() => EditSettingsPayload) | null = null;
  private getActivePhoto: (() => ActivePhotoInput | null) | null = null;

  constructor() {
    this.harness = new ParityHarness(256, 256);
  }

  public init(
    activeRenderer: WebGLRenderer,
    getActiveEdits: () => EditSettingsPayload,
    getActivePhoto?: () => ActivePhotoInput | null,
  ) {
    this.activeRenderer = activeRenderer;
    this.getActiveEdits = getActiveEdits;
    if (getActivePhoto) {
      this.getActivePhoto = getActivePhoto;
    }
    this.injectModalHtml();
    this.bindEvents();
  }

  private injectModalHtml() {
    if (document.getElementById('parity-audit-modal')) return;

    const modal = document.createElement('dialog');
    modal.id = 'parity-audit-modal';
    modal.className = 'parity-modal';
    modal.innerHTML = `
      <div class="parity-modal-content">
        <div class="parity-modal-header">
          <div class="parity-title-group">
            <h2>Mechanical Parity Diagnostic Suite</h2>
            <span class="parity-subtitle">Dual-Engine Bit-Level Verification: WebGL GPU Preview vs. Reference CPU Export</span>
          </div>
          <button id="btn-close-parity-modal" class="btn btn-icon" title="Close">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18M6 6l12 12"/></svg>
          </button>
        </div>

        <div class="parity-modal-toolbar">
          <button id="btn-run-full-audit" class="btn btn-primary">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"/></svg>
            Run Full Test Battery
          </button>
          <button id="btn-audit-active-photo" class="btn btn-secondary">
            Audit Active Photo Frame
          </button>
          <div class="parity-summary-pill" id="parity-overall-badge">Ready to audit</div>
        </div>

        <div class="parity-modal-body">
          <div class="parity-test-list-panel">
            <div class="parity-panel-title">Test Battery Cases</div>
            <div class="parity-table-scroll">
              <table class="parity-table">
                <thead>
                  <tr>
                    <th>Status</th>
                    <th>Test Scenario</th>
                    <th>Max Δ</th>
                    <th>Mean Δ</th>
                    <th>PSNR</th>
                  </tr>
                </thead>
                <tbody id="parity-test-rows">
                  <tr><td colspan="5" class="empty-hint">Click "Run Full Test Battery" to evaluate WebGL vs Export parity.</td></tr>
                </tbody>
              </table>
            </div>
          </div>

          <div class="parity-inspection-panel">
            <div class="parity-panel-title" id="parity-inspect-title">Diagnostic Visual Inspection</div>
            <div class="parity-canvas-container">
              <div class="parity-canvas-box">
                <span class="canvas-label">Difference Heatmap (Amplified 15×)</span>
                <canvas id="parity-heatmap-canvas" width="256" height="256"></canvas>
              </div>
            </div>

            <div class="parity-diagnosis-box" id="parity-diagnosis-box">
              <div class="diagnosis-header">Mathematical Analysis & Stage Diagnosis:</div>
              <div class="diagnosis-body" id="parity-diagnosis-text">Select a test case to inspect diagnosis details.</div>
            </div>

            <div class="parity-worst-pixel" id="parity-worst-pixel">
              <div><strong>Worst Pixel:</strong> <span id="worst-pixel-coord">-</span></div>
              <div><strong>WebGL Output:</strong> <span id="worst-gl-rgb">-</span></div>
              <div><strong>Reference Export:</strong> <span id="worst-exp-rgb">-</span></div>
            </div>
          </div>
        </div>
      </div>
    `;

    document.body.appendChild(modal);
    this.modalEl = modal;
  }

  private bindEvents() {
    const closeBtn = document.getElementById('btn-close-parity-modal');
    closeBtn?.addEventListener('click', () => this.modalEl?.close());

    const runBtn = document.getElementById('btn-run-full-audit');
    runBtn?.addEventListener('click', () => this.runFullAudit());

    const activePhotoBtn = document.getElementById('btn-audit-active-photo');
    activePhotoBtn?.addEventListener('click', () => this.auditActivePhoto());
  }

  public show() {
    this.modalEl?.showModal();
    if (this.results.length === 0) {
      this.runFullAudit();
    }
  }

  public runFullAudit() {
    const badge = document.getElementById('parity-overall-badge');
    if (badge) {
      badge.innerText = 'Evaluating GL & Export math...';
      badge.className = 'parity-summary-pill pending';
    }

    // Run audit
    this.results = this.harness.runFullAudit();
    this.renderTableRows();

    const passAll = this.results.every((r) => r.pass);
    if (badge) {
      badge.innerText = passAll
        ? '✓ 100% Bit-Level Parity Pass'
        : '⚠ Parity Divergence Detected';
      badge.className = `parity-summary-pill ${passAll ? 'pass' : 'fail'}`;
    }

    if (this.results.length > 0) {
      this.selectTest(0);
    }
  }

  private renderTableRows() {
    const tbody = document.getElementById('parity-test-rows');
    if (!tbody) return;

    tbody.innerHTML = '';
    this.results.forEach((res, idx) => {
      const tr = document.createElement('tr');
      tr.className = `parity-row ${idx === this.activeTestIndex ? 'active' : ''}`;
      tr.innerHTML = `
        <td><span class="status-indicator ${res.pass ? 'pass' : 'fail'}">${res.pass ? 'PASS' : 'FAIL'}</span></td>
        <td class="test-name">${res.testName}</td>
        <td class="num ${res.maxChannelDelta > 2 ? 'divergent' : ''}">${res.maxChannelDelta}/255</td>
        <td class="num">${res.meanChannelDelta.toFixed(2)}</td>
        <td class="num">${res.psnr.toFixed(1)} dB</td>
      `;
      tr.addEventListener('click', () => this.selectTest(idx));
      tbody.appendChild(tr);
    });
  }

  private selectTest(idx: number) {
    this.activeTestIndex = idx;
    const res = this.results[idx];
    if (!res) return;

    // Highlight row
    const rows = document.querySelectorAll('.parity-row');
    rows.forEach((r, i) => {
      if (i === idx) r.classList.add('active');
      else r.classList.remove('active');
    });

    const titleEl = document.getElementById('parity-inspect-title');
    if (titleEl)
      titleEl.innerText = `${res.testName} — ${res.pass ? 'Exact Match' : 'Divergence Analysis'}`;

    // Draw Heatmap
    const canvas = document.getElementById(
      'parity-heatmap-canvas',
    ) as HTMLCanvasElement;
    if (canvas && res.heatmapData) {
      canvas.width = res.heatmapData.width;
      canvas.height = res.heatmapData.height;
      const ctx = canvas.getContext('2d');
      if (ctx) {
        ctx.putImageData(res.heatmapData, 0, 0);
      }
    }

    // Update Diagnosis
    const diagEl = document.getElementById('parity-diagnosis-text');
    if (diagEl) {
      diagEl.innerText = res.stageDiagnosis;
      diagEl.className = `diagnosis-body ${res.pass ? 'pass' : 'fail'}`;
    }

    // Update Worst Pixel
    const coordEl = document.getElementById('worst-pixel-coord');
    const glEl = document.getElementById('worst-gl-rgb');
    const expEl = document.getElementById('worst-exp-rgb');
    if (coordEl)
      coordEl.innerText = `(${res.worstPixelCoord[0]}, ${res.worstPixelCoord[1]})`;
    if (glEl) glEl.innerText = `rgb(${res.worstGlRgb.join(', ')})`;
    if (expEl) expEl.innerText = `rgb(${res.worstExportRgb.join(', ')})`;
  }

  public auditActivePhoto() {
    if (!this.activeRenderer || !this.getActiveEdits) return;
    const edits = this.getActiveEdits();
    const photo = this.getActivePhoto ? this.getActivePhoto() : null;

    const testEdits = {
      ...edits,
      rotation: 0,
      flip_horizontal: false,
      flip_vertical: false,
      crop_left: 0.0,
      crop_top: 0.0,
      crop_right: 1.0,
      crop_bottom: 1.0,
      crop_angle: 0.0,
      perspective_horizontal: 0.0,
      perspective_vertical: 0.0,
    };

    let activeTestCase: ParityTestCase;

    if (photo && photo.isLinearRec2020 && photo.linearData) {
      // 1. Sample center 256x256 patch from the uncompressed 16-bit Linear Rec.2020 RAW sensor frame
      const w = Math.min(photo.width, 256);
      const h = Math.min(photo.height, 256);
      const startX = Math.floor((photo.width - w) / 2);
      const startY = Math.floor((photo.height - h) / 2);
      const sampledLin = new Uint16Array(w * h * 4);
      for (let y = 0; y < h; y++) {
        for (let x = 0; x < w; x++) {
          const srcIdx = ((startY + y) * photo.width + (startX + x)) * 4;
          const dstIdx = (y * w + x) * 4;
          sampledLin[dstIdx] = photo.linearData[srcIdx];
          sampledLin[dstIdx + 1] = photo.linearData[srcIdx + 1];
          sampledLin[dstIdx + 2] = photo.linearData[srcIdx + 2];
          sampledLin[dstIdx + 3] = photo.linearData[srcIdx + 3];
        }
      }

      activeTestCase = {
        id: 'active_photo_raw',
        name: 'Active RAW Frame (Linear Rec.2020) with Current Sliders',
        category: 'active_photo',
        edits: testEdits,
        isLinearRec2020: true,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => ({ data: sampledLin, width: w, height: h }),
      };
    } else if (photo && !photo.isLinearRec2020 && photo.imageSource) {
      // 2. Sample from the unadjusted JPEG source image (not the already-rendered display canvas!)
      const w = Math.min(photo.width, 256);
      const h = Math.min(photo.height, 256);
      const tempCanvas = document.createElement('canvas');
      tempCanvas.width = w;
      tempCanvas.height = h;
      const ctx = tempCanvas.getContext('2d');
      if (ctx) {
        ctx.drawImage(photo.imageSource, 0, 0, w, h);
      }
      const imgData = ctx?.getImageData(0, 0, w, h) || new ImageData(w, h);

      activeTestCase = {
        id: 'active_photo_jpeg',
        name: 'Active Photo Frame (sRGB Source) with Current Sliders',
        category: 'active_photo',
        edits: testEdits,
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => imgData,
      };
    } else {
      // 3. Fallback: No photo loaded yet — verify active sliders against Macbeth ColorChecker standard
      const w = 256;
      const h = 256;
      const imgData = this.harness.createMacbethTexture();

      activeTestCase = {
        id: 'active_sliders_macbeth',
        name: 'Active Sliders Verification (Macbeth Reference Target)',
        category: 'active_photo',
        edits,
        isLinearRec2020: false,
        textureWidth: w,
        textureHeight: h,
        createTexture: () => imgData,
      };
    }

    const res = this.harness.auditTestCase(activeTestCase);
    this.results.unshift(res);
    this.renderTableRows();
    this.selectTest(0);
  }
}
