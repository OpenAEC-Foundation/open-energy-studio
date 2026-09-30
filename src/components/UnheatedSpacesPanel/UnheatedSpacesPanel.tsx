import { useState } from 'react';
import { useEnergy } from '../../context/EnergyContext';
import { useI18n } from '../../i18n/i18n';
import './UnheatedSpacesPanel.css';

export function UnheatedSpacesPanel() {
  const { state, dispatch } = useEnergy();
  const { t } = useI18n();
  const spaces = state.project.unheatedSpaces ?? [];
  const [editingId, setEditingId] = useState<string | null>(null);
  const [name, setName] = useState('');
  const [factor, setFactor] = useState('');
  const [source, setSource] = useState('');
  const [error, setError] = useState(false);
  const edit = (id: string | null) => {
    const selected = spaces.find((space) => space.id === id);
    setEditingId(id ?? '');
    setName(selected?.name ?? '');
    setFactor(selected ? String(selected.reductionFactor) : '');
    setSource(selected?.factorSourceReference ?? '');
    setError(false);
  };
  const save = () => {
    const value = Number(factor);
    if (!name.trim() || !factor.trim() || !Number.isFinite(value) || value < 0 || value > 1 || !source.trim()) {
      setError(true);
      return;
    }
    const next = { id: editingId || crypto.randomUUID(), name: name.trim(), reductionFactor: value, factorSourceReference: source.trim() };
    dispatch({ type: 'SET_UNHEATED_SPACES', payload: editingId
      ? spaces.map((space) => space.id === editingId ? next : space)
      : [...spaces, next] });
    setEditingId(null);
  };
  return <section className="unheated-spaces" aria-label={t('kernel.unheated.title')}>
    <div className="unheated-spaces-head">
      <div><h3>{t('kernel.unheated.title')}</h3><p>{t('kernel.unheated.scope')}</p></div>
      <button type="button" onClick={() => edit(null)}>{t('kernel.unheated.add')}</button>
    </div>
    {spaces.length === 0 && <p>{t('kernel.unheated.empty')}</p>}
    {spaces.length > 0 && <ul>{spaces.map((space) => <li key={space.id}>
      <span><strong>{space.name}</strong><small>{t('kernel.unheated.factor')}: {space.reductionFactor} · {space.factorSourceReference}</small></span>
      <button type="button" onClick={() => edit(space.id)}>{t('kernel.inventory.edit')}</button>
      <button type="button" onClick={() => dispatch({ type: 'SET_UNHEATED_SPACES', payload: spaces.filter((item) => item.id !== space.id) })}
        aria-label={`${t('kernel.inventory.remove')}: ${space.id}`}>×</button>
    </li>)}</ul>}
    {editingId !== null && <div className="unheated-spaces-form">
      <label>{t('properties.name')}<input value={name} onChange={(event) => setName(event.target.value)} /></label>
      <label>{t('kernel.unheated.factor')}<input type="number" min="0" max="1" step="any" value={factor}
        onChange={(event) => setFactor(event.target.value)} /></label>
      <label>{t('kernel.unheated.source')}<input value={source} onChange={(event) => setSource(event.target.value)} /></label>
      {error && <p role="alert">{t('kernel.unheated.invalid')}</p>}
      <div><button type="button" onClick={() => setEditingId(null)}>{t('dialog.cancel')}</button>
        <button type="button" onClick={save}>{t('dialog.save')}</button></div>
    </div>}
  </section>;
}
