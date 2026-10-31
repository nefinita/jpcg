import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import App from "./App";
import * as api from "./api/commands";
import { fixture } from "./test/gear-fixture";
import { defaultComboDraft, defaultCockpit } from "./state/cockpit";
import { STORAGE_KEYS, DEFAULT_COEFFICIENT } from "./utils/constants";
import type { GearComposition, GearLoadout } from "./types/gear";
import type { SkillResultDTO } from "./types";
import { PROFESSION_ATTRIBUTE_RULES, gearInputKey } from "./state/gear";
vi.mock("./api/commands");
vi.mock("./components/ForumPage", () => ({ default: () => <div>论坛</div> }));
vi.mock("./components/ComboPage", () => ({ default: ({ draft, onDraftChange }: any) => <button onClick={() => onDraftChange((previous: any) => ({ ...previous, sequence: [...previous.sequence, previous.sequence[0]] }))}>追加连招</button> }));
vi.mock("./components/OptimizePage", () => ({ default: () => <div>导数</div> }));
vi.mock("./components/SkillEditorPage", () => ({ default: () => <div>编辑器</div> }));
const result = (q: number): SkillResultDTO[] => [{ skill_name: "测试", y: 1, b: 1, i: 1, n: q, h: q, q, dot_jumps: [], has_critical_strike: false, zhenshishanghai: 0, lost_hp_zhenshishanghai: 0 }];
let host: HTMLDivElement; let root: Root;
const wait = async (ms = 180) => { await act(async () => { await new Promise((resolve) => setTimeout(resolve, ms)); }); };
async function render() { await act(async () => root.render(<App />)); }
async function click(text: string, scope: ParentNode = host) { const button = [...scope.querySelectorAll<HTMLButtonElement>("button")].find((b) => b.textContent === text)!; expect(button, text).toBeDefined(); await act(async () => button.click()); }
function section(name: string) { return host.querySelector(`[aria-label="${name}"]`)!; }
function saved() { return JSON.parse(localStorage.getItem(STORAGE_KEYS.cockpit)!).data; }
beforeEach(() => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true }); vi.resetAllMocks(); localStorage.clear();
  const state = defaultCockpit(); state.form.value_set = "live-130"; state.manualReady = true;
  state.loadouts.mowen = fixture.cases[0].composed.loadout;
  localStorage.setItem(STORAGE_KEYS.cockpit, JSON.stringify({ version: 2, data: state }));
  vi.mocked(api.gearCatalog).mockResolvedValue(fixture.catalog);
  vi.mocked(api.composeGear).mockImplementation(async (loadout) => ({ ...fixture.cases[loadout.choices.find((c) => c.slot === "jie1")?.variant === 2 ? 1 : 0], composed: { ...fixture.cases[0].composed, loadout } }));
  vi.mocked(api.calculateDamage).mockResolvedValue(result(100));
  vi.mocked(api.listProfessions).mockResolvedValue([]); vi.mocked(api.loadProfessionConfig).mockResolvedValue(null);
  vi.mocked(api.getModuleVersions).mockResolvedValue({ app: "test", core: "test", combo: "test", update: "test", const: "test" });
  vi.mocked(api.listValueSets).mockResolvedValue([{ id: "live-130", name: "正式服", level: 130, is_default: true, available: true, source: "data", coefficient: DEFAULT_COEFFICIENT }]);
  vi.mocked(api.resolveValueSet).mockImplementation(async (id) => ({ id: id ?? "live-130", name: "正式服", level: 130, is_default: true, available: true, source: "data", coefficient: DEFAULT_COEFFICIENT }));
  host = document.createElement("div"); document.body.appendChild(host); root = createRoot(host);
});
afterEach(async () => { await act(async () => root.unmount()); host.remove(); });
it("12 槽按真实目录展示，改配装防抖自动合成/计算，切模式保留手动属性", async () => {
  await render(); await click("配装驱动"); await wait(); expect(api.calculateDamage).toHaveBeenCalledTimes(1); expect(host.querySelectorAll("article")).toHaveLength(12);
  expect(host.textContent).toContain("合成测试样例"); expect(api.composeGear).toHaveBeenCalledTimes(1);
  await click("手动属性");
  const input = [...host.querySelectorAll("label")].find((l) => l.textContent === "基础攻击")!.parentElement!.querySelector("input")!;
  await act(async () => { Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, "456"); input.dispatchEvent(new Event("input", { bubbles: true })); });
  await click("配装驱动"); await wait();
  const count = vi.mocked(api.composeGear).mock.calls.length;
  await click("威望 · 基础 · 属性套2", section("戒指一"));
  expect(api.composeGear).toHaveBeenCalledTimes(count); await wait(); expect(api.composeGear).toHaveBeenCalledTimes(count + 1);
  expect(saved().loadouts.mowen.choices.find((c: { slot: string }) => c.slot === "jie1").variant).toBe(2);
  await click("手动属性"); expect(input.isConnected).toBe(false);
  expect(saved().form.player.jichu_gongji).toBe(456);
});
it("配装基准包含选择与上下文，恢复后结果不被自动重算覆盖；重启无需重复计算", async () => {
  await render(); await wait(); await click("钉住基准");
  await click("威望 · 基础 · 属性套2", section("戒指一")); vi.mocked(api.calculateDamage).mockResolvedValue(result(150)); await wait();
  expect(host.textContent).toContain("+50 / +50.0%");
  await click("恢复基准"); await wait(); expect(host.textContent).toContain("+0 / +0.0%");
  expect(saved().loadouts.mowen.choices.find((c: { slot: string }) => c.slot === "jie1").variant).toBe(1);
  const count = vi.mocked(api.composeGear).mock.calls.length;
  await act(async () => root.unmount()); root = createRoot(host); await render(); await wait();
  expect(api.composeGear).toHaveBeenCalledTimes(count); expect(host.textContent).toContain("+0 / +0.0%");
});
it("迟到合成响应不覆盖较新的配装；版本/值集不匹配不会计算", async () => {
  let resolve!: (value: GearComposition) => void;
  vi.mocked(api.composeGear).mockReturnValueOnce(new Promise((done) => { resolve = done; }));
  await render(); await wait(); await click("威望 · 基础 · 属性套2", section("戒指一")); await wait();
  await act(async () => resolve(fixture.cases[0])); expect(api.calculateDamage).toHaveBeenCalledTimes(1);
  expect(saved().calculation.loadout.choices.find((c: { slot: string }) => c.slot === "jie1").variant).toBe(2);
  await act(async () => root.unmount()); root = createRoot(host);
  const state = saved(); state.loadouts.mowen.revision = "old"; localStorage.setItem(STORAGE_KEYS.cockpit, JSON.stringify({ version: 2, data: state }));
  const calls = vi.mocked(api.composeGear).mock.calls.length;
  await render(); await wait(); expect(api.composeGear).toHaveBeenCalledTimes(calls); expect(host.textContent).toContain("保存的配装与当前模板版本不一致");
});
it("合成失败显示错误且保留上次结果，TOML 导入调用宿主后再改变配装", async () => {
  await render(); await wait();
  vi.mocked(api.composeGear).mockRejectedValue(new Error("数据损坏"));
  await click("威望 · 基础 · 属性套2", section("戒指一")); await wait();
  expect(host.textContent).toContain("数据损坏"); expect(saved().calculation.results[0].q).toBe(100);
  vi.mocked(api.importGear).mockResolvedValue(fixture.cases[0]);
  const file = new File(["contract"], "loadout.toml"); Object.defineProperty(file, "text", { value: async () => "contract" });
  const input = host.querySelector<HTMLInputElement>('input[type="file"]')!;
  Object.defineProperty(input, "files", { value: [file] });
  await act(async () => input.dispatchEvent(new Event("change", { bubbles: true })));
  expect(api.importGear).toHaveBeenCalledWith("contract");
  expect(saved().loadouts.mowen.choices).toEqual(fixture.cases[0].composed.loadout.choices);
});

