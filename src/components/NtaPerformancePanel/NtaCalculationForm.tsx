import { useState } from 'react';
import type { IProject } from '../../core/energy/types';
import { useI18n } from '../../i18n/i18n';
import { FieldPathPrefixProvider, Section, write, type Draft, type Path } from './NtaFormFields';
import { syncVentilation } from '../../core/nta/NtaFormModels';
import { nullPaths } from '../../core/nta/KernelInput';
import { NTA_SECTIONS, type NtaSectionDef, type NtaSectionProps } from './NtaSections';

// The block is edited as plain JSON data; the Rust kernel is the validator.
// Since F6 the sections live in the register `NtaSections` and are edited on
// their workflow step; this form shows all of them under Controle › NTA-invoer.
export {
  GroundEdgeBridgesFields, setpointChecks, setpointWriteBack, table713Setpoints, TABLE_713_SOURCE,
  type SetpointCheckRow,
} from './NtaSections';

/** Kernel path of the draft root; every field carries `data-path` below it. */
export const NTA_PATH_PREFIX = 'ntaCalculation';

/** One section of the register: its fieldset (or its own fieldsets when bare). */
export function NtaSection({ def, ...props }: NtaSectionProps & { def: NtaSectionDef }) {
  const { t } = useI18n();
  const { Component } = def;
  if (def.bare) return <Component {...props} />;
  const [first, ...rest] = def.paths.map((path) => `${NTA_PATH_PREFIX}.${path}`);
  return <Section title={t(def.titleKey)} path={first} extraPaths={rest}>
    <Component {...props} />
  </Section>;
}

export function NtaCalculationForm({ project, initial, onSave, onCancel }: {
  project: IProject;
  initial: Draft;
  onSave: (block: Draft) => void;
  onCancel: () => void;
}) {
  const { t } = useI18n();
  const [draft, setDraft] = useState<Draft>(initial);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  const emptyPaths = nullPaths(draft);
  const props: NtaSectionProps = { draft, change, update: setDraft, project };

  return <form className="nta-form" aria-label={t('nta.form.title')} onSubmit={(event) => {
    event.preventDefault();
    onSave(syncVentilation(draft, project));
  }}>
    <p>{t('nta.form.help')}</p>
    <FieldPathPrefixProvider value={NTA_PATH_PREFIX}>
      {NTA_SECTIONS.filter((def) => !def.when || def.when(project, draft))
        .map((def) => <NtaSection key={def.id} def={def} {...props} />)}
    </FieldPathPrefixProvider>
    {emptyPaths.length > 0 && <p className="nta-form-note" role="status" data-testid="nta-open-fields">
      {t('nta.form.openFields', { n: emptyPaths.length, paths: emptyPaths.join(', ') })}</p>}
    <div className="nta-form-actions nta-form-footer">
      <button type="button" className="btn" onClick={onCancel}>{t('dialog.cancel')}</button>
      <button type="submit" className="btn btn-primary">{t('dialog.save')}</button>
    </div>
  </form>;
}
