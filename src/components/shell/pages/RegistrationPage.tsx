/**
 * Step 10 Registratie (UI redesign F9): readiness with its reasons, the
 * program identity and attest status, the kernel's BRL 9500 registration
 * check (open points and plausibility, each with "Ga naar") and the
 * registration data itself (`RegistrationForm`).
 */
import { CheckCircle2, CircleAlert } from 'lucide-react';
import { useI18n } from '../../../i18n/i18n';
import { useKernel } from '../../../context/KernelProvider';
import { routeForPath } from '../../../core/nta/gapRoutes';
import { softwareIdentity } from '../../../core/nta/Registration';
import type { IProject } from '../../../core/energy/types';
import type { RegistrationAssessment } from '../../../core/nta/KernelClient';
import { Banner, Card, IssueList, Pill } from '../../ui';
import type { ShellActions } from '../ShellActions';
import './delivery.css';
import { routeLabel } from '../PageHeader';
import { RegistrationForm } from './RegistrationForm';

/** Every reason the registration is not ready (dossier and/or attest), as i18n keys. */
export function readinessReasons(assessment: RegistrationAssessment | null | undefined): string[] {
  if (!assessment || assessment.readyForRegistration) return [];
  const reasons: string[] = [];
  if (!(assessment.dossierComplete ?? assessment.issues.length === 0)) reasons.push('registration.page.reason.dossier');
  if (assessment.softwareAttested === false) reasons.push('registration.page.reason.attest');
  return reasons;
}

export function RegistrationPage({ project, actions }: { project: IProject; actions: Pick<ShellActions, 'navigate' | 'openDialog'> }) {
  const { t } = useI18n();
  const registration = useKernel()?.settled?.registration ?? null;
  const yesNo = (value: boolean | null | undefined) => (value == null ? '—'
    : <Pill tone={value ? 'ok' : 'warn'}>{t(value ? 'registration.page.yes' : 'registration.page.no')}</Pill>);
  const data = project.registration;
  const reasons = readinessReasons(registration);
  // The kernel's view of the program once assessed; this program's identity before that.
  const software = registration?.software ?? softwareIdentity();
  const attested = registration ? registration.softwareAttested : Boolean(software.attestNumber);
  const dossierComplete = registration ? (registration.dossierComplete ?? registration.issues.length === 0) : null;
  const goTo = (path: string) => actions.navigate(routeForPath(path));
  const location = (path: string) => routeLabel(t, routeForPath(path));

  return <>
    {!attested && (
      <Banner tone="unv" role="note" title={t('registration.page.attestBanner')}>{t('registration.page.attestNote')}</Banner>
    )}
    <div className="registration-summary">
      <Card title={t('registration.page.readiness')} level={2} className="registration-readiness">
        {!registration && <p className="registration-pending">{t('registration.page.noAssessment')}</p>}
        {registration && <>
          <div className="registration-readiness__verdict" data-ready={registration.readyForRegistration}>
            {registration.readyForRegistration
              ? <CheckCircle2 aria-hidden="true" />
              : <CircleAlert aria-hidden="true" />}
            <div>
              <strong>{t('registration.page.ready')}: {t(registration.readyForRegistration ? 'registration.page.yes' : 'registration.page.no')}</strong>
              {reasons.length > 0 && (
                <ul className="registration-reasons">
                  {reasons.map((key) => <li key={key}>{t(key)}</li>)}
                </ul>
              )}
            </div>
          </div>
          <table className="kv-table"><tbody>
            <tr><td>{t('registration.page.dossier')}</td><td>{yesNo(dossierComplete)}</td></tr>
            <tr><td>{t('registration.page.attested')}</td><td>{registration.softwareAttested
              ? yesNo(true) : <Pill tone="unv">{t('status.unattested')}</Pill>}</td></tr>
            {registration.validUntil && <tr><td>{t('registration.page.validUntil')}</td><td>{registration.validUntil}</td></tr>}
            {registration.registrationDeadline && <tr><td>{t('registration.page.deadline')}</td><td>{registration.registrationDeadline}</td></tr>}
            {registration.relabelDeadline && <tr><td>{t('registration.page.relabelDeadline')}</td><td>{registration.relabelDeadline}</td></tr>}
            {registration.replacementDeadline && <tr><td>{t('registration.page.replacementDeadline')}</td><td>{registration.replacementDeadline}</td></tr>}
            {data?.epOnlineNumber && <tr><td>EP-Online</td><td className="mono">{data.epOnlineNumber}</td></tr>}
          </tbody></table>
        </>}
      </Card>
      <Card title={t('registration.page.software')} level={2}>
        <table className="kv-table"><tbody>
          <tr><td>{t('registration.page.program')}</td><td>{software.name} {software.version}</td></tr>
          {software.kernelVersion && <tr><td>{t('registration.page.kernelVersion')}</td>
            <td className="mono">{software.kernelVersion}</td></tr>}
          <tr><td>{t('registration.page.attestNumber')}</td>
            <td>{software.attestNumber || <Pill tone="unv">{t('status.unattested')}</Pill>}</td></tr>
        </tbody></table>
      </Card>
    </div>
    {registration && registration.issues.length > 0 && (
      <Card title={t('registration.page.issues')} level={2} flush>
        <IssueList prefixes={['registration.issue.']}
          issues={registration.issues.map((issue) => ({
            code: issue.code, severity: 'error' as const, path: issue.path, location: location(issue.path),
          }))}
          onGoTo={(issue) => goTo(issue.path ?? '')} />
      </Card>
    )}
    {registration && (registration.plausibility?.length ?? 0) > 0 && (
      <Card title={t('registration.page.plausibility')} subtitle={t('registration.page.plausibilityHint')} level={2} flush>
        <IssueList prefixes={['registration.issue.']}
          issues={(registration.plausibility ?? []).map((issue) => ({
            code: issue.code, severity: 'warning' as const, path: issue.path, location: location(issue.path),
          }))}
          onGoTo={(issue) => goTo(issue.path ?? '')} />
      </Card>
    )}
    <RegistrationForm />
  </>;
}
