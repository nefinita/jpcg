import type {
  CalculateRequest,
  SkillResultDTO,
  ForumFileInfo,
  UpdateCheckResult,
  UpdateProgressEvent,
  BuffConfigDTO,
  CoefficientConfigDTO,
  SkillPoolItemDTO,
  ComboStepDTO,
  ComboPresetDTO,
  ComboResultDTO,
  XinfaSummaryDTO,
  SkillEditorDataDTO,
  DerivativesOutputDTO,
  ModuleVersions,
} from "../types";

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
  return invoke("save_config_cmd", {
    player: _sanitizeNumbers(config.player as Record<string, unknown>),
    hostile: _sanitizeNumbers(config.hostile as Record<string, unknown>),
    xinfa: {
      profession: (config.xinfa_config as Record<string, unknown>).profession as string,
      xinfa_name: (config.xinfa_config as Record<string, unknown>).xinfa_name as string,
      xinfa_nom: (config.xinfa_config as Record<string, unknown>).xinfa_nom,
      atk_up: Number((config.xinfa_config as Record<string, unknown>).atk_up) || 0,
      pofang_up: Number((config.xinfa_config as Record<string, unknown>).pofang_up) || 0,
      huixin_up: Number((config.xinfa_config as Record<string, unknown>).huixin_up) || 0,
    },
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

export async function loadSkillPool(profession: string): Promise<SkillPoolItemDTO[]> {
  return invoke("load_skill_pool", { profession });
}

export async function calculateCombo(
  steps: ComboStepDTO[],
  player: Record<string, unknown>,
  hostile: Record<string, unknown>,
  xinfa: Record<string, unknown>,
  buff: BuffConfigDTO,
  coefficient: CoefficientConfigDTO,
): Promise<ComboResultDTO> {
  return invoke("calculate_combo_cmd", {
    steps,
    player,
    hostile,
    xinfa,
    buff,
    coefficient,
  });
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
    case "calculate_damage": {
      const req = args?.req as CalculateRequest | undefined;
      const base = (req?.player?.jichu_gongji ?? 1000) * 2;
      const skills = ["剑气纵横", "流风回雪", "万剑归宗", "听雷诀", "九溪弥烟"];
      return skills.map((name) => ({
        skill_name: name,
        y: 1024 + Math.floor(Math.random() * 200),
        b: base + Math.floor(Math.random() * 500),
        i: Math.floor(base * 1.2),
        n: Math.floor(base * 1.5),
        h: Math.floor(base * 2.8),
        q: Math.floor(base * 2.1),
      }));
    }
    case "load_config_cmd": {
      const saved = typeof localStorage !== "undefined"
        ? localStorage.getItem("jpcg_mock_config") : null;
      return saved ? JSON.parse(saved) : null;
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
        { skill_name: "宫", skill_id: 10447, base_damage1: 160, base_damage2: 200, atk_xishu: 501, watk_xishu: 0, hit_up: 0, huixin_up: 0, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0 },
        { skill_name: "商", skill_id: 10448, base_damage1: 120, base_damage2: 160, atk_xishu: 400, watk_xishu: 0, hit_up: 0, huixin_up: 5, huixiao_up: 0, wushifangyu: 0, wushihuajin: 0, dot_flag: 0 },
        { skill_name: "角", skill_id: 10449, base_damage1: 100, base_damage2: 140, atk_xishu: 350, watk_xishu: 0, hit_up: 10, huixin_up: 0, huixiao_up: 0, wushifangyu: 512, wushihuajin: 0, dot_flag: 0 },
      ];
    case "calculate_combo_cmd": {
      const steps = (args?.steps as ComboStepDTO[]) || [];
      let cum = 0;
      return {
        total_expected_damage_wan: 42.5,
        final_kill_prob: 0.85,
        kill_prob_curve: [[1, 5], [2, 15], [3, 35], [4, 60], [5, 85]],
        steps: steps.map((s, i) => {
          cum += 85000 + Math.random() * 30000;
          return {
            skill_name: s.skill.skill_name,
            g_damage: 50000 + Math.floor(Math.random() * 20000),
            h_damage: 120000 + Math.floor(Math.random() * 50000),
            q_damage: 65000 + Math.floor(Math.random() * 20000),
            crit_rate: 0.35,
            cumulative_mean_wan: cum / 10000,
            kill_prob: Math.min(1, (i + 1) * 0.2),
          };
        }),
      };
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
          wushijianshang: 0, zhenshishanghai: 0, dot_flag: 0, dot_num: 0, dot_up: 0,
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
        current_app_version: "v2.1.0-alpha.1",
        latest_app_version: "v2.1.0-alpha.1",
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
        core: "2.1.0-alpha.1",
        update: "2.1.0-alpha.1",
        const: "130.3.20260602",
        app: "2.1.0-alpha.1",
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
