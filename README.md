# Open Energy Studio

Open-source building energy performance calculator following the Dutch **NTA 8800** standard.

![License](https://img.shields.io/badge/license-LGPL--3.0-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)
![TypeScript](https://img.shields.io/badge/TypeScript-5.9-blue.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.0-orange.svg)

## Overview

Open Energy Studio is a free, open-source desktop application for calculating **BENG** (Building Energy Performance) indicators according to the NTA 8800 standard. It helps architects, engineers, and energy consultants evaluate building energy performance and demonstrate regulatory compliance in the Netherlands.

### BENG Indicators

| Indicator | Description | Unit |
|-----------|-------------|------|
| **BENG 1** | Energy demand (heating + cooling) | kWh/(m²/year) |
| **BENG 2** | Primary fossil energy use | kWh/(m²/year) |
| **BENG 3** | Renewable energy share | % (min. 50%) |
| **TO-juli** | Summer overheating risk | GTO index |
| **Energy Label** | Classification from A++++ to G | — |

### Key Features

- **Building Modeling** — Define thermal zones, surfaces (walls, roofs, floors), windows, thermal bridges, and air tightness
- **Construction Editor** — Build up wall/roof/floor constructions layer by layer with automatic U-value calculation
- **System Configuration** — Heating (heat pumps, gas boilers, district), ventilation (natural, type C/D), cooling, and hot water systems
- **Renewables** — Solar PV panels and solar thermal collectors with orientation and tilt settings
- **Monthly Calculation Engine** — Month-by-month energy simulation using Dutch climate data
- **3D Visualization** — Interactive 3D view of the building envelope
- **Report Generation** — Export detailed HTML/PDF reports
- **IFC Export** — Building Information Model export (BENG data and geometry)
- **Data Exchange** — Import/export UNIEC3 and VABI Elements formats
- **Collapsible Panels** — Resizable project browser and properties panel
- **14 Languages** — Dutch, English, German, French, Spanish, Italian, Portuguese, Polish, Turkish, Arabic, Farsi, Japanese, Korean, Chinese
- **4 Themes** — Dark, Light, Blue, High Contrast (plus System auto-detect)
- **Cross-Platform** — Windows, Linux, and macOS installers

## Technology Stack

| Technology | Purpose |
|------------|---------|
| **Tauri 2** | Desktop application framework (~10 MB app size) |
| **React 19** | Component-based UI |
| **TypeScript** | Type-safe frontend and calculation engine |
| **Vite** | Fast build tooling |
| **Rust** | Native backend (file I/O, window management) |
| **Lucide** | Icon library |

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri 2 (Rust Shell)                  │
│  - Native window management & custom title bar          │
│  - File system access (save/open dialogs)               │
│  - Cross-platform installers (NSIS, DMG, DEB, RPM)      │
├─────────────────────────────────────────────────────────┤
│                    React + TypeScript                    │
│  - Ribbon UI with File menu                             │
│  - Project browser & properties panel                   │
│  - Dialog system (draggable, reusable shell)            │
│  - i18n (14 languages, RTL support)                     │
├─────────────────────────────────────────────────────────┤
│                  Calculation Engine                      │
│  - NTA 8800 monthly method                              │
│  - BENG 1/2/3 + TO-juli + Energy Label                  │
│  - Transmission, ventilation, solar gain models          │
│  - IFC & report generation                              │
└─────────────────────────────────────────────────────────┘
```

## Getting Started

### Prerequisites

- **Node.js** 20+
- **Rust** 1.77+ (for Tauri backend)
- **npm**

#### Linux only

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/OpenAEC-Foundation/open-energy-studio.git
   cd open-energy-studio
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Run in development mode:
   ```bash
   npm run tauri dev
   ```

4. Build for production:
   ```bash
   npm run tauri build
   ```

## Usage

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New project |
| `Ctrl+O` | Open project |
| `Ctrl+S` | Save project |
| `Ctrl+Shift+S` | Save as |
| `Escape` | Close menu / dialog |

### Workflow

1. **Create a project** — Set building name, address, and function (residential, office, etc.)
2. **Define zones** — Add thermal zones with floor area, volume, and height
3. **Model the envelope** — Add surfaces, windows, constructions, and thermal bridges
4. **Configure systems** — Set up heating, ventilation, cooling, and hot water systems
5. **Add renewables** — Configure solar PV and/or solar thermal collectors
6. **Calculate** — Run BENG calculation and review results
7. **Export** — Generate reports, export IFC, or exchange data with other tools

### Built-in Tools

- **U-Value Calculator** — Compute U-values from construction layer definitions
- **Thermal Bridge Calculator** — Calculate linear thermal transmittance (psi-values)
- **Heat Pump Sizing Calculator** — Determine required heat pump capacity

### Building Functions

Residential, Office, Education, Healthcare, Retail, Industrial — each with specific NTA 8800 regulatory limits and default parameters.

## Project Structure

```
open-energy-studio/
├── src/                          # Frontend source
│   ├── components/               # React components
│   │   ├── AppMenu/             # File application menu
│   │   ├── Ribbon/              # Ribbon toolbar (Home, Envelope, etc.)
│   │   ├── TitleBar/            # Custom window title bar
│   │   ├── ProjectBrowser/      # Left panel - project tree
│   │   ├── PropertiesPanel/     # Right panel - item properties
│   │   ├── PreviewPanel/        # Right panel - live BENG preview
│   │   ├── MainView/            # Central content area
│   │   ├── StatusBar/           # Bottom status bar
│   │   ├── SettingsDialog/      # App settings (theme, language)
│   │   ├── dialogs/             # All editor dialogs + DialogShell
│   │   ├── Building3DView/      # 3D building visualization
│   │   ├── UValueCalculator/    # U-value calculation tool
│   │   ├── ThermalBridgeCalculator/
│   │   └── HeatPumpSizingCalculator/
│   ├── core/                    # Calculation engine
│   │   ├── energy/              # BENG monthly calculator, types
│   │   ├── ifc/                 # IFC exporters (BENG + model)
│   │   ├── io/                  # Project serializer, UNIEC3, VABI
│   │   └── report/              # HTML report generator
│   ├── context/                 # React context (state management)
│   └── i18n/                    # 14 language translations
├── src-tauri/                   # Rust backend
│   ├── src/
│   │   ├── lib.rs               # Plugin registration & setup
│   │   └── main.rs              # Tauri entry point
│   ├── capabilities/            # Tauri permissions
│   ├── icons/                   # App icons
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri configuration
├── .github/workflows/           # CI + release pipelines
└── package.json
```

## Versioning

The project uses **CalVer**: `YEAR.MONTH.BUILD`

Examples: `2026.2.0` → `2026.2.1` → `2026.3.0`

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the **LGPL-3.0-or-later** license.

## Acknowledgments

- Built with [Tauri](https://tauri.app/), [React](https://react.dev/), and [Rust](https://www.rust-lang.org/)
- Icons by [Lucide](https://lucide.dev/)
- Energy calculations based on the NTA 8800 standard

## Support

- [Report bugs or suggest features](https://github.com/OpenAEC-Foundation/open-energy-studio/issues)
- Star the repository if you find it useful
