import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import App from "./App";
import * as api from "./api/commands";
import { cockpitSchema, defaultCockpit, defaultManualForm, type CockpitState } from "./state/cockpit";
import { DEFAULT_COEFFICIENT, STORAGE_KEYS } from "./utils/constants";
import type { ValueSetDTO } from "./types";
import { toCalculateRequest } from "./utils/normalize";
import { fixture } from "./test/gear-fixture";

vi.mock("./api/commands");
vi.mock("./components/ForumPage", () => ({ default: () => <div>论坛</div> }));
vi.mock("./components/ComboPage", () => ({ default: () => <div>连招</div> }));
vi.mock("./components/OptimizePage", () => ({ default: () => <div>导数</div> }));
vi.mock("./components/SkillEditorPage", () => ({ default: () => <div>编辑器</div> }));

type ProfessionConfig = Awaited<ReturnType<typeof api.loadProfessionConfig>>;
const A: NonNullable<ProfessionConfig> = {
  profession: "mowen", xinfa_name: "莫问", xinfa_nom: "gengu",
  atk_up: 11, pofang_up: 12, huixin_up: 13,
};
const B: NonNullable<ProfessionConfig> = {
  profession: "qixiu", xinfa_name: "七秀坊", xinfa_nom: "shenfa",
  atk_up: 21, pofang_up: 22, huixin_up: 23,
};
const live: ValueSetDTO = {
  id: "live-130", name: "正式服130", level: 130, is_default: true,
  source: "data", available: true,
  coefficient: { ...DEFAULT_COEFFICIENT, huixin_xishu: 130 },
};

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: Error) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

let host: HTMLDivElement;
let root: Root;
function saved(): CockpitState { return JSON.parse(localStorage.getItem(STORAGE_KEYS.cockpit)!).data; }
async function mount() { await act(async () => root.render(<App />)); }
async function restart() {
  await act(async () => root.unmount());
  root = createRoot(host);
  await mount();
}
function button(text: string) {
  const found = [...host.querySelectorAll<HTMLButtonElement>("button")].find((node) => node.textContent === text);
  expect(found, `button ${text}`).toBeDefined();
  return found!;
}
async function click(text: string) { await act(async () => button(text).click()); }
async function selectXinfa(value: string) {
  await act(async () => {
    const select = host.querySelector<HTMLSelectElement>("select")!;
    select.value = value;
    select.dispatchEvent(new Event("change", { bubbles: true }));
  });
}
function numberInput(label: string) {
  return [...host.querySelectorAll("label")].find((node) => node.textContent === label)!
    .parentElement!.querySelector<HTMLInputElement>("input")!;
}
async function editNumber(label: string, value: string) {
  const input = numberInput(label);
  expect(input.disabled).toBe(false);
  await act(async () => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
}
async function editCoefficient(value: string) { await editNumber("会心系数", value); }
beforeEach(() => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  vi.resetAllMocks();
  localStorage.clear();
  const state = defaultCockpit();
  state.mode = "manual";
  localStorage.setItem(STORAGE_KEYS.cockpit, JSON.stringify({ version: cockpitSchema.version, data: state }));
  vi.mocked(api.gearCatalog).mockRejectedValue(new Error("No gear catalog in this manual-mode test"));
  vi.mocked(api.listProfessions).mockResolvedValue([]);
  vi.mocked(api.loadProfessionConfig).mockImplementation(async (xinfa) => xinfa === "qixiu" ? B : A);
  vi.mocked(api.getModuleVersions).mockResolvedValue({ app: "test", core: "test", combo: "test", update: "test", const: "test" });
  vi.mocked(api.listValueSets).mockResolvedValue([live]);
  vi.mocked(api.resolveValueSet).mockResolvedValue(live);
  vi.mocked(api.calculateDamage).mockResolvedValue([]);
  host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);
});
afterEach(async () => { await act(async () => root.unmount()); host.remove(); });

it.each(["心法先成功", "值集先成功"])("初始化两部分均成功后可恢复（%s）", async (order) => {
  const xinfa = deferred<ProfessionConfig>();
  const valueSet = deferred<ValueSetDTO>();
  vi.mocked(api.loadProfessionConfig).mockReturnValueOnce(xinfa.promise);
  vi.mocked(api.resolveValueSet).mockReturnValueOnce(valueSet.promise);
  await mount();
  if (order === "心法先成功") {
    await act(async () => xinfa.resolve(A));
    expect.soft(saved().manualReady).toBe(false);
    await act(async () => valueSet.resolve(live));
  } else {
    await act(async () => valueSet.resolve(live));
    expect.soft(saved().manualReady).toBe(false);
    await act(async () => xinfa.resolve(A));
  }
  expect(saved().manualReady).toBe(true);
  expect(saved().form.xinfa_config).toEqual(A);
  expect(saved().form.coefficient.huixin_xishu).toBe(130);
  const calls = vi.mocked(api.loadProfessionConfig).mock.calls.length;
  await restart();
  expect(api.loadProfessionConfig).toHaveBeenCalledTimes(calls);
  expect(saved().form.xinfa_config).toEqual(A);
  expect(saved().form.coefficient.huixin_xishu).toBe(130);
});

