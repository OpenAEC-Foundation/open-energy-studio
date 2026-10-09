/**
 * New project in one window (feedback 9 Oct 2026): what you are going to do,
 * the address from the BAG, the dwelling type (houseboat and caravan as their
 * own choice) and what the building has. Only what is still open comes back
 * as a question in the flow.
 */
import { useState, type ReactNode } from 'react';
import { Building, Building2, Caravan, Home, HousePlus, Search, Ship } from 'lucide-react';
import { Button, Dialog } from '../ui';
import { useI18n } from '../../i18n/i18n';
import { lookupAddresses, normalizePostcode, withBuildingData, type BagAddress } from '../../core/io/bagLookup';
import {
  SETUP_DWELLINGS, SETUP_HEATING, SETUP_VENTILATION, isSurveyKind, nameFromAddress, rememberAdviser, rememberedAdviser,
  type NewProjectKind, type NewProjectSetup, type SetupAdviser, type SetupDwelling,
} from '../../core/io/newProjectSetup';
import { EXAMPLE_KINDS, type ExampleKind } from '../../core/nta/ExampleProjects';

const KINDS: Array<{ id: NewProjectKind; icon: ReactNode }> = [
  { id: 'existing_residential', icon: <Home /> },
  { id: 'existing_utility', icon: <Building2 /> },
  { id: 'residential', icon: <HousePlus /> },
  { id: 'utility', icon: <Building /> },
];
const DWELLING_ICONS: Partial<Record<SetupDwelling, ReactNode>> = {
  apartment: <Building2 aria-hidden="true" />, houseboat: <Ship aria-hidden="true" />, houseboat_2018: <Ship aria-hidden="true" />, caravan: <Caravan aria-hidden="true" />,
};

type Lookup = { state: 'idle' } | { state: 'busy' } | { state: 'none' } | { state: 'error' } | { state: 'choose'; options: BagAddress[] };

/** A row of toggle chips; picking the selected one again clears it. */
function Chips<T extends string>({ label, options, value, onChange, text }: {
  label: string; options: readonly T[]; value: T | null | undefined; onChange: (value: T | null) => void; text: (id: T) => ReactNode;
}) {
  return <div className="new-project__chips" role="group" aria-label={label}>
    {options.map((id) => <button key={id} type="button" className="new-project__chip" aria-pressed={value === id}
      onClick={() => onChange(value === id ? null : id)}>{text(id)}</button>)}
  </div>;
}

/** Present / not present / not known yet. */
function Presence({ label, value, onChange }: { label: string; value: boolean | null | undefined; onChange: (value: boolean | null) => void }) {
  const { t } = useI18n();
  return <div className="new-project__presence">
    <span>{label}</span>
    <Chips label={label} options={['yes', 'no'] as const} value={value == null ? null : value ? 'yes' : 'no'}
      onChange={(next) => onChange(next == null ? null : next === 'yes')} text={(id) => t(`newProject.${id}`)} />
  </div>;
}

