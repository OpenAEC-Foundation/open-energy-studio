/**
 * The basisopname as a question flow (UI redesign 2026-10, mockup B and C):
 * one question per screen, choice cards for the main questions, "skip, fill in
 * later", a live provisional label, a Controle page with everything on one
 * page and a Label page. The fields themselves are the BasisopnamePanel parts.
 */
import { useEffect, useMemo, useRef, type ReactNode } from 'react';
import {
  Ban, Building2, CircleHelp, Factory, Fan, Flame, Home, Network, Plug, Thermometer, TreePine, Wind, Zap,
} from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import { useEnergy } from '../../context/EnergyContext';
import {
  BasisopnamePanel, HEATING_KINDS, HOT_WATER_KINDS, VENTILATION_PRINCIPLES, defaultPathLabel, defaultValueLabel,
} from '../BasisopnamePanel/BasisopnamePanel';
import { read, SelectField, write, type Draft, type Path } from '../NtaPerformancePanel/NtaFormFields';
import {
  heatingGeneratorTemplate, hotWaterGeneratorTemplate, surveyTemplate, type StoredSurvey, type SurveyKind,
} from '../../core/nta/SurveyTemplates';
import {
  markProgress, questionForPath, questionKey, questionState, stepForKind, stepState, surveyStep, surveySteps,
  type SurveyFlowStep, type SurveyPart, type SurveyProgress, type SurveyStepId,
} from '../../core/survey/surveyFlow';
import { assessSurvey, currentResult, useSurveyAssessment } from '../../core/survey/surveyAssessment';
import { labelColor } from '../shell/pages/results/resultsData';
import { KernelCode } from '../KernelCode/KernelCode';
import type { Route } from '../../core/navigation/routes';
import type { OpnameAssessment } from '../../core/nta/KernelClient';
import './SurveyWizard.css';

type T = (key: string, options?: Record<string, unknown>) => string;
type Stored = StoredSurvey & { progress?: SurveyProgress };

export interface SurveyWizardProps {
  route: Route;
  navigate: (route: Route) => void;
}

const HEATING_ICONS: Record<string, ReactNode> = {
  boiler: <Flame />, heat_pump: <Thermometer />, district_heat: <Network />, electric: <Zap />, biomass: <TreePine />,
  chp: <Factory />, local_fired: <Flame />, gas_air_heater: <Wind />, none_present: <Ban />,
};
const HOT_WATER_ICONS: Record<string, ReactNode> = {
  gas_appliance: <Flame />, gas_storage_heater: <Flame />, electric_boiler: <Plug />, electric_instantaneous: <Zap />,
  heat_pump: <Thermometer />, district_heat: <Network />, collective_unknown: <Building2 />, delivery_set_from_heating: <Network />,
  none: <Ban />,
};
const VENTILATION_ICONS: Record<string, ReactNode> = {
  natural: <Wind />, mechanical_extract: <Fan />, balanced: <Fan />, mechanical_supply: <Fan />,
};
const DWELLINGS = ['terraced', 'end_or_corner', 'detached', 'apartment'] as const;

/** The dwelling type of the survey as one of the choice cards. */
function dwellingChoice(draft: Draft): string | null {
  const kind = read(draft, ['dwelling', 'kind']);
  if (kind === 'apartment') return 'apartment';
  const position = read(draft, ['dwelling', 'position']);
  return typeof position === 'string' && position !== 'unknown' ? position : null;
}

function ChoiceCards({ options, selected, onPick, label }: {
  options: Array<{ id: string; title: string; hint: string; icon: ReactNode }>;
  selected: string | null;
  onPick: (id: string) => void;
  label: string;
}) {
  return <div className="survey-choices" role="group" aria-label={label}>
    {options.map((option) => (
      <button key={option.id} type="button" className="survey-choice" aria-pressed={selected === option.id}
        onClick={() => onPick(option.id)}>
        <span className="survey-choice-icon" aria-hidden="true">{option.icon}</span>
        <span className="survey-choice-title">{option.title}</span>
        <span className="survey-choice-hint">{option.hint}</span>
      </button>
    ))}
  </div>;
}

/** The fields of one question: choice cards, or the matching part of the survey form. */
/** The project and address of the survey, as on the ISSO opnameformulier (paragraph 1). */
function AddressFields({ t }: { t: T }) {
  const { state, dispatch } = useEnergy();
  const project = state.project;
  const registration = (project.registration ?? {}) as NonNullable<typeof project.registration>;
  const set = (payload: Record<string, unknown>) => dispatch({ type: 'UPDATE_PROJECT_INFO', payload });
  const text = (label: string, value: string | undefined, onChange: (value: string) => void, wide = false) =>
    <label className={wide ? 'survey-address-wide' : undefined}>{label}
      <input type="text" value={value ?? ''} onChange={(event) => onChange(event.target.value)} />
    </label>;
  return <div className="nta-form survey-address"><div className="nta-form-grid">
    {text(t('survey.address.name'), project.name, (name) => set({ name }), true)}
    {text(t('survey.address.street'), project.address, (address) => set({ address }), true)}
    {text(t('survey.address.postcode'), (registration as { postcode?: string }).postcode,
      (postcode) => set({ registration: { ...registration, postcode: postcode || undefined } }))}
    {text(t('survey.address.city'), project.city, (city) => set({ city }))}
    <p className="nta-form-subhead">{t('survey.address.adviser')}</p>
    {text(t('survey.address.adviserName'), registration.surveyingAdvisor?.name,
      (name) => set({ registration: { ...registration, surveyingAdvisor: { name, competenceNumber: registration.surveyingAdvisor?.competenceNumber ?? '' } } }))}
    {text(t('survey.address.adviserNumber'), registration.surveyingAdvisor?.competenceNumber,
      (competenceNumber) => set({ registration: { ...registration, surveyingAdvisor: { name: registration.surveyingAdvisor?.name ?? '', competenceNumber } } }))}
    {text(t('survey.address.certificate'), registration.certificateNumber,
      (certificateNumber) => set({ registration: { ...registration, certificateNumber: certificateNumber || undefined } }))}
  </div></div>;
}

