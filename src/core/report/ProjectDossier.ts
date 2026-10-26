import { strToU8, zipSync, type Zippable, type ZippableFile } from 'fflate';
import type { IProject } from '../energy/types';
import type {
  NtaEvidenceItem, NtaEvidenceKind, NtaRegistration, OpnameAssessment, ProjectPerformanceAssessment, RelabelAssessment,
} from '../nta/KernelClient';
import { evidenceArchiveName, loadEvidenceBytes, sha256Hex } from '../nta/Evidence';
import { evidenceUsage } from '../nta/EvidenceLinks';
import { serializeProject, type KernelStamp } from '../io/ProjectSerializer';
import { labelInputSha256, relabelDeadline } from '../nta/Registration';
import { isProductionPath } from '../nta/RelabelText';
import { measureEvidenceNotes } from '../nta/MwaTemplates';
import { buildEpOnlineOverview } from './EpOnlineOverview';
import { attestText, programIdentity, type ProgramIdentity } from './ProgramIdentity';

/**
 * Project dossier of an EP adviser (BRL 9500-W Bijlage 3, p. 61–63;
 * BRL 9500-U Bijlage 3): a completeness check that depends on the purpose
 * and survey type, and an export bundle with a hashed manifest. Only page
 * numbers of the BRL are cited; the checklist wording is our own.
 */

/** `pending`: the kernel run (or survey assessment) this item depends on is still running. */
export type DossierStatus = 'ok' | 'missing' | 'check' | 'not_applicable' | 'pending';

export interface DossierItem {
  id: string;
  group: 'general' | 'building' | 'installations' | 'elaboration' | 'representativity' | 'result' | 'evidence' | 'relabel';
  label: string;
  status: DossierStatus;
  detail?: string;
}

export interface DossierContext {
  project: IProject;
  assessment?: ProjectPerformanceAssessment | null;
  /** Basic survey output, for the collapse reasons of applied defaults. */
  opname?: OpnameAssessment | null;
  /** Relabel comparison; defaults to the one kept with the registration. */
  relabel?: RelabelAssessment | null;
  /** SHA-256 of the project's label input, to see whether the kept comparison is out of date. */
  labelInputSha256?: string | null;
  /** The kernel assessment or the survey assessment is still running (live check only). */
  pending?: boolean;
  /**
   * Whether the dossier carries the delivered report. `undefined` in the live
   * check (the report is produced at export); `false` makes the item missing.
   */
  reportPresent?: boolean;
}

/** Items whose status follows from the kernel run or the survey assessment. */
const KERNEL_DEPENDENT = new Set([
  'delivered_report', 'output_file', 'electronic_files', 'collapse_reasons', 'thermal_properties',
  'relabel_changes', 'relabel_review',
]);

function has(evidence: NtaEvidenceItem[], ...kinds: NtaEvidenceKind[]): boolean {
  return evidence.some((item) => kinds.includes(item.kind));
}

function filled(value: string | undefined): boolean {
  return value != null && value.trim() !== '';
}

/** Input keys that carry a product value from a kwaliteits- or gelijkwaardigheidsverklaring. */
const DECLARATION_KEYS = new Set([
  'qualityDeclaration', 'declaredControlFactor', 'declaredEfficiencies', 'declaredAuxiliaryConstants',
  'highEfficiencyEvidence',
]);
const DECLARATION_FLAGS = new Set(['qualityDeclarationPresent', 'qualityDeclarationUsed']);

/**
 * Whether the NTA input uses a value from a kwaliteits- of
 * gelijkwaardigheidsverklaring (BRL 9500-W §4.2.7 p. 25, Bijlage 3 p. 63; U p. 21).
 */
export function usesQualityDeclaration(value: unknown, path: string[] = []): boolean {
  if (Array.isArray(value)) return value.some((item) => usesQualityDeclaration(item, path));
  if (value == null || typeof value !== 'object') return false;
  for (const [key, child] of Object.entries(value as Record<string, unknown>)) {
    if (child == null) continue;
    if (DECLARATION_KEYS.has(key)) return true;
    if (DECLARATION_FLAGS.has(key) && child === true) return true;
    // A declared generator value (hot water 13.8.4.7, heat-pump COP).
    if (key === 'declared' && path.some((segment) => /generator/i.test(segment)) && typeof child === 'object') return true;
    if (usesQualityDeclaration(child, [...path, key])) return true;
  }
  return false;
}

