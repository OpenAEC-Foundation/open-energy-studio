import { readFileSync, writeFileSync } from 'fs';
import { fileURLToPath } from 'url';

/**
 * Every file that carries the program version. The desktop package takes its
 * version from tauri.conf.json and Cargo.toml, the reports and the
 * leveringsdocument from package.json; the gate builds with `--locked`, so the
 * lockfile entry of the app crate must follow too.
 */
export const VERSION_FILES = [
  'package.json',
  'src-tauri/tauri.conf.json',
  'src-tauri/Cargo.toml',
  'src-tauri/Cargo.lock',
];

/** Returns the files with the program version set to `version`. */
export function setVersion(files, version) {
  const pkg = JSON.parse(files['package.json']);
  pkg.version = version;
  const tauriConf = JSON.parse(files['src-tauri/tauri.conf.json']);
  tauriConf.version = version;
  return {
    'package.json': JSON.stringify(pkg, null, 2) + '\n',
    'src-tauri/tauri.conf.json': JSON.stringify(tauriConf, null, 2) + '\n',
    'src-tauri/Cargo.toml': files['src-tauri/Cargo.toml'].replace(/^version = ".*"/m, `version = "${version}"`),
    'src-tauri/Cargo.lock': files['src-tauri/Cargo.lock'].replace(
      /(name = "open-energy-studio"\nversion = )"[^"]*"/,
      `$1"${version}"`,
    ),
  };
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const version = process.argv[2];
  if (!version || !/^\d+\.\d+\.\d+$/.test(version)) {
    console.error('Usage: node scripts/bump-version.js <version>');
    console.error('Example: node scripts/bump-version.js 0.3.0');
    process.exit(1);
  }
  const files = Object.fromEntries(VERSION_FILES.map((path) => [path, readFileSync(path, 'utf8')]));
  for (const [path, text] of Object.entries(setVersion(files, version))) {
    writeFileSync(path, text);
    console.log(`${path} -> ${version}`);
  }
  console.log(`\nVersion bumped to ${version}`);
}
