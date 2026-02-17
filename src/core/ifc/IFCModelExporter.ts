/**
 * IFC Model Exporter — Generates IFC 4x3 with actual building geometry
 * Creates IfcWall, IfcRoof, IfcSlab, IfcWindow entities with extruded geometry.
 * This produces a viewable IFC model (unlike IFCEnergyExporter which only exports property sets).
 */

import type { IProject } from '../energy/types';

// ============================================================
// IFC entity types (reused from IFCEnergyExporter pattern)
// ============================================================

type AttrVal = string | number | boolean | null | IFCEnt | IFCEnt[] | number[];

interface IFCEnt {
  id: number;
  type: string;
  attrs: AttrVal[];
  label?: string;
}

interface Model {
  ents: Map<number, IFCEnt>;
  nextId: number;
}

function ent(m: Model, type: string, attrs: AttrVal[], label?: string): IFCEnt {
  const e: IFCEnt = { id: m.nextId++, type, attrs, label };
  m.ents.set(e.id, e);
  return e;
}

function fmtVal(v: AttrVal | undefined): string {
  if (v === null || v === undefined) return '$';
  if (typeof v === 'boolean') return v ? '.T.' : '.F.';
  if (typeof v === 'string') {
    if (v.startsWith('.') && v.endsWith('.')) return v;
    if (v.startsWith('#')) return v;
    if (v === '*') return '*';
    return `'${v.replace(/'/g, "''")}'`;
  }
  if (typeof v === 'number') {
    if (Number.isInteger(v)) return v.toString();
    return v.toExponential(6).toUpperCase().replace('E+', 'E');
  }
  if (Array.isArray(v)) {
    if (v.length === 0) return '()';
    if (typeof v[0] === 'number') return `(${(v as number[]).map(n => fmtVal(n)).join(',')})`;
    return `(${(v as IFCEnt[]).map(e => fmtVal(e)).join(',')})`;
  }
  if (typeof v === 'object' && 'id' in v) return `#${v.id}`;
  return '$';
}

function toSTEP(m: Model, projectName: string): string {
  const now = new Date().toISOString().replace(/[-:]/g, '').split('.')[0];
  const lines = [
    'ISO-10303-21;',
    'HEADER;',
    `FILE_DESCRIPTION(('ViewDefinition [CoordinationView_V2.0]'),'2;1');`,
    `FILE_NAME('${projectName}.ifc','${now}',('Open-Energy-Studio User'),('Open-Energy-Studio'),'','Open-Energy-Studio','');`,
    `FILE_SCHEMA(('IFC4X3_ADD2'));`,
    'ENDSEC;',
    '',
    'DATA;',
  ];

  const sorted = Array.from(m.ents.values()).sort((a, b) => a.id - b.id);
  for (const e of sorted) {
    const attrs = e.attrs.map(a => fmtVal(a)).join(',');
    const line = `#${e.id}=${e.type}(${attrs});`;
    lines.push(e.label ? `${line} /* ${e.label} */` : line);
  }

  lines.push('ENDSEC;');
  lines.push('END-ISO-10303-21;');
  return lines.join('\n');
}

function guid(): string {
  const c = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_$';
  let g = '';
  for (let i = 0; i < 22; i++) g += c[Math.floor(Math.random() * 64)];
  return g;
}

// ============================================================
// Geometry helpers
// ============================================================

function makeCartesianPoint(m: Model, coords: number[]): IFCEnt {
  return ent(m, 'IFCCARTESIANPOINT', [coords]);
}

function makeDirection(m: Model, dirs: number[]): IFCEnt {
  return ent(m, 'IFCDIRECTION', [dirs]);
}

function makePlacement3D(m: Model, origin: number[], zDir?: number[], xDir?: number[]): IFCEnt {
  const pt = makeCartesianPoint(m, origin);
  const z = zDir ? makeDirection(m, zDir) : null;
  const x = xDir ? makeDirection(m, xDir) : null;
  return ent(m, 'IFCAXIS2PLACEMENT3D', [pt, z, x]);
}

function makeLocalPlacement(m: Model, relative: IFCEnt | null, origin: number[], zDir?: number[], xDir?: number[]): IFCEnt {
  const placement = makePlacement3D(m, origin, zDir, xDir);
  return ent(m, 'IFCLOCALPLACEMENT', [relative, placement]);
}

function makeExtrudedAreaSolid(m: Model, points: number[][], height: number): IFCEnt {
  // Create polyline profile
  const cartesianPoints = points.map(p => makeCartesianPoint(m, p));
  // Close the polyline
  cartesianPoints.push(cartesianPoints[0]);
  const polyline = ent(m, 'IFCPOLYLINE', [cartesianPoints]);
  const profile = ent(m, 'IFCARBITRARYCLOSEDPROFILEDEF', ['.AREA.', null, polyline]);

  // Extrusion direction (up)
  const dir = makeDirection(m, [0.0, 0.0, 1.0]);
  const pos = makePlacement3D(m, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);

  return ent(m, 'IFCEXTRUDEDAREASOLID', [profile, pos, dir, height]);
}

