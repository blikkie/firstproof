import type { ExportExecutionOptions } from '../ExportModal';
import { FolderIntake, type PhotoFileEntry } from '../FolderIntake';
import { IntermediateTextureCache } from '../IntermediateTextureCache';
import {
  areCurvesActive,
  getDefaultEdits,
  hasModifications,
} from '../utils/edits';
import { yieldToMain } from '../utils/scheduler';
import { encodeTiff } from '../utils/tiffEncoder';
import type {
  EditSettingsPayload,
  PreviewResult,
  WasmBridge,
  XmpPayload,
} from '../WasmBridge';
import { WebGLRenderer } from '../WebGLRenderer';

export interface DecodedImagePayload {
  preview: PreviewResult;
  imageSlice: Uint8Array;
  linearData?: Uint16Array;
  linearW?: number;
  linearH?: number;
}

export interface DevelopedFramePayload {
  preview: PreviewResult;
  imageSlice: Uint8Array;
  imgBitmap: ImageBitmap | null;
  linearData: Uint16Array | null;
  linearW: number;
  linearH: number;
  isPrewarmedHit: boolean;
}

export interface PhotoRecord {
  edits: EditSettingsPayload;
  rating: number;
  status: string;
  title?: string;
  description?: string;
  creator?: string;
  keywords?: string[];
  hasSidecarOnDisk: boolean;
}

export interface PrewarmedPhoto {
  preview: PreviewResult;
  imageSlice: Uint8Array;
  imgBitmap: ImageBitmap;
  linearData?: Uint16Array;
  linearW?: number;
  linearH?: number;
}

export interface ExifOrientationInfo {
  rotation: number;
  flipH: boolean;
  flipV: boolean;
}

export class PhotoLoaderService {
  private bridge: WasmBridge;
  private rawWorker: Worker | null = null;
  private workerMsgId: number = 0;
  private workerPendingMap: Map<number, (res: any) => void> = new Map();

  private prewarmedCache: Map<number, PrewarmedPhoto> = new Map();
  private prewarmLoadingSet: Set<number> = new Set();

  private photoEditsCache: Map<string, EditSettingsPayload> = new Map();
  private photoExifCache: Map<string, ExifOrientationInfo> = new Map();
  private photoRecords: Map<string, PhotoRecord> = new Map();

  private exportCanvas: HTMLCanvasElement | null = null;
  private exportRenderer: WebGLRenderer | null = null;

  constructor(bridge: WasmBridge) {
    this.bridge = bridge;
    this.initWorker();
  }

  private initWorker() {
    try {
      this.rawWorker = new Worker(
        new URL('../RawDecodeWorker.ts', import.meta.url),
        { type: 'module' },
      );
      this.rawWorker.onmessage = (e: MessageEvent) => {
        const { id } = e.data;
        const resolve = this.workerPendingMap.get(id);
        if (resolve) {
          this.workerPendingMap.delete(id);
          resolve(e.data);
        }
      };
      console.log(
        '⚡ [PhotoLoaderService] Off-main-thread RawDecodeWorker active',
      );
    } catch (err) {
      console.warn(
        '[PhotoLoaderService] WebWorker pool initialization fallback:',
        err,
      );
    }
  }

  public async decodeInWorker(
    arrayBuffer: ArrayBuffer,
    fileName: string,
    fullResolution: boolean = false,
  ): Promise<DecodedImagePayload> {
    if (!this.rawWorker) {
      const bytes = new Uint8Array(arrayBuffer);
      const res = this.bridge.decodeImageBytes(bytes);
      const ext = fileName.split('.').pop()?.toLowerCase() || '';
      const isRawFile = [
        'cr2',
        'cr3',
        'nef',
        'arw',
        'dng',
        'rw2',
        'orf',
        'pef',
        'raf',
      ].includes(ext);
      let linearData: Uint16Array | undefined;
      let linearW: number | undefined;
      let linearH: number | undefined;
      if (isRawFile) {
        try {
          const rawLin = fullResolution
            ? this.bridge.decodeRawFullLinear(bytes)
            : this.bridge.decodeRawLinear(bytes);
          linearData = rawLin.linearData;
          linearW = rawLin.res.width;
          linearH = rawLin.res.height;
        } catch (e) {
          console.warn(
            '[PhotoLoaderService] Fallback decodeRawLinear failed:',
            e,
          );
        }
      }
      return {
        preview: res.preview,
        imageSlice: res.imageSlice,
        linearData,
        linearW,
        linearH,
      };
    }

    const id = ++this.workerMsgId;
    return new Promise((resolve, reject) => {
      this.workerPendingMap.set(id, (res) => {
        if (res.success && res.preview && res.imageSlice) {
          resolve({
            preview: res.preview,
            imageSlice: res.imageSlice,
            linearData: res.linearData,
            linearW: res.linearW,
            linearH: res.linearH,
          });
        } else {
          try {
            const bytes = new Uint8Array(arrayBuffer);
            resolve(this.bridge.decodeImageBytes(bytes));
          } catch (e) {
            reject(new Error(res.error || 'Worker decode failed'));
          }
        }
      });
      const workerBuffer = arrayBuffer.slice(0);
      this.rawWorker!.postMessage(
        { id, arrayBuffer: workerBuffer, fileName, fullResolution },
        [workerBuffer],
      );
    });
  }

