/**
 * Fast asynchronous EXIF orientation reader.
 * Reads only the first 64KB slice of a file to extract the TIFF IFD0 or JPEG APP1
 * orientation tag (1-8) in under 0.1ms without loading or decoding the entire RAW file.
 */

export async function readExifOrientationFromBlob(blob: Blob): Promise<number> {
  try {
    const slice = blob.slice(0, 65536);
    const buffer = await slice.arrayBuffer();
    const view = new DataView(buffer);

    if (view.byteLength < 8) return 1;

    // 1. TIFF-based RAW containers (CR2, NEF, ARW, DNG, TIFF)
    const isLittleEndian = view.getUint16(0, false) === 0x4949 && view.getUint16(2, true) === 0x002A;
    const isBigEndian = view.getUint16(0, false) === 0x4D4D && view.getUint16(2, false) === 0x002A;

    if (isLittleEndian || isBigEndian) {
      const le = isLittleEndian;
      const ifdOffset = view.getUint32(4, le);
      if (ifdOffset >= 8 && ifdOffset + 2 <= view.byteLength) {
        const numEntries = view.getUint16(ifdOffset, le);
        let entryOffset = ifdOffset + 2;
        for (let i = 0; i < numEntries && entryOffset + 12 <= view.byteLength; i++, entryOffset += 12) {
          const tag = view.getUint16(entryOffset, le);
          if (tag === 0x0112) { // Orientation tag
            const val = view.getUint16(entryOffset + 8, le);
            if (val >= 1 && val <= 8) return val;
          }
        }
      }
    }

    // 2. JPEG containers (starts with 0xFFD8)
    if (view.getUint16(0, false) === 0xFFD8) {
      let offset = 2;
      while (offset + 4 <= view.byteLength) {
        const marker = view.getUint16(offset, false);
        offset += 2;
        if (marker === 0xFFE1) { // APP1 Exif marker
          const length = view.getUint16(offset, false);
          offset += 2;
          if (offset + 6 <= view.byteLength) {
            const exifHeader = view.getUint32(offset, false);
            if (exifHeader === 0x45786966 && view.getUint16(offset + 4, false) === 0x0000) {
              const tiffStart = offset + 6;
              if (tiffStart + 8 <= view.byteLength) {
                const le = view.getUint16(tiffStart, false) === 0x4949;
                const ifdOff = view.getUint32(tiffStart + 4, le);
                const dirStart = tiffStart + ifdOff;
                if (dirStart + 2 <= view.byteLength) {
                  const entries = view.getUint16(dirStart, le);
                  let eOff = dirStart + 2;
                  for (let i = 0; i < entries && eOff + 12 <= view.byteLength; i++, eOff += 12) {
                    const tag = view.getUint16(eOff, le);
                    if (tag === 0x0112) {
                      const val = view.getUint16(eOff + 8, le);
                      if (val >= 1 && val <= 8) return val;
                    }
                  }
                }
              }
            }
          }
          break;
        } else if ((marker & 0xFF00) === 0xFF00 && marker !== 0xFFD8 && marker !== 0xFFD9) {
          const len = view.getUint16(offset, false);
          offset += len;
        } else {
          break;
        }
      }
    }
  } catch (_) {
    // Graceful fallback to unrotated default
  }

  return 1;
}
