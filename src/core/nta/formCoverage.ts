// Form coverage of the kernel input (docs/ui-redesign/formulierdekking.md).
//
// training-data/nta8800-input-options.json lists every enum variant and
// optional field of the project input and of both surveys, as serde lists
// them (crates/nta8800-core/tests/option_coverage.rs keeps it current). The
// check in src/__tests__/nta-form-coverage.test.ts counts an option of a
// route as covered only when the form sources of that route write it: its
// name at a write site (an option value, a path or a template key, not a
// read, a comparison or a comment) in a file that also names its parent
// key. Options a form sets in a way that check cannot see, options that stay
// JSON-only on purpose, and options that still lack a form are listed here
// with the reason.

/** Directories and files whose code makes up the input forms, per route. */
export const FORM_SOURCES = {
  project: [
    'src/components/NtaPerformancePanel',
    'src/components/dialogs/ProjectInfoDialog',
    // The forfait heat pump of the project is entered here and copied into
    // ntaCalculation.generator.forfait (NtaSystemTemplates).
    'src/components/HeatPumpInventoryPanel',
    'src/components/HeatPumpForfaitDiagnosticPanel',
    'src/components/shell/pages/BuildingPages.tsx',
    'src/core/nta/NtaSystemTemplates.ts',
    'src/core/nta/NtaFormModels.ts',
    'src/core/nta/NtaCalculationTemplate.ts',
    'src/core/nta/annexAaForm.ts',
  ],
  residentialSurvey: [
    'src/components/BasisopnamePanel',
    'src/core/nta/SurveyTemplates.ts',
  ],
  utilitySurvey: [
    'src/components/BasisopnamePanel',
    'src/core/nta/SurveyTemplates.ts',
  ],
} as const;

export type CoverageRoute = 'project' | 'residentialSurvey' | 'utilitySurvey';

/**
 * Form files that build a whole kernel object in one literal (`return {
 * generatorId, scope, ... }`) rather than field by field; the check accepts
 * their keys for the objects named here.
 */
export const FORM_BUILDERS: Array<{ route: CoverageRoute; file: string; parents: string[] }> = [
  {
    route: 'project', file: 'HeatPumpForfaitDiagnosticPanel/HeatPumpForfaitDiagnosticPanel.tsx',
    parents: ['forfait', 'highEfficiencyEvidence', 'qualityDeclaration'],
  },
];

export interface CoverageExemption {
  route: CoverageRoute;
  /** Schema path as in the catalogue (array indices as `[]`). */
  path: string;
  options: string[];
  /**
   * `form`: a form sets it, but not by name; `json`: JSON-only on purpose;
   * `missing`: no form yet (to be added); `parallel`: a form is being added
   * in a parallel change and this entry goes when that lands.
   */
  kind: 'form' | 'json' | 'missing' | 'parallel';
  reason: string;
}

