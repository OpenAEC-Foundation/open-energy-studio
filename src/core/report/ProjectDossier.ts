import { strToU8, zipSync } from 'fflate';
import type { IProject } from '../energy/types';
import type {
  NtaEvidenceItem, NtaEvidenceKind, NtaRegistration, OpnameAssessment, ProjectPerformanceAssessment, RelabelAssessment,
} from '../nta/KernelClient';
import { evidenceArchiveName, loadEvidenceBytes, sha256Hex } from '../nta/Evidence';
import { serializeProject, type KernelStamp } from '../io/ProjectSerializer';

/**
 * Project dossier of an EP adviser (BRL 9500-W Bijlage 3, p. 61–63;
 * BRL 9500-U Bijlage 3): a completeness check that depends on the purpose
 * and survey type, and an export bundle with a hashed manifest. Only page
 * numbers of the BRL are cited; the checklist wording is our own.
 */

export type DossierStatus = 'ok' | 'missing' | 'check' | 'not_applicable';

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
  relabel?: RelabelAssessment | null;
}

function has(evidence: NtaEvidenceItem[], ...kinds: NtaEvidenceKind[]): boolean {
  return evidence.some((item) => kinds.includes(item.kind));
}

function filled(value: string | undefined): boolean {
  return value != null && value.trim() !== '';
}

/** Completeness of the project dossier per BRL 9500 Bijlage 3. */
export function checkDossierCompleteness({ project, assessment, opname, relabel }: DossierContext): DossierItem[] {
  const registration: NtaRegistration = project.registration ?? {};
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
  add('delivered_report', 'general', 'Het aan de opdrachtgever geleverde energieprestatie-rapport', assessment != null);

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
  add('registration_data', 'result', 'Registratiedatum, registrerende adviseur en EP-Online-nummer',
    filled(registration.registrationDate) && filled(registrar?.name) && filled(registration.epOnlineNumber));
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
  add('quality_declarations', 'evidence', 'Gecontroleerde kwaliteits- en gelijkwaardigheidsverklaringen',
    bbl ? has(evidence, 'quality_declaration', 'declaration_of_performance') : null);
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

  // Herlabelen
  if (registration.relabel) {
    add('relabel_changes', 'relabel', 'Overzicht later aangebrachte wijzigingen (Bijlage 6a)',
      relabel ? relabel.allowed : false,
      relabel == null ? 'geen herlabelvergelijking uitgevoerd' : relabel.allowed ? undefined : 'wijziging volgens Bijlage 6b');
    if (relabel?.needsReview) {
      attention('relabel_review', 'relabel', 'Wijzigingen die beoordeling vragen', `${relabel.changes.filter((c) => c.verdict === 'review').length} wijziging(en)`);
    }
    add('relabel_invoice', 'relabel', 'Offerte en opdracht of gespecificeerde factuur van de verbetering', has(evidence, 'invoice'));
    add('relabel_improvement_date', 'relabel', 'Datum van de verbetering binnen 24 maanden na de opname', filled(registration.improvementDate));
    const production = relabel?.changes.some((change) => /pv|solar|production/i.test(change.path)) ?? false;
    add('relabel_production_photos', 'relabel', 'Foto\'s van PV of zonthermie, met beschaduwing',
      production ? has(evidence, 'photo_overview', 'photo_detail') : null);
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
  files: DossierManifestEntry[];
  missingEvidence: Array<{ id: string; fileName: string; reason: string }>;
  checklist: DossierItem[];
}

export interface DossierBundle {
  files: Record<string, Uint8Array>;
  manifest: DossierManifest;
}

/** Builds the dossier files and a manifest with the SHA-256 of each file. */
export async function buildProjectDossier(
  context: DossierContext & { reportHtml?: string | null; generatedAt?: string },
): Promise<DossierBundle> {
  const { project, assessment } = context;
  const kernel: KernelStamp | null = assessment
    ? { kernelVersion: assessment.kernelVersion, targetNormVersion: assessment.targetNormVersion, inputFingerprint: assessment.inputFingerprint }
    : null;
  const files: Record<string, Uint8Array> = {
    'project.oes.json': strToU8(serializeProject(project, kernel)),
  };
  if (assessment) files['kernel-output.json'] = strToU8(JSON.stringify(assessment, null, 2));
  if (context.reportHtml) files['rekenrapport.html'] = strToU8(context.reportHtml);
  if (context.opname) files['basisopname-output.json'] = strToU8(JSON.stringify(context.opname, null, 2));
  if (context.relabel) files['herlabel-vergelijking.json'] = strToU8(JSON.stringify(context.relabel, null, 2));
  const missingEvidence: DossierManifest['missingEvidence'] = [];
  for (const item of project.registration?.evidence ?? []) {
    const bytes = await loadEvidenceBytes(item);
    if (bytes) files[evidenceArchiveName(item)] = bytes;
    else missingEvidence.push({ id: item.id, fileName: item.fileName, reason: 'bestand niet beschikbaar of hash wijkt af' });
  }
  const checklist = checkDossierCompleteness(context);
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
    files: entries,
    missingEvidence,
    checklist,
  };
  files['manifest.json'] = strToU8(JSON.stringify(manifest, null, 2));
  return { files, manifest };
}

/** The bundle as one ZIP archive. */
export function zipProjectDossier(bundle: DossierBundle): Uint8Array {
  // Copy into this realm so zipSync treats each value as a file (see UNIEC3Exporter).
  const copies = Object.fromEntries(Object.entries(bundle.files).map(([path, data]) => [path, new Uint8Array(data)]));
  return zipSync(copies, { level: 6 });
}
