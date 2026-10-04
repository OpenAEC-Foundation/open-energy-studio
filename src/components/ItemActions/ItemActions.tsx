import { useState } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { deleteTarget, editAction } from '../../core/energy/projectItems';
import { containedItems, describeCascade } from '../../core/energy/cascadeLabels';
import './ItemActions.css';

/** Edit and delete buttons for one project item; delete asks for confirmation inline. */
export function ItemActions({ itemType, id, name }: { itemType: string; id: string; name: string }) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const [confirming, setConfirming] = useState(false);
  const [blocked, setBlocked] = useState<{ reason: 'constructionInUse' | 'manualMeasures'; usedBy: string[] } | null>(null);
  const [cascade, setCascade] = useState<Array<{ path: string; label: string }>>([]);
  const [contained, setContained] = useState({ surfaces: 0, windows: 0 });
  const [buildingMeasures, setBuildingMeasures] = useState<string[]>([]);
  const edit = editAction(itemType, id);

  const requestDelete = () => {
    const target = deleteTarget(state.project, itemType, id);
    if (target.kind === 'blocked') { setBlocked({ reason: target.reason, usedBy: target.usedBy }); return; }
    if (target.kind === 'none') return;
    setBlocked(null);
    setCascade(target.cascade.map((entry) => ({ path: entry.path, label: describeCascade(entry, state.project, t) })));
    setContained(containedItems(state.project, itemType, id));
    setBuildingMeasures(target.buildingMeasures);
    setConfirming(true);
  };
  const confirmDelete = () => {
    const target = deleteTarget(state.project, itemType, id);
    if (target.kind === 'action') {
      dispatch(target.action);
      if (state.selectedItemId === id) dispatch({ type: 'DESELECT_ITEM' });
    }
    setConfirming(false);
  };

  return <span className="item-actions" onClick={(event) => event.stopPropagation()}
    onDoubleClick={(event) => event.stopPropagation()}>
    {edit && <button type="button" className="btn btn-sm" aria-label={`${t('dialog.edit')}: ${name}`}
      onClick={() => dispatch(edit)}>{t('dialog.edit')}</button>}
    {!confirming && <button type="button" className="btn btn-sm btn-danger" aria-label={`${t('dialog.delete')}: ${name}`}
      onClick={requestDelete}>{t('dialog.delete')}</button>}
    {confirming && <>
      <span className="item-actions-confirm" role="alert">
        {t('item.deleteConfirm', { name })}
        {(contained.surfaces > 0 || contained.windows > 0) && <>
          {' '}{t('item.deleteContained', { surfaces: contained.surfaces, windows: contained.windows })}
        </>}
        {cascade.length > 0 && <>
          {' '}{t('item.deleteCascade', { count: cascade.length })}
          <ul className="item-actions-cascade">{cascade.map((entry) => <li key={entry.path} title={entry.path}>{entry.label}</li>)}</ul>
        </>}
        {buildingMeasures.length > 0 && <>
          {' '}{t('item.buildingMeasuresReview', { list: buildingMeasures.join(', ') })}
        </>}
      </span>
      <button type="button" className="btn btn-sm btn-danger" onClick={confirmDelete}>{t('item.deleteYes')}</button>
      <button type="button" className="btn btn-sm" onClick={() => setConfirming(false)}>{t('dialog.cancel')}</button>
    </>}
    {blocked && <span className="item-actions-blocked" role="alert">
      {blocked.reason === 'constructionInUse'
        ? t('item.constructionInUse', { count: blocked.usedBy.length, list: blocked.usedBy.join(', ') })
        : t('item.manualMeasuresShift', { count: blocked.usedBy.length, list: blocked.usedBy.join(', ') })}
    </span>}
  </span>;
}
