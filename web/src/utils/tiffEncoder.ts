/**
 * First Proof — Client-Side TIFF Encoder
 *
 * Implements a standards-compliant uncompressed TIFF 6.0 encoder for authentic
 * 8-bit and 16-bit RGB image export with bit-level parity to desktop libtiff output.
 */

export function encodeTiff(
  width: number,
  height: number,
  data: Uint8Array | Uint16Array,
  bitsPerSample: 8 | 16 = 8,
  flipY: boolean = true,
): Uint8Array {
  const bytesPerChannel = bitsPerSample === 16 ? 2 : 1;
  const bytesPerPixel = 3 * bytesPerChannel; // RGB
  const pixelBytes = width * height * bytesPerPixel;
  const totalBytes = 192 + pixelBytes;
  const buf = new ArrayBuffer(totalBytes);
  const view = new DataView(buf);
  const u8 = new Uint8Array(buf);

  // TIFF Header (8 bytes) — Little Endian 'II'
  view.setUint16(0, 0x4949, true); // 'II'
  view.setUint16(2, 42, true); // Magic number 42
  view.setUint32(4, 8, true); // Offset to first IFD (byte 8)

  // Image File Directory (IFD) at offset 8: 13 directory entries
  view.setUint16(8, 13, true);

  let offset = 10;
  function writeTag(
    tag: number,
    type: number,
    count: number,
    valOrOffset: number,
  ) {
    view.setUint16(offset, tag, true);
    view.setUint16(offset + 2, type, true);
    view.setUint32(offset + 4, count, true);
    view.setUint32(offset + 8, valOrOffset, true);
    offset += 12;
  }

  // Tags must strictly be written in ascending tag ID order per TIFF 6.0 spec:
  writeTag(0x0100, 4, 1, width); // ImageWidth (LONG)
  writeTag(0x0101, 4, 1, height); // ImageLength (LONG)
  writeTag(0x0102, 3, 3, 170); // BitsPerSample (SHORT[3] at offset 170)
  writeTag(0x0103, 3, 1, 1); // Compression (1 = Uncompressed)
  writeTag(0x0106, 3, 1, 2); // PhotometricInterpretation (2 = RGB)
  writeTag(0x0111, 4, 1, 192); // StripOffsets (pixel data starts at 192)
  writeTag(0x0115, 3, 1, 3); // SamplesPerPixel (3 = R, G, B)
  writeTag(0x0116, 4, 1, height); // RowsPerStrip (full frame single strip)
  writeTag(0x0117, 4, 1, pixelBytes); // StripByteCounts
  writeTag(0x011a, 5, 1, 176); // XResolution (RATIONAL at offset 176)
  writeTag(0x011b, 5, 1, 184); // YResolution (RATIONAL at offset 184)
  writeTag(0x011c, 3, 1, 1); // PlanarConfiguration (1 = Chunky)
  writeTag(0x0128, 3, 1, 2); // ResolutionUnit (2 = Inch)

  // Next IFD Offset (0 = none)
  view.setUint32(offset, 0, true);

  // Auxiliary data values:
  // Offset 170: BitsPerSample [bits, bits, bits]
  view.setUint16(170, bitsPerSample, true);
  view.setUint16(172, bitsPerSample, true);
  view.setUint16(174, bitsPerSample, true);

  // Offset 176: XResolution 72/1
  view.setUint32(176, 72, true);
  view.setUint32(180, 1, true);

  // Offset 184: YResolution 72/1
  view.setUint32(184, 72, true);
  view.setUint32(188, 1, true);

  // Pixel Raster data starting at offset 192
  // Note: WebGL gl.readPixels returns bottom-to-top raster (requires flipY = true).
  // TIFF expects top-to-bottom raster (row 0 is top).
  if (bitsPerSample === 16) {
    const u16Dst = new Uint16Array(buf, 192, width * height * 3);
    if (data instanceof Uint16Array) {
      const channels = data.length >= width * height * 4 ? 4 : 3;
      if (!flipY && channels === 3) {
        u16Dst.set(data.subarray(0, width * height * 3));
      } else {
        let dstIdx = 0;
        for (let y = 0; y < height; y++) {
          const srcRow = flipY ? height - 1 - y : y;
          const rowOffset = srcRow * width * channels;
          for (let x = 0; x < width; x++) {
            const srcIdx = rowOffset + x * channels;
            u16Dst[dstIdx++] = data[srcIdx];
            u16Dst[dstIdx++] = data[srcIdx + 1];
            u16Dst[dstIdx++] = data[srcIdx + 2];
          }
        }
      }
    } else {
      let dstIdx = 0;
      for (let y = 0; y < height; y++) {
        const srcRow = flipY ? height - 1 - y : y;
        const rowOffset = srcRow * width * 4;
        for (let x = 0; x < width; x++) {
          const srcIdx = rowOffset + x * 4;
          u16Dst[dstIdx++] = data[srcIdx] * 257;
          u16Dst[dstIdx++] = data[srcIdx + 1] * 257;
          u16Dst[dstIdx++] = data[srcIdx + 2] * 257;
        }
      }
    }
  } else {
    const u8Data = data as Uint8Array;
    let dst = 192;
    for (let y = 0; y < height; y++) {
      const srcRow = flipY ? height - 1 - y : y;
      const rowOffset = srcRow * width * 4;
      for (let x = 0; x < width; x++) {
        const srcIdx = rowOffset + x * 4;
        u8[dst++] = u8Data[srcIdx];
        u8[dst++] = u8Data[srcIdx + 1];
        u8[dst++] = u8Data[srcIdx + 2];
      }
    }
  }

  return u8;
}

export function encodeTiff16(
  width: number,
  height: number,
  data: Uint16Array | Uint8Array,
  flipY: boolean = true,
): Uint8Array {
  return encodeTiff(width, height, data, 16, flipY);
}
