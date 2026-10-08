import type { IProject } from '../energy/types';
import type { ProjectPerformanceAssessment } from '../nta/KernelClient';

/**
 * EP-Online data overview for the project dossier.
 *
 * The field names follow the public export schema `EpbdExportTypesV4`
 * (Pandcertificaat, add group) from RVO's "Handleiding EP-online.nl —
 * opvragen van bestanden" (2025), bijlage 1 and 2. That schema describes
 * what EP-Online publishes about a registered label. The upload format that
 * attested software uses to register a label is not public; it is obtained
 * via RVO once the program has a BRL 9501 attest. This overview therefore
 * serves only to check the registered label in EP-Online against the
 * calculation afterwards. It is NOT a registration file.
 */
export const EP_ONLINE_OVERVIEW_NOTICE =
  'Overzicht ter controle van het geregistreerde label in EP-Online. Veldnamen volgen het openbare exportschema '
  + 'EpbdExportTypesV4 (Pandcertificaat). Dit is GEEN registratiebestand: het uploadformaat van EP-Online is niet '
  + 'openbaar en wordt na attestering via RVO verkregen.';

/** Element order and kind of the add group of `Pandcertificaat` in EpbdExportTypesV4. */
export const EP_ONLINE_FIELDS = [
  ['Registratiedatum', 'date'],
  ['Opnamedatum', 'date'],
  ['Geldig_tot', 'date'],
  ['Certificaathouder', 'string'],
  ['Soort_opname', 'opnametype'],
  ['Status', 'string'],
  ['Berekeningstype', 'string'],
  ['Op_basis_van_referentiegebouw', 'boolean'],
  ['Gebouwklasse', 'gebouwklasse'],
  ['Gebouwtype', 'string'],
  ['Gebouwsubtype', 'string'],
  ['SBIcode', 'string'],
  ['Postcode', 'zipcode'],
  ['Huisnummer', 'integer'],
  ['Huisletter', 'string'],
  ['Huisnummertoevoeging', 'string'],
  ['Detailaanduiding', 'string'],
  ['BAGVerblijfsobjectID', 'bagid'],
  ['BAGLigplaatsID', 'bagid'],
  ['BAGStandplaatsID', 'bagid'],
  ['BAGPandIDs', 'bagids'],
  ['Projectnaam', 'string'],
  ['Projectobject', 'string'],
  ['Bouwjaar', 'integer'],
  ['Gebruiksoppervlakte_thermische_zone', 'decimal'],
  ['Compactheid', 'decimal'],
  ['Energieklasse', 'class'],
  ['Energie_Index', 'decimal'],
  ['Energieindex_EMG_forfaitair', 'decimal'],
  ['Energiebehoefte', 'decimal'],
  ['Primaire_fossiele_energie', 'decimal'],
  ['Primaire_fossiele_energie_EMG_forfaitair', 'decimal'],
  ['Aandeel_hernieuwbare_energie', 'decimal'],
  ['Aandeel_hernieuwbare_energie_EMG_forfaitair', 'decimal'],
  ['Temperatuuroverschrijding', 'decimal'],
  ['Warmtebehoefte', 'decimal'],
  ['Eis_energiebehoefte', 'decimal'],
  ['Eis_primaire_fossiele_energie', 'decimal'],
  ['Eis_aandeel_hernieuwbare_energie', 'decimal'],
  ['Eis_temperatuuroverschrijding', 'decimal'],
  ['BerekendeCO2Emissie', 'decimal'],
  ['BerekendeEnergieverbruik', 'decimal'],
] as const;

export type EpOnlineField = typeof EP_ONLINE_FIELDS[number][0];
export type EpOnlineValue = string | number | boolean | string[] | null;

export interface EpOnlineOverview {
  notice: string;
  schema: 'EpbdExportTypesV4 (exportschema, Pandcertificaat)';
  /** Fields in schema order; fields the calculation cannot fill are left out. */
  pandcertificaat: Partial<Record<EpOnlineField, EpOnlineValue>>;
  /** Fields left out on purpose, with the reason. */
  omitted: Array<{ field: EpOnlineField; reason: string }>;
  /** Fields the schema requires that the project does not supply yet. */
  missingRequired: EpOnlineField[];
  /**
   * The adviser's statements for label elements k and l (Omgevingsregeling art. 5.13a lid 1).
   * The public export schema V4 has no fields for them, so they are listed apart; null is not
   * answered.
   */
  labelStatements: {
    note: string;
    k_reageertOpExterneSignalen: boolean | null;
    l_afgiftesysteemLageTemperatuur: boolean | null;
  };
}

