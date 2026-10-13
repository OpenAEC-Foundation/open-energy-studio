/**
 * Survey input against the kernel (audit 13 Oct 2026): a BAG floor area counts
 * only once checked, a part cleared elsewhere shows off, and photo links of a
 * part switched off go with its data.
 */
import { describe, expect, it } from 'vitest';
import {
  BAG_AREA_CHECKED_SOURCE, BAG_AREA_SOURCE, missingSurveyAnswers, surveyTemplate, type StoredSurvey,
} from '../core/nta/SurveyTemplates';
import { componentActive, toggleComponent } from '../core/survey/surveyComponents';
import { linksAfterClear } from '../core/nta/EvidenceLinks';
import type { NtaEvidenceItem } from '../core/nta/KernelClient';

const residential = (): StoredSurvey => ({ ...surveyTemplate('residential') } as StoredSurvey);

describe('survey input against the kernel', () => {
  it('counts a BAG floor area only after the adviser checked it', () => {
    const stored = residential();
    const survey = stored.survey as Record<string, unknown>;
    survey.usableFloorAreaM2 = 145;
    survey.areaSourceReference = BAG_AREA_SOURCE;
    expect(missingSurveyAnswers(stored)).toContain('usableFloorAreaM2');
    survey.areaSourceReference = BAG_AREA_CHECKED_SOURCE;
    expect(missingSurveyAnswers(stored)).not.toContain('usableFloorAreaM2');
  });

  it('shows a ticked part off when its data was cleared elsewhere', () => {
    const on = toggleComponent(residential(), 'solarWater', true);
    expect(componentActive(on, 'solarWater')).toBe(true);
    // A checkbox in the full panel empties the list but leaves the tick.
    const cleared = { ...on, survey: { ...on.survey, hotWater: { ...(on.survey.hotWater as object), solar: [] } } };
    expect(cleared.components).toContain('solarWater');
    expect(componentActive(cleared, 'solarWater')).toBe(false);
  });

  it('drops the photo links into a part that was switched off', () => {
    const project = { basisopname: { survey: { hotWater: { solar: [{ id: 'zon-1' }] }, heating: { generator: {} } } } };
    const evidence = [
      { fileName: 'zonneboiler.jpg', linkedPaths: ['/basisopname/survey/hotWater/solar/0'] },
      { fileName: 'ketel.jpg', linkedPaths: ['/basisopname/survey/heating/generator'] },
      { fileName: 'beide.jpg', linkedPaths: ['/basisopname/survey/hotWater/solar/0/collector', '/basisopname/survey/heating/generator'] },
    ] as unknown as NtaEvidenceItem[];
    const kept = linksAfterClear(evidence, project, '/basisopname/survey/hotWater/solar');
    expect(kept[0].linkedPaths).toBeUndefined();
    expect(kept[1]).toBe(evidence[1]);
    expect(kept[2].linkedPaths).toEqual(['/basisopname/survey/heating/generator']);
  });
});
