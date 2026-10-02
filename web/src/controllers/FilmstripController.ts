import { PhotoFileEntry, FolderIntake } from '../FolderIntake';
import { ThumbnailCacheStore } from '../ThumbnailCacheStore';
import { readExifOrientationFromBlob, decodeExifOrientation } from '../utils/exifReader';
import { WasmBridge } from '../WasmBridge';
import { PhotoLoaderService } from '../services/PhotoLoaderService';

export interface FilmstripDelegate {
  onSelectPhoto: (idx: number, centerFilmstrip: boolean, keepMultiSelection: boolean) => void;
}

export class FilmstripController {
  private container: HTMLElement;
  private countBadge: HTMLElement | null;
  private emptyState: HTMLElement | null;
  private bridge: WasmBridge;
  private loaderService: PhotoLoaderService;
  private delegate: FilmstripDelegate;

  private fileEntries: PhotoFileEntry[] = [];
  private activeIdx: number = -1;
  private selectedIndices: Set<number> = new Set();
  private cardRotations: Map<number, { rotation: number; flipH: boolean; flipV: boolean }> = new Map();
  private thumbCache: Map<number, string> = new Map(); // idx -> Blob URL
  private thumbLoadingSet: Set<number> = new Set();

  private observer: IntersectionObserver | null = null;
  private thumbnailQueue: number[] = [];
  private activeDecodes: number = 0;
  private static readonly MAX_CONCURRENT_THUMBNAILS = 3;

  constructor(
    container: HTMLElement,
    countBadge: HTMLElement | null,
    emptyState: HTMLElement | null,
    bridge: WasmBridge,
    loaderService: PhotoLoaderService,
    delegate: FilmstripDelegate
  ) {
    this.container = container;
    this.countBadge = countBadge;
    this.emptyState = emptyState;
    this.bridge = bridge;
    this.loaderService = loaderService;
    this.delegate = delegate;

    this.initObserver();
  }

  private initObserver() {
    if ('IntersectionObserver' in window) {
      this.observer = new IntersectionObserver(
        (entries) => {
          for (const entry of entries) {
            if (entry.isIntersecting) {
              const idxStr = entry.target.getAttribute('data-idx');
              if (idxStr !== null) {
                const idx = parseInt(idxStr, 10);
                this.enqueueThumbnail(idx);
              }
            }
          }
        },
        {
          root: this.container,
          rootMargin: '300px', // Pre-fetch thumbnails 300px before scrolling into view
          threshold: 0.01,
        }
      );
    }
  }

  public getEntries(): PhotoFileEntry[] {
    return this.fileEntries;
  }

  public getActiveIndex(): number {
    return this.activeIdx;
  }

  public getActiveEntry(): PhotoFileEntry | null {
    if (this.activeIdx >= 0 && this.activeIdx < this.fileEntries.length) {
      return this.fileEntries[this.activeIdx];
    }
    return null;
  }

  public getSelectedPhotos(): PhotoFileEntry[] {
    if (this.selectedIndices.size > 1) {
      const sorted = Array.from(this.selectedIndices).sort((a, b) => a - b);
      return sorted.map((i) => this.fileEntries[i]).filter(Boolean);
    }
    if (this.activeIdx >= 0 && this.activeIdx < this.fileEntries.length) {
      return [this.fileEntries[this.activeIdx]];
    }
    return [];
  }