/** The relabel comparison the kernel ran again at registration, if any. */
function kernelRelabel(assessment: ProjectPerformanceAssessment | null | undefined): RelabelAssessment | null {
  return assessment?.registration?.relabelAssessment ?? null;
}

/** The project as written into the dossier: the stored original project lives in its own file. */
function withoutStoredOriginal(project: IProject): IProject {
  const comparison = project.registration?.relabelComparison;
  if (!comparison?.originalProjectText) return project;
  const { originalProjectText: _omitted, ...rest } = comparison;
  return { ...project, registration: { ...project.registration, relabelComparison: rest } };
}

/** Items that still need the adviser's attention; a pending item is not done yet either. */
export function openDossierItems(items: DossierItem[]): DossierItem[] {
  return items.filter((item) => item.status === 'missing' || item.status === 'check' || item.status === 'pending');
}

/** Completeness of the project dossier per BRL 9500 Bijlage 3. */
export function checkDossierCompleteness(context: DossierContext): DossierItem[] {
  const items = dossierItems(context);
  if (!context.pending) return items;
  return items.map((item) => KERNEL_DEPENDENT.has(item.id) && (item.status === 'missing' || item.status === 'check')
    ? { id: item.id, group: item.group, label: item.label, status: 'pending' as const }
    : item);
}