function QuestionBody({ part, stored, draft, change, t }: {
  part: SurveyPart; stored: Stored; draft: Draft; change: (path: Path, value: unknown) => void; t: T;
}) {
  const field = { draft, onChange: change };
  switch (part) {
    case 'address': return <AddressFields t={t} />;
    case 'dwellingType': {
      const choice = dwellingChoice(draft);
      const pick = (id: string) => {
        if (id === 'apartment') {
          change(['dwelling'], {
            kind: 'apartment',
            floor: read(draft, ['dwelling', 'floor']) ?? 'ground_or_intermediate',
            side: read(draft, ['dwelling', 'side']) ?? 'middle',
          });
        } else {
          change(['dwelling'], { kind: 'single_family', position: id, roofType: read(draft, ['dwelling', 'roofType']) ?? 'pitched' });
        }
      };
      return <>
        <ChoiceCards label={t('survey.q.woning.soort')} selected={choice} onPick={pick}
          options={DWELLINGS.map((id) => ({
            id, title: t(`survey.dwelling.${id}`), hint: t(`survey.dwelling.${id}.hint`),
            icon: id === 'apartment' ? <Building2 /> : <Home />,
          }))} />
        <div className="nta-form survey-followup">
          {choice === 'apartment' ? <>
            <SelectField {...field} path={['dwelling', 'floor']} label={t('opname.dwelling.floor')}
              options={['ground_or_intermediate', 'top', 'roof_and_floor'].map((key) => [key, t(`opname.dwelling.floorKind.${key}`)])} />
            <SelectField {...field} path={['dwelling', 'side']} label={t('survey.dwelling.side')}
              options={['middle', 'end_or_corner', 'unknown'].map((key) => [key, t(`survey.dwelling.sideKind.${key}`)])} />
          </> : choice != null && <SelectField {...field} path={['dwelling', 'roofType']} label={t('survey.dwelling.roofType')}
            options={['pitched', 'partly_flat', 'flat'].map((key) => [key, t(`survey.dwelling.roofTypeKind.${key}`)])} />}
        </div>
      </>;
    }
    case 'heatingKind': {
      const current = read(draft, ['heating', 'generator', 'kind']);
      return <ChoiceCards label={t('survey.q.verwarming.toestel')} selected={typeof current === 'string' ? current : null}
        onPick={(id) => { if (id !== current) change(['heating', 'generator'], heatingGeneratorTemplate(id)); }}
        options={HEATING_KINDS.map((id) => ({
          id, title: t(`opname.heating.kind.${id}`), hint: t(`survey.heating.${id}.hint`), icon: HEATING_ICONS[id] ?? <CircleHelp />,
        }))} />;
    }
    case 'hotWaterKind': {
      const current = read(draft, ['hotWater', 'generator', 'kind']);
      return <ChoiceCards label={t('survey.q.warm-water.toestel')} selected={typeof current === 'string' ? current : null}
        onPick={(id) => { if (id !== current) change(['hotWater', 'generator'], hotWaterGeneratorTemplate(id)); }}
        options={HOT_WATER_KINDS[stored.kind].map((id) => ({
          id, title: t(`opname.hotWater.kind.${id}`), hint: t(`survey.hotWater.${id}.hint`), icon: HOT_WATER_ICONS[id] ?? <CircleHelp />,
        }))} />;
    }
    case 'ventilationPrinciple': {
      const current = read(draft, ['ventilation', 'principle']);
      return <ChoiceCards label={t('survey.q.ventilatie.systeem')} selected={typeof current === 'string' ? current : null}
        onPick={(id) => change(['ventilation', 'principle'], id)}
        options={VENTILATION_PRINCIPLES.map((id) => ({
          id, title: t(`survey.ventilation.principleKind.${id}`), hint: t(`survey.ventilation.${id}.hint`),
          icon: VENTILATION_ICONS[id] ?? <Fan />,
        }))} />;
    }
    default:
      return <BasisopnamePanel part={part} />;
  }
}

/** Field labels of the answers a survey needs before the kernel can read it. */
const MISSING_LABEL: Record<string, string> = {
  dwelling: 'survey.q.woning.soort', 'dwelling.roofType': 'survey.dwelling.roofType', 'dwelling.floor': 'opname.dwelling.floor',
  constructionYear: 'opname.constructionYear', usableFloorAreaM2: 'opname.usableFloorArea', buildingHeightM: 'opname.buildingHeight',
  'construction.floor': 'survey.construction.floor', 'construction.wall': 'survey.construction.wall',
  'heating.generator': 'survey.q.verwarming.toestel', 'heating.emitters': 'opname.heating.emitters',
  'hotWater.generator': 'survey.q.warm-water.toestel', 'ventilation.principle': 'survey.ventilation.principle',
};

/** What a notice is about: the field of a missing answer, or the surface, window, door or panel by its name. */
function issueSubject(path: string, draft: Draft, t: T): string | null {
  const bare = path.replace(/^basisopname\./, '');
  if (MISSING_LABEL[bare]) return t(MISSING_LABEL[bare]);
  const item = /^(envelope\.(?:surfaces|windows|doors|rooflights)|pv)\[(\d+)\]/.exec(bare);
  if (item) {
    const list = read(draft, item[1].split('.')) as Array<Record<string, unknown>> | undefined;
    const id = list?.[Number(item[2])]?.id;
    if (id) return String(id);
  }
  return bare.startsWith('derivedInput') ? null : bare;
}

