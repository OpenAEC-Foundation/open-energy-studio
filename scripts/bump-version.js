import { execFileSync } from 'child_process';
import { readFileSync, writeFileSync } from 'fs';
import { fileURLToPath } from 'url';

/**
 * Every file that carries the program version. The desktop package takes its
 * version from tauri.conf.json and Cargo.toml, the reports and the
 * leveringsdocument from package.json; the gate builds with `--locked`, so the
 * lockfile entries of the app crate and the npm package must follow too.
 */
export const VERSION_FILES = [
  'package.json',
  'package-lock.json',
  'src-tauri/tauri.conf.json',
  'src-tauri/Cargo.toml',
  'src-tauri/Cargo.lock',
];

/** The OpenAEC program version JJJJ.W.N (issue #10): ISO year, ISO week without a leading zero, sequence. */
const WEEK_VERSION = /^(\d{4})\.([1-9]\d?)\.(\d+)$/;

/** ISO 8601 year and week of a date (weeks start on Monday; week 1 holds the year's first Thursday). */
export function isoWeek(date) {
  const day = new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate()));
  // The Thursday of this week decides the year.
  day.setUTCDate(day.getUTCDate() + 3 - ((day.getUTCDay() + 6) % 7));
  const year = day.getUTCFullYear();
  const firstThursday = new Date(Date.UTC(year, 0, 4));
  firstThursday.setUTCDate(firstThursday.getUTCDate() + 3 - ((firstThursday.getUTCDay() + 6) % 7));
  return { year, week: 1 + Math.round((day - firstThursday) / (7 * 24 * 3600 * 1000)) };
}

/**
 * The next program version for `date`: JJJJ.W.N with N one above the highest
 * release tag of that week (`vJJJJ.W.N` or `oes-vJJJJ.W.N-kernel-v…`), else 0.
 */
export function weekVersion(date, tags) {
  const { year, week } = isoWeek(date);
  const prefix = `${year}.${week}.`;
  const used = tags
    .map((tag) => /^(?:oes-)?v(\d{4}\.\d+\.\d+)(?:-kernel-v.*)?$/.exec(tag.trim())?.[1])
    .filter((version) => version?.startsWith(prefix))
    .map((version) => Number(version.slice(prefix.length)));
  return `${prefix}${used.length > 0 ? Math.max(...used) + 1 : 0}`;
}

/**
 * The Windows MSI version of a program version. An MSI major version stops at
 * 255, so JJJJ.W.N becomes JJ.W.N; other versions are valid as they are.
 */
export function msiVersion(version) {
  const match = WEEK_VERSION.exec(version);
  return match ? `${Number(match[1]) % 100}.${match[2]}.${match[3]}` : version;
}

/** Returns the files with the program version set to `version` (LF line endings). */
export function setVersion(files, version) {
  const pkg = JSON.parse(files['package.json']);
  pkg.version = version;
  const tauriConf = JSON.parse(files['src-tauri/tauri.conf.json']);
  tauriConf.version = version;
  const msi = msiVersion(version);
  if (msi !== version) {
    tauriConf.bundle = tauriConf.bundle ?? {};
    tauriConf.bundle.windows = tauriConf.bundle.windows ?? {};
    tauriConf.bundle.windows.wix = { ...(tauriConf.bundle.windows.wix ?? {}), version: msi };
  } else if (tauriConf.bundle?.windows?.wix?.version) {
    delete tauriConf.bundle.windows.wix.version;
  }
  const out = {
    'package.json': JSON.stringify(pkg, null, 2) + '\n',
    'src-tauri/tauri.conf.json': JSON.stringify(tauriConf, null, 2) + '\n',
    'src-tauri/Cargo.toml': files['src-tauri/Cargo.toml'].replace(/^version = ".*"/m, `version = "${version}"`),
    'src-tauri/Cargo.lock': files['src-tauri/Cargo.lock'].replace(
      /(name = "open-energy-studio"\nversion = )"[^"]*"/,
      `$1"${version}"`,
    ),
  };
  if (files['package-lock.json'] !== undefined) {
    const lock = JSON.parse(files['package-lock.json']);
    lock.version = version;
    if (lock.packages?.['']) lock.packages[''].version = version;
    out['package-lock.json'] = JSON.stringify(lock, null, 2) + '\n';
  }
  return out;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const argument = process.argv[2];
  let version = argument;
  if (!argument || argument === '--week') {
    const tags = execFileSync('git', ['tag', '--list'], { encoding: 'utf8' }).split('\n').filter(Boolean);
    version = weekVersion(new Date(), tags);
  }
  if (!/^\d+\.\d+\.\d+$/.test(version)) {
    console.error('Gebruik: node scripts/bump-version.js [--week | <versie>]');
    console.error('Zonder argument (of met --week): jaar.week.volgnummer van vandaag, na de bestaande tags.');
    console.error('Voorbeeld: node scripts/bump-version.js 2026.44.0');
    process.exit(1);
  }
  // A Windows checkout has CRLF line endings: edit with LF, write back as found.
  const crlf = {};
  const files = Object.fromEntries(VERSION_FILES.map((path) => {
    const text = readFileSync(path, 'utf8');
    crlf[path] = text.includes('\r\n');
    return [path, text.replace(/\r\n/g, '\n')];
  }));
  for (const [path, text] of Object.entries(setVersion(files, version))) {
    writeFileSync(path, crlf[path] ? text.replace(/\n/g, '\r\n') : text);
    console.log(`${path} -> ${version}`);
  }
  const msi = msiVersion(version);
  console.log(`\nProgrammaversie ${version}${msi !== version ? ` (MSI ${msi})` : ''}; tag de release als v${version}.`);
}