export function NewProjectDialog({ onCreate, onOpenExample, onClose }: {
  onCreate: (setup: NewProjectSetup) => void;
  onOpenExample?: (kind: ExampleKind) => void;
  onClose: () => void;
}) {
  const { t } = useI18n();
  const [kind, setKind] = useState<NewProjectKind>('existing_residential');
  const [postcode, setPostcode] = useState('');
  const [number, setNumber] = useState('');
  const [addition, setAddition] = useState('');
  const [address, setAddress] = useState<BagAddress | null>(null);
  const [manual, setManual] = useState({ street: '', city: '' });
  const [lookup, setLookup] = useState<Lookup>({ state: 'idle' });
  const [dwelling, setDwelling] = useState<SetupDwelling | null>(null);
  const [roofType, setRoofType] = useState<NonNullable<NewProjectSetup['roofType']>>('pitched');
  const [apartmentFloor, setApartmentFloor] = useState<NonNullable<NewProjectSetup['apartmentFloor']>>('ground_or_intermediate');
  const [heating, setHeating] = useState<string | null>(null);
  const [ventilation, setVentilation] = useState<string | null>(null);
  const [cooling, setCooling] = useState<boolean | null>(null);
  const [pv, setPv] = useState<boolean | null>(null);
  const [adviser, setAdviser] = useState<SetupAdviser>(() => rememberedAdviser() ?? { name: '', competenceNumber: '', certificateNumber: '' });
  const survey = isSurveyKind(kind);
  const dwellingKind = kind === 'existing_residential';

  const search = async () => {
    setAddress(null);
    if (!normalizePostcode(postcode) || !/^\d+$/.test(number.trim())) { setLookup({ state: 'none' }); return; }
    setLookup({ state: 'busy' });
    try {
      const found = await lookupAddresses(postcode, number, addition);
      if (found.length === 1) { setAddress(found[0]); setLookup({ state: 'idle' }); }
      else setLookup(found.length === 0 ? { state: 'none' } : { state: 'choose', options: found });
    } catch {
      setLookup({ state: 'error' });
    }
  };
  const choose = async (option: BagAddress) => {
    setLookup({ state: 'busy' });
    setAddress(await withBuildingData(option));
    setLookup({ state: 'idle' });
  };
  // Without a BAG match the address can be typed in.
  const typedAddress = (): BagAddress | null => {
    const code = normalizePostcode(postcode);
    if (!manual.street.trim() && !code) return null;
    return {
      display: '', street: manual.street.trim(), houseNumber: number.trim(), addition: addition.trim(),
      postcode: code ?? postcode.trim(), city: manual.city.trim(), bagObjectId: '',
    };
  };
  const create = () => {
    const chosen = address ?? typedAddress();
    if (adviser.name.trim()) rememberAdviser(adviser);
    onCreate({
      kind, address: chosen,
      ...(dwellingKind ? { dwelling, roofType, apartmentFloor } : {}),
      ...(survey ? { heating, ventilation, cooling, pv } : {}),
      adviser: adviser.name.trim() ? adviser : null,
    });
  };
  const showManual = lookup.state === 'none' || lookup.state === 'error';

  return <Dialog title={t('newProject.title')} onClose={onClose} width={720} className="new-project"
    footer={<>
      {onOpenExample && <span className="new-project__examples">{t('newProject.examples')}{' '}
        {EXAMPLE_KINDS.map((example, index) => <span key={example}>{index > 0 && ' · '}
          <button type="button" className="link-button" onClick={() => onOpenExample(example)}>{t(`welcome.example.${example}`)}</button>
        </span>)}
      </span>}
      <Button onClick={onClose}>{t('newProject.cancel')}</Button>
      <Button variant="primary" onClick={create}>{t('newProject.create')}</Button>
    </>}>
    <section className="new-project__section">
      <h3>{t('newProject.what')}</h3>
      <div className="new-project__kinds" role="group" aria-label={t('newProject.what')}>
        {KINDS.map((item) => <button key={item.id} type="button" className="new-project__kind" aria-pressed={kind === item.id}
          onClick={() => setKind(item.id)}>
          <span aria-hidden="true">{item.icon}</span>{t(`newProject.kind.${item.id}`)}
        </button>)}
      </div>
    </section>

    <section className="new-project__section">
      <h3>{t('newProject.address')}</h3>
      <p className="new-project__hint">{t('newProject.addressHint')}</p>
      <form className="new-project__address" onSubmit={(event) => { event.preventDefault(); void search(); }}>
        <label>{t('survey.address.postcode')}<input value={postcode} onChange={(event) => setPostcode(event.target.value)} autoComplete="postal-code" /></label>
        <label>{t('surveyReg.field.houseNumber')}<input value={number} onChange={(event) => setNumber(event.target.value)} inputMode="numeric" /></label>
        <label>{t('surveyReg.field.houseNumberAddition')}<input value={addition} onChange={(event) => setAddition(event.target.value)} /></label>
        <Button type="submit" icon={<Search aria-hidden="true" />} loading={lookup.state === 'busy'}>{t('newProject.lookup')}</Button>
      </form>
      {address && <p className="new-project__found" role="status">
        <strong>{address.display || nameFromAddress(address)}</strong>
        {[address.constructionYear && t('newProject.found.year', { year: String(address.constructionYear) }),
          address.floorAreaM2 && t('newProject.found.area', { area: String(address.floorAreaM2) }),
          address.purpose,
          address.unitsInBuilding && address.unitsInBuilding > 1 && t('newProject.found.units', { count: String(address.unitsInBuilding) }),
        ].filter(Boolean).map((part) => <span key={String(part)}> · {part}</span>)}
      </p>}
      {lookup.state === 'choose' && <div className="new-project__choose" role="group" aria-label={t('newProject.chooseAddress')}>
        <span>{t('newProject.chooseAddress')}</span>
        {lookup.options.map((option) => <button key={option.bagObjectId || option.display} type="button" className="new-project__chip"
          onClick={() => { void choose(option); }}>{option.display}</button>)}
      </div>}
      {showManual && <>
        <p className="new-project__warn" role="status">{t(lookup.state === 'error' ? 'newProject.lookupError' : 'newProject.notFound')}</p>
        <div className="new-project__address">
          <label className="new-project__wide">{t('survey.address.street')}<input value={manual.street} onChange={(event) => setManual({ ...manual, street: event.target.value })} /></label>
          <label className="new-project__wide">{t('survey.address.city')}<input value={manual.city} onChange={(event) => setManual({ ...manual, city: event.target.value })} /></label>
        </div>
      </>}
    </section>

    {dwellingKind && <section className="new-project__section">
      <h3>{t('newProject.dwelling')}</h3>
      <Chips label={t('newProject.dwelling')} options={SETUP_DWELLINGS} value={dwelling} onChange={setDwelling}
        text={(id) => <>{DWELLING_ICONS[id]}{t(`newProject.dwelling.${id}`)}</>} />
      {dwelling === 'apartment' && <label className="new-project__inline">{t('opname.dwelling.floor')}
        <select value={apartmentFloor} onChange={(event) => setApartmentFloor(event.target.value as typeof apartmentFloor)}>
          {(['ground_or_intermediate', 'top', 'roof_and_floor'] as const).map((key) => <option key={key} value={key}>{t(`opname.dwelling.floorKind.${key}`)}</option>)}
        </select></label>}
      {dwelling != null && dwelling !== 'apartment' && <label className="new-project__inline">{t('survey.dwelling.roofType')}
        <select value={roofType} onChange={(event) => setRoofType(event.target.value as typeof roofType)}>
          {(['pitched', 'partly_flat', 'flat'] as const).map((key) => <option key={key} value={key}>{t(`survey.dwelling.roofTypeKind.${key}`)}</option>)}
        </select></label>}
    </section>}

    {survey && <section className="new-project__section">
      <h3>{t('newProject.has')}</h3>
      <p className="new-project__hint">{t('newProject.hasHint')}</p>
      <div className="new-project__presence">
        <span>{t('newProject.heating')}</span>
        <Chips label={t('newProject.heating')} options={SETUP_HEATING} value={heating as typeof SETUP_HEATING[number] | null} onChange={setHeating}
          text={(id) => t(`opname.heating.kind.${id}`)} />
      </div>
      <div className="new-project__presence">
        <span>{t('newProject.ventilation')}</span>
        <Chips label={t('newProject.ventilation')} options={SETUP_VENTILATION} value={ventilation as typeof SETUP_VENTILATION[number] | null} onChange={setVentilation}
          text={(id) => t(`survey.ventilation.principleKind.${id}`)} />
      </div>
      <Presence label={t('newProject.cooling')} value={cooling} onChange={setCooling} />
      <Presence label={t('newProject.pv')} value={pv} onChange={setPv} />
    </section>}

    <section className="new-project__section">
      <h3>{t('survey.address.adviser')}</h3>
      <div className="new-project__adviser">
        <label>{t('newProject.adviserName')}<input value={adviser.name} onChange={(event) => setAdviser({ ...adviser, name: event.target.value })} /></label>
        <label>{t('newProject.adviserNumber')}<input value={adviser.competenceNumber} onChange={(event) => setAdviser({ ...adviser, competenceNumber: event.target.value })} /></label>
        <label title={t('survey.address.certificate')}>{t('newProject.certificate')}<input value={adviser.certificateNumber} onChange={(event) => setAdviser({ ...adviser, certificateNumber: event.target.value })} /></label>
      </div>
    </section>
  </Dialog>;
}
