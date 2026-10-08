#!/usr/bin/env node
// The NTA 8800 user manual (docs/handleiding-nta8800/) as part of every
// release (BRL 9501 §4.4, task B7): one Markdown parser shared by the in-app
// viewer and the release, a single-file HTML export and the version check.
//
//   node scripts/nta-manual.mjs check
//       Fails when the manual's stamp (`<!-- handleiding: rekenkern X.Y.Z -->`
//       in index.md) differs from KERNEL_VERSION, so a new kernel version
//       cannot be released with a manual that was not reviewed for it.
//   node scripts/nta-manual.mjs html --out <file.html> [--program <version>]
//       Writes the whole manual as one self-contained HTML file (images
//       embedded) with the program version and KERNEL_VERSION on the cover.

import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, extname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { kernelVersion } from './nta-kernel-version.mjs';
import { MANUAL_DIR, buildManualHtml, chapterId, checkManualStamp, orderChapters } from './nta-manual-markdown.mjs';

export * from './nta-manual-markdown.mjs';

const MIME = { '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.svg': 'image/svg+xml', '.gif': 'image/gif', '.webp': 'image/webp' };

/** Reads the manual folder: chapters in order and the images as data URIs. */
export function readManual(root) {
  const dir = join(root, MANUAL_DIR);
  const names = readdirSync(dir);
  const chapters = orderChapters(names).map((name) => ({ id: chapterId(name), text: readFileSync(join(dir, name), 'utf8') }));
  const images = {};
  for (const name of readdirSync(join(dir, 'img'))) {
    const mime = MIME[extname(name).toLowerCase()];
    if (mime) images[`img/${name}`] = `data:${mime};base64,${readFileSync(join(dir, 'img', name)).toString('base64')}`;
  }
  return { chapters, images };
}

function main(argv) {
  const root = join(dirname(fileURLToPath(import.meta.url)), '..');
  const [command, ...rest] = argv;
  const index = readFileSync(join(root, MANUAL_DIR, 'index.md'), 'utf8');
  if (command === 'check') {
    const result = checkManualStamp(index, kernelVersion(root));
    (result.ok ? console.log : console.error)(result.message);
    return result.ok ? 0 : 1;
  }
  if (command === 'html') {
    const options = {};
    for (let i = 0; i < rest.length; i += 2) {
      if (!['--out', '--program', '--date'].includes(rest[i]) || rest[i + 1] === undefined) throw new Error(`Onbekende of lege optie ${rest[i]}`);
      options[rest[i].slice(2)] = rest[i + 1];
    }
    if (!options.out) throw new Error('--out <bestand.html> is verplicht');
    const programVersion = options.program ?? JSON.parse(readFileSync(join(root, 'package.json'), 'utf8')).version;
    const html = buildManualHtml({ ...readManual(root), programVersion, kernelVersion: kernelVersion(root), generatedOn: options.date });
    writeFileSync(options.out, html);
    console.log(`Handleiding geschreven naar ${options.out}`);
    return 0;
  }
  console.error('Gebruik: nta-manual.mjs check | html --out <bestand.html> [--program <versie>] [--date <jjjj-mm-dd>]');
  return 2;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  process.exitCode = main(process.argv.slice(2));
}
