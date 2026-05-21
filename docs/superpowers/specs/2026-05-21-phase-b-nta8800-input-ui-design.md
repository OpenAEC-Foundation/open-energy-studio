# Phase B — NTA 8800 Input UI

**Date:** 2026-05-21
**Status:** Ready for implementation plan
**Dependencies:** Phase A complete

## Purpose

Build the full NTA 8800 input workflow on top of the new Tauri+Zustand+Router shell.
Replace today's dialog-heavy modal UI with a guided page flow where each page is
responsible for one section of `ProjectV2`. Result: a user can create a complete BENG
project from scratch — building info, zones, envelope, systems, PV — and see live
BENG indicators on the Results page.

## User Flow

```
WelcomeScreen
    │ "New project" / "Open project" / Recent
    ▼
ProjectSetup          (/project)
    │ identification + location + building type + climate
    ▼
Zones                 (/zones)
    │ list of Rekenzones; per zone: floor area, height, EnergiefunctieRuimte
    ▼
Constructions         (/constructions)
    │ envelope library + assignment to zones (walls/roof/floor/openings)
    │ + thermal bridges
    ▼
Systems               (/systems)
    │ tabbed sub-pages: Heating, Cooling, DHW, Ventilation, Lighting, PV,
    │ Humidity, Automation
    ▼
Results               (/results)
    │ BENG 1/2/3, energy label, monthly Q_H/Q_C/Q_W, primary energy breakdown
```

The ribbon mirrors this flow as primary tabs. Each page has a "next/back" footer
that nudges the user along but allows free navigation.

## Page Specifications

### ProjectSetup (`/project`)

Single-column form, sections collapsible.

- **Identification**: project name, project number, client, address, postcode, place,
  date, designer
- **Location**: lat/lon (auto-fill from postcode via offline Dutch postcode table),
  climate zone (NL = single zone for BENG)
- **Building**: usage function (woonfunctie / utility sub-types per NTA 8800 Annex A),
  build year, status (new / existing / renovation)
- **References**: notes field

Writes to `project.shared` (SharedProject from `openaec-project-shared`).

### Zones (`/zones`)

Master/detail layout: zone list on left, zone editor on right.

Per zone (Rekenzone):
- name, color (for diagrams)
- floor area A_g [m²]
- average height h [m] (derived volume V shown)
- ventilation supply zone (yes/no)
- list of EnergiefunctieRuimtes within (each has function + A_g share)
- attached construction set (link to Constructions page)

Quick-add: "Heel gebouw is één zone" button auto-creates a single zone matching
A_g_total. Multi-zone projects (commercial) get the full master/detail.

Writes to `project.geometry.spaces` + `project.calcs.isso51.rekenzones` (or NTA 8800
equivalent path; resolved during implementation against `openaec-project-shared`).

### Constructions (`/constructions`)

Two-pane: catalogue on left (project-local constructions + library import),
property editor on right.

Per construction:
- name, type (Wall / Roof / Floor / WindowFrame / Door / Glazing / ThermalBridge)
- layers (for opaque): material + thickness; computes Rc; warns if Rc < minima
- U_g, g-value, frame fraction (for windows)
- ψ-value, length (for thermal bridges)

Assignment matrix: zone × construction × area, with "boundary" dropdown
(Exterior / Ground / AdjacentZone / Unheated). Sticky footer shows total envelope
area and net A_use.

Writes to `project.geometry.constructions` + `project.geometry.openings` + the
boundary assignments per Rekenzone.

### Systems (`/systems`) — tabbed

Tab strip across top: **Heating · Cooling · DHW · Ventilation · Lighting · PV ·
Humidity · Automation**.

Each tab is a separate sub-page; they all share the same chrome but render
distinct forms.

#### Heating (`/systems/heating`)
- generator: HRBoiler / HeatPump (air/water/ground) / ElectricResistance /
  DistrictHeating
- emission: RadiatorHT / RadiatorLT / FloorHeating / AirHeating
- distribution: insulated yes/no, location indoor/outdoor
- control: f_reg (auto from selectable preset)

#### Cooling (`/systems/cooling`)
- system: Compression / Absorption / FreeFlow / None
- distribution + emission analogous to heating
- SEER / EER auto-computed if not entered

#### DHW (`/systems/dhw`)
- generator: HR-combi / ElectricBoiler / HeatPump / DistrictHeat
- distribution losses
- shower water heat recovery (douchewtw): efficiency, type

#### Ventilation (`/systems/ventilation`)
- system: A (natural) / B / C / D (with WTW) / E
- supply/extract/infiltration flows per zone
- WTW: efficiency, bypass threshold
- fan power (specific fan power if available)