function dossierItems({ project, assessment, opname, relabel: given, labelInputSha256: currentSha, reportPresent }: DossierContext): DossierItem[] {
  const registration: NtaRegistration = project.registration ?? {};
  // The verdict comes from the kernel's fresh comparison at registration,
  // never from the stored one (BRL 9500-W Bijlage 3, p. 63).
  const relabel = given ?? kernelRelabel(assessment);
  const evidence = registration.evidence ?? [];
  const bbl = registration.purpose === 'bbl_check';
  const delivery = registration.purpose === 'delivery';
  // For toets Bbl the installation items follow from design data (Bijlage 3).
  const installationEvidence: NtaEvidenceKind[] = bbl
    ? ['datasheet', 'drawing', 'declaration_of_performance', 'quality_declaration']
    : ['photo_overview', 'photo_detail', 'invoice', 'datasheet'];
  const items: DossierItem[] = [];
  const add = (id: string, group: DossierItem['group'], label: string, ok: boolean | null, detail?: string) => {
    const status: DossierStatus = ok === null ? 'not_applicable' : ok ? 'ok' : 'missing';
    items.push({ id, group, label, status, ...(detail ? { detail } : {}) });
  };
  const attention = (id: string, group: DossierItem['group'], label: string, detail: string) =>
    items.push({ id, group, label, status: 'check', detail });

  // Algemeen
  add('address', 'general', 'Postcode en huisnummer', filled(registration.postcode) && filled(registration.houseNumber));
  add('purpose', 'general', 'Doel: toets Bbl, oplevering of bestaand', registration.purpose != null);
  add('survey_type', 'general', 'Basis- of detailopname', registration.surveyType != null);
  add('client_information', 'general', 'Bewijs van het informeren van de opdrachtgever en van door de opdrachtgever aangereikte gegevens',
    has(evidence, 'client_statement'));
  // The report itself must be in the dossier (BRL 9500-W Bijlage 3 p. 61, U p. 53).
  add('delivered_report', 'general', 'Het aan de opdrachtgever geleverde energieprestatie-rapport',
    assessment != null && reportPresent !== false,
    reportPresent === false ? 'het dossier bevat geen energieprestatie-rapport' : undefined);

  // Bouwkundige gegevens
  add('floor_plan', 'building', 'Plattegrond met maatvoering en indeling', has(evidence, 'drawing'));
  add('section', 'building', 'Doorsnede of foto met de hoogte van het gebouw', has(evidence, 'drawing', 'photo_overview'));
  const surfaces = (project.zones ?? []).flatMap((zone) => zone.surfaces ?? []);
  add('entered_areas', 'building', 'Ingevoerde oppervlakten van vloer, dak, gevels, ramen en panelen',
    surfaces.length > 0 && surfaces.every((surface) => (surface.area ?? 0) > 0));
  add('construction_year', 'building', 'Bouwjaar en eventueel renovatiejaar', registration.constructionYear != null);
  add('roof_type', 'building', 'Type dak (tekening of foto)', has(evidence, 'drawing', 'photo_overview', 'photo_detail'));

  // Installatietechnische gegevens
  add('installation_location', 'installations', 'Locatie van de opwekkers (overzichtsfoto of tekening)',
    has(evidence, ...(bbl ? ['drawing', 'datasheet'] as NtaEvidenceKind[] : ['photo_overview', 'drawing'] as NtaEvidenceKind[])));
  add('installation_types', 'installations', 'Type opwekkers, ventilatie, tapwater en energieproductie (foto, factuur of ontwerpgegevens)',
    has(evidence, ...installationEvidence));
  // Utility buildings also record humidification and lighting (BRL 9500-U Bijlage 3 p. 53).
  const utility = assessment?.derivedInput?.calculationScope === 'utility'
    || (assessment?.derivedInput == null && project.buildingFunction != null && project.buildingFunction !== 'residential');
  if (utility) {
    add('humidification_type', 'installations', 'Type bevochtiging en/of ontvochtiging (foto, factuur of ontwerpgegevens)',
      has(evidence, ...installationEvidence));
    add('lighting_type', 'installations', 'Type verlichting (foto en/of factuur)',
      has(evidence, ...(bbl ? installationEvidence : ['photo_overview', 'photo_detail', 'invoice'] as NtaEvidenceKind[])));
  }

  // Uitwerking
  const applied = opname?.appliedDefaults ?? [];
  if (opname) {
    const without = applied.filter((item) => !filled(item.inklapReden));
    add('collapse_reasons', 'elaboration', 'Onderbouwing van het inklappen naar forfaitaire waarden', without.length === 0,
      without.length ? `${without.length} toegepaste forfaitaire waarde(n) zonder inklapreden` : undefined);
  } else {
    add('collapse_reasons', 'elaboration', 'Onderbouwing van het inklappen naar forfaitaire waarden',
      registration.surveyType === 'basic' ? false : null,
      registration.surveyType === 'basic' ? 'geen basisopname-uitvoer meegegeven' : undefined);
  }
  add('schematisation', 'elaboration', 'Onderbouwing schematisering en rekenzones', has(evidence, 'drawing'));
  add('thermal_properties', 'elaboration', 'Beschrijving van de bepaling van de thermische eigenschappen (materiaal en dikte, of bouwjaarklasse)',
    opname != null || has(evidence, 'datasheet', 'declaration_of_performance', 'photo_detail', 'drawing'));
  attention('parameters_used', 'elaboration', 'Onderbouwing welke elementen, parameters en gegevens zijn gebruikt',
    'leg vast in het dossier; de kernuitvoer en de invoer tonen de gebruikte waarden');
  attention('isso_calculations', 'elaboration', 'Berekeningen en aanvullende gegevens die volgens ISSO 82.1/75.1 in het dossier horen',
    'bijvoorbeeld de onderbouwing van koudebruggen, beschaduwing en bijzondere constructies');
  if (bbl) {
    attention('apartment_labels', 'elaboration', 'Onderbouwing als individuele appartementen geen energielabel krijgen',
      'alleen van toepassing bij toets Bbl van een woongebouw');
  }

  // Representativiteit
  const representative = registration.representation != null && registration.representation !== 'unique';
  // Both reference and similar objects need the substantiation, the subset
  // overview, the visited objects and the characteristics (Bijlage 3).
  const representativityEvidence = evidence.some((item) =>
    (item.linkedPaths ?? []).some((path) => path.startsWith('/registration/representation')));
  add('representativity', 'representativity', 'Onderbouwing representativiteit, deelverzameling, bezochte objecten en kenmerken',
    representative
      ? representativityEvidence && (registration.representation !== 'similar' || filled(registration.referenceObjectId))
      : null,
    representative && !representativityEvidence ? 'koppel bewijs aan /registration/representation' : undefined);

  // Resultaat berekening
  const surveyor = registration.surveyingAdvisor;
  add('input_file', 'result', 'Invoerbestand met opnamedatum en naam en nummer van de opnemende adviseur',
    filled(registration.surveyDate) && filled(surveyor?.name) && filled(surveyor?.competenceNumber));
  add('output_file', 'result', 'Volledige uitvoer van het rekenprogramma',
    assessment != null && assessment.status === 'calculated_unverified',
    assessment == null ? 'geen kernuitvoer' : assessment.status !== 'calculated_unverified' ? `kernstatus ${assessment.status}` : undefined);
  const registrar = registration.registeringAdvisor;
  // Name and registration number of the registering adviser (BRL 9500-W Bijlage 3 p. 62, U p. 54).
  add('registration_data', 'result', 'Registratiedatum, naam en registratienummer van de registrerende adviseur en EP-Online-nummer',
    filled(registration.registrationDate) && filled(registrar?.name) && filled(registrar?.competenceNumber)
      && filled(registration.epOnlineNumber),
    filled(registrar?.name) && !filled(registrar?.competenceNumber) ? 'registratienummer van de registrerende adviseur ontbreekt' : undefined);
  add('electronic_files', 'result', 'Elektronische bestanden van de energieprestatieberekening', assessment != null);
  if (bbl || delivery) {
    attention('gto_cooling_load', 'result', 'GTO- en koellastberekeningen, indien gemaakt', 'opnemen als ze zijn gemaakt');
  }
  // New buildings over 1000 m² tested against the Bbl from 1-1-2028, and
  // their later delivery (BRL 9500-W p. 21 and Bijlage 3 p. 62).
  const area = assessment?.geometry?.usableFloorAreaM2 ?? 0;
  const dated = registration.registrationDate ?? registration.surveyDate ?? '';
  if ((bbl || delivery) && area > 1000 && dated >= '2028-01-01') {
    attention('wlc_gwp', 'result', 'WLC-GWP-berekening (nieuwe gebouwen > 1000 m²)',
      'verplicht bij toets Bbl vanaf 1-1-2028 en bij de daaropvolgende oplevering (BRL 9500-W p. 21, Bijlage 3 p. 62)');
  }

  // Bewijsmateriaal
  add('evidence_present', 'evidence', 'Bewijsmateriaal met inhoudsopgave', evidence.length > 0);
  const unchecked = evidence.filter((item) => !filled(item.checkedBy));
  add('evidence_checked', 'evidence', 'Controle door de adviseur van aangeleverde gegevens',
    evidence.length ? unchecked.length === 0 : null,
    unchecked.length ? `${unchecked.length} bestand(en) zonder controle` : undefined);
  add('construction_phase', 'evidence', 'Tijdens de bouw verzamelde gegevens (Bijlage 5)',
    delivery ? has(evidence, 'photo_detail', 'photo_overview') : null);
  // Applies whenever the input uses a declared product value, for every purpose:
  // the declaration with merk and type, plus a photo or invoice (W Bijlage 3 p. 63,
  // §4.2.7 p. 25; U p. 21). At toets Bbl the design data stand in for the photo.
  const declared = usesQualityDeclaration(project.ntaCalculation);
  const declarationShown = has(evidence, 'quality_declaration', 'declaration_of_performance');
  const declarationProven = bbl || has(evidence, 'photo_detail', 'invoice');
  add('quality_declarations', 'evidence', 'Gecontroleerde kwaliteits- en gelijkwaardigheidsverklaringen (merk, type) met foto of factuur',
    declared || bbl ? declarationShown && declarationProven : null,
    declared && !declarationShown ? 'de invoer gebruikt een kwaliteitsverklaring; voeg de verklaring toe'
      : declared && !declarationProven ? 'voeg een foto of factuur van het product toe' : undefined);
  // Register snapshots (issue #15): confirmed by the adviser and not withdrawn;
  // the kernel's registration check gives the finding per snapshot.
  const snapshots = evidence.flatMap((item) => (item.registerSnapshot ? [item.registerSnapshot] : []));
  if (snapshots.length > 0) {
    const unconfirmed = snapshots.filter((snapshot) => !snapshot.confirmedBy?.trim() || !snapshot.confirmedOn).length;
    const withdrawn = snapshots.filter((snapshot) => snapshot.registryCheck?.state === 'withdrawn').length;
    const unchecked = snapshots.filter((snapshot) => (snapshot.registryCheck?.state ?? 'not_checked') === 'not_checked').length;
    add('register_snapshots', 'evidence', 'Overgenomen registerverklaringen: momentopname bevestigd, niet ingetrokken',
      unconfirmed === 0 && withdrawn === 0,
      [unconfirmed ? `${unconfirmed} niet bevestigd` : '', withdrawn ? `${withdrawn} ingetrokken` : '',
        unchecked ? `${unchecked} niet in het register gecontroleerd` : ''].filter(Boolean).join('; ') || undefined);
  }
  add('photos', 'evidence', 'Leesbare foto\'s van typeaanduiding en maatvoering',
    bbl ? null : has(evidence, 'photo_detail'));
  add('invoices', 'evidence', 'Facturen op naam en/of adres', bbl ? null : has(evidence, 'invoice'));
  if (has(evidence, 'datasheet', 'declaration_of_performance', 'quality_declaration')) {
    add('product_documentation', 'evidence', 'Productdocumentatie van de fabrikant met kwaliteitsverklaring en/of DoP', true);
  } else {
    attention('product_documentation', 'evidence', 'Productdocumentatie van de fabrikant met kwaliteitsverklaring en/of DoP',
      'bij fabrieksmatig geproduceerde elementen, indien van toepassing');
  }
  if (bbl) {
    attention('forfait_justification', 'evidence', 'Onderbouwing van gebruikte forfaitaire waarden', 'alleen bij toets Bbl');
  }
  // Maatwerkadvies: PV on minimal obstruction needs its evidence (table 17.3 situation a), p. 706–707).
  const pvNotes = (project.maatwerkadvies?.measures ?? []).flatMap((measure) =>
    measureEvidenceNotes(measure).map((note) => `${measure.name} / PV ${note.id}: ${note.source}`));
  if (pvNotes.length > 0) {
    attention('mwa_pv_minimal_obstruction', 'evidence', 'Onderbouwing minimale belemmering van PV in het maatwerkadvies (NTA 8800 tabel 17.3 situatie a)',
      pvNotes.join('; '));
  }

  // Herlabelen (BRL 9500-W §4.2.3 p. 23 en Bijlage 3 p. 63; 9500-U p. 18–19 en p. 54)
  if ((registration.messageType ?? (registration.relabel ? 'relabel' : 'regular')) === 'relabel') {
    const stored = registration.relabelComparison;
    const kernelOutdated = (assessment?.registration?.issues ?? [])
      .some((item) => item.code === 'relabel_comparison_outdated');
    const outdated = !given && (kernelOutdated
      || Boolean(stored?.currentSha256 && currentSha && stored.currentSha256 !== currentSha));
    if (!given && stored && !stored.originalProjectText) {
      attention('relabel_changes', 'relabel', 'Overzicht later aangebrachte wijzigingen (Bijlage 6a)',
        'de vergelijking bevat het oorspronkelijke projectbestand niet; vergelijk opnieuw zodat de registratiecontrole het kan nagaan');
    } else if (outdated) {
      attention('relabel_changes', 'relabel', 'Overzicht later aangebrachte wijzigingen (Bijlage 6a)',
        'het project is na de herlabelvergelijking gewijzigd; vergelijk opnieuw');
    } else if (relabel == null && stored) {
      attention('relabel_changes', 'relabel', 'Overzicht later aangebrachte wijzigingen (Bijlage 6a)',
        'de rekenkern heeft de bewaarde vergelijking niet opnieuw kunnen uitvoeren; controleer de vergelijking');
    } else {
      add('relabel_changes', 'relabel', 'Overzicht later aangebrachte wijzigingen (Bijlage 6a)',
        relabel ? relabel.allowed : false,
        relabel == null ? 'geen herlabelvergelijking uitgevoerd' : relabel.allowed ? undefined : 'wijziging volgens Bijlage 6b');
    }
    if (relabel?.needsReview) {
      attention('relabel_review', 'relabel', 'Wijzigingen die beoordeling vragen', `${relabel.changes.filter((c) => c.verdict === 'review').length} wijziging(en)`);
    }
    add('relabel_original_label', 'relabel', 'Certificaathouder en EP-Online-nummer van het oorspronkelijke label',
      filled(registration.originalCertificateNumber) && filled(registration.originalEpOnlineNumber)
        && registration.originalCertificateNumber!.trim().toLowerCase() === (registration.certificateNumber ?? '').trim().toLowerCase(),
      filled(registration.originalCertificateNumber) && registration.originalCertificateNumber!.trim().toLowerCase()
        !== (registration.certificateNumber ?? '').trim().toLowerCase() ? 'alleen de certificaathouder van het oorspronkelijke label mag herlabelen' : undefined);
    const proofs = evidence.filter((item) => item.relabelProof === 'quote_with_order' || item.relabelProof === 'specified_invoice');
    add('relabel_invoice', 'relabel', 'Offerte met opdracht of gespecificeerde factuur van de verbetering op dit adres',
      proofs.length > 0,
      proofs.length ? undefined : 'markeer het bewijsstuk als offerte met opdracht of gespecificeerde factuur van de herlabeling');
    const deadline = relabelDeadline(registration.surveyDate);
    const improvement = registration.improvementDate ?? '';
    const withinWindow = filled(improvement) && deadline != null
      && improvement >= (registration.surveyDate ?? '') && improvement <= deadline;
    add('relabel_improvement_date', 'relabel', 'Datum van de verbetering binnen 24 maanden na de opname', withinWindow,
      !filled(improvement) ? undefined
        : deadline == null ? 'opnamedatum ontbreekt'
          : withinWindow ? undefined : `buiten de periode ${registration.surveyDate} t/m ${deadline}`);
    // Which evidence is required follows the fresh comparison, else the
    // stored one: a requirement is never dropped for lack of a re-run.
    const scope = relabel ?? stored?.assessment ?? null;
    const production = scope?.changes.some((change) => isProductionPath(change.path)) ?? false;
    add('relabel_production_photos', 'relabel', 'Foto\'s van PV of zonthermie, met beschaduwing',
      production ? evidence.some((item) => item.relabelProof === 'production_photo') : null);
    add('relabel_production_connection', 'relabel', 'PV of zonthermie exclusief en fysiek verbonden met de gebouwinstallatie',
      production ? registration.productionPhysicallyConnected === true : null);
    add('relabel_utility_confirmation', 'relabel', 'Vastgesteld dat er geen wijzigingen volgens Bijlage 6b zijn (utiliteit)',
      scope?.scheme === 'u' ? registration.noExcludedChangesConfirmed === true : null);
  }
  return items;
}

