/**
 * IFC Energy Exporter - Generates IFC 4x3 STEP files with BENG results
 * Exports building energy performance data as IfcPropertySet on IfcBuilding
 */

import type { IProject, IBENGResult } from '../energy/types';

// ============================================================
// IFC Model types (same pattern as Open-FEM2D-Studio)
// ============================================================

type IFCAttrValue = string | number | boolean | null | IFCEntity | IFCEntity[] | number[];

interface IFCEntity {
  id: number;
  type: string;
  attributes: IFCAttrValue[];
  label?: string;
}

interface IFCModel {
  entities: Map<number, IFCEntity>;
  nextId: number;
  header: {
    description: string;
    implementationLevel: string;
    fileName: string;
    timeStamp: string;
    author: string;
    organization: string;
    application: string;
    schema: string;
  };
}

// ============================================================
// Core helpers
// ============================================================

function createIFCModel(projectName: string): IFCModel {
  const now = new Date();
  const timestamp = now.toISOString().replace(/[-:]/g, '').split('.')[0];

  return {
    entities: new Map(),
    nextId: 1,
    header: {
      description: 'ViewDefinition [EnergyAnalysisView]',
      implementationLevel: '2;1',
      fileName: `${projectName}.ifc`,
      timeStamp: timestamp,
      author: 'Open-Energy-Studio User',
      organization: 'Open-Energy-Studio',
      application: 'Open-Energy-Studio',
      schema: 'IFC4X3_ADD2',
    },
  };
}

function addEntity(model: IFCModel, type: string, attributes: IFCAttrValue[], label?: string): IFCEntity {
  const entity: IFCEntity = {
    id: model.nextId++,
    type,
    attributes,
    label,
  };
  model.entities.set(entity.id, entity);
  return entity;
}

function formatValue(value: IFCAttrValue | undefined): string {
  if (value === null || value === undefined) return '$';
  if (typeof value === 'boolean') return value ? '.T.' : '.F.';
  if (typeof value === 'string') {
    if (value.startsWith('.') && value.endsWith('.')) return value;
    if (value.startsWith('#')) return value;
    if (value === '*') return '*';
    return `'${value.replace(/'/g, "''")}'`;
  }
  if (typeof value === 'number') {
    if (Number.isInteger(value)) return value.toString();
    return value.toExponential(6).toUpperCase().replace('E+', 'E');
  }
  if (Array.isArray(value)) {
    if (value.length === 0) return '()';
    if (typeof value[0] === 'number') {
      return `(${(value as number[]).map(v => formatValue(v)).join(',')})`;
    }
    return `(${(value as IFCEntity[]).map(v => formatValue(v)).join(',')})`;
  }
  if (typeof value === 'object' && 'id' in value) {
    return `#${value.id}`;
  }
  return '$';
}

function formatEntity(entity: IFCEntity): string {
  const attrs = entity.attributes.map(a => formatValue(a)).join(',');
  return `#${entity.id}=${entity.type}(${attrs});`;
}

function generateGUID(): string {
  const chars = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_$';
  let guid = '';
  for (let i = 0; i < 22; i++) {
    guid += chars[Math.floor(Math.random() * 64)];
  }
  return guid;
}

function generateIFCString(model: IFCModel): string {
  const lines: string[] = [];

  lines.push('ISO-10303-21;');
  lines.push('HEADER;');
  lines.push(`FILE_DESCRIPTION(('${model.header.description}'),'${model.header.implementationLevel}');`);
  lines.push(`FILE_NAME('${model.header.fileName}','${model.header.timeStamp}',('${model.header.author}'),('${model.header.organization}'),'','${model.header.application}','');`);
  lines.push(`FILE_SCHEMA(('${model.header.schema}'));`);
  lines.push('ENDSEC;');
  lines.push('');
  lines.push('DATA;');

  const sortedEntities = Array.from(model.entities.values()).sort((a, b) => a.id - b.id);

  for (const entity of sortedEntities) {
    const line = formatEntity(entity);
    if (entity.label) {
      lines.push(`${line} /* ${entity.label} */`);
    } else {
      lines.push(line);
    }
  }

  lines.push('ENDSEC;');
  lines.push('END-ISO-10303-21;');

  return lines.join('\n');
}

// ============================================================
// Helper: create IfcPropertySingleValue with IfcReal wrapper
// ============================================================

function addRealProperty(model: IFCModel, name: string, value: number, unit: string | null): IFCEntity {
  // IfcPropertySingleValue(Name, Description, NominalValue, Unit)
  // NominalValue = IfcReal wrapped as IFCREAL(value)
  const realValue = addEntity(model, 'IFCREAL', [value]);
  return addEntity(model, 'IFCPROPERTYSINGLEVALUE', [
    name, unit, realValue, null,
  ]);
}

