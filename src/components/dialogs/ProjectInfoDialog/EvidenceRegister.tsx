import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import type { NtaEvidenceItem, NtaEvidenceKind, NtaRelabelProof } from '../../../core/nta/KernelClient';

/** Roles of a file in a relabel (BRL 9500-W §4.2.3, p. 23). */
const RELABEL_PROOFS: NtaRelabelProof[] = ['quote_with_order', 'specified_invoice', 'production_photo'];
import { createEvidenceItem, EVIDENCE_KINDS, EVIDENCE_REFERENCE_PREFIX } from '../../../core/nta/Evidence';

interface EvidenceRegisterProps {
  evidence: NtaEvidenceItem[];
  onChange: (evidence: NtaEvidenceItem[]) => void;
}

/** Parses "lat, lon"; `undefined` unless both are finite numbers. */
export function parseGps(text: string): NtaEvidenceItem['gps'] {
  const parts = text.split(/[,;\s]+/).filter(Boolean).map(Number);
  return parts.length === 2 && parts.every(Number.isFinite) ? { latitude: parts[0], longitude: parts[1] } : undefined;
}

function GpsField({ id, label, value, onChange }: {
  id: string; label: string; value: NtaEvidenceItem['gps']; onChange: (gps: NtaEvidenceItem['gps']) => void;
}) {
  const [text, setText] = useState(value ? `${value.latitude}, ${value.longitude}` : '');
  return (
    <div className="dialog-field">
      <label htmlFor={id}>{label}</label>
      <input id={id} type="text" placeholder="52.37, 4.89" value={text}
        onChange={(event) => { setText(event.target.value); onChange(parseGps(event.target.value)); }} />
    </div>
  );
}

/** BRL 9500 Bijlage 3 evidence register: files with hash, origin and check. */
export function EvidenceRegister({ evidence, onChange }: EvidenceRegisterProps) {
  const { t } = useI18n();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const update = (index: number, patch: Partial<NtaEvidenceItem>) =>
    onChange(evidence.map((item, i) => {
      if (i !== index) return item;
      const next: NtaEvidenceItem = { ...item, ...patch };
      for (const key of Object.keys(patch) as Array<keyof NtaEvidenceItem>) {
        const value = next[key];
        if (value === undefined || value === '' || (Array.isArray(value) && value.length === 0)) delete next[key];
      }
      return next;
    }));

  const addFiles = async (files: FileList | null) => {
    if (!files?.length) return;
    setBusy(true);
    setError(null);
    try {
      let next = [...evidence];
      for (const file of Array.from(files)) {
        const bytes = new Uint8Array(await file.arrayBuffer());
        next = [...next, await createEvidenceItem({ name: file.name, bytes, lastModified: file.lastModified }, next)];
      }
      onChange(next);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="evidence-register">
      <p className="dialog-hint">{t('evidence.hint', { prefix: EVIDENCE_REFERENCE_PREFIX })}</p>
      <div className="dialog-field">
        <label htmlFor="evidence-add">{t('evidence.add')}</label>
        <input id="evidence-add" type="file" multiple disabled={busy}
          onChange={(event) => { void addFiles(event.target.files); event.target.value = ''; }} />
      </div>
      {error && <p role="alert">{error}</p>}
      {evidence.length === 0 && <p className="dialog-hint">{t('evidence.empty')}</p>}
      {evidence.map((item, index) => (
        <fieldset className="evidence-item" key={item.id}>
          <legend>{`${EVIDENCE_REFERENCE_PREFIX}${item.id} — ${item.fileName}`}</legend>
          <p className="dialog-hint" title={item.sha256}>SHA-256 {item.sha256.slice(0, 16)}…{item.storedPath ? '' : ` ${t('evidence.sessionOnly')}`}</p>
          <div className="dialog-field">
            <label htmlFor={`evidence-${item.id}-kind`}>{t('evidence.kind')}</label>
            <select id={`evidence-${item.id}-kind`} value={item.kind}
              onChange={(event) => update(index, { kind: event.target.value as NtaEvidenceKind })}>
              {EVIDENCE_KINDS.map((kind) => <option key={kind} value={kind}>{t(`evidence.kind.${kind}`)}</option>)}
            </select>
          </div>
          <div className="dialog-field">
            <label htmlFor={`evidence-${item.id}-relabel`}>{t('evidence.relabelProof')}</label>
            <select id={`evidence-${item.id}-relabel`} value={item.relabelProof ?? ''}
              onChange={(event) => update(index, { relabelProof: (event.target.value || undefined) as NtaRelabelProof | undefined })}>
              <option value="">—</option>
              {RELABEL_PROOFS.map((proof) => <option key={proof} value={proof}>{t(`evidence.relabelProof.${proof}`)}</option>)}
            </select>
          </div>
          <div className="dialog-field">
            <label htmlFor={`evidence-${item.id}-date`}>{t('evidence.date')}</label>
            <input id={`evidence-${item.id}-date`} type="date" value={item.date ?? ''}
              onChange={(event) => update(index, { date: event.target.value || undefined })} />
          </div>
          <div className="dialog-field">
            <label htmlFor={`evidence-${item.id}-source`}>{t('evidence.sourceParty')}</label>
            <input id={`evidence-${item.id}-source`} type="text" value={item.sourceParty ?? ''}
              onChange={(event) => update(index, { sourceParty: event.target.value || undefined })} />
          </div>
          <div className="dialog-field">
            <label htmlFor={`evidence-${item.id}-checked`}>{t('evidence.checkedBy')}</label>
            <input id={`evidence-${item.id}-checked`} type="text" value={item.checkedBy ?? ''}
              onChange={(event) => update(index, { checkedBy: event.target.value || undefined })} />
          </div>
          <div className="dialog-field">
            <label htmlFor={`evidence-${item.id}-description`}>{t('evidence.description')}</label>
            <input id={`evidence-${item.id}-description`} type="text" value={item.description ?? ''}
              onChange={(event) => update(index, { description: event.target.value || undefined })} />
          </div>
          <div className="dialog-field">
            <label htmlFor={`evidence-${item.id}-links`}>{t('evidence.linkedPaths')}</label>
            <input id={`evidence-${item.id}-links`} type="text" placeholder="/zones/0/surfaces/1"
              value={(item.linkedPaths ?? []).join(', ')}
              onChange={(event) => update(index, {
                linkedPaths: event.target.value.split(',').map((path) => path.trim()).filter(Boolean),
              })} />
          </div>
          <GpsField id={`evidence-${item.id}-gps`} label={t('evidence.gps')} value={item.gps}
            onChange={(gps) => update(index, { gps })} />
          <button type="button" onClick={() => onChange(evidence.filter((_, i) => i !== index))}>{t('evidence.remove')}</button>
        </fieldset>
      ))}
    </div>
  );
}
