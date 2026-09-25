import { EditSettingsPayload, EvaluatedLut } from './WasmBridge';
import { FRAGMENT_SHADER_SOURCE as FRAGMENT_SHADER_SRC } from './shaders/FragmentShader';

const VERTEX_SHADER_SRC = `#version 300 es
in vec2 a_position;
in vec2 a_texCoord;

uniform vec2 u_fitScale;
uniform float u_zoomScale;
uniform vec2 u_panOffset;

out vec2 v_texCoord;

void main() {
    vec2 pos = a_position * u_fitScale * u_zoomScale + u_panOffset;
    gl_Position = vec4(pos, 0.0, 1.0);
    v_texCoord = a_texCoord;
}
`;

const EMULSION_MAP: Record<string, number> = {
  None: 0,
  RochesterPortrait400: 1,
  AshigaraChrome50: 2,
  RochesterK14Chrome64: 3,
  RochesterCinema500: 4,
  RochesterDaylight200: 5,
  MobberleySilver400: 6,
  CambridgeDiffusion100: 7,
};

const PRINT_MEDIA_MAP: Record<string, number> = {
  None: 0,
  ChromogenicTypeC: 1,
  SilverDyeBleach: 2,
  FineArtCottonRag: 3,
  BarytaSilverGelatin: 4,
  WarmtoneChlorobromide: 5,
  PlatinumPalladium: 6,
  Cyanotype: 7,
  PrussianCyanotype: 7,
};

export class WebGLRenderer {
  private gl: WebGL2RenderingContext;
  private program: WebGLProgram;
  private texture: WebGLTexture | null = null;
  private masterLutTex: WebGLTexture | null = null;
  private rgbLutTex: WebGLTexture | null = null;
  private isLinearRec2020: boolean = false;
  private vbo: WebGLBuffer | null = null;
  private currentRotation: number = 0;
  private currentFlipH: boolean = false;
  private currentFlipV: boolean = false;
  private currentCrop: { left: number; top: number; right: number; bottom: number } = { left: 0, top: 0, right: 1, bottom: 1 };
  private currentCropModeActive: boolean = false;
  private texWidth: number = 1920;
  private texHeight: number = 1080;

  public get textureWidth(): number {
    return this.texWidth;
  }

  public get textureHeight(): number {
    return this.texHeight;
  }

  public get glContext(): WebGL2RenderingContext {
    return this.gl;
  }

  public readRgbaPixels(width: number, height: number): Uint8Array {
    const pixels = new Uint8Array(width * height * 4);
    this.gl.readPixels(0, 0, width, height, this.gl.RGBA, this.gl.UNSIGNED_BYTE, pixels);
    return pixels;
  }

  // Uniform locations
  private uLocs: Record<string, WebGLUniformLocation | null> = {};

  constructor(canvas: HTMLCanvasElement) {
    const gl = canvas.getContext('webgl2', { alpha: false, antialias: true, preserveDrawingBuffer: true });
    if (!gl) {
      throw new Error('WebGL 2 is not supported by your browser');
    }
    this.gl = gl;
    this.program = this.initProgram();
    this.initBuffers();
    this.initLutTextures();
  }

  private createShader(type: number, src: string): WebGLShader {
    const shader = this.gl.createShader(type)!;
    this.gl.shaderSource(shader, src);
    this.gl.compileShader(shader);
    if (!this.gl.getShaderParameter(shader, this.gl.COMPILE_STATUS)) {
      const info = this.gl.getShaderInfoLog(shader);
      this.gl.deleteShader(shader);
      throw new Error(`Shader compile failure: ${info}`);
    }
    return shader;
  }

