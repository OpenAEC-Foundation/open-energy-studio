/**
 * Building 3D View — Isometric canvas-based visualization of the building energy model.
 * Shows walls, roof, floor, windows, PV panels, and thermal bridges.
 * Mouse drag to orbit, scroll to zoom. Export to IFC with geometry.
 */

import { useRef, useEffect, useState, useCallback } from 'react';
import { useEnergy } from '../../context/EnergyContext';
import { useI18n } from '../../i18n/i18n';
import { downloadModelIFC } from '../../core/ifc/IFCModelExporter';
import type { IProject, ISurface } from '../../core/energy/types';
import './Building3DView.css';

// ============================================================
// 3D math types
// ============================================================

interface Vec3 { x: number; y: number; z: number; }

interface Face {
  vertices: Vec3[];
  color: string;
  alpha: number;
  label?: string;
  type: 'wall' | 'roof' | 'floor' | 'window' | 'pv' | 'frame';
  depth: number; // for painter's sort
}

// ============================================================
// Isometric projection
// ============================================================

function projectToScreen(v: Vec3, rotY: number, rotX: number, zoom: number, cx: number, cy: number): { x: number; y: number } {
  // Rotate around Y axis
  const cosY = Math.cos(rotY);
  const sinY = Math.sin(rotY);
  let x = v.x * cosY - v.z * sinY;
  let z = v.x * sinY + v.z * cosY;
  let y = v.y;

  // Rotate around X axis (tilt)
  const cosX = Math.cos(rotX);
  const sinX = Math.sin(rotX);
  const y2 = y * cosX - z * sinX;
  const z2 = y * sinX + z * cosX;
  y = y2;
  z = z2;

  return {
    x: cx + x * zoom,
    y: cy - y * zoom, // flip Y for screen
  };
}

function faceCenter(verts: Vec3[]): Vec3 {
  const n = verts.length;
  let sx = 0, sy = 0, sz = 0;
  for (const v of verts) { sx += v.x; sy += v.y; sz += v.z; }
  return { x: sx / n, y: sy / n, z: sz / n };
}

function rotatedDepth(center: Vec3, rotY: number, rotX: number): number {
  const cosY = Math.cos(rotY);
  const sinY = Math.sin(rotY);
  const z1 = center.x * sinY + center.z * cosY;
  const cosX = Math.cos(rotX);
  const sinX = Math.sin(rotX);
  return center.y * sinX + z1 * cosX;
}

// ============================================================
// Building geometry generation
// ============================================================

const COLORS = {
  wall: '#6B8EC2',
  roof: '#C2756B',
  floor: '#8BC27A',
  window: '#7DD3FC',
  pv: '#3B82F6',
  frame: '#94A3B8',
};

