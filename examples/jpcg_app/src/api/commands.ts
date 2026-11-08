import type {
  CalculateRequest,
  PanelStatsDTO,
  SkillResultDTO,
  ForumFileInfo,
  UpdateCheckResult,
  UpdateProgressEvent,
  BuffConfigDTO,
  CoefficientConfigDTO,
  SkillPoolItemDTO,
  ComboStepDTO,
  ComboPresetDTO,
  XinfaSummaryDTO,
  SkillEditorDataDTO,
  DerivativesOutputDTO,
  ModuleVersions,
  PlayerConfigDTO,
  HostileConfigDTO,
  XinfaConfigDTO,
  ValueSetDTO,
  CombatBundleDTO,
  TimelineRequestDTO,
  TimelineResultDTO,
  TimelineStepSwitchDTO,
} from "../types";
import { sanitizeGearLoadout } from "../state/gear";

let _invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null = null;
let _listen: ((event: string, cb: (e: { payload: unknown }) => void) => Promise<() => void>) | null = null;

function getInvoke() {
  if (_invoke !== null) return _invoke;
  const api = window.__TAURI__?.core;
  if (api?.invoke) {
    _invoke = api.invoke.bind(api);
    return _invoke;
  }
  _invoke = async () => { throw new Error("not in Tauri"); };
  return _invoke;
}

function getListen() {
  if (_listen !== null) return _listen;
  const evt = window.__TAURI__?.event;
  if (evt?.listen) {
    _listen = evt.listen.bind(evt);
    return _listen;
  }
  _listen = null;
  return null;
}

function isTauri() {
  return !!window.__TAURI__?.core?.invoke;
}

async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const fn = getInvoke();
  if (!isTauri()) {
    return mockResponse(command, args) as Promise<T>;
  }
  try {
    return await fn(command, args) as T;
  } catch (err) {
    throw new Error(String(err));
  }
}

export async function calculateDamage(req: CalculateRequest): Promise<SkillResultDTO[]> {
  return invoke("calculate_damage", { req });
}

export async function saveConfig(config: Record<string, unknown>): Promise<void> {
  const xinfa = config.xinfa_config as Record<string, unknown>;
  const buff = (config.buff ?? {}) as Record<string, unknown>;
  return invoke("save_config_cmd", {
    player: _sanitizeNumbers(config.player as Record<string, unknown>),
    hostile: _sanitizeNumbers(config.hostile as Record<string, unknown>),
    xinfa: {
      profession: xinfa.profession as string,
      xinfa_name: xinfa.xinfa_name as string,
      xinfa_nom: xinfa.xinfa_nom,
      atk_up: Number(xinfa.atk_up) || 0,
      pofang_up: Number(xinfa.pofang_up) || 0,
      huixin_up: Number(xinfa.huixin_up) || 0,
    },
    buff: {
      base_atk_pct: Number(buff.base_atk_pct) || 0,
      huixin_pct: Number(buff.huixin_pct) || 0,
      huixiao_pct: Number(buff.huixiao_pct) || 0,
      pofang_pct: Number(buff.pofang_pct) || 0,
      wushi_fangyu_pct: Number(buff.wushi_fangyu_pct) || 0,
      shanghai_pct: Number(buff.shanghai_pct) || 0,
      mode_is_point: !!buff.mode_is_point,
    },
    coefficient: _sanitizeNumbers(config.coefficient as Record<string, unknown>),
    valueSet: (config.value_set as string | null | undefined) ?? null,
  });
}

export async function loadConfig(): Promise<CalculateRequest | null> {
  try {
    return await invoke("load_config_cmd");
  } catch {
    return null;
  }
}

export async function listProfessions(): Promise<XinfaSummaryDTO[]> {
  return invoke("list_professions_cmd");
}

/** 列出可用数值集（正式服/体验服等；来源 data/values/index.toml） */
export async function listValueSets(): Promise<ValueSetDTO[]> {
  return invoke("list_value_sets_cmd");
}

/** 解析指定值集**实际使用**的信息（含不可用回退；返回 coefficient 供 seed 系数） */
export async function resolveValueSet(id?: string | null): Promise<ValueSetDTO> {
  return invoke("resolve_value_set_cmd", { valueSet: id ?? null });
}

