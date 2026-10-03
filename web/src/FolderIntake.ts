export interface PhotoFileEntry {
  name: string;
  file: File;
  handle?: any; // FileSystemFileHandle if showDirectoryPicker used
  dirHandle?: any; // FileSystemDirectoryHandle
}

export class FolderIntake {
  private static RAW_EXTENSIONS = new Set([
    'cr2',
    'cr3',
    'nef',
    'arw',
    'dng',
    'rw2',
    'orf',
    'pef',
    'raf',
    'jpg',
    'jpeg',
  ]);

  public static isSupportedExtension(filename: string): boolean {
    const ext = filename.split('.').pop()?.toLowerCase() || '';
    return FolderIntake.RAW_EXTENSIONS.has(ext);
  }

  private static IGNORED_DIRS = new Set([
    'node_modules',
    '.git',
    '__MACOSX',
    '$RECYCLE.BIN',
    'target',
    'dist',
    'pkg',
  ]);

  private static async scanDirectoryHandle(
    dirHandle: any,
    rootDirHandle?: any,
    depth = 0,
  ): Promise<PhotoFileEntry[]> {
    const entries: PhotoFileEntry[] = [];
    if (depth > 8) return entries;

    const parentDir = rootDirHandle || dirHandle;
    for await (const entry of dirHandle.values()) {
      if (
        entry.kind === 'file' &&
        FolderIntake.isSupportedExtension(entry.name)
      ) {
        try {
          const file = await entry.getFile();
          entries.push({
            name: entry.name,
            file,
            handle: entry,
            dirHandle: parentDir,
          });
        } catch (e) {
          console.warn(`[FolderIntake] Could not read file ${entry.name}:`, e);
        }
      } else if (
        entry.kind === 'directory' &&
        !entry.name.startsWith('.') &&
        !FolderIntake.IGNORED_DIRS.has(entry.name)
      ) {
        try {
          const subEntries = await FolderIntake.scanDirectoryHandle(
            entry,
            parentDir,
            depth + 1,
          );
          entries.push(...subEntries);
        } catch (e) {
          console.warn(
            `[FolderIntake] Could not scan directory ${entry.name}:`,
            e,
          );
        }
      }
    }
    return entries;
  }

  // Tier 1: Modern File System Access API
  public static async openDirectoryPicker(): Promise<{
    entries: PhotoFileEntry[];
    dirHandle?: any;
    cancelled?: boolean;
  }> {
    const isSupported = 'showDirectoryPicker' in window;
    console.log(
      `[FolderIntake] openDirectoryPicker called (showDirectoryPicker supported: ${isSupported})`,
    );

    if (isSupported) {
      try {
        const dirHandle = await (window as any).showDirectoryPicker();
        const entries = await FolderIntake.scanDirectoryHandle(dirHandle);

        // Sort by name
        entries.sort((a, b) =>
          a.name.localeCompare(b.name, undefined, {
            numeric: true,
            sensitivity: 'base',
          }),
        );
        console.log(
          `[FolderIntake] Loaded ${entries.length} photos from directory: ${dirHandle.name}`,
        );
        await FolderIntake.saveRecentDirHandle(dirHandle);
        return { entries, dirHandle, cancelled: false };
      } catch (err: any) {
        if (err.name === 'AbortError') {
          return { entries: [], cancelled: true };
        }
        console.warn('[FolderIntake] showDirectoryPicker failed:', err);
        return { entries: [], cancelled: false };
      }
    }
    return { entries: [], cancelled: false };
  }

