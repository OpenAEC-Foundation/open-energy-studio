import { useI18n } from '../../i18n/i18n';
import { NumberField, read, TextField, type Draft, type Path } from './NtaFormFields';

// Ψ of distribution pipes from their geometry: 9.33–9.35 (heating),
// 10.24–10.26 (cooling) and 13.27–13.29 (hot water) are the same three
// formulas, for an insulated pipe in air, an insulated pipe embedded in the
// construction and an uninsulated pipe.

interface Props {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
  /** Path of the geometry object (`{ method, ... }`). */
  base: Path;
}

export const PIPE_METHODS = ['insulated_in_air', 'insulated_embedded', 'uninsulated'] as const;

/** An empty geometry of one method, with the kernel's field names. */
export function pipeGeometryTemplate(method: string = 'insulated_embedded'): Draft {
  switch (method) {
    case 'insulated_in_air':
      return { method, pipeOuterDiameterM: null, insulatedDiameterM: null, insulationLambdaWPerMK: null };
    case 'uninsulated':
      return { method, innerDiameterM: null, outerDiameterM: null, pipeLambdaWPerMK: null };
    default:
      return {
        method: 'insulated_embedded', pipeOuterDiameterM: null, insulatedDiameterM: null,
        insulationLambdaWPerMK: null, embeddingLambdaWPerMK: null, depthM: null,
      };
  }
}

export function PipeGeometryFields({ draft, change, base }: Props) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  // No geometry yet (e.g. just switched to "calculated"): only the method
  // choice, which writes a complete template, so no field is written into
  // an object without its `method` tag.
  const present = read(draft, base) != null;
  const method = present ? String(read(draft, [...base, 'method']) ?? '') : '';
  const at = (name: string): Path => [...base, name];
  return <fieldset className="nta-form-row">
    <legend>{t('nta.form.pipeGeometry.title')}</legend>
    <label>{t('nta.form.pipeGeometry.method')}
      <select value={method} onChange={(event) => change(base, pipeGeometryTemplate(event.target.value))}>
        {!present && <option value="">—</option>}
        {PIPE_METHODS.map((value) => <option key={value} value={value}>{t(`nta.form.pipeGeometry.method.${value}`)}</option>)}
      </select>
    </label>
    {present && <>
    {method === 'uninsulated' ? <>
      <NumberField {...field} path={at('innerDiameterM')} label={t('nta.form.pipeGeometry.innerDiameter')} step="0.001" />
      <NumberField {...field} path={at('outerDiameterM')} label={t('nta.form.pipeGeometry.outerDiameter')} step="0.001" />
      <NumberField {...field} path={at('pipeLambdaWPerMK')} label={t('nta.form.pipeGeometry.pipeLambda')} />
    </> : <>
      <NumberField {...field} path={at('pipeOuterDiameterM')} label={t('nta.form.pipeGeometry.pipeDiameter')} step="0.001" />
      <NumberField {...field} path={at('insulatedDiameterM')} label={t('nta.form.pipeGeometry.insulatedDiameter')} step="0.001" />
      <NumberField {...field} path={at('insulationLambdaWPerMK')} label={t('nta.form.pipeGeometry.insulationLambda')} step="0.001" />
    </>}
    {method === 'insulated_embedded' && <>
      <NumberField {...field} path={at('embeddingLambdaWPerMK')} label={t('nta.form.pipeGeometry.embeddingLambda')} />
      <NumberField {...field} path={at('depthM')} label={t('nta.form.pipeGeometry.depth')} step="0.001" />
    </>}
    </>}
  </fieldset>;
}

/** 13.27–13.29: a calculated Ψ for the hot-water circulation, when present. */
export function CirculationPsiFields({ draft, change }: Omit<Props, 'base'>) {
  const { t } = useI18n();
  const circulation = read(draft, ['hotWater', 'circulation']);
  if (circulation == null) return null;
  const base: Path = ['hotWater', 'circulation', 'calculatedPsi'];
  const calculated = read(draft, base) != null;
  return <>
    <label className="nta-form-check" data-path="ntaCalculation.hotWater.circulation.calculatedPsi">
      <input type="checkbox" checked={calculated}
        onChange={(event) => change(base, event.target.checked ? pipeGeometryTemplate() : null)} />
      {t('nta.form.pipeGeometry.circulation')}
    </label>
    {calculated && <PipeGeometryFields draft={draft} change={change} base={base} />}
  </>;
}

/** §9.1 with 9.85: A, B, C and B_nom of a boiler from a quality declaration. */
export function DeclaredAuxiliaryConstantsFields({ draft, change, base }: Props) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const declared = read(draft, base) != null;
  return <>
    <label className="nta-form-check">
      <input type="checkbox" checked={declared}
        onChange={(event) => change(base, event.target.checked
          ? { aKwh: null, bKw: null, c: null, nominalLoadKw: null, declarationReference: '' }
          : null)} />
      {t('nta.form.boilerAux.declared')}
    </label>
    {declared && <div className="nta-form-row">
      <NumberField {...field} path={[...base, 'aKwh']} label={t('nta.form.boilerAux.a')} />
      <NumberField {...field} path={[...base, 'bKw']} label={t('nta.form.boilerAux.b')} step="0.001" />
      <NumberField {...field} path={[...base, 'c']} label={t('nta.form.boilerAux.c')} step="0.1" />
      <NumberField {...field} path={[...base, 'nominalLoadKw']} label={t('nta.form.boilerAux.nominalLoad')} />
      <TextField {...field} path={[...base, 'declarationReference']} label={t('nta.form.boilerAux.reference')} />
      <p className="nta-form-note">{t('nta.form.boilerAux.note')}</p>
    </div>}
  </>;
}

/** §9.1 with 9.65: ε_chp;th and ε_chp;el of a CHP from a quality declaration. */
export function DeclaredChpEfficienciesFields({ draft, change, base }: Props) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const declared = read(draft, base) != null;
  return <>
    <label className="nta-form-check">
      <input type="checkbox" checked={declared}
        onChange={(event) => change(base, event.target.checked
          ? { thermal: null, electric: null, declarationReference: '' }
          : null)} />
      {t('nta.form.chp.declared')}
    </label>
    {declared && <div className="nta-form-row">
      <NumberField {...field} path={[...base, 'thermal']} label={t('nta.form.chp.declaredThermal')} step="0.01" />
      <NumberField {...field} path={[...base, 'electric']} label={t('nta.form.chp.declaredElectric')} step="0.01" />
      <TextField {...field} path={[...base, 'declarationReference']} label={t('nta.form.chp.declaredReference')} />
    </div>}
  </>;
}

