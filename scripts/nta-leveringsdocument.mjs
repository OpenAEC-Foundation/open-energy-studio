#!/usr/bin/env node
// Leveringsdocument of a release (BRL 9501 §6.1): program name, program
// version, KERNEL_VERSION, TARGET_NORM_VERSION, attest number and
// identification code (empty until the attest) and the SHA-256 of every
// package, filled into docs/templates/nta8800-leveringsdocument.md.
//
//   node scripts/nta-leveringsdocument.mjs --out <file.md> [--commit <sha>]
//        [--tag <tag>] [--date <yyyy-mm-dd>] [--manual <file>]... [package ...]
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { dutchDate, kernelVersion } from './nta-kernel-version.mjs';

const NOT_YET = 'nog niet toegekend';

export function readIdentity(root) {
  const attest = JSON.parse(readFileSync(join(root, 'src/core/nta/attest.json'), 'utf8'));
  const programVersion = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8')).version;
  const lib = readFileSync(join(root, 'crates/nta8800-core/src/lib.rs'), 'utf8');
  const norm = /pub const TARGET_NORM_VERSION: &str = "([^"]+)";/.exec(lib);
  if (!norm) throw new Error('TARGET_NORM_VERSION niet gevonden in crates/nta8800-core/src/lib.rs');
  return {
    softwareName: attest.softwareName,
    attestNumber: attest.attestNumber ?? '',
    identificationCode: attest.identificationCode ?? '',
    attestingBody: attest.attestingBody ?? '',
    programVersion,
    kernelVersion: kernelVersion(root),
    targetNormVersion: norm[1],
  };
}

export function sha256File(path) {
  return createHash('sha256').update(readFileSync(path)).digest('hex');
}

/** Fills the template; every {{placeholder}} must be known. */
export function renderLeveringsdocument(template, values) {
  const packages = values.packages.length
    ? values.packages.map((item) => `| \`${item.name}\` | \`${item.sha256}\` |`).join('\n')
    : '| (geen pakketten gebouwd) | – |';
  const manual = values.manual?.length
    ? values.manual.map((item) => `| \`${item.name}\` | \`${item.sha256}\` |`).join('\n')
    : '| (geen handleiding meegeleverd) | – |';
  const attested = Boolean(values.attestNumber);
  const fields = {
    ...values,
    attestNumber: values.attestNumber || NOT_YET,
    identificationCode: values.identificationCode || NOT_YET,
    attestingBody: values.attestingBody || NOT_YET,
    commit: values.commit || 'onbekend',
    tag: values.tag || 'geen',
    packages,
    manual,
    attestStatement: attested
      ? `Dit programma is geattesteerd volgens BRL 9501 onder attestnummer ${values.attestNumber}.`
      : 'Dit programma is niet geattesteerd volgens BRL 9501. Het attestnummer en de identificatiecode worden ingevuld zodra het attest is verleend.',
  };
  return template.replace(/\{\{(\w+)\}\}/g, (match, key) => {
    if (!(key in fields)) throw new Error(`Onbekend veld in het sjabloon: ${match}`);
    return String(fields[key]);
  });
}

function main(argv) {
  const root = join(dirname(fileURLToPath(import.meta.url)), '..');
  const options = { packages: [], manual: [] };
  for (let i = 0; i < argv.length; i += 1) {
    const flag = argv[i];
    if (flag === '--manual') {
      const value = argv[i + 1];
      if (value === undefined) throw new Error('--manual vraagt een bestand');
      options.manual.push(value);
      i += 1;
    } else if (['--out', '--commit', '--tag', '--date'].includes(flag)) {
      const value = argv[i + 1];
      if (value === undefined) throw new Error(`${flag} vraagt een waarde`);
      options[flag.slice(2)] = value;
      i += 1;
    } else {
      options.packages.push(flag);
    }
  }
  if (!options.out) throw new Error('--out <bestand> is verplicht');
  const date = options.date ?? new Date().toISOString().slice(0, 10);
  const template = readFileSync(join(root, 'docs/templates/nta8800-leveringsdocument.md'), 'utf8');
  const text = renderLeveringsdocument(template, {
    ...readIdentity(root),
    commit: options.commit,
    tag: options.tag,
    releaseDate: dutchDate(date),
    packages: options.packages.map((path) => ({ name: basename(path), sha256: sha256File(path) })),
    manual: options.manual.map((path) => ({ name: basename(path), sha256: sha256File(path) })),
  });
  writeFileSync(options.out, text);
  console.log(`Leveringsdocument geschreven: ${options.out}`);
  return 0;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  try {
    process.exit(main(process.argv.slice(2)));
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exit(2);
  }
}