  private initProgram(): WebGLProgram {
    const vert = this.createShader(this.gl.VERTEX_SHADER, VERTEX_SHADER_SRC);
    const frag = this.createShader(this.gl.FRAGMENT_SHADER, FRAGMENT_SHADER_SRC);
    const prog = this.gl.createProgram()!;
    this.gl.attachShader(prog, vert);
    this.gl.attachShader(prog, frag);
    this.gl.linkProgram(prog);
    if (!this.gl.getProgramParameter(prog, this.gl.LINK_STATUS)) {
      throw new Error(`Program link error: ${this.gl.getProgramInfoLog(prog)}`);
    }
    this.gl.useProgram(prog);

    // Cache uniforms
    const uniforms = [
      'u_texture', 'u_curveMasterLut', 'u_curveRgbLut',
      'u_exposure', 'u_contrast', 'u_highlights', 'u_shadows', 'u_whites', 'u_blacks',
      'u_vibrance', 'u_saturation', 'u_deepBlue', 'u_foliage', 'u_temperature', 'u_tint',
      'u_toneProfileMode', 'u_emulsionArchetype', 'u_emulsionExposure',
      'u_printMediaMode', 'u_printPaperGrade', 'u_printBaseWarmth', 'u_printProcessDepth', 'u_enlargerFilterMode',
      'u_splitShadowHue', 'u_splitShadowSat', 'u_splitHighlightHue', 'u_splitHighlightSat', 'u_splitBalance',
      'u_sharpeningAmount', 'u_sharpeningRadius', 'u_sharpeningMasking', 'u_noiseReduction',
      'u_fitScale', 'u_zoomScale', 'u_panOffset', 'u_cropLeft', 'u_cropTop', 'u_cropRight', 'u_cropBottom', 'u_cropModeActive',
      'u_isOriginal', 'u_isLinearRec2020', 'u_hasMasterCurve', 'u_hasRgbCurves'
    ];
    for (const u of uniforms) {
      this.uLocs[u] = this.gl.getUniformLocation(prog, u);
    }

    return prog;
  }

  private initBuffers() {
    const gl = this.gl;
    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);

    // Quad geometry: pos(2), uv(2)
    const vertices = new Float32Array([
      -1, -1,  0, 1,
       1, -1,  1, 1,
      -1,  1,  0, 0,
       1,  1,  1, 0,
    ]);