function makeShapeRepresentation(m: Model, context: IFCEnt, solid: IFCEnt): IFCEnt {
  const shape = ent(m, 'IFCSHAPEREPRESENTATION', [context, 'Body', 'SweptSolid', [solid]]);
  return ent(m, 'IFCPRODUCTDEFINITIONSHAPE', [null, null, [shape]]);
}

// ============================================================
// Main export
// ============================================================

export function exportModelToIFC(project: IProject): string {
  const m: Model = { ents: new Map(), nextId: 1 };

  // === Infrastructure ===
  const person = ent(m, 'IFCPERSON', [null, null, null, null, null, null, null, null]);
  const org = ent(m, 'IFCORGANIZATION', [null, 'Open-Energy-Studio', null, null, null]);
  const personOrg = ent(m, 'IFCPERSONANDORGANIZATION', [person, org, null]);
  const app = ent(m, 'IFCAPPLICATION', [org, '1.0', 'Open-Energy-Studio', 'OES']);
  const history = ent(m, 'IFCOWNERHISTORY', [
    personOrg, app, null, '.READWRITE.', null, null, null, Math.floor(Date.now() / 1000),
  ]);

  // Units
  const uLen = ent(m, 'IFCSIUNIT', ['*', '.LENGTHUNIT.', null, '.METRE.']);
  const uArea = ent(m, 'IFCSIUNIT', ['*', '.AREAUNIT.', null, '.SQUARE_METRE.']);
  const uVol = ent(m, 'IFCSIUNIT', ['*', '.VOLUMEUNIT.', null, '.CUBIC_METRE.']);
  const uAngle = ent(m, 'IFCSIUNIT', ['*', '.PLANEANGLEUNIT.', null, '.RADIAN.']);
  const units = ent(m, 'IFCUNITASSIGNMENT', [[uLen, uArea, uVol, uAngle]]);

  // Geometric context
  const origin = makeCartesianPoint(m, [0.0, 0.0, 0.0]);
  const axisZ = makeDirection(m, [0.0, 0.0, 1.0]);
  const axisX = makeDirection(m, [1.0, 0.0, 0.0]);
  const worldCS = ent(m, 'IFCAXIS2PLACEMENT3D', [origin, axisZ, axisX]);
  const geoCtx = ent(m, 'IFCGEOMETRICREPRESENTATIONCONTEXT', [
    'Model', 'Model', 3, 1.0E-5, worldCS, null,
  ]);
  const bodyCtx = ent(m, 'IFCGEOMETRICREPRESENTATIONSUBCONTEXT', [
    'Body', 'Model', null, null, null, null, geoCtx, null, '.MODEL_VIEW.', null,
  ]);

  // Project → Site → Building → Storey
  const ifcProject = ent(m, 'IFCPROJECT', [
    guid(), history, project.name || 'Building Model', project.description || null,
    null, null, null, [geoCtx], units,
  ], 'Project');

  const sitePlacement = makeLocalPlacement(m, null, [0.0, 0.0, 0.0]);
  const site = ent(m, 'IFCSITE', [
    guid(), history, 'Site', project.address || null, null, sitePlacement,
    null, null, '.ELEMENT.', null, null, null, null, null,
  ], 'Site');

  const buildingPlacement = makeLocalPlacement(m, sitePlacement, [0.0, 0.0, 0.0]);
  const building = ent(m, 'IFCBUILDING', [
    guid(), history, project.name || 'Building',
    `${project.city || ''} - ${project.address || ''}`,
    null, buildingPlacement, null, null, '.ELEMENT.', null, null, null,
  ], 'Building');

  ent(m, 'IFCRELAGGREGATES', [guid(), history, null, null, ifcProject, [site]]);
  ent(m, 'IFCRELAGGREGATES', [guid(), history, null, null, site, [building]]);

  // === Create storeys and building elements ===
  const totalFloorArea = project.zones.reduce((s, z) => s + z.floorArea, 0);
  const numStoreys = project.zones.length;
  const footprintArea = totalFloorArea / Math.max(numStoreys, 1);
  const aspect = 1.5;
  const bw = Math.sqrt(footprintArea * aspect);
  const bd = footprintArea / bw;
  const hw = bw / 2;
  const hd = bd / 2;

  const storeys: IFCEnt[] = [];
  const storeyProducts = new Map<number, IFCEnt[]>();

  for (let zi = 0; zi < project.zones.length; zi++) {
    const zone = project.zones[zi];
    const maxH = zone.height;
    const baseZ = zi * maxH;

    const storeyPlacement = makeLocalPlacement(m, buildingPlacement, [0.0, 0.0, baseZ]);
    const storey = ent(m, 'IFCBUILDINGSTOREY', [
      guid(), history, zone.name, null, null, storeyPlacement, null, null,
      '.ELEMENT.', baseZ,
    ], `Storey_${zone.name}`);
    storeys.push(storey);
    storeyProducts.set(zi, []);

    // Wall positions by orientation
    const wallPositions: Record<string, { pts: number[][]; xDir: number[]; length: number }> = {
      'N':  { pts: [[-hw, -hd], [hw, -hd]], xDir: [1, 0, 0], length: bw },
      'S':  { pts: [[hw, hd], [-hw, hd]], xDir: [-1, 0, 0], length: bw },
      'E':  { pts: [[hw, -hd], [hw, hd]], xDir: [0, 1, 0], length: bd },
      'W':  { pts: [[-hw, hd], [-hw, -hd]], xDir: [0, -1, 0], length: bd },
    };

    for (const surface of zone.surfaces) {
      if (surface.type === 'wall') {
        const wp = wallPositions[surface.orientation] || wallPositions['N'];
        const wallThickness = 0.3;
        const wallLength = wp.length;

        // Wall solid: extrude a rectangle along wall height
        const solid = makeExtrudedAreaSolid(m,
          [[0, 0], [wallLength, 0], [wallLength, wallThickness], [0, wallThickness]],
          maxH,
        );
        const shape = makeShapeRepresentation(m, bodyCtx, solid);

        // Wall placement: position at wall start, rotate to face correct direction
        const ox = wp.pts[0][0];
        const oy = wp.pts[0][1];
        const wallPlacement = makeLocalPlacement(m, storeyPlacement, [ox, oy, 0.0],
          [0.0, 0.0, 1.0], wp.xDir);

        const wall = ent(m, 'IFCWALL', [
          guid(), history, surface.name, null, null, wallPlacement, shape, null,
          '.NOTDEFINED.',
        ], `Wall_${surface.name}`);

        storeyProducts.get(zi)!.push(wall);

        // Windows on this wall
        let winOffset = wallLength * 0.1;
        for (const win of surface.windows) {
          const winWidth = Math.min(Math.sqrt(win.area * 1.5), wallLength * 0.4);
          const winHeight = win.area / winWidth;
          const sillHeight = maxH * 0.25;

          const winSolid = makeExtrudedAreaSolid(m,
            [[0, 0], [winWidth, 0], [winWidth, wallThickness + 0.05], [0, wallThickness + 0.05]],
            Math.min(winHeight, maxH * 0.6),
          );
          const winShape = makeShapeRepresentation(m, bodyCtx, winSolid);

          const winPlacement = makeLocalPlacement(m, wallPlacement,
            [winOffset, -0.025, sillHeight]);

          const window = ent(m, 'IFCWINDOW', [
            guid(), history, win.name, null, null, winPlacement, winShape, null,
            '.NOTDEFINED.', winHeight, winWidth,
          ], `Window_${win.name}`);

          storeyProducts.get(zi)!.push(window);
          winOffset += winWidth + 0.5;
        }
      }

      if (surface.type === 'floor' && zi === 0) {
        const slabThickness = 0.3;
        const solid = makeExtrudedAreaSolid(m,
          [[-hw, -hd], [hw, -hd], [hw, hd], [-hw, hd]],
          slabThickness,
        );
        const shape = makeShapeRepresentation(m, bodyCtx, solid);
        const slabPlacement = makeLocalPlacement(m, storeyPlacement, [0.0, 0.0, -slabThickness]);

        const slab = ent(m, 'IFCSLAB', [
          guid(), history, surface.name, null, null, slabPlacement, shape, null,
          '.FLOOR.',
        ], `Slab_${surface.name}`);

        storeyProducts.get(zi)!.push(slab);
      }

      if (surface.type === 'roof') {
        // Simplified: flat roof slab at top
        const roofThickness = 0.25;
        const solid = makeExtrudedAreaSolid(m,
          [[-hw, -hd], [hw, -hd], [hw, hd], [-hw, hd]],
          roofThickness,
        );
        const shape = makeShapeRepresentation(m, bodyCtx, solid);
        const roofPlacement = makeLocalPlacement(m, storeyPlacement, [0.0, 0.0, maxH]);

        const roof = ent(m, 'IFCROOF', [
          guid(), history, surface.name, null, null, roofPlacement, shape, null,
          '.FLAT_ROOF.',
        ], `Roof_${surface.name}`);

        storeyProducts.get(zi)!.push(roof);
      }
    }
  }

  // Aggregate storeys into building
  if (storeys.length > 0) {
    ent(m, 'IFCRELAGGREGATES', [guid(), history, null, null, building, storeys]);
  }

  // Contain products in storeys
  for (const [zi, products] of storeyProducts) {
    if (products.length > 0) {
      ent(m, 'IFCRELCONTAINEDINSPATIALSTRUCTURE', [
        guid(), history, null, null, products, storeys[zi],
      ]);
    }
  }

  return toSTEP(m, project.name || 'Building-Model');
}

// ============================================================
// Download
// ============================================================

export function downloadModelIFC(project: IProject): void {
  const step = exportModelToIFC(project);
  const blob = new Blob([step], { type: 'application/x-step' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `Model-${project.name || 'project'}.ifc`;
  a.click();
  URL.revokeObjectURL(url);
}