#### Lighting (`/systems/lighting`)
- per zone or building-level: P_n [W/m²], F_u, F_d, F_c
- preset library: kantoor/woning/winkel

#### PV (`/systems/pv`)
- per array: peak power [kWp], tilt, azimuth, system η, inverter η
- map widget for orientation (out of scope v1; numeric input only)

#### Humidity (`/systems/humidity`)
- humidification: Steam / Spray / None
- dehumidification: Adsorption / None
- target RH

#### Automation (`/systems/automation`)
- BACS class per service (A/B/C/D)
- preset: "Geen BACS" / "BACS klasse B" / "BACS klasse A"

### Results (`/results`)

Top section: large BENG-indicator cards (BENG 1, BENG 2, BENG 3 + energy label
A++++ … G). Each card shows computed value, normative limit, and pass/fail color.

Middle: monthly bar chart (stacked: heating, cooling, DHW, lighting, ventilation,
PV credit). Toggle: kWh/m² · year · primary / final.

Bottom: tabular breakdown by energy carrier and by post.

Actions on the page: **Recalculate · Export PDF · Export Vabi · Export Uniec ·
Verify** (jumps to Phase C).

## State

`projectStore` (defined in Phase A) holds `ProjectV2 | null`. Each page reads
slices and writes via:

```ts
useProjectStore.getState().patchProject((draft) => {
  draft.geometry.spaces.push(newSpace);
});
```

Auto-save: 2 s debounce after any patch, saves to `filePath` if known; otherwise
holds in memory and marks `isDirty`.

Auto-calc: when `project` mutates and `result` is non-null, schedule a
`calculate()` after 5 s idle. Cancellable. Status reflected in toolbar.

## Components

New components added in Phase B:

```
components/
├── forms/
│   ├── FormSection.tsx              # collapsible group
│   ├── FieldLabel.tsx
│   ├── NumberInput.tsx              # with unit suffix
│   ├── SelectField.tsx
│   └── ValidationMessage.tsx
├── layout/
│   ├── MasterDetail.tsx             # used by Zones + Constructions
│   └── TabStrip.tsx                 # used by Systems
└── results/
    ├── BengIndicatorCard.tsx
    ├── EnergyLabel.tsx              # A++++ ↔ G visual
    ├── MonthlyStackChart.tsx
    └── EnergyBreakdownTable.tsx
```

Existing dialogs in `src/components/dialogs/` are systematically migrated to the
relevant pages. Where a "dialog" is genuinely modal (e.g., choosing a material
from the library), we keep a slide-over panel instead.

## Validation

Per-field validation via Zod schemas mirroring the JSON Schemas exposed by Rust.
Zod schemas are derived from JSON Schema using `json-schema-to-zod`. A page shows
inline errors but never blocks navigation; the Results page surfaces a "Project
incomplete" banner with a checklist.

## Internationalisation

All labels are i18n keys. All 14 existing languages are kept; Dutch is the
authoritative source. New keys live under `pages.<page>.*` namespaces. Lokalise
or similar service handover is not in scope; the team continues to maintain
translations by hand.

## Accessibility

- All form fields have `<label htmlFor>` bindings
- Tab order matches visual order
- Color is never the only signal (BENG cards include text status)
- Targets ≥ 32 px high
- Keyboard shortcuts: `Ctrl+S` save, `Ctrl+Enter` recalculate, `Ctrl+1..6` page
  jump

## Testing

- Unit tests for each form component (Vitest + Testing Library)
- Store tests for `patchProject` + autosave debounce
- Page-level smoke tests: render `<ProjectSetup>` with empty project, fill fields,
  assert store updates correctly
- A "happy path" Playwright test (out of scope for v1; manual QA acceptable)

## Risks

| Risk | Mitigation |
|---|---|
| Form sprawl — too many fields per page | Strict YAGNI; only fields needed for valid NTA 8800 input |
| `ProjectV2` shape from `openaec-project-shared` doesn't expose all NTA 8800 inputs | Coordinate with crates-warehouse; add fields as needed |
| Migration of legacy projects loses metadata users care about | Migration UI shows a "what changed" diff before saving |

## Success Criteria

1. User can create a new project from scratch and complete all required NTA 8800
   inputs through the page flow.
2. Recalculating after every page change converges (no infinite loops, no stale
   results).
3. All form fields validated against the Rust schema.
4. Dutch UI strings reviewed by a domain expert (NTA 8800 terminology accuracy).
5. The Results page shows BENG indicators matching the legacy app within ±0.5%
   for a regression fixture.
