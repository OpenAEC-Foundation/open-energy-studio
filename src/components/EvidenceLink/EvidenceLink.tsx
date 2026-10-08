/**
 * "Bron & bewijs" controls (BRL 9500 Bijlage 3): attach a file to an input or
 * element and link it, or link a file already in the evidence register. A
 * file added here goes into `registration.evidence` with its SHA-256 like any
 * register entry, so it lands in the dossier ZIP and its manifest.
 *
 * `EvidenceAttach` links by JSON pointer (`linkedPaths`): building elements
 * and survey items (photos), stored by element id where the element has one. `EvidenceReferencePicker` fills a `…Reference`
 * text with `evidence:<id>`: the source fields of the NTA input.
 */
import { useEffect, useMemo, useState } from 'react';
import { Camera, Paperclip, X } from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import type { NtaEvidenceItem } from '../../core/nta/KernelClient';
import { createEvidenceItem, loadEvidenceBytes } from '../../core/nta/Evidence';
import {
  evidenceForPointer, evidenceIdsIn, isImageFile, linkEvidence, stablePointer, unlinkEvidence, withEvidenceReference,
} from '../../core/nta/EvidenceLinks';
import { FileButton } from '../ui';
import './EvidenceLink.css';

/** The evidence register of the open project and a setter for it. */
export function useEvidenceRegister(): [NtaEvidenceItem[], (next: NtaEvidenceItem[]) => void] {
  const { state, dispatch } = useEnergy();
  const registration = state.project.registration;
  const evidence = registration?.evidence ?? [];
  const set = (next: NtaEvidenceItem[]) =>
    dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { registration: { ...(registration ?? {}), evidence: next } } });
  return [evidence, set];
}

/** Reads files the user chose into new register entries (hashed, kept for the dossier). */
async function addFiles(files: FileList | null, evidence: NtaEvidenceItem[], photo: boolean): Promise<NtaEvidenceItem[]> {
  let next = [...evidence];
  const added: NtaEvidenceItem[] = [];
  for (const file of Array.from(files ?? [])) {
    const bytes = new Uint8Array(await file.arrayBuffer());
    const item = await createEvidenceItem({ name: file.name, bytes, lastModified: file.lastModified }, next);
    const entry = photo && isImageFile(file.name) && item.kind === 'other' ? { ...item, kind: 'photo_detail' as const } : item;
    next = [...next, entry];
    added.push(entry);
  }
  return added;
}

/** A small preview of an image file in the register (when its bytes are on this machine). */
function Thumbnail({ item }: { item: NtaEvidenceItem }) {
  const [url, setUrl] = useState<string | null>(null);
  useEffect(() => {
    if (!isImageFile(item.fileName) || typeof URL.createObjectURL !== 'function') return undefined;
    let live = true;
    let created: string | null = null;
    void loadEvidenceBytes(item).then((bytes) => {
      if (!live || !bytes) return;
      created = URL.createObjectURL(new Blob([new Uint8Array(bytes)]));
      setUrl(created);
    });
    return () => {
      live = false;
      if (created) URL.revokeObjectURL(created);
    };
  }, [item]);
  if (!url) return <Paperclip className="evidence-link__icon" aria-hidden="true" />;
  return <img className="evidence-link__thumb" src={url} alt="" />;
}

function ItemRow({ item, onUnlink }: { item: NtaEvidenceItem; onUnlink: () => void }) {
  const { t } = useI18n();
  return (
    <li className="evidence-link__item">
      <Thumbnail item={item} />
      <span className="evidence-link__name" title={`SHA-256 ${item.sha256}`}>
        <strong>{item.id}</strong> {item.fileName}
        <small> · {t(`evidence.kind.${item.kind}`)}{item.date ? ` · ${item.date}` : ''}</small>
      </span>
      <button type="button" className="ui-btn ui-btn--ghost ui-btn--sm" onClick={onUnlink}
        aria-label={t('evidenceLink.unlinkNamed', { name: item.fileName })}>
        <X aria-hidden="true" />
      </button>
    </li>
  );
}

function LinkExisting({ options, onLink, label }: { options: NtaEvidenceItem[]; onLink: (id: string) => void; label: string }) {
  const { t } = useI18n();
  if (options.length === 0) return null;
  return (
    <select className="evidence-link__select" aria-label={label} value=""
      onChange={(event) => { if (event.target.value) onLink(event.target.value); }}>
      <option value="">{t('evidenceLink.choose')}</option>
      {options.map((item) => <option key={item.id} value={item.id}>{item.id} · {item.fileName}</option>)}
    </select>
  );
}

