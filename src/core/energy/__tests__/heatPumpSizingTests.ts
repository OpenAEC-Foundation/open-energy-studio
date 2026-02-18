// ============================================================
// Heat Pump Sizing – 20 Reference Calculations
// Sources: Techniek Nederland, Milieu Centraal, Daikin, Vaillant,
// NIBE, BouwTotaal, ISSO, installer knowledge bases
// ============================================================

import {
  calculateHeatPumpSizing,
  type HeatPumpSizingInput,
} from '../HeatPumpSizingCalc';
import type { IZone, IConstruction } from '../types';

interface ReferenceCase {
  id: number;
  name: string;
  source: string;
  expectedKW: number;          // expected design load in kW (without safety margin)
  tolerancePercent: number;    // allowed deviation (default 15%)
  input: HeatPumpSizingInput;
}

// Helper to create a simple zone with one surface type
function makeZone(
  floorArea: number,
  volume: number,
  wallArea: number,
  roofArea: number,
  floorAreaSurf: number,
  windowArea: number,
  windowUValue: number,
  qv10: number,
  tbLength: number,
  tbPsi: number,
): IZone {
  return {
    id: 'z1',
    name: 'Zone',
    floorArea,
    volume,
    height: volume / floorArea,
    surfaces: [
      {
        id: 'wall', name: 'Walls', type: 'wall', area: wallArea,
        orientation: 'N', constructionId: 'c-wall', zoneId: 'z1',
        windows: windowArea > 0 ? [{
          id: 'w1', name: 'Windows', area: windowArea, uValue: windowUValue,
          gValue: 0.4, orientation: 'S', surfaceId: 'wall',
        }] : [],
      },
      {
        id: 'roof', name: 'Roof', type: 'roof', area: roofArea,
        orientation: 'horizontal', constructionId: 'c-roof', zoneId: 'z1',
        windows: [],
      },
      {
        id: 'floor', name: 'Floor', type: 'floor', area: floorAreaSurf,
        orientation: 'horizontal', constructionId: 'c-floor', zoneId: 'z1',
        windows: [],
      },
    ],
    thermalBridges: tbLength > 0 ? [{
      id: 'tb1', name: 'Thermal bridges', psiValue: tbPsi, length: tbLength, zoneId: 'z1',
    }] : [],
    airTightness: { qv10 },
  };
}

function makeConstructions(wallU: number, roofU: number, floorU: number): IConstruction[] {
  return [
    { id: 'c-wall', name: 'Wall', layers: [], rcValue: 1 / wallU, uValue: wallU },
    { id: 'c-roof', name: 'Roof', layers: [], rcValue: 1 / roofU, uValue: roofU },
    { id: 'c-floor', name: 'Floor', layers: [], rcValue: 1 / floorU, uValue: floorU },
  ];
}

// ============================================================
// 20 Reference Cases
// ============================================================

