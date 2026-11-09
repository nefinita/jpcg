import { afterEach, expect, it, vi } from "vitest";
import { fixture } from "../test/gear-fixture";
import { defaultManualForm } from "../state/cockpit";
import { toCalculateRequest } from "../utils/normalize";
import type { GearOptimizeRequest } from "../types/gear";

afterEach(() => { delete window.__TAURI__; vi.resetModules(); });

it("所有配装入口清理旧存档 sockets，且不修改原始配装或调优参数", async () => {
  const invoke = vi.fn().mockResolvedValue(null);
  window.__TAURI__ = { core: { invoke } } as unknown as typeof window.__TAURI__;
  const { composeGear, exportGear, optimizeGear } = await import("./commands");
  const loadout = { ...fixture.cases[0].composed.loadout, sockets: { tou: ["legacy-gem"] }, wucai: "selected-stone", enchants: { tou: "selected-enchant" } };
  const request: GearOptimizeRequest = {
    loadout, config: toCalculateRequest(defaultManualForm()),
    objective: { kind: "expected", weights: { "1": 1 } },
    constraints: { sources: ["weiwang"], allow_upgraded: true, locked: [], banned: [], required_sources: {}, banned_sources: {}, stats: {}, allow_cross_source_2plus2: true },
    max_combinations: 123,
  };
  const original = structuredClone(request);
  const { sockets: _legacy, ...clean } = loadout;
  await composeGear(loadout);
  await exportGear(loadout);
  await optimizeGear(request);
  expect(invoke.mock.calls).toEqual([
    ["compose_gear", { loadout: clean }],
    ["export_gear", { loadout: clean }],
    ["optimize_gear", { request: { ...request, loadout: clean } }],
  ]);
  expect(request).toEqual(original);
});