export interface DossierManifestEntry {
  path: string;
  sha256: string;
  bytes: number;
}

export interface DossierManifest {
  generatedAt: string;
  projectName: string;
  kernel: KernelStamp | null;
  attestStatus: string | null;
  /**
   * The program that made the files (Omgevingsregeling art. 5.14 lid 1 onder b; BRL 9501 §5.2
   * and §6.1): name and program version apart from the rekenkern, and the attest number, or
   * `attestNumberText` "nog niet toegekend" while there is none.
   */
  software: ProgramIdentity & { attestNumberText: string };
  files: DossierManifestEntry[];
  /** Table of contents of the evidence (Bijlage 3 "bewijsmateriaal met inhoudsopgave"): file, hash and what it supports. */
  evidence: DossierEvidenceEntry[];
  missingEvidence: Array<{ id: string; fileName: string; reason: string }>;
  checklist: DossierItem[];
}

export interface DossierEvidenceEntry {
  id: string;
  kind: NtaEvidenceKind;
  fileName: string;
  sha256: string;
  date: string | null;
  /** Path in the ZIP; null when the file was not available. */
  archivePath: string | null;
  checkedBy: string | null;
  /** Inputs it supports: JSON pointers into the project (linked paths and `evidence:<id>` references). */
  supports: string[];
  /** What was taken over from a register declaration, with the adviser's confirmation and register check (issue #15). */
  registerSnapshot?: NtaEvidenceItem['registerSnapshot'];
}

