/**
 * The maatwerkadvies kernel writes its advice warnings and specialist notes as Dutch
 * sentences. In another UI language the known kernel sentences are translated here;
 * text entered by the adviser (package and measure names, own notes) stays as entered.
 */
const ENGLISH: Array<[RegExp, (...groups: string[]) => string]> = [
  [/^ventilation_practice_not_applied: .*$/s, () =>
    'ventilation_practice_not_applied: the ventilation practice factors (ISSO 82.2 table 2.7) need the chapter 11 route; zones with supplied ventilation flows are not corrected'],
  [/^ISSO 82\.2\/75\.2 §4\.2\.2: combineer de maatregelen in minimaal twee pakketten$/, () =>
    'ISSO 82.2/75.2 §4.2.2: combine the measures into at least two packages'],
  [/^(.+): berekening ongeldig$/s, (name) => `${name}: calculation invalid`],
  [/^(.+): TOjuli stijgt van (\S+) naar (\S+); het risico op oververhitting neemt toe \(ISSO 82\.2 §4\.2\.3\)$/s,
    (name, before, after) => `${name}: TOjuli rises from ${before} to ${after}; the overheating risk increases (ISSO 82.2 §4.2.3)`],
  [/^(.+): systeemeis Bbl art\. 4\.248 voor (.+) niet gehaald \((.+)\); geldt bij vervanging of verbetering van het systeem \(ISSO 82\.2 §5\.2\)$/s,
    (name, system, values) => `${name}: Bbl art. 4.248 system requirement for ${system} not met (${values}); applies when the system is replaced or improved (ISSO 82.2 §5.2)`],
  [/^(.+): de energiekosten nemen toe$/s, (name) => `${name}: the energy costs increase`],
  [/^(.+): vermogensbepaling door een specialist \(ISSO 82\.2 §6\.2\.3\)$/s,
    (name) => `${name}: capacity to be determined by a specialist (ISSO 82.2 §6.2.3)`],
  [/^(.+): samenstelling van het isolatiepakket bij condensatierisico laten uitwerken door een specialist \(ISSO 82\.2 §6\.2\.3\)$/s,
    (name) => `${name}: have a specialist work out the insulation build-up where there is a condensation risk (ISSO 82.2 §6.2.3)`],
];

/** The advice text in the UI language: Dutch as written, English for the known kernel sentences. */
export function adviceText(text: string, locale: string): string {
  if (locale.toLowerCase().startsWith('nl')) return text;
  for (const [pattern, render] of ENGLISH) {
    const match = pattern.exec(text);
    if (match) return render(...match.slice(1));
  }
  return text;
}