/** Files linked to a JSON pointer, with add (file or photo) and link/unlink. */
export function EvidenceAttach({ pointer, title, hint, photo = false }: {
  pointer: string; title?: string; hint?: string; photo?: boolean;
}) {
  const { t } = useI18n();
  const { state: { project } } = useEnergy();
  const [evidence, setEvidence] = useEvidenceRegister();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  // Stored by element id where the element has one, so the link follows it.
  const stable = stablePointer(project, pointer);
  const linked = evidenceForPointer(evidence, pointer, project);
  const others = evidence.filter((item) => !linked.includes(item) && (!photo || isImageFile(item.fileName)));
  const heading = title ?? (photo ? t('evidenceLink.photos') : t('evidenceLink.title'));

  const onFiles = async (files: FileList | null) => {
    if (!files?.length) return;
    setBusy(true);
    setError(null);
    try {
      const added = await addFiles(files, evidence, photo);
      setEvidence([...evidence, ...added.map((item) => ({ ...item, linkedPaths: [stable] }))]);
    } catch {
      setError(t('evidenceLink.failed'));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="evidence-link" role="group" aria-label={`${heading} ${pointer}`} data-pointer={pointer}>
      <div className="evidence-link__head">
        {photo ? <Camera aria-hidden="true" /> : <Paperclip aria-hidden="true" />}
        <strong>{heading}</strong>
      </div>
      {hint && <p className="evidence-link__hint">{hint}</p>}
      {linked.length === 0
        ? <p className="evidence-link__none">{t('evidenceLink.none')}</p>
        : <ul className="evidence-link__list">
          {linked.map((item) => <ItemRow key={item.id} item={item}
            onUnlink={() => setEvidence(unlinkEvidence(evidence, item.id, pointer, project))} />)}
        </ul>}
      <div className="evidence-link__actions">
        <FileButton label={photo ? t('evidenceLink.addPhoto') : t('evidenceLink.add')} status={busy ? t('evidenceLink.busy') : ''}
          accept={photo ? 'image/*' : 'application/pdf,image/*'} multiple disabled={busy}
          {...(photo ? { capture: 'environment' as const } : {})}
          aria-label={`${photo ? t('evidenceLink.addPhoto') : t('evidenceLink.add')} ${pointer}`}
          onChange={(event) => {
            const input = event.currentTarget;
            void onFiles(input.files).then(() => { input.value = ''; });
          }} />
        <LinkExisting options={others} label={t('evidenceLink.linkExisting')}
          onLink={(id) => setEvidence(linkEvidence(evidence, id, stable))} />
      </div>
      {error && <p className="evidence-link__error" role="alert">{error}</p>}
    </div>
  );
}

/**
 * Picker for a `…Reference` field: shows the linked file of `evidence:<id>`,
 * links another register entry or adds a file and links it.
 */
export function EvidenceReferencePicker({ path, value, onChange }: {
  path: string; value: unknown; onChange: (next: string) => void;
}) {
  const { t } = useI18n();
  const [evidence, setEvidence] = useEvidenceRegister();
  const [busy, setBusy] = useState(false);
  const ids = evidenceIdsIn(value);
  const byId = useMemo(() => new Map(evidence.map((item) => [item.id, item])), [evidence]);
  const label = t('evidenceLink.reference', { path });

  const onFiles = async (files: FileList | null) => {
    if (!files?.length) return;
    setBusy(true);
    try {
      const added = await addFiles(files, evidence, false);
      setEvidence([...evidence, ...added]);
      if (added[0]) onChange(withEvidenceReference(value, added[0].id));
    } finally {
      setBusy(false);
    }
  };

  return (
    <span className="evidence-ref">
      {ids.map((id) => {
        const item = byId.get(id);
        return <span key={id} className={item ? 'evidence-ref__file' : 'evidence-ref__file evidence-ref__file--missing'}>
          <Paperclip aria-hidden="true" /> {item ? item.fileName : t('evidenceLink.referenceUnknown', { id })}
        </span>;
      })}
      <LinkExisting options={evidence.filter((item) => !ids.includes(item.id))} label={label}
        onLink={(id) => onChange(withEvidenceReference(value, id))} />
      <FileButton label={t('evidenceLink.add')} status={busy ? t('evidenceLink.busy') : ''} accept="application/pdf,image/*"
        disabled={busy} aria-label={`${t('evidenceLink.add')}: ${path}`}
        onChange={(event) => {
          const input = event.currentTarget;
          void onFiles(input.files).then(() => { input.value = ''; });
        }} />
    </span>
  );
}