export interface DossierBundle {
  files: Record<string, Uint8Array>;
  manifest: DossierManifest;
}

function dossierSoftware(assessment: ProjectPerformanceAssessment | null | undefined): DossierManifest['software'] {
  const identity = programIdentity(assessment?.kernelVersion, assessment?.targetNormVersion);
  return { ...identity, attestNumberText: attestText(identity) };
}

/** Builds the dossier files and a manifest with the SHA-256 of each file. */
export async function buildProjectDossier(
  context: DossierContext & { reportHtml?: string | null; generatedAt?: string },
): Promise<DossierBundle> {
  const { project, assessment } = context;
  const stored = project.registration?.relabelComparison;
  const rerun = kernelRelabel(assessment);
  const labelSha = context.labelInputSha256 ?? await labelInputSha256(project).catch(() => null);
  const kernel: KernelStamp | null = assessment
    ? { kernelVersion: assessment.kernelVersion, targetNormVersion: assessment.targetNormVersion, inputFingerprint: assessment.inputFingerprint }
    : null;
  const files: Record<string, Uint8Array> = {
    // The original project of a relabel comparison goes in once, as its
    // own file, not again inside the project file.
    'project.oes.json': strToU8(serializeProject(withoutStoredOriginal(project), kernel)),
  };
  if (assessment) files['kernel-output.json'] = strToU8(JSON.stringify(assessment, null, 2));
  // Check overview against the label in EP-Online afterwards; not an upload file.
  files['ep-online-gegevensoverzicht.json'] = strToU8(JSON.stringify(buildEpOnlineOverview(project, assessment), null, 2));
  if (context.reportHtml) files['rekenrapport.html'] = strToU8(context.reportHtml);
  if (context.opname) files['basisopname-output.json'] = strToU8(JSON.stringify(context.opname, null, 2));
  if (context.relabel) {
    files['herlabel-vergelijking.json'] = strToU8(JSON.stringify({ assessment: context.relabel }, null, 2));
  } else if (stored) {
    // The original project goes in as its own file, so the manifest's
    // SHA-256 of it equals the comparison's `originalSha256`. The verdict
    // is the kernel's fresh comparison; without one the file says so.
    const { originalProjectText, assessment: _storedVerdict, ...rest } = stored;
    const record = rerun
      ? { ...rest, assessment: rerun, kernelRecheck: true }
      : { ...rest, kernelRecheck: false };
    files['herlabel-vergelijking.json'] = strToU8(JSON.stringify(record, null, 2));
    if (originalProjectText) files['herlabel-origineel.oes.json'] = strToU8(originalProjectText);
  }
  const missingEvidence: DossierManifest['missingEvidence'] = [];
  const evidenceEntries: DossierEvidenceEntry[] = [];
  const usage = evidenceUsage(project);
  for (const item of project.registration?.evidence ?? []) {
    const bytes = await loadEvidenceBytes(item);
    // Two entries can map to the same name (a repeated id, or ids that
    // differ only in characters safeName drops): the second gets a suffix,
    // so no file overwrites another and the manifest hash stays true.
    const archivePath = bytes ? uniqueArchivePath(evidenceArchiveName(item), files) : null;
    if (bytes && archivePath) files[archivePath] = bytes;
    else missingEvidence.push({ id: item.id, fileName: item.fileName, reason: 'bestand niet beschikbaar of hash wijkt af' });
    evidenceEntries.push({
      id: item.id, kind: item.kind, fileName: item.fileName, sha256: item.sha256, date: item.date ?? null,
      archivePath, checkedBy: item.checkedBy ?? null,
      supports: usage.get(item.id) ?? [],
      ...(item.registerSnapshot ? { registerSnapshot: item.registerSnapshot } : {}),
    });
  }
  const checklist = checkDossierCompleteness({ ...context, pending: false, labelInputSha256: labelSha, reportPresent: Boolean(context.reportHtml) });
  files['dossier-checklist.json'] = strToU8(JSON.stringify(checklist, null, 2));
  const entries: DossierManifestEntry[] = [];
  for (const [path, bytes] of Object.entries(files).sort(([a], [b]) => a.localeCompare(b))) {
    entries.push({ path, sha256: await sha256Hex(bytes), bytes: bytes.length });
  }
  const manifest: DossierManifest = {
    generatedAt: context.generatedAt ?? new Date().toISOString(),
    projectName: project.name,
    kernel,
    attestStatus: assessment?.attestStatus ?? null,
    software: dossierSoftware(assessment),
    files: entries,
    evidence: evidenceEntries,
    missingEvidence,
    checklist,
  };
  files['manifest.json'] = strToU8(JSON.stringify(manifest, null, 2));
  return { files, manifest };
}

