/**
 * Kernel gap and warning paths → workflow step (ontwerp §3.4, `GAP_ROUTES`).
 *
 * The kernel reports where input is missing as a path, e.g.
 * `zones[0].surfaces[1].windows[0].gValue`, `ntaCalculation.dynamicWindows[0]`
 * or `/constructions/0/uValue`. The longest matching prefix decides the step
 * and sub page that "Ga naar" opens; paths without a match land on Controle,
 * which lists the raw code. Paths in the NTA input go to the step page that
 * edits their section (F6).
 */
import { formatKernelPath, parseKernelPath, type PathSegment } from './pathUtil';
import { normalizeRoute, type Route, type StepId } from '../navigation/routes';

/** `*` matches any array index; other segments match literally. */
type Pattern = string;

interface GapRoute {
  prefix: Pattern;
  step: StepId;
  sub?: string;
}

type NtaTarget = [StepId, string?];

/**
 * Members of `ntaCalculation` → the step page with their section. The register
 * in `components/NtaPerformancePanel/NtaSections.tsx` uses the same targets
 * (checked by gap-routes.test.ts).
 */
export const NTA_INPUT_ROUTES: Record<string, NtaTarget> = {
  '': ['check', 'input'],
  normVersion: ['project'],
  calculationScope: ['project'],
  areaSourceReference: ['project'],
  usageFunction: ['project'],
  dwellingType: ['project'],
  bblFunction: ['project'],
  zebHeatDeliveryTemperature: ['project'],
  permitApplicationAfter20260529: ['project'],
  constructionYear: ['project'],
  fossilAppliancesOutsideCalculation: ['project'],
  labelFunctions: ['building', 'zones'],
  bblFunctions: ['building', 'zones'],
  functionAreas: ['building', 'zones'],
  setpoints: ['building', 'zones'],
  zoneData: ['building', 'zones'],
  'zoneData.*.verticalPipes': ['installations', 'heating'],
  'zoneData.*.ventilation': ['installations', 'ventilation'],
  'zoneData.*.ventilationFlows': ['installations', 'ventilation'],
  thermalMass: ['building', 'zones'],
  internalGains: ['building', 'zones'],
  windowSolar: ['building', 'envelope'],
  dynamicWindows: ['building', 'envelope'],
  windowObstructions: ['building', 'envelope'],
  surfaceTilts: ['building', 'envelope'],
  groundFloors: ['building', 'envelope'],
  unheatedSpaces: ['building', 'unheated'],
  sunrooms: ['building', 'unheated'],
  generator: ['installations', 'heating'],
  identicalSystems: ['installations', 'heating'],
  collectiveConnection: ['installations', 'heating'],
  heatPumpRenewable: ['installations', 'heating'],
  emission: ['installations', 'heating'],
  distribution: ['installations', 'heating'],
  distributionSystem: ['installations', 'heating'],
  verticalPipes: ['installations', 'heating'],
  additionalHeatingSystems: ['installations', 'heating'],
  heatingSystems: ['installations', 'heating'],
  spaceHeatingSolar: ['installations', 'heating'],
  ventilation: ['installations', 'ventilation'],
  ventilationFlows: ['installations', 'ventilation'],
  demandUsesFixedC1Ventilation: ['installations', 'ventilation'],
  hotWater: ['installations', 'hotWater'],
  additionalHotWaterSystems: ['installations', 'hotWater'],
  activeCooling: ['installations', 'cooling'],
  cooling: ['installations', 'cooling'],
  coolingSystems: ['installations', 'cooling'],
  humidifiers: ['installations', 'humidification'],
  lighting: ['installations', 'lighting'],
  pvSystems: ['installations', 'generation'],
  externalSupply: ['installations', 'generation'],
  declaredUses: ['installations', 'generation'],
  onSiteProduction: ['installations', 'generation'],
  declaredRenewableHeat: ['installations', 'generation'],
  batteryStoragePresent: ['installations', 'generation'],
  storage: ['installations', 'generation'],
  bacs: ['installations', 'bacs'],
  bacsFactor: ['installations', 'bacs'],
  bacsSourceReference: ['installations', 'bacs'],
  useInventoryComplete: ['check', 'input'],
  productionInventoryComplete: ['check', 'input'],
};

function ntaRoutes(): GapRoute[] {
  return Object.entries(NTA_INPUT_ROUTES).map(([member, [step, sub]]) => ({
    prefix: member ? `ntaCalculation.${member}` : 'ntaCalculation', step, ...(sub ? { sub } : {}),
  }));
}

/**
 * Members that only exist in the NTA input: a kernel path that starts with one
 * of them (a path of the building assessment, without `ntaCalculation.`) is
 * routed and focused as a path in the NTA input.
 */
const NTA_ONLY_MEMBERS = new Set(Object.keys(NTA_INPUT_ROUTES)
  .filter((member) => member !== '' && !member.includes('.'))
  .filter((member) => !['heatingSystems', 'coolingSystems', 'unheatedSpaces'].includes(member)));