it("连招在驾驶台计算，配置页不计算；钉住恢复连招草稿与击杀率", async () => {
  const draft = defaultComboDraft();
  draft.sequence = [{ skill: { skill_name: "测试连招", skill_id: 1, sub_id: 0, base_damage1: 1, base_damage2: 1, atk_xishu: 1, watk_xishu: 0, hit_up: 0, huixin_up: 0, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0, has_critical_strike: false, lost_hp_zhenshishanghai: 0 }, overrides: null }];
  localStorage.setItem(`${STORAGE_KEYS.comboDraft}:mowen`, JSON.stringify({ version: 1, data: draft }));
  vi.mocked(api.calculateCombo).mockResolvedValue({ steps: [], total_expected_damage_wan: 10, final_kill_prob: .5, kill_prob_curve: [] });
  await render(); await wait(); await wait();
  expect(section("连招结果").textContent).toContain("击杀率 50.0%"); await click("钉住基准");
  await act(async () => host.querySelector<HTMLButtonElement>('button[title="排轴器"]')!.click());
  await click("追加连招"); await wait(); expect(api.calculateCombo).toHaveBeenCalledTimes(1);
  vi.mocked(api.calculateCombo).mockResolvedValue({ steps: [], total_expected_damage_wan: 12, final_kill_prob: .8, kill_prob_curve: [] });
  await act(async () => host.querySelector<HTMLButtonElement>('button[title="计算"]')!.click()); await wait();
  expect(section("连招结果").textContent).toContain("Δ +30.0 个百分点");
  const count = vi.mocked(api.calculateCombo).mock.calls.length;
  await click("恢复基准"); await wait();
  expect(section("连招结果").textContent).toContain("击杀率 50.0%");
  expect(api.calculateCombo).toHaveBeenCalledTimes(count);
  expect(JSON.parse(localStorage.getItem(`${STORAGE_KEYS.comboDraft}:mowen`)!).data.sequence).toHaveLength(1);
});

