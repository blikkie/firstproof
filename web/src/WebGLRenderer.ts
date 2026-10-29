import {
  FRAGMENT_SHADER_SOURCE as FRAGMENT_SHADER_SRC,
  SHADER_SAMPLER_NAMES,
  SHADER_UNIFORM_BLOCK_NAME,
  SHADER_UNIFORM_SIZE_BYTES,
} from './shaders/GeneratedGLSL';
import { isCurveActive } from './utils/edits';
import type {
  EditSettingsPayload,
  EvaluatedLut,
  LensCorrectionCoefficients,
} from './WasmBridge';

const VERTEX_SHADER_SRC = `#version 300 es
in vec2 a_position;
in vec2 a_texCoord;

uniform vec2 u_fitScale;
uniform float u_zoomScale;
uniform vec2 u_panOffset;

smooth out vec2 _vs2fs_location0;

void main() {
    vec2 pos = a_position * u_fitScale * u_zoomScale + u_panOffset;
    gl_Position = vec4(pos, 0.0, 1.0);
    _vs2fs_location0 = a_texCoord;
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
  LeverkusenPortrait160: 8,
  AshigaraPortrait400: 9,
  LiguriaCinema80: 10,
  BaodingSilver100: 11,
};

const GRAIN_MODE_MAP: Record<string, number> = {
  Off: 0,
  SeedFromSensor: 1,
  SynthesizePhysical: 2,
};

const PRINT_MEDIA_MAP: Record<string, number> = {
  None: 0,
  ChromogenicTypeC: 1,
  SilverDyeBleach: 2,
  BarytaSilverGelatin: 3,
  WarmtoneChlorobromide: 4,
  PlatinumPalladium: 5,
  Cyanotype: 6,
  PrussianCyanotype: 6,
  BamakoSilverContact: 7,
};

export class WebGLRenderer {
  private gl: WebGL2RenderingContext;
  private program: WebGLProgram;
  private texture: WebGLTexture | null = null;
  private masterLutTex: WebGLTexture | null = null;
  private rgbLutTex: WebGLTexture | null = null;
  private proofLutTex: WebGLTexture | null = null;
  private isIntermediate: boolean = false;
  private isLinearRec2020: boolean = false;
  private baselineSharpness: number = 1.0;
  public clippingOverlayActive: boolean = false;
  public lensCoefficients: LensCorrectionCoefficients | null = null;
  private vbo: WebGLBuffer | null = null;
  private texWidth: number = 1920;
  private texHeight: number = 1080;

  public setLensCoefficients(coeffs: LensCorrectionCoefficients | null): void {
    this.lensCoefficients = coeffs;
  }

  public setBaselineSharpness(val: number): void {
    this.baselineSharpness = val;
  }

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
    this.gl.readPixels(
      0,
      0,
      width,
      height,
      this.gl.RGBA,
      this.gl.UNSIGNED_BYTE,
      pixels,
    );
    return pixels;
  }

  // Uniform locations
  private uLocs: Record<string, WebGLUniformLocation | null> = {};
  private uboBuffer: WebGLBuffer | null = null;
  private uboBufferData = new ArrayBuffer(SHADER_UNIFORM_SIZE_BYTES);
  private uboFloatView = new Float32Array(this.uboBufferData);
  private uboIntView = new Int32Array(this.uboBufferData);

  constructor(canvas: HTMLCanvasElement) {
    const gl = canvas.getContext('webgl2', {
      alpha: false,
      antialias: true,
      preserveDrawingBuffer: true,
    });
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
    const frag = this.createShader(
      this.gl.FRAGMENT_SHADER,
      FRAGMENT_SHADER_SRC,
    );
    const prog = this.gl.createProgram()!;
    this.gl.attachShader(prog, vert);
    this.gl.attachShader(prog, frag);
    this.gl.linkProgram(prog);
    if (!this.gl.getProgramParameter(prog, this.gl.LINK_STATUS)) {
      throw new Error(`Program link error: ${this.gl.getProgramInfoLog(prog)}`);
    }
    this.gl.useProgram(prog);

    // Bind Uniform Buffer Object (UBO) for ShaderUniforms
    const blockIndex = this.gl.getUniformBlockIndex(
      prog,
      SHADER_UNIFORM_BLOCK_NAME,
    );
    if (blockIndex !== this.gl.INVALID_INDEX) {
      this.gl.uniformBlockBinding(prog, blockIndex, 0);
    }

    this.uboBuffer = this.gl.createBuffer();
    this.gl.bindBuffer(this.gl.UNIFORM_BUFFER, this.uboBuffer);
    this.gl.bufferData(
      this.gl.UNIFORM_BUFFER,
      SHADER_UNIFORM_SIZE_BYTES,
      this.gl.DYNAMIC_DRAW,
    );
    this.gl.bindBufferBase(this.gl.UNIFORM_BUFFER, 0, this.uboBuffer);

    // Bind texture samplers to texture units
    const uTexture = this.gl.getUniformLocation(
      prog,
      SHADER_SAMPLER_NAMES.texture,
    );
    const uLut3d = this.gl.getUniformLocation(prog, SHADER_SAMPLER_NAMES.lut3d);
    const uMasterLut = this.gl.getUniformLocation(
      prog,
      SHADER_SAMPLER_NAMES.curveMaster,
    );
    const uRgbLut = this.gl.getUniformLocation(
      prog,
      SHADER_SAMPLER_NAMES.curveRgb,
    );
    const uGainMap = this.gl.getUniformLocation(
      prog,
      SHADER_SAMPLER_NAMES.gainMap,
    );
    const uProofLut = this.gl.getUniformLocation(
      prog,
      SHADER_SAMPLER_NAMES.proofLut,
    );

    if (uTexture) this.gl.uniform1i(uTexture, 0);
    if (uLut3d) this.gl.uniform1i(uLut3d, 1);
    if (uMasterLut) this.gl.uniform1i(uMasterLut, 2);
    if (uRgbLut) this.gl.uniform1i(uRgbLut, 3);
    if (uGainMap) this.gl.uniform1i(uGainMap, 4);
    if (uProofLut) this.gl.uniform1i(uProofLut, 5);

    // Vertex shader geometry uniforms
    this.uLocs.u_fitScale = this.gl.getUniformLocation(prog, 'u_fitScale');
    this.uLocs.u_zoomScale = this.gl.getUniformLocation(prog, 'u_zoomScale');
    this.uLocs.u_panOffset = this.gl.getUniformLocation(prog, 'u_panOffset');

    return prog;
  }

  private initBuffers() {
    const gl = this.gl;
    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);

    // Static immutable full-screen quad: pos(2), uv(2)
    // TRIANGLE_STRIP order: (-1, -1), (1, -1), (-1, 1), (1, 1)
    const vertices = new Float32Array([
      -1, -1, 0, 1, 1, -1, 1, 1, -1, 1, 0, 0, 1, 1, 1, 0,
    ]);

    const vbo = gl.createBuffer();
    this.vbo = vbo;
    gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    const aPos = gl.getAttribLocation(this.program, 'a_position');
    const aTex = gl.getAttribLocation(this.program, 'a_texCoord');

    gl.enableVertexAttribArray(aPos);
    gl.vertexAttribPointer(aPos, 2, gl.FLOAT, false, 16, 0);

    gl.enableVertexAttribArray(aTex);
    gl.vertexAttribPointer(aTex, 2, gl.FLOAT, false, 16, 8);
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
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA,
      256,
      1,
      0,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      identityBytes,
    );
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);

    this.rgbLutTex = gl.createTexture();
    gl.activeTexture(gl.TEXTURE2);
    gl.bindTexture(gl.TEXTURE_2D, this.rgbLutTex);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA,
      256,
      1,
      0,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      identityBytes,
    );
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);

    this.proofLutTex = gl.createTexture();
    gl.activeTexture(gl.TEXTURE5);
    gl.bindTexture(gl.TEXTURE_3D, this.proofLutTex);
    const defaultProofBytes = new Uint8Array([
      0, 0, 0, 255, 255, 0, 0, 255, 0, 255, 0, 255, 255, 255, 0, 255, 0, 0, 255,
      255, 255, 0, 255, 255, 0, 255, 255, 255, 255, 255, 255, 255,
    ]);
    gl.texImage3D(
      gl.TEXTURE_3D,
      0,
      gl.RGBA,
      2,
      2,
      2,
      0,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      defaultProofBytes,
    );
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_R, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  }

  public clearCanvas() {
    const gl = this.gl;
    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
    gl.clearColor(0.05, 0.06, 0.08, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);
  }

  public uploadImage(
    image: HTMLImageElement | HTMLCanvasElement | ImageBitmap | ImageData,
  ) {
    this.isLinearRec2020 = false;
    this.isIntermediate = false;
    this.texWidth = image.width || 1920;
    this.texHeight = image.height || 1080;
    const gl = this.gl;
    if (!this.texture) {
      this.texture = gl.createTexture();
    }
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false);

    if (image instanceof ImageData) {
      gl.texImage2D(
        gl.TEXTURE_2D,
        0,
        gl.RGBA,
        image.width,
        image.height,
        0,
        gl.RGBA,
        gl.UNSIGNED_BYTE,
        image.data,
      );
    } else {
      gl.texImage2D(
        gl.TEXTURE_2D,
        0,
        gl.RGBA,
        gl.RGBA,
        gl.UNSIGNED_BYTE,
        image,
      );
    }
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  }

  public uploadIntermediate(
    image: HTMLImageElement | HTMLCanvasElement | ImageBitmap | ImageData,
  ) {
    this.isLinearRec2020 = false;
    this.isIntermediate = true;
    this.texWidth = image.width || 1920;
    this.texHeight = image.height || 1080;
    const gl = this.gl;
    if (!this.texture) {
      this.texture = gl.createTexture();
    }
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false);

    if (image instanceof ImageData) {
      gl.texImage2D(
        gl.TEXTURE_2D,
        0,
        gl.RGBA,
        image.width,
        image.height,
        0,
        gl.RGBA,
        gl.UNSIGNED_BYTE,
        image.data,
      );
    } else {
      gl.texImage2D(
        gl.TEXTURE_2D,
        0,
        gl.RGBA,
        gl.RGBA,
        gl.UNSIGNED_BYTE,
        image,
      );
    }
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  }

  public uploadLinearRec2020Texture(
    width: number,
    height: number,
    data: Uint16Array,
  ) {
    this.isLinearRec2020 = true;
    this.isIntermediate = false;
    this.texWidth = width;
    this.texHeight = height;
    const gl = this.gl;
    gl.getExtension('EXT_color_buffer_float');
    gl.getExtension('OES_texture_float_linear');

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
      data,
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
    gl.activeTexture(gl.TEXTURE2);
    gl.bindTexture(gl.TEXTURE_2D, this.masterLutTex);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA,
      256,
      1,
      0,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      masterBytes,
    );
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    // RGB LUT
    const rgbBytes = new Uint8Array(256 * 4);
    for (let i = 0; i < 256; i++) {
      rgbBytes[i * 4] = Math.round(Math.min(Math.max(lut.red[i], 0), 1) * 255);
      rgbBytes[i * 4 + 1] = Math.round(
        Math.min(Math.max(lut.green[i], 0), 1) * 255,
      );
      rgbBytes[i * 4 + 2] = Math.round(
        Math.min(Math.max(lut.blue[i], 0), 1) * 255,
      );
      rgbBytes[i * 4 + 3] = 255;
    }
    gl.activeTexture(gl.TEXTURE3);
    gl.bindTexture(gl.TEXTURE_2D, this.rgbLutTex);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA,
      256,
      1,
      0,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      rgbBytes,
    );
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  }

  public computeFitScale(
    rotation: number = 0,
    crop: { left: number; top: number; right: number; bottom: number } = {
      left: 0,
      top: 0,
      right: 1,
      bottom: 1,
    },
    cropModeActive: boolean = false,
  ): { fitScaleX: number; fitScaleY: number } {
    const origW = this.texWidth;
    const origH = this.texHeight;
    const cropWFrac = cropModeActive
      ? 1.0
      : Math.max(0.01, (crop.right ?? 1.0) - (crop.left ?? 0.0));
    const cropHFrac = cropModeActive
      ? 1.0
      : Math.max(0.01, (crop.bottom ?? 1.0) - (crop.top ?? 0.0));
    const effectiveOrigW = origW * cropWFrac;
    const effectiveOrigH = origH * cropHFrac;

    const isOrthogonal = rotation === 90 || rotation === 270;
    const dispW = Math.max(1, isOrthogonal ? effectiveOrigH : effectiveOrigW);
    const dispH = Math.max(1, isOrthogonal ? effectiveOrigW : effectiveOrigH);
    const imgAspect = dispW / dispH;

    const viewW = this.gl.canvas.width;
    const viewH = this.gl.canvas.height;
    const viewAspect = viewW > 0 && viewH > 0 ? viewW / viewH : imgAspect;

    let fitScaleX = 1.0;
    let fitScaleY = 1.0;
    if (viewAspect > imgAspect) {
      fitScaleY = 1.0;
      fitScaleX = (viewH * imgAspect) / Math.max(viewW, 1);
    } else {
      fitScaleX = 1.0;
      fitScaleY = viewW / imgAspect / Math.max(viewH, 1);
    }
    return { fitScaleX, fitScaleY };
  }

  public getPhotoViewportBounds(
    edits: EditSettingsPayload,
    cropModeActive: boolean = false,
  ): { x: number; y: number; width: number; height: number } {
    const { fitScaleX, fitScaleY } = this.computeFitScale(
      edits.rotation ?? 0,
      {
        left: edits.crop_left ?? 0.0,
        top: edits.crop_top ?? 0.0,
        right: edits.crop_right ?? 1.0,
        bottom: edits.crop_bottom ?? 1.0,
      },
      cropModeActive,
    );
    const canvasW = this.gl.canvas.width;
    const canvasH = this.gl.canvas.height;
    const photoW = Math.max(1, Math.round(canvasW * fitScaleX));
    const photoH = Math.max(1, Math.round(canvasH * fitScaleY));
    const photoX = Math.max(0, Math.round((canvasW - photoW) / 2));
    const photoY = Math.max(0, Math.round((canvasH - photoH) / 2));
    return { x: photoX, y: photoY, width: photoW, height: photoH };
  }

  public async captureIntermediateSnapshot(
    edits: EditSettingsPayload,
  ): Promise<ImageBitmap | null> {
    if (this.isIntermediate) return null; // Don't snapshot an intermediate of an intermediate
    const bounds = this.getPhotoViewportBounds(edits, false);
    try {
      return await createImageBitmap(
        this.gl.canvas,
        bounds.x,
        bounds.y,
        bounds.width,
        bounds.height,
      );
    } catch (err) {
      console.warn('[WebGLRenderer] captureIntermediateSnapshot failed:', err);
      return null;
    }
  }

  public render(
    edits: EditSettingsPayload,
    isOriginal: boolean = false,
    zoomScale: number = 1.0,
    panOffset: { x: number; y: number } = { x: 0, y: 0 },
    cropModeActive: boolean = false,
  ) {
    const gl = this.gl;
    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
    gl.clearColor(0.05, 0.06, 0.08, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.useProgram(this.program);

    // Intermediate Bypass & Idempotence: intermediate textures already have geometry baked in.
    // Use identity geometry when rendering an intermediate texture.
    const effectiveEdits = this.isIntermediate
      ? {
          ...edits,
          rotation: 0,
          flip_horizontal: false,
          flip_vertical: false,
          crop_left: 0.0,
          crop_top: 0.0,
          crop_right: 1.0,
          crop_bottom: 1.0,
          crop_angle: 0.0,
          crop_constrain: false,
        }
      : edits;

    const { fitScaleX, fitScaleY } = this.computeFitScale(
      effectiveEdits.rotation ?? 0,
      {
        left: effectiveEdits.crop_left ?? 0.0,
        top: effectiveEdits.crop_top ?? 0.0,
        right: effectiveEdits.crop_right ?? 1.0,
        bottom: effectiveEdits.crop_bottom ?? 1.0,
      },
      cropModeActive,
    );

    // Viewport Geometry & Fit Scale
    gl.uniform2f(this.uLocs.u_fitScale, fitScaleX, fitScaleY);
    gl.uniform1f(this.uLocs.u_zoomScale, zoomScale);
    gl.uniform2f(this.uLocs.u_panOffset, panOffset.x, panOffset.y);

    if (this.texture) {
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, this.texture);
    }
    if (this.masterLutTex) {
      gl.activeTexture(gl.TEXTURE2);
      gl.bindTexture(gl.TEXTURE_2D, this.masterLutTex);
    }
    if (this.rgbLutTex) {
      gl.activeTexture(gl.TEXTURE3);
      gl.bindTexture(gl.TEXTURE_2D, this.rgbLutTex);
    }
    if (this.proofLutTex) {
      gl.activeTexture(gl.TEXTURE5);
      gl.bindTexture(gl.TEXTURE_3D, this.proofLutTex);
    }

    const modeMap: Record<string, number> = {
      AgxVivid: 0,
      AgxFilmic: 1,
      CameraStandard: 2,
      CameraPortrait: 3,
      CameraLandscape: 4,
      CameraVivid: 5,
      AgxPhotographic: 6,
    };
    const filterMap: Record<string, number> = {
      NeutralPanchromatic: 0,
      FilterGrade2: 0,
      YellowK2: 1,
      Orange16: 2,
      Red25A: 3,
      GreenX1: 4,
      Orthochromatic: 5,
    };

    const isStrippedCinema =
      edits.emulsion_archetype === 'RochesterCinema500' &&
      edits.remjet_backing === 'Stripped';
    let halationMode = 0;
    if (edits.emulsion_archetype === 'BaodingSilver100') {
      halationMode = 1;
    } else if (isStrippedCinema) {
      halationMode = 2;
    }

    const grainSize = edits.grain_size ?? edits.grain_scale ?? 1.0;
    const grainAmount = edits.grain_amount ?? 1.0;
    const hasMaster = isCurveActive(edits.curves?.master) ? 1 : 0;
    const hasRgb =
      isCurveActive(edits.curves?.red) ||
      isCurveActive(edits.curves?.green) ||
      isCurveActive(edits.curves?.blue)
        ? 1
        : 0;

    const f = this.uboFloatView;
    const i = this.uboIntView;

    f[0] = edits.exposure;
    f[1] = edits.temperature;
    f[2] = edits.tint;
    f[3] = edits.contrast;
    f[4] = edits.highlights;
    f[5] = edits.shadows;
    f[6] = edits.whites;
    f[7] = edits.blacks;
    f[8] = edits.vibrance;
    f[9] = edits.saturation;
    f[10] = edits.deep_blue;
    f[11] = edits.foliage;
    f[12] = ((effectiveEdits.crop_angle ?? 0.0) * Math.PI) / 180.0;
    f[13] = this.texWidth / Math.max(this.texHeight, 1);
    const lc = this.lensCoefficients;
    const lensCorrectionEnabled =
      (effectiveEdits.lens_correction_enabled ?? false) &&
      (lc?.has_distortion ?? false);
    i[14] = lensCorrectionEnabled ? 1 : 0;
    i[15] = lc?.has_distortion && lc.distortion_model === 'ptlens' ? 1 : 0;
    f[16] = lc?.has_distortion ? (lc.distortion[0] ?? 0.0) : 0.0;
    f[17] = lc?.has_distortion ? (lc.distortion[1] ?? 0.0) : 0.0;
    f[18] = lc?.has_distortion ? (lc.distortion[2] ?? 0.0) : 0.0;
    f[19] = lc?.has_distortion ? (lc.distortion[3] ?? 0.0) : 0.0;
    f[20] = isOriginal ? 1.0 : 0.0;
    i[21] = this.isLinearRec2020 ? 1 : 0;
    i[22] = modeMap[edits.tone_profile] ?? 0;
    i[23] = 0; // lutEnabled
    f[24] = 0.0; // lutIntensity
    f[25] = 0.0; // lutSize
    f[26] = edits.sharpening_amount ?? 0.0;
    f[27] = edits.sharpening_radius ?? 1.0;
    f[28] = edits.sharpening_masking ?? 0.0;
    f[29] = edits.noise_reduction ?? 0.0;
    f[30] = this.baselineSharpness;
    f[31] = effectiveEdits.crop_left ?? 0.0;
    f[32] = effectiveEdits.crop_top ?? 0.0;
    f[33] = effectiveEdits.crop_right ?? 1.0;
    f[34] = effectiveEdits.crop_bottom ?? 1.0;
    f[35] = edits.perspective_vertical ?? 0.0;
    f[36] = edits.perspective_horizontal ?? 0.0;
    i[37] = effectiveEdits.crop_constrain ? 1 : 0;
    i[38] = cropModeActive ? 1 : 0;
    i[39] = hasMaster || hasRgb ? 1 : 0;
    i[40] = hasMaster;
    i[41] = hasRgb;
    f[42] = 1.0; // edrHeadroom
    i[43] = 0; // edrEnabled
    i[44] = 0; // hasGainMap
    f[45] = 0.0; // gainMapMin
    f[46] = 1.0; // gainMapMax
    f[47] = 1.0; // gainMapGamma
    f[48] = 1.0; // gainMapCapacityMax
    i[49] = EMULSION_MAP[edits.emulsion_archetype] ?? 0;
    i[50] = 0; // emulsionFormat
    f[51] = edits.emulsion_exposure;
    i[52] = GRAIN_MODE_MAP[edits.emulsion_grain_mode] ?? 0;
    f[53] = grainSize;
    f[54] = grainSize;
    f[55] = grainAmount;
    i[56] = this.isIntermediate ? 1 : 0;
    f[57] = edits.split_toning_shadow_hue;
    f[58] = edits.split_toning_shadow_sat;
    f[59] = edits.split_toning_highlight_hue;
    f[60] = edits.split_toning_highlight_sat;
    f[61] = edits.split_toning_balance;
    i[62] = PRINT_MEDIA_MAP[edits.print_media] ?? 0;
    i[63] = filterMap[edits.enlarger_filter] ?? 0;
    f[64] = edits.print_paper_grade;
    f[65] = 0.0; // printBaseWarmth
    f[66] = edits.print_process_depth ?? 100.0;
    i[67] = halationMode;
    i[68] = (((effectiveEdits.rotation ?? 0) % 360) + 360) % 360;
    f[69] = 5500.0; // asShotTemperature
    f[70] = 0.0; // asShotTint
    const hasTca = lc?.has_tca ?? false;
    i[71] = hasTca ? 1 : 0;
    f[72] = hasTca ? (lc?.tca[0] ?? 1.0) : 1.0; // tcaVr
    f[73] = hasTca ? (lc?.tca[1] ?? 1.0) : 1.0; // tcaVb
    f[74] = hasTca ? (lc?.tca[2] ?? 0.0) : 0.0; // tcaBr
    f[75] = hasTca ? (lc?.tca[3] ?? 0.0) : 0.0; // tcaBb
    f[76] = hasTca ? (lc?.tca[4] ?? 0.0) : 0.0; // tcaCr
    f[77] = hasTca ? (lc?.tca[5] ?? 0.0) : 0.0; // tcaCb
    f[78] = edits.defringe_amount ?? 0.0;
    i[79] = (effectiveEdits.tca_correction_enabled ?? true) ? 1 : 0;
    i[80] = this.clippingOverlayActive ? 1 : 0;
    i[81] = (effectiveEdits.vignetting_correction_enabled ?? false) ? 1 : 0;
    i[82] = (lc?.has_vignetting ?? false) ? 1 : 0;
    f[83] = lc?.has_vignetting ? (lc.vignetting[0] ?? 0.0) : 0.0; // vignettingK1
    f[84] = lc?.has_vignetting ? (lc.vignetting[1] ?? 0.0) : 0.0; // vignettingK2
    f[85] = lc?.has_vignetting ? (lc.vignetting[2] ?? 0.0) : 0.0; // vignettingK3
    f[86] = 0.0; // vignettingK4
    f[87] = 0.0; // pad0

    if (this.uboBuffer) {
      gl.bindBuffer(gl.UNIFORM_BUFFER, this.uboBuffer);
      gl.bufferSubData(gl.UNIFORM_BUFFER, 0, this.uboFloatView);
    }

    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
  }

  public getCanvas(): HTMLCanvasElement {
    return this.gl.canvas as HTMLCanvasElement;
  }

  public readPixels(): Uint8Array {
    const gl = this.gl;
    const pixels = new Uint8Array(gl.canvas.width * gl.canvas.height * 4);
    gl.readPixels(
      0,
      0,
      gl.canvas.width,
      gl.canvas.height,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      pixels,
    );
    return pixels;
  }

  public dispose(): void {
    if (this.vbo) {
      this.gl.deleteBuffer(this.vbo);
      this.vbo = null;
    }
    if (this.texture) {
      this.gl.deleteTexture(this.texture);
      this.texture = null;
    }
    if (this.masterLutTex) {
      this.gl.deleteTexture(this.masterLutTex);
      this.masterLutTex = null;
    }
    if (this.rgbLutTex) {
      this.gl.deleteTexture(this.rgbLutTex);
      this.rgbLutTex = null;
    }
    if (this.proofLutTex) {
      this.gl.deleteTexture(this.proofLutTex);
      this.proofLutTex = null;
    }
    if (this.uboBuffer) {
      this.gl.deleteBuffer(this.uboBuffer);
      this.uboBuffer = null;
    }
  }

  public static createOffscreen(width: number, height: number): WebGLRenderer {
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    return new WebGLRenderer(canvas);
  }
}
