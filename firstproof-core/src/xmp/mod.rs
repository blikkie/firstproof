pub mod data;
pub mod writer;
pub mod reader;
pub mod io;

pub use data::*;
pub use writer::*;
pub use reader::*;
pub use io::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CurvePointRecord;

    #[test]
    fn test_xmp_roundtrip() {
        let mut data = XmpData::default();
        data.rating = Some(4);
        data.status = "Pick".to_string();
        data.title = Some("San Francisco Bridge".to_string());
        data.description = Some("Taken in high fog.".to_string());
        data.latitude = Some(37.8199);
        data.longitude = Some(-122.4783);
        
        data.edits.exposure = 0.85;
        data.edits.contrast = 25.0;
        data.edits.highlights = -40.0;
        data.edits.shadows = 35.0;
        data.edits.whites = 15.0;
        data.edits.blacks = -20.0;
        data.edits.vibrance = 25.0;
        data.edits.saturation = -10.0;
        data.edits.deep_blue = 40.0;
        data.edits.foliage = 30.0;
        data.edits.sharpening_amount = 45.0;
        data.edits.sharpening_radius = 1.5;
        data.edits.sharpening_masking = 25.0;
        data.edits.noise_reduction = 10.0;
        data.edits.temperature = 5800.0;
        data.edits.tint = -12.0;
        data.edits.crop_left = 0.05;
        data.edits.crop_top = 0.05;
        data.edits.crop_right = 0.95;
        data.edits.crop_bottom = 0.95;
        data.edits.crop_angle = -0.5;
        data.edits.perspective_vertical = -15.0;
        data.edits.perspective_horizontal = 8.5;
        data.edits.crop_constrain = true;
        data.edits.rotation = 90;
        data.edits.flip_horizontal = true;
        data.edits.flip_vertical = false;
        
        let path = "test_image_metadata.xmp";
        write_xmp_file(path, &data).unwrap();
        
        let parsed = read_xmp_file(path).unwrap();
        let _ = std::fs::remove_file(path);
        
        assert_eq!(parsed.rating, Some(4));
        assert_eq!(parsed.status, "Pick");
        assert_eq!(parsed.title.as_deref(), Some("San Francisco Bridge"));
        assert_eq!(parsed.description.as_deref(), Some("Taken in high fog."));
        
        // Assert float coordinates within rounding tolerance
        assert!((parsed.latitude.unwrap() - 37.8199).abs() < 0.0001);
        assert!((parsed.longitude.unwrap() - -122.4783).abs() < 0.0001);
        
        assert_eq!(parsed.edits.exposure, 0.85);
        assert_eq!(parsed.edits.contrast, 25.0);
        assert_eq!(parsed.edits.highlights, -40.0);
        assert_eq!(parsed.edits.shadows, 35.0);
        assert_eq!(parsed.edits.whites, 15.0);
        assert_eq!(parsed.edits.blacks, -20.0);
        assert_eq!(parsed.edits.vibrance, 25.0);
        assert_eq!(parsed.edits.saturation, -10.0);
        assert_eq!(parsed.edits.deep_blue, 40.0);
        assert_eq!(parsed.edits.foliage, 30.0);
        assert_eq!(parsed.edits.sharpening_amount, 45.0);
        assert_eq!(parsed.edits.sharpening_radius, 1.5);
        assert_eq!(parsed.edits.sharpening_masking, 25.0);
        assert_eq!(parsed.edits.noise_reduction, 10.0);
        assert_eq!(parsed.edits.temperature, 5800.0);
        assert_eq!(parsed.edits.tint, -12.0);
        assert_eq!(parsed.edits.crop_angle, -0.5);
        assert_eq!(parsed.edits.perspective_vertical, -15.0);
        assert_eq!(parsed.edits.perspective_horizontal, 8.5);
        assert_eq!(parsed.edits.crop_constrain, true);
        assert_eq!(parsed.edits.rotation, 90);
        assert_eq!(parsed.edits.flip_horizontal, true);
        assert_eq!(parsed.edits.flip_vertical, false);
    }

    #[test]
    fn test_xmp_adobe_perspective_ingestion() {
        let adobe_xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
    crs:PerspectiveVertical="+25.0"
    crs:PerspectiveHorizontal="-12.5"
    crs:CropConstrainToWarp="1">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;
        let parsed = parse_xmp(adobe_xmp).unwrap();
        assert_eq!(parsed.edits.perspective_vertical, 25.0);
        assert_eq!(parsed.edits.perspective_horizontal, -12.5);
        assert_eq!(parsed.edits.crop_constrain, true);
    }

    #[test]
    fn test_xmp_adobe_tone_recovery_ingestion() {
        let adobe_xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
    crs:Highlights2012="-65"
    crs:Shadows2012="+45"
    crs:Whites2012="+20"
    crs:Blacks2012="-15">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;
        let parsed = parse_xmp(adobe_xmp).unwrap();
        assert_eq!(parsed.edits.highlights, -65.0);
        assert_eq!(parsed.edits.shadows, 45.0);
        assert_eq!(parsed.edits.whites, 20.0);
        assert_eq!(parsed.edits.blacks, -15.0);
    }

    #[test]
    fn test_xmp_adobe_color_sliders_ingestion() {
        let adobe_xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
    crs:Vibrance="35"
    crs:Saturation="-15">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;
        let parsed = parse_xmp(adobe_xmp).unwrap();
        assert_eq!(parsed.edits.vibrance, 35.0);
        assert_eq!(parsed.edits.saturation, -15.0);
    }

    #[test]
    fn test_xmp_adobe_detail_sliders_ingestion() {
        let adobe_xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
    crs:SharpenAmount="40"
    crs:SharpenRadius="1.2"
    crs:SharpenEdgeMasking="20"
    crs:LuminanceSmoothing="15">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;
        let parsed = parse_xmp(adobe_xmp).unwrap();
        assert_eq!(parsed.edits.sharpening_amount, 40.0);
        assert_eq!(parsed.edits.sharpening_radius, 1.2);
        assert_eq!(parsed.edits.sharpening_masking, 20.0);
        assert_eq!(parsed.edits.noise_reduction, 15.0);
    }

    #[test]
    fn test_xmp_sidecar_resolution_from_image_path() {
        let raw_photo = "test_photo.CR2";
        let expected_xmp = "test_photo.xmp";
        
        let mut data = XmpData::default();
        data.edits.exposure = 1.25;
        data.edits.temperature = 6200.0;
        data.status = "Pick".to_string();
        
        // Writing with RAW image path creates test_photo.xmp
        write_xmp_file(raw_photo, &data).unwrap();
        assert!(std::path::Path::new(expected_xmp).exists());
        
        // Reading with RAW image path automatically finds test_photo.xmp
        let read_back = read_xmp_file(raw_photo).unwrap();
        assert_eq!(read_back.edits.exposure, 1.25);
        assert_eq!(read_back.edits.temperature, 6200.0);
        assert_eq!(read_back.status, "Pick");
        
        let _ = std::fs::remove_file(expected_xmp);
    }

    #[test]
    fn test_dual_sidecar_mtime_precedence_and_metadata_merge() {
        let photo = "test_dual_mtime.CR2";
        let sidecar_old = "test_dual_mtime.CR2.xmp";
        let sidecar_new = "test_dual_mtime.xmp";

        let mut old_data = XmpData::default();
        old_data.edits.temperature = 3800.0;
        old_data.edits.exposure = 0.6;
        old_data.creator = Some("Darktable User".to_string());
        old_data.keywords = vec!["subway".to_string()];
        std::fs::write(sidecar_old, generate_xmp(&old_data)).unwrap();

        // Brief delay to ensure distinct filesystem modification timestamps
        std::thread::sleep(std::time::Duration::from_millis(50));

        let mut new_data = XmpData::default();
        new_data.edits.temperature = 7400.0;
        new_data.edits.exposure = 0.6;
        // creator and keywords left empty in newer file
        std::fs::write(sidecar_new, generate_xmp(&new_data)).unwrap();

        let read_back = read_xmp_file(photo).unwrap();
        // Newer file edits win
        assert_eq!(read_back.edits.temperature, 7400.0);
        // Metadata from older file is safely preserved
        assert_eq!(read_back.creator, Some("Darktable User".to_string()));
        assert_eq!(read_back.keywords, vec!["subway".to_string()]);

        let _ = std::fs::remove_file(sidecar_old);
        let _ = std::fs::remove_file(sidecar_new);
    }

    #[test]
    fn test_xmp_third_party_orientation_hints() {
        let lr_xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:tiff="http://ns.adobe.com/tiff/1.0/"
    xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
    tiff:Orientation="6"
    crs:Orientation="Rotate 90 CW">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;
        let parsed_lr = parse_xmp(lr_xmp).unwrap();
        assert_eq!(parsed_lr.edits.rotation, 90);
        assert_eq!(parsed_lr.edits.flip_horizontal, false);
        assert_eq!(parsed_lr.edits.flip_vertical, false);

        let c1_xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:tiff="http://ns.adobe.com/tiff/1.0/"
    tiff:Orientation="8">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;
        let parsed_c1 = parse_xmp(c1_xmp).unwrap();
        assert_eq!(parsed_c1.edits.rotation, 270);
        assert_eq!(parsed_c1.edits.flip_horizontal, false);

        let mirror_xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:exif="http://ns.adobe.com/exif/1.0/"
    exif:Orientation="5">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;
        let parsed_mirror = parse_xmp(mirror_xmp).unwrap();
        assert_eq!(parsed_mirror.edits.rotation, 270);
        assert_eq!(parsed_mirror.edits.flip_horizontal, true);
    }

    #[test]
    fn test_xmp_curves_roundtrip() {
        let mut data = XmpData::default();
        data.edits.curves.master = vec![
            CurvePointRecord { x: 0.0, y: 0.05 },
            CurvePointRecord { x: 0.25, y: 0.20 },
            CurvePointRecord { x: 0.75, y: 0.85 },
            CurvePointRecord { x: 1.0, y: 0.98 },
        ];
        data.edits.curves.red = vec![
            CurvePointRecord { x: 0.0, y: 0.0 },
            CurvePointRecord { x: 0.5, y: 0.6 },
            CurvePointRecord { x: 1.0, y: 1.0 },
        ];

        let xml = generate_xmp(&data);
        assert!(xml.contains("curveMaster"));
        assert!(xml.contains("curveRed"));

        let parsed = parse_xmp(&xml).unwrap();
        assert_eq!(parsed.edits.curves.master.len(), 4);
        assert!((parsed.edits.curves.master[0].y - 0.05).abs() < 1e-3);
        assert!((parsed.edits.curves.master[2].y - 0.85).abs() < 1e-3);
        assert_eq!(parsed.edits.curves.red.len(), 3);
        assert!((parsed.edits.curves.red[1].y - 0.6).abs() < 1e-3);
        assert_eq!(parsed.edits.curves.green.len(), 2);
        assert_eq!(parsed.edits.curves.blue.len(), 2);
    }
}