export const FORM_COVERAGE_EXEMPTIONS: CoverageExemption[] = [
  {
    route: 'project', path: '/ntaCalculation/windowSolar/movableShading/device/colour', options: ['dark', 'white', 'other', 'unknown'], kind: 'form',
    reason: 'Zonwering per raam en de projectbrede zonwering kiezen het toestel als samengestelde waarde "soort:kleur" (NtaWindowShadings).',
  },
  {
    route: 'project', path: '/ntaCalculation/windowSolar/movableShading/device/kind',
    options: ['external_screen', 'external_venetian_blind', 'external_roller_shutter'], kind: 'form',
    reason: 'Zonwering per raam en de projectbrede zonwering kiezen het toestel als samengestelde waarde "soort:kleur" (NtaWindowShadings).',
  },
  {
    route: 'project', path: '/ntaCalculation/windowShadings/[]/movableShading/device/colour', options: ['dark', 'white', 'other', 'unknown'], kind: 'form',
    reason: 'Zonwering per raam kiest het toestel als samengestelde waarde "soort:kleur" (NtaWindowShadings).',
  },
  {
    route: 'project', path: '/ntaCalculation/windowShadings/[]/movableShading/device/kind',
    options: ['external_screen', 'external_venetian_blind', 'external_roller_shutter'], kind: 'form',
    reason: 'Zonwering per raam kiest het toestel als samengestelde waarde "soort:kleur" (NtaWindowShadings).',
  },
  {
    route: 'project', path: '/ntaCalculation/groundFloors/[]/heatedBasement', options: ['forfaitDeltaUWPerM2k'], kind: 'json',
    reason: 'De kern leidt ΔU_for van de verwarmde kelder af uit het projectmodel (8.38); alleen opgeven om daarvan af te wijken (expert). De overname uit de opname laat een opgegeven waarde staan.',
  },
  {
    route: 'project', path: '/ntaCalculation/distributionSystem', options: ['usageFunction'], kind: 'form',
    reason: 'Het sjabloon van het distributiesysteem neemt de gebruiksfunctie van het blok over (distributionSystemTemplate).',
  },
  {
    route: 'project', path: '/ntaCalculation/internalGains/lighting/method', options: ['resolved'], kind: 'json',
    reason: 'Interne waarde: de kern vult Φ_int;L zelf in uit hoofdstuk 14; geen invoerkeuze.',
  },
  {
    route: 'project', path: '/ntaCalculation/lighting/[]', options: ['burningHoursFactor'], kind: 'json',
    reason: 'Alleen voor maatwerkadvies (ISSO 75.2 tabel 2.7); het maatwerkadvies zet hem, nooit voor het energielabel.',
  },
  {
    route: 'project', path: '/ntaCalculation/ventilation', options: ['practice'], kind: 'json',
    reason: 'Alleen voor maatwerkadvies (ISSO 82.2/75.2 praktijkfactoren); het maatwerkadvies zet hem.',
  },
  {
    route: 'project', path: '/ntaCalculation/zoneData/[]/ventilation', options: ['practice'], kind: 'json',
    reason: 'Alleen voor maatwerkadvies (ISSO 82.2/75.2 praktijkfactoren); het maatwerkadvies zet hem.',
  },
  {
    route: 'project', path: '/ntaCalculation/ventilation', options: ['overventilation'], kind: 'json',
    reason: 'Maandwaarden die de kern uit de warmtepompketen afleidt; alleen opgeven bij afwijkende meetgegevens (expert).',
  },
  {
    route: 'project', path: '/ntaCalculation/zoneData/[]/ventilation', options: ['overventilation'], kind: 'json',
    reason: 'Maandwaarden die de kern uit de warmtepompketen afleidt; alleen opgeven bij afwijkende meetgegevens (expert).',
  },
  {
    route: 'project', path: '/ntaCalculation/ventilationFlows/[]/months/[]', options: ['coolingConductanceWPerK', 'coolingSupplyTemperatureC'], kind: 'json',
    reason: 'Afwijkende koelwaarden per maand van een opgegeven luchtstroom; leeg is de verwarmingswaarde (expert).',
  },
  {
    route: 'project', path: '/ntaCalculation/zoneData/[]/ventilationFlows/[]/months/[]', options: ['coolingConductanceWPerK', 'coolingSupplyTemperatureC'], kind: 'json',
    reason: 'Afwijkende koelwaarden per maand van een opgegeven luchtstroom van de zone; leeg is de verwarmingswaarde (expert).',
  },
  {
    route: 'project', path: '/ntaCalculation/zoneData/[]/sunrooms/[]', options: ['borderingZones'], kind: 'json',
    reason: 'De kern leidt de aangrenzende zones van een gedeelde serre af uit de zones met hetzelfde serre-id (7.38, review 54); een opgegeven lijst wordt vervangen.',
  },
  {
    route: 'project', path: '/ntaCalculation/thermalMass', options: ['annexBElements'], kind: 'json',
    reason: 'Bijlage B per bouwdeel vervangt tabel 7.10; vraagt een lagenopbouw per element (expert).',
  },
  {
    route: 'project', path: '/ntaCalculation/zoneData/[]/thermalMass', options: ['annexBElements'], kind: 'json',
    reason: 'Bijlage B per bouwdeel vervangt tabel 7.10; vraagt een lagenopbouw per element (expert).',
  },
  {
    route: 'project', path: '/ntaCalculation/generator', options: ['auxiliaryMeasurements'], kind: 'json',
    reason: 'Componentmetingen volgens bijlage O (9.86–9.90) uit een meetrapport; de opgegeven A/B/C uit een kwaliteitsverklaring hebben wel een formulier.',
  },
  {
    route: 'project', path: '/ntaCalculation/hotWater/generator', options: ['annexT', 'annexTConditions'], kind: 'json',
    reason: 'Beproevingsrapport volgens bijlage T (gaskeur-koppeling); zeldzaam, uit het rapport over te nemen.',
  },
  {
    route: 'project', path: '/ntaCalculation/distributionSystem', options: ['unheatedPipeTransmittance'], kind: 'json',
    reason: 'Afwijkende Ψ voor het deel in onverwarmde ruimten; leeg geldt de Ψ van het hele systeem.',
  },
  {
    route: 'project', path: '/ntaCalculation/cooling', options: ['boosterHeatPumpExtractionKwh'], kind: 'json',
    reason: 'Maandwaarden van de warmte die een boosterwarmtepomp aan de koeling onttrekt; uit een meetrapport (expert).',
  },
  {
    route: 'project', path: '/ntaCalculation/ventilation/fans/control', options: ['monthly'], kind: 'json',
    reason: 'Maandfracties van de ventilatorregeling uit een meetrapport (expert); de forfaitaire regeling heeft een formulier.',
  },
];
