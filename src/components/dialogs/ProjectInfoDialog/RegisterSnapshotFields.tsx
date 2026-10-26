import { useI18n } from '../../../i18n/i18n';
import type { NtaEvidenceItem, NtaRegisterSnapshot, NtaRegisterValue } from '../../../core/nta/KernelClient';
import {
  REGISTRY_STATES, newSnapshot, snapshotStatus, withField, withValues,
} from '../../../core/nta/registerSnapshot';

interface RegisterSnapshotFieldsProps {
  item: NtaEvidenceItem;
  onChange: (snapshot: NtaRegisterSnapshot | undefined) => void;
}

const today = () => new Date().toISOString().slice(0, 10);

/**
 * The register snapshot of a declaration in the evidence register (issue
 * #15): the identity of the declaration, the values copied from it with the
 * project input each fills, the adviser's confirmation and, separately, the
 * check of the declaration's current status in the register.
 */
export function RegisterSnapshotFields({ item, onChange }: RegisterSnapshotFieldsProps) {
  const { t } = useI18n();
  const snapshot = item.registerSnapshot;
  const id = (field: string) => `evidence-${item.id}-snapshot-${field}`;
  if (!snapshot) {
    return (
      <div className="register-snapshot">
        <button type="button" onClick={() => onChange(newSnapshot(item))}>{t('snapshot.add')}</button>
        <p className="dialog-hint">{t('snapshot.hint')}</p>
      </div>
    );
  }
  const status = snapshotStatus(snapshot);
  const setValues = (values: NtaRegisterValue[]) => onChange(withValues(snapshot, values));
  const setValue = (index: number, patch: Partial<NtaRegisterValue>) => setValues(snapshot.values.map((value, at) => {
    if (at !== index) return value;
    const next: NtaRegisterValue = { ...value, ...patch };
    if (!next.unit) delete next.unit;
    if (!next.path) delete next.path;
    return next;
  }));
  const text = (field: 'register' | 'declarationNumber' | 'revision' | 'productName' | 'manufacturer' | 'scope', label: string) => (
    <div className="dialog-field">
      <label htmlFor={id(field)}>{label}</label>
      <input id={id(field)} type="text" value={snapshot[field] ?? ''}
        onChange={(event) => onChange(withField(snapshot, field, event.target.value as never))} />
    </div>
  );
  const check = snapshot.registryCheck ?? { state: 'not_checked' as const };
  return (
    <fieldset className="register-snapshot">
      <legend>{t('snapshot.title')}</legend>
      <p className="dialog-hint" role="status">
        {t(status.confirmed ? 'snapshot.status.confirmed' : 'snapshot.status.unconfirmed')} · {t(`snapshot.registry.${status.registry}`)}
      </p>
      {text('register', t('snapshot.register'))}
      {text('declarationNumber', t('snapshot.declarationNumber'))}
      {text('revision', t('snapshot.revision'))}
      {text('productName', t('snapshot.productName'))}
      {text('manufacturer', t('snapshot.manufacturer'))}
      {text('scope', t('snapshot.scope'))}
      <div className="dialog-field">
        <label htmlFor={id('captured')}>{t('snapshot.capturedOn')}</label>
        <input id={id('captured')} type="date" value={snapshot.capturedOn}
          onChange={(event) => onChange(withField(snapshot, 'capturedOn', event.target.value))} />
      </div>
      <table className="register-snapshot-values">
        <caption>{t('snapshot.values')}</caption>
        <thead><tr>
          <th scope="col">{t('snapshot.value.label')}</th><th scope="col">{t('snapshot.value.value')}</th>
          <th scope="col">{t('snapshot.value.unit')}</th><th scope="col">{t('snapshot.value.path')}</th><th />
        </tr></thead>
        <tbody>
          {snapshot.values.map((value, index) => (
            <tr key={index}>
              <td><input aria-label={`${t('snapshot.value.label')} ${index + 1}`} type="text" value={value.label}
                onChange={(event) => setValue(index, { label: event.target.value })} /></td>
              <td><input aria-label={`${t('snapshot.value.value')} ${index + 1}`} type="text" value={value.value}
                onChange={(event) => setValue(index, { value: event.target.value })} /></td>
              <td><input aria-label={`${t('snapshot.value.unit')} ${index + 1}`} type="text" value={value.unit ?? ''}
                onChange={(event) => setValue(index, { unit: event.target.value })} /></td>
              <td><input aria-label={`${t('snapshot.value.path')} ${index + 1}`} type="text" placeholder="/heatingSystems/0/cop" value={value.path ?? ''}
                onChange={(event) => setValue(index, { path: event.target.value })} /></td>
              <td><button type="button" onClick={() => setValues(snapshot.values.filter((_, at) => at !== index))}>{t('snapshot.value.remove')}</button></td>
            </tr>
          ))}
        </tbody>
      </table>
      <button type="button" onClick={() => setValues([...snapshot.values, { label: '', value: '' }])}>{t('snapshot.value.add')}</button>
      <p className="dialog-hint" title={snapshot.valuesSha256}>SHA-256 {snapshot.valuesSha256.slice(0, 16)}…</p>
      <div className="dialog-field">
        <label htmlFor={id('confirmedBy')}>{t('snapshot.confirmedBy')}</label>
        <input id={id('confirmedBy')} type="text" value={snapshot.confirmedBy ?? ''}
          onChange={(event) => {
            // Another name is another confirmation: the date goes until the adviser confirms again.
            const { confirmedBy: _by, confirmedOn: _on, ...rest } = snapshot;
            onChange(event.target.value ? { ...rest, confirmedBy: event.target.value } : rest);
          }} />
      </div>
      <button type="button" disabled={!snapshot.confirmedBy?.trim() || status.confirmed}
        onClick={() => onChange({ ...snapshot, confirmedOn: today() })}>{t('snapshot.confirm')}</button>
      <p className="dialog-hint">{t('snapshot.confirmHint')}</p>
      <div className="dialog-field">
        <label htmlFor={id('registry')}>{t('snapshot.registryCheck')}</label>
        <select id={id('registry')} value={check.state}
          onChange={(event) => onChange({ ...snapshot, registryCheck: {
            ...check, state: event.target.value as typeof check.state,
            ...(event.target.value !== 'not_checked' && !check.checkedOn ? { checkedOn: today() } : {}),
          } })}>
          {REGISTRY_STATES.map((state) => <option key={state} value={state}>{t(`snapshot.registry.${state}`)}</option>)}
        </select>
      </div>
      {check.state !== 'not_checked' && (
        <div className="dialog-field">
          <label htmlFor={id('checkedOn')}>{t('snapshot.checkedOn')}</label>
          <input id={id('checkedOn')} type="date" value={check.checkedOn ?? ''}
            onChange={(event) => onChange({ ...snapshot, registryCheck: { ...check, checkedOn: event.target.value || undefined } })} />
        </div>
      )}
      <p className="dialog-hint">{t('snapshot.registryHint')}</p>
      <button type="button" onClick={() => onChange(undefined)}>{t('snapshot.remove')}</button>
    </fieldset>
  );
}
