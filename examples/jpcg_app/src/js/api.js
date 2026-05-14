/**
 * ============================================================================
 * api.js — Tauri v2 API 通信封装
 * 采用纯全局对象方案（无需 npm 包），直接使用 window.__TAURI__.core.invoke。
 * 全局对象方式在 Tauri v2 的 beforeDevServer / beforeBuildCommand 中注入。
 * ============================================================================
 */

import { TAURI_COMMANDS, FORUM_URL } from './config.js';

/**
 * 获取 Tauri invoke 函数（兼容 Tauri v1/v2）
 * @returns {Function|null} invoke 函数，非 Tauri 环境返回 null
 */
function getInvoke() {
  // Tauri v2 标准全局 API 路径
  if (window.__TAURI__?.core?.invoke) {
    return window.__TAURI__.core.invoke;
  }
  // Tauri v1 向后兼容（可选降级）
  if (window.__TAURI__?.tauri?.invoke) {
    return window.__TAURI__.tauri.invoke;
  }
  return null;
}

/**
 * 检测当前是否运行在 Tauri 桌面环境中
 * @returns {boolean}
 */
export const isTauri = () => getInvoke() !== null;

/**
 * 调用 Tauri 命令的通用封装
 * - 非 Tauri 环境自动降级为 Mock 响应
 * - 错误统一包装为 Error 对象抛出
 *
 * @param {string} command - Tauri 命令名称
 * @param {Object} args - 命令参数
 * @returns {Promise<any>} 命令执行结果
 */
export async function invokeCommand(command, args = {}) {
  const invoke = getInvoke();

  // 非 Tauri 环境：使用 Mock 数据（开发/调试用）
  if (!invoke) {
    console.warn(`[Mock] invokeCommand: ${command}`, args);
    return _mockResponse(command, args);
  }

  try {
    // Tauri v2 标准调用方式
    return await invoke(command, args);
  } catch (error) {
    console.error(`[Tauri] ${command} failed:`, error);
    throw new Error(error.message || `调用 ${command} 失败`);
  }
}

/**
 * ============================================================================
 * 业务接口
 * ============================================================================
 */

/** 🧮 伤害计算 */
export async function calculateDamage(config) {
  console.log('[calculateDamage] config.jcsx =', config.jcsx, 'type:', typeof config.jcsx);
  const payload = {
    req: {
      player: _sanitizeNumbers(config.player),
      hostile: _sanitizeNumbers(config.hostile),
      xinfa_config: {
        xinfa_name: config.xinfa || config.xinfa_config?.xinfa_name || 'mowen',
        xinfa_nom: config.xinfa_config?.xinfa_nom || 'gengu',
        atk_up: Number(config.xinfa_config?.atk_up) || 0,
        pofang_up: Number(config.xinfa_config?.pofang_up) || 0,
        huixin_up: Number(config.xinfa_config?.huixin_up) || 0,
      },
    },
  };
  return await invokeCommand(TAURI_COMMANDS.calculate, payload);
}

/** 💾 保存玩家配置到本地文件 */
export async function saveConfig(config) {
  const payload = {
    player: _sanitizeNumbers(config.player),
    hostile: _sanitizeNumbers(config.hostile),
    xinfa: {
      xinfa_name: config.xinfa || config.xinfa_config?.xinfa_name || 'mowen',
      xinfa_nom: config.xinfa_config?.xinfa_nom || 'gengu',
      atk_up: Number(config.xinfa_config?.atk_up) || 0,
      pofang_up: Number(config.xinfa_config?.pofang_up) || 0,
      huixin_up: Number(config.xinfa_config?.huixin_up) || 0,
    },
  };
  return await invokeCommand(TAURI_COMMANDS.save, payload);
}

/** 📥 从本地文件加载配置 */
export async function loadConfig() {
  try {
    return await invokeCommand(TAURI_COMMANDS.load);
  } catch (error) {
    console.warn('[API] loadConfig failed:', error);
    return null;
  }
}

/** 🎯 按心法名称加载职业配置 */
export async function loadProfessionConfig(profession) {
  return await invokeCommand(TAURI_COMMANDS.loadProfession, { profession });
}

/** 🔍 检查更新（仅检查，不下载） */
export async function checkUpdate(beta = false, force = false) {
  return await invokeCommand(TAURI_COMMANDS.checkUpdate, { beta, force });
}