function buildModel(project: IProject): Face[] {
  const faces: Face[] = [];
  if (project.zones.length === 0) return faces;

  // Calculate building dimensions from zones
  const totalFloorArea = project.zones.reduce((s, z) => s + z.floorArea, 0);
  const maxHeight = Math.max(...project.zones.map(z => z.height), 2.6);
  const numStoreys = project.zones.length;
  const buildingHeight = maxHeight * numStoreys;

  // Approximate building footprint as a rectangle
  const footprintArea = totalFloorArea / numStoreys;
  const aspect = 1.5; // width / depth ratio
  const buildingWidth = Math.sqrt(footprintArea * aspect);
  const buildingDepth = footprintArea / buildingWidth;

  const hw = buildingWidth / 2;
  const hd = buildingDepth / 2;

  // Collect all surfaces from all zones
  const allSurfaces: { surface: ISurface; zoneIndex: number }[] = [];
  project.zones.forEach((zone, zi) => {
    zone.surfaces.forEach(s => allSurfaces.push({ surface: s, zoneIndex: zi }));
  });

  // Sort surfaces by type for rendering
  const walls = allSurfaces.filter(s => s.surface.type === 'wall');
  const roofs = allSurfaces.filter(s => s.surface.type === 'roof');
  const floors = allSurfaces.filter(s => s.surface.type === 'floor');

  // --- Floor plane ---
  faces.push({
    vertices: [
      { x: -hw, y: 0, z: -hd },
      { x: hw, y: 0, z: -hd },
      { x: hw, y: 0, z: hd },
      { x: -hw, y: 0, z: hd },
    ],
    color: COLORS.floor,
    alpha: 0.7,
    label: floors.length > 0 ? floors[0].surface.name : 'Vloer',
    type: 'floor',
    depth: 0,
  });

  // --- Walls ---
  // Distribute walls around the building perimeter based on orientation
  const wallOrientationMap: Record<string, { x1: number; z1: number; x2: number; z2: number }> = {
    'N':  { x1: -hw, z1: -hd, x2: hw, z2: -hd },
    'S':  { x1: hw, z1: hd, x2: -hw, z2: hd },
    'E':  { x1: hw, z1: -hd, x2: hw, z2: hd },
    'W':  { x1: -hw, z1: hd, x2: -hw, z2: -hd },
    'NE': { x1: hw, z1: -hd, x2: hw, z2: -hd },
    'SE': { x1: hw, z1: hd, x2: hw, z2: hd },
    'SW': { x1: -hw, z1: hd, x2: -hw, z2: hd },
    'NW': { x1: -hw, z1: -hd, x2: -hw, z2: -hd },
  };

  // Group walls by orientation to stack them
  const wallsByOrientation = new Map<string, typeof walls>();
  for (const w of walls) {
    const key = w.surface.orientation;
    if (!wallsByOrientation.has(key)) wallsByOrientation.set(key, []);
    wallsByOrientation.get(key)!.push(w);
  }

  for (const [orient, wallGroup] of wallsByOrientation) {
    const base = wallOrientationMap[orient];
    if (!base) continue;

    for (let wi = 0; wi < wallGroup.length; wi++) {
      const w = wallGroup[wi];
      const storyBase = w.zoneIndex * maxHeight;
      const wallHeight = project.zones[w.zoneIndex]?.height ?? maxHeight;
      const { x1, z1, x2, z2 } = base;

      faces.push({
        vertices: [
          { x: x1, y: storyBase, z: z1 },
          { x: x2, y: storyBase, z: z2 },
          { x: x2, y: storyBase + wallHeight, z: z2 },
          { x: x1, y: storyBase + wallHeight, z: z1 },
        ],
        color: COLORS.wall,
        alpha: 0.6,
        label: w.surface.name,
        type: 'wall',
        depth: 0,
      });

      // Windows on this wall
      const wallLength = Math.sqrt((x2 - x1) ** 2 + (z2 - z1) ** 2);
      const wallArea = wallLength * wallHeight;
      let windowOffset = 0;

      for (const win of w.surface.windows) {
        const winFraction = win.area / Math.max(wallArea, 1);
        const winWidth = wallLength * Math.min(winFraction, 0.8);
        const winHeight = win.area / Math.max(winWidth, 0.5);
        const clampedHeight = Math.min(winHeight, wallHeight * 0.7);

        // Position window on wall
        const startFrac = 0.1 + windowOffset;
        const dx = x2 - x1;
        const dz = z2 - z1;

        const wx1 = x1 + dx * startFrac;
        const wz1 = z1 + dz * startFrac;
        const endFrac = startFrac + winWidth / Math.max(wallLength, 0.1);
        const wx2 = x1 + dx * Math.min(endFrac, 0.95);
        const wz2 = z1 + dz * Math.min(endFrac, 0.95);

        const sillHeight = storyBase + wallHeight * 0.25;

        // Small offset from wall so windows render on top
        const normal = orient === 'N' || orient === 'NE' || orient === 'NW' ? 0.05 :
                        orient === 'S' || orient === 'SE' || orient === 'SW' ? -0.05 :
                        orient === 'E' ? -0.05 : 0.05;
        const nx = (orient === 'E' || orient === 'W' || orient === 'NE' || orient === 'SE' || orient === 'SW' || orient === 'NW') ? normal : 0;
        const nz = (orient === 'N' || orient === 'S' || orient === 'NE' || orient === 'NW' || orient === 'SE' || orient === 'SW') ? normal : 0;

        faces.push({
          vertices: [
            { x: wx1 + nx, y: sillHeight, z: wz1 + nz },
            { x: wx2 + nx, y: sillHeight, z: wz2 + nz },
            { x: wx2 + nx, y: sillHeight + clampedHeight, z: wz2 + nz },
            { x: wx1 + nx, y: sillHeight + clampedHeight, z: wz1 + nz },
          ],
          color: COLORS.window,
          alpha: 0.5,
          label: win.name,
          type: 'window',
          depth: 0,
        });

        windowOffset += winWidth / Math.max(wallLength, 0.1) + 0.05;
      }
    }
  }

  // --- Roof ---
  if (roofs.length > 0) {
    // Check if we have sloped roof (different orientations)
    const roofOrientations = new Set(roofs.map(r => r.surface.orientation));
    const hasRidge = roofOrientations.size >= 2 && !roofOrientations.has('horizontal');

    if (hasRidge) {
      // Pitched roof - create two sloped planes with a ridge
      const ridgeHeight = buildingHeight + Math.min(buildingDepth, buildingWidth) * 0.3;

      // East-facing slope
      faces.push({
        vertices: [
          { x: hw, y: buildingHeight, z: -hd },
          { x: hw, y: buildingHeight, z: hd },
          { x: 0, y: ridgeHeight, z: hd },
          { x: 0, y: ridgeHeight, z: -hd },
        ],
        color: COLORS.roof,
        alpha: 0.7,
        label: roofs.find(r => r.surface.orientation === 'E')?.surface.name ?? 'Dak Oost',
        type: 'roof',
        depth: 0,
      });

      // West-facing slope
      faces.push({
        vertices: [
          { x: -hw, y: buildingHeight, z: hd },
          { x: -hw, y: buildingHeight, z: -hd },
          { x: 0, y: ridgeHeight, z: -hd },
          { x: 0, y: ridgeHeight, z: hd },
        ],
        color: COLORS.roof,
        alpha: 0.7,
        label: roofs.find(r => r.surface.orientation === 'W')?.surface.name ?? 'Dak West',
        type: 'roof',
        depth: 0,
      });

      // Gable ends (triangles)
      faces.push({
        vertices: [
          { x: -hw, y: buildingHeight, z: -hd },
          { x: hw, y: buildingHeight, z: -hd },
          { x: 0, y: ridgeHeight, z: -hd },
        ],
        color: COLORS.wall,
        alpha: 0.5,
        type: 'frame',
        depth: 0,
      });
      faces.push({
        vertices: [
          { x: hw, y: buildingHeight, z: hd },
          { x: -hw, y: buildingHeight, z: hd },
          { x: 0, y: ridgeHeight, z: hd },
        ],
        color: COLORS.wall,
        alpha: 0.5,
        type: 'frame',
        depth: 0,
      });

      // PV panels on roof
      for (const pv of project.solarPV) {
        const pvFraction = Math.min(pv.area / (buildingWidth * buildingDepth / 2), 0.8);
        const isEast = pv.orientation === 'E' || pv.orientation === 'NE' || pv.orientation === 'SE';
        const pvInset = 0.15;

        if (isEast) {
          const bx = hw * (1 - pvInset);
          const tx = hw * pvInset;
          const bz = -hd * (1 - pvInset);
          const tz = hd * (pvFraction - pvInset);
          const by = buildingHeight + (ridgeHeight - buildingHeight) * pvInset;
          const ty = buildingHeight + (ridgeHeight - buildingHeight) * (1 - pvInset);
          faces.push({
            vertices: [
              { x: bx, y: by + 0.08, z: bz },
              { x: bx, y: by + 0.08, z: tz },
              { x: tx, y: ty + 0.08, z: tz },
              { x: tx, y: ty + 0.08, z: bz },
            ],
            color: COLORS.pv,
            alpha: 0.8,
            label: pv.name,
            type: 'pv',
            depth: 0,
          });
        } else {
          const bx = -hw * (1 - pvInset);
          const tx = -hw * pvInset;
          const bz = -hd * (1 - pvInset);
          const tz = hd * (pvFraction - pvInset);
          const by = buildingHeight + (ridgeHeight - buildingHeight) * pvInset;
          const ty = buildingHeight + (ridgeHeight - buildingHeight) * (1 - pvInset);
          faces.push({
            vertices: [
              { x: bx, y: by + 0.08, z: bz },
              { x: bx, y: by + 0.08, z: tz },
              { x: tx, y: ty + 0.08, z: tz },
              { x: tx, y: ty + 0.08, z: bz },
            ],
            color: COLORS.pv,
            alpha: 0.8,
            label: pv.name,
            type: 'pv',
            depth: 0,
          });
        }
      }
    } else {
      // Flat roof
      faces.push({
        vertices: [
          { x: -hw, y: buildingHeight, z: -hd },
          { x: hw, y: buildingHeight, z: -hd },
          { x: hw, y: buildingHeight, z: hd },
          { x: -hw, y: buildingHeight, z: hd },
        ],
        color: COLORS.roof,
        alpha: 0.7,
        label: 'Dak',
        type: 'roof',
        depth: 0,
      });

      // PV panels on flat roof
      let pvXOffset = -hw * 0.8;
      for (const pv of project.solarPV) {
        const pvW = Math.min(Math.sqrt(pv.area), buildingWidth * 0.4);
        const pvD = pv.area / pvW;
        faces.push({
          vertices: [
            { x: pvXOffset, y: buildingHeight + 0.1, z: -hd * 0.3 },
            { x: pvXOffset + pvW, y: buildingHeight + 0.1, z: -hd * 0.3 },
            { x: pvXOffset + pvW, y: buildingHeight + 0.3, z: -hd * 0.3 + pvD },
            { x: pvXOffset, y: buildingHeight + 0.3, z: -hd * 0.3 + pvD },
          ],
          color: COLORS.pv,
          alpha: 0.8,
          label: pv.name,
          type: 'pv',
          depth: 0,
        });
        pvXOffset += pvW + 0.5;
      }
    }
  }

  return faces;
}

