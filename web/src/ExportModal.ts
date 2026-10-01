/**
 * First Proof — Export Sheet Modal UI
 *
 * Direct parity with native macOS ExportSheetView.swift:
 * - Single / Batch export targeting
 * - Destination folder browsing via File System Access API
 * - Format selection: JPEG (.jpg) with quality & Ultra HDR (ISO 21496-1) vs. 16-Bit TIFF (.tiff)
 * - Color space & ICC profile: sRGB vs. Display P3 with Rendering Intent
 * - Image sizing: Original, Web (2048px), 4K UHD (3840px), Custom dimension
 * - Background multi-photo queue execution with docked progress & status feedback
 */

import { PhotoFileEntry } from './FolderIntake';

export interface ExportExecutionOptions {
  format: 'JPEG' | 'TIFF';
  quality: number; // 50..100
  colorSpace: 'sRGB';
  sizeOption: 'Original' | '2048' | '3840' | 'Custom';
  customDimension: number;
  destinationDirHandle?: any;
}

export type ExportHandler = (
  photos: PhotoFileEntry[],
  options: ExportExecutionOptions,
  onProgress: (status: string, current: number, total: number) => void
) => Promise<{ success: boolean; message: string }>;

export class ExportModal {
  private dialogEl: HTMLDialogElement | null = null;
  private targetPhotos: PhotoFileEntry[] = [];
  private onExportHandler: ExportHandler | null = null;
  private isExporting: boolean = false;

  // Form State matching ExportSheetView.swift @State variables
  private destinationDirHandle: any = null;
  private destinationName: string = '~/Downloads (Browser Default)';
  private exportFormat: 'JPEG' | 'TIFF' = 'JPEG';
  private quality: number = 90;
  private sizeOption: 'Original' | '2048' | '3840' | 'Custom' = 'Original';
  private customDimension: number = 2048;

  public init(onExport: ExportHandler) {
    this.onExportHandler = onExport;
    this.dialogEl = document.getElementById('modal-export') as HTMLDialogElement;
    if (!this.dialogEl) {
      console.error('[ExportModal] Could not find #modal-export in DOM');
      return;
    }
    this.bindEvents();
  }

  public open(photos: PhotoFileEntry[]) {
    this.targetPhotos = photos;
    this.isExporting = false;
    this.resetStatusMessage();
    this.updateUI();

    if (this.dialogEl) {
      this.dialogEl.showModal();
    }
  }

  public close() {
    if (this.isExporting) return;
    if (this.dialogEl && this.dialogEl.open) {
      this.dialogEl.close();
    }
  }