/** Display names of the surfaces: "Gevel 1", "Dak 2", "Vloer 1" (the order within each element). */
function surfaceNames(draft: Draft, t: T): Map<string, string> {
  const counts: Record<string, number> = {};
  const names = new Map<string, string>();
  for (const surface of (read(draft, ['envelope', 'surfaces']) as Array<Record<string, unknown>> | undefined) ?? []) {
    const element = String(surface.element ?? 'facade');
    counts[element] = (counts[element] ?? 0) + 1;
    names.set(String(surface.id ?? ''), `${t(`survey.element.${element}`)} ${counts[element]}`);
  }
  return names;
}

/** "80 mm" or "100–120 mm" for surfaces with a known thickness; null when none has one. */
function thicknessRange(items: Array<Record<string, unknown>>, locale: string): string | null {
  const values = items.map((item) => (item.insulation as { kind?: string; thicknessMm?: number } | undefined))
    .filter((insulation) => insulation?.kind === 'thickness' && typeof insulation.thicknessMm === 'number')
    .map((insulation) => insulation!.thicknessMm as number);
  if (values.length === 0) return null;
  const low = Math.min(...values);
  const high = Math.max(...values);
  return low === high ? `${formatNumber(low, locale, 0)} mm` : `${formatNumber(low, locale, 0)}–${formatNumber(high, locale, 0)} mm`;
}

/** The compass direction of an azimuth (0 = north, clockwise). */
function compass(azimuth: number): string {
  const keys = ['north', 'north_east', 'east', 'south_east', 'south', 'south_west', 'west', 'north_west'];
  return keys[Math.round((((azimuth % 360) + 360) % 360) / 45) % 8];
}