  public async exportInWorker(
    arrayBuffer: ArrayBuffer,
    fileName: string,
    edits: EditSettingsPayload,
    format: 'JPEG' | 'TIFF',
    quality: number,
    sizeOption: 'Original' | '2048' | '3840' | 'Custom',
    customDimension: number,
    demosaicQuality: 'Standard' | 'Archival',
  ): Promise<Uint8Array> {
    if (!this.rawWorker) {
      const rawBytes = new Uint8Array(arrayBuffer);
      return this.bridge.exportRaw(
        rawBytes,
        edits,
        format,
        quality,
        sizeOption,
        customDimension,
        demosaicQuality,
      );
    }

    const id = ++this.workerMsgId;
    return new Promise((resolve, reject) => {
      this.workerPendingMap.set(id, (res) => {
        if (res.success && res.outputBytes) {
          resolve(res.outputBytes);
        } else {
          reject(new Error(res.error || 'Worker export failed'));
        }
      });
      const workerBuffer = arrayBuffer.slice(0);
      this.rawWorker!.postMessage(
        {
          type: 'export',
          id,
          arrayBuffer: workerBuffer,
          fileName,
          editsJson: JSON.stringify(edits),
          format,
          quality,
          sizeOption,
          customDimension,
          demosaicQuality: demosaicQuality === 'Archival' ? 1 : 0,
        },
        [workerBuffer],
      );
    });
  }

  public decodeImageBytes(bytes: Uint8Array): {
    preview: PreviewResult;
    imageSlice: Uint8Array;
  } {
    return this.bridge.decodeImageBytes(bytes);
  }

  public async prewarmPhoto(idx: number, entry: PhotoFileEntry): Promise<void> {
    if (this.prewarmedCache.has(idx) || this.prewarmLoadingSet.has(idx)) return;
    this.prewarmLoadingSet.add(idx);
    try {
      const arrayBuffer = await entry.file.arrayBuffer();
      const res = await this.decodeInWorker(arrayBuffer, entry.name);
      const blob = new Blob([new Uint8Array(res.imageSlice)], {
        type: 'image/jpeg',
      });
      let imgBitmap: ImageBitmap;
      try {
        imgBitmap = await createImageBitmap(blob, { resizeWidth: 2560 });
      } catch (_) {
        imgBitmap = await createImageBitmap(blob);
      }

      if (this.prewarmedCache.size >= 4) {
        const firstKey = this.prewarmedCache.keys().next().value;
        if (firstKey !== undefined) {
          const old = this.prewarmedCache.get(firstKey);
          old?.imgBitmap.close();
          this.prewarmedCache.delete(firstKey);
        }
      }

      this.prewarmedCache.set(idx, {
        preview: res.preview,
        imageSlice: res.imageSlice,
        imgBitmap,
        linearData: res.linearData,
        linearW: res.linearW,
        linearH: res.linearH,
      });
    } catch (_) {
      // Ignore background pre-warming errors
    } finally {
      this.prewarmLoadingSet.delete(idx);
    }
  }

  public getPrewarmed(idx: number): PrewarmedPhoto | undefined {
    return this.prewarmedCache.get(idx);
  }

  public hasPrewarmed(idx: number): boolean {
    return this.prewarmedCache.has(idx);
  }