  public loadPhotos(entries: PhotoFileEntry[], initialPhotoName?: string | null) {
    this.fileEntries = entries;
    this.activeIdx = -1;
    this.selectedIndices.clear();

    if (this.emptyState) {
      this.emptyState.classList.add('hidden');
    }
    if (this.countBadge) {
      this.countBadge.innerText = `${entries.length} Photos Loaded`;
    }

    // Revoke previous session Blob URLs to avoid memory leaks
    for (const [, oldUrl] of this.thumbCache) {
      URL.revokeObjectURL(oldUrl);
    }
    this.thumbCache.clear();
    this.cardRotations.clear();
    this.thumbnailQueue = [];
    this.thumbLoadingSet.clear();

    if (this.observer) {
      this.observer.disconnect();
    }

    // Render Filmstrip Cards
    this.container.innerHTML = '';

    entries.forEach((entry, idx) => {
      const card = document.createElement('div');
      card.className = `thumb-card ${idx === 0 ? 'active' : ''}`;
      card.setAttribute('data-idx', `${idx}`);
      card.id = `thumb-card-${idx}`;

      const iconContainer = document.createElement('div');
      iconContainer.className = 'thumb-icon-placeholder';
      iconContainer.innerHTML = `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/></svg>`;

      const title = document.createElement('div');
      title.className = 'thumb-title';
      title.innerText = entry.name;

      card.appendChild(iconContainer);
      card.appendChild(title);

      card.addEventListener('click', (e: MouseEvent) => {
        if (e.metaKey || e.ctrlKey) {
          if (this.selectedIndices.has(idx)) {
            if (this.selectedIndices.size > 1) {
              this.selectedIndices.delete(idx);
            }
          } else {
            this.selectedIndices.add(idx);
          }
          this.updateCardSelectionClasses();
          this.delegate.onSelectPhoto(idx, false, true);
        } else if (e.shiftKey && this.activeIdx >= 0) {
          const start = Math.min(this.activeIdx, idx);
          const end = Math.max(this.activeIdx, idx);
          this.selectedIndices.clear();
          for (let i = start; i <= end; i++) {
            this.selectedIndices.add(i);
          }
          this.updateCardSelectionClasses();
          this.delegate.onSelectPhoto(idx, false, true);
        } else {
          this.selectedIndices.clear();
          this.selectedIndices.add(idx);
          this.updateCardSelectionClasses();
          this.delegate.onSelectPhoto(idx, false, false);
        }
      });

      this.container.appendChild(card);

      if (this.observer) {
        this.observer.observe(card);
      }

      // Fast background EXIF orientation scan from raw byte slice (sub-0.1ms per image)
      readExifOrientationFromBlob(entry.file).then(async (orientation) => {
        if (this.fileEntries[idx] !== entry) return;
        const exifOri = decodeExifOrientation(orientation);
        let rot = exifOri.rotation;
        let flipH = exifOri.flipH;
        let flipV = exifOri.flipV;

        const sidecarXml = await FolderIntake.readSidecarFromDisk(entry);
        if (this.fileEntries[idx] !== entry) return;
        if (sidecarXml) {
          try {
            const xmp = this.bridge.parseXmp(sidecarXml);
            if (xmp.edits?.rotation !== undefined && xmp.edits?.rotation !== null) {
              rot = xmp.edits.rotation;
              flipH = xmp.edits.flip_horizontal ?? false;
              flipV = xmp.edits.flip_vertical ?? false;
            }
          } catch (_) {}
        }
        this.updateThumbnailCardOrientation(idx, rot, flipH, flipV);
      });
    });

    // Select initial photo
    if (entries.length > 0) {
      let initialIdx = 0;
      if (initialPhotoName) {
        const found = entries.findIndex((e) => e.name === initialPhotoName);
        if (found >= 0) initialIdx = found;
      }
      this.selectIndex(initialIdx, true, false);
      this.delegate.onSelectPhoto(initialIdx, true, false);
    }
  }

  public selectIndex(idx: number, centerFilmstrip: boolean = false, keepMultiSelection: boolean = false) {
    if (idx < 0 || idx >= this.fileEntries.length) return;
    this.activeIdx = idx;

    if (!keepMultiSelection) {
      this.selectedIndices.clear();
      this.selectedIndices.add(idx);
    }

    this.updateCardSelectionClasses();

    if (centerFilmstrip) {
      this.centerActiveThumb(true);
    }
  }

  public selectNext(wrap: boolean = false): void {
    if (this.fileEntries.length === 0) return;
    let nextIdx = this.activeIdx + 1;
    if (nextIdx >= this.fileEntries.length) {
      nextIdx = wrap ? 0 : this.fileEntries.length - 1;
    }
    if (nextIdx !== this.activeIdx) {
      this.selectIndex(nextIdx, true, false);
      this.delegate.onSelectPhoto(nextIdx, true, false);
    }
  }

  public selectPrevious(wrap: boolean = false): void {
    if (this.fileEntries.length === 0) return;
    let prevIdx = this.activeIdx - 1;
    if (prevIdx < 0) {
      prevIdx = wrap ? this.fileEntries.length - 1 : 0;
    }
    if (prevIdx !== this.activeIdx) {
      this.selectIndex(prevIdx, true, false);
      this.delegate.onSelectPhoto(prevIdx, true, false);
    }
  }

  public updateCardSelectionClasses() {
    const cards = this.container.querySelectorAll('.thumb-card');
    cards.forEach((c, i) => {
      c.classList.toggle('active', i === this.activeIdx);
      c.classList.toggle('selected', this.selectedIndices.has(i));
    });
  }