/** Short facts of a step for the Controle page: the answers that matter, in a few lines. */
export function stepFacts(step: SurveyStepId, draft: Draft, t: T, locale: string, address?: string): string[] {
  const n = (value: unknown, digits = 0) => typeof value === 'number' ? formatNumber(value, locale, digits) : '—';
  const list = (path: Path) => (read(draft, path) as Array<Record<string, unknown>> | undefined) ?? [];
  const surfaces = list(['envelope', 'surfaces']);
  const windows = list(['envelope', 'windows']);
  const doors = list(['envelope', 'doors']);
  const rooflights = list(['envelope', 'rooflights']);
  const area = (items: Array<Record<string, unknown>>, key: string) => items.reduce((sum, item) => sum + (Number(item[key]) || 0), 0);
  const elementOf = (surfaceId: unknown) => surfaces.find((surface) => surface.id === surfaceId)?.element;
  const label = (key: string) => { const text = t(key); return text === key ? null : text; };
  const short = (key: string) => (label(key) ?? '').split(':')[0];
  const year = read(draft, ['constructionYear']);
  switch (step) {
    case 'woning': {
      const choice = dwellingChoice(draft);
      const roof = read(draft, ['dwelling', 'roofType']);
      const floor = read(draft, ['construction', 'floor']);
      const wall = read(draft, ['construction', 'wall']);
      const pipes = read(draft, ['verticalPipes']);
      const facts = [[choice ? t(`survey.dwelling.${choice}`) : '—', roof ? label(`survey.dwelling.roofTypeKind.${String(roof)}`)?.toLowerCase() : null]
        .filter(Boolean).join(', ')];
      if (address) facts.unshift(address);
      facts.push(t('survey.fact.basics', { year: typeof year === 'number' ? String(year) : '—', area: n(read(draft, ['usableFloorAreaM2']), 1),
        storeys: n(read(draft, ['storeys'])) }));
      if (floor || wall) facts.push(t('survey.fact.construction', { floor: floor ? short(`survey.construction.floorKind.${String(floor)}`).toLowerCase() : '—',
        wall: wall ? short(`survey.construction.wallKind.${String(wall)}`).toLowerCase() : '—' }));
      if (Array.isArray(pipes)) {
        const insulated = pipes.filter((pipe) => (pipe as { insulated?: boolean }).insulated === true).length;
        facts.push(pipes.length === 0 ? t('survey.fact.noPipes')
          : t('survey.fact.pipes', { count: pipes.length, insulated: insulated === pipes.length ? t('survey.fact.insulated') : insulated === 0 ? t('survey.fact.notInsulated') : t('survey.fact.partlyInsulated') }));
      }
      return facts;
    }
    case 'gebouw': {
      const functions = list(['functions']);
      return [t('survey.fact.year', { year: typeof year === 'number' ? String(year) : '—' }),
        t('survey.fact.functions', { count: functions.length, area: n(area(functions, 'areaM2'), 0) })];
    }
    case 'zones': return [t('survey.fact.zones', { count: list(['zones']).length })];
    case 'gevels': {
      const facades = surfaces.filter((surface) => surface.element === 'facade');
      const own = windows.filter((window) => elementOf(window.surfaceId) !== 'roof');
      const glass = own[0]?.glass;
      const facts = [[t('survey.fact.facades', { count: facades.length, area: n(area(facades, 'grossAreaM2'), 1) }), thicknessRange(facades, locale)]
        .filter(Boolean).join(', ')];
      if (own.length > 0) facts.push([t('survey.fact.windows', { count: own.length, area: n(area(own, 'areaM2'), 1) }),
        glass && own.every((window) => window.glass === glass) ? label(`opname.window.glassKind.${String(glass)}`) : null].filter(Boolean).join(', '));
      if (doors.length > 0) facts.push([t('survey.fact.doors', { count: doors.length, area: n(area(doors, 'areaM2'), 2) }),
        doors.every((door) => door.insulated === true) ? t('survey.fact.insulated') : null].filter(Boolean).join(', '));
      return facts;
    }
    case 'dak-vloer': {
      const roofs = surfaces.filter((surface) => surface.element === 'roof');
      const floors = surfaces.filter((surface) => surface.element === 'floor');
      const roofWindows = windows.filter((window) => elementOf(window.surfaceId) === 'roof');
      const names = surfaceNames(draft, t);
      const facts = [[t('survey.fact.roofs', { count: roofs.length, area: n(area(roofs, 'grossAreaM2'), 1) }), thicknessRange(roofs, locale)]
        .filter(Boolean).join(', ')];
      if (roofWindows.length > 0) facts.push(t('survey.fact.roofWindows', { count: roofWindows.length, area: n(area(roofWindows, 'areaM2'), 2),
        where: [...new Set(roofWindows.map((window) => names.get(String(window.surfaceId)) ?? ''))].join(', ').toLowerCase() }));
      if (rooflights.length > 0) facts.push(t('survey.fact.rooflights', { count: rooflights.length, area: n(area(rooflights, 'areaM2'), 2) }));
      for (const floor of floors) {
        const boundary = (floor.boundary as { kind?: string } | undefined)?.kind;
        facts.push([t('survey.fact.floor', { area: n(floor.grossAreaM2, 1), boundary: boundary ? (label(`opname.surface.boundaryKind.${boundary}`) ?? boundary).toLowerCase() : '—' }),
          thicknessRange([floor], locale)].filter(Boolean).join(', '));
      }
      return facts;
    }
    case 'verwarming': {
      const kind = read(draft, ['heating', 'generator', 'kind']);
      const source = read(draft, ['heating', 'generator', 'source']);
      const evidence = read(draft, ['heating', 'generator', 'highEfficiencyEvidence']);
      const emitters = read(draft, ['heating', 'emitters']);
      const design = read(draft, ['heating', 'designClass']);
      const distribution = read(draft, ['heating', 'distributionType', 'kind']);
      const balanced = read(draft, ['heating', 'balanced']);
      const control = read(draft, ['heating', 'control']);
      const unheated = read(draft, ['heating', 'unheatedPipes', 'kind']);
      const facts = [[kind ? t(`opname.heating.kind.${String(kind)}`) : '—',
        kind === 'heat_pump' && source ? (label(`opname.heating.hpSource.${String(source)}`) ?? String(source)).toLowerCase() : null,
        evidence ? t('survey.fact.table928') : null].filter(Boolean).join(', ')];
      if (emitters) facts.push([t(`opname.heating.emitters.${String(emitters)}`), design ? label(`opname.heating.designClassKind.${String(design)}`) : null,
        distribution ? label(`opname.heating.distributionTypeKind.${String(distribution)}`)?.toLowerCase() : null].filter(Boolean).join(', '));
      const regime = [balanced === true ? t('survey.fact.balanced') : balanced === false ? t('survey.fact.notBalanced') : null,
        control && control !== 'unknown' ? label(`survey.heating.controlKind.${String(control)}`)?.toLowerCase() : null].filter(Boolean);
      if (regime.length > 0) facts.push(regime.join(', '));
      if (unheated) facts.push(unheated === 'absent' ? t('survey.fact.noUnheatedPipes') : t('survey.fact.unheatedPipes'));
      return facts;
    }
    case 'warm-water': {
      const kind = read(draft, ['hotWater', 'generator', 'kind']);
      const served = read(draft, ['hotWater', 'served']);
      const kitchen = read(draft, ['hotWater', 'kitchenLengthM']);
      const bathroom = read(draft, ['hotWater', 'bathroomLengthM']);
      const showers = read(draft, ['hotWater', 'showers']);
      const recovery = read(draft, ['hotWater', 'showerHeatRecovery']);
      const facts = [kind ? t(`opname.hotWater.kind.${String(kind)}`) : '—'];
      if (served) facts.push([label(`survey.hotWater.servedKind.${String(served)}`),
        typeof kitchen === 'number' || typeof bathroom === 'number'
          ? t('survey.fact.pipeLengths', { kitchen: n(kitchen, 1), bathroom: n(bathroom, 1) }) : null].filter(Boolean).join(', '));
      if (typeof showers === 'number') facts.push([t('survey.fact.showers', { count: showers }),
        recovery && recovery !== 'none' && recovery !== 'unknown' ? t('survey.fact.dwtw', { kind: (label(`survey.hotWater.showerHeatRecoveryKind.${String(recovery)}`) ?? '').toLowerCase() }) : null]
        .filter(Boolean).join(' '));
      return facts;
    }
    case 'ventilatie': {
      const principle = read(draft, ['ventilation', 'principle']);
      const recovery = read(draft, ['ventilation', 'heatRecovery']);
      const motor = read(draft, ['ventilation', 'motor']);
      const unitYear = read(draft, ['ventilation', 'unitManufactureYear']);
      const ducts = read(draft, ['ventilation', 'ductAirtightness']);
      const facts = [[principle ? t(`survey.ventilation.principleKind.${String(principle)}`) : '—',
        principle === 'balanced' ? (recovery ? label(`opname.ventilation.heatRecoveryKind.${String(recovery)}`) : t('survey.fact.noRecovery')) : null]
        .filter(Boolean).join(', ')];
      const unit = [motor && motor !== 'unknown' ? label(`survey.ventilation.motorKind.${String(motor)}`) : null,
        typeof unitYear === 'number' ? t('survey.fact.unitYear', { year: String(unitYear) }) : null].filter(Boolean);
      if (unit.length > 0) facts.push(unit.join(', '));
      if (ducts && ducts !== 'unknown') facts.push(t('survey.fact.ducts', { kind: label(`survey.ventilation.ductsKind.${String(ducts)}`) ?? String(ducts) }));
      return facts;
    }
    case 'koeling': return [read(draft, ['cooling']) != null ? t('survey.fact.cooling') : t('survey.fact.noCooling')];
    case 'zonnepanelen': {
      const pv = list(['pv']);
      if (pv.length === 0) return [t('survey.fact.noPv')];
      return pv.flatMap((system) => [
        [t('survey.fact.pvArea', { area: n(system.panelAreaM2, 1) }), label(`opname.pv.module.${String(system.moduleType)}`)?.toLowerCase(),
          typeof system.installationYear === 'number' ? String(system.installationYear) : null].filter(Boolean).join(', '),
        [typeof system.azimuthDeg === 'number' ? label(`opname.orientationKind.${compass(system.azimuthDeg)}`) : null,
          typeof system.tiltDeg === 'number' ? `${n(system.tiltDeg)}°` : null,
          system.mounting && system.mounting !== 'unknown' ? label(`survey.pv.mountingKind.${String(system.mounting)}`)?.toLowerCase() : null].filter(Boolean).join(', '),
      ]);
    }
    default: return [];
  }
}