it.each([
  { outcome: "失败", custom: false }, { outcome: "挂起", custom: false },
  { outcome: "失败", custom: true }, { outcome: "挂起", custom: true },
])("已就绪 A 切 B 后$outcome，重启修复 B 属性并保留系数（自定义=$custom）", async ({ outcome, custom }) => {
  await mount();
  expect(saved().form.xinfa_config).toEqual(A);
  expect(saved().manualReady).toBe(true);
  if (custom) await editCoefficient("777");
  const pendingB = deferred<ProfessionConfig>();
  vi.mocked(api.loadProfessionConfig).mockReturnValueOnce(pendingB.promise);
  await selectXinfa("qixiu");
  if (outcome === "失败") await act(async () => pendingB.reject(new Error("B unavailable")));
  expect.soft(saved().manualReady).toBe(false);
  const calls = vi.mocked(api.loadProfessionConfig).mock.calls.length;
  await restart();
  expect.soft(api.loadProfessionConfig).toHaveBeenCalledTimes(calls + 1);
  expect.soft(saved().form.xinfa).toBe("qixiu");
  expect.soft(saved().form.xinfa_config).toEqual(B);
  expect.soft(saved().form.coefficient.huixin_xishu).toBe(custom ? 777 : 130);
  expect.soft(saved().manualReady).toBe(true);
});

it("切换心法未完成时，编辑普通属性和系数不能使该心法被当作已就绪", async () => {
  await mount();
  const pendingB = deferred<ProfessionConfig>();
  vi.mocked(api.loadProfessionConfig).mockReturnValueOnce(pendingB.promise);
  await selectXinfa("qixiu");
  await editNumber("基础攻击", "456");
  await editCoefficient("777");
  expect.soft(saved().manualReady).toBe(false);
  await restart();
  expect.soft(saved().form.xinfa_config).toEqual(B);
  expect.soft(saved().form.player.jichu_gongji).toBe(456);
  expect.soft(saved().form.coefficient.huixin_xishu).toBe(777);
  expect.soft(saved().manualReady).toBe(true);
});

it("值集切换失败后心法成功，不会把新值集的旧系数标记为已就绪", async () => {
  const next: ValueSetDTO = { ...live, id: "next-140", name: "体验140", is_default: false,
    coefficient: { ...DEFAULT_COEFFICIENT, huixin_xishu: 140 } };
  vi.mocked(api.listValueSets).mockResolvedValue([live, next]);
  await mount();
  await editCoefficient("777");
  const pendingValueSet = deferred<ValueSetDTO>();
  vi.mocked(api.resolveValueSet).mockReturnValueOnce(pendingValueSet.promise).mockResolvedValue(next);
  await act(async () => {
    const select = host.querySelectorAll<HTMLSelectElement>("select")[1];
    select.value = next.id;
    select.dispatchEvent(new Event("change", { bubbles: true }));
  });
  await act(async () => pendingValueSet.reject(new Error("new value set unavailable")));
  await selectXinfa("qixiu");
  expect.soft(saved().form.xinfa_config).toEqual(B);
  expect.soft(saved().manualReady).toBe(false);
  await restart();
  expect.soft(saved().form.value_set).toBe(next.id);
  expect.soft(saved().form.coefficient.huixin_xishu).toBe(140);
  expect.soft(saved().form.xinfa_config).toEqual(B);
  expect.soft(saved().manualReady).toBe(true);
});

it("当前 B 成功后迟到 A 元数据不回退已持久化的 B 就绪状态", async () => {
  const oldA = deferred<ProfessionConfig>();
  vi.mocked(api.loadProfessionConfig).mockReturnValueOnce(oldA.promise);
  await mount();
  await selectXinfa("qixiu");
  await editCoefficient("777");
  await act(async () => oldA.resolve(A));
  expect(saved().form.xinfa_config).toEqual(B);
  expect(saved().manualReady).toBe(true);
  await restart();
  expect(saved().form.xinfa_config).toEqual(B);
  expect(saved().form.coefficient.huixin_xishu).toBe(777);
});

