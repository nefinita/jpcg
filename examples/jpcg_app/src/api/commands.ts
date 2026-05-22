import type {
  CalculateRequest,
  SkillResultDTO,
  ForumFileInfo,
  UpdateCheckResult,
  UpdateProgressEvent,
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
      xinfa_name: (config.xinfa_config as Record<string, unknown>).xinfa_name,
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

export async function forumListFiles(forumUrl = "http://localhost:8080"): Promise<ForumFileInfo[]> {
  return invoke("forum_list_files", { forumUrl });
}

export async function forumDownloadFile(filename: string, forumUrl = "http://localhost:8080"): Promise<string> {
  return invoke("forum_download_file", { forumUrl, filename });
}

export function listenUpdateProgress(callback: (event: UpdateProgressEvent) => void): () => void {
  const listen = getListen();
  if (!listen) return () => {};
  let unlisten: (() => void) | null = null;
  listen("update-progress", (e) => callback(e.payload as UpdateProgressEvent))
    .then((fn) => { unlisten = fn; });
  return () => unlisten?.();
}

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
    default:
      return { success: true };
  }
}
