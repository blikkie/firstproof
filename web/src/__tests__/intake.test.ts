import { describe, expect, it } from 'vitest';
import { FolderIntake } from '../FolderIntake';

describe('Folder and Photo Intake Processing', () => {
  describe('isSupportedExtension', () => {
    it('accepts standard RAW camera extensions case-insensitively', () => {
      const valid = [
        'photo.dng',
        'PHOTO.DNG',
        'DSC_0001.NEF',
        '_MG_1234.CR2',
        '_MG_5678.CR3',
        'image.ARW',
        'test.orf',
        'test.rw2',
        'test.pef',
        'test.raf',
      ];
      for (const f of valid) {
        expect(FolderIntake.isSupportedExtension(f)).toBe(true);
      }
    });

    it('accepts standard raster image formats', () => {
      expect(FolderIntake.isSupportedExtension('photo.jpg')).toBe(true);
      expect(FolderIntake.isSupportedExtension('photo.JPEG')).toBe(true);
      expect(FolderIntake.isSupportedExtension('render.png')).toBe(true);
      expect(FolderIntake.isSupportedExtension('web.webp')).toBe(true);
      expect(FolderIntake.isSupportedExtension('web.WEBP')).toBe(true);
      expect(FolderIntake.isSupportedExtension('scan.tif')).toBe(true);
      expect(FolderIntake.isSupportedExtension('scan.tiff')).toBe(true);
    });

    it('distinguishes RAW camera captures from raster formats via isRawExtension', () => {
      expect(FolderIntake.isRawExtension('photo.dng')).toBe(true);
      expect(FolderIntake.isRawExtension('capture.CR3')).toBe(true);
      expect(FolderIntake.isRawExtension('legacy.raw')).toBe(true);
      expect(FolderIntake.isRawExtension('image.ARW')).toBe(true);

      expect(FolderIntake.isRawExtension('photo.png')).toBe(false);
      expect(FolderIntake.isRawExtension('photo.jpg')).toBe(false);
      expect(FolderIntake.isRawExtension('photo.jpeg')).toBe(false);
      expect(FolderIntake.isRawExtension('graphic.webp')).toBe(false);
      expect(FolderIntake.isRawExtension('scan.tiff')).toBe(false);
    });

    it('rejects unsupported extensions', () => {
      expect(FolderIntake.isSupportedExtension('video.mp4')).toBe(false);
      expect(FolderIntake.isSupportedExtension('metadata.xmp')).toBe(false);
      expect(FolderIntake.isSupportedExtension('document.pdf')).toBe(false);
      expect(FolderIntake.isSupportedExtension('no_ext')).toBe(false);
    });
  });

  describe('processFileList', () => {
    it('processes FileList and filters out unsupported files', () => {
      const mockFiles = [
        new File(['data'], 'IMG_0002.DNG', { type: 'image/x-adobe-dng' }),
        new File(['data'], 'notes.txt', { type: 'text/plain' }),
        new File(['data'], 'IMG_0001.JPG', { type: 'image/jpeg' }),
      ];

      // Create a mock FileList object
      const fileList = {
        0: mockFiles[0],
        1: mockFiles[1],
        2: mockFiles[2],
        length: 3,
        item: (index: number) => mockFiles[index] || null,
        [Symbol.iterator]: function* () {
          yield* mockFiles;
        },
      } as unknown as FileList;

      const entries = FolderIntake.processFileList(fileList);

      // Only supported files remain, sorted numerically / naturally by name
      expect(entries).toHaveLength(2);
      expect(entries[0].name).toBe('IMG_0001.JPG');
      expect(entries[1].name).toBe('IMG_0002.DNG');
      expect(entries[0].dirHandle).toBeUndefined();
    });
  });
});
