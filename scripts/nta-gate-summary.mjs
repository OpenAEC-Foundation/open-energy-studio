#!/usr/bin/env node
// Summary of a release gate run (BRL 9501 §6.2–6.3), read from what the gate
// itself wrote: the gate log of scripts/verify-nta.sh and the reference-suite
// report of reference_gate. scripts/release-nta.sh writes it as
// gate-samenvatting.json and fills the test report and the GitHub release
// text from it, so no test count in a delivery document is typed by hand.
//
//   node scripts/nta-gate-summary.mjs --log <gate.log> [--reference <reference-report.json>]
//        [--planned-reference <reference-report.json>]
//        --out <gate-samenvatting.json> [--require-pass [--allow-missing-steps]]
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { normaliseNewlines } from './nta-kernel-version.mjs';

const RUST_RESULT = /^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;/;
const VITEST_FILES = /^\s*Test Files\s+(.*)\((\d+)\)\s*$/;
const VITEST_TESTS = /^\s*Tests\s+(.*)\((\d+)\)\s*$/;
const STEP_TIME = /^\s*(\d+)s {2}(.+)$/;
// Colour codes, should a tool print them into the log (FORCE_COLOR, a terminal under tee).
const ANSI = /\u001b\[[0-9;]*[A-Za-z]/g;
const PASSED_LINE = 'All configured technical NTA gates passed';

/**
 * Steps every release gate runs (verify-nta.sh). A log that passed without one
 * of them was run with a skip switch (NTA_SKIP_MSRV, NTA_SKIP_KERNEL_E2E) or by
 * another script, and is no evidence for a release. Matched by name prefix.
 */
export const REQUIRED_STEPS = [
  'kernel version and release notes',
  'manual stamped for this kernel version',
  'tests crates/nta8800-core',
  'clippy crates/nta8800-core',
  'tests crates/nta8800-service',
  'clippy crates/nta8800-service',
  'reference suites',
  'TypeScript',
  'core on Rust 1.77.2',
  'frontend bundle',
  'Tauri check',
  'Tauri command tests',
  'UI tests',
  'app against the real kernel',
  'project-form states through the kernel',
];

function vitestCounts(text) {
  const count = (word) => Number((new RegExp(`(\\d+) ${word}`).exec(text) ?? [])[1] ?? 0);
  return { passed: count('passed'), failed: count('failed'), skipped: count('skipped') };
}

/**
 * The gate steps in run order with their Rust and Vitest counts, and the step
 * times of the closing "Gate step times:" table. A step that ran in the
 * background prints its own header again when it is collected; both parts
 * count towards the one step.
 */
export function summariseGateLog(logText) {
  const lines = normaliseNewlines(logText).replace(ANSI, '').split('\n');
  const steps = [];
  const byName = new Map();
  let current = null;
  let inTimes = false;
  const times = [];
  for (const line of lines) {
    if (line.startsWith('Gate step times:')) {
      inTimes = true;
      continue;
    }
    if (inTimes) {
      const time = STEP_TIME.exec(line);
      if (time) {
        // "(failed, exit N)" marks the step the gate stopped at; the name is the step's own.
        const failedMark = / \(failed, exit \d+\)$/.exec(time[2]);
        const name = time[2].trim().replace(/\(report in \S*?\/?evidence\/reference\)/, '(report in evidence/reference)');
        times.push({
          name: failedMark ? name.slice(0, name.length - failedMark[0].length).trim() : name,
          seconds: Number(time[1]),
          failed: Boolean(failedMark),
        });
      }
      continue;
    }
    const header = /^==> (.+?)(?: \(started in the background, log .*\))?$/.exec(line);
    if (header) {
      // The evidence path of a release is local to the machine: name it by
      // its place in the archive.
      const name = header[1].trim().replace(/\(report in \S*?\/?evidence\/reference\)/, '(report in evidence/reference)');
      current = byName.get(name);
      if (!current) {
        current = { name, rust: null, vitest: null };
        byName.set(name, current);
        steps.push(current);
      }
      continue;
    }
    if (!current) continue;
    const rust = RUST_RESULT.exec(line);
    if (rust) {
      current.rust ??= { binaries: 0, passed: 0, failed: 0, ignored: 0 };
      current.rust.binaries += 1;
      current.rust.passed += Number(rust[2]);
      current.rust.failed += Number(rust[3]);
      current.rust.ignored += Number(rust[4]);
      continue;
    }
    const files = VITEST_FILES.exec(line);
    if (files) {
      current.vitest ??= {};
      current.vitest.files = { ...vitestCounts(files[1]), total: Number(files[2]) };
      continue;
    }
    const tests = VITEST_TESTS.exec(line);
    if (tests) {
      current.vitest ??= {};
      current.vitest.tests = { ...vitestCounts(tests[1]), total: Number(tests[2]) };
    }
  }
  const totalTime = times.find((time) => time.name === 'total');
  for (const step of steps) {
    // A background step is timed as "<name> (background)".
    const time =
      times.find((entry) => entry.name === step.name) ??
      times.find((entry) => entry.name === `${step.name} (background)`);
    step.seconds = time ? time.seconds : null;
    if (time?.failed) step.failed = true;
  }
  const passedLine = lines.some((line) => line.startsWith(PASSED_LINE));
  const failedTests = steps.some((step) => (step.rust?.failed ?? 0) > 0 || (step.vitest?.tests?.failed ?? 0) > 0
    || (step.vitest?.files?.failed ?? 0) > 0);
  // Without the closing line the gate stopped somewhere: the last step it started is where.
  if (!passedLine && steps.length > 0 && !steps.some((step) => step.failed)) steps[steps.length - 1].failed = true;
  const missingSteps = REQUIRED_STEPS.filter((required) => !steps.some((step) => step.name.startsWith(required)));
  const status = passedLine && !failedTests ? 'passed' : totalTime ? 'failed' : 'incomplete';
  return {
    steps,
    totalSeconds: totalTime ? totalTime.seconds : null,
    passed: status === 'passed',
    status,
    missingSteps,
  };
}

/** Why a summary is no release evidence, or [] when it is. */
export function gateSummaryProblems(gate, { requireAllSteps = true } = {}) {
  const problems = [];
  if (gate.status === 'incomplete') problems.push('de gate-log is onvolledig (geen slotregel en geen tijdentabel)');
  else if (!gate.passed) problems.push('de gate is niet geslaagd');
  const failed = gate.steps.filter((step) => step.failed).map((step) => step.name);
  if (failed.length) problems.push(`mislukte stap: ${failed.join(', ')}`);
  if (requireAllSteps && gate.missingSteps.length) problems.push(`ontbrekende stappen: ${gate.missingSteps.join(', ')}`);
  return problems;
}

/** Cases per suite and verdict from reference_gate's reference-report.json. */
export function summariseReferenceReport(report) {
  const suites = new Map();
  for (const item of report.cases ?? []) {
    const suite = item.suite ?? '(los)';
    const entry = suites.get(suite) ?? { suite, total: 0, compared: 0, passed: 0, pending: 0, other: 0 };
    entry.total += 1;
    const status = item.comparison?.status;
    if (status === 'compared_pass') {
      entry.compared += 1;
      entry.passed += 1;
    } else if (status === 'compared_fail') {
      entry.compared += 1;
    } else if (status === 'pending_expectation') {
      entry.pending += 1;
    } else {
      entry.other += 1;
    }
    suites.set(suite, entry);
  }
  return {
    verdict: report.comparisonVerdict ?? null,
    comparedCases: report.comparedCases ?? null,
    pendingExpectationCases: report.pendingExpectationCases ?? null,
    totalCases: (report.cases ?? []).length,
    kernelVersion: report.kernelVersion ?? null,
    commit: report.commit ?? null,
    suites: [...suites.values()],
  };
}

/** Keep the separately planned batch distinct from the bundled internal suites. */
export function summarisePlannedReferenceReport(report) {
  return {
    verdict: report.comparisonVerdict ?? null,
    comparedCases: report.comparedCases ?? null,
    pendingExpectationCases: report.pendingExpectationCases ?? null,
    totalCases: (report.cases ?? []).length,
    numericComparisonPassed: report.numericComparisonPassed === true,
    plannedCoveragePassed: report.plannedCoveragePassed === true,
    coveragePlanFingerprint: report.coveragePlanFingerprint ?? null,
    commit: report.commit ?? null,
    referenceVerified: report.referenceVerified === true,
    attestStatus: report.attestStatus ?? null,
  };
}

function rustText(rust) {
  const extra = [rust.failed ? `${rust.failed} mislukt` : '', rust.ignored ? `${rust.ignored} genegeerd` : '']
    .filter(Boolean)
    .join(', ');
  return `Rust: ${rust.passed} geslaagd${extra ? ` (${extra})` : ''}, ${rust.binaries} ${rust.binaries === 1 ? 'testprogramma' : 'testprogramma’s'}`;
}

function vitestText(vitest) {
  const parts = [];
  if (vitest.tests) {
    const skipped = vitest.tests.skipped ? `, ${vitest.tests.skipped} overgeslagen` : '';
    const failed = vitest.tests.failed ? `, ${vitest.tests.failed} mislukt` : '';
    parts.push(`Vitest: ${vitest.tests.passed} van ${vitest.tests.total} tests geslaagd${failed}${skipped}`);
  }
  if (vitest.files) parts.push(`${vitest.files.passed} van ${vitest.files.total} bestanden`);
  return parts.join(', ');
}

/** The summary as Markdown, for the test report (Dutch, like the template). */
export function renderGateSummaryMarkdown(summary) {
  if (!summary) return 'Gate overgeslagen (proefvrijgave): geen uitvoer om samen te vatten.';
  const { gate, reference, plannedReference } = summary;
  const lines = [
    `Uit \`gate.log\` en \`evidence/reference/reference-report.json\` van deze vrijgave. ` +
      `Gate ${gate.passed ? 'geslaagd' : gate.status === 'incomplete' ? 'onvolledig (log afgebroken)' : 'niet geslaagd'}` +
      (gate.totalSeconds !== null ? `, totale duur ${gate.totalSeconds} s.` : '.'),
    '',
    '| Stap | Resultaat | Duur (s) |',
    '|---|---|---|',
  ];
  for (const step of gate.steps) {
    const counts = [step.rust ? rustText(step.rust) : '', step.vitest ? vitestText(step.vitest) : '']
      .filter(Boolean)
      .join('; ');
    const result = step.failed && counts ? `niet geslaagd; ${counts}` : counts;
    const fallback = step.failed
      ? (gate.status === 'incomplete' ? 'onvolledig (gate afgebroken)' : 'niet geslaagd')
      : /\bskipped\b/.test(step.name) ? 'overgeslagen' : 'geslaagd';
    lines.push(`| ${step.name} | ${result || fallback} | ${step.seconds ?? ''} |`);
  }
  if (reference) {
    lines.push(
      '',
      `Referentiesuites: oordeel \`${reference.verdict}\`, ${reference.comparedCases} van ` +
        `${reference.totalCases} gevallen vergeleken, ${reference.pendingExpectationCases} zonder verwachting.`,
      '',
      '| Suite | Gevallen | Vergeleken | Binnen de band | Zonder verwachting | Overig |',
      '|---|---|---|---|---|---|',
    );
    for (const suite of reference.suites) {
      lines.push(
        `| ${suite.suite} | ${suite.total} | ${suite.compared} | ${suite.passed} | ${suite.pending} | ${suite.other} |`,
      );
    }
  }
  if (plannedReference) {
    lines.push(
      '',
      `Afzonderlijke geplande referentiebatch: ${plannedReference.comparedCases} van ${plannedReference.totalCases} ` +
        `gevallen vergeleken; numerieke vergelijking ${plannedReference.numericComparisonPassed ? 'geslaagd' : 'niet geslaagd'}; ` +
        `plandekking ${plannedReference.plannedCoveragePassed ? 'geslaagd' : 'niet geslaagd'}.`,
      `Plan-SHA-256: \`${plannedReference.coveragePlanFingerprint ?? 'ontbreekt'}\`; ` +
        `commit: \`${plannedReference.commit ?? 'ontbreekt'}\`. ` +
        `Onafhankelijke referentieverificatie: ${plannedReference.referenceVerified ? 'ja' : 'nee'}; ` +
        `atteststatus: \`${plannedReference.attestStatus ?? 'onbekend'}\`.`,
    );
  } else {
    lines.push('', 'Afzonderlijke geplande referentiebatch: niet uitgevoerd of rapport ontbreekt.');
  }
  return lines.join('\n');
}

/** One English sentence for the GitHub release body. */
export function renderGateSummaryEnglish(summary) {
  if (!summary) return '';
  const { gate, reference, plannedReference } = summary;
  let rust = 0;
  let ui = null;
  for (const step of gate.steps) {
    if (step.rust) rust += step.rust.passed;
    if (step.name === 'UI tests' && step.vitest?.tests) ui = step.vitest;
  }
  const parts = [`${rust} Rust tests passed across all gate steps (the kernel counted again on Rust 1.77.2)`];
  if (ui) parts.push(`${ui.tests.passed} UI tests in ${ui.files.passed} files`);
  if (reference) {
    parts.push(
      `reference suites: ${reference.comparedCases} of ${reference.totalCases} cases compared ` +
        `(verdict \`${reference.verdict}\`), ${reference.pendingExpectationCases} without expected values`,
    );
  }
  parts.push(plannedReference
    ? `separate planned reference batch: ${plannedReference.comparedCases} of ${plannedReference.totalCases} cases compared, ` +
      `numeric comparison ${plannedReference.numericComparisonPassed ? 'passed' : 'failed'}, ` +
      `plan coverage ${plannedReference.plannedCoveragePassed ? 'passed' : 'failed'} ` +
      `(independent verification ${plannedReference.referenceVerified ? 'recorded' : 'not recorded'})`
    : 'separate planned reference batch: not run or report missing');
  const time = gate.totalSeconds !== null ? ` Gate time ${Math.round(gate.totalSeconds / 60)} min.` : '';
  return `${parts.join('; ')}.${time}`;
}

function main(argv) {
  const options = {};
  const flags = new Set(['--require-pass', '--allow-missing-steps']);
  for (let i = 0; i < argv.length; i += 1) {
    if (flags.has(argv[i])) options[argv[i].slice(2)] = true;
    else options[argv[i].replace(/^--/, '')] = argv[++i];
  }
  if (!options.log || !options.out) {
    console.error('--log en --out zijn verplicht');
    return 2;
  }
  const gate = summariseGateLog(readFileSync(options.log, 'utf8'));
  const reference =
    options.reference && existsSync(options.reference)
      ? summariseReferenceReport(JSON.parse(readFileSync(options.reference, 'utf8')))
      : null;
  const plannedReference =
    options['planned-reference'] && existsSync(options['planned-reference'])
      ? summarisePlannedReferenceReport(JSON.parse(readFileSync(options['planned-reference'], 'utf8')))
      : null;
  writeFileSync(options.out, `${JSON.stringify({ gate, reference, plannedReference }, null, 2)}\n`);
  // --require-pass: the release stops on a summary that is no evidence (BRL 9501 §6.3).
  // --allow-missing-steps: a dry run may use the skip switches.
  if (options['require-pass'] !== undefined) {
    const problems = gateSummaryProblems(gate, { requireAllSteps: options['allow-missing-steps'] === undefined });
    if (gate.steps.some((step) => step.name.startsWith('reference suites')) && !reference) {
      problems.push('rapport van de interne referentiesuites ontbreekt');
    }
    if (reference && reference.verdict !== 'passed') {
      problems.push('interne referentiesuites zijn niet geslaagd');
    }
    if (gate.steps.some((step) => step.name === 'planned reference comparisons') && !plannedReference) {
      problems.push('rapport van de uitgevoerde geplande referentiebatch ontbreekt');
    }
    if (plannedReference && (!plannedReference.numericComparisonPassed || !plannedReference.plannedCoveragePassed)) {
      problems.push('geplande referentiebatch is niet numeriek en qua plandekking geslaagd');
    }
    if (plannedReference && plannedReference.verdict !== 'passed') {
      problems.push('oordeel van de geplande referentiebatch is niet geslaagd');
    }
    if (plannedReference && (!reference?.commit || !plannedReference.commit || reference.commit !== plannedReference.commit)) {
      problems.push('broncommit van de geplande referentiebatch komt niet overeen met de interne gate');
    }
    if (plannedReference && !/^sha256:[0-9a-f]{64}$/.test(plannedReference.coveragePlanFingerprint ?? '')) {
      problems.push('geldige planvingerafdruk van de geplande referentiebatch ontbreekt');
    }
    if (problems.length) {
      console.error(`Gate-samenvatting is geen bewijs voor een vrijgave: ${problems.join('; ')}.`);
      return 1;
    }
  }
  return 0;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  process.exit(main(process.argv.slice(2)));
}
