import type { PreviewResult } from '../WasmBridge';
import {
  formatAperture,
  formatDimensions,
  formatFocalLength,
  formatISO,
  formatShutterSpeed,
} from './formatters';

export function updateMetadataHeaderElements(
  elements: Record<string, HTMLElement>,
  preview: PreviewResult,
  fileName: string,
): void {
  const meta = preview.metadata;
  if (elements['meta-dimensions']) {
    elements['meta-dimensions'].innerText = formatDimensions(
      preview.width,
      preview.height,
      meta.raw_width,
      meta.raw_height,
    );
  }
  if (elements['meta-camera']) {
    elements['meta-camera'].innerText = `${meta.camera_model || 'Bayer Raw'}`;
  }
  if (elements['meta-iso']) {
    elements['meta-iso'].innerText = formatISO(meta.iso);
  }
  if (elements['meta-aperture']) {
    elements['meta-aperture'].innerText = formatAperture(meta.aperture);
  }
  if (elements['meta-shutter']) {
    elements['meta-shutter'].innerText = formatShutterSpeed(meta.shutter_speed);
  }

  const extBadge = fileName.split('.').pop()?.toUpperCase() || 'RAW';
  if (elements['meta-format-badge']) {
    elements['meta-format-badge'].innerText = extBadge;
  }

  const make = meta.camera_make || '';
  const model = meta.camera_model || '';
  const fullCamera =
    `${make} ${model}`.trim() || meta.camera_model || 'Bayer Sensor';
  if (elements['meta-full-camera']) {
    elements['meta-full-camera'].innerText = fullCamera;
  }
  if (elements['meta-lens']) {
    elements['meta-lens'].innerText = meta.lens_model || '—';
  }
  if (elements['meta-focal-length']) {
    elements['meta-focal-length'].innerText = formatFocalLength(
      meta.focal_length,
    );
  }
  if (elements['meta-exposure-summary']) {
    elements['meta-exposure-summary'].innerText =
      `${formatShutterSpeed(meta.shutter_speed)} • ${formatAperture(meta.aperture)} • ISO ${formatISO(meta.iso)}`;
  }
  if (elements['meta-sensor-res']) {
    elements['meta-sensor-res'].innerText = formatDimensions(
      preview.width,
      preview.height,
      meta.raw_width,
      meta.raw_height,
    );
  }
  if (elements['meta-date-taken']) {
    let dateStr = '—';
    if (meta.taken_time && meta.taken_time > 0) {
      dateStr = new Date(meta.taken_time * 1000).toLocaleString(undefined, {
        dateStyle: 'medium',
        timeStyle: 'short',
      });
    }
    elements['meta-date-taken'].innerText = dateStr;
  }
}