/** Order does not matter: the longest matching prefix wins. */
export const GAP_ROUTES: GapRoute[] = [
  // Project data and registration.
  { prefix: 'name', step: 'project' },
  { prefix: 'buildingFunction', step: 'project' },
  { prefix: 'address', step: 'project' },
  { prefix: 'city', step: 'project' },
  { prefix: 'registration', step: 'registration' },
  { prefix: 'evidence', step: 'registration' },
  { prefix: 'registration.relabelComparison', step: 'relabel' },
  { prefix: 'relabelComparison', step: 'relabel' },
  // Building (geometry, envelope, zone use).
  { prefix: 'zones', step: 'building', sub: 'zones' },
  { prefix: 'zones.*.surfaces', step: 'building', sub: 'envelope' },
  { prefix: 'zones.*.thermalBridges', step: 'building', sub: 'thermalBridges' },
  { prefix: 'zones.*.pointThermalBridges', step: 'building', sub: 'thermalBridges' },
  { prefix: 'zones.*.airTightness', step: 'building', sub: 'airTightness' },
  { prefix: 'constructions', step: 'building', sub: 'constructions' },
  { prefix: 'unheatedSpaces', step: 'building', sub: 'unheated' },
  // Installations (simplified model).
  { prefix: 'heatingSystems', step: 'installations', sub: 'heating' },
  { prefix: 'ventilationSystems', step: 'installations', sub: 'ventilation' },
  { prefix: 'coolingSystems', step: 'installations', sub: 'cooling' },
  { prefix: 'hotWaterSystems', step: 'installations', sub: 'hotWater' },
  { prefix: 'solarPV', step: 'installations', sub: 'generation' },
  { prefix: 'solarThermal', step: 'installations', sub: 'generation' },
  { prefix: 'ntaHeatPumps', step: 'installations', sub: 'heatPumps' },
  // NTA input: the step page that edits the section (F6, see NtaSections).
  ...ntaRoutes(),
  // Existing buildings and delivery.
  // The survey and advice pages are split into sub pages (F8).
  { prefix: 'basisopname', step: 'survey', sub: 'general' },
  { prefix: 'basisopname.zones', step: 'survey', sub: 'zones' },
  { prefix: 'basisopname.lighting', step: 'survey', sub: 'zones' },
  { prefix: 'basisopname.envelope', step: 'survey', sub: 'envelope' },
  { prefix: 'basisopname.heating', step: 'survey', sub: 'heating' },
  { prefix: 'basisopname.hotWater', step: 'survey', sub: 'hotWater' },
  { prefix: 'basisopname.additionalHotWaterSystems', step: 'survey', sub: 'hotWater' },
  { prefix: 'basisopname.ventilation', step: 'survey', sub: 'ventilation' },
  { prefix: 'basisopname.cooling', step: 'survey', sub: 'cooling' },
  { prefix: 'basisopname.coolingPresent', step: 'survey', sub: 'cooling' },
  { prefix: 'basisopname.coolingCollective', step: 'survey', sub: 'cooling' },
  { prefix: 'basisopname.pv', step: 'survey', sub: 'pv' },
  { prefix: 'basisopname.derivedInput', step: 'survey', sub: 'result' },
  { prefix: 'basisopname.inklapRedenen', step: 'survey', sub: 'result' },
  { prefix: 'maatwerkadvies', step: 'advice', sub: 'measures' },
  { prefix: 'maatwerkadvies.currentUse', step: 'advice', sub: 'use' },
  { prefix: 'maatwerkadvies.measured', step: 'advice', sub: 'use' },
  { prefix: 'maatwerkadvies.tariffs', step: 'advice', sub: 'use' },
  { prefix: 'maatwerkadvies.economics', step: 'advice', sub: 'use' },
  { prefix: 'maatwerkadvies.renovationPassport', step: 'advice', sub: 'passport' },
  { prefix: 'maatwerkadvies.advisedPackageId', step: 'advice', sub: 'advice' },
];

const compiled = GAP_ROUTES.map((route) => ({ ...route, segments: parseKernelPath(route.prefix) }));

function matches(path: PathSegment[], prefix: PathSegment[]): boolean {
  if (prefix.length > path.length) return false;
  return prefix.every((segment, index) => segment === '*' ? typeof path[index] === 'number' : segment === path[index]);
}

/** Fallback for paths without a route: the check overview, which shows the raw code. */
export const FALLBACK_ROUTE: Route = { step: 'check', sub: 'overview' };

/** The step and sub page for a kernel path; `focusPath` carries the path for "Ga naar". */
export function routeForPath(path: string | null | undefined): Route {
  let segments = parseKernelPath(path)
    // Kernel paths below the derived input use the same names as the project model.
    .filter((segment, index) => !(index === 0 && segment === 'derivedInput'));
  let focusPath = path;
  if (typeof segments[0] === 'string' && NTA_ONLY_MEMBERS.has(segments[0])) {
    segments = ['ntaCalculation', ...segments];
    focusPath = formatKernelPath(segments);
  }
  let best: (typeof compiled)[number] | null = null;
  for (const route of compiled) {
    if (matches(segments, route.segments) && (best == null || route.segments.length > best.segments.length)) best = route;
  }
  const target: Route = best ? { step: best.step, ...(best.sub ? { sub: best.sub } : {}) } : FALLBACK_ROUTE;
  return normalizeRoute({ ...target, ...(focusPath ? { focusPath } : {}) });
}

/** True when the path has an explicit route (not the check fallback). */
export function hasExplicitRoute(path: string | null | undefined): boolean {
  const parsed = parseKernelPath(path);
  const segments = typeof parsed[0] === 'string' && NTA_ONLY_MEMBERS.has(parsed[0]) ? ['ntaCalculation', ...parsed] : parsed;
  return compiled.some((route) => matches(segments, route.segments));
}
