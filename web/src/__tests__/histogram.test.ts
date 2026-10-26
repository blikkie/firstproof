import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { HistogramWidget } from '../HistogramWidget';
import type { EvHistogramRecord } from '../WasmBridge';

describe('HistogramWidget (Sensitometric EV Zone Histogram Parity)', () => {
  let container: HTMLElement;
  let widget: HistogramWidget;
  let onToggleClippingOverlay: ReturnType<typeof vi.fn>;
  let onToggleCollapse: ReturnType<typeof vi.fn>;

  const mockRecord: EvHistogramRecord = {
    bins_luminance: new Array(120).fill(0.1),
    bins_r: new Array(120).fill(0.08),
    bins_g: new Array(120).fill(0.12),
    bins_b: new Array(120).fill(0.05),
    shadow_clip_percent: 1.5,
    sensor_clip_percent: 2.1,
    display_rolloff_percent: 5.6,
    ev_min: -8.0,
    ev_max: 4.0,
    num_bins: 120,
  };

  beforeEach(() => {
    container = document.createElement('div');
    document.body.appendChild(container);

    onToggleClippingOverlay = vi.fn();
    onToggleCollapse = vi.fn();

    widget = new HistogramWidget(container, {
      onToggleClippingOverlay,
      onToggleCollapse,
    });
  });

  afterEach(() => {
    document.body.innerHTML = '';
    vi.restoreAllMocks();
  });

  it('renders the 2-line header, graph canvas, and 2-line scale legend skeleton', () => {
    expect(container.classList.contains('histogram-card')).toBe(true);
    expect(container.querySelector('.histogram-header-line1')).not.toBeNull();
    expect(container.querySelector('.histogram-header-line2')).not.toBeNull();
    expect(container.querySelector('.histogram-canvas')).not.toBeNull();
    expect(container.querySelector('.histogram-scale-legend')).not.toBeNull();

    const title = container.querySelector('#histogram-title-readout');
    expect(title?.textContent?.trim()).toBe('HISTOGRAM (EV)');

    const scaleStops = container.querySelectorAll('.scale-stop');
    expect(scaleStops.length).toBe(3);
    expect(scaleStops[0].textContent).toContain('−8 EV');
    expect(scaleStops[1].textContent).toContain('0 EV (18%)');
    expect(scaleStops[2].textContent).toContain('+4 EV');

    const scaleZones = container.querySelectorAll('.scale-zone');
    expect(scaleZones.length).toBe(3);
    expect(scaleZones[0].textContent).toContain('Zone 0');
    expect(scaleZones[1].textContent).toContain('Zone V');
    expect(scaleZones[2].textContent).toContain('Zone VIII+');
  });

  it('updates telemetry badges and rolloff tag when a record is provided', () => {
    widget.setRecord(mockRecord);

    const shadowVal = container.querySelector('#hist-shadow-val');
    expect(shadowVal?.textContent).toBe('Floor 1.5%');

    const sensorVal = container.querySelector('#hist-sensor-val');
    expect(sensorVal?.textContent).toBe('Clip 2.1%');

    const agxVal = container.querySelector('#hist-agx-val');
    expect(agxVal?.textContent).toBe('5.6%');
    expect(
      container.querySelector('#hist-badge-agx')?.classList.contains('hidden'),
    ).toBe(false);

    // Active state classes for clipping
    expect(
      container
        .querySelector('#hist-badge-shadow')
        ?.classList.contains('badge-active'),
    ).toBe(true);
    expect(
      container
        .querySelector('#hist-badge-sensor')
        ?.classList.contains('badge-active'),
    ).toBe(true);
  });

  it('triggers onToggleClippingOverlay when clicking the shadow or sensor badge', () => {
    const shadowBtn = container.querySelector(
      '#hist-badge-shadow',
    ) as HTMLButtonElement;
    shadowBtn.click();
    expect(onToggleClippingOverlay).toHaveBeenCalledTimes(1);

    const sensorBtn = container.querySelector(
      '#hist-badge-sensor',
    ) as HTMLButtonElement;
    sensorBtn.click();
    expect(onToggleClippingOverlay).toHaveBeenCalledTimes(2);
  });

  it('collapses and expands the graph and scale legend when toggling collapse', () => {
    expect(widget.getIsCollapsed()).toBe(false);

    const collapseBtn = container.querySelector(
      '#histogram-collapse-btn',
    ) as HTMLButtonElement;
    collapseBtn.click();

    expect(widget.getIsCollapsed()).toBe(true);
    expect(onToggleCollapse).toHaveBeenCalledWith(true);

    const graphContainer = container.querySelector(
      '#histogram-graph-container',
    );
    const scaleLegend = container.querySelector('#histogram-scale-legend');
    expect(graphContainer?.classList.contains('hidden')).toBe(true);
    expect(scaleLegend?.classList.contains('hidden')).toBe(true);

    // Toggle back
    collapseBtn.click();
    expect(widget.getIsCollapsed()).toBe(false);
    expect(onToggleCollapse).toHaveBeenCalledWith(false);
    expect(graphContainer?.classList.contains('hidden')).toBe(false);
    expect(scaleLegend?.classList.contains('hidden')).toBe(false);
  });

  it('updates Line 1 with Ansel Adams zone readout on canvas hover', () => {
    const canvas = container.querySelector(
      '.histogram-canvas',
    ) as HTMLCanvasElement;
    vi.spyOn(canvas, 'getBoundingClientRect').mockReturnValue({
      left: 0,
      top: 0,
      width: 240,
      height: 64,
      right: 240,
      bottom: 64,
      x: 0,
      y: 0,
      toJSON: () => {},
    });

    // Hover at 66.7% across the width (Zone V / 0.0 EV)
    const mouseMoveEvent = new MouseEvent('mousemove', {
      clientX: 160,
      clientY: 32,
    });
    canvas.dispatchEvent(mouseMoveEvent);

    const titleEl = container.querySelector('#histogram-title-readout');
    expect(titleEl?.textContent).toContain('Zone V');
    expect(titleEl?.textContent).toContain('18% Middle Grey');

    // Mouse leave restores idle title
    const mouseLeaveEvent = new MouseEvent('mouseleave');
    canvas.dispatchEvent(mouseLeaveEvent);
    expect(titleEl?.textContent?.trim()).toBe('HISTOGRAM (EV)');
  });
});
