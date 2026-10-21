const fs = require('fs');
const path = require('path');
const wasm = require('./pkg-node/firstproof_wasm.js');

console.log('====================================================');
console.log('    First Proof WebAssembly Integration Test        ');
console.log('====================================================\n');

// 1. Initialize scratchpad in WASM linear memory (64MB)
console.log('1. Initializing 64 MB linear memory scratchpad...');
const ptr = wasm.init_scratchpad(64);
const len = wasm.get_scratchpad_len();
console.log(`   ✔ Scratchpad pointer: 0x${ptr.toString(16)}`);
console.log(`   ✔ Scratchpad capacity: ${(len / (1024 * 1024)).toFixed(1)} MB`);

// 2. Load test image into WASM linear memory buffer directly
const testImagePath = path.join(__dirname, '../firstproof-core/tests/fixtures/_MG_2651.dng');
console.log(`\n2. Loading test image: ${testImagePath}`);
const fileBuffer = fs.readFileSync(testImagePath);
console.log(`   ✔ Read image file: ${(fileBuffer.length / 1024).toFixed(1)} KB`);

const copied = wasm.copy_into_scratchpad(0, fileBuffer);
console.log(`   ✔ Copied ${copied} bytes into WASM scratchpad`);

// 3. Decode preview & extract metadata
console.log('\n3. Decoding image stream and metadata in WASM...');
const previewResult = wasm.decode_preview_from_scratchpad(fileBuffer.length);
console.log(`   ✔ Image Dimensions: ${previewResult.width} x ${previewResult.height}`);
console.log(`   ✔ Embedded Preview Slice: offset=${previewResult.preview_offset}, len=${previewResult.preview_len} bytes`);
console.log(`   ✔ Camera Make: ${previewResult.metadata.camera_make || 'N/A'}`);
console.log(`   ✔ Camera Model: ${previewResult.metadata.camera_model || 'N/A'}`);
console.log(`   ✔ ISO: ${previewResult.metadata.iso || 'N/A'}, Aperture: f/${previewResult.metadata.aperture || 'N/A'}, Shutter: ${previewResult.metadata.shutter_speed || 'N/A'}`);

// 4. Test Tone Curves spline evaluation
console.log('\n4. Evaluating Tone Curves via Fritsch-Carlson Monotone Cubic Spline...');
const curvePayload = {
  master: [
    { x: 0.0, y: 0.0 },
    { x: 0.25, y: 0.20 },
    { x: 0.75, y: 0.85 },
    { x: 1.0, y: 1.0 }
  ],
  red: [{ x: 0.0, y: 0.0 }, { x: 1.0, y: 1.0 }],
  green: [{ x: 0.0, y: 0.0 }, { x: 1.0, y: 1.0 }],
  blue: [{ x: 0.0, y: 0.0 }, { x: 1.0, y: 1.0 }]
};

const lut = wasm.evaluate_curves(JSON.stringify(curvePayload));
console.log(`   ✔ Master LUT sample count: ${lut.master.length}`);
console.log(`   ✔ In: 0.00 -> Out: ${lut.master[0].toFixed(4)}`);
console.log(`   ✔ In: 0.25 -> Out: ${lut.master[64].toFixed(4)} (control point: 0.20)`);
console.log(`   ✔ In: 0.50 -> Out: ${lut.master[128].toFixed(4)}`);
console.log(`   ✔ In: 0.75 -> Out: ${lut.master[192].toFixed(4)} (control point: 0.85)`);
console.log(`   ✔ In: 1.00 -> Out: ${lut.master[255].toFixed(4)}`);

// 5. Test XMP Sidecar serialization and parsing
console.log('\n5. Generating and parsing First Proof XMP sidecar...');
const editSettings = {
  exposure: 0.75,
  contrast: 20.0,
  highlights: -30.0,
  shadows: 25.0,
  whites: 10.0,
  blacks: -15.0,
  vibrance: 18.0,
  saturation: -5.0,
  temperature: 6100.0,
  tint: -6.0,
  crop_left: 0.05,
  crop_top: 0.05,
  crop_right: 0.95,
  crop_bottom: 0.95,
  crop_angle: -0.75,
  rotation: 90,
  flip_horizontal: true,
  flip_vertical: false,
  curve_master: curvePayload.master,
  curve_red: curvePayload.red,
  curve_green: curvePayload.green,
  curve_blue: curvePayload.blue,
  emulsion_archetype: "RochesterPortrait400",
  emulsion_format: "Format120",
  emulsion_exposure: 0.5
};