export const LABEL_STATEMENTS_NOTE =
  'Verklaringen van de adviseur voor de labelelementen k en l (Omgevingsregeling art. 5.13a lid 1). '
  + 'Het openbare exportschema EpbdExportTypesV4 heeft hier geen velden voor; ze staan daarom apart.';

/** Fields with minOccurs 1 in the add group. */
export const EP_ONLINE_REQUIRED: EpOnlineField[] = [
  'Registratiedatum', 'Opnamedatum', 'Geldig_tot', 'Certificaathouder',
  'Op_basis_van_referentiegebouw', 'Gebouwklasse', 'Energieklasse',
];

const STATUS_BY_PURPOSE: Record<string, string> = {
  existing_building: 'Bestaand',
  delivery: 'Oplevering',
  bbl_check: 'Vergunningsaanvraag',
};

function epbdDate(value: string | null | undefined): string | null {
  if (!value || !/^\d{4}-\d{2}-\d{2}$/.test(value)) return null;
  return value.replace(/-/g, '');
}

function round(value: number | null | undefined, digits = 2): number | null {
  if (value == null || !Number.isFinite(value)) return null;
  const factor = 10 ** digits;
  return Math.round(value * factor) / factor;
}

/** Splits a Dutch house number like "12A" or "12-bis" into number, letter and addition. */
function houseNumberParts(raw: string | undefined): { number: number | null; letter: string | null; addition: string | null } {
  const match = /^\s*(\d+)\s*([A-Za-z])?(?:\s*[- ]?\s*(\S{1,4}))?\s*$/.exec(raw ?? '');
  if (!match) return { number: null, letter: null, addition: null };
  return { number: Number(match[1]), letter: match[2]?.toUpperCase() ?? null, addition: match[3] ?? null };
}

