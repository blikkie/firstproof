/**
 * Fast asynchronous EXIF orientation reader.
 * Reads the first 64KB slice of a file to extract the TIFF IFD0 or JPEG APP1
 * orientation tag (1-8) via shared WASM `extract_metadata_from_bytes`
 * without loading or decoding the entire RAW file.
 */
import initWasm, { extract_metadata_from_bytes } from 'firstproof-wasm';
import type { ImageMetadataRecord } from 'firstproof-wasm';

export async function readExifOrientationFromBlob(blob: Blob): Promise<number> {
  try {
    const slice = blob.slice(0, 65536);
    const buffer = await slice.arrayBuffer();
    const bytes = new Uint8Array(buffer);

    await initWasm();
    const meta: ImageMetadataRecord = extract_metadata_from_bytes(bytes);
    if (meta && typeof meta.orientation === 'number' && meta.orientation >= 1 && meta.orientation <= 8) {
      return meta.orientation;
    }
  } catch (_) {
    // Graceful fallback to unrotated default
  }

  return 1;
}
