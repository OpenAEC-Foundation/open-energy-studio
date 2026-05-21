/**
 * generate-types — produce TypeScript types for ProjectV2 and ProjectResult
 * from the Rust JSON Schemas exposed by `schema-export`.
 *
 * Pipeline:
 *   1. `cargo run --bin schema-export -- --out schemas`
 *   2. `npx json2ts -i schemas/<name>.schema.json -o src/types/<name>.ts`
 *
 * Run via `npm run generate-types`. CI must run it before `npm run build`.
 *
 * The script tolerates a missing Rust toolchain (logs a warning, keeps the
 * existing placeholder types) so frontend-only developers can still build the
 * app. Once Rust is available, re-run to get the real types.
 */

import { spawn } from 'node:child_process';
import { mkdirSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO = join(HERE, '..');
const SCHEMAS = join(REPO, 'schemas');
const TYPES = join(REPO, 'src', 'types');

const TARGETS = [
  { schema: 'project.schema.json', out: 'project.generated.ts', name: 'ProjectV2' },
  { schema: 'result.schema.json', out: 'result.generated.ts', name: 'ProjectResult' },
];

function run(cmd, args, opts = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, { stdio: 'inherit', shell: false, ...opts });
    child.on('error', reject);
    child.on('exit', (code) => {
      if (code === 0) resolve();
      else reject(new Error(`${cmd} ${args.join(' ')} exited ${code}`));
    });
  });
}

async function main() {
  mkdirSync(SCHEMAS, { recursive: true });
  mkdirSync(TYPES, { recursive: true });

  // Step 1 — export schemas from Rust.
  try {
    await run('cargo', ['run', '--bin', 'schema-export', '--quiet', '--', '--out', SCHEMAS], {
      cwd: REPO,
    });
  } catch (err) {
    console.warn(
      '[generate-types] cargo run failed; keeping existing placeholder types.\n' +
        `  Reason: ${err.message}\n` +
        '  Install Rust + MSVC Build Tools to enable real type generation.',
    );
    return;
  }

  // Step 2 — convert each schema into TypeScript.
  for (const t of TARGETS) {
    const schemaPath = join(SCHEMAS, t.schema);
    if (!existsSync(schemaPath)) {
      console.warn(`[generate-types] missing ${schemaPath}, skipping`);
      continue;
    }
    const outPath = join(TYPES, t.out);
    await run(
      'npx',
      ['--yes', 'json-schema-to-typescript', '-i', schemaPath, '-o', outPath],
      { cwd: REPO },
    );
    console.log(`[generate-types] ${schemaPath} -> ${outPath}`);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