export async function loadProfessionConfig(profession: string): Promise<Record<string, unknown> | null> {
  try {
    return await invoke("load_profession_config", { profession });
  } catch {
    return null;
  }
}

export async function checkUpdate(beta = false, force = false): Promise<UpdateCheckResult> {
  return invoke("check_update", { beta, force });
}

export async function getModuleVersions(): Promise<ModuleVersions> {
  return invoke("get_module_versions");
}

export async function performUpdate(
  beta: boolean,
  checkResult: UpdateCheckResult,
): Promise<string> {
  return invoke("perform_update", {
    beta,
    hasDataUpdate: checkResult.has_data_update,
    latestDataVersion: checkResult.latest_data_version,
    dataFilesToUpdate: checkResult.data_files_to_update,
  });
}

export async function performModulesUpdate(
  beta: boolean,
  checkResult: UpdateCheckResult,
): Promise<string> {
  return invoke("perform_modules_update", {
    beta,
    modulesVersion: checkResult.modules_version,
    modulesFilesToUpdate: checkResult.modules_files_to_update,
  });
}

export async function forumListFiles(forumUrl = "http://localhost:8080", category?: string): Promise<ForumFileInfo[]> {
  const args: Record<string, unknown> = { forumUrl };
  if (category !== undefined) args.category = category;
  return invoke("forum_list_files", args);
}

export async function forumDownloadFile(filename: string, forumUrl = "http://localhost:8080", category?: string): Promise<string> {
  const args: Record<string, unknown> = { forumUrl, filename };
  if (category !== undefined) args.category = category;
  return invoke("forum_download_file", args);
}

export async function forumListDownloaded(category?: string): Promise<string[]> {
  const args: Record<string, unknown> = {};
  if (category !== undefined) args.category = category;
  return invoke("forum_list_downloaded", args);
}

export async function forumDeleteDownloaded(filename: string, category?: string): Promise<string> {
  const args: Record<string, unknown> = { filename };
  if (category !== undefined) args.category = category;
  return invoke("forum_delete_downloaded", args);
}

export async function forumListCategories(forumUrl = "http://localhost:8080"): Promise<string[]> {
  return invoke("forum_list_categories", { forumUrl });
}

export function listenUpdateProgress(callback: (event: UpdateProgressEvent) => void): () => void {
  const listen = getListen();
  if (!listen) return () => {};
  let unlisten: (() => void) | null = null;
  listen("update-progress", (e) => callback(e.payload as UpdateProgressEvent))
    .then((fn) => { unlisten = fn; });
  return () => unlisten?.();
}

/** 配装自动调优进度（payload 为 0..100 的整数百分比）。 */
export function listenGearOptimizeProgress(callback: (pct: number) => void): () => void {
  const listen = getListen();
  if (!listen) return () => {};
  let unlisten: (() => void) | null = null;
  let cancelled = false;
  listen("gear-optimize-progress", (e) => { if (!cancelled) callback(Number(e.payload) || 0); })
    .then((fn) => { if (cancelled) fn(); else unlisten = fn; });
  return () => { cancelled = true; unlisten?.(); };
}

export async function loadSkillPool(profession: string): Promise<SkillPoolItemDTO[]> {
  return invoke("load_skill_pool", { profession });
}

/** 读取某门派的战斗时序 overlay + 技能池（#92 时间轴预览） */
export async function loadCombatBundle(profession: string): Promise<CombatBundleDTO> {
  return invoke("load_combat_bundle", { profession });
}

/** 时间轴模拟（窗口伤害 / DPS / 事件序列） */
export async function calculateTimeline(req: TimelineRequestDTO): Promise<TimelineResultDTO> {
  return invoke("calculate_timeline_cmd", { req });
}

export async function saveComboPreset(name: string, steps: ComboStepDTO[]): Promise<void> {
  return invoke("save_combo_preset", { name, steps });
}

export async function listComboPresets(): Promise<string[]> {
  return invoke("list_combo_presets");
}

export async function loadComboPreset(name: string): Promise<ComboPresetDTO> {
  return invoke("load_combo_preset", { name });
}

export async function deleteComboPreset(name: string): Promise<void> {
  return invoke("delete_combo_preset", { name });
}

export async function loadSkillData(profession: string): Promise<SkillEditorDataDTO> {
  return invoke("load_skill_data", { profession });
}

