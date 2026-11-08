import { expect, it } from 'vitest';
import { cockpitSchema, defaultCockpit } from './state/cockpit';
import { readPersistentState } from './hooks/usePersistentState';
import { fixture } from './test/gear-fixture';
import * as api from './api/commands';
it.each(['combo', 'baseline'])('旧 v2 %s 迁移保留输入和配装，仅丢弃旧连招缓存', (field) => {
 const state: any = defaultCockpit();
 state.form.player.jichu_gongji = 777;
 state.loadouts.mowen = fixture.cases[0].composed.loadout;
 const result = { steps: [], total_expected_damage_wan: 12, final_kill_prob: .5, kill_prob_curve: [] };
 if (field === 'combo') state.combo = { inputKey: 'old', result };
 else state.baseline = { form: state.form, results: [], comboResult: result };
 localStorage.setItem('review-probe', JSON.stringify({version: 2, data: state}));
 const restored = readPersistentState('review-probe', defaultCockpit, cockpitSchema);
 expect(restored.form.player.jichu_gongji).toBe(777);
 expect(restored.loadouts).toEqual(state.loadouts);
 expect(restored.combo).toBeUndefined();
 if (field === "baseline") expect(restored.baseline).toMatchObject({ form: state.form, results: [] });
 localStorage.removeItem('review-probe');
});
it('浏览器预览战斗包使用完整模拟技能池',async()=>{
 const b=await api.loadCombatBundle('mowen');
 expect(b.skills.length).toBeGreaterThan(0);
 expect(b.skills).toEqual(await api.loadSkillPool("mowen"));
});