// ============================================================
// Canvas renderer
// ============================================================

function renderScene(
  ctx: CanvasRenderingContext2D,
  faces: Face[],
  width: number,
  height: number,
  rotY: number,
  rotX: number,
  zoom: number,
  hoveredFace: number | null,
  isDark: boolean,
) {
  // Clear
  ctx.clearRect(0, 0, width, height);

  const cx = width / 2;
  const cy = height / 2;

  // Draw grid on ground plane
  ctx.save();
  ctx.strokeStyle = isDark ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.06)';
  ctx.lineWidth = 1;
  const gridSize = 20;
  const gridStep = 2;
  for (let i = -gridSize; i <= gridSize; i += gridStep) {
    const p1 = projectToScreen({ x: i, y: 0, z: -gridSize }, rotY, rotX, zoom, cx, cy);
    const p2 = projectToScreen({ x: i, y: 0, z: gridSize }, rotY, rotX, zoom, cx, cy);
    ctx.beginPath();
    ctx.moveTo(p1.x, p1.y);
    ctx.lineTo(p2.x, p2.y);
    ctx.stroke();

    const p3 = projectToScreen({ x: -gridSize, y: 0, z: i }, rotY, rotX, zoom, cx, cy);
    const p4 = projectToScreen({ x: gridSize, y: 0, z: i }, rotY, rotX, zoom, cx, cy);
    ctx.beginPath();
    ctx.moveTo(p3.x, p3.y);
    ctx.lineTo(p4.x, p4.y);
    ctx.stroke();
  }
  ctx.restore();

  // Draw axes
  ctx.save();
  ctx.lineWidth = 1.5;
  const o = projectToScreen({ x: 0, y: 0, z: 0 }, rotY, rotX, zoom, cx, cy);
  const axLen = 3;
  // X axis (red)
  const ax = projectToScreen({ x: axLen, y: 0, z: 0 }, rotY, rotX, zoom, cx, cy);
  ctx.strokeStyle = '#EF4444';
  ctx.beginPath(); ctx.moveTo(o.x, o.y); ctx.lineTo(ax.x, ax.y); ctx.stroke();
  ctx.fillStyle = '#EF4444';
  ctx.font = '10px monospace';
  ctx.fillText('X', ax.x + 4, ax.y);
  // Y axis (green, up)
  const ay = projectToScreen({ x: 0, y: axLen, z: 0 }, rotY, rotX, zoom, cx, cy);
  ctx.strokeStyle = '#22C55E';
  ctx.beginPath(); ctx.moveTo(o.x, o.y); ctx.lineTo(ay.x, ay.y); ctx.stroke();
  ctx.fillStyle = '#22C55E';
  ctx.fillText('Y', ay.x + 4, ay.y);
  // Z axis (blue)
  const az = projectToScreen({ x: 0, y: 0, z: axLen }, rotY, rotX, zoom, cx, cy);
  ctx.strokeStyle = '#3B82F6';
  ctx.beginPath(); ctx.moveTo(o.x, o.y); ctx.lineTo(az.x, az.y); ctx.stroke();
  ctx.fillStyle = '#3B82F6';
  ctx.fillText('Z', az.x + 4, az.y);
  ctx.restore();

  // Compute depth for each face and sort (painter's algorithm)
  const sortedFaces = faces.map((f, i) => {
    const center = faceCenter(f.vertices);
    return { face: f, index: i, depth: rotatedDepth(center, rotY, rotX) };
  });
  sortedFaces.sort((a, b) => a.depth - b.depth);

  // Draw faces
  for (const { face, index } of sortedFaces) {
    const projected = face.vertices.map(v => projectToScreen(v, rotY, rotX, zoom, cx, cy));

    ctx.beginPath();
    ctx.moveTo(projected[0].x, projected[0].y);
    for (let i = 1; i < projected.length; i++) {
      ctx.lineTo(projected[i].x, projected[i].y);
    }
    ctx.closePath();

    // Fill
    ctx.globalAlpha = face.alpha;
    ctx.fillStyle = face.color;
    if (hoveredFace === index) {
      ctx.fillStyle = '#FCD34D';
      ctx.globalAlpha = 0.8;
    }
    ctx.fill();

    // Stroke
    ctx.globalAlpha = 1;
    ctx.strokeStyle = isDark ? 'rgba(255,255,255,0.3)' : 'rgba(0,0,0,0.3)';
    ctx.lineWidth = index === hoveredFace ? 2 : 1;
    ctx.stroke();
  }

  // Draw labels for hovered face
  if (hoveredFace !== null) {
    const face = faces[hoveredFace];
    if (face.label) {
      const center = faceCenter(face.vertices);
      const p = projectToScreen(center, rotY, rotX, zoom, cx, cy);
      ctx.save();
      ctx.font = '11px system-ui, sans-serif';
      ctx.fillStyle = isDark ? '#FCD34D' : '#92400E';
      const metrics = ctx.measureText(face.label);
      const pad = 4;
      const bgColor = isDark ? 'rgba(0,0,0,0.7)' : 'rgba(255,255,255,0.85)';
      ctx.fillStyle = bgColor;
      ctx.fillRect(p.x - pad, p.y - 14, metrics.width + pad * 2, 18);
      ctx.fillStyle = isDark ? '#FCD34D' : '#92400E';
      ctx.fillText(face.label, p.x, p.y);
      ctx.restore();
    }
  }
}