it.each(["心法成功后值集失败", "值集失败后心法成功"])("部分初始化不会把占位系数当自定义（%s）", async (order) => {
  const xinfa = deferred<ProfessionConfig>();
  const valueSet = deferred<ValueSetDTO>();
  vi.mocked(api.loadProfessionConfig).mockReturnValueOnce(xinfa.promise);
  vi.mocked(api.resolveValueSet).mockReturnValueOnce(valueSet.promise);
  await mount();
  if (order === "心法成功后值集失败") {
    await act(async () => xinfa.resolve(A));
    await act(async () => valueSet.reject(new Error("value set unavailable")));
  } else {
    await act(async () => valueSet.reject(new Error("value set unavailable")));
    await act(async () => xinfa.resolve(A));
  }
  expect.soft(saved().manualReady).toBe(false);
  expect(saved().form.coefficient.huixin_xishu).toBe(DEFAULT_COEFFICIENT.huixin_xishu);
  await restart();
  expect.soft(saved().form.coefficient.huixin_xishu).toBe(130);
  expect.soft(numberInput("会心系数").value).toBe("130");
  expect.soft(saved().form.xinfa_config).toEqual(A);
  expect.soft(saved().manualReady).toBe(true);
});

it.each([false, true])("值集先就绪而心法失败，重启只重试心法（提前自定义=%s）", async (custom) => {
  const xinfa = deferred<ProfessionConfig>();
  vi.mocked(api.loadProfessionConfig).mockReturnValueOnce(xinfa.promise);
  await mount();
  expect(saved().form.coefficient.huixin_xishu).toBe(130);
  if (custom) await editCoefficient("777");
  await act(async () => xinfa.reject(new Error("A unavailable")));
  expect(saved().manualReady).toBe(false);
  await restart();
  expect.soft(api.loadProfessionConfig).toHaveBeenCalledTimes(2);
  expect.soft(saved().form.xinfa_config).toEqual(A);
  expect.soft(saved().form.coefficient.huixin_xishu).toBe(custom ? 777 : 130);
  expect.soft(saved().manualReady).toBe(true);
});

it.each(["首次加载", "切换心法"])("心法尚未就绪时禁止保存和计算（%s）", async (stage) => {
  const pending = deferred<ProfessionConfig>();
  if (stage === "首次加载") vi.mocked(api.loadProfessionConfig).mockReturnValueOnce(pending.promise);
  await mount();
  if (stage === "切换心法") {
    vi.mocked(api.loadProfessionConfig).mockReturnValueOnce(pending.promise);
    await selectXinfa("qixiu");
  }
  expect.soft(button("保存").disabled).toBe(true);
  expect.soft(button("开始计算").disabled).toBe(true);
  await click("保存");
  await click("开始计算");
  expect.soft(api.saveConfig).not.toHaveBeenCalled();
  expect.soft(api.calculateDamage).not.toHaveBeenCalled();
});

it("初始化挂起时加载完整存档，随后重启仍保留该心法及自定义系数", async () => {
  const pendingXinfa = deferred<ProfessionConfig>();
  const pendingStartupValueSet = deferred<ValueSetDTO>();
  const pendingLoadedValueSet = deferred<ValueSetDTO>();
  vi.mocked(api.loadProfessionConfig).mockReturnValueOnce(pendingXinfa.promise);
  vi.mocked(api.resolveValueSet).mockReturnValueOnce(pendingStartupValueSet.promise)
    .mockReturnValueOnce(pendingLoadedValueSet.promise);
  const form = defaultManualForm();
  form.xinfa = "qixiu";
  form.xinfa_config = { profession: "qixiu", xinfa_name: "七秀坊", xinfa_nom: "shenfa", atk_up: 31, pofang_up: 32, huixin_up: 33 };
  form.coefficient.huixin_xishu = 999;
  form.value_set = live.id;
  vi.mocked(api.loadConfig).mockResolvedValue(toCalculateRequest(form));
  await mount();
  await click("加载");
  expect.soft(saved().manualReady).toBe(true);
  await act(async () => pendingXinfa.resolve(A));
  await act(async () => pendingStartupValueSet.resolve(live));
  expect(saved().form.xinfa_config).toEqual(form.xinfa_config);
  const calls = vi.mocked(api.loadProfessionConfig).mock.calls.length;
  await restart();
  expect.soft(api.loadProfessionConfig).toHaveBeenCalledTimes(calls);
  expect.soft(saved().form.xinfa_config).toEqual(form.xinfa_config);
  expect.soft(saved().form.coefficient.huixin_xishu).toBe(999);
});

it("清空后值集解析失败，重启补齐系数并保留明确清空的心法属性", async () => {
  await mount();
  await editCoefficient("777");
  vi.mocked(api.resolveValueSet).mockRejectedValueOnce(new Error("clear value set unavailable"));
  await click("清空");
  expect.soft(saved().manualReady).toBe(false);
  expect(saved().form.xinfa_config.atk_up).toBe(0);
  const calls = vi.mocked(api.loadProfessionConfig).mock.calls.length;
  await restart();
  expect.soft(api.loadProfessionConfig).toHaveBeenCalledTimes(calls);
  expect.soft(saved().form.coefficient.huixin_xishu).toBe(130);
  expect.soft(saved().form.xinfa_config.atk_up).toBe(0);
  expect.soft(saved().manualReady).toBe(true);
});

