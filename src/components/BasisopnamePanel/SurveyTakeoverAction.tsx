import { useMemo, useState } from 'react';
import { useI18n } from '../../i18n/i18n';
import { changedPaths, useNtaDraft } from '../../context/NtaDraftProvider';
import type { OpnameAssessment } from '../../core/nta/KernelClient';
import { parseKernelPath } from '../../core/nta/pathUtil';
import { canTakeOver, previewValue, surveyTakeover } from '../../core/nta/SurveyTakeover';
import { read, type Draft } from '../NtaPerformancePanel/NtaFormFields';
import { Button, Dialog } from '../ui';

/** Rows shown in the preview; the rest is counted. */
const MAX_ROWS = 60;

/**
 * "Overnemen in projectmodel": writes the NTA input derived from the survey
 * into the shared NTA draft after a preview of every changed value. Nothing
 * is applied here; the apply bar ("Toepassen") does that, and undo works.
 */
export function SurveyTakeoverAction({ result }: { result: OpnameAssessment }) {
  const { t } = useI18n();
  const shared = useNtaDraft();
  const [open, setOpen] = useState(false);
  const [done, setDone] = useState(false);
  const available = shared != null && canTakeOver(result);
  const current = (shared?.draft ?? shared?.base ?? null) as Draft | null;
  const preview = useMemo(() => {
    if (!open || !canTakeOver(result)) return null;
    const takeover = surveyTakeover(result.derivedInput, current);
    const changes = changedPaths(current ?? {}, takeover.next).map((path) => {
      const segments = parseKernelPath(path);
      return { path, from: read(current ?? {}, segments), to: read(takeover.next, segments) };
    });
    return { ...takeover, changes };
  }, [open, result, current]);

  if (!shared) return null;
  return (
    <div className="opname-takeover">
      <Button variant="secondary" size="sm" disabled={!available} onClick={() => { setDone(false); setOpen(true); }}>
        {t('opname.takeover.button')}
      </Button>
      {!available && <small className="nta-form-note"> {t('opname.takeover.unavailable')}</small>}
      {done && <p className="nta-form-note" role="status">{t('opname.takeover.done')}</p>}
      {open && preview && <Dialog title={t('opname.takeover.title')} onClose={() => setOpen(false)} width={720}
        footer={<>
          <Button variant="ghost" onClick={() => setOpen(false)}>{t('opname.takeover.cancel')}</Button>
          <Button variant="primary" disabled={preview.changes.length === 0} onClick={() => {
            if (!canTakeOver(result)) return;
            const derived = result.derivedInput;
            shared.update((draft) => surveyTakeover(derived, draft).next as Draft);
            setOpen(false);
            setDone(true);
          }}>{t('opname.takeover.confirm')}</Button>
        </>}>
        <p>{t('opname.takeover.intro')}</p>
        <p><strong>{t('opname.takeover.count', { count: preview.changes.length })}</strong></p>
        {preview.changes.length === 0
          ? <p className="nta-form-note">{t('opname.takeover.noChanges')}</p>
          : <table className="opname-takeover__diff" aria-label={t('opname.takeover.title')}>
            <thead><tr>
              <th scope="col">{t('opname.takeover.path')}</th>
              <th scope="col">{t('opname.takeover.from')}</th>
              <th scope="col">{t('opname.takeover.to')}</th>
            </tr></thead>
            <tbody>
              {preview.changes.slice(0, MAX_ROWS).map((row) => <tr key={row.path}>
                <td><code>{row.path}</code></td>
                <td>{previewValue(row.from)}</td>
                <td>{previewValue(row.to)}</td>
              </tr>)}
            </tbody>
          </table>}
        {preview.changes.length > MAX_ROWS &&
          <p className="nta-form-note">{t('opname.takeover.more', { count: preview.changes.length - MAX_ROWS })}</p>}
        <h4>{t('opname.takeover.skipped')}</h4>
        <ul className="opname-takeover__skipped">
          {[...new Set(preview.skipped.map((part) => part.reason))].map((reason) =>
            <li key={reason}>{t(`opname.takeover.skipped.${reason}`)}</li>)}
        </ul>
      </Dialog>}
    </div>
  );
}
