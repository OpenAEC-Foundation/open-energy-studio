import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { FileButton } from '../../ui';
import type { NtaEvidenceItem, NtaEvidenceKind, NtaRelabelProof } from '../../../core/nta/KernelClient';

/** Roles of a file in a relabel (BRL 9500-W §4.2.3, p. 23). */
const RELABEL_PROOFS: NtaRelabelProof[] = ['quote_with_order', 'specified_invoice', 'production_photo', 'review'];
import {
  checkEvidenceFile, createEvidenceItem, EVIDENCE_ACCEPT, EVIDENCE_KINDS, EVIDENCE_REFERENCE_PREFIX,
  forgetSessionEvidence, releaseEvidenceIds,
} from '../../../core/nta/Evidence';
import { evidenceErrorText } from '../../EvidenceLink/EvidenceLink';
import { canHoldSnapshot } from '../../../core/nta/registerSnapshot';
import { RegisterSnapshotFields } from './RegisterSnapshotFields';

interface EvidenceRegisterProps {
  evidence: NtaEvidenceItem[];
  onChange: (evidence: NtaEvidenceItem[]) => void;
}

/** Parses "lat, lon"; `undefined` unless both are finite numbers. */
export function parseGps(text: string): NtaEvidenceItem['gps'] {
  const parts = text.split(/[,;\s]+/).filter(Boolean).map(Number);
  return parts.length === 2 && parts.every(Number.isFinite) ? { latitude: parts[0], longitude: parts[1] } : undefined;
}

/** Parses "a, b": the trimmed, non-empty paths. */
export function parsePaths(text: string): string[] {
  return text.split(',').map((path) => path.trim()).filter(Boolean);
}

/**
 * A text field for a parsed value: keeps what the adviser types (a trailing
 * comma, a half-typed coordinate) and shows the value again when it changes
 * from outside (Verwerpen, undo), review 35.
 */
function ParsedTextField<T>({ id, label, placeholder, value, format, parse, onChange }: {
  id: string; label: string; placeholder: string; value: T;
  format: (value: T) => string; parse: (text: string) => T; onChange: (value: T) => void;
}) {
  const [text, setText] = useState(() => format(value));
  const shown = JSON.stringify(parse(text)) === JSON.stringify(value) ? text : format(value);
  return (
    <div className="dialog-field">
      <label htmlFor={id}>{label}</label>
      <input id={id} type="text" placeholder={placeholder} value={shown}
        onChange={(event) => { setText(event.target.value); onChange(parse(event.target.value)); }} />
    </div>
  );
}

const formatGps = (gps: NtaEvidenceItem['gps']) => (gps ? `${gps.latitude}, ${gps.longitude}` : '');
const formatPaths = (paths: string[]) => paths.join(', ');

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
      const chosen = Array.from(files);
      // Size and type first, so a huge or unsupported file is never read.
      for (const file of chosen) checkEvidenceFile({ name: file.name, size: file.size });
      let next = [...evidence];
      for (const file of chosen) {
        const bytes = new Uint8Array(await file.arrayBuffer());
        next = [...next, await createEvidenceItem({ name: file.name, bytes, lastModified: file.lastModified }, next)];
      }
      onChange(next);
      releaseEvidenceIds(next.slice(evidence.length).map((item) => item.id));
    } catch (reason) {
      setError(evidenceErrorText(reason, t));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="evidence-register">
      <p className="dialog-hint">{t('evidence.hint', { prefix: EVIDENCE_REFERENCE_PREFIX })}</p>
      <div className="dialog-field">
        <label htmlFor="evidence-add">{t('evidence.add')}</label>
        <FileButton id="evidence-add" multiple disabled={busy} accept={EVIDENCE_ACCEPT}
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
          <ParsedTextField id={`evidence-${item.id}-links`} label={t('evidence.linkedPaths')} placeholder="/zones/0/surfaces/1"
            value={item.linkedPaths ?? []} format={formatPaths} parse={parsePaths}
            onChange={(linkedPaths) => update(index, { linkedPaths })} />
          <ParsedTextField id={`evidence-${item.id}-gps`} label={t('evidence.gps')} placeholder="52.37, 4.89"
            value={item.gps} format={formatGps} parse={parseGps} onChange={(gps) => update(index, { gps })} />
          {(canHoldSnapshot(item) || item.registerSnapshot) && (
            <RegisterSnapshotFields item={item} onChange={(registerSnapshot) => update(index, { registerSnapshot })} />
          )}
          <button type="button" onClick={() => {
            const next = evidence.filter((_, i) => i !== index);
            onChange(next);
            // The removed file's bytes are no longer needed in this session.
            forgetSessionEvidence(item.sha256, next);
          }}>{t('evidence.remove')}</button>
        </fieldset>
      ))}
    </div>
  );
}