it("切换模式重新挂载未就绪心法时重试元数据，不丢已自定义系数", async () => {
  await mount();
  await editCoefficient("777");
  const pendingB = deferred<ProfessionConfig>();
  vi.mocked(api.loadProfessionConfig).mockReturnValueOnce(pendingB.promise);
  await selectXinfa("qixiu");
  await click("配装驱动");
  await click("手动属性");
  expect.soft(saved().form.xinfa_config).toEqual(B);
  expect.soft(saved().form.coefficient.huixin_xishu).toBe(777);
  expect.soft(saved().manualReady).toBe(true);
});

it.each(["心法", "系数", "两者"])("同心法配装导入补齐未就绪的%s，不把占位值持久化为就绪", async (pendingPart) => {
  const state = saved();
  state.mode = "gear";
  localStorage.setItem(STORAGE_KEYS.cockpit, JSON.stringify({ version: cockpitSchema.version, data: state }));
  vi.mocked(api.gearCatalog).mockResolvedValue(fixture.catalog);
  vi.mocked(api.composeGear).mockResolvedValue(fixture.cases[0]);
  vi.mocked(api.importGear).mockResolvedValue(fixture.cases[0]);
  const pendingXinfa = deferred<ProfessionConfig>();
  const pendingValueSet = deferred<ValueSetDTO>();
  if (pendingPart !== "系数") vi.mocked(api.loadProfessionConfig).mockReturnValueOnce(pendingXinfa.promise);
  if (pendingPart !== "心法") vi.mocked(api.resolveValueSet).mockReturnValueOnce(pendingValueSet.promise);
  await mount();
  const file = new File(["fixture"], "loadout.toml");
  Object.defineProperty(file, "text", { value: async () => "fixture" });
  const input = host.querySelector<HTMLInputElement>('input[type="file"]')!;
  expect(input).not.toBeNull();
  Object.defineProperty(input, "files", { configurable: true, value: [file] });
  await act(async () => input.dispatchEvent(new Event("change", { bubbles: true })));
  expect.soft(api.importGear).toHaveBeenCalledWith("fixture");
  expect.soft(saved().form.xinfa_config).toEqual(A);
  expect.soft(saved().form.coefficient.huixin_xishu).toBe(130);
  expect.soft(saved().manualReady).toBe(true);
  await act(async () => pendingXinfa.resolve({ ...A, atk_up: 91 }));
  await act(async () => pendingValueSet.resolve({ ...live, coefficient: { ...DEFAULT_COEFFICIENT, huixin_xishu: 91 } }));
  expect.soft(saved().form.xinfa_config).toEqual(A);
  expect.soft(saved().form.coefficient.huixin_xishu).toBe(130);
  await restart();
  expect.soft(saved().form.xinfa_config).toEqual(A);
  expect.soft(saved().form.coefficient.huixin_xishu).toBe(130);
});

it("同一批次完成心法回填和导入，不重复请求已成功的心法", async () => {
  const state = saved(); state.mode = "gear";
  localStorage.setItem(STORAGE_KEYS.cockpit, JSON.stringify({ version: cockpitSchema.version, data: state }));
  vi.mocked(api.gearCatalog).mockResolvedValue(fixture.catalog);
  vi.mocked(api.composeGear).mockResolvedValue(fixture.cases[0]);
  const startup = deferred<ProfessionConfig>();
  const imported = deferred<Awaited<ReturnType<typeof api.importGear>>>();
  vi.mocked(api.loadProfessionConfig).mockReturnValueOnce(startup.promise).mockResolvedValue(null);
  vi.mocked(api.importGear).mockReturnValueOnce(imported.promise);
  await mount();
  const file = new File(["fixture"], "loadout.toml");
  Object.defineProperty(file, "text", { value: async () => "fixture" });
  const input = host.querySelector<HTMLInputElement>('input[type="file"]')!;
  Object.defineProperty(input, "files", { configurable: true, value: [file] });
  await act(async () => input.dispatchEvent(new Event("change", { bubbles: true })));
  await act(async () => { startup.resolve(A); imported.resolve(fixture.cases[1]); });
  expect.soft(api.loadProfessionConfig).toHaveBeenCalledTimes(1);
  expect.soft(saved().loadouts.mowen.choices.find((choice) => choice.slot === "jie1")!.variant).toBe(2);
  expect.soft(saved().form.xinfa_config).toEqual(A);
});
