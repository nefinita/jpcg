import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import GearPanel from "./GearPanel";
import { fixture } from "../test/gear-fixture";
import { MOWEN_ATTRIBUTE_RULES, isGearComposition } from "../state/gear";
import type { GearComposition, GearLoadout } from "../types/gear";

let host: HTMLDivElement;
let root: Root;
beforeEach(() => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  host = document.createElement("div"); document.body.appendChild(host); root = createRoot(host);
});
afterEach(async () => { await act(async () => root.unmount()); host.remove(); });
const onChange = vi.fn();
async function render(loadout: GearLoadout, composition?: GearComposition) {
  onChange.mockReset();
  await act(async () => root.render(<GearPanel catalog={fixture.catalog} loadout={loadout} composition={composition}
    onChange={onChange} onReset={vi.fn()} onExport={vi.fn()} onImport={async () => {}} pending={false} />));
}
function statInput(name: string) {
  return [...host.querySelectorAll("label")].find((label) => label.textContent === name)!.querySelector("input")!;
}

it("旧存档补齐五维编辑框，选择版本后可输入体质并保留原配装", async () => {
  const legacy = fixture.cases[0].composed.loadout;
  await render(legacy);
  expect(statInput("体质").value).toBe("0");
  expect(statInput("体质").disabled).toBe(true);
  const select = host.querySelector<HTMLSelectElement>('select[aria-label="属性换算规则"]')!;
  await act(async () => { select.value = MOWEN_ATTRIBUTE_RULES; select.dispatchEvent(new Event("change", { bubbles: true })); });
  const selected = onChange.mock.calls[0][0] as GearLoadout;
  expect(selected).toEqual({ ...legacy, attribute_rules: MOWEN_ATTRIBUTE_RULES });
  await render(selected);
  expect(statInput("体质").disabled).toBe(false);
  await act(async () => {
    const input = statInput("体质");
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, "222");
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
  expect(onChange.mock.calls[0][0].base.tizhi).toBe(222);
  expect(onChange.mock.calls[0][0].choices).toEqual(legacy.choices);
  expect(host.textContent).toContain("不可直接抄裸装最终面板");
  expect(host.textContent).toContain("同一来源只填写一次");
});

it("显示宿主返回的换算面板，不在前端重算或声称通过实测", async () => {
  const original = fixture.cases[0];
  const composed = { ...original.composed, loadout: { ...original.composed.loadout, attribute_rules: MOWEN_ATTRIBUTE_RULES }, panel: {
    rule_id: MOWEN_ATTRIBUTE_RULES, source: "test arithmetic fixture", verified_in_game: false,
    tizhi: 222, liliang: 0, shenfa: 0, gengu: 3, yuanqi: 0,
    gongji_base: 100, gongji_extra: 5, gongji: 105, huixin: 3, pofang: 20, qixue: 2885, qixue_base: 2785, qixue_additional: 100,
  } };
  const composition = { ...original, composed, player: { ...original.player, zhuanhua_gongji: 5 } };
  expect(isGearComposition(composition)).toBe(true);
  for (const damage_kind of ["physical", "neutral", "lunar"]) {
    expect(isGearComposition({ ...composition, composed: { ...composed, panel: { ...composed.panel, damage_kind } } })).toBe(true);
  }
  expect(isGearComposition({ ...composition, composed: { ...composed, panel: { ...composed.panel, damage_kind: 123 } } })).toBe(false);
  expect(isGearComposition({ ...composition, composed: { ...composed, panel: { ...composed.panel, qixue: "wrong" } } })).toBe(false);
  expect(isGearComposition({ ...composition, composed: { ...composed, panel: { ...composed.panel, qixue_additional: "wrong" } } })).toBe(false);
  const { qixue_base: _base, qixue_additional: _additional, ...olderPanel } = composed.panel;
  expect(isGearComposition({ ...composition, composed: { ...composed, panel: olderPanel } })).toBe(true);
  await render(composed.loadout, composition);
  const panel = host.querySelector('[aria-label="换算后玩家属性"]')!;
  expect(panel.textContent).toContain("气血 2885");
  expect(panel.textContent).toContain("攻击 105（基础 100 + 转化 5）");
  expect(panel.textContent).toContain("会心等级 3");
  expect(panel.textContent).toContain("基础换算后 2785 + 额外气血上限 100");
  expect(panel.textContent).toContain("尚未通过真实游戏面板验证");
});

it("其他心法不能选择莫问规则，未知已保存规则保留以供修正", async () => {
  const loadout = { ...fixture.cases[0].composed.loadout, school: "bingxin" };
  await render(loadout);
  expect(host.querySelector(`option[value="${MOWEN_ATTRIBUTE_RULES}"]`)).toBeNull();
  await render({ ...loadout, attribute_rules: "future-rules" });
  expect(host.querySelector<HTMLSelectElement>("select")!.value).toBe("future-rules");
  expect(host.textContent).toContain("当前范围不支持");
});

it.each([
  ["10026", "傲血战意", "jx3-130-aoxue-20260814"],
  ["10390", "分山劲", "jx3-130-fenshan-20260814"],
  ["10021", "花间游", "jx3-130-huajian-20260814"],
])("心法 %s 仅展示对应历史规则 %s，不生成不存在的装备", async (school, name, rule) => {
  const loadout = { ...fixture.cases[0].composed.loadout, school };
  await render(loadout);
  const select = host.querySelector<HTMLSelectElement>('select[aria-label="属性换算规则"]')!;
  expect([...select.options].map((option) => option.value)).toEqual(["", rule]);
  expect(select.options[1].textContent).toContain(`130 级${name}`);
  expect(host.textContent).toContain("模板不可用");
  await act(async () => { select.value = rule; select.dispatchEvent(new Event("change", { bubbles: true })); });
  expect(onChange.mock.calls[0][0]).toEqual({ ...loadout, attribute_rules: rule });
});

it("其他数值集不提供历史130规则，也不把已保存的不适用规则称为已核对", async () => {
  const loadout = { ...fixture.cases[0].composed.loadout, school: "10021", value_set: "experience-140" };
  await render(loadout);
  expect(host.querySelector<HTMLSelectElement>("select")!.options).toHaveLength(1);
  await render({ ...loadout, attribute_rules: "jx3-130-huajian-20260814" });
  expect(host.textContent).toContain("已保存规则不适用于当前心法或数值集");
  expect(host.textContent).not.toContain("参考规则已核对来源");
});

it("已填写五维后切回旧模式仍可清零，不会困在禁用输入框里", async () => {
  const loadout = { ...fixture.cases[0].composed.loadout, attribute_rules: MOWEN_ATTRIBUTE_RULES,
    base: { ...fixture.cases[0].composed.loadout.base, tizhi: 222, gengu: 3, qixue_additional: 100 } };
  await render(loadout);
  const select = host.querySelector<HTMLSelectElement>('select[aria-label="属性换算规则"]')!;
  await act(async () => { select.value = ""; select.dispatchEvent(new Event("change", { bubbles: true })); });
  let current = onChange.mock.calls[0][0] as GearLoadout;
  expect(current.base.tizhi).toBe(222);
  await render(current);
  expect(host.querySelector('[role="alert"]')!.textContent).toContain("清零后再计算");
  for (const name of ["体质", "根骨", "额外气血上限"]) {
    const input = statInput(name);
    expect(input.disabled).toBe(false);
    await act(async () => {
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, "0");
      input.dispatchEvent(new Event("input", { bubbles: true }));
    });
    current = onChange.mock.calls[0][0] as GearLoadout;
    await render(current);
  }
  expect(current.base.tizhi).toBe(0);
  expect(current.base.gengu).toBe(0);
  expect(current.base.qixue_additional).toBe(0);
  expect(host.querySelector('[role="alert"]')).toBeNull();
});