  // IndexedDB Store for persisting FileSystemDirectoryHandle across browser restarts
  private static async getIDB(): Promise<IDBDatabase> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open('firstproof_db', 2);
      request.onupgradeneeded = () => {
        const db = request.result;
        if (!db.objectStoreNames.contains('handles')) {
          db.createObjectStore('handles', { keyPath: 'id' });
        }
      };
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }

  public static async saveRecentDirHandle(dirHandle: any): Promise<void> {
    try {
      if (dirHandle && dirHandle.name) {
        localStorage.setItem('fp_recent_folder_name', dirHandle.name);
      }
      const db = await FolderIntake.getIDB();
      const tx = db.transaction('handles', 'readwrite');
      const store = tx.objectStore('handles');
      const req = store.put({
        id: 'recent_dir',
        handle: dirHandle,
        name: dirHandle.name,
      });
      req.onsuccess = () =>
        console.log(
          `[FolderIntake] Persisted recent directory handle to IndexedDB: ${dirHandle.name}`,
        );
      req.onerror = (e) =>
        console.warn('[FolderIntake] Failed to persist directory handle:', e);
    } catch (err) {
      console.warn('[FolderIntake] IndexedDB save error:', err);
    }
  }

  public static async getRecentDirHandle(): Promise<{
    dirHandle: any;
    name: string;
  } | null> {
    try {
      const storedName =
        localStorage.getItem('fp_recent_folder_name') || 'shoot';
      const db = await FolderIntake.getIDB();
      return new Promise((resolve) => {
        const tx = db.transaction('handles', 'readonly');
        const req = tx.objectStore('handles').get('recent_dir');
        req.onsuccess = () => {
          if (req.result && req.result.handle) {
            resolve({
              dirHandle: req.result.handle,
              name: req.result.name || storedName,
            });
          } else {
            resolve(null);
          }
        };
        req.onerror = () => resolve(null);
      });
    } catch (err) {
      console.warn('[FolderIntake] IndexedDB get error:', err);
      return null;
    }
  }

  public static async processDirectoryHandle(
    dirHandle: any,
  ): Promise<PhotoFileEntry[]> {
    if (dirHandle.queryPermission) {
      let perm = await dirHandle.queryPermission({ mode: 'readwrite' });
      if (perm !== 'granted') {
        perm = await dirHandle.requestPermission({ mode: 'readwrite' });
      }
      if (perm !== 'granted') {
        perm = await dirHandle.requestPermission({ mode: 'read' });
      }
    }

    const entries = await FolderIntake.scanDirectoryHandle(dirHandle);
    entries.sort((a, b) =>
      a.name.localeCompare(b.name, undefined, {
        numeric: true,
        sensitivity: 'base',
      }),
    );
    return entries;
  }

  // Tier 2: HTML5 File Input (webkitdirectory fallback)
  public static processFileList(fileList: FileList): PhotoFileEntry[] {
    const entries: PhotoFileEntry[] = [];
    for (let i = 0; i < fileList.length; i++) {
      const file = fileList[i];
      if (FolderIntake.isSupportedExtension(file.name)) {
        entries.push({
          name: file.name,
          file,
        });
      }
    }
    entries.sort((a, b) =>
      a.name.localeCompare(b.name, undefined, {
        numeric: true,
        sensitivity: 'base',
      }),
    );
    return entries;
  }

  // Direct bi-directional write-back of .xmp sidecars to disk (Tier 1)
  public static async writeSidecarToDisk(
    entry: PhotoFileEntry,
    xmlContent: string,
  ): Promise<boolean> {
    if (entry.dirHandle) {
      try {
        if (entry.dirHandle.queryPermission) {
          const perm = await entry.dirHandle.queryPermission({
            mode: 'readwrite',
          });
          if (perm !== 'granted') {
            const req = await entry.dirHandle.requestPermission({
              mode: 'readwrite',
            });
            if (req !== 'granted') {
              console.warn(
                '[FolderIntake] Write permission for directory denied by user',
              );
              return false;
            }
          }
        }
        // Write stem-based XMP sidecar (e.g., _MG_2231.xmp)
        const stem =
          entry.name.substring(0, entry.name.lastIndexOf('.')) || entry.name;
        const sidecarName = `${stem}.xmp`;
        const fileHandle = await entry.dirHandle.getFileHandle(sidecarName, {
          create: true,
        });
        const writable = await fileHandle.createWritable();
        await writable.write(xmlContent);
        await writable.close();
        console.log(`[FolderIntake] Wrote sidecar to disk: ${sidecarName}`);
        return true;
      } catch (err) {
        console.error(
          `[FolderIntake] Failed to write sidecar for ${entry.name}:`,
          err,
        );
      }
    }
    return false;
  }

  // Read adjacent .xmp sidecar if exists (Tier 1)
  public static async readSidecarFromDisk(
    entry: PhotoFileEntry,
  ): Promise<string | null> {
    if (entry.dirHandle) {
      const stem =
        entry.name.substring(0, entry.name.lastIndexOf('.')) || entry.name;
      const candidates = [`${stem}.xmp`, `${entry.name}.xmp`];

      for (const sidecarName of candidates) {
        try {
          const fileHandle = await entry.dirHandle.getFileHandle(sidecarName);
          const file = await fileHandle.getFile();
          const text = await file.text();
          if (text) return text;
        } catch (_) {
          // Try next candidate
        }
      }
    }
    return null;
  }
}
