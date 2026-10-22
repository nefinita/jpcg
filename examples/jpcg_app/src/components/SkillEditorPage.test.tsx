import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import SkillEditorPage from "./SkillEditorPage";
import * as api from "../api/commands";
import type { SkillEditorDataDTO } from "../types";
vi.mock("../api/commands", () => ({ listProfessions: vi.fn(), loadSkillData: vi.fn(), saveSkillData: vi.fn() }));
const data = (profession: string): SkillEditorDataDTO => ({ xinfa: { profession, xinfa_name: profession, xinfa_nom: "根骨", atk_up: 1, pofang_up: 2, huixin_up: 0 }, version: null, skills: [] });
function deferred() { let resolve!: (v: SkillEditorDataDTO) => void; let reject!: (e: Error) => void; const promise = new Promise<SkillEditorDataDTO>((a,b) => { resolve=a; reject=b; }); return {promise,resolve,reject}; }
let root: Root, container: HTMLDivElement;
const toast = vi.fn();
beforeEach(() => {
  vi.resetAllMocks();
  (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
  vi.mocked(api.listProfessions).mockResolvedValue(["mowen", "huajian"].map(value => ({value,label:value,nom:"根骨",version_label:null})));
  vi.mocked(api.saveSkillData).mockResolvedValue(undefined);
  container=document.createElement("div"); document.body.appendChild(container); root=createRoot(container);
});
afterEach(async () => { await act(async () => root.unmount()); container.remove(); });
const save = () => Array.from(container.querySelectorAll("button")).find(b=>b.textContent==="保存")!;
async function render() { await act(async () => root.render(<SkillEditorPage addToast={toast} />)); }
async function select(value:string) { await act(async () => { const s=container.querySelector("select")!; s.value=value; s.dispatchEvent(new Event("change",{bubbles:true})); }); }
it("切换心法加载期间不能把前一心法的数据保存到新路径", async () => {
 const b=deferred(); vi.mocked(api.loadSkillData).mockResolvedValueOnce(data("mowen")).mockReturnValueOnce(b.promise);
 await render(); await select("huajian"); expect(save().disabled).toBe(true);
 await act(async()=>save().click()); expect(api.saveSkillData).not.toHaveBeenCalled();
 await act(async()=>b.resolve(data("huajian"))); await act(async()=>save().click());
 expect(api.saveSkillData).toHaveBeenCalledWith("huajian",data("huajian"));
});
it("新心法加载失败后不能保存旧数据", async () => {
 vi.mocked(api.loadSkillData).mockResolvedValueOnce(data("mowen")).mockRejectedValueOnce(new Error("failed"));
 await render(); await select("huajian"); expect(save().disabled).toBe(true);
 await act(async()=>save().click()); expect(api.saveSkillData).not.toHaveBeenCalled();
});
it("迟到的旧心法响应不能覆盖新心法并写到新路径", async () => {
 const a=deferred(),b=deferred(); vi.mocked(api.loadSkillData).mockReturnValueOnce(a.promise).mockReturnValueOnce(b.promise);
 await render(); await select("huajian"); await act(async()=>b.resolve(data("huajian"))); await act(async()=>a.resolve(data("mowen")));
 await act(async()=>save().click()); expect(api.saveSkillData).toHaveBeenCalledWith("huajian",data("huajian"));
});
it("旧请求失败不清除当前加载状态或报告过期错误", async () => {
 const a=deferred(),b=deferred(); vi.mocked(api.loadSkillData).mockReturnValueOnce(a.promise).mockReturnValueOnce(b.promise);
 await render(); await select("huajian"); await act(async()=>a.reject(new Error("stale")));
 expect(container.textContent).toContain("加载中..."); expect(toast).not.toHaveBeenCalled();
 await act(async()=>b.resolve(data("huajian"))); expect(save().disabled).toBe(false);
});
it("A 到 B 再到 A 时第一次 A 不能覆盖最后一次 A", async () => {
 const a=deferred(),b=deferred(),latest=deferred();
 vi.mocked(api.loadSkillData).mockReturnValueOnce(a.promise).mockReturnValueOnce(b.promise).mockReturnValueOnce(latest.promise);
 await render(); await select("huajian"); await select("mowen");
 const finalData=data("mowen"); finalData.xinfa.atk_up=99;
 await act(async()=>latest.resolve(finalData)); await act(async()=>b.resolve(data("huajian"))); await act(async()=>a.resolve(data("mowen")));
 await act(async()=>save().click()); expect(api.saveSkillData).toHaveBeenCalledWith("mowen",finalData);
});
it("卸载后旧请求失败不产生提示", async () => {
 const a=deferred(); vi.mocked(api.loadSkillData).mockReturnValueOnce(a.promise);
 await render(); await act(async()=>root.render(null)); await act(async()=>a.reject(new Error("unmounted")));
 expect(toast).not.toHaveBeenCalled();
});