it("实际解析回退到其他值集时不提交配装计算", async () => {
  vi.mocked(api.resolveValueSet).mockResolvedValue({ id: "fallback", name: "回退", level: 130, is_default: true, available: true, source: "builtin", coefficient: DEFAULT_COEFFICIENT });
  await render(); await wait(); expect(api.composeGear).not.toHaveBeenCalled();
  await click("使用模板数值集 live-130"); expect(host.textContent).toContain("模板对应数值集不可用");
  expect(api.calculateDamage).not.toHaveBeenCalled();
});

it("规则选择后将宿主换算攻击与加速送到计算链，重启保留选择且不改手动值", async () => {
  const originalManual = saved().form.player;
  vi.mocked(api.composeGear).mockImplementation(async (loadout) => ({
    ...fixture.cases[0], composed: { ...fixture.cases[0].composed, loadout },
    player: { ...fixture.cases[0].player, jiasu_dengji: 135, ...(loadout.attribute_rules ? { zhuanhua_gongji: 2007 } : {}) },
  }));
  await render(); await wait();
  const select = host.querySelector<HTMLSelectElement>('select[aria-label="属性换算规则"]')!;
  await act(async () => { select.value = PROFESSION_ATTRIBUTE_RULES; select.dispatchEvent(new Event("change", { bubbles: true })); });
  await wait();
  expect(vi.mocked(api.composeGear).mock.lastCall?.[0].attribute_rules).toBe(PROFESSION_ATTRIBUTE_RULES);
  expect(vi.mocked(api.calculateDamage).mock.lastCall?.[0].player).toMatchObject({ zhuanhua_gongji: 2007, jiasu_dengji: 135 });
  expect(saved().form.player).toEqual(originalManual);
  expect(saved().loadouts.mowen.attribute_rules).toBe(PROFESSION_ATTRIBUTE_RULES);
  await act(async () => root.unmount()); root = createRoot(host); await render(); await wait();
  expect(host.querySelector<HTMLSelectElement>('select[aria-label="属性换算规则"]')!.value).toBe(PROFESSION_ATTRIBUTE_RULES);
  expect(saved().form.player).toEqual(originalManual);
});

it("职业 TOML 缺少换算规则时显示宿主错误，不猜系数继续计算", async () => {
  await render(); await wait();
  const previousCalls = vi.mocked(api.calculateDamage).mock.calls.length;
  const originalManual = saved().form.player;
  vi.mocked(api.composeGear).mockRejectedValue(new Error("职业 TOML 缺少 [attributes]，请填写并验证50级规则"));
  const select = host.querySelector<HTMLSelectElement>('select[aria-label="属性换算规则"]')!;
  await act(async () => { select.value = PROFESSION_ATTRIBUTE_RULES; select.dispatchEvent(new Event("change", { bubbles: true })); });
  await wait();
  expect(host.textContent).toContain("职业 TOML 缺少 [attributes]");
  expect(vi.mocked(api.calculateDamage).mock.calls).toHaveLength(previousCalls);
  expect(saved().form.player).toEqual(originalManual);
  expect(saved().loadouts.mowen.attribute_rules).toBe(PROFESSION_ATTRIBUTE_RULES);
});

it("历史规则即使有匹配缓存也不能继续驱动伤害或连招，保留输入等待重新选择", async () => {
  const state = saved();
  const loadout = { ...state.loadouts.mowen, attribute_rules: "jx3-130-mowen-20260814" };
  state.loadouts.mowen = loadout;
  state.calculation = { mode: "gear", form: state.form, manualForm: state.form, loadout,
    composition: { ...fixture.cases[0], composed: { ...fixture.cases[0].composed, loadout } },
    inputKey: gearInputKey(state.form, loadout), results: result(100) };
  localStorage.setItem(STORAGE_KEYS.cockpit, JSON.stringify({ version: 2, data: state }));
  const draft = defaultComboDraft();
  draft.sequence = [{ skill: { skill_name: "缓存校验", skill_id: 1, sub_id: 0, base_damage1: 1, base_damage2: 1, atk_xishu: 1, watk_xishu: 0, hit_up: 0, huixin_up: 0, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0, has_critical_strike: false, lost_hp_zhenshishanghai: 0 }, overrides: null }];
  localStorage.setItem(`${STORAGE_KEYS.comboDraft}:mowen`, JSON.stringify({ version: 1, data: draft }));
  await render(); await wait(); await wait();
  expect(host.textContent).toContain("已保存规则已不受支持");
  expect(api.composeGear).not.toHaveBeenCalled();
  expect(api.calculateDamage).not.toHaveBeenCalled();
  expect(api.calculateCombo).not.toHaveBeenCalled();
  expect(saved().loadouts.mowen.attribute_rules).toBe("jx3-130-mowen-20260814");
});

