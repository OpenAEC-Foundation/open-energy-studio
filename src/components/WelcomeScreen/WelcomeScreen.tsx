/**
 * Start screen without an open document: the project library (feedback
 * 8 Oct 2026) with folders per client, search, a card per project with
 * what it is, its last outcome and its state, and the ways to start: new,
 * open, import and the two fictional examples. The recent project files of
 * the desktop app that are not in the library yet are listed as files.
 */
import { useMemo, useState, useSyncExternalStore, type ReactNode } from 'react';
import {
  Archive, Building, Building2, Caravan, Copy, FileInput, Folder, FolderOpen, FolderPlus, HousePlus, Home, LayoutGrid, List, MoreHorizontal,
  Plus, Search, Ship, Trash2, X,
} from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import { EXAMPLE_KINDS, type ExampleKind } from '../../core/nta/ExampleProjects';
import type { RecentProject } from '../../core/io/recentProjects';
import {
  browserStore, libraryFolders, libraryVersion, readLibrary, subscribeLibrary, type LibraryEntry, type LibraryKind,
} from '../../core/io/projectLibrary';
import { IconButton } from '../ui';
import { labelClassName } from '../shell/labelClass';
import './WelcomeScreen.css';

/** New-build calculation, or a basisopname of an existing dwelling or utility building. */
export type NewProjectKind = 'residential' | 'utility' | 'existing_residential' | 'existing_utility';

interface WelcomeScreenProps {
  onNewProject: (kind: NewProjectKind) => void;
  onOpenProject: () => void;
  onOpenExample: (kind: ExampleKind) => void;
  onImportUNIEC3?: () => void;
  onImportVABI?: () => void;
  recent?: RecentProject[];
  onOpenRecent?: (path: string) => void;
  onForgetRecent?: (path: string) => void;
  onOpenEntry?: (id: string) => void;
  onDuplicateEntry?: (id: string) => void;
  onDeleteEntry?: (id: string) => void;
  onMoveEntry?: (id: string, folder: string | null) => void;
  onArchiveEntry?: (id: string, archived: boolean) => void;
  /** Shown over open documents: a way back to the active project. */
  onBack?: () => void;
}

const ALL = '__all__';
const ARCHIVE = '__archive__';
const NONE = '__none__';
const KINDS: LibraryKind[] = ['existing_residential', 'existing_utility', 'new_residential', 'new_utility'];