/** ⬇️ 执行更新下载 */
export async function performUpdate(beta = false, result) {
  return await invokeCommand(TAURI_COMMANDS.performUpdate, {
    beta,
    hasDataUpdate: result.has_data_update,
    latestDataVersion: result.latest_data_version,
    dataFilesToUpdate: result.data_files_to_update,
  });
}

/** 📋 获取论坛上的 .toml 文件列表 */
export async function forumListFiles(forumUrl = FORUM_URL) {
  return await invokeCommand(TAURI_COMMANDS.forumListFiles, { forumUrl });
}

/** ⬇️ 从论坛下载 .toml 文件到 data/pvp36500/ 目录 */
export async function forumDownloadFile(filename, forumUrl = FORUM_URL) {
  return await invokeCommand(TAURI_COMMANDS.forumDownload, { forumUrl, filename });
}

/**
 * 监听 Tauri 更新进度事件
 * 通过 window.__TAURI__.event.listen 订阅 "update-progress" 事件。
 *
 * @param {Function} callback - 回调函数，接收 { stage, message, progress, file }
 * @returns {Function} 取消监听的函数（调用后停止接收事件）
 */
export function listenUpdateProgress(callback) {
  if (window.__TAURI__?.event?.listen) {
    let unlisten = null;
    window.__TAURI__.event
      .listen('update-progress', (event) => {
        const { stage, message, progress, file } = event.payload;
        callback({ stage, message, progress, file });
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => {
      if (unlisten) unlisten();
    };
  }
  // 非 Tauri 环境返回空操作
  return () => {};
}

/**
 * ============================================================================
 * 工具函数
 * ============================================================================
 */

/**
 * 数值清洗工具
 * 将表单输入（字符串）转为数字或保留字符串：
 * - 空字符串 → 0
 * - 非数字字符串（如 jcsx 的 "gengu"）→ 保持原字符串
 * - 数字字符串 → Number
 *
 * @param {Object} obj - 待清洗的对象
 * @returns {Object} 清洗后的对象
 */
function _sanitizeNumbers(obj) {
  if (!obj) return {};
  const result = {};
  for (const [key, value] of Object.entries(obj)) {
    if (typeof value === 'string' && value.trim() === '') {
      // 空字符串转为 0（未填写的数字字段）
      result[key] = 0;
    } else if (typeof value === 'string' && isNaN(Number(value))) {
      // 非数字字符串保留原值（如 jcsx 的 "gengu"）
      result[key] = value;
    } else {
      // 数字字符串转为 Number
      const num = Number(value);
      result[key] = isNaN(num) ? 0 : num;
    }
  }
  return result;
}

/**
 * ============================================================================
 * Mock 降级方案
 * 在非 Tauri 环境（浏览器直接打开）下提供示例数据
 * ============================================================================
 */

async function _mockResponse(command, args) {
  await new Promise((resolve) => setTimeout(resolve, 200));
  switch (command) {
    case TAURI_COMMANDS.calculate:
      return _mockCalculate(args.req);
    case TAURI_COMMANDS.load:
      const saved = localStorage.getItem('jpcg_mock_config');
      return saved ? JSON.parse(saved) : null;
    case TAURI_COMMANDS.forumListFiles:
      return [
        { name: 'mowen.toml', size: 2048, modified: '2026-05-14 10:00:00' },
        { name: 'zhoutian.toml', size: 1536, modified: '2026-05-14 09:30:00' },
        { name: 'template.toml', size: 1024, modified: '2026-05-14 08:00:00' },
      ];
    case TAURI_COMMANDS.forumDownload:
      return '下载成功（模拟）';
    default:
      return { success: true };
  }
}

function _mockCalculate(req) {
  const skills = ['剑气纵横', '流风回雪', '万剑归宗', '听雷诀', '九溪弥烟'];
  const base = (req?.player?.jichu_gongji || 1000) * 2;
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

/** 📝 简易日志工具 */
export const logger = {
  info:  (m, d) => console.log(`[ℹ️] ${m}`, d || ''),
  warn:  (m, d) => console.warn(`[⚠️] ${m}`, d || ''),
  error: (m, e) => console.error(`[❌] ${m}`, e),
};