export async function saveSkillData(profession: string, data: SkillEditorDataDTO): Promise<void> {
  return invoke("save_skill_data", { profession, data });
}

export async function performAppUpdate(beta: boolean): Promise<string> {
  return invoke("perform_app_update", { beta });
}

export async function computeDerivatives(req: CalculateRequest): Promise<DerivativesOutputDTO> {
  return invoke("compute_derivatives", { req });
}

export async function exportConfig(): Promise<string> {
  return invoke("export_config_cmd");
}

export async function importConfig(tomlStr: string): Promise<void> {
  return invoke("import_config_cmd", { tomlStr });
}

const XINFA_LIST_MOCK: XinfaSummaryDTO[] = [
  { value: "mowen", label: "莫问", nom: "根骨", version_label: "130级第3赛季" },
  { value: "badao", label: "霸刀山庄", nom: "力道", version_label: "130级第3赛季" },
  { value: "wufang", label: "无方", nom: "元气", version_label: "130级第3赛季" },
  { value: "cangjian", label: "藏剑山庄", nom: "身法", version_label: "130级第3赛季" },
  { value: "qixiu", label: "七秀坊", nom: "根骨", version_label: "130级第3赛季" },
  { value: "wanhua", label: "万花谷", nom: "根骨", version_label: "130级第3赛季" },
  { value: "tiance", label: "天策府", nom: "力道", version_label: "130级第3赛季" },
  { value: "shaolin", label: "少林寺", nom: "根骨", version_label: "130级第3赛季" },
  { value: "chunyang", label: "纯阳观", nom: "根骨", version_label: "130级第3赛季" },
  { value: "wudu", label: "五毒教", nom: "根骨", version_label: "130级第3赛季" },
  { value: "tangmen", label: "唐门", nom: "力道", version_label: "130级第3赛季" },
  { value: "mingjiao", label: "明教", nom: "身法", version_label: "130级第3赛季" },
  { value: "gaibang", label: "丐帮", nom: "力道", version_label: "130级第3赛季" },
  { value: "cangyun", label: "苍云", nom: "身法", version_label: "130级第3赛季" },
  { value: "penglai", label: "蓬莱", nom: "根骨", version_label: "130级第3赛季" },
  { value: "yantian", label: "衍天宗", nom: "根骨", version_label: "130级第3赛季" },
  { value: "yaozong", label: "药宗", nom: "根骨", version_label: "130级第3赛季" },
  { value: "zhoutian", label: "周天功", nom: "元气", version_label: "130级第3赛季" },
];

function _sanitizeNumbers(obj: Record<string, unknown>): Record<string, unknown> {
  const result: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(obj)) {
    if (typeof value === "string" && value.trim() === "") {
      result[key] = 0;
    } else if (typeof value === "string" && isNaN(Number(value))) {
      result[key] = value;
    } else {
      const num = Number(value);
      result[key] = isNaN(num) ? 0 : num;
    }
  }
  return result;
}

