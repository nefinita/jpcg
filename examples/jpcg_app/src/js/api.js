/**
 * Tauri v2 API 通信封装 - 纯全局对象方案（无需 npm 包）
 * 直接使用 window.__TAURI__.core.invoke
 */

import { TAURI_COMMANDS } from './config.js';

/**
 * 获取 Tauri invoke 函数（兼容检测）
 */
function getInvoke() {
  // Tauri v2: 全局对象结构
  if (window.__TAURI__?.core?.invoke) {
    return window.__TAURI__.core.invoke;
  }
  // Tauri v1 降级（可选）
  if (window.__TAURI__?.tauri?.invoke) {
    return window.__TAURI__.tauri.invoke;
  }
  return null;
}

/**
 * 检测是否运行在 Tauri 环境
 */
export const isTauri = () => getInvoke() !== null;

/**
 * 调用 Tauri 命令
 */
export async function invokeCommand(command, args = {}) {
  const invoke = getInvoke();
  
  if (!invoke) {
    console.warn(`[Mock] invokeCommand: ${command}`, args);
    return _mockResponse(command, args);
  }

  try {
    // ✅ Tauri v2 全局调用方式
    return await invoke(command, args);
  } catch (error) {
    console.error(`[Tauri] ${command} failed:`, error);
    throw new Error(error.message || `调用 ${command} 失败`);
  }
}

/**
 * 伤害计算接口
 */
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

/**
 * 保存配置接口
 */
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

/**
 * 加载配置接口
 */
export async function loadConfig() {
  try {
    return await invokeCommand(TAURI_COMMANDS.load);
  } catch (error) {
    console.warn('[API] loadConfig failed:', error);
    return null;
  }
}

/**
 * 按职业加载配置
 */
export async function loadProfessionConfig(profession) {
  return await invokeCommand(TAURI_COMMANDS.loadProfession, { profession });
}

/**
 * 数值清洗工具
 */
function _sanitizeNumbers(obj) {
  if (!obj) return {};
  const result = {};
  for (const [key, value] of Object.entries(obj)) {
    if (typeof value === 'string' && value.trim() === '') {
      result[key] = 0;
    } else {
      const num = Number(value);
      result[key] = isNaN(num) ? 0 : num;
    }
  }
  return result;
}

/**
 * Mock 降级方案
 */
async function _mockResponse(command, args) {
  await new Promise(resolve => setTimeout(resolve, 200));
  switch (command) {
    case TAURI_COMMANDS.calculate:
      return _mockCalculate(args.req);
    case TAURI_COMMANDS.load:
      const saved = localStorage.getItem('jpcg_mock_config');
      return saved ? JSON.parse(saved) : null;
    default:
      return { success: true };
  }
}

function _mockCalculate(req) {
  const skills = ['剑气纵横', '流风回雪', '万剑归宗', '听雷诀', '九溪弥烟'];
  const base = (req?.player?.jichu_gongji || 1000) * 2;
  return skills.map(name => ({
    skill_name: name,
    y: 1024 + Math.floor(Math.random() * 200),
    b: base + Math.floor(Math.random() * 500),
    i: Math.floor(base * 1.2),
    n: Math.floor(base * 1.5),
    h: Math.floor(base * 2.8),
    q: Math.floor(base * 2.1),
  }));
}

export const logger = {
  info: (m, d) => console.log(`[ℹ️] ${m}`, d || ''),
  warn: (m, d) => console.warn(`[⚠️] ${m}`, d || ''),
  error: (m, e) => console.error(`[❌] ${m}`, e),
};