  public clearPrewarmed(): void {
    for (const [, item] of this.prewarmedCache) {
      try {
        item.imgBitmap.close();
      } catch (_) {}
    }
    this.prewarmedCache.clear();
    this.prewarmLoadingSet.clear();
  }

  public getCachedEdits(fileName: string): EditSettingsPayload | undefined {
    return this.photoEditsCache.get(fileName);
  }

  public setCachedEdits(fileName: string, edits: EditSettingsPayload): void {
    this.photoEditsCache.set(fileName, { ...edits });
  }

  public getCachedExif(fileName: string): ExifOrientationInfo | undefined {
    return this.photoExifCache.get(fileName);
  }

  public setCachedExif(fileName: string, exif: ExifOrientationInfo): void {
    this.photoExifCache.set(fileName, { ...exif });
  }

  public getPhotoRecord(fileName: string): PhotoRecord | undefined {
    return this.photoRecords.get(fileName);
  }

  public setPhotoRecord(fileName: string, record: PhotoRecord): void {
    this.photoRecords.set(fileName, { ...record });
    this.photoEditsCache.set(fileName, { ...record.edits });
  }

  public async loadPhotoRecord(
    entry: PhotoFileEntry,
    isRaw: boolean,
    defaultRotation: number = 0,
    defaultFlipH: boolean = false,
    defaultFlipV: boolean = false,
  ): Promise<PhotoRecord> {
    const existing = this.photoRecords.get(entry.name);
    if (existing) {
      return existing;
    }

    const sidecar = await this.loadSidecar(entry);
    const xmp = sidecar.xmp;

    let edits: EditSettingsPayload;
    if (xmp?.edits) {
      edits = { ...getDefaultEdits(isRaw), ...xmp.edits };
    } else {
      edits = getDefaultEdits(isRaw);
    }

    if (edits.rotation === undefined || edits.rotation === null) {
      edits.rotation = defaultRotation;
      edits.flip_horizontal = defaultFlipH;
      edits.flip_vertical = defaultFlipV;
    }

    const record: PhotoRecord = {
      edits,
      rating: xmp?.rating || 0,
      status: xmp?.status || 'Unflagged',
      title: xmp?.title || '',
      description: xmp?.description || '',
      creator: xmp?.creator || '',
      keywords: xmp?.keywords || [],
      hasSidecarOnDisk: sidecar.rawXml !== null,
    };

    this.photoRecords.set(entry.name, record);
    this.photoEditsCache.set(entry.name, record.edits);
    return record;
  }

  public getIntermediate(fileName: string, edits: EditSettingsPayload) {
    return IntermediateTextureCache.get(fileName, edits);
  }

  public async loadPhotoFrame(
    entry: PhotoFileEntry,
    isRawFile: boolean,
    idx: number,
  ): Promise<DevelopedFramePayload> {
    const prewarmed = this.prewarmedCache.get(idx);
    if (prewarmed) {
      return {
        preview: prewarmed.preview,
        imageSlice: prewarmed.imageSlice,
        imgBitmap: prewarmed.imgBitmap,
        linearData: prewarmed.linearData || null,
        linearW: prewarmed.linearW || 0,
        linearH: prewarmed.linearH || 0,
        isPrewarmedHit: true,
      };
    }

    const arrayBuffer = await entry.file.arrayBuffer();
    const res = await this.decodeInWorker(arrayBuffer, entry.name, false);

    let imgBitmap: ImageBitmap | null = null;
    let linear16Data: Uint16Array | null = res.linearData || null;
    let linearW = res.linearW || 0;
    let linearH = res.linearH || 0;

    if (!isRawFile || !linear16Data) {
      const blob = new Blob([new Uint8Array(res.imageSlice)], {
        type: 'image/jpeg',
      });
      try {
        imgBitmap = await createImageBitmap(blob, { resizeQuality: 'high' });
      } catch (decodeErr) {
        console.warn(
          `[PhotoLoaderService] createImageBitmap failed for ${entry.name}, creating fallback canvas:`,
          decodeErr,
        );
        const fallbackCanvas = document.createElement('canvas');
        fallbackCanvas.width =
          res.preview.width && res.preview.width > 0 ? res.preview.width : 1920;
        fallbackCanvas.height =
          res.preview.height && res.preview.height > 0
            ? res.preview.height
            : 1080;
        const ctx = fallbackCanvas.getContext('2d');
        if (ctx) {
          ctx.fillStyle = '#111318';
          ctx.fillRect(0, 0, fallbackCanvas.width, fallbackCanvas.height);
          ctx.fillStyle = '#f1f5f9';
          ctx.font = 'bold 24px sans-serif';
          ctx.fillText(entry.name, 40, 60);
        }
        imgBitmap = await createImageBitmap(fallbackCanvas);
      }
    }

    if (isRawFile && (!linear16Data || linearW === 0 || linearH === 0)) {
      try {
        const rawBytes = new Uint8Array(await entry.file.arrayBuffer());
        const rawLin = this.bridge.decodeRawLinear(rawBytes);
        linear16Data = rawLin.linearData;
        linearW = rawLin.res.width;
        linearH = rawLin.res.height;
      } catch (linErr) {
        console.error(
          `[PhotoLoaderService] Linear Rec.2020 RAW decode failed for ${entry.name}:`,
          linErr,
        );
      }
    }

    return {
      preview: res.preview,
      imageSlice: res.imageSlice,
      imgBitmap,
      linearData: linear16Data,
      linearW,
      linearH,
      isPrewarmedHit: false,
    };
  }

