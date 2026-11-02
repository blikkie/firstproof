import type { PhotoFileEntry } from '../FolderIntake';

export interface SamplePhotoInfo {
  filename: string;
  xmpFilename: string;
  title: string;
  description: string;
}

export const SAMPLE_PHOTOS: readonly SamplePhotoInfo[] = [
  {
    filename: '_MG_1304.dng',
    xmpFilename: '_MG_1304.xmp',
    title: 'Night Street Scene',
    description: 'Low-Key Tungsten (2900K), Specular Halation bloom challenge',
  },
  {
    filename: '_MG_1422.dng',
    xmpFilename: '_MG_1422.xmp',
    title: 'Textured Documentary',
    description: 'Architectural stone texture & high micro-contrast',
  },
  {
    filename: 'IMG_0730.dng',
    xmpFilename: 'IMG_0730.xmp',
    title: 'HDR Golden Sunset',
    description: '12.7 stops dynamic range, AgX filmic toe & shoulder response',
  },
  {
    filename: '20250409_0017.dng',
    xmpFilename: '20250409_0017.xmp',
    title: 'Saturated Primaries',
    description: 'Wide-gamut saturated reds & deep foliage greens',
  },
] as const;

export type SampleProgressCallback = (
  current: number,
  total: number,
  name: string,
  percent?: number,
) => void;

/**
 * Downloads the 4 curated lab proof sample RAW files and seeds
 * their corresponding XMP sidecars into browser session/local storage.
 */
export async function loadSampleShoot(
  onProgress?: SampleProgressCallback,
): Promise<PhotoFileEntry[]> {
  const entries: PhotoFileEntry[] = [];
  const total = SAMPLE_PHOTOS.length;

  for (let i = 0; i < total; i++) {
    const sample = SAMPLE_PHOTOS[i];
    onProgress?.(i + 1, total, sample.filename, 0);

    // 1. Fetch XMP sidecar and seed into localStorage if not already present
    try {
      const xmpRes = await fetch(`/samples/${sample.xmpFilename}`);
      if (xmpRes.ok) {
        const xmpText = await xmpRes.text();
        const storageKey = `fp_xmp_${sample.filename}`;
        // Preserve user modifications if already set; seed defaults on initial load
        if (!localStorage.getItem(storageKey)) {
          localStorage.setItem(storageKey, xmpText);
        }
      }
    } catch (err) {
      console.warn(
        `[SamplePhotos] Failed to load XMP for ${sample.filename}:`,
        err,
      );
    }

    // 2. Fetch DNG RAW frame
    const res = await fetch(`/samples/${sample.filename}`);
    if (!res.ok) {
      throw new Error(
        `Failed to download sample ${sample.filename}: HTTP ${res.status}`,
      );
    }

    let blob: Blob;
    const contentLength = res.headers.get('content-length');
    const totalBytes = contentLength ? parseInt(contentLength, 10) : 0;

    if (res.body && totalBytes > 0) {
      const reader = res.body.getReader();
      const chunks: Uint8Array[] = [];
      let receivedBytes = 0;

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        if (value) {
          chunks.push(value);
          receivedBytes += value.length;
          const pct = Math.round((receivedBytes / totalBytes) * 100);
          onProgress?.(i + 1, total, sample.filename, pct);
        }
      }

      blob = new Blob(chunks as BlobPart[], { type: 'image/x-adobe-dng' });
    } else {
      blob = await res.blob();
      onProgress?.(i + 1, total, sample.filename, 100);
    }

    const file = new File([blob], sample.filename, {
      type: 'image/x-adobe-dng',
    });
    entries.push({
      name: sample.filename,
      file,
    });
  }

  return entries;
}

/**
 * Resets a sample photo back to its original proof run settings.
 */
export async function resetSamplePhotoToShootDefaults(
  filename: string,
): Promise<boolean> {
  const sample = SAMPLE_PHOTOS.find((s) => s.filename === filename);
  if (!sample) return false;

  try {
    const xmpRes = await fetch(`/samples/${sample.xmpFilename}`);
    if (xmpRes.ok) {
      const xmpText = await xmpRes.text();
      localStorage.setItem(`fp_xmp_${sample.filename}`, xmpText);
      return true;
    }
  } catch (err) {
    console.warn(`[SamplePhotos] Failed to reset XMP for ${filename}:`, err);
  }
  return false;
}
