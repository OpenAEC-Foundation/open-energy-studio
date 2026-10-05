import { useRef, type KeyboardEvent, type ReactNode } from 'react';
import { ArrowRight, AlertTriangle, Info, XCircle } from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { kernelCodeLabel } from '../../i18n/format';
import { cx } from './Button';

// ── DataTable ────────────────────────────────────────────────────────

export interface Column<Row> {
  key: string;
  header: ReactNode;
  /** Unit shown in the header (never repeated in every cell). */
  unit?: ReactNode;
  /** Right-align numeric columns. */
  numeric?: boolean;
  render: (row: Row) => ReactNode;
  width?: number | string;
}

export interface DataTableProps<Row> {
  columns: Array<Column<Row>>;
  rows: Row[];
  rowKey: (row: Row) => string;
  caption?: ReactNode;
  /** Optional group label per row; consecutive rows with the same group share a header row. */
  groupBy?: (row: Row) => string | undefined;
  selectedKey?: string | null;
  onSelect?: (row: Row) => void;
  /** Enter on a row (open the inspector or editor). */
  onActivate?: (row: Row) => void;
  empty?: ReactNode;
  className?: string;
}

/**
 * Table with a sticky header, numeric columns right aligned, optional grouping
 * and selection. ↑/↓ move between rows, Enter activates.
 */
export function DataTable<Row>({
  columns, rows, rowKey, caption, groupBy, selectedKey, onSelect, onActivate, empty, className,
}: DataTableProps<Row>) {
  const body = useRef<HTMLTableSectionElement>(null);

  const onKeyDown = (event: KeyboardEvent<HTMLTableRowElement>, row: Row) => {
    if (event.key === 'Enter' && onActivate) { event.preventDefault(); onActivate(row); return; }
    if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return;
    const items = Array.from(body.current?.querySelectorAll<HTMLTableRowElement>('tr[data-row]') ?? []);
    const index = items.indexOf(event.currentTarget);
    const next = items[index + (event.key === 'ArrowDown' ? 1 : -1)];
    if (next) { event.preventDefault(); next.focus(); }
  };

  const interactive = Boolean(onSelect || onActivate);
  let lastGroup: string | undefined;
  return (
    <div className={cx('ui-table-wrap', className)}>
      <table className="ui-table">
        {caption && <caption className="visually-hidden">{caption}</caption>}
        <thead>
          <tr>
            {columns.map((column) => (
              <th key={column.key} scope="col" className={column.numeric ? 'ui-num-cell' : undefined} style={{ width: column.width }}>
                {column.header}
                {column.unit && <>{' '}<span className="ui-table__unit">({column.unit})</span></>}
              </th>
            ))}
          </tr>
        </thead>
        <tbody ref={body}>
          {rows.length === 0 && (
            <tr><td colSpan={columns.length} className="ui-table__empty">{empty ?? '–'}</td></tr>
          )}
          {rows.flatMap((row) => {
            const key = rowKey(row);
            const group = groupBy?.(row);
            const out: ReactNode[] = [];
            if (group !== undefined && group !== lastGroup) {
              out.push(<tr key={`group-${group}`} className="ui-table__group"><th scope="rowgroup" colSpan={columns.length}>{group}</th></tr>);
            }
            lastGroup = group;
            const selected = selectedKey === key;
            out.push(
              <tr
                key={key}
                data-row=""
                className={cx(selected && 'ui-table__row--selected', interactive && 'ui-table__row--interactive')}
                tabIndex={interactive ? 0 : undefined}
                aria-selected={onSelect ? selected : undefined}
                onClick={onSelect ? () => onSelect(row) : undefined}
                onDoubleClick={onActivate ? () => onActivate(row) : undefined}
                onKeyDown={interactive ? (event) => onKeyDown(event, row) : undefined}
              >
                {columns.map((column) => (
                  <td key={column.key} className={column.numeric ? 'ui-num-cell' : undefined}>{column.render(row)}</td>
                ))}
              </tr>,
            );
            return out;
          })}
        </tbody>
      </table>
    </div>
  );
}

// ── IssueList ────────────────────────────────────────────────────────

export interface IssueItem {
  code: string;
  severity: 'error' | 'warning' | 'info';
  /** Kernel path; shown as the location and used by "Ga naar". */
  path?: string;
  detail?: ReactNode;
  /** Human location, e.g. "Installaties › Verwarming"; falls back to the path. */
  location?: ReactNode;
}

export interface IssueListProps {
  issues: IssueItem[];
  /** Translation prefixes tried first for the code label. */
  prefixes?: string[];
  onGoTo?: (issue: IssueItem) => void;
  empty?: ReactNode;
  className?: string;
}

const SEVERITY_ICON = {
  error: <XCircle className="ui-issue__icon ui-issue__icon--error" aria-hidden="true" />,
  warning: <AlertTriangle className="ui-issue__icon ui-issue__icon--warning" aria-hidden="true" />,
  info: <Info className="ui-issue__icon ui-issue__icon--info" aria-hidden="true" />,
};

/** Severity icon, translated title, place, technical code in small mono, and "Ga naar". */
export function IssueList({ issues, prefixes, onGoTo, empty, className }: IssueListProps) {
  const { t } = useI18n();
  if (issues.length === 0) return empty ? <div className="ui-issues ui-issues--empty">{empty}</div> : null;
  return (
    <ul className={cx('ui-issues', className)}>
      {issues.map((issue, index) => {
        const label = kernelCodeLabel(t, issue.code, prefixes);
        return (
          <li key={`${issue.code}-${issue.path ?? ''}-${index}`} className={`ui-issue ui-issue--${issue.severity}`}>
            {SEVERITY_ICON[issue.severity]}
            <span className="visually-hidden">{t(`ui.severity.${issue.severity}`)}: </span>
            <div className="ui-issue__body">
              <span className="ui-issue__title">{label.text}</span>
              {(issue.location || issue.path) && <span className="ui-issue__where">{issue.location ?? issue.path}</span>}
              {issue.detail && <span className="ui-issue__detail">{issue.detail}</span>}
              {label.known && <code className="ui-issue__code">{issue.code}</code>}
            </div>
            {onGoTo && (
              <button type="button" className="ui-btn ui-btn--ghost ui-btn--sm" onClick={() => onGoTo(issue)}>
                {t('ui.goTo')} <ArrowRight aria-hidden="true" />
              </button>
            )}
          </li>
        );
      })}
    </ul>
  );
}