async function mockResponse(command: string, args?: Record<string, unknown>): Promise<unknown> {
  await new Promise((r) => setTimeout(r, 200));
  switch (command) {
    case "panel_stats":
      // 浏览器仅演示固定结果，不能以 JS 实现另一套属性公式。
      return { huixin_pct: 30, pofang_pct: 45, jianshang_pct: 0 };
    case "calculate_damage": {
      const req = args?.req as CalculateRequest | undefined;
      const base = (req?.player?.jichu_gongji ?? 1000) * 2;
      // 模拟技能（dev 模式兜底数据）：dot=跳数、wuzhi=无质标记，由标记生成，不依赖技能名判定
      const skills = [
        { name: "剑气纵横" },
        { name: "流风回雪" },
        { name: "万剑归宗" },
        { name: "听雷诀" },
        { name: "九溪弥烟" },
        { name: "商（dot）", dot: 6 },
        { name: "相依", wuzhi: true },
      ];
      return skills.map(({ name, dot, wuzhi }) => {
        const jumps = dot ? Array.from({ length: dot }, () => Math.floor(base * 1.2)) : [];
        return {
          skill_name: name,
          y: 1024 + Math.floor(Math.random() * 200),
          b: base + Math.floor(Math.random() * 500),
          i: Math.floor(base * 1.2),
          n: Math.floor(base * 1.5),
          h: Math.floor(base * 2.8),
          q: jumps.length > 0 ? jumps.reduce((s, j) => s + j, 0) : Math.floor(base * 2.1),
          dot_jumps: jumps,
          has_critical_strike: !!wuzhi,
          zhenshishanghai: 0,
        };
      });
    }
    case "load_config_cmd": {
      const saved = typeof localStorage !== "undefined"
        ? localStorage.getItem("jpcg_mock_config") : null;
      return saved ? JSON.parse(saved) : null;
    }
    case "list_value_sets_cmd":
      return [
        { id: "cszj-exp-260908", name: "体验服·苍生铸世一测", level: 50, is_default: true, source: "builtin", available: true,
          coefficient: { pofang_xishu: 10378.17, huixin_xishu: 9512.91, huixiao_xishu: 3504.60, yujin_xishu: 19025.82, yuhui_xishu: 9422.82, huajin_xishu: 5148.00, fangyu_xishu: 10802.88, pvp_global_jianshang: 0.9 } },
        { id: "live-130", name: "正式服·130级", level: 130, is_default: false, source: "builtin", available: true,
          coefficient: { pofang_xishu: 225957.6, huixin_xishu: 197703, huixiao_xishu: 72844.2, yujin_xishu: 197703, yuhui_xishu: 55123.2, huajin_xishu: 30115.8, fangyu_xishu: 126007.2, pvp_global_jianshang: 0.9 } },
      ];
    case "resolve_value_set_cmd": {
      const id = (args?.valueSet ?? args?.value_set) as string | null | undefined;
      const sets = (await mockResponse("list_value_sets_cmd")) as ValueSetDTO[];
      return sets.find((s) => s.id === id) ?? sets.find((s) => s.is_default) ?? sets[0];
    }
    case "forum_list_files":
      return [
        { name: "mowen.toml", size: 2048, modified: "2026-05-14 10:00:00" },
        { name: "zhoutian.toml", size: 1536, modified: "2026-05-14 09:30:00" },
        { name: "template.toml", size: 1024, modified: "2026-05-14 08:00:00" },
      ];
    case "forum_download_file":
      return "下载成功（模拟）";
    case "forum_list_downloaded":
      return ["mowen.toml", "template.toml"];
    case "forum_delete_downloaded":
      return "删除成功（模拟）";
    case "forum_list_categories":
      return ["shuxing", "combo"];
    case "load_skill_pool":
      return [
        { skill_name: "宫", skill_id: 10447, sub_id: 14474, base_damage1: 160, base_damage2: 200, atk_xishu: 501, watk_xishu: 0, hit_up: 0, huixin_up: 0, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0, has_critical_strike: false },
        { skill_name: "商", skill_id: 10448, sub_id: 14475, base_damage1: 120, base_damage2: 160, atk_xishu: 400, watk_xishu: 0, hit_up: 0, huixin_up: 5, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0, has_critical_strike: false },
        { skill_name: "角", skill_id: 10449, sub_id: 14476, base_damage1: 100, base_damage2: 140, atk_xishu: 350, watk_xishu: 0, hit_up: 10, huixin_up: 0, huixiao_up: 0, wushifangyu: 512, wushihuajin: 0, dot_flag: 0, has_critical_strike: false },
        { skill_name: "徵", skill_id: 14067, sub_id: 14227, base_damage1: 84, base_damage2: 92, atk_xishu: 0.219602, watk_xishu: 0, hit_up: 0, huixin_up: 0, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0, has_critical_strike: false, cast_time: 2.8125, channel: true, channel_ticks: 3 },
        { skill_name: "停手", skill_id: 999999, sub_id: 0, base_damage1: 0, base_damage2: 0, atk_xishu: 0, watk_xishu: 0, hit_up: 0, huixin_up: 0, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0, has_critical_strike: false },
        { skill_name: "引窍·0点任脉", skill_id: 38009, sub_id: 38438, base_damage1: 20, base_damage2: 20, atk_xishu: 3.51, watk_xishu: 0, hit_up: 0, huixin_up: 0, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0, has_critical_strike: false },
        { skill_name: "引窍·50点任脉", skill_id: 38009, sub_id: 38438, base_damage1: 20, base_damage2: 20, atk_xishu: 10.53, watk_xishu: 0, hit_up: 0, huixin_up: 0, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0, has_critical_strike: false },
        { skill_name: "引窍·100点任脉", skill_id: 38009, sub_id: 38438, base_damage1: 20, base_damage2: 20, atk_xishu: 17.55, watk_xishu: 0, hit_up: 0, huixin_up: 0, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0, has_critical_strike: false },
        { skill_name: "破·0点能量", skill_id: 38014, sub_id: 38093, base_damage1: 20, base_damage2: 20, atk_xishu: 8.53, watk_xishu: 0, hit_up: 0, huixin_up: 0, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0, has_critical_strike: false },
        { skill_name: "破·50点能量", skill_id: 38014, sub_id: 38093, base_damage1: 20, base_damage2: 20, atk_xishu: 8.53, watk_xishu: 0, hit_up: 0, huixin_up: 0, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0, has_critical_strike: false },
      ];
    case "load_combat_bundle": {
      const profession = String(args?.profession ?? "");
      const forms = [
        { id: "qin", name: "琴", gcd_base: 1.5, initial: true, skills: [] },
        { id: "jian", name: "剑", gcd_base: 1.5, initial: false, skills: [] },
      ];
      const sc = (skill_id: number, sub_id: number, extra: Record<string, unknown>) => ({
        skill_id, sub_id, requires_instrument: "qin", requires_qifeng: null,
        gcd_lane: "", on_cast: { switch_instrument: null, switch_qifeng: null, gcd_lanes: [], lock_skills: [] },
        cast_time: null, gcd_base: null, gcd_kind: null, cooldown: null,
        charges: null, recharge: null, cd_haste: null, channel: null, channel_ticks: null, buff_id: null,
        jihuoqixue: null, qifeng_gain: null, spend_all_qifeng: null,
        ...extra,
      });
      return {
        skills: await mockResponse("load_skill_pool", args),
        combat: {
          meta: { profession, version: "mock", note: "浏览器预览数据", miji_per_skill_cap: 4, mixed_qixue_cap: 3, mixed_slot_from: 7 },
          form: forms,
          qifeng: ["yangchun", "gaoshan", "pingsha"],
          skill_combat: [
            sc(10447, 14474, { cast_time: 1.75, miji_ids: ["mj_gong_1", "mj_gong_2", "mj_gong_3"] }),
            sc(10448, 14475, { qifeng_gain: 1, miji_ids: ["mj_shang_1"] }),
            sc(10449, 14476, { qifeng_gain: 1, miji_ids: ["mj_jiao_1"] }),
            sc(14067, 14227, { cast_time: 2.8125, channel: true, channel_ticks: 3 }),
            sc(38009, 38438, { requires_instrument: "jian" }),
            sc(38014, 38093, { requires_instrument: "jian", jihuoqixue: "qx_14" }),
            sc(14069, 0, { gcd_kind: 1, cooldown: 3.0, on_cast: { switch_instrument: null, switch_qifeng: "gaoshan", gcd_lanes: [], lock_skills: [] } }),
          ],
          buff: [],
          miji: [
            { id: "mj_gong_1", name: "经脉图·残页", skill_id: 10447, sub_id: 14474, modifiers: [{ cast_time_add: -0.125 }], effects: ["运功-0.125秒"] },
            { id: "mj_gong_2", name: "参悟·断篇", skill_id: 10447, sub_id: 14474, modifiers: [{ hit_up_add: 4 }], effects: ["伤害+4%"] },
            { id: "mj_gong_3", name: "秘法·绝章", skill_id: 10447, sub_id: 14474, modifiers: [{ huixin_up_add: 4 }], effects: ["会心+4%"] },
            { id: "mj_shang_1", name: "参悟·残页", skill_id: 10448, sub_id: 14475 },
            { id: "mj_jiao_1", name: "秘法·绝章", skill_id: 10449, sub_id: 14476, modifiers: [{ huixin_up_add: 4 }], effects: ["会心+4%"] },
          ],
          qixue: [
            { id: "dongtian", name: "洞天", slot: 1 },
            { id: "juexian", shelved: true, name: "绝弦", slot: 1 },
            { id: "ruyue", shelved: true, name: "入月", slot: 1 },
            { id: "luming", shelved: true, name: "鹿鸣", slot: 1 },
            { id: "shuqu", name: "殊曲", slot: 2 },
            { id: "xiaozhu", name: "削竹", slot: 2 },
            { id: "zuijian", shelved: true, name: "醉剑", slot: 2 },
            { id: "ningmei", shelved: true, name: "凝眉", slot: 3 },
            { id: "luanyu", name: "鸾羽", slot: 3 },
            { id: "daoyi", shelved: true, name: "捣衣", slot: 3 },
            { id: "canlian", name: "参连", slot: 3 },
            { id: "dongquan", name: "冻泉", slot: 4 },
            { id: "dongquan-gong", name: "冻泉·宫", slot: 4 },
            { id: "tingpai", shelved: true, name: "停拍", slot: 4 },
            { id: "guanzhu", shelved: true, name: "贯珠", slot: 4 },
            { id: "lingdong", name: "凌冬", slot: 4 },
            { id: "xiangyi", shelved: true, name: "相依", slot: 5 },
            { id: "yince", name: "音彻", slot: 5 },
            { id: "jingzhi", name: "净植", slot: 5 },
            { id: "chaoran", name: "超然", slot: 5 },
            { id: "taichanglv", name: "太常律", slot: 6 },
            { id: "bukuijun", name: "不愧君", slot: 6 },
            { id: "duanshuiliu", name: "断水流", slot: 6 },
            { id: "zhanchangjing", name: "斩长鲸", slot: 6 },
            { id: "qiushuang", shelved: true, name: "秋霜", slot: 7 },
            { id: "haoqing", shelved: true, name: "豪情", slot: 7 },
            { id: "shixiang", name: "师襄", slot: 7 },
            { id: "jingyi", shelved: true, name: "井仪", slot: 7 },
            { id: "biehe", shelved: true, name: "别鹤", slot: 7 },
            { id: "jianshan", name: "兼山", slot: 7 },
            { id: "youyu", shelved: true, name: "游羽", slot: 7 },
            { id: "wangxian", shelved: true, name: "忘弦", slot: 7 },
            { id: "lianpo", shelved: true, name: "莲魄", slot: 7 },
            { id: "wujinzang", shelved: true, name: "无尽藏", slot: 7 },
            { id: "liuyun", shelved: true, name: "流韵", slot: 7 },
            { id: "chongqiuming", name: "冲秋冥", slot: 7 },
          ],
        },
      } as CombatBundleDTO;
    }
    case "calculate_timeline_cmd": {
      const req = args?.req as TimelineRequestDTO | undefined;
      const actions = req?.actions ?? [];
      const events: TimelineResultDTO["events"] = [];
      const castTimes: number[] = [];
      const initialInstrument = req?.initial_instrument ?? "qin";
      const initialQifeng = req?.initial_qifeng ?? "yangchun";
      let t = 0;
      let total = 0;
      for (const a of actions) {
        if (a.kind !== "cast") continue;
        const skill = req?.skills?.find((s) => s.skill_id === a.skill_id && (s.sub_id ?? 0) === a.sub_id);
        const damage = Math.floor(80000 + Math.random() * 20000);
        total += damage;
        const name = skill?.skill_name ?? "技能";
        castTimes.push(t);
        events.push({
          time: t,
          kind: { kind: "cast", skill_name: name, damage, hit_time: t + (skill?.cast_time ?? 1.5), gcd_secs: 1.5, gcd_ready_at: t + 1.5 },
        });
        // 浏览器预览：徵 产生 3 段引导结算
        if (name.includes("徵")) {
          for (let i = 1; i <= 3; i += 1) {
            events.push({
              time: t + (2.8125 * i) / 3,
              kind: { kind: "channel_tick", skill_name: name, index: i, damage: 28_000 },
            });
          }
          t += 2.8125;
        }
        // 浏览器预览：商/角 产生 6 跳 DOT，便于检查右侧持续伤害记录
        if (name.includes("商") || name.includes("角")) {
          for (let i = 0; i < 6; i += 1) {
            events.push({
              time: t + 3 * (i + 1),
              kind: { kind: "dot_tick", skill_name: `${name}·持续`, index: i + 1, damage: name.includes("商") ? 750 : 731 },
            });
          }
        }
        t += skill?.cast_time ?? 1.5;
      }
      // 浏览器预览：补一条器形切换与一条曲风切换事件，便于检查视觉
      if (events.length >= 2) {
        events.splice(1, 0, {
          time: events[0].time,
          kind: { kind: "qifeng_switch", from: "yangchun", to: "gaoshan" },
        });
        events.push({
          time: t,
          kind: { kind: "form_switch", from: "qin", to: "jian" },
        });
      }
      events.sort((a, b) => a.time - b.time);
      let cumulative = 0;
      for (const e of events) {
        if (e.kind.kind === "cast" || e.kind.kind === "dot_tick" || e.kind.kind === "channel_tick") {
          cumulative += e.kind.damage;
        }
        e.cumulative = cumulative;
      }
      const castActions = actions.filter((a) => a.kind === "cast");
      const steps = castActions.map((a, i) => {
        const skill = req?.skills?.find((sk) => sk.skill_id === a.skill_id && (sk.sub_id ?? 0) === a.sub_id);
        const def = (req?.combat?.skill_combat ?? []).find((sc) => sc.skill_id === a.skill_id && (sc.sub_id ?? 0) === (a.sub_id ?? 0));
        const time = castTimes[i] ?? null;
        return {
          time,
          gcd_secs: time == null ? 0 : 1.5,
          gcd_ready_at: time == null ? 0 : time + 1.5,
          legal: time != null,
          instrument: initialInstrument,
          qifeng: initialQifeng,
          skipped_reason: time == null ? "当前器形/曲风不满足技能要求" : "",
          required_instrument: def?.requires_instrument ?? null,
          required_qifeng: def?.requires_qifeng ?? null,
          switches: [] as TimelineStepSwitchDTO[],
        };
      });
      const hp = req?.hostile?.target_hp ?? 0;
      const final_kill_prob = hp > 0 ? Math.min(1, total / hp) : 0;
      const kill_prob_curve = castActions.map((_, i) => [i + 1, final_kill_prob] as [number, number]);
      return {
        events,
        steps,
        total_damage: total,
        window_dps: t > 0 ? total / t : 0,
        skipped_actions: 0,
        final_instrument: initialInstrument,
        final_qifeng: initialQifeng,
        final_kill_prob,
        kill_prob_curve,
      } as TimelineResultDTO;
    }
    case "list_combo_presets":
      return ["莫问PVP连招", "霸刀爆发", "测试连招"];
    case "load_combo_preset":
      return { name: args?.name || "预设", steps: [] };
    case "list_professions_cmd":
      return XINFA_LIST_MOCK;
    case "export_config_cmd":
      return "# 模拟导出配置\n[xinfa]\nxinfa_name = \"莫问\"\n";
    case "load_skill_data": {
      const prof = (args?.profession as string) || "mowen";
      return {
        xinfa: { profession: prof, xinfa_name: "莫问", xinfa_nom: "根骨", atk_up: 1.96, pofang_up: 2.0, huixin_up: 0 },
        version: { level: 130, season: 3, modified: 20260602 },
        skills: [
          { skill_name: "宫", skill_id: 10447, base_damage1: 160, base_damage2: 200, atk_xishu: 2.609375 },
          { skill_name: "商", skill_id: 10448, base_damage1: 14, base_damage2: 19, atk_xishu: 2.447916 },
          { skill_name: "角", skill_id: 10449, base_damage1: 14, base_damage2: 19, atk_xishu: 2.447916 },
        ].map((s) => ({
          ...s,
          sub_id: 0, group: 0, weapon_request: 0, design_effect: 0,
          kind_type: 0, cast_mode: 0, guaranteed_hit: false, has_critical_strike: false,
          effect_type: 0, jihuoqixue: "", watk_xishu: 0, hit_up: 0,
          huixin_up: 0, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0,
          wushijianshang: 0, zhenshishanghai: 0, dot_flag: 0, dot_interval: 0, dot_duration: 0, dot_up: 0,
        })),
      };
    }
    case "compute_derivatives": {
      const req = args?.req as CalculateRequest | undefined;
      const skills = ["宫", "商", "角", "徵", "羽"];
      const baseVal = (idx: number) => {
        const vals = [15000, 55000, 15000, 1200, 11000, 0];
        return vals[idx] || 0;
      };
      const baseDer = (idx: number) => {
        const ders = [87.17, 44.48, 11.97, 11.14, 16.13, 44.49];
        return ders[idx] || 0;
      };
      const attrs = [
        { attr_name: "基础属性", attr_id: "jichu_shuxing" },
        { attr_name: "基础攻击", attr_id: "jichu_gongji" },
        { attr_name: "会心等级", attr_id: "huixin_dengji" },
        { attr_name: "会心效果", attr_id: "huixin_xiaoguo" },
        { attr_name: "破防等级", attr_id: "pofang_dengji" },
        { attr_name: "武器伤害", attr_id: "wuqi_shanghai" },
      ];
      const ders = attrs.map((a, ai) => ({
        attr_name: a.attr_name,
        attr_id: a.attr_id,
        current_value: baseVal(ai),
        total_derivative: baseDer(ai) * skills.length + (Math.random() - 0.5) * 10,
        per_skill: skills.map((sn) => ({
          skill_name: sn,
          derivative: baseDer(ai) * (0.8 + Math.random() * 0.4),
        })),
      }));
      const sorted = [...ders].sort((a, b) => b.total_derivative - a.total_derivative);
      return {
        derivatives: sorted,
        recommendation: {
          crit_vs_pofang: {
            better: ders.find((d) => d.attr_id === "pofang_dengji")!.total_derivative >
              ders.find((d) => d.attr_id === "huixin_dengji")!.total_derivative
              ? "破防等级" : "会心等级",
            huixin_total: ders.find((d) => d.attr_id === "huixin_dengji")!.total_derivative,
            pofang_total: ders.find((d) => d.attr_id === "pofang_dengji")!.total_derivative,
          },
          top3: sorted.slice(0, 3).map((d) => ({
            attr_name: d.attr_name,
            attr_id: d.attr_id,
            total_derivative: d.total_derivative,
          })),
        },
      };
    }
    case "check_update":
      return {
        current_app_version: "v2.2.0-alpha.1",
        latest_app_version: "v2.2.0-alpha.1",
        has_app_update: false,
        current_data_version: null,
        latest_data_version: null,
        has_data_update: false,
        data_files_to_update: [],
        has_modules_update: false,
        modules_version: null,
        modules_files_to_update: [],
      };
    case "get_module_versions":
      return {
        core: "2.2.0-alpha.1",
        update: "2.2.0-alpha.1",
        const: "130.3.20260602",
        combo: "2.2.0-alpha.1",
        app: "2.2.0-alpha.1",
      };
    case "perform_app_update":
      return "重启中...（模拟）";
    case "perform_modules_update":
      return "重启中...（模拟）";
    case "save_skill_data":
      return null;
    default:
      return { success: true };
  }
}

