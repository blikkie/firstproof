use rayon::prelude::*;
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;
use walkdir::WalkDir;

use crate::raw;
use crate::takeout;
use crate::xmp;

#[derive(Debug, Clone, PartialEq)]
pub struct PhotoRecord {
    pub id: i64,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub modified_time: i64,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub iso: Option<u32>,
    pub aperture: Option<f32>,
    pub shutter_speed: Option<String>,
    pub focal_length: Option<f32>,
    pub taken_time: Option<i64>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub creator: Option<String>,
    pub copyright: Option<String>,
    pub keywords: Option<String>,
    pub rating: Option<i32>,
    pub status: String, // "Pick", "Reject", "None"
    pub has_xmp: bool,
    pub has_takeout: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Opens or creates a SQLite database at the specified path.
    pub fn open<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_path = db_path.as_ref();
        if let Some(parent) = db_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let open_result = (|| -> Result<Connection> {
            let conn = Connection::open(db_path)?;
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA foreign_keys = ON;",
            )?;
            // Quick integrity check: verify B-tree and page allocation integrity
            {
                let mut stmt = conn.prepare("PRAGMA quick_check(1);")?;
                let status: String = stmt.query_row([], |row| row.get(0))?;
                if status != "ok" {
                    return Err(rusqlite::Error::SqliteFailure(
                        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CORRUPT),
                        Some(format!("Database corruption detected: {}", status)),
                    ));
                }
            }
            Ok(conn)
        })();

        let conn = match open_result {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[FirstProof] Database at {} corrupted or unreadable ({:?}). Self-healing with fresh index...", db_path.display(), e);
                // Self-healing: backup corrupt db and recreate clean database
                let corrupt_backup = db_path.with_extension("db.corrupt");
                let _ = fs::rename(db_path, corrupt_backup);
                let _ = fs::remove_file(format!("{}-wal", db_path.display()));
                let _ = fs::remove_file(format!("{}-shm", db_path.display()));

                let conn = Connection::open(db_path)?;
                conn.execute_batch(
                    "PRAGMA journal_mode = WAL;
                     PRAGMA synchronous = NORMAL;
                     PRAGMA foreign_keys = ON;",
                )?;
                conn
            }
        };

        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;

        // If a legacy LumenBox catalog exists, migrate records safely using pure SQL ATTACH
        let legacy_db = db_path
            .parent()
            .and_then(|p| p.parent())
            .map(|root| root.join(".lumenbox").join("catalog.db"));
        if let Some(legacy_path) = legacy_db {
            if legacy_path.exists() {
                let conn = db.conn.lock().unwrap();
                let legacy_str = legacy_path.to_string_lossy();
                if conn.execute("ATTACH DATABASE ? AS legacy;", [&*legacy_str]).is_ok() {
                    let _ = conn.execute(
                        "INSERT OR IGNORE INTO photos (
                            file_path, file_name, file_size, modified_time,
                            camera_make, camera_model, lens_model, iso, aperture, shutter_speed, focal_length,
                            taken_time, latitude, longitude, title, description, creator, copyright, keywords, rating, status,
                            has_xmp, has_takeout, created_at, updated_at
                        ) SELECT 
                            file_path, file_name, file_size, modified_time,
                            camera_make, camera_model, lens_model, iso, aperture, shutter_speed, focal_length,
                            taken_time, latitude, longitude, title, description, creator, copyright, keywords, rating, status,
                            has_xmp, has_takeout, created_at, updated_at
                        FROM legacy.photos;",
                        [],
                    );
                    let _ = conn.execute("DETACH DATABASE legacy;", []);
                }
                let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
                drop(conn);
                if let Some(legacy_dir) = legacy_path.parent() {
                    let _ = fs::remove_dir_all(legacy_dir);
                }
            }
        }

        Ok(db)
    }

    /// Checkpoints all WAL journal pages into the main SQLite database and truncates WAL log.
    pub fn checkpoint_and_close(&self) {
        if let Ok(conn) = self.conn.lock() {
            let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
        }
    }

    /// Creates in-memory database for testing
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS photos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL UNIQUE,
                file_name TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                modified_time INTEGER NOT NULL,
                camera_make TEXT,
                camera_model TEXT,
                lens_model TEXT,
                iso INTEGER,
                aperture REAL,
                shutter_speed TEXT,
                focal_length REAL,
                taken_time INTEGER,
                latitude REAL,
                longitude REAL,
                title TEXT,
                description TEXT,
                creator TEXT,
                copyright TEXT,
                keywords TEXT,
                rating INTEGER,
                status TEXT NOT NULL DEFAULT 'None',
                has_xmp INTEGER NOT NULL DEFAULT 0,
                has_takeout INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_photos_taken_time ON photos(taken_time);
            CREATE INDEX IF NOT EXISTS idx_photos_rating ON photos(rating);
            CREATE INDEX IF NOT EXISTS idx_photos_status ON photos(status);
            CREATE INDEX IF NOT EXISTS idx_photos_file_path ON photos(file_path);",
        )?;

        // Execute schema migrations if existing catalog database does not have new columns
        let _ = conn.execute("ALTER TABLE photos ADD COLUMN title TEXT;", []);
        let _ = conn.execute("ALTER TABLE photos ADD COLUMN creator TEXT;", []);
        let _ = conn.execute("ALTER TABLE photos ADD COLUMN copyright TEXT;", []);
        let _ = conn.execute("ALTER TABLE photos ADD COLUMN keywords TEXT;", []);

        Ok(())
    }

    /// Helper to resolve the database path for a catalog.
    /// Every session or library folder has its own isolated, self-contained database at `<folder>/.firstproof/catalog.db`.
    pub fn resolve_catalog_path<P: AsRef<Path>>(folder_path: P, _portable: bool) -> PathBuf {
        let folder = folder_path.as_ref();
        folder.join(".firstproof").join("catalog.db")
    }

    /// Indexes a folder recursively, inserting/updating records in SQLite with parallel CPU processing.
    pub fn index_folder<P: AsRef<Path>>(&self, folder_path: P) -> Result<u32> {
        let folder_path = folder_path.as_ref();
        let folder_prefix = folder_path.to_string_lossy().to_string();
        let cache_map = self.preload_cache(&folder_prefix);

        // 1. Fast directory walk: collect all RAW and image file paths
        let mut candidates = Vec::new();
        for entry in WalkDir::new(folder_path)
            .follow_links(false)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if matches!(
                        ext_lower.as_str(),
                        "cr2" | "rw2" | "dng" | "jpg" | "jpeg" | "png" | "heic" | "arw" | "nef" | "tiff" | "tif"
                    ) {
                        candidates.push(path.to_path_buf());
                    }
                }
            }
        }

        let total_candidates = candidates.len() as u32;
        let now = current_timestamp();

        // 2. Filter out already-cached unmodified files
        let files_to_process: Vec<(PathBuf, i64, i64)> = candidates
            .into_iter()
            .filter_map(|path| {
                let metadata = fs::metadata(&path).ok()?;
                let file_path_str = path.to_string_lossy().to_string();
                let file_size = metadata.len() as i64;
                let modified_time = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);

                if let Some(&(cached_mtime, cached_size)) = cache_map.get(&file_path_str) {
                    if cached_mtime == modified_time && cached_size == file_size {
                        return None; // Unmodified, skip!
                    }
                }
                Some((path, file_size, modified_time))
            })
            .collect();

        if files_to_process.is_empty() {
            return Ok(total_candidates);
        }

        // 3. Parallel metadata extraction across all CPU cores with Rayon
        let records: Vec<PhotoRecord> = files_to_process
            .into_par_iter()
            .map(|(path, file_size, modified_time)| {
                let file_path_str = path.to_string_lossy().to_string();
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                // 1. RAW EXIF Extraction
                let raw_meta = raw::extract_metadata(&file_path_str).ok();
                let camera_make = raw_meta.as_ref().and_then(|m| m.camera_make.clone());
                let camera_model = raw_meta.as_ref().and_then(|m| m.camera_model.clone());
                let lens_model = raw_meta.as_ref().and_then(|m| m.lens_model.clone());
                let iso = raw_meta.as_ref().and_then(|m| m.iso);
                let aperture = raw_meta.as_ref().and_then(|m| m.aperture);
                let shutter_speed = raw_meta.as_ref().and_then(|m| m.shutter_speed.clone());
                let focal_length = raw_meta.as_ref().and_then(|m| m.focal_length);
                let mut taken_time = raw_meta.as_ref().and_then(|m| m.taken_time);
                let mut latitude = raw_meta.as_ref().and_then(|m| m.latitude);
                let mut longitude = raw_meta.as_ref().and_then(|m| m.longitude);
                let mut description = raw_meta.as_ref().and_then(|m| m.description.clone());
                let mut creator = raw_meta.as_ref().and_then(|m| m.creator.clone());
                let mut copyright = raw_meta.as_ref().and_then(|m| m.copyright.clone());
                let mut title = None;
                let mut keywords = None;

                // 2. XMP Sidecar Integration (Support both Lightroom foo.jpg.xmp and standard foo.xmp)
                let xmp_path1 = PathBuf::from(format!("{}.xmp", file_path_str));
                let xmp_path2 = path.with_extension("xmp");
                let target_xmp = if xmp_path1.exists() {
                    Some(xmp_path1)
                } else if xmp_path2.exists() {
                    Some(xmp_path2)
                } else {
                    None
                };

                let mut has_xmp = false;
                let mut rating = None;
                let mut status = "None".to_string();

                if let Some(xmp_path) = target_xmp {
                    if let Ok(xmp_data) = xmp::read_xmp_file(&xmp_path) {
                        has_xmp = true;
                        if xmp_data.rating.is_some() {
                            rating = xmp_data.rating;
                        }
                        status = xmp_data.status;
                        if xmp_data.latitude.is_some() {
                            latitude = xmp_data.latitude;
                        }
                        if xmp_data.longitude.is_some() {
                            longitude = xmp_data.longitude;
                        }
                        if xmp_data.title.is_some() {
                            title = xmp_data.title;
                        }
                        if xmp_data.description.is_some() {
                            description = xmp_data.description;
                        }
                        if xmp_data.creator.is_some() {
                            creator = xmp_data.creator;
                        }
                        if xmp_data.copyright.is_some() {
                            copyright = xmp_data.copyright;
                        }
                        if !xmp_data.keywords.is_empty() {
                            keywords = Some(xmp_data.keywords.join(", "));
                        }
                    }
                }

                // 3. Google Takeout JSON Sidecar Integration
                let mut has_takeout = false;
                if let Some(json_path) = takeout::match_takeout_json(&path) {
                    if let Ok(takeout_meta) = takeout::parse_takeout_json(&json_path) {
                        has_takeout = true;
                        if latitude.is_none() {
                            latitude = takeout_meta.latitude;
                        }
                        if longitude.is_none() {
                            longitude = takeout_meta.longitude;
                        }
                        if taken_time.is_none() {
                            taken_time = takeout_meta.taken_time;
                        }
                        if description.is_none() {
                            description = takeout_meta.description;
                        }
                    }
                }

                PhotoRecord {
                    id: 0,
                    file_path: file_path_str,
                    file_name,
                    file_size,
                    modified_time,
                    camera_make,
                    camera_model,
                    lens_model,
                    iso,
                    aperture,
                    shutter_speed,
                    focal_length,
                    taken_time,
                    latitude,
                    longitude,
                    title,
                    description,
                    creator,
                    copyright,
                    keywords,
                    rating,
                    status,
                    has_xmp,
                    has_takeout,
                    created_at: now,
                    updated_at: now,
                }
            })
            .collect();

        // 4. Batched SQLite insert
        for chunk in records.chunks(300) {
            self.flush_batch(chunk)?;
        }

        Ok(total_candidates)
    }

    fn preload_cache(&self, folder_prefix: &str) -> std::collections::HashMap<String, (i64, i64)> {
        let mut map = std::collections::HashMap::new();
        let conn = self.conn.lock().unwrap();
        let query = "SELECT file_path, modified_time, file_size FROM photos WHERE file_path LIKE ?1 || '%'";
        if let Ok(mut stmt) = conn.prepare(query) {
            let res = stmt.query_map([folder_prefix], |row| {
                Ok((row.get::<_, String>(0)?, (row.get::<_, i64>(1)?, row.get::<_, i64>(2)?)))
            });
            if let Ok(rows) = res {
                for r in rows.flatten() {
                    map.insert(r.0, r.1);
                }
            }
        }
        map
    }

    fn flush_batch(&self, batch: &[PhotoRecord]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO photos (
                    file_path, file_name, file_size, modified_time,
                    camera_make, camera_model, lens_model, iso, aperture, shutter_speed, focal_length,
                    taken_time, latitude, longitude, title, description, creator, copyright, keywords, rating, status,
                    has_xmp, has_takeout, created_at, updated_at
                ) VALUES (
                    ?1, ?2, ?3, ?4,
                    ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                    ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21,
                    ?22, ?23, ?24, ?25
                ) ON CONFLICT(file_path) DO UPDATE SET
                    file_name = excluded.file_name,
                    file_size = excluded.file_size,
                    modified_time = excluded.modified_time,
                    camera_make = excluded.camera_make,
                    camera_model = excluded.camera_model,
                    lens_model = excluded.lens_model,
                    iso = excluded.iso,
                    aperture = excluded.aperture,
                    shutter_speed = excluded.shutter_speed,
                    focal_length = excluded.focal_length,
                    taken_time = excluded.taken_time,
                    latitude = excluded.latitude,
                    longitude = excluded.longitude,
                    title = excluded.title,
                    description = excluded.description,
                    creator = excluded.creator,
                    copyright = excluded.copyright,
                    keywords = excluded.keywords,
                    rating = excluded.rating,
                    status = excluded.status,
                    has_xmp = excluded.has_xmp,
                    has_takeout = excluded.has_takeout,
                    updated_at = excluded.updated_at;"
            )?;

            for p in batch {
                stmt.execute(rusqlite::params![
                    p.file_path,
                    p.file_name,
                    p.file_size,
                    p.modified_time,
                    p.camera_make,
                    p.camera_model,
                    p.lens_model,
                    p.iso,
                    p.aperture,
                    p.shutter_speed,
                    p.focal_length,
                    p.taken_time,
                    p.latitude,
                    p.longitude,
                    p.title,
                    p.description,
                    p.creator,
                    p.copyright,
                    p.keywords,
                    p.rating,
                    p.status,
                    p.has_xmp,
                    p.has_takeout,
                    p.created_at,
                    p.updated_at,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn upsert_photo(&self, photo: &PhotoRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = current_timestamp();

        conn.execute(
            "INSERT INTO photos (
                file_path, file_name, file_size, modified_time,
                camera_make, camera_model, lens_model, iso, aperture, shutter_speed, focal_length,
                taken_time, latitude, longitude, title, description, creator, copyright, keywords, rating, status,
                has_xmp, has_takeout, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4,
                ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21,
                ?22, ?23, ?24, ?25
            ) ON CONFLICT(file_path) DO UPDATE SET
                file_name = excluded.file_name,
                file_size = excluded.file_size,
                modified_time = excluded.modified_time,
                camera_make = excluded.camera_make,
                camera_model = excluded.camera_model,
                lens_model = excluded.lens_model,
                iso = excluded.iso,
                aperture = excluded.aperture,
                shutter_speed = excluded.shutter_speed,
                focal_length = excluded.focal_length,
                taken_time = excluded.taken_time,
                latitude = excluded.latitude,
                longitude = excluded.longitude,
                title = COALESCE(excluded.title, photos.title),
                description = COALESCE(excluded.description, photos.description),
                creator = COALESCE(excluded.creator, photos.creator),
                copyright = COALESCE(excluded.copyright, photos.copyright),
                keywords = COALESCE(excluded.keywords, photos.keywords),
                rating = COALESCE(excluded.rating, photos.rating),
                status = CASE WHEN excluded.status != 'None' THEN excluded.status ELSE photos.status END,
                has_xmp = excluded.has_xmp,
                has_takeout = excluded.has_takeout,
                updated_at = excluded.updated_at;",
            params![
                photo.file_path,
                photo.file_name,
                photo.file_size,
                photo.modified_time,
                photo.camera_make,
                photo.camera_model,
                photo.lens_model,
                photo.iso,
                photo.aperture,
                photo.shutter_speed,
                photo.focal_length,
                photo.taken_time,
                photo.latitude,
                photo.longitude,
                photo.title,
                photo.description,
                photo.creator,
                photo.copyright,
                photo.keywords,
                photo.rating,
                photo.status,
                if photo.has_xmp { 1 } else { 0 },
                if photo.has_takeout { 1 } else { 0 },
                photo.created_at,
                now,
            ],
        )?;

        Ok(())
    }

    pub fn get_photo(&self, file_path: &str) -> Result<Option<PhotoRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, file_path, file_name, file_size, modified_time,
                    camera_make, camera_model, lens_model, iso, aperture, shutter_speed, focal_length,
                    taken_time, latitude, longitude, title, description, creator, copyright, keywords, rating, status,
                    has_xmp, has_takeout, created_at, updated_at
             FROM photos WHERE file_path = ?1",
        )?;

        let mut rows = stmt.query(params![file_path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(PhotoRecord {
                id: row.get(0)?,
                file_path: row.get(1)?,
                file_name: row.get(2)?,
                file_size: row.get(3)?,
                modified_time: row.get(4)?,
                camera_make: row.get(5)?,
                camera_model: row.get(6)?,
                lens_model: row.get(7)?,
                iso: row.get(8)?,
                aperture: row.get(9)?,
                shutter_speed: row.get(10)?,
                focal_length: row.get(11)?,
                taken_time: row.get(12)?,
                latitude: row.get(13)?,
                longitude: row.get(14)?,
                title: row.get(15)?,
                description: row.get(16)?,
                creator: row.get(17)?,
                copyright: row.get(18)?,
                keywords: row.get(19)?,
                rating: row.get(20)?,
                status: row.get(21)?,
                has_xmp: row.get::<_, i32>(22)? != 0,
                has_takeout: row.get::<_, i32>(23)? != 0,
                created_at: row.get(24)?,
                updated_at: row.get(25)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_photos(&self) -> Result<Vec<PhotoRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, file_path, file_name, file_size, modified_time,
                    camera_make, camera_model, lens_model, iso, aperture, shutter_speed, focal_length,
                    taken_time, latitude, longitude, title, description, creator, copyright, keywords, rating, status,
                    has_xmp, has_takeout, created_at, updated_at
             FROM photos ORDER BY taken_time DESC, file_name ASC",
        )?;

        let photo_iter = stmt.query_map([], |row| {
            Ok(PhotoRecord {
                id: row.get(0)?,
                file_path: row.get(1)?,
                file_name: row.get(2)?,
                file_size: row.get(3)?,
                modified_time: row.get(4)?,
                camera_make: row.get(5)?,
                camera_model: row.get(6)?,
                lens_model: row.get(7)?,
                iso: row.get(8)?,
                aperture: row.get(9)?,
                shutter_speed: row.get(10)?,
                focal_length: row.get(11)?,
                taken_time: row.get(12)?,
                latitude: row.get(13)?,
                longitude: row.get(14)?,
                title: row.get(15)?,
                description: row.get(16)?,
                creator: row.get(17)?,
                copyright: row.get(18)?,
                keywords: row.get(19)?,
                rating: row.get(20)?,
                status: row.get(21)?,
                has_xmp: row.get::<_, i32>(22)? != 0,
                has_takeout: row.get::<_, i32>(23)? != 0,
                created_at: row.get(24)?,
                updated_at: row.get(25)?,
            })
        })?;

        let mut photos = Vec::new();
        for p in photo_iter {
            photos.push(p?);
        }
        Ok(photos)
    }

    pub fn update_photo_metadata(
        &self,
        file_path: &str,
        title: Option<&str>,
        description: Option<&str>,
        creator: Option<&str>,
        copyright: Option<&str>,
        keywords: Option<&str>,
    ) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let now = current_timestamp();
        let updated = conn.execute(
            "UPDATE photos SET title = ?1, description = ?2, creator = ?3, copyright = ?4, keywords = ?5, updated_at = ?6 WHERE file_path = ?7",
            params![title, description, creator, copyright, keywords, now, file_path],
        )?;
        Ok(updated > 0)
    }

    pub fn update_rating_status(
        &self,
        file_path: &str,
        rating: Option<i32>,
        status: &str,
    ) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let now = current_timestamp();
        let updated = conn.execute(
            "UPDATE photos SET rating = ?1, status = ?2, updated_at = ?3 WHERE file_path = ?4",
            params![rating, status, now, file_path],
        )?;
        Ok(updated > 0)
    }

    /// Updates rating and/or status for multiple photos in a single fast SQLite transaction.
    pub fn update_rating_status_batch(
        &self,
        file_paths: &[String],
        rating: Option<i32>,
        update_rating: bool,
        status: Option<&str>,
    ) -> Result<u32> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let now = current_timestamp();
        let mut count = 0u32;
        {
            if update_rating {
                if let Some(st) = status {
                    let mut stmt = tx.prepare_cached(
                        "UPDATE photos SET rating = ?1, status = ?2, updated_at = ?3 WHERE file_path = ?4",
                    )?;
                    for p in file_paths {
                        count += stmt.execute(params![rating, st, now, p])? as u32;
                    }
                } else {
                    let mut stmt = tx.prepare_cached(
                        "UPDATE photos SET rating = ?1, updated_at = ?2 WHERE file_path = ?3",
                    )?;
                    for p in file_paths {
                        count += stmt.execute(params![rating, now, p])? as u32;
                    }
                }
            } else if let Some(st) = status {
                let mut stmt = tx.prepare_cached(
                    "UPDATE photos SET status = ?1, updated_at = ?2 WHERE file_path = ?3",
                )?;
                for p in file_paths {
                    count += stmt.execute(params![st, now, p])? as u32;
                }
            }
        }
        tx.commit()?;
        Ok(count)
    }

    pub fn delete_photo(&self, file_path: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute("DELETE FROM photos WHERE file_path = ?1", params![file_path])?;
        Ok(deleted > 0)
    }

    /// Exports a selection of photos to an isolated portable project package.
    /// Copies/moves RAW images, XMP sidecars, and Takeout JSON files, and builds a dedicated .firstproof/catalog.db.
    pub fn export_portable_package<P: AsRef<Path>>(
        &self,
        source_photo_paths: &[String],
        destination_folder: P,
        move_files: bool,
    ) -> std::result::Result<ExportPackageStats, String> {
        let dest_folder = destination_folder.as_ref();
        fs::create_dir_all(dest_folder).map_err(|e| e.to_string())?;

        let dest_catalog_path = Self::resolve_catalog_path(dest_folder, true);
        let dest_db = Self::open(&dest_catalog_path).map_err(|e| e.to_string())?;

        let mut exported_count = 0u32;
        let mut error_count = 0u32;
        let mut error_messages = Vec::new();

        for src_path_str in source_photo_paths {
            let src_path = Path::new(src_path_str);
            if !src_path.exists() {
                error_count += 1;
                error_messages.push(format!("Source file not found: {}", src_path_str));
                continue;
            }

            let file_name = match src_path.file_name() {
                Some(n) => n,
                None => {
                    error_count += 1;
                    error_messages.push(format!("Invalid file name for: {}", src_path_str));
                    continue;
                }
            };

            let dest_file_path = dest_folder.join(file_name);

            // 1. Transfer master image file
            if let Err(e) = transfer_file(src_path, &dest_file_path, move_files) {
                error_count += 1;
                error_messages.push(format!("Failed to transfer {}: {}", src_path_str, e));
                continue;
            }

            // 2. Transfer accompanying sidecars (XMP and Takeout)
            let sidecar_candidates = [
                // standard stem.xmp
                (src_path.with_extension("xmp"), dest_folder.join(format!("{}.xmp", src_path.file_stem().unwrap_or_default().to_string_lossy()))),
                // Lightroom filename.ext.xmp
                (PathBuf::from(format!("{}.xmp", src_path_str)), dest_folder.join(format!("{}.xmp", file_name.to_string_lossy()))),
            ];

            for (src_sidecar, dst_sidecar) in sidecar_candidates {
                if src_sidecar.exists() && src_sidecar != src_path {
                    let _ = transfer_file(&src_sidecar, &dst_sidecar, move_files);
                }
            }

            // Google Takeout sidecar
            if let Some(src_json) = crate::takeout::match_takeout_json(src_path) {
                if let Some(json_name) = src_json.file_name() {
                    let dst_json = dest_folder.join(json_name);
                    let _ = transfer_file(&src_json, &dst_json, move_files);
                }
            }

            // 3. Populate metadata in destination catalog
            if let Ok(Some(mut record)) = self.get_photo(src_path_str) {
                record.file_path = dest_file_path.to_string_lossy().to_string();
                record.file_name = file_name.to_string_lossy().to_string();
                record.updated_at = current_timestamp();
                let _ = dest_db.upsert_photo(&record);
            } else {
                // If not in current DB, index the new file directly
                let _ = dest_db.index_folder(dest_folder);
            }

            // 4. Remove from source DB if move_files requested
            if move_files {
                let _ = self.delete_photo(src_path_str);
            }

            exported_count += 1;
        }

        Ok(ExportPackageStats {
            total_requested: source_photo_paths.len() as u32,
            exported_count,
            error_count,
            destination_catalog_path: dest_catalog_path.to_string_lossy().to_string(),
            error_messages,
        })
    }

    /// Imports photos from source paths (or folders) into a target library folder.
    /// Implements deterministic deduplication (RAW over JPEG, active edits win, DNG preference),
    /// sidecar consolidation into a single First Proof XMP, and bit-level checksum verification before deletion.
    pub fn import_to_library<P: AsRef<Path>>(
        &self,
        source_paths_or_folders: &[String],
        destination_folder: P,
        options: &ImportOptions,
    ) -> std::result::Result<ImportReport, String> {
        let base_dest = destination_folder.as_ref();
        let dest_folder = if let Some(sub) = &options.subfolder_name {
            let trimmed = sub.trim();
            if !trimmed.is_empty() {
                base_dest.join(trimmed)
            } else {
                base_dest.to_path_buf()
            }
        } else {
            base_dest.to_path_buf()
        };
        fs::create_dir_all(&dest_folder).map_err(|e| e.to_string())?;

        let dest_catalog_path = Self::resolve_catalog_path(base_dest, false);
        let dest_db = Self::open(&dest_catalog_path).map_err(|e| e.to_string())?;

        // 1. Gather all candidate images
        let mut candidates: Vec<PathBuf> = Vec::new();
        for src_str in source_paths_or_folders {
            let p = Path::new(src_str);
            if p.is_dir() {
                for entry in WalkDir::new(p)
                    .follow_links(false)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let path = entry.path();
                    if path.is_file() && is_supported_image(path) {
                        candidates.push(path.to_path_buf());
                    }
                }
            } else if p.is_file() && is_supported_image(p) {
                candidates.push(p.to_path_buf());
            }
        }

        let total_found = candidates.len() as u32;
        let mut imported_count = 0u32;
        let mut skipped_duplicates = 0u32;
        let mut raw_over_jpeg_suppressed = 0u32;
        let mut consolidated_sidecars = 0u32;
        let mut error_count = 0u32;
        let mut error_messages = Vec::new();

        // 2. Group candidates by normalized stem and cluster into distinct captures
        use std::collections::BTreeMap;
        let mut stem_groups: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
        for c in candidates {
            let stem = c.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            let norm = normalize_stem(&stem);
            stem_groups.entry(norm).or_default().push(c);
        }

        let mut capture_groups: Vec<Vec<PathBuf>> = Vec::new();
        for (_stem, group) in stem_groups {
            let mut partitioned: Vec<Vec<PathBuf>> = Vec::new();
            for item in group {
                let mut matched = false;
                for cluster in &mut partitioned {
                    if cluster.iter().any(|existing| are_same_capture(existing, &item)) {
                        cluster.push(item.clone());
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    partitioned.push(vec![item]);
                }
            }
            capture_groups.extend(partitioned);
        }

        // 3. Deduplicate within capture groups according to precedence hierarchy
        let mut winning_candidates: Vec<(PathBuf, Vec<PathBuf>)> = Vec::new();
        for group in capture_groups {
            if group.len() == 1 {
                winning_candidates.push((group[0].clone(), Vec::new()));
                continue;
            }

            let mut raws = Vec::new();
            let mut non_raws = Vec::new();
            for item in &group {
                if is_raw_format(item) {
                    raws.push(item.clone());
                } else {
                    non_raws.push(item.clone());
                }
            }

            if options.deduplicate_raw_over_jpeg && !raws.is_empty() {
                raw_over_jpeg_suppressed += non_raws.len() as u32;
                // Rule 3: Active edits win
                let mut chosen = None;
                for r in &raws {
                    if let Ok(data) = xmp::io::read_xmp_file(r) {
                        if xmp::io::has_custom_adjustments(&data) {
                            chosen = Some(r.clone());
                            break;
                        }
                    }
                }
                // Rule 4: Prefer DNG
                if chosen.is_none() {
                    for r in &raws {
                        if r.extension().map(|e| e.to_string_lossy().eq_ignore_ascii_case("dng")).unwrap_or(false) {
                            chosen = Some(r.clone());
                            break;
                        }
                    }
                }
                // Rule 5: Deterministic tie-break
                let winner = chosen.unwrap_or_else(|| raws[0].clone());
                let siblings: Vec<PathBuf> = group.into_iter().filter(|p| p != &winner).collect();
                winning_candidates.push((winner, siblings));
            } else {
                let mut chosen = None;
                for item in non_raws.iter().chain(raws.iter()) {
                    if let Ok(data) = xmp::io::read_xmp_file(item) {
                        if xmp::io::has_custom_adjustments(&data) {
                            chosen = Some(item.clone());
                            break;
                        }
                    }
                }
                let winner = chosen.unwrap_or_else(|| {
                    if !raws.is_empty() { raws[0].clone() } else { non_raws[0].clone() }
                });
                let siblings: Vec<PathBuf> = group.into_iter().filter(|p| p != &winner).collect();
                winning_candidates.push((winner, siblings));
            }
        }

        // 4. Process each winning candidate
        let mut records_to_insert: Vec<PhotoRecord> = Vec::new();
        for (src_path, siblings) in winning_candidates {
            let file_name = match src_path.file_name() {
                Some(n) => n,
                None => {
                    error_count += 1;
                    error_messages.push(format!("Invalid file name: {}", src_path.display()));
                    continue;
                }
            };

            let dest_file_path = dest_folder.join(file_name);
            let src_path_str = src_path.to_string_lossy().to_string();

            // Check duplicate suppression
            if options.skip_duplicates && dest_file_path.exists() {
                if let (Ok(src_meta), Ok(dst_meta)) = (fs::metadata(&src_path), fs::metadata(&dest_file_path)) {
                    if src_meta.len() == dst_meta.len() {
                        skipped_duplicates += 1;
                        continue;
                    }
                }
            }

            // Consolidate or transfer sidecars
            if options.consolidate_sidecars {
                let mut xmp_data = xmp::io::read_xmp_file(&src_path).ok();
                if xmp_data.is_none() {
                    for sib in &siblings {
                        if let Ok(data) = xmp::io::read_xmp_file(sib) {
                            xmp_data = Some(data);
                            break;
                        }
                    }
                }
                if let Some(json_path) = takeout::match_takeout_json(&src_path) {
                    if let Ok(takeout_meta) = takeout::parse_takeout_json(&json_path) {
                        let mut data = xmp_data.unwrap_or_default();
                        if data.title.is_none() && !takeout_meta.title.is_empty() {
                            data.title = Some(takeout_meta.title);
                        }
                        if data.description.is_none() && takeout_meta.description.is_some() {
                            data.description = takeout_meta.description;
                        }
                        if data.latitude.is_none() && takeout_meta.latitude.is_some() {
                            data.latitude = takeout_meta.latitude;
                        }
                        if data.longitude.is_none() && takeout_meta.longitude.is_some() {
                            data.longitude = takeout_meta.longitude;
                        }
                        xmp_data = Some(data);
                    }
                }

                if let Some(data) = xmp_data {
                    let dst_xmp = dest_folder.join(format!("{}.xmp", src_path.file_stem().unwrap_or_default().to_string_lossy()));
                    let xml_str = xmp::generate_xmp(&data);
                    if let Err(e) = fs::write(&dst_xmp, xml_str) {
                        error_messages.push(format!("Failed to write consolidated XMP for {}: {}", src_path.display(), e));
                    } else {
                        consolidated_sidecars += 1;
                    }

                    // If moving, clean up disparate source sidecars
                    if options.move_files {
                        let sidecars = [
                            src_path.with_extension("xmp"),
                            PathBuf::from(format!("{}.xmp", src_path_str)),
                        ];
                        for s in &sidecars {
                            if s.exists() && s != &src_path {
                                let _ = fs::remove_file(s);
                            }
                        }
                        if let Some(json_path) = takeout::match_takeout_json(&src_path) {
                            let _ = fs::remove_file(json_path);
                        }
                        // Clean up suppressed siblings and their sidecars
                        for sib in &siblings {
                            let _ = fs::remove_file(sib);
                            let sib_str = sib.to_string_lossy().to_string();
                            let sib_sidecars = [
                                sib.with_extension("xmp"),
                                PathBuf::from(format!("{}.xmp", sib_str)),
                            ];
                            for s in &sib_sidecars {
                                if s.exists() && s != sib {
                                    let _ = fs::remove_file(s);
                                }
                            }
                            if let Some(json_path) = takeout::match_takeout_json(sib) {
                                let _ = fs::remove_file(json_path);
                            }
                        }
                    }
                }
            } else {
                let sidecar_candidates = [
                    (src_path.with_extension("xmp"), dest_folder.join(format!("{}.xmp", src_path.file_stem().unwrap_or_default().to_string_lossy()))),
                    (PathBuf::from(format!("{}.xmp", src_path_str)), dest_folder.join(format!("{}.xmp", file_name.to_string_lossy()))),
                ];
                for (src_s, dst_s) in sidecar_candidates {
                    if src_s.exists() && src_s != src_path {
                        let _ = fs::copy(&src_s, &dst_s);
                        if options.move_files {
                            let _ = fs::remove_file(&src_s);
                        }
                    }
                }
                if let Some(src_json) = takeout::match_takeout_json(&src_path) {
                    if let Some(jname) = src_json.file_name() {
                        let dst_json = dest_folder.join(jname);
                        let _ = fs::copy(&src_json, &dst_json);
                        if options.move_files {
                            let _ = fs::remove_file(&src_json);
                        }
                    }
                }
            }

            // Transfer master negative
            if options.move_files {
                if let Err(e) = fs::copy(&src_path, &dest_file_path) {
                    error_count += 1;
                    error_messages.push(format!("Failed to copy {}: {}", src_path.display(), e));
                    continue;
                }

                // Bit-level cryptographic verification before source deletion
                match crate::checksum::verify_files_identical(&src_path, &dest_file_path) {
                    Ok(true) => {
                        if let Err(e) = fs::remove_file(&src_path) {
                            error_count += 1;
                            error_messages.push(format!("Failed to remove source negative {}: {}", src_path.display(), e));
                        }
                    }
                    Ok(false) => {
                        error_count += 1;
                        error_messages.push(format!("Cryptographic checksum verification failed for {}: source negative preserved", src_path.display()));
                        continue;
                    }
                    Err(e) => {
                        error_count += 1;
                        error_messages.push(format!("Verification I/O error for {}: {}. Source negative preserved", src_path.display(), e));
                        continue;
                    }
                }
            } else {
                if let Err(e) = fs::copy(&src_path, &dest_file_path) {
                    error_count += 1;
                    error_messages.push(format!("Failed to copy {}: {}", src_path.display(), e));
                    continue;
                }
            }

            // Collect record for atomic SQL batch insert into destination catalog database
            if let Ok(Some(mut record)) = self.get_photo(&src_path_str) {
                record.file_path = dest_file_path.to_string_lossy().to_string();
                record.file_name = file_name.to_string_lossy().to_string();
                record.updated_at = current_timestamp();
                records_to_insert.push(record);
            }

            if options.move_files {
                if let Ok(conn) = self.conn.lock() {
                    let _ = conn.execute("DELETE FROM photos WHERE file_path = ?1", [&src_path_str]);
                }
            }

            imported_count += 1;
        }

        // Atomic SQL transaction insert into destination catalog database
        if !records_to_insert.is_empty() {
            let mut conn = dest_db.conn.lock().unwrap();
            let tx = conn.transaction().map_err(|e| e.to_string())?;
            for record in &records_to_insert {
                let mut stmt = tx.prepare_cached(
                    "INSERT INTO photos (
                        file_path, file_name, file_size, modified_time,
                        camera_make, camera_model, lens_model, iso, aperture, shutter_speed, focal_length,
                        taken_time, latitude, longitude, title, description, creator, copyright, keywords, rating, status,
                        has_xmp, has_takeout, created_at, updated_at
                    ) VALUES (
                        ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25
                    ) ON CONFLICT(file_path) DO UPDATE SET
                        file_name = excluded.file_name,
                        file_size = excluded.file_size,
                        modified_time = excluded.modified_time,
                        camera_make = excluded.camera_make,
                        camera_model = excluded.camera_model,
                        lens_model = excluded.lens_model,
                        iso = excluded.iso,
                        aperture = excluded.aperture,
                        shutter_speed = excluded.shutter_speed,
                        focal_length = excluded.focal_length,
                        taken_time = excluded.taken_time,
                        latitude = excluded.latitude,
                        longitude = excluded.longitude,
                        title = excluded.title,
                        description = excluded.description,
                        creator = excluded.creator,
                        copyright = excluded.copyright,
                        keywords = excluded.keywords,
                        rating = excluded.rating,
                        status = excluded.status,
                        has_xmp = excluded.has_xmp,
                        has_takeout = excluded.has_takeout,
                        updated_at = excluded.updated_at"
                ).map_err(|e| e.to_string())?;

                stmt.execute(rusqlite::params![
                    record.file_path, record.file_name, record.file_size, record.modified_time,
                    record.camera_make, record.camera_model, record.lens_model, record.iso,
                    record.aperture, record.shutter_speed, record.focal_length, record.taken_time,
                    record.latitude, record.longitude, record.title, record.description,
                    record.creator, record.copyright, record.keywords, record.rating, record.status,
                    record.has_xmp as i32, record.has_takeout as i32, record.created_at, record.updated_at
                ]).map_err(|e| e.to_string())?;
            }
            tx.commit().map_err(|e| e.to_string())?;
            let _ = conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);");
        } else if imported_count > 0 {
            let _ = dest_db.index_folder(&dest_folder);
        }

        Ok(ImportReport {
            total_found,
            imported_count,
            skipped_duplicates,
            raw_over_jpeg_suppressed,
            consolidated_sidecars,
            error_count,
            destination_catalog_path: dest_catalog_path.to_string_lossy().to_string(),
            error_messages,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportOptions {
    pub move_files: bool,
    pub consolidate_sidecars: bool,
    pub deduplicate_raw_over_jpeg: bool,
    pub skip_duplicates: bool,
    pub verify_checksum: bool,
    pub subfolder_name: Option<String>,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            move_files: false,
            consolidate_sidecars: true,
            deduplicate_raw_over_jpeg: true,
            skip_duplicates: true,
            verify_checksum: true,
            subfolder_name: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportReport {
    pub total_found: u32,
    pub imported_count: u32,
    pub skipped_duplicates: u32,
    pub raw_over_jpeg_suppressed: u32,
    pub consolidated_sidecars: u32,
    pub error_count: u32,
    pub destination_catalog_path: String,
    pub error_messages: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ExportPackageStats {
    pub total_requested: u32,
    pub exported_count: u32,
    pub error_count: u32,
    pub destination_catalog_path: String,
    pub error_messages: Vec<String>,
}

fn is_supported_image(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        matches!(
            ext_lower.as_str(),
            "cr2" | "rw2" | "dng" | "jpg" | "jpeg" | "png" | "heic" | "arw" | "nef" | "tiff" | "tif"
        )
    } else {
        false
    }
}

fn is_raw_format(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        matches!(ext_lower.as_str(), "cr2" | "rw2" | "dng" | "arw" | "nef")
    } else {
        false
    }
}

fn normalize_stem(stem: &str) -> String {
    let lower = stem.to_lowercase();
    let suffixes = [
        "-edit", "_edit", "-edited", "_edited", 
        "-export", "_export", "_v1", "_v2", 
        "_01", "_02", " (1)", " (2)"
    ];
    for s in &suffixes {
        if lower.ends_with(s) {
            return lower[..lower.len() - s.len()].to_string();
        }
    }
    lower
}

fn are_same_capture(p1: &Path, p2: &Path) -> bool {
    let r1 = is_raw_format(p1);
    let r2 = is_raw_format(p2);
    
    // Check EXIF capture timestamp if available
    let meta1 = crate::raw::extract_metadata(p1).ok();
    let meta2 = crate::raw::extract_metadata(p2).ok();
    if let (Some(m1), Some(m2)) = (meta1, meta2) {
        if let (Some(t1), Some(t2)) = (m1.taken_time, m2.taken_time) {
            return (t1 - t2).abs() <= 5;
        }
    }

    // If one is RAW and the other is non-raw, within the same import tree
    if r1 != r2 {
        return true;
    }

    // Both RAW or both non-raw:
    if p1.parent() == p2.parent() {
        return true;
    }
    
    let p1_str = p1.to_string_lossy().to_lowercase();
    let p2_str = p2.to_string_lossy().to_lowercase();
    let export_keywords = ["export", "output", "darktable", "processed", "web", "jpeg", "jpg", "proof", "lyft"];
    let p1_is_export = export_keywords.iter().any(|k| p1_str.contains(k));
    let p2_is_export = export_keywords.iter().any(|k| p2_str.contains(k));
    if p1_is_export || p2_is_export {
        return true;
    }
    
    if let (Some(parent1), Some(parent2)) = (p1.parent(), p2.parent()) {
        if parent1.starts_with(parent2) || parent2.starts_with(parent1) {
            return true;
        }
    }
    
    false
}

fn transfer_file<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q, move_file: bool) -> std::io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    if move_file {
        if fs::rename(src, dst).is_err() {
            // Cross-device move fallback
            fs::copy(src, dst)?;
            fs::remove_file(src)?;
        }
    } else {
        fs::copy(src, dst)?;
    }
    Ok(())
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_db_init_and_crud() {
        let db = Database::open_in_memory().expect("Should create in-memory db");

        let record = PhotoRecord {
            id: 0,
            file_path: "/photos/IMG_0001.CR2".to_string(),
            file_name: "IMG_0001.CR2".to_string(),
            file_size: 25000000,
            modified_time: 1600000000,
            camera_make: Some("Canon".to_string()),
            camera_model: Some("Canon EOS 5D Mark IV".to_string()),
            lens_model: Some("EF 24-70mm f/2.8L II USM".to_string()),
            iso: Some(100),
            aperture: Some(2.8),
            shutter_speed: Some("1/200".to_string()),
            focal_length: Some(50.0),
            taken_time: Some(1600000000),
            latitude: Some(37.7749),
            longitude: Some(-122.4194),
            title: Some("Half Dome".to_string()),
            description: Some("Test Photo".to_string()),
            creator: Some("Ansel Adams".to_string()),
            copyright: Some("© 2026".to_string()),
            keywords: Some("nature, mountain".to_string()),
            rating: Some(5),
            status: "Pick".to_string(),
            has_xmp: true,
            has_takeout: false,
            created_at: 1600000000,
            updated_at: 1600000000,
        };

        db.upsert_photo(&record).expect("Should insert photo");

        let fetched = db
            .get_photo("/photos/IMG_0001.CR2")
            .expect("Query should succeed")
            .expect("Photo should exist");

        assert_eq!(fetched.file_name, "IMG_0001.CR2");
        assert_eq!(fetched.rating, Some(5));
        assert_eq!(fetched.status, "Pick");
        assert_eq!(fetched.camera_make.as_deref(), Some("Canon"));
        assert_eq!(fetched.title.as_deref(), Some("Half Dome"));
        assert_eq!(fetched.creator.as_deref(), Some("Ansel Adams"));
        assert_eq!(fetched.keywords.as_deref(), Some("nature, mountain"));

        // Test update metadata
        let meta_updated = db
            .update_photo_metadata(
                "/photos/IMG_0001.CR2",
                Some("El Capitan"),
                Some("Updated Caption"),
                Some("Galen Rowell"),
                Some("© 2026 All Rights Reserved"),
                Some("climbing, yosemite"),
            )
            .expect("Metadata update should succeed");
        assert!(meta_updated);

        let refetched_meta = db.get_photo("/photos/IMG_0001.CR2").unwrap().unwrap();
        assert_eq!(refetched_meta.title.as_deref(), Some("El Capitan"));
        assert_eq!(refetched_meta.creator.as_deref(), Some("Galen Rowell"));
        assert_eq!(refetched_meta.keywords.as_deref(), Some("climbing, yosemite"));

        // Test update rating & status
        let updated = db
            .update_rating_status("/photos/IMG_0001.CR2", Some(3), "Reject")
            .expect("Update should succeed");
        assert!(updated);

        let refetched = db
            .get_photo("/photos/IMG_0001.CR2")
            .unwrap()
            .unwrap();
        assert_eq!(refetched.rating, Some(3));
        assert_eq!(refetched.status, "Reject");

        let all = db.get_all_photos().unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_catalog_path_resolution() {
        let external = Path::new("/Volumes/ExternalSSD/Photos");
        let path = Database::resolve_catalog_path(external, false);
        assert_eq!(path, external.join(".firstproof").join("catalog.db"));

        let internal = Path::new("/Users/test/Pictures");
        let path_portable = Database::resolve_catalog_path(internal, true);
        assert_eq!(path_portable, internal.join(".firstproof").join("catalog.db"));
    }

    #[test]
    fn test_index_folder() {
        let temp_dir = std::env::temp_dir().join("firstproof_test_db_index");
        let _ = fs::create_dir_all(&temp_dir);

        let image_path = temp_dir.join("sample.jpg");
        let json_path = temp_dir.join("sample.jpg.json");

        let mut f = File::create(&image_path).unwrap();
        f.write_all(b"dummy jpeg content").unwrap();

        let json_content = r#"{
            "title": "sample.jpg",
            "description": "Takeout Description",
            "geoData": { "latitude": 40.7128, "longitude": -74.0060, "altitude": 5.0 },
            "photoTakenTime": { "timestamp": "1620000000" }
        }"#;
        fs::write(&json_path, json_content).unwrap();

        let db = Database::open_in_memory().unwrap();
        let indexed = db.index_folder(&temp_dir).unwrap();
        assert_eq!(indexed, 1);

        let photo = db.get_photo(&image_path.to_string_lossy()).unwrap().unwrap();
        assert_eq!(photo.file_name, "sample.jpg");
        assert_eq!(photo.latitude, Some(40.7128));
        assert_eq!(photo.longitude, Some(-74.0060));
        assert_eq!(photo.taken_time, Some(1620000000));
        assert_eq!(photo.description.as_deref(), Some("Takeout Description"));
        assert!(photo.has_takeout);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_export_portable_package() {
        let src_dir = std::env::temp_dir().join("firstproof_test_src_export");
        let dst_dir = std::env::temp_dir().join("firstproof_test_dst_export");
        let _ = fs::remove_dir_all(&src_dir);
        let _ = fs::remove_dir_all(&dst_dir);
        let _ = fs::create_dir_all(&src_dir);

        let img_path = src_dir.join("photo1.jpg");
        let xmp_path = src_dir.join("photo1.jpg.xmp");
        let _ = fs::write(&img_path, b"dummy img data");
        let _ = fs::write(&xmp_path, b"<x:xmpmeta></x:xmpmeta>");

        let src_catalog = Database::resolve_catalog_path(&src_dir, true);
        let src_db = Database::open(&src_catalog).unwrap();
        src_db.index_folder(&src_dir).unwrap();
        assert_eq!(src_db.get_all_photos().unwrap().len(), 1);

        // Test Copy Mode
        let paths = vec![img_path.to_string_lossy().to_string()];
        let report = src_db.export_portable_package(&paths, &dst_dir, false).unwrap();
        assert_eq!(report.exported_count, 1);
        assert_eq!(report.error_count, 0);

        // Verify destination has photo, xmp sidecar, and catalog
        assert!(dst_dir.join("photo1.jpg").exists());
        assert!(dst_dir.join("photo1.jpg.xmp").exists());
        let dst_catalog = dst_dir.join(".firstproof").join("catalog.db");
        assert!(dst_catalog.exists());

        let dst_db = Database::open(&dst_catalog).unwrap();
        let dst_photos = dst_db.get_all_photos().unwrap();
        assert_eq!(dst_photos.len(), 1);
        assert_eq!(dst_photos[0].file_name, "photo1.jpg");

        // Verify source is still intact (Copy mode)
        assert!(img_path.exists());
        assert!(xmp_path.exists());
        assert_eq!(src_db.get_all_photos().unwrap().len(), 1);

        // Test Move Mode on a second image
        let img2_path = src_dir.join("photo2.jpg");
        let _ = fs::write(&img2_path, b"dummy img data 2");
        src_db.index_folder(&src_dir).unwrap();
        assert_eq!(src_db.get_all_photos().unwrap().len(), 2);

        let paths2 = vec![img2_path.to_string_lossy().to_string()];
        let report2 = src_db.export_portable_package(&paths2, &dst_dir, true).unwrap();
        assert_eq!(report2.exported_count, 1);

        // Verify photo2 is now in destination and REMOVED from source
        assert!(dst_dir.join("photo2.jpg").exists());
        assert!(!img2_path.exists());
        assert_eq!(src_db.get_all_photos().unwrap().len(), 1);

        let _ = fs::remove_dir_all(&src_dir);
        let _ = fs::remove_dir_all(&dst_dir);
    }

    #[test]
    fn test_import_to_library_deduplication_and_consolidation() {
        let src_dir = std::env::temp_dir().join("firstproof_test_src_import");
        let dst_dir = std::env::temp_dir().join("firstproof_test_dst_import");
        let _ = fs::remove_dir_all(&src_dir);
        let _ = fs::remove_dir_all(&dst_dir);
        let _ = fs::create_dir_all(&src_dir);
        let _ = fs::create_dir_all(&dst_dir);

        // 1. Create RAW + JPEG pair: RAW should win over JPEG
        let raw_path = src_dir.join("IMG_0100.CR2");
        let jpg_path = src_dir.join("IMG_0100.JPG");
        fs::write(&raw_path, b"RAW sensor data 1234567890").unwrap();
        fs::write(&jpg_path, b"JPEG compressed 1234").unwrap();

        // Also create a cross-directory exported JPEG duplicate (e.g. Darktable export subfolder)
        let export_dir = src_dir.join("darktable_exported").join("web");
        fs::create_dir_all(&export_dir).unwrap();
        let exported_jpg = export_dir.join("IMG_0100.JPG");
        fs::write(&exported_jpg, b"JPEG export 1234").unwrap();

        // 2. Create third-party Lightroom/Darktable sidecar and Takeout sidecar
        let xmp_path = src_dir.join("IMG_0100.CR2.xmp");
        fs::write(&xmp_path, b"<x:xmpmeta xmlns:x=\"adobe:ns:meta/\"><rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"><rdf:Description xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\"><xmp:Rating>5</xmp:Rating><dc:title><rdf:Alt><rdf:li xml:lang=\"x-default\">Authentic Horizon</rdf:li></rdf:Alt></dc:title></rdf:Description></rdf:RDF></x:xmpmeta>").unwrap();

        let src_catalog = Database::resolve_catalog_path(&src_dir, true);
        let src_db = Database::open(&src_catalog).unwrap();
        src_db.index_folder(&src_dir).unwrap();

        let options = ImportOptions {
            move_files: false,
            consolidate_sidecars: true,
            deduplicate_raw_over_jpeg: true,
            skip_duplicates: true,
            verify_checksum: true,
            subfolder_name: Some("2026-09-13".to_string()),
        };

        let report = src_db.import_to_library(
            &[src_dir.to_string_lossy().to_string()],
            &dst_dir,
            &options,
        ).unwrap();

        // RAW over JPEG: 1 imported, 2 suppressed (sibling + exported subfolder)
        assert_eq!(report.total_found, 3);
        assert_eq!(report.imported_count, 1);
        assert_eq!(report.raw_over_jpeg_suppressed, 2);
        assert_eq!(report.consolidated_sidecars, 1);
        assert_eq!(report.error_count, 0);

        let target_subfolder = dst_dir.join("2026-09-13");
        assert!(target_subfolder.join("IMG_0100.CR2").exists());
        // JPEG suppressed: not copied
        assert!(!target_subfolder.join("IMG_0100.JPG").exists());
        // Unified single XMP exists
        assert!(target_subfolder.join("IMG_0100.xmp").exists());
        // Third-party .CR2.xmp was NOT copied to destination: consolidated into .xmp!
        assert!(!target_subfolder.join("IMG_0100.CR2.xmp").exists());

        // Verify metadata in consolidated XMP
        let parsed_xmp = xmp::io::read_xmp_file(target_subfolder.join("IMG_0100.xmp")).unwrap();
        assert_eq!(parsed_xmp.rating, Some(5));
        assert_eq!(parsed_xmp.title.as_deref(), Some("Authentic Horizon"));

        // Duplicate suppression test: re-importing should skip
        let report_dup = src_db.import_to_library(
            &[src_dir.to_string_lossy().to_string()],
            &dst_dir,
            &options,
        ).unwrap();
        assert_eq!(report_dup.skipped_duplicates, 1);
        assert_eq!(report_dup.imported_count, 0);

        let _ = fs::remove_dir_all(&src_dir);
        let _ = fs::remove_dir_all(&dst_dir);
    }

    #[test]
    fn test_import_move_with_checksum_verification() {
        let src_dir = std::env::temp_dir().join("firstproof_test_src_move");
        let dst_dir = std::env::temp_dir().join("firstproof_test_dst_move");
        let _ = fs::remove_dir_all(&src_dir);
        let _ = fs::remove_dir_all(&dst_dir);
        let _ = fs::create_dir_all(&src_dir);
        let _ = fs::create_dir_all(&dst_dir);

        let raw_path = src_dir.join("FRAME_001.ARW");
        fs::write(&raw_path, b"RAW Sony CFA frame 987654321").unwrap();

        let src_catalog = Database::resolve_catalog_path(&src_dir, true);
        let src_db = Database::open(&src_catalog).unwrap();
        src_db.index_folder(&src_dir).unwrap();
        assert_eq!(src_db.get_all_photos().unwrap().len(), 1);

        let options = ImportOptions {
            move_files: true,
            consolidate_sidecars: true,
            deduplicate_raw_over_jpeg: true,
            skip_duplicates: true,
            verify_checksum: true,
            subfolder_name: None,
        };

        let report = src_db.import_to_library(
            &[raw_path.to_string_lossy().to_string()],
            &dst_dir,
            &options,
        ).unwrap();

        assert_eq!(report.imported_count, 1);
        assert_eq!(report.error_count, 0);

        // Verify destination exists and has exact same byte content
        let dst_file = dst_dir.join("FRAME_001.ARW");
        assert!(dst_file.exists());
        assert_eq!(fs::read(&dst_file).unwrap(), b"RAW Sony CFA frame 987654321");

        // Verify source was safely removed only after verified bit parity
        assert!(!raw_path.exists());
        assert_eq!(src_db.get_all_photos().unwrap().len(), 0);

        let _ = fs::remove_dir_all(&src_dir);
        let _ = fs::remove_dir_all(&dst_dir);
    }
}
