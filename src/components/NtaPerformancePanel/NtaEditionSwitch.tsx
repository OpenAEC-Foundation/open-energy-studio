import { useEffect, useState } from 'react';
import { useI18n } from '../../i18n/i18n';
import {
  AUTOMATIC_EDITION_FIXES, compareProjectEditionsWithRust, DEFAULT_NORM_VERSION, IMPLEMENTED_NORM_VERSIONS,
  type EditionSwitchItem, type EditionSwitchReport, type NormVersion,
} from '../../core/nta/KernelClient';
import { routeForPath } from '../../core/nta/gapRoutes';
import { previewValue } from '../../core/nta/SurveyTakeover';
import type { IProject } from '../../core/energy/types';
import { KernelCode } from '../KernelCode/KernelCode';
import { useShellActions } from '../shell/ShellActions';
import { Button, Dialog } from '../ui';
import { read, SelectField, write, type Draft, type Path } from './NtaFormFields';

const CATEGORIES: Array<EditionSwitchItem['category']> = ['refused', 'required', 'ignored', 'reinterpreted', 'warning'];

export interface EditionSwitchProps {
  draft: Draft;
  project: IProject;
  change: (path: Path, value: unknown) => void;
  update: (next: (current: Draft) => Draft) => void;
}

/**
 * The edition select. A change of edition first shows what the kernel then
 * refuses, newly requires, ignores or reads differently (compare_project_editions).
 * The adviser confirms; "Automatisch aanpassen" also applies the safe changes
 * (drop a refused member, switch a flag off, set the table 9.28 test standard).
 * Either way the switch is one draft update, so one undo restores everything.
 */
export function EditionSwitchField({ draft, project, change, update }: EditionSwitchProps) {
  const { t } = useI18n();
  const [target, setTarget] = useState<NormVersion | null>(null);
  const current = (read(draft, ['normVersion']) as NormVersion | undefined) ?? DEFAULT_NORM_VERSION;
  return <>
    <SelectField draft={draft} path={['normVersion']} label={t('nta.form.normVersion')} fallback={DEFAULT_NORM_VERSION}
      options={IMPLEMENTED_NORM_VERSIONS.map((edition) => [edition, t(`nta.edition.${edition}`)])}
      onChange={(_, value) => {
        const next = (value ?? DEFAULT_NORM_VERSION) as NormVersion;
        if (next !== current) setTarget(next);
      }} />
    {target && <EditionSwitchDialog draft={draft} project={project} target={target}
      onCancel={() => setTarget(null)}
      onSwitch={(adjusted) => {
        if (adjusted) update(() => adjusted);
        else change(['normVersion'], target);
        setTarget(null);
      }} />}
  </>;
}

interface DialogProps {
  draft: Draft;
  project: IProject;
  target: NormVersion;
  onCancel: () => void;
  /** With the adjusted NTA block, or null to change only the edition. */
  onSwitch: (adjusted: Draft | null) => void;
}

type Load = { state: 'loading' } | { state: 'ready'; report: EditionSwitchReport } | { state: 'unavailable' };