// ============================================================
// Hit test
// ============================================================

function pointInPolygon(px: number, py: number, polygon: { x: number; y: number }[]): boolean {
  let inside = false;
  for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i++) {
    const xi = polygon[i].x, yi = polygon[i].y;
    const xj = polygon[j].x, yj = polygon[j].y;
    if (((yi > py) !== (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi)) {
      inside = !inside;
    }
  }
  return inside;
}

// ============================================================
// Component
// ============================================================

export function Building3DView() {
  const { state } = useEnergy();
  const { t } = useI18n();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [rotY, setRotY] = useState(-Math.PI / 6);
  const [rotX, setRotX] = useState(Math.PI / 6);
  const [zoom, setZoom] = useState(30);
  const [hoveredFace, setHoveredFace] = useState<number | null>(null);
  const isDragging = useRef(false);
  const lastMouse = useRef({ x: 0, y: 0 });
  const facesRef = useRef<Face[]>([]);

  const project = state.project;

  // Rebuild model when project changes
  useEffect(() => {
    facesRef.current = buildModel(project);
  }, [project]);

  // Render loop
  useEffect(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const resize = () => {
      const rect = container.getBoundingClientRect();
      const dpr = window.devicePixelRatio || 1;
      canvas.width = rect.width * dpr;
      canvas.height = rect.height * dpr;
      canvas.style.width = `${rect.width}px`;
      canvas.style.height = `${rect.height}px`;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    };

    resize();
    const observer = new ResizeObserver(resize);
    observer.observe(container);

    return () => observer.disconnect();
  }, []);

  // Redraw on state change
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const rect = canvas.getBoundingClientRect();
    const isDark = document.documentElement.dataset.theme === 'dark';
    renderScene(ctx, facesRef.current, rect.width, rect.height, rotY, rotX, zoom, hoveredFace, isDark);
  }, [rotY, rotX, zoom, hoveredFace, project]);

  // Mouse handlers
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    isDragging.current = true;
    lastMouse.current = { x: e.clientX, y: e.clientY };
  }, []);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (isDragging.current) {
      const dx = e.clientX - lastMouse.current.x;
      const dy = e.clientY - lastMouse.current.y;
      setRotY(prev => prev - dx * 0.005);
      setRotX(prev => Math.max(-Math.PI / 2.5, Math.min(Math.PI / 2.5, prev - dy * 0.005)));
      lastMouse.current = { x: e.clientX, y: e.clientY };
    } else {
      // Hit test for hover
      const canvas = canvasRef.current;
      if (!canvas) return;
      const rect = canvas.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;
      const cx = rect.width / 2;
      const cy = rect.height / 2;

      // Check faces in reverse depth order (front first)
      const sortedFaces = facesRef.current.map((f, i) => {
        const center = faceCenter(f.vertices);
        return { face: f, index: i, depth: rotatedDepth(center, rotY, rotX) };
      });
      sortedFaces.sort((a, b) => b.depth - a.depth);

      let found = -1;
      for (const { face, index } of sortedFaces) {
        const projected = face.vertices.map(v => projectToScreen(v, rotY, rotX, zoom, cx, cy));
        if (pointInPolygon(mx, my, projected)) {
          found = index;
          break;
        }
      }
      setHoveredFace(found >= 0 ? found : null);
    }
  }, [rotY, rotX, zoom]);

  const handleMouseUp = useCallback(() => {
    isDragging.current = false;
  }, []);

  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();
    setZoom(prev => Math.max(5, Math.min(100, prev - e.deltaY * 0.05)));
  }, []);

  const handleReset = useCallback(() => {
    setRotY(-Math.PI / 6);
    setRotX(Math.PI / 6);
    setZoom(30);
  }, []);

  const handleExportIFC = useCallback(() => {
    downloadModelIFC(project);
  }, [project]);

  if (project.zones.length === 0) {
    return (
      <div className="building-3d-view">
        <div className="building-3d-empty">{t('model3d.noData')}</div>
      </div>
    );
  }

  return (
    <div className="building-3d-view">
      <div className="building-3d-toolbar">
        <button onClick={handleReset}>{t('model3d.resetView')}</button>
        <div className="toolbar-separator" />
        <button onClick={handleExportIFC}>{t('model3d.exportIFC')}</button>
      </div>

      <div className="building-3d-canvas-container" ref={containerRef}>
        <canvas
          ref={canvasRef}
          onMouseDown={handleMouseDown}
          onMouseMove={handleMouseMove}
          onMouseUp={handleMouseUp}
          onMouseLeave={handleMouseUp}
          onWheel={handleWheel}
        />

        <div className="building-3d-legend">
          <div className="building-3d-legend-item">
            <div className="building-3d-legend-swatch" style={{ background: COLORS.wall }} />
            <span>{t('surfaceType.wall')}</span>
          </div>
          <div className="building-3d-legend-item">
            <div className="building-3d-legend-swatch" style={{ background: COLORS.roof }} />
            <span>{t('surfaceType.roof')}</span>
          </div>
          <div className="building-3d-legend-item">
            <div className="building-3d-legend-swatch" style={{ background: COLORS.floor }} />
            <span>{t('surfaceType.floor')}</span>
          </div>
          <div className="building-3d-legend-item">
            <div className="building-3d-legend-swatch" style={{ background: COLORS.window }} />
            <span>{t('ribbon.windows')}</span>
          </div>
          <div className="building-3d-legend-item">
            <div className="building-3d-legend-swatch" style={{ background: COLORS.pv }} />
            <span>{t('ribbon.solarPV')}</span>
          </div>
        </div>

        <div className="building-3d-info">
          {t('model3d.dragToRotate')}
        </div>
      </div>
    </div>
  );
}
