#!/usr/bin/env node
// Maakt referentiemanifesten voor de EDR-testen van ISSO-publicatie 54 (BRL 9501).
//
// De ISSO-bestanden (opnameformulieren, referentiewaarden) zijn gelicentieerd en
// staan buiten de repository. Dit script leest ze uit een map en schrijft per
// EDR-test een manifest dat `reference_gate` kan vergelijken, plus een dekkingsplan.
//
// Gebruik:
//   node scripts/edr-manifests.js build <edr-map> <referentiewaarden.json> <uitvoermap>
//       <edr-map> bevat <test>.survey.json (ResidentialSurvey/UtilitySurvey) of
//       <test>.project.json (.oes-project); <test> is bijvoorbeeld EPWRealB01.
//   node scripts/edr-manifests.js plan <gate-rapport.json> <plan.json>
//       Leest het JSON-rapport van reference_gate en schrijft het dekkingsplan met
//       de manifestvingerafdrukken, zodat elke wijziging van een manifest opvalt.
//
// Voorbeeld van de volledige keten:
//   node scripts/edr-manifests.js build ~/edr "~/edr/edr-referentiewaarden-2026.json" ~/edr/manifests
//   cargo run --manifest-path crates/nta8800-service/Cargo.toml --bin reference_gate -- ~/edr/manifests/*.json > rapport.json
//   node scripts/edr-manifests.js plan rapport.json ~/edr/plan.json
//   NTA_REFERENCE_PLAN=~/edr/plan.json NTA_REFERENCE_CASE_DIR=~/edr/manifests scripts/verify-nta.sh

import fs from 'node:fs';
import path from 'node:path';

const TARGET_NORM_VERSION = 'NTA 8800:2025+C1:2026';
const TOLERANCE_FRACTION = 0.01; // BRL 9501: maximaal 1 % afwijking per resultaat.
const ZERO_TOLERANCE = 0.005; // Bij een verwachte 0 geldt een vaste kleine marge.

// ISSO 54-post → kernelpad in het referentiemanifest.
const POSTS = [
  { post: 'E;we;H+C;nd;ventsys=C1', path: 'beng1', unit: 'kWh/m2.year', norm: 'NTA 8800 §5.4 (BENG 1, vast ventilatiesysteem C1)' },
  { post: 'E;we;PTot', path: 'beng2', unit: 'kWh/m2.year', norm: 'NTA 8800 §5.5 (BENG 2)' },
  { post: 'RER;PrenTot', path: 'beng3', unit: '%', norm: 'NTA 8800 §5.6 (BENG 3)' },
  { post: 'TOjul;max', path: 'tojuliMax', unit: 'K', norm: 'NTA 8800 §5.7 (TOjuli)' },
  { post: 'QH;nd;net', path: 'heatingNeedPerM2', unit: 'kWh/m2.year', norm: 'NTA 8800 hoofdstuk 7 (netto warmtebehoefte per m²)' },
  { post: 'EweFinal', path: 'finalEnergyPerM2', unit: 'kWh/m2.year', norm: 'NTA 8800 §5.9 (finaal energiegebruik)' },
];

function fail(message) {
  console.error(message);
  process.exit(2);
}

function referenceTestId(caseId) {
  // EPWRealB01 → "EPWReal B01", EPUReal B01 idem; deeltests (EPW001a) blijven gelijk.
  const match = /^(EP[WU]Real)([BD]\d{2})$/.exec(caseId);
  return match ? `${match[1]} ${match[2]}` : caseId;
}

function build(edrDir, referenceFile, outDir) {
  const reference = JSON.parse(fs.readFileSync(referenceFile, 'utf8'));
  const sets = {
    ...reference.samenvatting.basistests,
    ...reference.samenvatting.realistischeGebouwen,
  };
  const source = {
    publisher: 'ISSO / InstallQ',
    documentId: `ISSO-publicatie 54 bijlage 2: ${reference.bron.bestand}`,
    edition: `EDR-testen NTA 8800 2025, correctie 16-04-2026 (laatst gewijzigd ${reference.bron.laatstGewijzigd})`,
    usePermission: 'gelicentieerd ISSO-materiaal; alleen lokaal gebruik, niet in de openbare repository',
    independentReviewer: 'ISSO (kolom Resultaat; waarden van INNAX, Uniec, Vabi, 2Snoeken en LabelWise in de Excel)',
  };
  fs.mkdirSync(outDir, { recursive: true });
  const written = [];
  for (const file of fs.readdirSync(edrDir).sort()) {
    const survey = file.endsWith('.survey.json');
    const project = file.endsWith('.project.json');
    if (!survey && !project) continue;
    const caseId = file.replace(/\.(survey|project)\.json$/, '');
    const expectedSet = sets[referenceTestId(caseId)];
    if (!expectedSet) {
      console.warn(`${file}: geen referentiewaarden voor ${referenceTestId(caseId)}, overgeslagen`);
      continue;
    }
    const input = JSON.parse(fs.readFileSync(path.join(edrDir, file), 'utf8'));
    const expected = [];
    for (const { post, path: metricPath, unit, norm } of POSTS) {
      const entry = expectedSet[post];
      if (!entry || typeof entry.value !== 'number') continue;
      expected.push({
        path: metricPath,
        value: entry.value,
        unit,
        normReference: `${norm}; ISSO 54 resultaatnr. ${entry.resultaatnr}`,
        absoluteTolerance: entry.value === 0 ? ZERO_TOLERANCE : Math.abs(entry.value) * TOLERANCE_FRACTION,
      });
    }
    const manifest = {
      caseId,
      normVersion: TARGET_NORM_VERSION,
      ...(survey
        ? { survey: { kind: caseId.startsWith('EPU') ? 'utility' : 'residential', input } }
        : { project: input }),
      source,
      expected,
    };
    const target = path.join(outDir, `${caseId}.json`);
    fs.writeFileSync(target, JSON.stringify(manifest, null, 1) + '\n');
    written.push(target);
  }
  if (written.length === 0) fail(`geen *.survey.json of *.project.json in ${edrDir}`);
  console.log(`${written.length} manifesten geschreven naar ${outDir}`);
}

function plan(reportFile, planFile) {
  const report = JSON.parse(fs.readFileSync(reportFile, 'utf8'));
  if (!Array.isArray(report.cases) || report.cases.length === 0) fail('rapport bevat geen cases');
  const requiredCases = report.cases.map(({ comparison }) => ({
    caseId: comparison.caseId,
    manifestFingerprint: comparison.manifestFingerprint,
    requiredPaths: comparison.metrics.map((metric) => metric.path),
  }));
  const output = { targetNormVersion: report.targetNormVersion, requiredCases };
  fs.writeFileSync(planFile, JSON.stringify(output, null, 1) + '\n');
  console.log(`dekkingsplan met ${requiredCases.length} gevallen geschreven naar ${planFile}`);
}

const [, , command, ...args] = process.argv;
if (command === 'build' && args.length === 3) build(...args);
else if (command === 'plan' && args.length === 2) plan(...args);
else fail('gebruik: edr-manifests.js build <edr-map> <referentiewaarden.json> <uitvoermap> | plan <rapport.json> <plan.json>');
