# Manual screenshots

Reproducible screenshots for the NTA 8800 manual (`docs/handleiding-nta8800/img/`).

```sh
node scripts/manual-screenshots/capture.mjs            # all shots in shots.mjs
node scripts/manual-screenshots/capture.mjs resultaten # one or more by name
```

Needs the project toolchain (`cargo`, `node`), `firefox` on the PATH and `npm install` done (the script uses `vite` and `sharp`).

## How it works

- `capture.mjs` builds the `kernel_once` example of `crates/nta8800-service`, starts a temporary vite dev server on a free loopback port (never 3917), runs `firefox --headless --screenshot` once per shot with a fresh profile, colour-reduces the PNG with `sharp` (128 colours) into the image folder and stops the server.
- `vite.config.mjs` serves the real app from the repository root. The app's `/api/…` calls go to `kernel_once`, which runs the requested kernel operation in-process (`operations::execute`, as the HTTP API does) and exits. No API server is started and no port is opened for the kernel, so the figures on the screenshots are the current kernel's.
- `boot.js` is loaded before the app. It sets the browser storage (light theme, Dutch, inspector closed, a fixed recent-projects list), navigates to the shot's route once the example is open, runs the shot's action and then releases a held image request. Firefox takes the screenshot at the load event, so it waits for that release.
- `shots.mjs` lists the shots: name, window size, URL (`?example=…` opens an example project, `#/…` is the route) and an optional action.
- Set `SHOT_TMPDIR` to put the temporary Firefox profiles somewhere other than the system temp directory.

## Images in docs/handleiding-nta8800/img/

| Image | Chapter | Made with |
|---|---|---|
| `welkom.png` | 0 | `capture.mjs` (all tabs closed) |
| `projectoverzicht.png` | 0 | `capture.mjs` |
| `verwarming.png` | 0 | `capture.mjs` |
| `palet.png` | 0 | `capture.mjs` (palette opened, "ventil" typed) |
| `controle.png` | 0 | `capture.mjs` |
| `resultaten.png` | 0 | `capture.mjs` |
| `rapport.png` | 0 | `capture.mjs` |
| `registratie.png` | 0 | `capture.mjs` |
| `handleiding-venster.png` | 0 | component harness, not kept |
| `opname-voortgang.png`, `opname-uitkomst-onderdeel.png`, `overnemen-vloeren.png` | 2 | component harness, not kept |
| `gemeenschappelijke-ruimte.png`, `belemmering-per-raam.png`, `zonwering-per-raam.png`, `glas-per-raam.png`, `bijlage-aa.png`, `psi-leidinggegevens.png`, `kwaliteitsverklaring-ketel-wkk.png`, `fctrl-gedeclareerd.png` | 3 | component harness, not kept |
| `koudebrugmethode.png`, `gedeelde-serre.png`, `kelder-wanddelen.png`, `smalle-luchtlaag.png` | 3 | component harness, not kept |
| `bron-en-bewijs.png`, `verklaringen-label.png` | 7 | component harness, not kept |
| `uitgave-keuze.png`, `uitgave-wisselen.png`, `resultaten-oudere-uitgave.png` | 10 | component harness, not kept |

"Component harness, not kept": these show one part of a page, rendered on its own with an example project or public case from `training-data/` (and, where a kernel answer was needed, a recorded one). The harness for them was temporary and is not in the repository; to redo one, add a shot to `shots.mjs` that opens the page and crop, or render the component in a similar harness. `docs/handleiding-nta8800/index.md` describes what each one shows.