/** The library of this browser, re-read on every change. */
export function useLibrary(): LibraryEntry[] {
  const version = useSyncExternalStore(subscribeLibrary, libraryVersion, libraryVersion);
  return useMemo(() => {
    const store = browserStore();
    return store ? readLibrary(store) : [];
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [version]);
}

function KindIcon({ entry }: { entry: Pick<LibraryEntry, 'kind' | 'dwelling'> }) {
  if (entry.dwelling === 'floating') return <Ship aria-hidden="true" />;
  if (entry.dwelling === 'caravan') return <Caravan aria-hidden="true" />;
  if (entry.dwelling === 'apartment') return <Building aria-hidden="true" />;
  switch (entry.kind) {
    case 'existing_residential': return <Home aria-hidden="true" />;
    case 'new_residential': return <HousePlus aria-hidden="true" />;
    default: return <Building2 aria-hidden="true" />;
  }
}

/** The file name of a path, for the file list (the full path is the title). */
function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

function Menu({ label, items }: { label: ReactNode; items: Array<{ key: string; label: ReactNode; onClick: () => void; danger?: boolean }> }) {
  const [open, setOpen] = useState(false);
  return <span className="library-menu" onBlur={(event) => { if (!event.currentTarget.contains(event.relatedTarget as Node)) setOpen(false); }}>
    <button type="button" className="btn btn-sm" aria-haspopup="menu" aria-expanded={open} onClick={() => setOpen((value) => !value)}>{label}</button>
    {open && <ul className="library-menu__list" role="menu">
      {items.map((item) => <li key={item.key} role="none">
        <button type="button" role="menuitem" className={item.danger ? 'library-menu__item library-menu__item--danger' : 'library-menu__item'}
          onClick={() => { setOpen(false); item.onClick(); }}>{item.label}</button>
      </li>)}
    </ul>}
  </span>;
}

export function WelcomeScreen({
  onNewProject, onOpenProject, onOpenExample, onImportUNIEC3, onImportVABI, recent = [], onOpenRecent, onForgetRecent,
  onOpenEntry, onDuplicateEntry, onDeleteEntry, onMoveEntry, onArchiveEntry, onBack,
}: WelcomeScreenProps) {
  const { t, locale } = useI18n();
  const entries = useLibrary();
  const [folder, setFolder] = useState(ALL);
  const [kind, setKind] = useState<LibraryKind | 'all'>('all');
  const [query, setQuery] = useState('');
  const [view, setView] = useState<'cards' | 'list'>(() => {
    try { return localStorage.getItem('oes.library.view') === 'list' ? 'list' : 'cards'; } catch { return 'cards'; }
  });
  const chooseView = (next: 'cards' | 'list') => { setView(next); try { localStorage.setItem('oes.library.view', next); } catch { /* per-viewer convenience */ } };
  const folders = libraryFolders(entries);
  const date = (iso: string) => {
    const value = new Date(iso);
    return Number.isNaN(value.getTime()) ? '' : value.toLocaleString(locale, { day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' });
  };
  const needle = query.trim().toLowerCase();
  const shown = entries
    .filter((entry) => folder === ARCHIVE ? entry.archived : !entry.archived && (folder === ALL || (folder === NONE ? !entry.folder : entry.folder === folder)))
    .filter((entry) => kind === 'all' || entry.kind === kind)
    .filter((entry) => !needle || [entry.name, entry.address, entry.city, entry.folder ?? ''].some((text) => text.toLowerCase().includes(needle)))
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt));
  const knownPaths = new Set(entries.map((entry) => entry.filePath).filter(Boolean));
  const files = recent.filter((entry) => !knownPaths.has(entry.path));
  const count = (test: (entry: LibraryEntry) => boolean) => entries.filter(test).length;

  const statusChips = (entry: LibraryEntry): Array<{ text: string; tone?: 'ok' | 'warn' }> => {
    const chips: Array<{ text: string; tone?: 'ok' | 'warn' }> = [];
    const { status } = entry;
    if (status.openAnswers) chips.push({ text: t('library.status.openAnswers', { count: status.openAnswers }), tone: 'warn' });
    if (entry.outcome?.defaultsWithoutReason) chips.push({ text: t('library.status.defaults', { count: entry.outcome.defaultsWithoutReason }), tone: 'warn' });
    if (status.importNotes) chips.push({ text: t('library.status.importNotes', { count: status.importNotes }) });
    if (status.surveyTotal != null && status.surveyDone != null && status.surveyDone < status.surveyTotal) {
      chips.push({ text: t('library.status.survey', { done: status.surveyDone, total: status.surveyTotal }) });
    } else if (status.buildTotal != null && status.buildDone != null && status.buildDone < status.buildTotal) {
      chips.push({ text: t('library.status.build', { done: status.buildDone, total: status.buildTotal }) });
    }
    if (chips.length === 0 && entry.outcome && !entry.outcomeStale) chips.push({ text: t('library.status.complete'), tone: 'ok' });
    return chips;
  };
  const kindText = (entry: LibraryEntry) => entry.dwelling && entry.dwelling !== 'regular'
    ? `${t(`library.kind.${entry.kind}`)} · ${t(`library.dwelling.${entry.dwelling}`)}` : t(`library.kind.${entry.kind}`);
  const moveItems = (entry: LibraryEntry) => [
    ...folders.filter((name) => name !== entry.folder).map((name) => ({ key: name, label: t('library.menu.moveTo', { folder: name }), onClick: () => onMoveEntry?.(entry.id, name) })),
    { key: '__new', label: t('library.menu.newFolder'), onClick: () => { const name = window.prompt(t('library.folderPrompt'))?.trim(); if (name) onMoveEntry?.(entry.id, name); } },
    ...(entry.folder ? [{ key: '__none', label: t('library.menu.noFolder'), onClick: () => onMoveEntry?.(entry.id, null) }] : []),
  ];
  const menuItems = (entry: LibraryEntry) => [
    { key: 'open', label: t('library.menu.open'), onClick: () => onOpenEntry?.(entry.id) },
    { key: 'copy', label: <><Copy aria-hidden="true" /> {t('library.menu.duplicate')}</>, onClick: () => onDuplicateEntry?.(entry.id) },
    ...moveItems(entry).map((item) => ({ ...item, label: <><Folder aria-hidden="true" /> {item.label}</> })),
    { key: 'archive', label: <><Archive aria-hidden="true" /> {t(entry.archived ? 'library.menu.restore' : 'library.menu.archive')}</>, onClick: () => onArchiveEntry?.(entry.id, !entry.archived) },
    { key: 'delete', label: <><Trash2 aria-hidden="true" /> {t('library.menu.delete')}</>, onClick: () => onDeleteEntry?.(entry.id), danger: true },
  ];
  const outcome = (entry: LibraryEntry) => entry.outcome
    ? <span className="library-outcome">
      <span className={`label-badge ${labelClassName(entry.outcome.label)}`}>{entry.outcome.label ?? '—'}</span>
      <span className="library-outcome__text">EP2 {formatNumber(entry.outcome.ep2, locale, 1)}{entry.outcome.beng1 != null ? ` · BENG 1 ${formatNumber(entry.outcome.beng1, locale, 1)}` : ''}
        {entry.outcomeStale && <em className="library-outcome__stale"> · {t('library.outcome.stale')}</em>}</span>
    </span>
    : <span className="library-outcome library-outcome--none"><span className="label-badge lbl-none">—</span><span className="library-outcome__text">{t('library.outcome.none')}</span></span>;

  return (
    <div className="welcome-screen library">
      <div className="welcome-content library-content">
        <header className="welcome-hero">
          <img src="/icon.png" alt="" className="welcome-icon" />
          <div>
            <h1 className="welcome-title">{t('library.title')}</h1>
            <p className="welcome-subtitle">{t('library.subtitle')}</p>
          </div>
        </header>

        <div className="library-layout">
          <aside className="library-side" aria-label={t('library.folders')}>
            <h2 className="welcome-heading">{t('library.folders')}</h2>
            <ul className="library-folders">
              {[
                { id: ALL, label: t('library.all'), n: count((entry) => !entry.archived) },
                ...folders.map((name) => ({ id: name, label: name, n: count((entry) => !entry.archived && entry.folder === name) })),
                ...(folders.length > 0 ? [{ id: NONE, label: t('library.noFolder'), n: count((entry) => !entry.archived && !entry.folder) }] : []),
                { id: ARCHIVE, label: t('library.archive'), n: count((entry) => Boolean(entry.archived)) },
              ].map((item) => <li key={item.id}>
                <button type="button" className="library-folder" aria-current={folder === item.id ? 'true' : undefined} onClick={() => setFolder(item.id)}>
                  {item.id === ARCHIVE ? <Archive aria-hidden="true" /> : item.id === ALL ? <LayoutGrid aria-hidden="true" /> : <Folder aria-hidden="true" />}
                  <span>{item.label}</span><small>{item.n}</small>
                </button>
              </li>)}
            </ul>
            <h2 className="welcome-heading">{t('library.kinds')}</h2>
            <ul className="library-folders">
              {(['all', ...KINDS] as const).map((item) => <li key={item}>
                <button type="button" className="library-folder" aria-current={kind === item ? 'true' : undefined} onClick={() => setKind(item)}>
                  <span>{item === 'all' ? t('library.allKinds') : t(`library.kind.${item}`)}</span>
                  <small>{item === 'all' ? entries.filter((entry) => !entry.archived).length : count((entry) => !entry.archived && entry.kind === item)}</small>
                </button>
              </li>)}
            </ul>
          </aside>

          <section className="library-main" aria-label={t('library.title')}>
            <div className="library-toolbar">
              {onBack && <button type="button" className="btn btn-sm" onClick={onBack}>← {t('shell.libraryBack')}</button>}
              <label className="library-search"><Search aria-hidden="true" />
                <input type="search" value={query} placeholder={t('library.search')} aria-label={t('library.search')} onChange={(event) => setQuery(event.target.value)} />
              </label>
              <Menu label={<span><Plus aria-hidden="true" /> {t('library.new')}</span>} items={[
                { key: 'existing_residential', label: <><Home aria-hidden="true" /> {t('welcome.existingResidential')}</>, onClick: () => onNewProject('existing_residential') },
                { key: 'existing_utility', label: <><Building2 aria-hidden="true" /> {t('welcome.existingUtility')}</>, onClick: () => onNewProject('existing_utility') },
                { key: 'residential', label: <><HousePlus aria-hidden="true" /> {t('welcome.newResidential')}</>, onClick: () => onNewProject('residential') },
                { key: 'utility', label: <><Building2 aria-hidden="true" /> {t('welcome.newUtility')}</>, onClick: () => onNewProject('utility') },
                ...EXAMPLE_KINDS.map((example) => ({ key: example, label: t(`welcome.example.${example}`), onClick: () => onOpenExample(example) })),
              ]} />
              <button type="button" className="btn btn-sm" onClick={onOpenProject}><FolderOpen aria-hidden="true" /> {t('ribbon.open')}</button>
              {(onImportUNIEC3 || onImportVABI) && <Menu label={<span><FileInput aria-hidden="true" /> {t('library.import')}</span>} items={[
                ...(onImportUNIEC3 ? [{ key: 'uniec3', label: t('ribbon.importUNIEC3'), onClick: onImportUNIEC3 }] : []),
                ...(onImportVABI ? [{ key: 'vabi', label: t('ribbon.importVABI'), onClick: onImportVABI }] : []),
              ]} />}
              <span className="library-view" role="group" aria-label={t('library.view')}>
                <IconButton size="sm" icon={<LayoutGrid aria-hidden="true" />} aria-label={t('library.view.cards')} aria-pressed={view === 'cards'} onClick={() => chooseView('cards')} />
                <IconButton size="sm" icon={<List aria-hidden="true" />} aria-label={t('library.view.list')} aria-pressed={view === 'list'} onClick={() => chooseView('list')} />
              </span>
            </div>

            {shown.length === 0 && files.length === 0 && <p className="welcome-empty">{entries.length === 0 ? t('library.empty') : t('library.noMatch')}</p>}

            {view === 'cards' ? <div className="library-grid">
              {shown.map((entry) => <article key={entry.id} className="library-card" data-kind={entry.kind}>
                <button type="button" className="library-card__open" onClick={() => onOpenEntry?.(entry.id)} aria-label={t('library.openNamed', { name: entry.name })}>
                  <span className="library-card__icon"><KindIcon entry={entry} /></span>
                  <strong>{entry.name || t('library.unnamed')}</strong>
                  <span className="library-card__kind">{kindText(entry)}{entry.address || entry.city ? ` · ${[entry.address, entry.city].filter(Boolean).join(', ')}` : ''}</span>
                  {outcome(entry)}
                  <span className="library-chips">{statusChips(entry).map((chip) => <span key={chip.text} className={`library-chip${chip.tone ? ` library-chip--${chip.tone}` : ''}`}>{chip.text}</span>)}</span>
                  <span className="library-card__meta"><span>{entry.folder ?? ''}</span><span>{date(entry.updatedAt)}</span></span>
                </button>
                <span className="library-card__menu"><Menu label={<MoreHorizontal aria-hidden="true" />} items={menuItems(entry)} /></span>
              </article>)}
              <button type="button" className="library-card library-card--new" onClick={() => onNewProject('existing_residential')}>
                <Plus aria-hidden="true" /><strong>{t('library.newCard')}</strong><span className="library-card__kind">{t('library.newCardHint')}</span>
              </button>
            </div>
            : <table className="library-table">
              <thead><tr><th scope="col">{t('properties.name')}</th><th scope="col">{t('library.kindColumn')}</th><th scope="col">{t('library.outcomeColumn')}</th>
                <th scope="col">{t('library.statusColumn')}</th><th scope="col">{t('library.folderColumn')}</th><th scope="col">{t('library.updated')}</th><th scope="col"><span className="visually-hidden">{t('library.menu.open')}</span></th></tr></thead>
              <tbody>{shown.map((entry) => <tr key={entry.id}>
                <td><button type="button" className="library-link" onClick={() => onOpenEntry?.(entry.id)}><KindIcon entry={entry} /> {entry.name || t('library.unnamed')}</button></td>
                <td>{kindText(entry)}</td><td>{outcome(entry)}</td>
                <td>{statusChips(entry).map((chip) => chip.text).join(' · ')}</td><td>{entry.folder ?? ''}</td><td>{date(entry.updatedAt)}</td>
                <td><Menu label={<MoreHorizontal aria-hidden="true" />} items={menuItems(entry)} /></td>
              </tr>)}</tbody>
            </table>}

            {files.length > 0 && <>
              <h2 className="welcome-heading">{t('library.files')}</h2>
              <ul className="welcome-recent">
                {files.map((entry) => <li key={entry.path} className="welcome-recent__item">
                  <button type="button" className="welcome-recent__open" title={entry.path} onClick={() => onOpenRecent?.(entry.path)}>
                    <span className="welcome-recent__name">
                      {entry.labelClass && <span className={`label-badge label-badge--sm ${labelClassName(entry.labelClass)}`}>{entry.labelClass}</span>}
                      {entry.name || fileName(entry.path)}
                    </span>
                    <span className="welcome-recent__path">{entry.path}</span>
                  </button>
                  {onForgetRecent && <IconButton size="sm" icon={<X aria-hidden="true" />}
                    aria-label={t('welcome.recentForget', { name: entry.name || fileName(entry.path) })} onClick={() => onForgetRecent(entry.path)} />}
                </li>)}
              </ul>
            </>}
            <p className="welcome-examples-note"><FolderPlus aria-hidden="true" /> {t('library.note')}</p>
          </section>
        </div>
      </div>
    </div>
  );
}