export function EditionSwitchDialog({ draft, project, target, onCancel, onSwitch }: DialogProps) {
  const { t, locale } = useI18n();
  const shell = useShellActions();
  const [load, setLoad] = useState<Load>({ state: 'loading' });
  // The project as it was when the edition was chosen; later renders of the
  // page must not start the comparison again.
  const [snapshot] = useState<IProject>(() => ({ ...project, ntaCalculation: draft as never }));
  useEffect(() => {
    let live = true;
    setLoad({ state: 'loading' });
    compareProjectEditionsWithRust(snapshot, target)
      .then((report) => { if (live) setLoad({ state: 'ready', report }); })
      .catch(() => { if (live) setLoad({ state: 'unavailable' }); });
    return () => { live = false; };
  }, [snapshot, target]);

  const report = load.state === 'ready' ? load.report : null;
  const automatic = report?.items.filter((item) => AUTOMATIC_EDITION_FIXES.includes(item.fix)) ?? [];
  const open = report?.remainingAfterAutomaticChanges.length ?? 0;
  const adjusted = report?.adjustedProject.ntaCalculation as Draft | undefined;
  const goTo = shell ? (path: string) => { onCancel(); shell.navigate(routeForPath(path)); } : null;
  // A field the new edition requires is often only shown in that edition
  // (the 8.47 wall height of 2022): going there first changes the edition.
  const switchAndGoTo = shell ? (path: string) => { onSwitch(null); shell.navigate(routeForPath(path)); } : null;
  const fixText = (item: EditionSwitchItem) => {
    if (item.fix === 'set') return t('nta.editionSwitch.fix.set', { value: previewValue(item.value, locale) });
    // A value the new edition asks for but that is evidence (the dated
    // standard of a test report) is named, never written.
    if (item.fix === 'choose' && item.value != null) {
      return t('nta.editionSwitch.fix.chooseValue', { value: previewValue(item.value, locale) });
    }
    return t(`nta.editionSwitch.fix.${item.fix}`);
  };

  return <Dialog title={t('nta.editionSwitch.title')} onClose={onCancel} width={760}
    footer={<>
      <Button variant="ghost" onClick={onCancel}>{t('nta.editionSwitch.cancel')}</Button>
      {/* Also while the kernel is still busy: the review is guidance, not a gate. */}
      <Button variant={automatic.length > 0 ? 'secondary' : 'primary'}
        onClick={() => onSwitch(null)}>{t('nta.editionSwitch.switchOnly')}</Button>
      {automatic.length > 0 && adjusted && <Button variant="primary"
        onClick={() => onSwitch(write(adjusted, ['normVersion'], target))}>{t('nta.editionSwitch.switchAdjusted')}</Button>}
    </>}>
    <p>{t('nta.editionSwitch.intro', {
      from: t(`nta.edition.${(read(draft, ['normVersion']) as string | undefined) ?? DEFAULT_NORM_VERSION}`),
      to: t(`nta.edition.${target}`),
    })}</p>
    {load.state === 'loading' && <p role="status">{t('nta.editionSwitch.loading')}</p>}
    {load.state === 'unavailable' && <p role="status" className="nta-form-note">{t('nta.editionSwitch.unavailable')}</p>}
    {report && <>
      {report.items.length === 0
        ? <p role="status">{t('nta.editionSwitch.none')}</p>
        : <p role="status"><strong>{t('nta.editionSwitch.summary', { count: report.items.length, automatic: automatic.length, open })}</strong></p>}
      {CATEGORIES.map((category) => {
        const items = report.items.filter((item) => item.category === category);
        if (items.length === 0) return null;
        return <section key={category} className="nta-edition-switch__group" aria-labelledby={`edition-switch-${category}`}>
          <h4 id={`edition-switch-${category}`}>{t(`nta.editionSwitch.category.${category}`)}</h4>
          <ul className="nta-edition-switch__list">
            {items.map((item) => <li key={`${item.code} ${item.path}`}>
              <KernelCode code={item.code} prefixes={['nta.gap.', 'nta.warning.', 'kernel.issue.']} />
              {' '}<code>{item.path}</code>
              {item.points.length > 0 && <small> {t('nta.editionSwitch.point', { points: item.points.join(', '), subject: item.subject ?? '' })}</small>}
              <div className="nta-form-note">
                {fixText(item)}
                {item.resolvedByAutomaticChanges && !AUTOMATIC_EDITION_FIXES.includes(item.fix) && <> {t('nta.editionSwitch.resolved')}</>}
                {item.category === 'required'
                  ? switchAndGoTo && <> <button type="button" className="ui-btn ui-btn--ghost ui-btn--sm"
                    aria-label={t('nta.editionSwitch.switchAndGoToItem', { path: item.path })}
                    onClick={() => switchAndGoTo(item.path)}>{t('nta.editionSwitch.switchAndGoTo')}</button></>
                  : goTo && <> <button type="button" className="ui-btn ui-btn--ghost ui-btn--sm"
                    aria-label={t('nta.editionSwitch.goToItem', { path: item.path })}
                    onClick={() => goTo(item.path)}>{t('nta.editionSwitch.goTo')}</button></>}
              </div>
            </li>)}
          </ul>
        </section>;
      })}
      {report.switchPoints.some((point) => !point.hasInputs) && <details>
        <summary>{t('nta.editionSwitch.valuesOnly', { count: report.switchPoints.filter((point) => !point.hasInputs).length })}</summary>
        <ul>
          {report.switchPoints.filter((point) => !point.hasInputs).map((point) =>
            <li key={point.points.join(',')}>{t('nta.editionSwitch.point', { points: point.points.join(', '), subject: point.subject })}</li>)}
        </ul>
      </details>}
      <p className="nta-form-note">{t('nta.editionSwitch.undoNote')}</p>
    </>}
  </Dialog>;
}