function addTextProperty(model: IFCModel, name: string, value: string): IFCEntity {
  const textValue = addEntity(model, 'IFCTEXT', [value]);
  return addEntity(model, 'IFCPROPERTYSINGLEVALUE', [
    name, null, textValue, null,
  ]);
}

// ============================================================
// Main export function
// ============================================================

export function exportBENGToIFC(project: IProject, result: IBENGResult): IFCModel {
  const model = createIFCModel(project.name || 'BENG-Project');

  // === Base IFC infrastructure ===

  const person = addEntity(model, 'IFCPERSON', [null, null, null, null, null, null, null, null]);
  const org = addEntity(model, 'IFCORGANIZATION', [null, 'Open-Energy-Studio', null, null, null]);
  const personOrg = addEntity(model, 'IFCPERSONANDORGANIZATION', [person, org, null]);
  const app = addEntity(model, 'IFCAPPLICATION', [org, '1.0', 'Open-Energy-Studio', 'OES']);
  const ownerHistory = addEntity(model, 'IFCOWNERHISTORY', [
    personOrg, app, null, '.READWRITE.', null, null, null, Math.floor(Date.now() / 1000),
  ]);

  // Units
  const unitLength = addEntity(model, 'IFCSIUNIT', ['*', '.LENGTHUNIT.', null, '.METRE.']);
  const unitArea = addEntity(model, 'IFCSIUNIT', ['*', '.AREAUNIT.', null, '.SQUARE_METRE.']);
  const unitEnergy = addEntity(model, 'IFCSIUNIT', ['*', '.ENERGYUNIT.', '.KILO.', '.WATT.']);  // kWh approximation
  const units = addEntity(model, 'IFCUNITASSIGNMENT', [[unitLength, unitArea, unitEnergy]]);

  // Geometric context
  const origin = addEntity(model, 'IFCCARTESIANPOINT', [[0.0, 0.0, 0.0]]);
  const axis = addEntity(model, 'IFCDIRECTION', [[0.0, 0.0, 1.0]]);
  const refDir = addEntity(model, 'IFCDIRECTION', [[1.0, 0.0, 0.0]]);
  const worldCS = addEntity(model, 'IFCAXIS2PLACEMENT3D', [origin, axis, refDir]);
  const geoContext = addEntity(model, 'IFCGEOMETRICREPRESENTATIONCONTEXT', [
    'Model', 'Model', 3, 1.0E-5, worldCS, null,
  ]);

  // IfcProject
  const ifcProject = addEntity(model, 'IFCPROJECT', [
    generateGUID(), ownerHistory, project.name || 'BENG Project',
    project.description || null, null, null, null, [geoContext], units,
  ], 'Project');

  // IfcSite
  const site = addEntity(model, 'IFCSITE', [
    generateGUID(), ownerHistory, 'Site', project.address || null,
    null, null, null, null, '.ELEMENT.', null, null, null, null, null,
  ], 'Site');

  // IfcBuilding
  const building = addEntity(model, 'IFCBUILDING', [
    generateGUID(), ownerHistory, project.name || 'Building',
    `${project.city || ''} - ${project.address || ''}`,
    null, null, null, null, '.ELEMENT.', null, null, null,
  ], 'Building');

  // Spatial hierarchy: Project → Site → Building
  addEntity(model, 'IFCRELAGGREGATES', [generateGUID(), ownerHistory, null, null, ifcProject, [site]]);
  addEntity(model, 'IFCRELAGGREGATES', [generateGUID(), ownerHistory, null, null, site, [building]]);

  // ==========================================================
  // Pset_BuildingCommon — standard IFC property set
  // ==========================================================

  const totalFloorArea = result.totalFloorArea;
  const totalVolume = project.zones.reduce((sum, z) => sum + z.volume, 0);

  const buildingCommonProps = [
    addRealProperty(model, 'GrossPlannedArea', totalFloorArea, 'm2'),
    addRealProperty(model, 'NetPlannedArea', totalFloorArea, 'm2'),
    addTextProperty(model, 'BuildingFunction', project.buildingFunction),
    addRealProperty(model, 'NumberOfStoreys', project.zones.length, null),
    addRealProperty(model, 'TotalVolume', totalVolume, 'm3'),
  ];

  const psetBuildingCommon = addEntity(model, 'IFCPROPERTYSET', [
    generateGUID(), ownerHistory, 'Pset_BuildingCommon',
    'Standard building properties', buildingCommonProps,
  ], 'Pset_BuildingCommon');

  addEntity(model, 'IFCRELDEFINESBYPROPERTIES', [
    generateGUID(), ownerHistory, null, null, [building], psetBuildingCommon,
  ]);

  // ==========================================================
  // Pset_BuildingEnergyPerformance — BENG results
  // ==========================================================

  const bengProps = [
    addRealProperty(model, 'BENG1_EnergyDemand', result.beng1, 'kWh/m2.year'),
    addRealProperty(model, 'BENG1_Limit', result.beng1Limit, 'kWh/m2.year'),
    addTextProperty(model, 'BENG1_Status', 'INDICATIVE'),
    addRealProperty(model, 'BENG2_PrimaryFossilEnergy', result.beng2, 'kWh/m2.year'),
    addRealProperty(model, 'BENG2_Limit', result.beng2Limit, 'kWh/m2.year'),
    addTextProperty(model, 'BENG2_Status', 'INDICATIVE'),
    addRealProperty(model, 'BENG3_RenewableShare', result.beng3, '%'),
    addRealProperty(model, 'BENG3_Limit', result.beng3Limit, '%'),
    addTextProperty(model, 'BENG3_Status', 'INDICATIVE'),
    addTextProperty(model, 'VerificationStatus', 'UNVERIFIED'),
    addTextProperty(model, 'CalculationMethod', 'Legacy simplified monthly model; not attested NTA 8800'),
    addTextProperty(model, 'CalculationTool', 'Open-Energy-Studio'),
    addTextProperty(model, 'CalculationDate', new Date().toISOString().split('T')[0]),
  ];

  const psetBENG = addEntity(model, 'IFCPROPERTYSET', [
    generateGUID(), ownerHistory, 'Pset_BuildingEnergyPerformance',
    'Indicative energy estimates; not a verified NTA 8800 calculation', bengProps,
  ], 'Pset_BuildingEnergyPerformance');

  addEntity(model, 'IFCRELDEFINESBYPROPERTIES', [
    generateGUID(), ownerHistory, null, null, [building], psetBENG,
  ]);

  // ==========================================================
  // Pset_EnergyBreakdown — detailed energy balance
  // ==========================================================

  const bd = result.breakdown;
  const breakdownProps = [
    addRealProperty(model, 'TransmissionLoss', bd.transmissionLoss, 'kWh/year'),
    addRealProperty(model, 'VentilationLoss', bd.ventilationLoss, 'kWh/year'),
    addRealProperty(model, 'InfiltrationLoss', bd.infiltrationLoss, 'kWh/year'),
    addRealProperty(model, 'SolarGain', bd.solarGain, 'kWh/year'),
    addRealProperty(model, 'InternalGain', bd.internalGain, 'kWh/year'),
    addRealProperty(model, 'HeatingDemand', bd.heatingDemand, 'kWh/year'),
    addRealProperty(model, 'CoolingDemand', bd.coolingDemand, 'kWh/year'),
    addRealProperty(model, 'HeatingEnergy_Delivered', bd.heatingEnergy, 'kWh/year'),
    addRealProperty(model, 'CoolingEnergy_Delivered', bd.coolingEnergy, 'kWh/year'),
    addRealProperty(model, 'VentilationEnergy_Fans', bd.ventilationEnergy, 'kWh/year'),
    addRealProperty(model, 'HotWaterEnergy', bd.hotWaterEnergy, 'kWh/year'),
    addRealProperty(model, 'LightingEnergy', bd.lightingEnergy, 'kWh/year'),
    addRealProperty(model, 'TotalPrimaryEnergy', bd.totalPrimaryEnergy, 'kWh/year'),
    addRealProperty(model, 'RenewableEnergy', bd.renewableEnergy, 'kWh/year'),
    addRealProperty(model, 'PV_Production', bd.pvProduction, 'kWh/year'),
    addRealProperty(model, 'SolarThermal_Production', bd.solarThermalProduction, 'kWh/year'),
  ];

  const psetBreakdown = addEntity(model, 'IFCPROPERTYSET', [
    generateGUID(), ownerHistory, 'Pset_EnergyBreakdown',
    'Detailed energy balance breakdown', breakdownProps,
  ], 'Pset_EnergyBreakdown');

  addEntity(model, 'IFCRELDEFINESBYPROPERTIES', [
    generateGUID(), ownerHistory, null, null, [building], psetBreakdown,
  ]);

  // ==========================================================
  // Pset_BuildingEnvelope — per-zone thermal properties
  // ==========================================================

  for (const zone of project.zones) {
    const zoneProps = [
      addTextProperty(model, 'ZoneName', zone.name),
      addRealProperty(model, 'FloorArea', zone.floorArea, 'm2'),
      addRealProperty(model, 'Volume', zone.volume, 'm3'),
      addRealProperty(model, 'Height', zone.height, 'm'),
      addRealProperty(model, 'AirTightness_qv10', zone.airTightness.qv10, 'dm3/s.m2'),
      addRealProperty(model, 'SurfaceCount', zone.surfaces.length, null),
      addRealProperty(model, 'ThermalBridgeCount', zone.thermalBridges.length, null),
    ];

    // Add U-values per surface
    for (const surface of zone.surfaces) {
      const construction = project.constructions.find(c => c.id === surface.constructionId);
      if (construction) {
        zoneProps.push(addRealProperty(model,
          `Surface_${surface.name}_U`,
          construction.uValue,
          'W/m2K',
        ));
      }
    }

    const psetZone = addEntity(model, 'IFCPROPERTYSET', [
      generateGUID(), ownerHistory, `Pset_ThermalZone_${zone.name}`,
      `Thermal zone properties for ${zone.name}`, zoneProps,
    ], `Pset_ThermalZone_${zone.name}`);

    addEntity(model, 'IFCRELDEFINESBYPROPERTIES', [
      generateGUID(), ownerHistory, null, null, [building], psetZone,
    ]);
  }

  // ==========================================================
  // Pset_Installations — heating, ventilation, etc.
  // ==========================================================

  const installProps: IFCEntity[] = [];

  for (const hs of project.heatingSystems) {
    installProps.push(addTextProperty(model, `Heating_${hs.name}_Type`, hs.type));
    installProps.push(addRealProperty(model, `Heating_${hs.name}_COP`, hs.cop, null));
    installProps.push(addRealProperty(model, `Heating_${hs.name}_Coverage`, hs.coverageFraction * 100, '%'));
  }

  for (const vs of project.ventilationSystems) {
    installProps.push(addTextProperty(model, `Ventilation_${vs.name}_Type`, vs.type));
    installProps.push(addRealProperty(model, `Ventilation_${vs.name}_HeatRecovery`, vs.heatRecoveryEfficiency * 100, '%'));
    installProps.push(addRealProperty(model, `Ventilation_${vs.name}_SFP`, vs.sfp, 'W/(dm3/s)'));
  }

  for (const cs of project.coolingSystems) {
    installProps.push(addTextProperty(model, `Cooling_${cs.name}_Type`, cs.type));
    installProps.push(addRealProperty(model, `Cooling_${cs.name}_EER`, cs.eer, null));
  }

  for (const hw of project.hotWaterSystems) {
    installProps.push(addTextProperty(model, `HotWater_${hw.name}_Type`, hw.type));
    installProps.push(addRealProperty(model, `HotWater_${hw.name}_Efficiency`, hw.efficiency, null));
  }

  if (installProps.length > 0) {
    const psetInstall = addEntity(model, 'IFCPROPERTYSET', [
      generateGUID(), ownerHistory, 'Pset_Installations',
      'Building installation systems', installProps,
    ], 'Pset_Installations');

    addEntity(model, 'IFCRELDEFINESBYPROPERTIES', [
      generateGUID(), ownerHistory, null, null, [building], psetInstall,
    ]);
  }

  // ==========================================================
  // Pset_RenewableEnergy — PV and solar thermal
  // ==========================================================

  const renewProps: IFCEntity[] = [];

  for (const pv of project.solarPV) {
    renewProps.push(addTextProperty(model, `PV_${pv.name}_Orientation`, pv.orientation));
    renewProps.push(addRealProperty(model, `PV_${pv.name}_PeakPower`, pv.peakPower, 'kWp'));
    renewProps.push(addRealProperty(model, `PV_${pv.name}_Tilt`, pv.tilt, 'deg'));
    renewProps.push(addRealProperty(model, `PV_${pv.name}_Area`, pv.area, 'm2'));
  }

  for (const st of project.solarThermal) {
    renewProps.push(addTextProperty(model, `SolarThermal_${st.name}_Type`, st.type));
    renewProps.push(addRealProperty(model, `SolarThermal_${st.name}_Area`, st.collectorArea, 'm2'));
    renewProps.push(addRealProperty(model, `SolarThermal_${st.name}_Tilt`, st.tilt, 'deg'));
  }

  if (renewProps.length > 0) {
    const psetRenew = addEntity(model, 'IFCPROPERTYSET', [
      generateGUID(), ownerHistory, 'Pset_RenewableEnergy',
      'Renewable energy systems', renewProps,
    ], 'Pset_RenewableEnergy');

    addEntity(model, 'IFCRELDEFINESBYPROPERTIES', [
      generateGUID(), ownerHistory, null, null, [building], psetRenew,
    ]);
  }

  return model;
}

// ============================================================
// Public API: download IFC file
// ============================================================

export function downloadBENGIFC(project: IProject, result: IBENGResult): void {
  const model = exportBENGToIFC(project, result);
  const stepString = generateIFCString(model);
  const blob = new Blob([stepString], { type: 'application/x-step' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `BENG-${project.name || 'project'}.ifc`;
  a.click();
  URL.revokeObjectURL(url);
}
