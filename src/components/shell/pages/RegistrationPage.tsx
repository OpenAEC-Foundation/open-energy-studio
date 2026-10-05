/**
 * Step 10 Registratie: the registration data (project-info dialog for now)
 * and the kernel's BRL 9500 registration check with its open points.
 */
import { Pencil } from 'lucide-react';
import { useI18n } from '../../../i18n/i18n';
import { useKernel } from '../../../context/KernelProvider';
import { routeForPath } from '../../../core/nta/gapRoutes';
import type { IProject } from '../../../core/energy/types';
import { Banner, Button, Card, IssueList, Pill } from '../../ui';
import type { ShellActions } from '../ShellActions';
import { routeLabel } from '../PageHeader';

export function RegistrationEditButton({ onOpen }: { onOpen: () => void }) {
  const { t } = useI18n();
  return <Button icon={<Pencil aria-hidden="true" />} onClick={onOpen}>{t('registration.page.edit')}</Button>;
}

export function RegistrationPage({ project, actions }: { project: IProject; actions: Pick<ShellActions, 'navigate' | 'openDialog'> }) {
  const { t } = useI18n();
  const registration = useKernel()?.settled?.registration ?? null;
  const yesNo = (value: boolean | null | undefined) => (value == null ? '—'
    : <Pill tone={value ? 'ok' : 'warn'}>{t(value ? 'registration.page.yes' : 'registration.page.no')}</Pill>);
  const data = project.registration;

  return <>
    {!registration && <Banner tone="info">{t('registration.page.noAssessment')}</Banner>}
    {registration && (
      <Card title={t('nav.step.registration')} level={2}>
        <table className="kv-table"><tbody>
          <tr><td>{t('registration.page.ready')}</td><td>{yesNo(registration.readyForRegistration)}</td></tr>
          <tr><td>{t('registration.page.dossier')}</td><td>{yesNo(registration.dossierComplete)}</td></tr>
          <tr><td>{t('registration.page.attested')}</td><td>{registration.softwareAttested
            ? yesNo(true) : <Pill tone="unv">{t('status.unattested')}</Pill>}</td></tr>
          {registration.validUntil && <tr><td>{t('registration.page.validUntil')}</td><td>{registration.validUntil}</td></tr>}
          {registration.registrationDeadline && <tr><td>{t('registration.page.deadline')}</td><td>{registration.registrationDeadline}</td></tr>}
          {data?.epOnlineNumber && <tr><td>EP-Online</td><td className="mono">{data.epOnlineNumber}</td></tr>}
        </tbody></table>
      </Card>
    )}
    {registration && registration.issues.length > 0 && (
      <Card title={t('registration.page.issues')} level={2} flush>
        <IssueList prefixes={['registration.issue.']}
          issues={registration.issues.map((issue) => ({
            code: issue.code, severity: 'error' as const, path: issue.path, location: routeLabel(t, routeForPath(issue.path)),
          }))}
          onGoTo={(issue) => {
            const route = routeForPath(issue.path);
            if (route.step === 'registration' || route.step === 'project') actions.openDialog('project-info');
            else actions.navigate(route);
          }} />
      </Card>
    )}
  </>;
}