/** EP-Online data overview from the project and the kernel assessment. */
export function buildEpOnlineOverview(
  project: IProject,
  assessment: ProjectPerformanceAssessment | null | undefined,
): EpOnlineOverview {
  const registration = project.registration ?? {};
  const performance = assessment?.performance ?? null;
  const calculated = performance?.status === 'calculated_unverified';
  const indicators = assessment?.labelData?.indicators ?? null;
  const residential = project.buildingFunction === 'residential';
  const existing = (registration.purpose ?? 'existing_building') === 'existing_building';
  const values: Partial<Record<EpOnlineField, EpOnlineValue>> = {};
  const omitted: EpOnlineOverview['omitted'] = [];
  const set = (field: EpOnlineField, value: EpOnlineValue | undefined) => {
    if (value !== null && value !== undefined && value !== '') values[field] = value;
  };

  set('Registratiedatum', epbdDate(registration.registrationDate));
  set('Opnamedatum', epbdDate(registration.surveyDate));
  set('Geldig_tot', epbdDate(assessment?.registration?.validUntil));
  set('Certificaathouder', registration.certificateNumber?.trim() || null);
  set('Soort_opname', registration.surveyType === 'detailed' ? 'Detailopname' : registration.surveyType === 'basic' ? 'Basisopname' : null);
  set('Status', STATUS_BY_PURPOSE[registration.purpose ?? 'existing_building'] ?? null);
  set('Berekeningstype', 'NTA 8800');
  values.Op_basis_van_referentiegebouw = registration.representation === 'reference';
  values.Gebouwklasse = residential ? 'W' : 'U';
  set('Gebouwtype', registration.buildingType?.trim() || null);

  const postcode = registration.postcode?.replace(/\s+/g, '').toUpperCase();
  set('Postcode', postcode && /^\d{4}[A-Z]{2}$/.test(postcode) ? postcode : null);
  const house = houseNumberParts(registration.houseNumber);
  set('Huisnummer', house.number);
  set('Huisletter', house.letter);
  set('Huisnummertoevoeging', registration.houseNumberAddition?.trim() || house.addition);
  const bag = registration.bagObjectId?.trim();
  if (bag && /^\d{16}$/.test(bag)) {
    // Digits 5–6 give the BAG object type.
    const kind = bag.slice(4, 6);
    if (kind === '01') set('BAGVerblijfsobjectID', bag);
    else if (kind === '02') set('BAGLigplaatsID', bag);
    else if (kind === '03') set('BAGStandplaatsID', bag);
    else if (kind === '10') set('BAGPandIDs', [bag]);
  }

  set('Bouwjaar', assessment?.labelData?.general.constructionYear ?? registration.constructionYear ?? null);
  set('Gebruiksoppervlakte_thermische_zone', round(assessment?.geometry?.usableFloorAreaM2));
  set('Compactheid', round(assessment?.geometry?.lossAreaRatio));

  if (calculated && performance) {
    set('Energieklasse', performance.indicativeLabelClass);
    set('Energiebehoefte', round(performance.needIndicatorKwhPerM2Year));
    set('Primaire_fossiele_energie', round(performance.primaryFossilIndicatorKwhPerM2Year));
    set('Aandeel_hernieuwbare_energie', round(performance.renewableSharePercent, 1));
    // The forfait values only when they differ: a dwelling with area measures (EMG).
    const labelEp2 = performance.labelPrimaryFossilIndicatorKwhPerM2Year;
    if (residential && labelEp2 != null && labelEp2 !== performance.primaryFossilIndicatorKwhPerM2Year) {
      set('Primaire_fossiele_energie_EMG_forfaitair', round(labelEp2));
      set('Aandeel_hernieuwbare_energie_EMG_forfaitair', round(performance.labelRenewableSharePercent, 1));
    }
    if (residential) {
      set('Temperatuuroverschrijding', round(performance.tojuliMaxK));
      set('Warmtebehoefte', round(indicators?.heatingNeedKwhPerM2));
    }
    const limits = performance.bblCheck?.limits;
    if (limits && !existing) {
      set('Eis_energiebehoefte', round(limits.energyNeedMaxKwhPerM2));
      set('Eis_primaire_fossiele_energie', round(limits.primaryFossilMaxKwhPerM2));
      set('Eis_aandeel_hernieuwbare_energie', round(limits.renewableShareMinPercent, 1));
      if (residential) set('Eis_temperatuuroverschrijding', 1.2);
    }
  } else {
    values.Energieklasse = null;
    omitted.push({ field: 'Energieklasse', reason: 'geen berekend resultaat van de rekenkern' });
  }
  omitted.push(
    { field: 'Energie_Index', reason: 'wordt niet gebruikt bij registraties volgens NTA 8800' },
    { field: 'Energieindex_EMG_forfaitair', reason: 'wordt niet gebruikt bij registraties volgens NTA 8800' },
    { field: 'SBIcode', reason: 'alleen gevuld bij opnames die niet volgens NTA 8800 zijn' },
    { field: 'BerekendeCO2Emissie', reason: 'eenheid en definitie niet openbaar beschreven' },
    { field: 'BerekendeEnergieverbruik', reason: 'eenheid en definitie niet openbaar beschreven' },
  );

  // Schema order.
  const ordered: Partial<Record<EpOnlineField, EpOnlineValue>> = {};
  for (const [field] of EP_ONLINE_FIELDS) {
    if (field in values) ordered[field] = values[field] as EpOnlineValue;
  }
  return {
    notice: EP_ONLINE_OVERVIEW_NOTICE,
    schema: 'EpbdExportTypesV4 (exportschema, Pandcertificaat)',
    pandcertificaat: ordered,
    omitted,
    missingRequired: EP_ONLINE_REQUIRED.filter((field) => !(field in ordered)),
    labelStatements: {
      note: LABEL_STATEMENTS_NOTE,
      // The kernel's label elements k and l first; the registration when there is no label data.
      k_reageertOpExterneSignalen: indicators?.elements.respondsToExternalSignals
        ?? registration.labelStatements?.respondsToExternalSignals ?? null,
      l_afgiftesysteemLageTemperatuur: indicators?.elements.lowTemperatureHeating
        ?? registration.labelStatements?.lowTemperatureHeating ?? null,
    },
  };
}

function escapeXml(value: string): string {
  return value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

/**
 * The overview as the XML of one `Mutatiebericht` of the export schema, for
 * validating the structure against the published XSD. Not an upload file.
 */
export function epOnlineOverviewXml(overview: EpOnlineOverview): string {
  const ns = 'http://schemas.ep-online.nl/EpbdExportTypesV4';
  const body = Object.entries(overview.pandcertificaat).map(([field, value]) => {
    if (value === null) return `<${field} xsi:nil="true"/>`;
    if (Array.isArray(value)) return `<${field}>${value.map((id) => `<BAGPandID>${escapeXml(id)}</BAGPandID>`).join('')}</${field}>`;
    return `<${field}>${escapeXml(String(value))}</${field}>`;
  }).join('');
  return `<?xml version="1.0" encoding="utf-8"?>`
    + `<MutationMessage xmlns="${ns}" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">`
    + `<Mutatiebericht><Mutatievolgnummer>1</Mutatievolgnummer><Stuurcode>1</Stuurcode>`
    + `<Pandcertificaat>${body}</Pandcertificaat></Mutatiebericht></MutationMessage>`;
}