    const vbo = gl.createBuffer();
    this.vbo = vbo;
    gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.DYNAMIC_DRAW);

    const aPos = gl.getAttribLocation(this.program, 'a_position');
    const aTex = gl.getAttribLocation(this.program, 'a_texCoord');

    gl.enableVertexAttribArray(aPos);
    gl.vertexAttribPointer(aPos, 2, gl.FLOAT, false, 16, 0);

    gl.enableVertexAttribArray(aTex);
    gl.vertexAttribPointer(aTex, 2, gl.FLOAT, false, 16, 8);
  }

  public updateQuadGeometry(
    rotation: number = 0,
    flipH: boolean = false,
    flipV: boolean = false,
    crop: { left: number; top: number; right: number; bottom: number } = { left: 0, top: 0, right: 1, bottom: 1 },
    cropModeActive: boolean = false
  ) {
    if (!this.vbo) return;
    const effectiveCrop = cropModeActive ? { left: 0, top: 0, right: 1, bottom: 1 } : crop;
    if (
      this.currentRotation === rotation &&
      this.currentFlipH === flipH &&
      this.currentFlipV === flipV &&
      this.currentCropModeActive === cropModeActive &&
      this.currentCrop.left === effectiveCrop.left &&
      this.currentCrop.top === effectiveCrop.top &&
      this.currentCrop.right === effectiveCrop.right &&
      this.currentCrop.bottom === effectiveCrop.bottom
    ) {
      return;
    }
    this.currentRotation = rotation;
    this.currentFlipH = flipH;
    this.currentFlipV = flipV;
    this.currentCropModeActive = cropModeActive;
    this.currentCrop = { ...effectiveCrop };

    const rot = ((rotation % 360) + 360) % 360;
    const mapScreenToTex = (xs: number, ys: number): [number, number] => {
      switch (rot) {
        case 90: return [ys, 1.0 - xs];
        case 180: return [1.0 - xs, 1.0 - ys];
        case 270: return [1.0 - ys, xs];
        default: return [xs, ys];
      }
    };

    let tl = mapScreenToTex(effectiveCrop.left, effectiveCrop.top);
    let tr = mapScreenToTex(effectiveCrop.right, effectiveCrop.top);
    let bl = mapScreenToTex(effectiveCrop.left, effectiveCrop.bottom);
    let br = mapScreenToTex(effectiveCrop.right, effectiveCrop.bottom);

    if (flipH) {
      const tmpT = tl; tl = tr; tr = tmpT;
      const tmpB = bl; bl = br; br = tmpB;
    }
    if (flipV) {
      const tmpL = tl; tl = bl; bl = tmpL;
      const tmpR = tr; tr = br; br = tmpR;
    }

    // Quad geometry matching TRIANGLE_STRIP: (-1, -1), (1, -1), (-1, 1), (1, 1)
    const vertices = new Float32Array([
      -1, -1,  bl[0], bl[1],
       1, -1,  br[0], br[1],
      -1,  1,  tl[0], tl[1],
       1,  1,  tr[0], tr[1],
    ]);

    const gl = this.gl;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.vbo);
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, vertices);
  }

  private initLutTextures() {
    const gl = this.gl;
    const identityBytes = new Uint8Array(256 * 4);
    for (let i = 0; i < 256; i++) {
      identityBytes[i * 4] = i;
      identityBytes[i * 4 + 1] = i;
      identityBytes[i * 4 + 2] = i;
      identityBytes[i * 4 + 3] = 255;
    }

    this.masterLutTex = gl.createTexture();
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.masterLutTex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 256, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, identityBytes);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);

    this.rgbLutTex = gl.createTexture();
    gl.activeTexture(gl.TEXTURE2);
    gl.bindTexture(gl.TEXTURE_2D, this.rgbLutTex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 256, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, identityBytes);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  }

  public uploadImage(image: HTMLImageElement | HTMLCanvasElement | ImageBitmap | ImageData) {
    this.isLinearRec2020 = false;
    this.texWidth = (image as any).width || 1920;
    this.texHeight = (image as any).height || 1080;
    const gl = this.gl;
    if (!this.texture) {
      this.texture = gl.createTexture();
    }
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false);

    if (image instanceof ImageData) {
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, image.width, image.height, 0, gl.RGBA, gl.UNSIGNED_BYTE, image.data);
    } else {
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
    }
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  }

  public uploadLinearRec2020Texture(width: number, height: number, data: Uint16Array) {
    this.isLinearRec2020 = true;
    this.texWidth = width;
    this.texHeight = height;
    const gl = this.gl;
    gl.getExtension('EXT_color_buffer_float');

    if (!this.texture) {
      this.texture = gl.createTexture();
    }
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false);

    // Upload 16-bit IEEE 754 Half-Float RGBA16F linear Rec.2020 texture (un-clipped 0.0 .. 4.0+)
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA16F,
      width,
      height,
      0,
      gl.RGBA,
      gl.HALF_FLOAT,
      data
    );
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  }

  public updateToneCurvesLut(lut: EvaluatedLut) {
    this.uploadCurveLuts(lut);
  }

  public uploadCurveLuts(lut: EvaluatedLut) {
    const gl = this.gl;

    // Master LUT
    const masterBytes = new Uint8Array(256 * 4);
    for (let i = 0; i < 256; i++) {
      const val = Math.round(Math.min(Math.max(lut.master[i], 0), 1) * 255);
      masterBytes[i * 4] = val;
      masterBytes[i * 4 + 1] = val;
      masterBytes[i * 4 + 2] = val;
      masterBytes[i * 4 + 3] = 255;
    }
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.masterLutTex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 256, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, masterBytes);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    // RGB LUT
    const rgbBytes = new Uint8Array(256 * 4);
    for (let i = 0; i < 256; i++) {
      rgbBytes[i * 4] = Math.round(Math.min(Math.max(lut.red[i], 0), 1) * 255);
      rgbBytes[i * 4 + 1] = Math.round(Math.min(Math.max(lut.green[i], 0), 1) * 255);
      rgbBytes[i * 4 + 2] = Math.round(Math.min(Math.max(lut.blue[i], 0), 1) * 255);
      rgbBytes[i * 4 + 3] = 255;
    }
    gl.activeTexture(gl.TEXTURE2);
    gl.bindTexture(gl.TEXTURE_2D, this.rgbLutTex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 256, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, rgbBytes);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  }

  public computeFitScale(
    rotation: number = 0,
    crop: { left: number; top: number; right: number; bottom: number } = { left: 0, top: 0, right: 1, bottom: 1 },
    cropModeActive: boolean = false
  ): { fitScaleX: number; fitScaleY: number } {
    const isOrthogonal = (rotation === 90 || rotation === 270);
    const origW = isOrthogonal ? this.texHeight : this.texWidth;
    const origH = isOrthogonal ? this.texWidth : this.texHeight;
    const cropW = cropModeActive ? 1.0 : Math.max(0.01, (crop.right ?? 1.0) - (crop.left ?? 0.0));
    const cropH = cropModeActive ? 1.0 : Math.max(0.01, (crop.bottom ?? 1.0) - (crop.top ?? 0.0));
    const dispW = Math.max(1, origW * cropW);
    const dispH = Math.max(1, origH * cropH);
    const imgAspect = dispW / dispH;

    const viewW = this.gl.canvas.width;
    const viewH = this.gl.canvas.height;
    const viewAspect = (viewW > 0 && viewH > 0) ? (viewW / viewH) : imgAspect;

    let fitScaleX = 1.0;
    let fitScaleY = 1.0;
    if (viewAspect > imgAspect) {
      fitScaleY = 1.0;
      fitScaleX = (viewH * imgAspect) / Math.max(viewW, 1);
    } else {
      fitScaleX = 1.0;
      fitScaleY = (viewW / imgAspect) / Math.max(viewH, 1);
    }
    return { fitScaleX, fitScaleY };
  }

  public render(
    edits: EditSettingsPayload,
    isOriginal: boolean = false,
    zoomScale: number = 1.0,
    panOffset: { x: number; y: number } = { x: 0, y: 0 },
    cropModeActive: boolean = false
  ) {
    const gl = this.gl;
    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
    gl.clearColor(0.05, 0.06, 0.08, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.useProgram(this.program);

    this.updateQuadGeometry(
      edits.rotation ?? 0,
      edits.flip_horizontal ?? false,
      edits.flip_vertical ?? false,
      {
        left: edits.crop_left ?? 0.0,
        top: edits.crop_top ?? 0.0,
        right: edits.crop_right ?? 1.0,
        bottom: edits.crop_bottom ?? 1.0,
      },
      cropModeActive
    );

    const { fitScaleX, fitScaleY } = this.computeFitScale(
      edits.rotation ?? 0,
      {
        left: edits.crop_left ?? 0.0,
        top: edits.crop_top ?? 0.0,
        right: edits.crop_right ?? 1.0,
        bottom: edits.crop_bottom ?? 1.0,
      },
      cropModeActive
    );

    // Zoom & Pan & Fit Scale
    gl.uniform2f(this.uLocs['u_fitScale'], fitScaleX, fitScaleY);
    gl.uniform1f(this.uLocs['u_zoomScale'], zoomScale);
    gl.uniform2f(this.uLocs['u_panOffset'], panOffset.x, panOffset.y);

    // Crop Coordinates
    gl.uniform1f(this.uLocs['u_cropLeft'], edits.crop_left ?? 0.0);
    gl.uniform1f(this.uLocs['u_cropTop'], edits.crop_top ?? 0.0);
    gl.uniform1f(this.uLocs['u_cropRight'], edits.crop_right ?? 1.0);
    gl.uniform1f(this.uLocs['u_cropBottom'], edits.crop_bottom ?? 1.0);
    gl.uniform1i(this.uLocs['u_cropModeActive'], cropModeActive ? 1 : 0);

    if (this.texture) {
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, this.texture);
    }
    if (this.masterLutTex) {
      gl.activeTexture(gl.TEXTURE1);
      gl.bindTexture(gl.TEXTURE_2D, this.masterLutTex);
    }
    if (this.rgbLutTex) {
      gl.activeTexture(gl.TEXTURE2);
      gl.bindTexture(gl.TEXTURE_2D, this.rgbLutTex);
    }

    gl.uniform1i(this.uLocs['u_texture'], 0);
    gl.uniform1i(this.uLocs['u_curveMasterLut'], 1);
    gl.uniform1i(this.uLocs['u_curveRgbLut'], 2);

    gl.uniform1f(this.uLocs['u_exposure'], edits.exposure);
    gl.uniform1f(this.uLocs['u_contrast'], edits.contrast);
    gl.uniform1f(this.uLocs['u_highlights'], edits.highlights);
    gl.uniform1f(this.uLocs['u_shadows'], edits.shadows);
    gl.uniform1f(this.uLocs['u_whites'], edits.whites);
    gl.uniform1f(this.uLocs['u_blacks'], edits.blacks);

    gl.uniform1f(this.uLocs['u_vibrance'], edits.vibrance);
    gl.uniform1f(this.uLocs['u_saturation'], edits.saturation);
    gl.uniform1f(this.uLocs['u_deepBlue'], edits.deep_blue);
    gl.uniform1f(this.uLocs['u_foliage'], edits.foliage);
    gl.uniform1f(this.uLocs['u_temperature'], edits.temperature);
    gl.uniform1f(this.uLocs['u_tint'], edits.tint);

    const modeMap: Record<string, number> = {
      AgxVivid: 0, AgxFilmic: 1, CameraStandard: 2, CameraPortrait: 3, CameraLandscape: 4, CameraVivid: 5
    };
    gl.uniform1i(this.uLocs['u_toneProfileMode'], modeMap[edits.tone_profile] ?? 0);
    gl.uniform1i(this.uLocs['u_emulsionArchetype'], EMULSION_MAP[edits.emulsion_archetype] ?? 0);
    gl.uniform1f(this.uLocs['u_emulsionExposure'], edits.emulsion_exposure);

    gl.uniform1i(this.uLocs['u_printMediaMode'], PRINT_MEDIA_MAP[edits.print_media] ?? 0);
    gl.uniform1f(this.uLocs['u_printPaperGrade'], edits.print_paper_grade);
    gl.uniform1f(this.uLocs['u_printBaseWarmth'], edits.print_base_warmth);
    gl.uniform1f(this.uLocs['u_printProcessDepth'], (edits as any).print_process_depth ?? 100.0);
    const filterMap: Record<string, number> = {
      NeutralPanchromatic: 0, FilterGrade2: 0, YellowK2: 1, Orange16: 2, Red25A: 3, GreenX1: 4, Orthochromatic: 5
    };
    gl.uniform1i(this.uLocs['u_enlargerFilterMode'], filterMap[(edits as any).enlarger_filter] ?? 0);

    gl.uniform1f(this.uLocs['u_splitShadowHue'], edits.split_toning_shadow_hue);
    gl.uniform1f(this.uLocs['u_splitShadowSat'], edits.split_toning_shadow_sat);
    gl.uniform1f(this.uLocs['u_splitHighlightHue'], edits.split_toning_highlight_hue);
    gl.uniform1f(this.uLocs['u_splitHighlightSat'], edits.split_toning_highlight_sat);
    gl.uniform1f(this.uLocs['u_splitBalance'], edits.split_toning_balance);
    gl.uniform1f(this.uLocs['u_sharpeningAmount'], edits.sharpening_amount ?? 0.0);
    gl.uniform1f(this.uLocs['u_sharpeningRadius'], edits.sharpening_radius ?? 1.0);
    gl.uniform1f(this.uLocs['u_sharpeningMasking'], edits.sharpening_masking ?? 0.0);
    gl.uniform1f(this.uLocs['u_noiseReduction'], edits.noise_reduction ?? 0.0);

    gl.uniform1i(this.uLocs['u_isOriginal'], isOriginal ? 1 : 0);
    gl.uniform1i(this.uLocs['u_isLinearRec2020'], this.isLinearRec2020 ? 1 : 0);
    gl.uniform1i(this.uLocs['u_hasMasterCurve'], edits.curves.master.length > 2 ? 1 : 0);
    gl.uniform1i(this.uLocs['u_hasRgbCurves'], (edits.curves.red.length > 2 || edits.curves.green.length > 2 || edits.curves.blue.length > 2) ? 1 : 0);

    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
  }

  public getCanvas(): HTMLCanvasElement {
    return this.gl.canvas as HTMLCanvasElement;
  }

  public readPixels(): Uint8Array {
    const gl = this.gl;
    const pixels = new Uint8Array(gl.canvas.width * gl.canvas.height * 4);
    gl.readPixels(0, 0, gl.canvas.width, gl.canvas.height, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    return pixels;
  }

  public static createOffscreen(width: number, height: number): WebGLRenderer {
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    return new WebGLRenderer(canvas);
  }
}
