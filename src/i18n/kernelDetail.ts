import { formatNumber, kernelCodeLabel } from './format';

/*
 * The kernel adds an English technical `detail` to some gaps and warnings. These
 * patterns turn the known ones into a translated sentence with numbers in the reader's
 * locale; any other detail is shown as it is, marked as a technical detail.
 */

type Translate = (key: string, options?: Record<string, unknown>) => string;

const NUMBER = '(-?\\d+(?:\\.\\d+)?(?:e[+-]?\\d+)?)';

interface DetailPattern {
  /** Matched against the whole detail. */
  pattern: RegExp;
  /** i18n key of the sentence; capture groups fill `{{p1}}`, `{{p2}}`, … */
  key: string;
  /** Capture groups (1-based) that are numbers and get locale formatting. */
  numbers?: number[];
}

const n = NUMBER;
const DETAIL_PATTERNS: DetailPattern[] = [
  { pattern: new RegExp(`^\\|?${n}\\|? > ${n}$`), key: 'kernel.detail.aboveBound', numbers: [1, 2] },
  { pattern: new RegExp(`^${n} m² per dwelling$`), key: 'kernel.detail.areaPerDwelling', numbers: [1] },
  { pattern: new RegExp(`^A_ls/A_g ${n}$`), key: 'kernel.detail.lossAreaRatio', numbers: [1] },
  { pattern: /^registration (\d+) against ntaCalculation (\d+)$/, key: 'kernel.detail.constructionYearMismatch' },
  { pattern: /^ventilation \(table 11\.13\) (\d+) before bouwjaar (\d+)$/, key: 'kernel.detail.ventilationYearBefore' },
  {
    pattern: new RegExp(`^Q_W;nd ≈ ${n} kWh/yr against ${n} kWh/yr declared fuel: η_W ≈ ${n} > 1$`),
    key: 'kernel.detail.hotWaterEfficiency',
    numbers: [1, 2, 3],
  },
  {
    pattern: new RegExp(`^H_ve ${n} W/K is below ρc·q_V;ODA;req ≈ ${n} W/K \\(${n} m³/h, 11\\.22 with the lowest table 11\\.5 f_ctrl·f_sys ${n}\\) of the mechanical system without heat recovery$`),
    key: 'kernel.detail.ventilationBelowRequired',
    numbers: [1, 2, 3, 4],
  },
  {
    pattern: new RegExp(`^${n} m²K/W is below R_si = 0,17: the input is R_si \\+ R_c of the floor$`),
    key: 'kernel.detail.floorResistanceBelowRsi',
    numbers: [1],
  },
  {
    pattern: new RegExp(`^P = ${n} m on A = ${n} m² implies a mean floor width below ${n} m \\(B' = ${n} m\\)$`),
    key: 'kernel.detail.floorPerimeterImplausible',
    numbers: [1, 2, 3, 4],
  },
  {
    pattern: new RegExp(`^declared b_U ${n} and H_zi;ztu ${n} W/K; unheated space (.+) gives b_U ${n} and H_zi;ztu ${n} W/K \\(8\\.4\\.1\\)$`),
    key: 'kernel.detail.sunroomDiffers',
    numbers: [1, 2, 4, 5],
  },
  { pattern: new RegExp(`^8\\.38 takes ΔU_for of 8\\.2\\.1 \\(8\\.3\\): ${n} W/\\(m²K\\)$`), key: 'kernel.detail.basementDeltaU', numbers: [1] },
  { pattern: /^§5\.5\.8: without the bacs block/, key: 'kernel.detail.bacsWithoutEvidence' },
  { pattern: /^Table 7\.10 a: utility buildings take the closed or suspended ceiling column/, key: 'kernel.detail.utilityOpenCeiling' },
  { pattern: /^Detailed thermal-bridge route \(8\.2\.1\) without linear thermal bridges/, key: 'kernel.detail.detailedBridgesNone' },
  { pattern: /^7\.3\.3: zone list \[\] \(none\) conflicts with the project-level pipes/, key: 'kernel.detail.verticalPipesConflicting' },
  { pattern: /^7\.3\.3: state the pipes, \[\] for none;/, key: 'kernel.detail.verticalPipesUnknown' },
];

function decimals(raw: string): number {
  const fraction = raw.split('e')[0].split('.')[1];
  return fraction ? fraction.length : 0;
}

/**
 * The detail in the reader's language. `translated` is false when no pattern matched;
 * the caller then shows the raw text as a technical detail.
 */
export function kernelDetailText(detail: string, t: Translate, locale: string): { text: string; translated: boolean } {
  // Some details carry a nested kernel code (for example a variant's own gap).
  if (/^[a-z][a-z0-9_]*$/.test(detail)) {
    const label = kernelCodeLabel(t, detail);
    if (label.known) return { text: `${label.text} (${detail})`, translated: true };
  }
  for (const item of DETAIL_PATTERNS) {
    const match = item.pattern.exec(detail);
    if (!match) continue;
    const params: Record<string, string> = {};
    match.slice(1).forEach((value, index) => {
      const group = index + 1;
      params[`p${group}`] = item.numbers?.includes(group)
        ? formatNumber(Number(value), locale, decimals(value))
        : value;
    });
    const template = t(item.key, params);
    if (template === item.key) continue;
    // Plain lookups (such as the Dutch report tables) do not interpolate; fill in here.
    const text = template.replace(/\{\{(p\d+)\}\}/g, (whole, name: string) => params[name] ?? whole);
    return { text, translated: true };
  }
  return { text: detail, translated: false };
}
