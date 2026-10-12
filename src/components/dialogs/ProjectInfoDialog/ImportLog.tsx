import { useI18n } from '../../../i18n/i18n';
import type { ProjectImportRecord } from '../../../core/energy/types';
import { importIssueLine } from '../../../core/io/importText';

interface ImportLogProps {
  records: ProjectImportRecord[];
}

/**
 * What was read in with another program (BRL 9501 §4.3.1, p. 8): tool,
 * file, SHA-256, the version the file names and every value the importer
 * assumed or left out, so the adviser can check them after the import toast
 * is gone.
 */
export function ImportLog({ records }: ImportLogProps) {
  const { t } = useI18n();
  if (records.length === 0) return null;
  return (
    <section className="dialog-section" aria-labelledby="project-info-import-log">
      <h3 className="dialog-section-title" id="project-info-import-log">{t('import.log.title')}</h3>
      <p className="dialog-hint">{t('import.log.intro')}</p>
      {records.map((record, index) => (
        <div key={`${record.importedAt}-${index}`} className="import-log-record">
          <dl className="dialog-grid">
            <dt>{record.tool}</dt>
            <dd>{t('import.log.date')}: {record.importedAt}</dd>
            {record.fileName && <><dt>{t('import.log.file')}</dt><dd>{record.fileName}</dd></>}
            {record.sha256 && <><dt>{t('import.log.sha256')}</dt><dd><code>{record.sha256}</code></dd></>}
            {record.sourceVersion && <><dt>{t('import.log.source')}</dt><dd>{record.sourceVersion}</dd></>}
          </dl>
          {record.issues && record.issues.length > 0 ? (
            <ul className="import-log-issues">
              {record.issues.map((issue, issueIndex) => (
                <li key={`${issue.path}-${issueIndex}`} title={issue.message}>
                  {importIssueLine(issue, t)} — {issue.message}
                </li>
              ))}
            </ul>
          ) : record.issues ? <p className="dialog-hint">{t('import.log.noIssues')}</p> : null}
        </div>
      ))}
    </section>
  );
}
