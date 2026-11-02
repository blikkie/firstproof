import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  loadSampleShoot,
  resetSamplePhotoToShootDefaults,
  SAMPLE_PHOTOS,
} from '../services/SamplePhotosService';

describe('Sample Photos Service (Sample Challenge Shoot)', () => {
  const store = new Map<string, string>();
  const mockLocalStorage = {
    getItem: (key: string) => store.get(key) ?? null,
    setItem: (key: string, val: string) => {
      store.set(key, String(val));
    },
    removeItem: (key: string) => {
      store.delete(key);
    },
    clear: () => {
      store.clear();
    },
  };

  beforeEach(() => {
    Object.defineProperty(globalThis, 'localStorage', {
      value: mockLocalStorage,
      configurable: true,
      writable: true,
    });
    mockLocalStorage.clear();
    vi.restoreAllMocks();
  });

  it('defines the 4 privacy-safe lab challenge scenes', () => {
    expect(SAMPLE_PHOTOS).toHaveLength(4);
    const filenames = SAMPLE_PHOTOS.map((s) => s.filename);
    expect(filenames).toEqual([
      '_MG_1304.dng',
      '_MG_1422.dng',
      'IMG_0730.dng',
      '20250409_0017.dng',
    ]);

    // Invariant: Friends & family portrait files must never be bundled into public sample shoot
    for (const s of SAMPLE_PHOTOS) {
      expect(s.filename).not.toMatch(/20250409_0282/);
    }
  });

  it('downloads sample photos, seeds XMP into localStorage, and reports progress', async () => {
    const mockXmp =
      '<x:xmpmeta><rdf:RDF><rdf:Description/></rdf:RDF></x:xmpmeta>';
    const mockDngBytes = new Uint8Array([0x49, 0x49, 0x2a, 0x00]); // TIFF header

    const fetchSpy = vi
      .spyOn(globalThis, 'fetch')
      .mockImplementation(async (input) => {
        const url = String(input);
        if (url.endsWith('.xmp')) {
          return new Response(mockXmp, {
            status: 200,
            headers: { 'Content-Type': 'application/xml' },
          });
        }
        if (url.endsWith('.dng')) {
          return new Response(mockDngBytes, {
            status: 200,
            headers: {
              'Content-Type': 'image/x-adobe-dng',
              'Content-Length': `${mockDngBytes.byteLength}`,
            },
          });
        }
        return new Response('Not found', { status: 404 });
      });

    const progressCalls: { current: number; total: number; name: string }[] =
      [];
    const entries = await loadSampleShoot((current, total, name) => {
      progressCalls.push({ current, total, name });
    });

    expect(fetchSpy).toHaveBeenCalled();
    expect(entries).toHaveLength(4);
    expect(entries[0].name).toBe('_MG_1304.dng');
    expect(entries[0].file.type).toBe('image/x-adobe-dng');
    expect(progressCalls.length).toBeGreaterThanOrEqual(4);

    // Verify XMP was seeded into localStorage
    expect(localStorage.getItem('fp_xmp__MG_1304.dng')).toBe(mockXmp);
    expect(localStorage.getItem('fp_xmp_20250409_0017.dng')).toBe(mockXmp);
  });

  it('preserves user modified XMP edits in localStorage without overwriting on subsequent loads', async () => {
    const existingCustomXmp =
      '<x:xmpmeta><rdf:Description><pm:exposure>1.500</pm:exposure></rdf:Description></x:xmpmeta>';
    localStorage.setItem('fp_xmp__MG_1304.dng', existingCustomXmp);

    const mockDngBytes = new Uint8Array([0x49, 0x49, 0x2a, 0x00]);
    vi.spyOn(globalThis, 'fetch').mockImplementation(async (input) => {
      const url = String(input);
      if (url.endsWith('.xmp')) {
        return new Response('<default_xmp/>', { status: 200 });
      }
      return new Response(mockDngBytes, { status: 200 });
    });

    await loadSampleShoot();

    // Must still retain user's custom edits
    expect(localStorage.getItem('fp_xmp__MG_1304.dng')).toBe(existingCustomXmp);
  });

  it('allows explicit reset to default shoot settings via resetSamplePhotoToShootDefaults', async () => {
    localStorage.setItem('fp_xmp__MG_1422.dng', '<custom_user_edits/>');

    const defaultShootXmp = '<shoot_original_xmp/>';
    vi.spyOn(globalThis, 'fetch').mockImplementation(async (input) => {
      const url = String(input);
      if (url.includes('_MG_1422.xmp')) {
        return new Response(defaultShootXmp, { status: 200 });
      }
      return new Response('Not found', { status: 404 });
    });

    const resetSuccess = await resetSamplePhotoToShootDefaults('_MG_1422.dng');
    expect(resetSuccess).toBe(true);
    expect(localStorage.getItem('fp_xmp__MG_1422.dng')).toBe(defaultShootXmp);

    // Non-existent sample photo should return false
    const unknownResult = await resetSamplePhotoToShootDefaults('unknown.dng');
    expect(unknownResult).toBe(false);
  });
});
