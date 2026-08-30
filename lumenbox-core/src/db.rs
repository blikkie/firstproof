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
    pub description: Option<String>,
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

        let conn = Connection::open(db_path)?;

        // Enable Write-Ahead Logging (WAL) and synchronous=NORMAL for high performance
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )?;

        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;
        Ok(db)
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
                description TEXT,
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
        Ok(())
    }

    /// Helper to resolve the database path for a catalog.
    /// If portable is true or if target folder is under /Volumes/, uses `<folder>/.lumenbox/catalog.db`.
    pub fn resolve_catalog_path<P: AsRef<Path>>(folder_path: P, portable: bool) -> PathBuf {
        let folder = folder_path.as_ref();
        let is_external = folder.starts_with("/Volumes") || portable;

        if is_external {
            folder.join(".lumenbox").join("catalog.db")
        } else {
            // Default macOS Application Support location
            if let Some(home) = dirs_next_home() {
                home.join("Library")
                    .join("Application Support")
                    .join("LumenBox")
                    .join("catalog.db")
            } else {
                folder.join(".lumenbox").join("catalog.db")
            }
        }
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

                // 2. XMP Sidecar Integration
                let xmp_path = path.with_extension("xmp");
                let mut has_xmp = false;
                let mut rating = None;
                let mut status = "None".to_string();

                if xmp_path.exists() {
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
                        if xmp_data.description.is_some() {
                            description = xmp_data.description;
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
                    description,
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
                    taken_time, latitude, longitude, description, rating, status,
                    has_xmp, has_takeout, created_at, updated_at
                ) VALUES (
                    ?1, ?2, ?3, ?4,
                    ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                    ?12, ?13, ?14, ?15, ?16, ?17,
                    ?18, ?19, ?20, ?21
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
                    description = excluded.description,
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
                    p.description,
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
                taken_time, latitude, longitude, description, rating, status,
                has_xmp, has_takeout, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4,
                ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                ?12, ?13, ?14, ?15, ?16, ?17,
                ?18, ?19, ?20, ?21
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
                description = excluded.description,
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
                photo.description,
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
                    taken_time, latitude, longitude, description, rating, status,
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
                description: row.get(15)?,
                rating: row.get(16)?,
                status: row.get(17)?,
                has_xmp: row.get::<_, i32>(18)? != 0,
                has_takeout: row.get::<_, i32>(19)? != 0,
                created_at: row.get(20)?,
                updated_at: row.get(21)?,
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
                    taken_time, latitude, longitude, description, rating, status,
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
                description: row.get(15)?,
                rating: row.get(16)?,
                status: row.get(17)?,
                has_xmp: row.get::<_, i32>(18)? != 0,
                has_takeout: row.get::<_, i32>(19)? != 0,
                created_at: row.get(20)?,
                updated_at: row.get(21)?,
            })
        })?;

        let mut photos = Vec::new();
        for p in photo_iter {
            photos.push(p?);
        }
        Ok(photos)
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
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn dirs_next_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
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
            description: Some("Test Photo".to_string()),
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
        assert_eq!(path, external.join(".lumenbox").join("catalog.db"));

        let internal = Path::new("/Users/test/Pictures");
        let path_portable = Database::resolve_catalog_path(internal, true);
        assert_eq!(path_portable, internal.join(".lumenbox").join("catalog.db"));
    }

    #[test]
    fn test_index_folder() {
        let temp_dir = std::env::temp_dir().join("lumenbox_test_db_index");
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
}
