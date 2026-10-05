/**
 * Kernel gap and warning paths → workflow step (ontwerp §3.4, `GAP_ROUTES`).
 *
 * The kernel reports where input is missing as a path, e.g.
 * `zones[0].surfaces[1].windows[0].gValue`, `ntaCalculation.dynamicWindows[0]`
 * or `/constructions/0/uValue`. The longest matching prefix decides the step
 * and sub page that "Ga naar" opens; paths without a match land on Controle,
 * which lists the raw code.
 */
import { parseKernelPath, type PathSegment } from './pathUtil';
import { normalizeRoute, type Route, type StepId } from '../navigation/routes';

/** `*` matches any array index; other segments match literally. */
type Pattern = string;

interface GapRoute {
  prefix: Pattern;
  step: StepId;
  sub?: string;
}

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
  { prefix: 'ntaCalculation.unheatedSpaces', step: 'building', sub: 'unheated' },
  { prefix: 'ntaCalculation.sunrooms', step: 'building', sub: 'unheated' },
  { prefix: 'ntaCalculation.calculationScope', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.usageFunction', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.setpoints', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.thermalMass', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.internalGains', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.surfaceTilts', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.windowSolar', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.dynamicWindows', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.groundFloors', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.zoneData', step: 'check', sub: 'input' },
  // Installations (simplified model and NTA input).
  { prefix: 'heatingSystems', step: 'installations', sub: 'heating' },
  { prefix: 'ventilationSystems', step: 'installations', sub: 'ventilation' },
  { prefix: 'coolingSystems', step: 'installations', sub: 'cooling' },
  { prefix: 'hotWaterSystems', step: 'installations', sub: 'hotWater' },
  { prefix: 'solarPV', step: 'installations', sub: 'generation' },
  { prefix: 'solarThermal', step: 'installations', sub: 'generation' },
  { prefix: 'ntaHeatPumps', step: 'installations', sub: 'heatPumps' },
  { prefix: 'ntaCalculation.emission', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.distribution', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.distributionSystem', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.generator', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.additionalHeatingSystems', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.verticalPipes', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.ventilation', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.ventilationFlows', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.hotWater', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.additionalHotWaterSystems', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.cooling', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.coolingSystems', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.humidifiers', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.lighting', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.pvSystems', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.onSiteProduction', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.externalSupply', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.declaredUses', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.spaceHeatingSolar', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation.bacs', step: 'check', sub: 'input' },
  { prefix: 'ntaCalculation', step: 'check', sub: 'input' },
  // Existing buildings and delivery.
  { prefix: 'basisopname', step: 'survey' },
  { prefix: 'maatwerkadvies', step: 'advice' },
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
  const segments = parseKernelPath(path)
    // Kernel paths below the derived input use the same names as the project model.
    .filter((segment, index) => !(index === 0 && segment === 'derivedInput'));
  let best: (typeof compiled)[number] | null = null;
  for (const route of compiled) {
    if (matches(segments, route.segments) && (best == null || route.segments.length > best.segments.length)) best = route;
  }
  const target: Route = best ? { step: best.step, ...(best.sub ? { sub: best.sub } : {}) } : FALLBACK_ROUTE;
  return normalizeRoute({ ...target, ...(path ? { focusPath: path } : {}) });
}

/** True when the path has an explicit route (not the check fallback). */
export function hasExplicitRoute(path: string | null | undefined): boolean {
  const segments = parseKernelPath(path);
  return compiled.some((route) => matches(segments, route.segments));
}
