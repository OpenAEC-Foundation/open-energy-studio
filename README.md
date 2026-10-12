# Open Energy Studio

Open-source energy-performance software for buildings in the Netherlands, calculated according to **NTA 8800**, for both dwellings and utility buildings.

![License](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)

The goal is a complete, verifiable NTA 8800 calculation tool that can be attested under **BRL 9501**.

> **Status:** not yet attested. What is done and what is still open, requirement by requirement, is in [docs/nta8800-brl9501-gereedheid.md](docs/nta8800-brl9501-gereedheid.md).

## Versions

| Program | Calculation kernel (`KERNEL_VERSION`) | Manual | Release tag |
|---|---|---|---|
| **0.2.0** | **0.3.0** | stamped for kernel 0.3.0 | `oes-v0.2.0-kernel-v0.3.0` |

Program 0.2.0 · Kernel 0.3.0 · Manual stamped for kernel 0.3.0.

The branch `nta8800-kernel` is ahead of this release: it carries kernel **0.4.0** (unreleased, manual stamped for 0.4.0). Its changes are under "Onuitgebracht" in [docs/nta8800-releasenotes.md](docs/nta8800-releasenotes.md).

**Supported norm editions** (`normVersion` in the input → label stamped on results):

| `normVersion` | Edition | Status |
|---|---|---|
| `2025+C1` (default) | NTA 8800:2025+C1:2026 | registrable |
| `2024` | NTA 8800:2024 met INT-V1:2024 | legacy, not registrable |
| `2023` | NTA 8800:2023 | legacy, not registrable |
| `2022` | NTA 8800:2022 | legacy, not registrable |
| `2020+A1` | NTA 8800:2020+A1:2020 | legacy, not registrable |

**Toolchain:**

| Tool | Version |
|---|---|
| Rust | 1.77.2 minimum (MSRV) for the kernel; 1.88 for the desktop app and the service crate (API/MCP) |
| Node.js | 20 or later (CI, development and release builds use 22) |
| Tauri | 2 |

## Features

| Area | What it does |
|---|---|
| **Calculation kernel** | The NTA 8800 calculation (heat need, BENG 1/2/3, TOjuli, primary energy, the energy label and label data) is a Rust crate, `crates/nta8800-core`. Results carry the kernel version and an input fingerprint. |
| **Norm editions** | NTA 8800:2025+C1 is the default and the only registrable edition. 2024, 2023, 2022 and 2020+A1 can be calculated as *legacy* editions, which are not registrable. Switching the edition first shows what the new edition refuses or requires, traced to its switch point. See [docs/nta8800-normversies.md](docs/nta8800-normversies.md). |
| **Basisopname (survey)** | Survey of existing dwellings (ISSO 82.1) and utility buildings (ISSO 75.1). It derives the project input from the survey answers and can take that input over into the project model, including floors to the ground value by value. |
| **Maatwerkadvies** | Tailored advice following ISSO 82.2/75.2: user profiles, fitting to measured consumption, measure packages and NPV. |
| **Herlabelen (relabel)** | Compares an original and a changed project per BRL 9500 annex 6a/6b. |
| **Registration and dossier** | Registration data, label statements, evidence (*Bron & bewijs*) linked to inputs, and a dossier ZIP with a manifest and SHA-256 per file (BRL 9500 annex 3). |
| **EP-Online overview** | The data overview in the EP-Online export structure, with XSD field limits. |
| **Reports** | An energy-performance report at summary, standard and detailed level, plus a maatwerkadvies report. |
| **HTTP API and MCP server** | Every kernel operation (46) is available over a local HTTP API (OpenAPI 3.1) and as MCP tools. See [docs/nta8800-api.md](docs/nta8800-api.md) and [docs/nta8800-mcp.md](docs/nta8800-mcp.md). |
| **Import / export** | Uniec 3 and VABI exchange. Every import records the file name, a SHA-256 and a list of issues. |
| **Desktop and web** | A Tauri 2 desktop app (Windows, Linux, macOS) and a browser build of the same React UI. |
| **Manual** | Dutch user manual, available in the app and shipped with every release: [docs/handleiding-nta8800/](docs/handleiding-nta8800/index.md). |

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│ React + TypeScript UI (src/)                                 │
│  workflow shell · NTA forms · survey · reports · dossier     │
├───────────────────────────────┬──────────────────────────────┤
│ Tauri 2 desktop (src-tauri/)  │ Browser build                │
│  async kernel commands,       │  talks to the HTTP API       │
│  atomic save, file scope      │                              │
├───────────────────────────────┴──────────────────────────────┤
│ nta8800-service (crates/nta8800-service)                     │
│  operation registry · HTTP API (api) · MCP server (mcp)      │
│  reference_gate (reference suites)                           │
├──────────────────────────────────────────────────────────────┤
│ nta8800-core (crates/nta8800-core) — the calculation kernel  │
│  norm editions (norm_versions/) · survey (opname/) ·         │
│  maatwerkadvies · relabel · registration · label data        │
└──────────────────────────────────────────────────────────────┘
```

The kernel has no I/O and is deterministic. The desktop commands and the HTTP/MCP adapters all call the same kernel functions, through the same edition handling.

## Getting started

### Prerequisites

- **Node.js** 20 or later (CI and release builds use 22), and npm.
- **Rust**:
  - the kernel (`crates/nta8800-core`) builds with Rust 1.77.2 or later (MSRV);
  - the desktop app (`src-tauri/`) and the service crate (API/MCP) need Rust 1.88 or later.
- **Tauri** 2 (installed through npm as `@tauri-apps/cli`).
- **Linux, for the desktop app:**
  ```bash
  sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
  ```

### Development

```bash
git clone https://github.com/Erika98-be/open-energy-studio.git
cd open-energy-studio
npm install