  public centerActiveThumb(smooth: boolean = true) {
    const cardEl = document.getElementById(`thumb-card-${this.activeIdx}`);
    if (cardEl && this.container) {
      const containerWidth = this.container.clientWidth;
      const cardOffsetLeft = cardEl.offsetLeft;
      const cardWidth = cardEl.offsetWidth;
      const targetScrollLeft = cardOffsetLeft - containerWidth / 2 + cardWidth / 2;

      this.container.scrollTo({
        left: Math.max(0, targetScrollLeft),
        behavior: smooth ? 'smooth' : 'auto',
      });
    }
  }

  public updateThumbnailCardOrientation(
    idx: number,
    rotation: number = 0,
    flipH: boolean = false,
    flipV: boolean = false
  ) {
    this.cardRotations.set(idx, { rotation, flipH, flipV });
    const cardEl = document.getElementById(`thumb-card-${idx}`);
    if (!cardEl) return;
    const img = cardEl.querySelector('.thumb-img') as HTMLImageElement | null;
    if (!img) return;

    const rot = ((rotation % 360) + 360) % 360;
    const transforms: string[] = [];
    if (rot !== 0) transforms.push(`rotate(${rot}deg)`);
    if (rot === 90 || rot === 270) transforms.push('scale(0.6667)');
    if (flipH) transforms.push('scaleX(-1)');
    if (flipV) transforms.push('scaleY(-1)');

    img.style.transform = transforms.length > 0 ? transforms.join(' ') : 'none';
  }

  public updateRatingFlagBadge(idx: number, rating: number, status: string) {
    const cardEl = document.getElementById(`thumb-card-${idx}`);
    if (!cardEl) return;

    let ratingBadge = cardEl.querySelector('.thumb-badge-rating') as HTMLElement;
    if (rating > 0) {
      if (!ratingBadge) {
        ratingBadge = document.createElement('div');
        ratingBadge.className = 'thumb-badge-rating';
        cardEl.appendChild(ratingBadge);
      }
      ratingBadge.innerText = '★'.repeat(rating);
    } else if (ratingBadge) {
      ratingBadge.remove();
    }

    let flagBadge = cardEl.querySelector('.thumb-badge-flag') as HTMLElement;
    if (status === 'Pick' || status === 'Reject') {
      if (!flagBadge) {
        flagBadge = document.createElement('div');
        flagBadge.className = 'thumb-badge-flag';
        cardEl.appendChild(flagBadge);
      }
      flagBadge.className = `thumb-badge-flag ${status.toLowerCase()}`;
      flagBadge.innerText = status.toUpperCase();
    } else if (flagBadge) {
      flagBadge.remove();
    }
  }

  public setThumbnailImage(idx: number, previewUrl: string) {
    this.thumbCache.set(idx, previewUrl);
    const cardEl = document.getElementById(`thumb-card-${idx}`);
    if (!cardEl) return;

    const existingImg = cardEl.querySelector('img') as HTMLImageElement | null;
    if (existingImg) {
      if (existingImg.src !== previewUrl) existingImg.src = previewUrl;
    } else {
      const img = document.createElement('img');
      img.className = 'thumb-img';
      img.src = previewUrl;
      const placeholder = cardEl.querySelector('.thumb-icon-placeholder');
      if (placeholder) placeholder.remove();
      cardEl.insertBefore(img, cardEl.firstChild);
    }

    const knownRot = this.cardRotations.get(idx);
    if (knownRot) {
      this.updateThumbnailCardOrientation(idx, knownRot.rotation, knownRot.flipH, knownRot.flipV);
    }
  }

  // --- Concurrency-Controlled Lazy Thumbnail Loading ---
  private enqueueThumbnail(idx: number) {
    if (idx < 0 || idx >= this.fileEntries.length) return;
    if (this.thumbCache.has(idx) || this.thumbLoadingSet.has(idx)) return;
    if (this.thumbnailQueue.includes(idx)) return;

    this.thumbnailQueue.push(idx);
    this.processQueue();
  }

  private async processQueue() {
    if (this.activeDecodes >= FilmstripController.MAX_CONCURRENT_THUMBNAILS) return;
    if (this.thumbnailQueue.length === 0) return;

    const idx = this.thumbnailQueue.shift();
    if (idx === undefined) return;

    this.activeDecodes++;
    this.thumbLoadingSet.add(idx);

    try {
      await this.loadThumbnailForCard(idx);
    } finally {
      this.thumbLoadingSet.delete(idx);
      this.activeDecodes--;
      this.processQueue();
    }
  }