/** `path`, or `name-2.ext`, `name-3.ext` … when `path` is already taken (case-insensitively). */
export function uniqueArchivePath(path: string, taken: Record<string, unknown>): string {
  const used = new Set(Object.keys(taken).map((key) => key.toLowerCase()));
  if (!used.has(path.toLowerCase())) return path;
  const slash = path.lastIndexOf('/');
  const dot = path.lastIndexOf('.');
  const split = dot > slash + 1 ? dot : path.length;
  const base = path.slice(0, split);
  const extension = path.slice(split);
  let index = 2;
  while (used.has(`${base}-${index}${extension}`.toLowerCase())) index += 1;
  return `${base}-${index}${extension}`;
}

/**
 * Largest dossier written as one ZIP. fflate writes no ZIP64 records, so an
 * archive over 4 GiB would be corrupt; and every byte sits in memory twice
 * while zipping. Above this limit the export stops with a clear message.
 */
export const MAX_DOSSIER_BYTES = 2 * 1024 * 1024 * 1024;

/** Thrown when a dossier would exceed `MAX_DOSSIER_BYTES`. */
export class DossierTooLargeError extends Error {
  constructor(readonly totalBytes: number) {
    super(`dossier_too_large: ${totalBytes} bytes`);
    this.name = 'DossierTooLargeError';
  }
}