/** The step of an applied default: as its path, except the crawl-space rules (floor question). */
export function defaultStep(item: { path: string; rule: string }, stored: Stored): SurveyStepId {
  if (item.rule.startsWith('crawlspace')) return 'dak-vloer';
  return questionForPath(item.path, stored).step;
}

/** The defaults of one step, identical values on several surfaces taken together ("Gevel 1–7"). */
/** Common reasons for falling back on a default (BRL 9500-W §4.2.2), offered as suggestions. */
const REASON_KEYS = ['notVisible', 'noDocuments', 'destructive', 'clientUnknown'];

export function StepDefaults({ items, draft, t, locale, plain = false, onReason }: {
  items: OpnameAssessment['appliedDefaults']; draft: Draft; t: T; locale: string;
  /** The list without the fold (report). */
  plain?: boolean;
  /** Sets the reason for the defaults at these paths; without it the reasons are only shown. */
  onReason?: (paths: string[], reason: string) => void;
}) {
  const reasons = (read(draft, ['inklapRedenen']) as Record<string, string> | undefined) ?? {};
  const names = surfaceNames(draft, t);
  const surfaces = (read(draft, ['envelope', 'surfaces']) as Array<Record<string, unknown>> | undefined) ?? [];
  const groups: Array<{ subjects: string[]; value: string; rule: string; paths: string[]; reason: string }> = [];
  for (const item of items) {
    const surface = /^envelope\.surfaces\[(\d+)\]/.exec(item.path);
    const subject = surface ? names.get(String(surfaces[Number(surface[1])]?.id ?? '')) ?? null : null;
    const label = t(`survey.rule.${item.rule}`);
    const fallback = label !== `survey.rule.${item.rule}` ? label : defaultPathLabel(t, item.path);
    const group = groups.find((candidate) => candidate.value === item.value && candidate.rule === item.rule);
    const reason = item.inklapReden ?? reasons[item.path] ?? reasons[item.rule] ?? '';
    if (group && subject) { group.subjects.push(subject); group.paths.push(item.path); }
    else groups.push({ subjects: [subject ?? fallback], value: item.value, rule: item.rule, paths: [item.path], reason });
  }
  if (items.length === 0) return <p className="survey-defaults-none">✓ {t('survey.check.noDefaults')}</p>;
  const subjectText = (subjects: string[]) => {
    if (subjects.length < 3) return subjects.join(', ');
    const [word] = subjects[0].split(' ');
    const sameKind = subjects.every((subject) => subject.startsWith(`${word} `));
    return sameKind ? `${word} ${subjects[0].slice(word.length + 1)}–${subjects[subjects.length - 1].slice(word.length + 1)}` : subjects.join(', ');
  };
  const rows = groups.map((group, index) => <li key={index}>{subjectText(group.subjects)}: {defaultValueLabel(t, group.value, locale)}
    {plain && <span className="survey-report-reason"> — {group.reason || t('survey.reason.missing')}</span>}
    {!plain && onReason && <input className="survey-reason" type="text" list="survey-reasons" defaultValue={group.reason}
      aria-label={t('survey.reason.label', { subject: subjectText(group.subjects) })} placeholder={t('survey.reason.placeholder')}
      onBlur={(event) => { if (event.target.value !== group.reason) onReason(group.paths, event.target.value.trim()); }} />}
  </li>);
  const missing = groups.filter((group) => !group.reason).length;
  if (plain) return <ul className="survey-report-defaults">{rows}</ul>;
  return <details className="survey-step-defaults">
    <summary>{t(items.length === 1 ? 'survey.check.oneDefault' : 'survey.check.stepDefaults', { count: items.length })}
      {onReason && missing > 0 && <span className="survey-reason-open"> · {t('survey.reason.open', { count: missing })}</span>}</summary>
    <datalist id="survey-reasons">{REASON_KEYS.map((key) => <option key={key} value={t(`survey.reason.${key}`)} />)}</datalist>
    <ul>{rows}</ul>
  </details>;
}

function ResultCard({ result, busy, error, t, locale, onIssues }: {
  result: OpnameAssessment | null; busy: boolean; error: string | null; t: T; locale: string;
  /** Opens the list of points (Controle). */
  onIssues?: () => void;
}) {
  const performance = result?.performance;
  const label = performance?.indicativeLabelClass ?? null;
  return <div className="survey-card" aria-live="polite">
    <span className="survey-overline">{t('survey.result.title')}</span>
    {label ? <div className="survey-label-row">
      <span className="survey-label" style={{ background: labelColor(label) }}>{label}</span>
      <span>{t('survey.result.ep2')}<br /><strong>{formatNumber(performance?.primaryFossilIndicatorKwhPerM2Year, locale, 1)}</strong> {t('unit.kwhPerM2Year')}</span>
    </div> : <p className="survey-muted">{busy ? t('survey.result.busy') : error ? `${t('survey.result.error')} (${error})` : t('survey.result.none')}</p>}
    {result && result.issues.length > 0 && (onIssues
      ? <button type="button" className="survey-issues-note survey-issues-link" onClick={onIssues}>{t('survey.result.issues', { count: result.issues.length })} ›</button>
      : <p className="survey-issues-note">{t('survey.result.issues', { count: result.issues.length })}</p>)}
    <p className="survey-muted">{t('survey.result.note')}</p>
  </div>;
}