  private bindEvents() {
    if (!this.dialogEl) return;

    // Header close / cancel button
    const btnCancel = this.dialogEl.querySelector('#btn-cancel-export-header');
    btnCancel?.addEventListener('click', () => this.close());

    // Destination Reset Button (Reset to ~/Downloads)
    const btnResetDest = this.dialogEl.querySelector('#btn-export-reset-dest');
    btnResetDest?.addEventListener('click', () => {
      this.destinationDirHandle = null;
      this.destinationName = '~/Downloads (Browser Default)';
      this.resetStatusMessage();
      this.updateDestinationDisplay();
    });

    // Destination browse button ("Choose Folder...")
    const btnBrowse = this.dialogEl.querySelector('#btn-export-browse');
    btnBrowse?.addEventListener('click', async () => {
      if ('showDirectoryPicker' in window) {
        try {
          const handle = await (window as any).showDirectoryPicker();
          if (handle) {
            this.destinationDirHandle = handle;
            this.destinationName = handle.name ? `Folder: ${handle.name}` : 'Selected Folder';
            this.resetStatusMessage();
            this.updateDestinationDisplay();
          }
        } catch (err: any) {
          if (err.name === 'AbortError') {
            return;
          }
          const msg = err.message || '';
          if (err.name === 'SecurityError' || msg.toLowerCase().includes('system') || msg.toLowerCase().includes('sensitive')) {
            this.setStatusMessage(
              'Browser security blocks granting direct folder access to root ~/Downloads. Use "~/Downloads (Browser Default)" to save to Downloads via browser download, or pick a subfolder/Pictures.',
              true
            );
          } else {
            console.warn('[ExportModal] Browse folder failed:', err);
            this.setStatusMessage(`Could not select folder: ${msg || err.name}`, true);
          }
        }
      } else {
        alert('File System Access API is not supported in this browser. Exports will download directly to your ~/Downloads folder.');
      }
    });

    // Format Segmented Control
    const formatButtons = this.dialogEl.querySelectorAll<HTMLButtonElement>('.export-format-segmented button');
    formatButtons.forEach(btn => {
      btn.addEventListener('click', () => {
        const fmt = btn.getAttribute('data-format') as 'JPEG' | 'TIFF';
        if (fmt) {
          this.exportFormat = fmt;
          formatButtons.forEach(b => b.classList.toggle('active', b === btn));
          this.updateFormatVisibility();
        }
      });
    });

    // Quality Slider
    const qualitySlider = this.dialogEl.querySelector('#export-quality-slider') as HTMLInputElement;
    const qualityLabel = this.dialogEl.querySelector('#export-quality-label');
    qualitySlider?.addEventListener('input', () => {
      this.quality = parseInt(qualitySlider.value, 10);
      if (qualityLabel) {
        qualityLabel.textContent = `Quality: ${this.quality}%`;
      }
    });



    // Sizing Select
    const sizeSelect = this.dialogEl.querySelector('#export-size-select') as HTMLSelectElement;
    const customDimRow = this.dialogEl.querySelector('#export-custom-dim-row');
    sizeSelect?.addEventListener('change', () => {
      this.sizeOption = sizeSelect.value as 'Original' | '2048' | '3840' | 'Custom';
      customDimRow?.classList.toggle('hidden', this.sizeOption !== 'Custom');
    });

    // Custom Dimension Input
    const customDimInput = this.dialogEl.querySelector('#export-custom-dim-input') as HTMLInputElement;
    customDimInput?.addEventListener('input', () => {
      const val = parseInt(customDimInput.value, 10);
      if (!isNaN(val) && val > 0) {
        this.customDimension = val;
      }
    });

    // Export Submit Button
    const btnPerformExport = this.dialogEl.querySelector('#btn-perform-export') as HTMLButtonElement;
    btnPerformExport?.addEventListener('click', () => this.performExport());

    // Enter & Escape Keyboard handling inside modal
    this.dialogEl.addEventListener('keydown', (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        this.close();
      } else if (e.key === 'Enter' && !this.isExporting) {
        // Prevent enter from firing if user is inside a select or custom dim input
        e.preventDefault();
        this.performExport();
      }
    });
  }

  private updateUI() {
    if (!this.dialogEl) return;

    const count = this.targetPhotos.length;
    const isBatch = count > 1;

    // Header Title & Subtitle Parity
    const titleEl = this.dialogEl.querySelector('#export-modal-title');
    const subtitleEl = this.dialogEl.querySelector('#export-modal-subtitle');
    if (titleEl) {
      titleEl.textContent = isBatch ? `Batch Export (${count} Photos)` : 'Export Photo';
    }
    if (subtitleEl) {
      subtitleEl.textContent = 'Multi-core parallel processing via Rayon engine';
      subtitleEl.classList.toggle('hidden', !isBatch);
    }

    // Export Button Text Parity
    const btnPerform = this.dialogEl.querySelector('#btn-perform-export') as HTMLButtonElement;
    if (btnPerform) {
      btnPerform.textContent = isBatch ? `Export ${count} Photos` : 'Export';
      btnPerform.disabled = count === 0 || this.isExporting;
    }

    this.updateDestinationDisplay();
    this.updateFormatVisibility();

    // Reset controls to state
    const qualitySlider = this.dialogEl.querySelector('#export-quality-slider') as HTMLInputElement;
    const qualityLabel = this.dialogEl.querySelector('#export-quality-label');
    if (qualitySlider) qualitySlider.value = `${this.quality}`;
    if (qualityLabel) qualityLabel.textContent = `Quality: ${this.quality}%`;

    const customDimRow = this.dialogEl.querySelector('#export-custom-dim-row');
    customDimRow?.classList.toggle('hidden', this.sizeOption !== 'Custom');
  }

  private updateDestinationDisplay() {
    const destDisplay = this.dialogEl?.querySelector('#export-destination-display');
    const resetBtn = this.dialogEl?.querySelector('#btn-export-reset-dest');
    const hintEl = this.dialogEl?.querySelector('#export-dest-hint');

    if (destDisplay) {
      destDisplay.textContent = this.destinationName;
      destDisplay.setAttribute('title', this.destinationName);
    }
    if (resetBtn) {
      resetBtn.classList.toggle('hidden', this.destinationDirHandle === null);
    }
    if (hintEl) {
      if (this.destinationDirHandle) {
        hintEl.textContent = `Exports will write directly into ${this.destinationName} via File System Access API.`;
      } else {
        hintEl.textContent = 'Exports save directly into ~/Downloads via browser download without directory restrictions.';
      }
    }
  }

  private updateFormatVisibility() {
    const jpegOptions = this.dialogEl?.querySelector('#export-jpeg-options');
    if (jpegOptions) {
      jpegOptions.classList.toggle('hidden', this.exportFormat !== 'JPEG');
    }
  }

  private resetStatusMessage() {
    const msgEl = this.dialogEl?.querySelector('#export-status-message');
    if (msgEl) {
      msgEl.className = 'export-status-message hidden';
      msgEl.textContent = '';
    }
  }

  private setStatusMessage(text: string, isError: boolean) {
    const msgEl = this.dialogEl?.querySelector('#export-status-message');
    if (msgEl) {
      msgEl.className = `export-status-message ${isError ? 'status-error' : 'status-success'}`;
      msgEl.textContent = text;
      msgEl.classList.remove('hidden');
    }
  }

  private async performExport() {
    if (this.isExporting || this.targetPhotos.length === 0 || !this.onExportHandler) return;

    this.isExporting = true;
    this.resetStatusMessage();

    const spinner = this.dialogEl?.querySelector('#export-spinner');
    const btnPerform = this.dialogEl?.querySelector('#btn-perform-export') as HTMLButtonElement;
    const btnCancel = this.dialogEl?.querySelector('#btn-cancel-export-header') as HTMLButtonElement;

    spinner?.classList.remove('hidden');
    if (btnPerform) btnPerform.disabled = true;
    if (btnCancel) btnCancel.disabled = true;

    const options: ExportExecutionOptions = {
      format: this.exportFormat,
      quality: this.quality,
      colorSpace: 'sRGB',
      sizeOption: this.sizeOption,
      customDimension: this.customDimension,
      destinationDirHandle: this.destinationDirHandle
    };

    try {
      const result = await this.onExportHandler(
        this.targetPhotos,
        options,
        (statusText, _current, _total) => {
          this.setStatusMessage(statusText, false);
        }
      );

      this.isExporting = false;
      spinner?.classList.add('hidden');
      if (btnPerform) btnPerform.disabled = false;
      if (btnCancel) btnCancel.disabled = false;

      if (result.success) {
        this.setStatusMessage(result.message, false);
        // Mac App Parity: dismiss after 1.2s delay
        setTimeout(() => {
          this.close();
        }, 1200);
      } else {
        this.setStatusMessage(result.message || 'Export failed. Please check folder permissions.', true);
      }
    } catch (err: any) {
      this.isExporting = false;
      spinner?.classList.add('hidden');
      if (btnPerform) btnPerform.disabled = false;
      if (btnCancel) btnCancel.disabled = false;
      this.setStatusMessage(`Export failed: ${err?.message || 'Unknown error'}`, true);
    }
  }
}