/** File types that are already compressed: stored, not deflated again. */
const STORED_EXTENSIONS = new Set([
  'jpg', 'jpeg', 'png', 'gif', 'webp', 'heic', 'heif', 'pdf', 'zip', 'docx', 'xlsx', 'odt', 'ods',
  'mp4', 'mov', 'm4v', 'mp3', 'gz', '7z',
]);

function alreadyCompressed(path: string): boolean {
  const dot = path.lastIndexOf('.');
  return dot > path.lastIndexOf('/') && STORED_EXTENSIONS.has(path.slice(dot + 1).toLowerCase());
}

/** The bundle as one ZIP archive. */
export function zipProjectDossier(bundle: DossierBundle): Uint8Array {
  const total = Object.values(bundle.files).reduce((sum, data) => sum + data.byteLength, 0);
  if (total > MAX_DOSSIER_BYTES) throw new DossierTooLargeError(total);
  // Copy into this realm so zipSync treats each value as a file (see UNIEC3Exporter).
  // Photos, PDFs and other compressed files are stored (level 0): deflating
  // them again only costs time and memory.
  const copies: Zippable = Object.fromEntries(Object.entries(bundle.files).map(([path, data]): [string, ZippableFile] => [
    path,
    alreadyCompressed(path) ? [new Uint8Array(data), { level: 0 }] : new Uint8Array(data),
  ]));
  return zipSync(copies, { level: 6 });
}
