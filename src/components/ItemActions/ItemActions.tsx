import { useState } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { deleteTarget, editAction } from '../../core/energy/projectItems';
import './ItemActions.css';

/** Edit and delete buttons for one project item; delete asks for confirmation inline. */
export function ItemActions({ itemType, id, name }: { itemType: string; id: string; name: string }) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const [confirming, setConfirming] = useState(false);
  const [blocked, setBlocked] = useState<string[] | null>(null);
  const edit = editAction(itemType, id);

  const requestDelete = () => {
    const target = deleteTarget(state.project, itemType, id);
    if (target.kind === 'blocked') { setBlocked(target.usedBy); return; }
    if (target.kind === 'none') return;
    setBlocked(null);
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
      <span className="item-actions-confirm" role="alert">{t('item.deleteConfirm', { name })}</span>
      <button type="button" className="btn btn-sm btn-danger" onClick={confirmDelete}>{t('item.deleteYes')}</button>
      <button type="button" className="btn btn-sm" onClick={() => setConfirming(false)}>{t('dialog.cancel')}</button>
    </>}
    {blocked && <span className="item-actions-blocked" role="alert">
      {t('item.constructionInUse', { count: blocked.length, list: blocked.join(', ') })}
    </span>}
  </span>;
}
