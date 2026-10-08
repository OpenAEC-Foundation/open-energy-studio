import type { ReactNode } from 'react';
import { BookOpen } from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { STEP_GROUPS, stepDefinition, type Route } from '../../core/navigation/routes';
import { chapterForRoute } from '../../core/manual/manual-routes';
import { useShellActions } from './ShellActions';
import { Button } from '../ui';

/** "Gebouw › 3D-model" for a route; used by the header, issue locations and the palette. */
export function routeLabel(t: (key: string) => string, route: Route): string {
  const definition = stepDefinition(route.step);
  const sub = definition.subs.find((candidate) => candidate.id === route.sub);
  return sub && definition.subs.length > 1 ? `${t(definition.labelKey)} › ${t(sub.labelKey)}` : t(definition.labelKey);
}

/** Page head of a workflow step: breadcrumb, title, lead and the page actions (mockup 01). */
export function PageHeader({ route, title, lead, actions }: { route: Route; title?: ReactNode; lead?: ReactNode; actions?: ReactNode }) {
  const { t } = useI18n();
  const definition = stepDefinition(route.step);
  const group = STEP_GROUPS.find((candidate) => candidate.id === definition.group);
  const sub = definition.subs.find((candidate) => candidate.id === route.sub);
  const shell = useShellActions();
  // Every workflow page links to the chapter of the manual that explains it (BRL 9501 §4.4).
  const manualLink = shell && route.step !== 'tool' && (
    <Button variant="ghost" icon={<BookOpen aria-hidden="true" />} title={t('manual.openChapter')}
      onClick={() => shell.navigate({ step: 'tool', sub: 'manual', chapter: chapterForRoute(route) })}>{t('manual.title')}</Button>
  );
  return (
    <div className="page-head">
      <div className="page-head-text">
        <div className="page-crumbs">
          {route.step !== 'tool' && group && <><span>{t(group.labelKey)}</span><span aria-hidden="true">›</span></>}
          <b>{t(definition.labelKey)}</b>
          {sub && definition.subs.length > 1 && <><span aria-hidden="true">›</span><span>{t(sub.labelKey)}</span></>}
        </div>
        <h1 className="page-title" id="page-title" tabIndex={-1}>{title ?? t(sub && (route.step === 'tool' || sub.id !== definition.subs[0]?.id) ? sub.labelKey : definition.labelKey)}</h1>
        {lead && <p className="page-lead">{lead}</p>}
      </div>
      {(actions || manualLink) && <div className="page-actions">{actions}{manualLink}</div>}
    </div>
  );
}

/** Sub pages of a step as a row of links under the page head. */
export function SubTabs({ route, onSelect }: { route: Route; onSelect: (sub: string) => void }) {
  const { t } = useI18n();
  const definition = stepDefinition(route.step);
  if (definition.subs.length < 2) return null;
  return (
    <nav className="page-subtabs" aria-label={t(definition.labelKey)}>
      {definition.subs.map((sub) => (
        <button key={sub.id} type="button" className="page-subtab" aria-current={route.sub === sub.id ? 'page' : undefined}
          onClick={() => onSelect(sub.id)}>{t(sub.labelKey)}</button>
      ))}
    </nav>
  );
}