/** Everything on one page: the outcome, a card per step, the kernel notices and the defaults. */
/** "EDR-straat 25, 3013 AL Rotterdam" from the project data; null when empty. */
export function projectAddress(project: { address?: string; city?: string; registration?: { postcode?: string } }): string | null {
  const place = [project.registration?.postcode, project.city].filter((part) => part && part.trim()).join(' ');
  const text = [project.address?.trim(), place].filter(Boolean).join(', ');
  return text || null;
}

function CheckPage({ stored, steps, result, onEdit, onGoToPath, onReason, t, locale }: {
  stored: Stored; steps: SurveyFlowStep[]; result: OpnameAssessment | null;
  onEdit: (step: SurveyStepId) => void; onGoToPath: (path: string) => void;
  onReason: (paths: string[], reason: string) => void; t: T; locale: string;
}) {
  const draft = stored.survey as Draft;
  const address = projectAddress(useEnergy().state.project) ?? undefined;
  const performance = result?.performance;
  const issuesFor = (step: SurveyStepId) => result?.issues.filter((item) => questionForPath(item.path, stored).step === step).length ?? 0;
  return <div className="survey-check">
    <section className="survey-card survey-outcome" aria-label={t('survey.check.outcome')}>
      {performance?.indicativeLabelClass
        ? <span className="survey-label survey-label--big" style={{ background: labelColor(performance.indicativeLabelClass) }}>
          {performance.indicativeLabelClass}</span>
        : <span className="survey-muted">{t('survey.result.none')}</span>}
      <dl className="survey-indicators">
        <div><dt>{t('survey.result.ep2')}</dt><dd>{formatNumber(performance?.primaryFossilIndicatorKwhPerM2Year, locale, 1)}</dd></div>
        <div><dt>BENG 1</dt><dd>{formatNumber(performance?.needIndicatorKwhPerM2Year, locale, 1)}</dd></div>
        <div><dt>BENG 3</dt><dd>{formatNumber(performance?.renewableSharePercent, locale, 1)} %</dd></div>
        <div><dt>{t('survey.check.defaults')}</dt><dd>{result?.appliedDefaults.length ?? '—'}</dd></div>
      </dl>
    </section>

    <div className="survey-check-grid">
      {steps.filter((step) => !step.special).map((step) => {
        const state = stepState(step, stored.progress);
        const issues = issuesFor(step.id);
        return <section key={step.id} className="survey-card survey-check-card" aria-label={t(step.labelKey)}>
          <div className="survey-check-head">
            <h2>{t(step.labelKey)}</h2>
            <button type="button" className="btn btn-sm" onClick={() => onEdit(step.id)}>{t('survey.check.edit')}</button>
          </div>
          <ul>{stepFacts(step.id, draft, t, locale, address).map((fact, index) => <li key={index}>{fact}</li>)}</ul>
          {result && <StepDefaults items={result.appliedDefaults.filter((item) => defaultStep(item, stored) === step.id)} draft={draft} t={t} locale={locale}
            onReason={onReason} />}
          {state === 'skipped' && <p className="survey-skipped-note">{t('survey.check.skipped')}</p>}
          {issues > 0 && <p className="survey-issues-note">{t('survey.result.issues', { count: issues })}</p>}
        </section>;
      })}
    </div>

    {result && result.issues.length > 0 && <section className="survey-card" aria-label={t('survey.check.notices')}>
      <h2>{t('survey.check.notices')}</h2>
      <ul className="survey-notices">
        {result.issues.map((item, index) => {
          const target = questionForPath(item.path, stored);
          return <li key={index}>
            <span><KernelCode code={item.code} prefixes={['opname.issue.', 'nta.gap.', 'kernel.issue.']} hideCode />
              {issueSubject(item.path, draft, t) && <span className="survey-muted"> · {issueSubject(item.path, draft, t)}</span>}</span>
            <button type="button" className="btn btn-sm" onClick={() => onGoToPath(item.path)}>
              {t('survey.check.goTo', { step: t(`survey.step.${target.step}`) })}</button>
          </li>;
        })}
      </ul>
    </section>}

    {result && result.appliedDefaults.length > 0 && <details className="survey-card survey-defaults">
      <summary>{t('survey.check.defaultsList', { count: result.appliedDefaults.length })}</summary>
      <table>
        <thead><tr><th>{t('opname.defaults.path')}</th><th>{t('opname.defaults.value')}</th></tr></thead>
        <tbody>{result.appliedDefaults.map((item, index) => <tr key={index}>
          <td>{defaultPathLabel(t, item.path)}{issueSubject(item.path, draft, t) && <span className="survey-muted"> · {issueSubject(item.path, draft, t)}</span>}</td>
          <td>{defaultValueLabel(t, item.value, locale)}</td>
        </tr>)}</tbody>
      </table>
    </details>}

    <details className="survey-card survey-all-fields">
      <summary>{t('survey.check.allFields')}</summary>
      <BasisopnamePanel />
    </details>
  </div>;
}