export const referenceCases: ReferenceCase[] = [
  // ---- 1. Nieuwbouw tussenwoning (Techniek Nederland richtlijn) ----
  {
    id: 1,
    name: 'Nieuwbouw tussenwoning 110 m² – goed geïsoleerd',
    source: 'Techniek Nederland richtlijn warmtepompdimensionering',
    expectedKW: 3.1,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(110, 286, 55, 0, 55, 16, 1.1, 0.8, 40, 0.04)],
      constructions: makeConstructions(0.18, 0.18, 0.20),
      buildingFunction: 'residential',
      ventilationSystems: [{ type: 'type_d', heatRecoveryEfficiency: 0.85 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 2. Nieuwbouw vrijstaande woning (Milieu Centraal referentie) ----
  {
    id: 2,
    name: 'Nieuwbouw vrijstaande woning 150 m²',
    source: 'Milieu Centraal – warmtepomp kiezen',
    expectedKW: 4.9,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(150, 405, 120, 80, 75, 25, 1.1, 0.8, 70, 0.05)],
      constructions: makeConstructions(0.17, 0.16, 0.19),
      buildingFunction: 'residential',
      ventilationSystems: [{ type: 'type_d', heatRecoveryEfficiency: 0.85 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 3. Bestaande woning jaren '70 – matig geïsoleerd ----
  {
    id: 3,
    name: 'Bestaande tussenwoning 100 m² – jaren 70 (spouwmuur)',
    source: 'NIBE dimensioneringstabel',
    expectedKW: 7.5,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(100, 260, 55, 0, 50, 14, 1.6, 1.5, 35, 0.10)],
      constructions: makeConstructions(0.50, 0.40, 0.50),
      buildingFunction: 'residential',
      ventilationSystems: [{ type: 'natural', heatRecoveryEfficiency: 0 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 4. Bestaande vrijstaande woning – slecht geïsoleerd ----
  {
    id: 4,
    name: 'Vrijstaande woning 160 m² – slecht geïsoleerd (voor 1975)',
    source: 'Daikin dimensioneringsrichtlijn',
    expectedKW: 18.0,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(160, 432, 160, 80, 80, 24, 2.8, 2.0, 80, 0.15)],
      constructions: makeConstructions(1.10, 0.80, 0.70),
      buildingFunction: 'residential',
      ventilationSystems: [{ type: 'natural', heatRecoveryEfficiency: 0 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 5. Nieuwbouw hoekwoning compacte ----
  {
    id: 5,
    name: 'Nieuwbouw hoekwoning 95 m²',
    source: 'Vaillant dimensioneringsgids NL',
    expectedKW: 3.0,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(95, 247, 70, 48, 48, 14, 1.1, 0.8, 45, 0.05)],
      constructions: makeConstructions(0.17, 0.15, 0.19),
      buildingFunction: 'residential',
      ventilationSystems: [{ type: 'type_d', heatRecoveryEfficiency: 0.85 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 6. Klein kantoor nieuwbouw ----
  {
    id: 6,
    name: 'Klein kantoor 200 m² nieuwbouw',
    source: 'ISSO 51 – kantoorgebouwen',
    expectedKW: 9.0,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(200, 600, 100, 100, 100, 40, 1.1, 0.6, 60, 0.05)],
      constructions: makeConstructions(0.20, 0.18, 0.22),
      buildingFunction: 'office',
      ventilationSystems: [{ type: 'type_d', heatRecoveryEfficiency: 0.80 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 7. Basisschool nieuwbouw ----
  {
    id: 7,
    name: 'Basisschool 500 m² nieuwbouw',
    source: 'ISSO 89 – schoolgebouwen',
    expectedKW: 20.5,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(500, 1750, 200, 250, 250, 80, 1.1, 0.8, 120, 0.05)],
      constructions: makeConstructions(0.20, 0.18, 0.22),
      buildingFunction: 'education',
      ventilationSystems: [{ type: 'type_d', heatRecoveryEfficiency: 0.80 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 8. Appartement nieuwbouw ----
  {
    id: 8,
    name: 'Appartement 70 m² nieuwbouw (binnenwanden intern)',
    source: 'Techniek Nederland – appartementen',
    expectedKW: 1.7,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(70, 189, 20, 0, 0, 10, 1.1, 0.8, 20, 0.04)],
      constructions: makeConstructions(0.18, 0.16, 0.20),
      buildingFunction: 'residential',
      ventilationSystems: [{ type: 'type_d', heatRecoveryEfficiency: 0.85 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 9. Bestaande 2-onder-1-kap met isolatie ----
  {
    id: 9,
    name: '2-onder-1-kap 130 m² – na-geïsoleerd (Rc 2.5)',
    source: 'BouwTotaal – warmtepomp dimensionering',
    expectedKW: 9.0,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(130, 351, 70, 65, 65, 20, 1.6, 1.2, 55, 0.08)],
      constructions: makeConstructions(0.35, 0.30, 0.40),
      buildingFunction: 'residential',
      ventilationSystems: [{ type: 'type_c', heatRecoveryEfficiency: 0 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 10. Grote villa nieuwbouw ----
  {
    id: 10,
    name: 'Villa 250 m² nieuwbouw – zeer goed geïsoleerd',
    source: 'NIBE systeemkeuze villa',
    expectedKW: 7.0,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(250, 700, 180, 125, 125, 40, 1.0, 0.6, 100, 0.04)],
      constructions: makeConstructions(0.15, 0.13, 0.17),
      buildingFunction: 'residential',
      ventilationSystems: [{ type: 'type_d', heatRecoveryEfficiency: 0.90 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 11. Winkelpand nieuwbouw ----
  {
    id: 11,
    name: 'Winkelunit 300 m² nieuwbouw',
    source: 'ISSO 74 – winkelgebouwen',
    expectedKW: 12.5,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(300, 1200, 120, 150, 150, 60, 1.1, 0.8, 80, 0.05)],
      constructions: makeConstructions(0.22, 0.20, 0.25),
      buildingFunction: 'retail',
      ventilationSystems: [{ type: 'type_d', heatRecoveryEfficiency: 0.75 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 12. Industriehal klein ----
  {
    id: 12,
    name: 'Kleine bedrijfshal 400 m² (licht geïsoleerd)',
    source: 'Daikin industrie-advies',
    expectedKW: 35.0,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(400, 2400, 200, 400, 0, 20, 2.0, 2.0, 100, 0.10)],
      constructions: makeConstructions(0.40, 0.35, 0.50),
      buildingFunction: 'industrial',
      ventilationSystems: [{ type: 'natural', heatRecoveryEfficiency: 0 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 13. Zorginstelling nieuwbouw ----
  {
    id: 13,
    name: 'Zorgcentrum 600 m² nieuwbouw (22°C binnentemperatuur)',
    source: 'ISSO 75 – zorginstellingen',
    expectedKW: 22.0,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(600, 1920, 250, 300, 300, 80, 1.1, 0.6, 150, 0.05)],
      constructions: makeConstructions(0.18, 0.16, 0.20),
      buildingFunction: 'healthcare',
      ventilationSystems: [{ type: 'type_d', heatRecoveryEfficiency: 0.80 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 14. Rijtjeshuis bouwjaar 1990 ----
  {
    id: 14,
    name: 'Rijtjeshuis 105 m² – bouwjaar 1990 (Rc ~2.5)',
    source: 'Milieu Centraal – energielabel B',
    expectedKW: 6.0,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(105, 273, 50, 0, 52, 15, 1.6, 1.2, 35, 0.08)],
      constructions: makeConstructions(0.40, 0.35, 0.40),
      buildingFunction: 'residential',
      ventilationSystems: [{ type: 'type_c', heatRecoveryEfficiency: 0 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 15. Passiefhuis ----
  {
    id: 15,
    name: 'Passiefhuis 140 m² – extreem goed geïsoleerd',
    source: 'Passivhaus Institut – dimensionering',
    expectedKW: 1.7,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(140, 378, 100, 70, 70, 25, 0.8, 0.4, 60, 0.03)],
      constructions: makeConstructions(0.10, 0.09, 0.10),
      buildingFunction: 'residential',
      ventilationSystems: [{ type: 'type_d', heatRecoveryEfficiency: 0.93 }],
      safetyMargin: 0,
      includeReheat: false,
    },
  },
  // ---- 16. Twee-laags woning jaren '60 – ongeïsoleerd ----
  {
    id: 16,
    name: 'Woning jaren 60 – 120 m² ongeïsoleerd',
    source: 'Vaillant renovatie-advies',
    expectedKW: 12.0,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(120, 324, 100, 60, 60, 18, 2.8, 2.5, 60, 0.15)],
      constructions: makeConstructions(1.00, 0.80, 0.70),
      buildingFunction: 'residential',
      ventilationSystems: [{ type: 'natural', heatRecoveryEfficiency: 0 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 17. Groot kantoorgebouw ----
  {
    id: 17,
    name: 'Kantoor 1000 m² – nieuwbouw met veel glas',
    source: 'ISSO 51 – groot kantoor referentie',
    expectedKW: 36.0,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(1000, 3500, 400, 500, 500, 200, 1.0, 0.5, 250, 0.04)],
      constructions: makeConstructions(0.18, 0.16, 0.20),
      buildingFunction: 'office',
      ventilationSystems: [{ type: 'type_d', heatRecoveryEfficiency: 0.85 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 18. Nieuwbouw levensloopbestendige woning ----
  {
    id: 18,
    name: 'Gelijkvloerse woning 85 m² – NOM (bijna energieneutraal)',
    source: 'Techniek Nederland – NOM woning',
    expectedKW: 2.3,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(85, 230, 60, 42, 42, 12, 1.0, 0.6, 35, 0.04)],
      constructions: makeConstructions(0.15, 0.13, 0.17),
      buildingFunction: 'residential',
      ventilationSystems: [{ type: 'type_d', heatRecoveryEfficiency: 0.90 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 19. Bestaande bovenwoning ----
  {
    id: 19,
    name: 'Bovenwoning 65 m² – matig geïsoleerd',
    source: 'Milieu Centraal – warmtepomp huurwoning',
    expectedKW: 4.0,
    tolerancePercent: 15,
    input: {
      zones: [makeZone(65, 176, 30, 32, 0, 10, 1.6, 1.5, 25, 0.08)],
      constructions: makeConstructions(0.45, 0.35, 0.40),
      buildingFunction: 'residential',
      ventilationSystems: [{ type: 'type_c', heatRecoveryEfficiency: 0 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
  // ---- 20. Nieuwbouw woning Goejanverwelledijk (project default) ----
  {
    id: 20,
    name: 'Goejanverwelledijk 85 – vrijstaande nieuwbouwwoning 133 m²',
    source: 'Open Energy Studio voorbeeldproject',
    expectedKW: 8.4,
    tolerancePercent: 15,
    input: {
      zones: [{
        id: 'zone-main',
        name: 'Woonfunctie',
        floorArea: 133.06,
        volume: 345.96,
        height: 2.6,
        surfaces: [
          {
            id: 'surf-wall-n', name: 'Gevel Noord', type: 'wall', area: 28.6,
            orientation: 'N', constructionId: 'con-wall', zoneId: 'zone-main',
            windows: [
              { id: 'win-n1', name: 'Raam Noord 1', area: 1.8, uValue: 1.1, gValue: 0.40, orientation: 'N', surfaceId: 'surf-wall-n' },
              { id: 'win-n2', name: 'Raam Noord 2', area: 1.2, uValue: 1.2, gValue: 0.40, orientation: 'N', surfaceId: 'surf-wall-n' },
            ],
          },
          {
            id: 'surf-wall-e', name: 'Gevel Oost', type: 'wall', area: 36.4,
            orientation: 'E', constructionId: 'con-wall', zoneId: 'zone-main',
            windows: [
              { id: 'win-e1', name: 'Raam Oost 1', area: 2.4, uValue: 1.1, gValue: 0.40, orientation: 'E', surfaceId: 'surf-wall-e' },
              { id: 'win-e2', name: 'Raam Oost 2', area: 1.6, uValue: 1.2, gValue: 0.40, orientation: 'E', surfaceId: 'surf-wall-e' },
            ],
          },
          {
            id: 'surf-wall-s', name: 'Gevel Zuid', type: 'wall', area: 28.6,
            orientation: 'S', constructionId: 'con-wall', zoneId: 'zone-main',
            windows: [
              { id: 'win-s1', name: 'Raam Zuid groot', area: 4.8, uValue: 1.1, gValue: 0.40, orientation: 'S', surfaceId: 'surf-wall-s' },
              { id: 'win-s2', name: 'Raam Zuid 2', area: 2.4, uValue: 1.1, gValue: 0.40, orientation: 'S', surfaceId: 'surf-wall-s' },
              { id: 'win-s3', name: 'Deur Zuid', area: 2.1, uValue: 2.0, gValue: 0.00, orientation: 'S', surfaceId: 'surf-wall-s' },
            ],
          },
          {
            id: 'surf-wall-w', name: 'Gevel West', type: 'wall', area: 36.4,
            orientation: 'W', constructionId: 'con-wall', zoneId: 'zone-main',
            windows: [
              { id: 'win-w1', name: 'Raam West 1', area: 2.4, uValue: 1.1, gValue: 0.40, orientation: 'W', surfaceId: 'surf-wall-w' },
              { id: 'win-w2', name: 'Deur West', area: 2.1, uValue: 2.0, gValue: 0.00, orientation: 'W', surfaceId: 'surf-wall-w' },
            ],
          },
          {
            id: 'surf-roof-e', name: 'Dak Oost', type: 'roof', area: 42,
            orientation: 'E', constructionId: 'con-roof', zoneId: 'zone-main',
            windows: [
              { id: 'win-dakraam-e', name: 'Dakraam Oost', area: 1.2, uValue: 1.2, gValue: 0.40, orientation: 'E', surfaceId: 'surf-roof-e' },
            ],
          },
          {
            id: 'surf-roof-w', name: 'Dak West', type: 'roof', area: 42,
            orientation: 'W', constructionId: 'con-roof', zoneId: 'zone-main',
            windows: [
              { id: 'win-dakraam-w', name: 'Dakraam West', area: 1.2, uValue: 1.2, gValue: 0.40, orientation: 'W', surfaceId: 'surf-roof-w' },
            ],
          },
          {
            id: 'surf-floor', name: 'Vloer begane grond', type: 'floor', area: 72,
            orientation: 'horizontal', constructionId: 'con-floor', zoneId: 'zone-main',
            windows: [],
          },
        ],
        thermalBridges: [
          { id: 'tb-1', name: 'Gevel-vloer', psiValue: 0.05, length: 34, zoneId: 'zone-main' },
          { id: 'tb-2', name: 'Gevel-dak', psiValue: 0.05, length: 34, zoneId: 'zone-main' },
          { id: 'tb-3', name: 'Raamkozijnen', psiValue: 0.03, length: 65, zoneId: 'zone-main' },
        ],
        airTightness: { qv10: 0.98 },
      }],
      constructions: [
        { id: 'con-wall', name: 'Gevel Rc=6.76', layers: [], rcValue: 6.76, uValue: 0.145 },
        { id: 'con-roof', name: 'Dak Rc=6.30', layers: [], rcValue: 6.30, uValue: 0.155 },
        { id: 'con-floor', name: 'Vloer Rc=5.09', layers: [], rcValue: 5.09, uValue: 0.190 },
      ],
      buildingFunction: 'residential',
      ventilationSystems: [{ type: 'type_d', heatRecoveryEfficiency: 0 }],
      safetyMargin: 0,
      includeReheat: true,
    },
  },
];

// ============================================================
// Test runner
// ============================================================

export interface TestResult {
  id: number;
  name: string;
  source: string;
  expectedKW: number;
  calculatedKW: number;
  deviationPercent: number;
  pass: boolean;
}

export function runHeatPumpSizingTests(): { results: TestResult[]; passCount: number; failCount: number } {
  const results: TestResult[] = [];
  let passCount = 0;
  let failCount = 0;

  for (const ref of referenceCases) {
    const calc = calculateHeatPumpSizing(ref.input);
    const calculatedKW = calc.totalDesignLoad / 1000;
    const deviation = ref.expectedKW > 0
      ? Math.abs(calculatedKW - ref.expectedKW) / ref.expectedKW * 100
      : 0;
    const pass = deviation <= ref.tolerancePercent;

    if (pass) passCount++;
    else failCount++;

    results.push({
      id: ref.id,
      name: ref.name,
      source: ref.source,
      expectedKW: ref.expectedKW,
      calculatedKW: Math.round(calculatedKW * 100) / 100,
      deviationPercent: Math.round(deviation * 10) / 10,
      pass,
    });
  }

  return { results, passCount, failCount };
}