  public async loadThumbnailForCard(idx: number) {
    if (idx < 0 || idx >= this.fileEntries.length) return;
    const cardEl = document.getElementById(`thumb-card-${idx}`);
    if (!cardEl || cardEl.querySelector('img')) return;

    try {
      let previewUrl = this.thumbCache.get(idx);
      let cardRot = 0;
      let cardFlipH = false;
      let cardFlipV = false;

      const entry = this.fileEntries[idx];
      const cacheKey = `${entry.name}_${entry.file.size}_${entry.file.lastModified}`;

      if (!previewUrl) {
        const cached = await ThumbnailCacheStore.get(cacheKey);
        if (cached) {
          if (cached instanceof Blob) {
            previewUrl = URL.createObjectURL(cached);
          } else if ((cached as any).blob) {
            previewUrl = URL.createObjectURL((cached as any).blob);
            cardRot = (cached as any).rotation ?? 0;
            cardFlipH = (cached as any).flipH ?? false;
            cardFlipV = (cached as any).flipV ?? false;
          }
          if (previewUrl) {
            this.thumbCache.set(idx, previewUrl);
          }
        }
      }

      if (!previewUrl) {
        const arrayBuffer = await entry.file.arrayBuffer();
        const bytes = new Uint8Array(arrayBuffer);
        const { imageSlice, preview } = this.loaderService.decodeImageBytes(bytes);
        const previewBlob = new Blob([new Uint8Array(imageSlice)], { type: 'image/jpeg' });
        previewUrl = URL.createObjectURL(previewBlob);
        this.thumbCache.set(idx, previewUrl);

        const orientation = preview.metadata?.orientation ?? 1;
        const exifOri = decodeExifOrientation(orientation);
        cardRot = exifOri.rotation;
        cardFlipH = exifOri.flipH;
        cardFlipV = exifOri.flipV;

        const sidecarXml = await FolderIntake.readSidecarFromDisk(entry);
        if (sidecarXml) {
          try {
            const xmp = this.bridge.parseXmp(sidecarXml);
            if (xmp.edits?.rotation !== undefined && xmp.edits?.rotation !== null) {
              cardRot = xmp.edits.rotation;
              cardFlipH = xmp.edits.flip_horizontal ?? false;
              cardFlipV = xmp.edits.flip_vertical ?? false;
            }
          } catch (_) {}
        }

        await ThumbnailCacheStore.set(cacheKey, {
          blob: previewBlob,
          rotation: cardRot,
          flipH: cardFlipH,
          flipV: cardFlipV,
        });
      } else if (!this.cardRotations.has(idx)) {
        try {
          const orientation = await readExifOrientationFromBlob(entry.file);
          const exifOri = decodeExifOrientation(orientation);
          cardRot = exifOri.rotation;
          cardFlipH = exifOri.flipH;
          cardFlipV = exifOri.flipV;
        } catch (_) {}

        const sidecarXml = await FolderIntake.readSidecarFromDisk(entry);
        if (sidecarXml) {
          try {
            const xmp = this.bridge.parseXmp(sidecarXml);
            if (xmp.edits?.rotation !== undefined && xmp.edits?.rotation !== null) {
              cardRot = xmp.edits.rotation;
              cardFlipH = xmp.edits.flip_horizontal ?? false;
              cardFlipV = xmp.edits.flip_vertical ?? false;
            }
          } catch (_) {}
        }
      }

      this.setThumbnailImage(idx, previewUrl);

      const knownRot = this.cardRotations.get(idx);
      if (knownRot) {
        this.updateThumbnailCardOrientation(idx, knownRot.rotation, knownRot.flipH, knownRot.flipV);
      } else {
        this.updateThumbnailCardOrientation(idx, cardRot, cardFlipH, cardFlipV);
      }
    } catch (err) {
      console.warn(`[Filmstrip] Could not extract preview thumbnail for ${this.fileEntries[idx].name}:`, err);
    }
  }

  public dispose(): void {
    if (this.observer) {
      this.observer.disconnect();
      this.observer = null;
    }
    for (const [, oldUrl] of this.thumbCache) {
      URL.revokeObjectURL(oldUrl);
    }
    this.thumbCache.clear();
  }
}
