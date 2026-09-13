use xmp_writer::{CustomNamespace, Namespace, XmpWriter};
use super::data::{format_gps_xmp, serialize_curve_points, XmpData};

pub fn generate_xmp(data: &XmpData) -> String {
    let mut writer = XmpWriter::new();

    // Standard Dublin Core schemas
    if let Some(t) = &data.title {
        writer.title([(None, t.as_str())]);
    }
    if let Some(d) = &data.description {
        writer.description([(None, d.as_str())]);
    }
    if let Some(c) = &data.creator {
        writer.creator([c.as_str()]);
    }
    if let Some(r) = &data.copyright {
        writer.rights([(None, r.as_str())]);
    }
    if !data.keywords.is_empty() {
        writer.subject(data.keywords.iter().map(|s| s.as_str()));
    }

    // Standard XMP Rating
    if let Some(r) = data.rating {
        writer.element("Rating", Namespace::Xmp).value(r);
    }

    // EXIF namespace
    let exif_ns = Namespace::Custom(Box::new(CustomNamespace::new(
        "Exif",
        "exif",
        "http://ns.adobe.com/exif/1.0/",
    )));
    if let (Some(lat), Some(lon)) = (data.latitude, data.longitude) {
        writer.element("GPSLatitude", exif_ns.clone()).value(format_gps_xmp(lat, true).as_str());
        writer.element("GPSLongitude", exif_ns.clone()).value(format_gps_xmp(lon, false).as_str());
        writer.element("GPSVersionID", exif_ns).value("2.2.0.0");
    }

    // Custom First Proof PM namespace
    let pm_ns = Namespace::Custom(Box::new(CustomNamespace::new(
        "First Proof Photo Metadata",
        "pm",
        "https://firstproof.photo/ns/pm/1.0/",
    )));

    writer.element("status", pm_ns.clone()).value(data.status.as_str());
    writer.element("exposure", pm_ns.clone()).value(format!("{:.4}", data.edits.exposure).as_str());
    writer.element("contrast", pm_ns.clone()).value(format!("{:.4}", data.edits.contrast).as_str());
    writer.element("highlights", pm_ns.clone()).value(format!("{:.4}", data.edits.highlights).as_str());
    writer.element("shadows", pm_ns.clone()).value(format!("{:.4}", data.edits.shadows).as_str());
    writer.element("whites", pm_ns.clone()).value(format!("{:.4}", data.edits.whites).as_str());
    writer.element("blacks", pm_ns.clone()).value(format!("{:.4}", data.edits.blacks).as_str());
    writer.element("vibrance", pm_ns.clone()).value(format!("{:.4}", data.edits.vibrance).as_str());
    writer.element("saturation", pm_ns.clone()).value(format!("{:.4}", data.edits.saturation).as_str());
    writer.element("deepBlue", pm_ns.clone()).value(format!("{:.4}", data.edits.deep_blue).as_str());
    writer.element("foliage", pm_ns.clone()).value(format!("{:.4}", data.edits.foliage).as_str());
    writer.element("sharpeningAmount", pm_ns.clone()).value(format!("{:.4}", data.edits.sharpening_amount).as_str());
    writer.element("sharpeningRadius", pm_ns.clone()).value(format!("{:.4}", data.edits.sharpening_radius).as_str());
    writer.element("sharpeningMasking", pm_ns.clone()).value(format!("{:.4}", data.edits.sharpening_masking).as_str());
    writer.element("noiseReduction", pm_ns.clone()).value(format!("{:.4}", data.edits.noise_reduction).as_str());
    writer.element("grainAmount", pm_ns.clone()).value(format!("{:.4}", data.edits.grain_amount).as_str());
    writer.element("grainSize", pm_ns.clone()).value(format!("{:.4}", data.edits.grain_size).as_str());
    writer.element("grainRoughness", pm_ns.clone()).value(format!("{:.4}", data.edits.grain_roughness).as_str());
    writer.element("grainChroma", pm_ns.clone()).value(format!("{:.4}", data.edits.grain_chroma).as_str());
    writer.element("temperature", pm_ns.clone()).value(format!("{:.4}", data.edits.temperature).as_str());
    writer.element("tint", pm_ns.clone()).value(format!("{:.4}", data.edits.tint).as_str());
    writer.element("cropLeft", pm_ns.clone()).value(format!("{:.4}", data.edits.crop_left).as_str());
    writer.element("cropTop", pm_ns.clone()).value(format!("{:.4}", data.edits.crop_top).as_str());
    writer.element("cropRight", pm_ns.clone()).value(format!("{:.4}", data.edits.crop_right).as_str());
    writer.element("cropBottom", pm_ns.clone()).value(format!("{:.4}", data.edits.crop_bottom).as_str());
    writer.element("cropAngle", pm_ns.clone()).value(format!("{:.4}", data.edits.crop_angle).as_str());
    writer.element("perspectiveVertical", pm_ns.clone()).value(format!("{:.4}", data.edits.perspective_vertical).as_str());
    writer.element("perspectiveHorizontal", pm_ns.clone()).value(format!("{:.4}", data.edits.perspective_horizontal).as_str());
    writer.element("cropConstrain", pm_ns.clone()).value(if data.edits.crop_constrain { "true" } else { "false" });
    writer.element("rotation", pm_ns.clone()).value(format!("{}", data.edits.rotation).as_str());
    writer.element("flipHorizontal", pm_ns.clone()).value(if data.edits.flip_horizontal { "true" } else { "false" });
    writer.element("flipVertical", pm_ns.clone()).value(if data.edits.flip_vertical { "true" } else { "false" });

    let tone_profile_str = match data.edits.tone_profile {
        crate::ColorProfileMode::AgxVivid => "AgxVivid",
        crate::ColorProfileMode::AgxFilmic => "AgxFilmic",
        crate::ColorProfileMode::CameraStandard => "CameraStandard",
        crate::ColorProfileMode::CameraPortrait => "CameraPortrait",
        crate::ColorProfileMode::CameraLandscape => "CameraLandscape",
        crate::ColorProfileMode::CameraVivid => "CameraVivid",
    };
    writer.element("toneProfile", pm_ns.clone()).value(tone_profile_str);

    if let Some(name) = &data.edits.lut_name {
        writer.element("lutName", pm_ns.clone()).value(name.as_str());
        writer.element("lutIntensity", pm_ns.clone()).value(format!("{:.4}", data.edits.lut_intensity).as_str());
    }

    if !data.edits.curves.master.is_empty() {
        writer.element("curveMaster", pm_ns.clone()).value(serialize_curve_points(&data.edits.curves.master).as_str());
    }
    if !data.edits.curves.red.is_empty() {
        writer.element("curveRed", pm_ns.clone()).value(serialize_curve_points(&data.edits.curves.red).as_str());
    }
    if !data.edits.curves.green.is_empty() {
        writer.element("curveGreen", pm_ns.clone()).value(serialize_curve_points(&data.edits.curves.green).as_str());
    }
    if !data.edits.curves.blue.is_empty() {
        writer.element("curveBlue", pm_ns.clone()).value(serialize_curve_points(&data.edits.curves.blue).as_str());
    }

    // Adobe Camera Raw Settings (CRS) namespace for compatibility with Lightroom / Camera Raw
    let crs_ns = Namespace::Custom(Box::new(CustomNamespace::new(
        "Camera Raw Settings",
        "crs",
        "http://ns.adobe.com/camera-raw-settings/1.0/",
    )));
    writer.element("PerspectiveVertical", crs_ns.clone()).value(format!("{:.4}", data.edits.perspective_vertical).as_str());
    writer.element("PerspectiveHorizontal", crs_ns.clone()).value(format!("{:.4}", data.edits.perspective_horizontal).as_str());
    writer.element("CropConstrainToWarp", crs_ns.clone()).value(if data.edits.crop_constrain { "1" } else { "0" });
    writer.element("GrainAmount", crs_ns.clone()).value(format!("{:.4}", data.edits.grain_amount).as_str());
    writer.element("GrainSize", crs_ns.clone()).value(format!("{:.4}", data.edits.grain_size).as_str());
    writer.element("GrainFrequency", crs_ns).value(format!("{:.4}", data.edits.grain_roughness).as_str());

    writer.finish(None)
}