  public scheduleIntermediateSnapshot(
    fileName: string,
    edits: EditSettingsPayload,
    renderer: WebGLRenderer,
    allowed: boolean,
  ): void {
    if (!allowed) return;
    renderer
      .captureIntermediateSnapshot(edits)
      .then((bitmap) => {
        if (bitmap) {
          IntermediateTextureCache.set(fileName, edits, bitmap);
        }
      })
      .catch((err) => {
        console.warn(
          '[PhotoLoaderService] Failed to capture intermediate bitmap:',
          err,
        );
      });
  }

  public async loadSidecar(
    entry: PhotoFileEntry,
  ): Promise<{ xmp: any | null; rawXml: string | null }> {
    let sidecarXml = await FolderIntake.readSidecarFromDisk(entry);
    if (!sidecarXml) {
      try {
        sidecarXml = localStorage.getItem(`fp_xmp_${entry.name}`);
      } catch (_) {}
    }

    if (!sidecarXml) return { xmp: null, rawXml: null };

    try {
      const xmp = this.bridge.parseXmp(sidecarXml);
      return { xmp, rawXml: sidecarXml };
    } catch (_) {
      return { xmp: null, rawXml: sidecarXml };
    }
  }

  public async saveSidecar(
    entry: PhotoFileEntry,
    edits: EditSettingsPayload,
    metadata: {
      rating: number;
      status: string;
      title?: string;
      description?: string;
      creator?: string;
      keywords?: string[];
    },
  ): Promise<void> {
    this.setCachedEdits(entry.name, edits);

    const xmpPayload: XmpPayload = {
      rating: metadata.rating > 0 ? metadata.rating : undefined,
      status: metadata.status,
      title: metadata.title || undefined,
      description: metadata.description || undefined,
      creator: metadata.creator || undefined,
      keywords:
        metadata.keywords && metadata.keywords.length > 0
          ? metadata.keywords
          : undefined,
      edits,
    };

    const xml = this.bridge.generateXmp(xmpPayload);

    try {
      localStorage.setItem(`fp_xmp_${entry.name}`, xml);
    } catch (_) {}

    await FolderIntake.writeSidecarToDisk(entry, xml);
  }

  public async autoSaveSidecar(
    entry: PhotoFileEntry,
    record: PhotoRecord,
    orientation: number = 1,
  ): Promise<boolean> {
    this.setPhotoRecord(entry.name, record);

    const modified = hasModifications(
      record.edits,
      orientation,
      record.rating,
      record.status,
      {
        title: record.title,
        description: record.description,
        creator: record.creator,
        keywords: record.keywords?.join(', '),
      },
    );

    if (modified || record.hasSidecarOnDisk) {
      const xmpPayload: XmpPayload = {
        rating: record.rating > 0 ? record.rating : undefined,
        status: record.status,
        title: record.title || undefined,
        description: record.description || undefined,
        creator: record.creator || undefined,
        keywords:
          record.keywords && record.keywords.length > 0
            ? record.keywords
            : undefined,
        edits: record.edits,
      };

      const xml = this.bridge.generateXmp(xmpPayload);

      try {
        localStorage.setItem(`fp_xmp_${entry.name}`, xml);
      } catch (_) {}

      await FolderIntake.writeSidecarToDisk(entry, xml);
      record.hasSidecarOnDisk = true;
      return true;
    }

    return false;
  }

