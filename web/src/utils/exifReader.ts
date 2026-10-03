/**
 * Fast asynchronous EXIF orientation reader.
 * Reads the first 64KB slice of a file to extract the TIFF IFD0 or JPEG APP1
 * orientation tag (1-8) via shared WASM `extract_metadata_from_bytes`
 * without loading or decoding the entire RAW file.
 */

import type { ImageMetadataRecord } from 'firstproof-wasm';
import initWasm, {
  decode_exif_orientation,
  extract_metadata_from_bytes,
} from 'firstproof-wasm';

export async function readExifOrientationFromBlob(blob: Blob): Promise<number> {
  try {
    const slice = blob.slice(0, 65536);
    const buffer = await slice.arrayBuffer();
    const bytes = new Uint8Array(buffer);

    await initWasm();
    const meta: ImageMetadataRecord = extract_metadata_from_bytes(bytes);
    if (
      meta &&
      typeof meta.orientation === 'number' &&
      meta.orientation >= 1 &&
      meta.orientation <= 8
    ) {
      return meta.orientation;
    }
  } catch (_) {
    // Graceful fallback to unrotated default
  }

  return 1;
}

export function decodeExifOrientation(orientation?: number): {
  rotation: number;
  flipH: boolean;
  flipV: boolean;
  flip_horizontal: boolean;
  flip_vertical: boolean;
} {
  try {
    if (orientation) {
      const rec = decode_exif_orientation(orientation);
      if (rec) {
        return {
          rotation: rec.rotation,
          flipH: rec.flip_horizontal,
          flipV: rec.flip_vertical,
          flip_horizontal: rec.flip_horizontal,
          flip_vertical: rec.flip_vertical,
        };
      }
    }
  } catch (_) {
    // Fall back to pure JS EXIF mapping if WASM runtime is not yet initialized
  }

  // Standard EXIF 1..8 orientation tag mapping fallback
  let rotation = 0;
  let flipH = false;
  let flipV = false;
  switch (orientation) {
    case 2:
      flipH = true;
      break;
    case 3:
      rotation = 180;
      break;
    case 4:
      flipV = true;
      break;
    case 5:
      rotation = 270;
      flipH = true;
      break;
    case 6:
      rotation = 90;
      break;
    case 7:
      rotation = 90;
      flipH = true;
      break;
    case 8:
      rotation = 270;
      break;
    default:
      break;
  }

  return {
    rotation,
    flipH,
    flipV,
    flip_horizontal: flipH,
    flip_vertical: flipV,
  };
}