function persistProfessionSnapshot() {
  const state = saved();
  const loadout = { ...state.loadouts.mowen, attribute_rules: PROFESSION_ATTRIBUTE_RULES };
  const composition: GearComposition = { ...fixture.cases[0], composed: { ...fixture.cases[0].composed, loadout },
    player: { ...fixture.cases[0].player, zhuanhua_gongji: 111 } };
  state.loadouts.mowen = loadout;
  state.calculation = { mode: "gear", form: { ...state.form, player: composition.player }, manualForm: state.form, loadout,
    composition, inputKey: gearInputKey(state.form, loadout), results: result(100) };
  localStorage.setItem(STORAGE_KEYS.cockpit, JSON.stringify({ version: 2, data: state }));
  const draft = defaultComboDraft();
  draft.sequence = [{ skill: { skill_name: "职业规则缓存校验", skill_id: 1, sub_id: 0, base_damage1: 1, base_damage2: 1, atk_xishu: 1, watk_xishu: 0, hit_up: 0, huixin_up: 0, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0, has_critical_strike: false, lost_hp_zhenshishanghai: 0 }, overrides: null }];
  localStorage.setItem(`${STORAGE_KEYS.comboDraft}:mowen`, JSON.stringify({ version: 1, data: draft }));
  return composition;
}

it("重启的职业规则缓存需重新合成；手动重读立即停用旧结果，失败后不再驱动连招", async () => {
  const old = persistProfessionSnapshot();
  let resolveComposition!: (value: GearComposition) => void;
  vi.mocked(api.composeGear).mockReturnValueOnce(new Promise((resolve) => { resolveComposition = resolve; }));
  vi.mocked(api.calculateCombo).mockResolvedValue({ steps: [], total_expected_damage_wan: 10, final_kill_prob: .5, kill_prob_curve: [] });
  await render(); await wait();
  expect(api.composeGear).toHaveBeenCalledTimes(1);
  expect(api.calculateDamage).not.toHaveBeenCalled();
  expect(api.calculateCombo).not.toHaveBeenCalled();
  expect(section("连招结果").textContent).toContain("先完成当前属性计算");
  await act(async () => resolveComposition({ ...old, player: { ...old.player, zhuanhua_gongji: 777 } }));
  await wait();
  expect(vi.mocked(api.calculateDamage).mock.lastCall?.[0].player.zhuanhua_gongji).toBe(777);
  expect(api.calculateCombo).toHaveBeenCalledTimes(1);
  const damageCalls = vi.mocked(api.calculateDamage).mock.calls.length;
  let resolveCatalog!: (value: typeof fixture.catalog) => void;
  vi.mocked(api.gearCatalog).mockReturnValueOnce(new Promise((resolve) => { resolveCatalog = resolve; }));
  vi.mocked(api.composeGear).mockRejectedValue(new Error("职业规则校验失败：等级不匹配"));
  await click("重新读取职业规则");
  // 目录读取尚未完成时，旧 DTO 就已不能送给连招。
  expect(section("连招结果").textContent).toContain("先完成当前属性计算");
  expect(api.composeGear).toHaveBeenCalledTimes(1);
  await act(async () => resolveCatalog(fixture.catalog));
  await wait(); await wait();
  expect(api.composeGear).toHaveBeenCalledTimes(2);
  expect(host.textContent).toContain("职业规则校验失败：等级不匹配");
  expect(vi.mocked(api.calculateDamage).mock.calls).toHaveLength(damageCalls);
  expect(api.calculateCombo).toHaveBeenCalledTimes(1);
  expect(section("连招结果").textContent).toContain("先完成当前属性计算");
});

it("已保存的职业规则结果不能掩盖启动时缺失规则；失败不自动循环重试", async () => {
  persistProfessionSnapshot();
  vi.mocked(api.composeGear).mockRejectedValue(new Error("职业 TOML 缺少 [attributes]"));
  await render(); await wait(); await wait();
  expect(api.composeGear).toHaveBeenCalledTimes(1);
  expect(api.calculateDamage).not.toHaveBeenCalled();
  expect(api.calculateCombo).not.toHaveBeenCalled();
  expect(host.textContent).toContain("职业 TOML 缺少 [attributes]");
  expect(saved().calculation.results[0].q).toBe(100);
  expect(section("连招结果").textContent).toContain("先完成当前属性计算");
});
