// The full-window screenshots of the manual (docs/handleiding-nta8800/img/),
// produced by capture.mjs from the real app. Each shot opens the app with a
// query and a route hash; `action` runs in the page (boot.js) once the app is
// idle, before the screenshot is taken.

export const SHOTS = [
  {
    name: 'welkom',
    width: 1280, height: 570,
    url: '/',
    action: 'closeAllTabs',
  },
  {
    name: 'projectoverzicht',
    width: 1280, height: 860,
    url: '/?example=terraced_dwelling#/project',
  },
  {
    name: 'verwarming',
    width: 1280, height: 820,
    url: '/?example=terraced_dwelling#/installaties/heating',
  },
  {
    name: 'palet',
    width: 1280, height: 820,
    url: '/?example=terraced_dwelling#/installaties/heating',
    action: 'openPalette',
    query: 'ventil',
  },
  {
    name: 'controle',
    width: 1280, height: 860,
    url: '/?example=terraced_dwelling#/controle',
  },
  {
    name: 'resultaten',
    width: 1280, height: 860,
    url: '/?example=terraced_dwelling#/resultaten',
  },
  {
    name: 'rapport',
    width: 1280, height: 860,
    url: '/?example=terraced_dwelling#/rapport',
  },
  {
    name: 'registratie',
    width: 1280, height: 860,
    url: '/?example=terraced_dwelling#/registratie',
  },
];

/** Browser storage every shot starts from: light theme, Dutch, inspector closed, a fixed recent list. */
export const STORAGE = {
  'energy-theme': 'light',
  'energy-locale': 'nl',
  'oes.inspector.open': '0',
  'oes-recent-projects': JSON.stringify([
    { path: 'Projecten/rijwoning-voorbeeld.oes', name: 'Rijwoning (voorbeeld)', buildingFunction: 'residential', labelClass: 'A++', at: '2026-10-09T10:00:00.000Z' },
    { path: 'Projecten/kantoor-voorbeeld.oes', name: 'Kantoor (voorbeeld)', buildingFunction: 'office', labelClass: 'A+', at: '2026-10-08T10:00:00.000Z' },
    { path: 'Projecten/basisopname-1930.oes', name: 'Basisopname woning 1930', buildingFunction: 'residential', at: '2026-10-06T10:00:00.000Z' },
  ]),
};