npm run dev          # web UI on http://localhost:3006
npm run tauri:dev    # desktop app
```

The browser build calculates through the local HTTP API (port 3007 by default). `npm run dev` proxies `/api/*` to it; a static `dist/` build needs the same reverse proxy (`/api` → `http://127.0.0.1:3007`, with `/api` stripped). Start the API with:

```bash
cargo run --release --manifest-path crates/nta8800-service/Cargo.toml --bin api
cargo build --release --manifest-path crates/nta8800-service/Cargo.toml --bin mcp   # MCP server (stdio)
```

The API binds to `127.0.0.1` by default and limits concurrent calculations, queue time and request size. All flags and environment variables are in [docs/nta8800-api.md](docs/nta8800-api.md).

### Building

```bash
npm run build                  # type check + web bundle (dist/)
npm run tauri:build            # desktop installers for the host OS (NSIS on Windows; deb, rpm, AppImage on Linux; dmg on macOS)
```

## Verification

Every change to the calculation goes through the gate:

```bash
scripts/verify-nta.sh
```

The gate runs:

- **Version checks:** the kernel-version and release-notes check, and the manual-stamp check.
- **Kernel tests:** the unit tests, the edition tests and the public-comparison tests.
- **Option-coverage sweep:** every enum variant and optional field of the kernel input, under every edition.
- **Numeric fuzz test:** boundary and random values, plus monotonicity checks.
- **Large-project tests.**
- **Reference suites** (`reference_gate`):
  - public calculation reports A–O, as a regression on this kernel's recorded results; the differences from the published values are explained post by post in [docs/nta8800-vergelijking-openbare-rapporten.md](docs/nta8800-vergelijking-openbare-rapporten.md);
  - the RVO voorbeeldwoningen;
  - 260 of the 266 ISSO 54 v2.0 sub-tests, encoded as *pending*: they must calculate, but cannot be judged until the test set's expected results are available. See [docs/nta8800-isso54-voorbereiding.md](docs/nta8800-isso54-voorbereiding.md).
- **Service, API, MCP and desktop-command tests.**
- **Front-end tests (Vitest):** end-to-end workflow tests, form-coverage and form-guard tests (form states run through the kernel), accessibility and i18n checks.

The kernel is also tested on its minimum supported Rust version:

```bash
cd crates/nta8800-core && cargo +1.77.2 test --locked
```

The gate prints the current test counts. The test protocol (what is tested, how, and the acceptance criteria) is in [docs/nta8800-testprotocol.md](docs/nta8800-testprotocol.md).

## Versioning and releases

- **Kernel version.** `KERNEL_VERSION` is the version in `crates/nta8800-core/Cargo.toml`. Every result is stamped with it. A relabel against a different kernel version is refused.
- **Release notes.** [docs/nta8800-releasenotes.md](docs/nta8800-releasenotes.md) is grouped by kernel version, with new entries under `## Onuitgebracht`. An entry that does not change results ends in `(geen rekenwijziging)`. The gate fails when a result-changing entry has no version bump.
- **Release.** `scripts/release-nta.sh` runs the gate and builds the Linux package (`.deb`). It then:
  - exports the manual (HTML, and PDF when a converter is installed);
  - commits the released notes (only for a new kernel version);
  - writes the leveringsdocument, a pre-filled test report, a manifest and `SHA256SUMS`;
  - creates an annotated local tag `oes-v<program>-kernel-v<kernel>`.
  
  It never pushes. Use `--dry-run` to try it without committing or tagging.

The policy is in [docs/nta8800-versiebeheer.md](docs/nta8800-versiebeheer.md).

## Documentation map

| Topic | Where |
|---|---|
| User manual (Dutch) | [docs/handleiding-nta8800/](docs/handleiding-nta8800/index.md) |
| Wiki (Dutch, summary per topic) | [GitHub wiki](https://github.com/Erika98-be/open-energy-studio/wiki) (source in [wiki/](wiki/Home.md)) |
| Norm editions and switch points | [docs/nta8800-normversies.md](docs/nta8800-normversies.md) |
| Test protocol | [docs/nta8800-testprotocol.md](docs/nta8800-testprotocol.md) |
| BRL 9501 readiness | [docs/nta8800-brl9501-gereedheid.md](docs/nta8800-brl9501-gereedheid.md) |
| Program description (for the certifying body) | [docs/nta8800-programmabeschrijving.md](docs/nta8800-programmabeschrijving.md) |
| Public-report comparisons | [docs/nta8800-vergelijking-openbare-rapporten.md](docs/nta8800-vergelijking-openbare-rapporten.md) |
| Open interpretation questions | [docs/nta8800-vragen-nen.md](docs/nta8800-vragen-nen.md) |
| HTTP API / MCP | [docs/nta8800-api.md](docs/nta8800-api.md), [docs/nta8800-mcp.md](docs/nta8800-mcp.md) |
| Form coverage | [docs/ui-redesign/formulierdekking.md](docs/ui-redesign/formulierdekking.md) |
| Quality procedures (concept) | [docs/kwaliteit/](docs/kwaliteit/kwaliteitshandboek.md) |

## Norm texts and licensing

- **No norm texts.** NTA 8800, ISSO publications and BRL texts are copyrighted and are **not** part of this repository. Code and documentation cite them by paragraph, table and page number only.
- **What you need yourself.** Your own licensed copies of the norm texts. Registration also requires a BRL 9501 attest for the software, which Open Energy Studio does not have yet.

The software is licensed under **LGPL-3.0-or-later**, as declared in the crate manifests. The licence texts are in [`COPYING.LESSER`](COPYING.LESSER) (LGPL-3.0) and [`COPYING`](COPYING) (GPL-3.0, which the LGPL builds on).

## Contributing

1. Create a branch from `nta8800-kernel`.
2. Make your change, with tests. Cite the norm paragraph and page for any calculation change.
3. Add a release-notes entry under `## Onuitgebracht`.
4. Run `scripts/verify-nta.sh`. A change is only accepted when the gate exits 0.
5. Open a pull request.

## Acknowledgments

- Built with [Tauri](https://tauri.app/), [React](https://react.dev/) and [Rust](https://www.rust-lang.org/).
- Icons by [Lucide](https://lucide.dev/).