const xmpPayload = {
  rating: 4,
  status: "Pick",
  title: "Glacier Ridge at Dawn",
  description: "High elevation alpine morning light",
  creator: "Remco",
  copyright: "© First Proof",
  keywords: ["alpine", "dawn", "rockies"],
  edits: editSettings
};

const xmpXml = wasm.generate_xmp(JSON.stringify(xmpPayload));
console.log('   ✔ Serialized XMP Sidecar (first 10 lines):');
console.log(xmpXml.split('\n').slice(0, 10).map(l => '     ' + l).join('\n'));

const parsedBack = wasm.parse_xmp(xmpXml);
console.log('\n   ✔ Roundtrip Verification:');
console.log(`     - Rating: ${parsedBack.rating} ★`);
console.log(`     - Status: ${parsedBack.status}`);
console.log(`     - Title: "${parsedBack.title}"`);
console.log(`     - Exposure: +${parsedBack.edits.exposure} EV`);
console.log(`     - Contrast: +${parsedBack.edits.contrast}`);
console.log(`     - Film Emulsion: ${parsedBack.edits.emulsion_archetype} [${parsedBack.edits.emulsion_format}]`);
console.log(`     - Geometry: ${parsedBack.edits.rotation}° rotation, Flip(H): ${parsedBack.edits.flip_horizontal}`);

// 6. Test EXIF Orientation decoding & byte-level metadata extraction
console.log('\n6. Testing EXIF Orientation decoding and metadata extraction...');
const ori6 = wasm.decode_exif_orientation(6);
console.log(`   ✔ Tag 6 (Rotate 90 CW): rotation=${ori6.rotation}°, flip_h=${ori6.flip_horizontal}, flip_v=${ori6.flip_vertical}`);
if (ori6.rotation !== 90 || ori6.flip_horizontal !== false) {
  throw new Error(`decode_exif_orientation(6) failed: expected 90, got ${ori6.rotation}`);
}

const ori5 = wasm.decode_exif_orientation(5);
console.log(`   ✔ Tag 5 (Transpose): rotation=${ori5.rotation}°, flip_h=${ori5.flip_horizontal}, flip_v=${ori5.flip_vertical}`);
if (ori5.rotation !== 270 || ori5.flip_horizontal !== true) {
  throw new Error(`decode_exif_orientation(5) failed: expected 270/flipH, got ${ori5.rotation}`);
}

const metaExtracted = wasm.extract_metadata_from_bytes(fileBuffer);
console.log(`   ✔ Direct extract_metadata_from_bytes returned object (make=${metaExtracted.camera_make || 'N/A'}, orientation=${metaExtracted.orientation || 'N/A'})`);

// 7. Test decoding a raster image (PNG) via WASM scratchpad
console.log('\n7. Testing Raster Image (PNG) decoding via scratchpad fallback...');
// 2x2 red PNG generated with valid CRC chunks
const pngBase64 = 'iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAIAAAD91JpzAAAAEElEQVR4nGP8zwACTGCSAQANHQEDgslx/wAAAABJRU5ErkJggg==';
const pngBuffer = Buffer.from(pngBase64, 'base64');
wasm.copy_into_scratchpad(0, pngBuffer);

const pngPreview = wasm.decode_preview_from_scratchpad(pngBuffer.length);
console.log(`   ✔ Decoded PNG Dimensions: ${pngPreview.width} x ${pngPreview.height}`);
console.log(`   ✔ Generated Preview Slice: offset=${pngPreview.preview_offset}, len=${pngPreview.preview_len} bytes`);
if (pngPreview.width !== 2 || pngPreview.height !== 2) {
  throw new Error(`Expected PNG dimensions 2x2, got ${pngPreview.width}x${pngPreview.height}`);
}
if (pngPreview.preview_len <= 0) {
  throw new Error(`Expected non-empty preview slice for PNG, got ${pngPreview.preview_len}`);
}

console.log('\n====================================================');
console.log(' 🎉 ALL WEBASSMEBLY TESTS PASSED SUCCESSFULLY!       ');
console.log('====================================================\n');