// 桌面配装入口：失败必须显示，浏览器不伪造 Rust 合成或 TOML 解析。
// 旧版本存档可能带已删除字段（如 sockets）；统一在这里剔除，避免后端 deny_unknown_fields 拒绝。
export async function gearCatalog(): Promise<import("../types/gear").GearCatalog> {
  if (!isTauri()) throw new Error("配装模板需要桌面应用；可切换手动属性模式");
  return invoke("gear_catalog");
}
export async function composeGear(loadout: import("../types/gear").GearLoadout): Promise<import("../types/gear").GearComposition> {
  if (!isTauri()) throw new Error("配装合成需要桌面应用");
  return invoke("compose_gear", { loadout: sanitizeGearLoadout(loadout) });
}
export async function exportGear(loadout: import("../types/gear").GearLoadout): Promise<string> {
  if (!isTauri()) throw new Error("配装导出需要桌面应用");
  return invoke("export_gear", { loadout: sanitizeGearLoadout(loadout) });
}
export async function importGear(toml: string): Promise<import("../types/gear").GearComposition> {
  if (!isTauri()) throw new Error("配装导入需要桌面应用");
  return invoke("import_gear", { toml });
}

export async function optimizeGear(request: import("../types/gear").GearOptimizeRequest): Promise<import("../types/gear").GearOptimizeResult> {
  if (!isTauri()) throw new Error("配装调优需要桌面应用");
  return invoke("optimize_gear", { request });
}

export async function panelStats(req: CalculateRequest): Promise<PanelStatsDTO> {
  return invoke("panel_stats", { req });
}