export function SurveyWizard({ route, navigate }: SurveyWizardProps) {
  const { t, locale } = useI18n();
  const { state, dispatch } = useEnergy();
  const stored = state.project.basisopname as Stored | undefined;
  const assessment = useSurveyAssessment();

  const kind: SurveyKind = stored?.kind ?? 'residential';
  const steps = useMemo(() => surveySteps(kind), [kind]);
  const stepId = stepForKind(route.sub, kind);
  const step = surveyStep(stepId, kind);
  const stepIndex = steps.findIndex((candidate) => candidate.id === step.id);
  const surveyJson = stored ? JSON.stringify(stored.survey) : null;

  // Recalculate after each change, once the typing pauses.
  useEffect(() => {
    if (!stored) return undefined;
    const timer = window.setTimeout(() => { void assessSurvey(stored); }, 400);
    return () => window.clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [surveyJson]);

  // The question is part of the route, so the navigation can show it; "Ga naar" lands on the
  // question of the kernel path. A step without a question starts at its first.
  const requestedQuestion = route.question
    ?? (stored && route.focusPath ? questionForPath(route.focusPath, stored).question : undefined);
  const questionIndex = Math.max(0, step.questions.findIndex((question) => question.id === requestedQuestion));
  // A question left after a change counts as answered, also when left through the navigation.
  const latest = useRef(stored);
  latest.current = stored;
  const visitedKey = !step.special && step.questions.length > 0
    ? questionKey(step.id, step.questions[Math.min(questionIndex, step.questions.length - 1)].id) : null;
  useEffect(() => {
    if (!visitedKey || !latest.current) return undefined;
    const entry = JSON.stringify(latest.current.survey);
    return () => {
      const now = latest.current;
      if (!now || JSON.stringify(now.survey) === entry || questionState(now.progress, visitedKey) !== 'todo') return;
      dispatch({ type: 'SET_BASISOPNAME', payload: { ...now, progress: markProgress(now.progress, visitedKey, 'done') } });
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [visitedKey]);

  const setQuestionIndex = (index: number) =>
    navigate({ step: 'survey', sub: step.id, question: step.questions[index]?.id });

  if (!stored) {
    const start = (surveyKind: SurveyKind) => {
      dispatch({ type: 'SET_BASISOPNAME', payload: { ...surveyTemplate(surveyKind), progress: {} } });
      navigate({ step: 'survey', sub: surveySteps(surveyKind)[0].id });
    };
    return <div className="survey-start">
      <h1>{t('survey.start.title')}</h1>
      <p className="survey-muted">{t('survey.start.lead')}</p>
      <div className="survey-choices">
        <button type="button" className="survey-choice" onClick={() => start('residential')}>
          <span className="survey-choice-icon" aria-hidden="true"><Home /></span>
          <span className="survey-choice-title">{t('survey.start.residential')}</span>
          <span className="survey-choice-hint">{t('survey.start.residentialHint')}</span>
        </button>
        <button type="button" className="survey-choice" onClick={() => start('utility')}>
          <span className="survey-choice-icon" aria-hidden="true"><Building2 /></span>
          <span className="survey-choice-title">{t('survey.start.utility')}</span>
          <span className="survey-choice-hint">{t('survey.start.utilityHint')}</span>
        </button>
      </div>
    </div>;
  }

  const draft = stored.survey as Draft;
  const result = currentResult(assessment, stored);
  const save = (next: Partial<Stored>) => dispatch({ type: 'SET_BASISOPNAME', payload: { ...stored, ...next } });
  const change = (path: Path, value: unknown) => save({ survey: write(draft, path, value) });
  const goToStep = (id: SurveyStepId) => navigate({ step: 'survey', sub: id });
  const goToPath = (path: string) => {
    const target = questionForPath(path, stored);
    navigate({ step: 'survey', sub: target.step, focusPath: `basisopname.${path.replace(/^basisopname\./, '')}` });
  };

  const question = step.questions[Math.min(questionIndex, Math.max(0, step.questions.length - 1))];
  const lastQuestion = questionIndex >= step.questions.length - 1;
  const nextStep = steps[stepIndex + 1];
  const previousStep = steps[stepIndex - 1];

  const advance = (mark: 'done' | 'skipped' | null) => {
    let progress = stored.progress;
    if (mark) progress = markProgress(progress, step.special ? step.id : questionKey(step.id, question.id), mark);
    if (progress !== stored.progress) save({ progress });
    if (!step.special && !lastQuestion) setQuestionIndex(questionIndex + 1);
    else if (nextStep) goToStep(nextStep.id);
  };
  const back = () => {
    if (!step.special && questionIndex > 0) setQuestionIndex(questionIndex - 1);
    // Back from the first question lands on the last question of the previous step.
    else if (previousStep) navigate({ step: 'survey', sub: previousStep.id, question: previousStep.questions[previousStep.questions.length - 1]?.id });
  };
  const nextLabel = !step.special && !lastQuestion
    ? t('survey.nav.nextQuestion')
    : nextStep ? t('survey.nav.next', { step: t(nextStep.labelKey).toLowerCase() }) : t('survey.nav.finish');

  const open = steps.filter((candidate) => candidate.id !== step.id && !candidate.special
    && stepState(candidate, stored.progress) !== 'done');

  return <div className="survey-wizard">
    <div className="survey-main">
      <p className="survey-crumb">{t('survey.crumb', {
        number: stepIndex + 1, total: steps.length, step: t(step.labelKey),
      })}{!step.special && step.questions.length > 1 && ` · ${t('survey.crumbQuestion', { number: questionIndex + 1, total: step.questions.length })}`}</p>

      {step.special === 'check' && <>
        <h1>{t('survey.check.title')}</h1>
        <p className="survey-lead">{t('survey.check.lead')}</p>
        <CheckPage stored={stored} steps={steps} result={result} onEdit={goToStep} onGoToPath={goToPath} t={t} locale={locale}
          onReason={(paths, reason) => {
            const map = { ...((draft.inklapRedenen as Record<string, string> | undefined) ?? {}) };
            for (const path of paths) { if (reason) map[path] = reason; else delete map[path]; }
            change(['inklapRedenen'], map);
          }} />
      </>}

      {step.special === 'label' && <>
        <h1>{t('survey.label.title')}</h1>
        <p className="survey-lead">{t('survey.label.lead')}</p>
        <section className="survey-card survey-final">
          <div className="survey-final-top">
            {result?.performance?.indicativeLabelClass
              ? <span className="survey-label survey-label--big" style={{ background: labelColor(result.performance.indicativeLabelClass) }}>
                {result.performance.indicativeLabelClass}</span>
              : <span className="survey-muted">{t('survey.result.none')}</span>}
            <dl className="survey-indicators survey-indicators--units">
              <div><dt>{t('survey.result.ep2')}</dt><dd>{formatNumber(result?.performance?.primaryFossilIndicatorKwhPerM2Year, locale, 1)} <small>kWh/m²</small></dd></div>
              <div><dt>{t('survey.final.beng1')}</dt><dd>{formatNumber(result?.performance?.needIndicatorKwhPerM2Year, locale, 1)} <small>kWh/m²</small></dd></div>
              <div><dt>{t('survey.final.beng3')}</dt><dd>{formatNumber(result?.performance?.renewableSharePercent, locale, 1)} <small>%</small></dd></div>
              <div><dt>TOjuli</dt><dd>{formatNumber(result?.performance?.tojuliMaxK, locale, 2)} <small>K</small></dd></div>
            </dl>
          </div>
          <p className="survey-final-status">{t('survey.final.status')}</p>
        </section>
        <div className="survey-final-actions">
          <section className="survey-card">
            <h2>{t('survey.label.report')}</h2>
            <p className="survey-muted">{t('survey.final.reportHint')}</p>
            <button type="button" className="btn btn-primary" onClick={() => navigate({ step: 'report' })}>{t('survey.final.report')}</button>
          </section>
          <section className="survey-card">
            <h2>{t('survey.label.advice')}</h2>
            <p className="survey-muted">{t('survey.final.adviceHint')}</p>
            <button type="button" className="btn" onClick={() => navigate({ step: 'advice' })}>{t('survey.final.advice')}</button>
          </section>
          <section className="survey-card">
            <h2>{t('survey.label.registration')}</h2>
            <p className="survey-muted">{t('survey.final.registrationHint')}</p>
            <button type="button" className="btn" onClick={() => navigate({ step: 'registration' })}>{t('survey.final.registration')}</button>
          </section>
        </div>
      </>}

      {!step.special && question && <>
        <h1>{t(question.titleKey)}</h1>
        <p className="survey-lead">{t(question.helpKey)}</p>
        <div className="survey-question" key={`${step.id}.${question.id}`}>
          <QuestionBody part={question.part} stored={stored} draft={draft} change={change} t={t} />
        </div>
        {questionState(stored.progress, questionKey(step.id, question.id)) === 'skipped' &&
          <p className="survey-skipped-note">{t('survey.skippedNote')}</p>}
      </>}

      {step.special !== 'label' && <div className="survey-actions">
        <button type="button" className="btn" onClick={back} disabled={stepIndex === 0 && questionIndex === 0}>{t('survey.nav.previous')}</button>
        <div className="survey-actions-right">
          {!step.special && <button type="button" className="btn survey-skip" onClick={() => advance('skipped')}>{t('survey.nav.skip')}</button>}
          <button type="button" className="btn btn-primary" onClick={() => advance('done')}>{nextLabel}</button>
        </div>
      </div>}
    </div>

    <aside className="survey-aside" aria-label={t('survey.aside')}>
      {step.special === 'label' ? <>
        <div className="survey-card">
          <span className="survey-overline">{t('survey.final.survey')}</span>
          <p className="survey-final-facts">{[
            typeof draft.id === 'string' && draft.id.trim() ? draft.id : null,
            stored.surveyDate ? t('survey.final.date', { date: new Date(stored.surveyDate).toLocaleDateString(locale) }) : t('survey.final.noDate'),
            typeof draft.sourceReference === 'string' && draft.sourceReference.trim() ? t('survey.final.source', { source: draft.sourceReference }) : null,
          ].filter(Boolean).map((line, index) => <span key={index}>{line}<br /></span>)}</p>
        </div>
        <div className="survey-card">
          <span className="survey-overline">{t('survey.final.dossier')}</span>
          <p className="survey-final-facts">
            {open.length === 0 ? <span className="survey-done">✓ {t('survey.final.allAnswered')}</span> : t('survey.final.openSteps', { count: open.length })}<br />
            {t('survey.check.stepDefaults', { count: result?.appliedDefaults.length ?? 0 })}<br />
            {result && result.issues.length > 0 ? t('survey.result.issues', { count: result.issues.length }) : t('survey.final.noIssues')}
          </p>
        </div>
      </> : <>
      <ResultCard result={result} busy={assessment.busy} error={assessment.error} t={t} locale={locale}
        onIssues={step.special === 'check' ? undefined : () => goToStep('controle')} />
      <div className="survey-card">
        <span className="survey-overline">{t('survey.open.title')}</span>
        {open.length === 0 ? <p className="survey-done">{t('survey.open.none')}</p> : <ul className="survey-open">
          {open.map((candidate) => {
            const candidateState = stepState(candidate, stored.progress);
            return <li key={candidate.id}>
              <button type="button" onClick={() => goToStep(candidate.id)}>
                <span>{t(candidate.labelKey)}</span>
                <span className={`survey-open-tag survey-open-tag--${candidateState}`}>{t(`survey.open.${candidateState}`)}</span>
              </button>
            </li>;
          })}
        </ul>}
      </div>
      <p className="survey-defaults-note">{t('survey.defaultsNote')}</p>
      </>}
    </aside>
  </div>;
}