  public getExportRenderer(): {
    canvas: HTMLCanvasElement;
    renderer: WebGLRenderer;
  } {
    if (!this.exportCanvas || !this.exportRenderer) {
      this.exportCanvas = document.createElement('canvas');
      this.exportCanvas.style.display = 'none';
      document.body.appendChild(this.exportCanvas);
      this.exportRenderer = new WebGLRenderer(this.exportCanvas);
    }
    return { canvas: this.exportCanvas, renderer: this.exportRenderer };
  }

  public async handleExport(
    photos: PhotoFileEntry[],
    activeEntryName: string | null,
    activeEdits: EditSettingsPayload | null,
    getDefaultEdits: (isRaw: boolean) => EditSettingsPayload,
    options: ExportExecutionOptions,
    onProgress: (status: string, current: number, total: number) => void,
  ): Promise<{ success: boolean; message: string }> {
    if (photos.length === 0) {
      return { success: false, message: 'No photos selected for export.' };
    }

    const { canvas: expCanvas, renderer: expRenderer } =
      this.getExportRenderer();
    let successCount = 0;
    let lastOutputPath = '';

    for (let i = 0; i < photos.length; i++) {
      const entry = photos[i];
      const prefix = photos.length > 1 ? `[${i + 1}/${photos.length}] ` : '';

      onProgress(`${prefix}Preparing ${entry.name}...`, i + 1, photos.length);
      await yieldToMain();

      const fileExt = entry.name.split('.').pop()?.toLowerCase() || '';
      const isRawFile = [
        'cr2',
        'cr3',
        'nef',
        'arw',
        'dng',
        'rw2',
        'orf',
        'pef',
        'raf',
      ].includes(fileExt);

      let photoEdits: EditSettingsPayload;
      if (entry.name === activeEntryName && activeEdits) {
        photoEdits = JSON.parse(JSON.stringify(activeEdits));
      } else {
        const sidecar = await this.loadSidecar(entry);
        if (sidecar.xmp?.edits) {
          photoEdits = { ...getDefaultEdits(isRawFile), ...sidecar.xmp.edits };
        } else {
          photoEdits =
            this.getCachedEdits(entry.name) ?? getDefaultEdits(isRawFile);
        }
      }

      const stem =
        entry.name.substring(0, entry.name.lastIndexOf('.')) || entry.name;
      const ext = options.format === 'TIFF' ? 'tiff' : 'jpg';
      const filename = `${stem}_FirstProof.${ext}`;
      lastOutputPath = filename;

      let fileBlob: Blob;

      if (isRawFile) {
        onProgress(
          `${prefix}Exporting ${entry.name} (${options.format})...`,
          i + 1,
          photos.length,
        );
        await yieldToMain();

        const arrayBuffer = await entry.file.arrayBuffer();
        const outputBytes = await this.exportInWorker(
          arrayBuffer,
          entry.name,
          photoEdits,
          options.format,
          options.quality,
          options.sizeOption,
          options.customDimension,
          options.demosaicQuality ?? 'Standard',
        );

        fileBlob = new Blob([outputBytes.buffer as ArrayBuffer], {
          type: options.format === 'TIFF' ? 'image/tiff' : 'image/jpeg',
        });
      } else {
        onProgress(`${prefix}Decoding ${entry.name}...`, i + 1, photos.length);
        await yieldToMain();

        const arrayBuffer = await entry.file.arrayBuffer();
        const res = await this.decodeInWorker(arrayBuffer, entry.name, false);
        const blob = new Blob([new Uint8Array(res.imageSlice)], {
          type: 'image/jpeg',
        });
        const imgBitmap = await createImageBitmap(blob);
        expRenderer.uploadImage(imgBitmap);
        expRenderer.setBaselineSharpness(
          res.preview?.metadata?.baseline_sharpness ?? 1.0,
        );
        let texW = imgBitmap.width;
        let texH = imgBitmap.height;

        if (texW <= 0) texW = 1920;
        if (texH <= 0) texH = 1080;

        if (photoEdits.curves && areCurvesActive(photoEdits.curves)) {
          try {
            const lut = this.bridge.evaluateCurves(photoEdits.curves);
            expRenderer.updateToneCurvesLut(lut);
          } catch (err) {
            console.warn(
              '[PhotoLoaderService] Could not evaluate curve LUT:',
              err,
            );
          }
        }

        const rot = (photoEdits.rotation ?? 0) % 360;
        const isOrthogonal = rot === 90 || rot === 270;
        const baseW = isOrthogonal ? texH : texW;
        const baseH = isOrthogonal ? texW : texH;

        const cropL = photoEdits.crop_left ?? 0.0;
        const cropT = photoEdits.crop_top ?? 0.0;
        const cropR = photoEdits.crop_right ?? 1.0;
        const cropB = photoEdits.crop_bottom ?? 1.0;
        const effectiveW = Math.max(
          1,
          Math.round(baseW * Math.max(0.01, cropR - cropL)),
        );
        const effectiveH = Math.max(
          1,
          Math.round(baseH * Math.max(0.01, cropB - cropT)),
        );

        let targetW = effectiveW;
        let targetH = effectiveH;
        let maxDim: number | null = null;
        if (options.sizeOption === '2048') maxDim = 2048;
        else if (options.sizeOption === '3840') maxDim = 3840;
        else if (options.sizeOption === 'Custom')
          maxDim = options.customDimension;

        if (maxDim && (effectiveW > maxDim || effectiveH > maxDim)) {
          if (effectiveW >= effectiveH) {
            targetW = maxDim;
            targetH = Math.max(
              1,
              Math.round((effectiveH * maxDim) / effectiveW),
            );
          } else {
            targetH = maxDim;
            targetW = Math.max(
              1,
              Math.round((effectiveW * maxDim) / effectiveH),
            );
          }
        }

        const maxGl =
          expRenderer.glContext.getParameter(
            expRenderer.glContext.MAX_VIEWPORT_DIMS,
          )?.[0] || 8192;
        if (targetW > maxGl || targetH > maxGl) {
          const scale = maxGl / Math.max(targetW, targetH);
          targetW = Math.max(1, Math.round(targetW * scale));
          targetH = Math.max(1, Math.round(targetH * scale));
        }

        onProgress(
          `${prefix}Rendering WebGL canvas (${targetW}×${targetH})...`,
          i + 1,
          photos.length,
        );
        await yieldToMain();

        expCanvas.width = targetW;
        expCanvas.height = targetH;
        expRenderer.render(photoEdits, false, 1.0, { x: 0, y: 0 }, false);

        onProgress(
          `${prefix}Encoding ${options.format}...`,
          i + 1,
          photos.length,
        );
        await yieldToMain();

        if (options.format === 'TIFF') {
          const pixels = expRenderer.readRgbaPixels(targetW, targetH);
          const tiffBytes = encodeTiff(targetW, targetH, pixels, 16);
          fileBlob = new Blob([tiffBytes.buffer as ArrayBuffer], {
            type: 'image/tiff',
          });
        } else {
          fileBlob = await new Promise<Blob>((resolve, reject) => {
            expCanvas.toBlob(
              (b) =>
                b ? resolve(b) : reject(new Error('Canvas toBlob failed')),
              'image/jpeg',
              options.quality / 100,
            );
          });
        }
        await yieldToMain();
      }

      onProgress(`${prefix}Saving ${filename}...`, i + 1, photos.length);
      await yieldToMain();

      if (options.destinationDirHandle) {
        try {
          const fileHandle = await options.destinationDirHandle.getFileHandle(
            filename,
            { create: true },
          );
          const writable = await fileHandle.createWritable();
          await writable.write(fileBlob);
          await writable.close();
        } catch (writeErr) {
          console.warn(
            '[PhotoLoaderService] File system write failed, falling back to download:',
            writeErr,
          );
          this.triggerDownload(fileBlob, filename);
        }
      } else {
        this.triggerDownload(fileBlob, filename);
      }

      await yieldToMain();
      successCount++;
    }

    const isBatch = photos.length > 1;
    const message = isBatch
      ? `Finished exporting ${successCount} of ${photos.length} photos`
      : `Successfully exported to ${lastOutputPath}`;

    return { success: successCount > 0, message };
  }

  public triggerDownload(blob: Blob, filename: string): void {
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1500);
  }
  public dispose(): void {
    if (this.rawWorker) {
      this.rawWorker.terminate();
      this.rawWorker = null;
    }
    this.clearPrewarmed();
  }
}
