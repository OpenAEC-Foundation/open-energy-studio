/**
 * Command palette (Ctrl K, ontwerp §3.5): one search over the workflow steps
 * and sub pages, every command of the former ribbon, project data fields and
 * the project items (zones, surfaces, windows, constructions, systems).
 */
import { useEffect, useId, useMemo, useRef, useState } from 'react';
import { Search } from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { Kbd } from '../ui';
import { SHELL_COMMANDS } from './commands';
import type { ShellActions } from './ShellActions';
import { TOOL_STEP, WORKFLOW_STEPS, type Route } from '../../core/navigation/routes';
import { projectItems } from '../../core/navigation/projectPaths';
import type { IProject } from '../../core/energy/types';

type Group = 'steps' | 'commands' | 'fields' | 'items';

export interface PaletteEntry {
  id: string;
  group: Group;
  label: string;
  where?: string;
  shortcut?: string;
  /** Extra search words (the other language is not needed: labels follow the locale). */
  keywords?: string;
  run: () => void;
}

const GROUP_ORDER: Group[] = ['steps', 'commands', 'fields', 'items'];

/** Project data fields; they live in the project-info dialog for now. */
const FIELD_KEYS = [
  'dialog.projectInfo.name', 'dialog.projectInfo.description', 'dialog.projectInfo.function', 'dialog.projectInfo.address',
  'dialog.projectInfo.city', 'reg.purpose', 'reg.constructionYear', 'reg.messageType', 'reg.surveyType', 'reg.advisorName',
  'reg.wlcGwp', 'evidence.title',
];

export function buildPaletteEntries(
  t: (key: string, params?: Record<string, string>) => string,
  project: IProject | null,
  actions: ShellActions,
  onSelectItem: (item: { id: string; itemType: string; route: Route }) => void,
): PaletteEntry[] {
  const entries: PaletteEntry[] = [];
  for (const step of [...WORKFLOW_STEPS, TOOL_STEP]) {
    if (step.id !== 'tool') {
      entries.push({ id: `step:${step.id}`, group: 'steps', label: t(step.labelKey), where: `${step.number}`, run: () => actions.navigate({ step: step.id }) });
    }
    for (const sub of step.subs) {
      entries.push({
        id: `step:${step.id}/${sub.id}`, group: 'steps', label: t(sub.labelKey), where: t(step.labelKey),
        run: () => actions.navigate({ step: step.id, sub: sub.id }),
      });
    }
  }
  for (const command of SHELL_COMMANDS) {
    if (command.id === 'results' || command.id === 'model3d' || command.id.startsWith('tool-')) continue; // already a step
    entries.push({
      id: `cmd:${command.id}`, group: 'commands', label: t(command.labelKey), shortcut: command.shortcut,
      run: () => command.run(actions),
    });
  }
  if (project) {
    for (const key of FIELD_KEYS) {
      entries.push({
        id: `field:${key}`, group: 'fields', label: t(key), where: t('dialog.projectInfo.title'),
        run: () => actions.openDialog('project-info'),
      });
    }
    for (const item of projectItems(project)) {
      entries.push({
        id: `item:${item.itemType}:${item.id}`, group: 'items', label: item.name, where: t(item.kindKey),
        run: () => onSelectItem(item),
      });
    }
  }
  return entries;
}

/** Case- and accent-insensitive match on every word; prefix matches rank first. */
export function filterEntries(entries: PaletteEntry[], query: string): PaletteEntry[] {
  const fold = (text: string) => text.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase();
  const words = fold(query).split(/\s+/).filter(Boolean);
  if (words.length === 0) return entries.filter((entry) => entry.group !== 'items' && entry.group !== 'fields');
  const scored = entries.flatMap((entry) => {
    const haystack = fold(`${entry.label} ${entry.where ?? ''} ${entry.keywords ?? ''}`);
    if (!words.every((word) => haystack.includes(word))) return [];
    const label = fold(entry.label);
    const score = label.startsWith(words[0]) ? 0 : label.includes(words[0]) ? 1 : 2;
    return [{ entry, score }];
  });
  return scored
    .sort((a, b) => a.score - b.score || GROUP_ORDER.indexOf(a.entry.group) - GROUP_ORDER.indexOf(b.entry.group))
    .map(({ entry }) => entry);
}

export function CommandPalette({ entries, onClose }: { entries: PaletteEntry[]; onClose: () => void }) {
  const { t } = useI18n();
  const [query, setQuery] = useState('');
  const [active, setActive] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);
  const returnFocus = useRef<HTMLElement | null>(document.activeElement as HTMLElement | null);
  const listId = useId();

  const shown = useMemo(() => filterEntries(entries, query).slice(0, 60), [entries, query]);
  useEffect(() => { setActive(0); }, [query]);
  useEffect(() => {
    inputRef.current?.focus();
    const previous = returnFocus.current;
    return () => { if (previous && document.contains(previous)) previous.focus(); };
  }, []);
  useEffect(() => {
    listRef.current?.querySelector(`[data-index="${active}"]`)?.scrollIntoView?.({ block: 'nearest' });
  }, [active]);

  const run = (entry: PaletteEntry | undefined) => {
    if (!entry) return;
    returnFocus.current = null; // the command decides where focus goes
    onClose();
    entry.run();
  };

  const onKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      if (shown.length === 0) return;
      setActive((index) => (index + (event.key === 'ArrowDown' ? 1 : -1) + shown.length) % shown.length);
    } else if (event.key === 'Enter') {
      event.preventDefault();
      run(shown[active]);
    } else if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      onClose();
    } else if (event.key === 'Tab') {
      event.preventDefault(); // focus stays in the palette
    }
  };

  let lastGroup: Group | null = null;
  return (
    <div className="palette-backdrop" onMouseDown={(event) => { if (event.target === event.currentTarget) onClose(); }}>
      <div className="palette" role="dialog" aria-modal="true" aria-label={t('palette.title')} onKeyDown={onKeyDown}>
        <div className="palette-input-row">
          <Search aria-hidden="true" />
          <input ref={inputRef} className="palette-input" value={query} onChange={(event) => setQuery(event.target.value)}
            placeholder={t('palette.placeholder')} aria-label={t('palette.placeholder')} role="combobox" aria-expanded="true"
            aria-controls={listId} aria-autocomplete="list"
            aria-activedescendant={shown[active] ? `${listId}-${active}` : undefined} />
          <Kbd>Esc</Kbd>
        </div>
        {shown.length === 0
          ? <div className="palette-empty" role="status">{t('palette.empty')}</div>
          : (
            <ul className="palette-list" id={listId} role="listbox" ref={listRef} aria-label={t('palette.title')}>
              {shown.map((entry, index) => {
                const header = entry.group !== lastGroup ? entry.group : null;
                lastGroup = entry.group;
                return [
                  header && <li key={`g-${header}`} role="presentation" className="palette-group">{t(`palette.group.${header}`)}</li>,
                  <li key={entry.id} id={`${listId}-${index}`} data-index={index} role="option" aria-selected={index === active}
                    className="palette-option" onMouseMove={() => setActive(index)} onClick={() => run(entry)}>
                    <span>{entry.label}</span>
                    {entry.where && <span className="where">{entry.where}</span>}
                    {entry.shortcut && <Kbd>{entry.shortcut}</Kbd>}
                  </li>,
                ];
              })}
            </ul>
          )}
        <div className="palette-hint" aria-hidden="true">{t('palette.hint')}</div>
      </div>
    </div>
  );
}
