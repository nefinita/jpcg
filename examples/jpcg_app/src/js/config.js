/**
 * ============================================================================
 * config.js — 应用配置与常量定义
 * 集中管理心法列表、输入字段配置、UI 参数、Tauri 命令映射、默认值等。
 * 所有数字常量集中在此文件，便于维护和调整。
 * ============================================================================
 */

/** 🧘 心法列表（与 Rust 心法名称一一对应） */
export const XINFA_LIST = [
  { value: 'yinlong',  label: '隐龙诀',   icon: '' },
  { value: 'wufang',   label: '无方',     icon: '' },
  { value: 'cangjian', label: '藏剑山庄', icon: '' },
  { value: 'qixiu',    label: '七秀坊',   icon: '' },
  { value: 'wanhua',   label: '万花谷',   icon: '' },
  { value: 'tiance',   label: '天策府',   icon: '' },
  { value: 'shaolin',  label: '少林寺',   icon: '' },
  { value: 'chunyang', label: '纯阳观',   icon: '' },
  { value: 'wudu',     label: '五毒教',   icon: '' },
  { value: 'tangmen',  label: '唐门',     icon: '' },
  { value: 'mingjiao', label: '明教',     icon: '' },
  { value: 'gaibang',  label: '丐帮',     icon: '' },
  { value: 'cangyun',  label: '苍云',     icon: '' },
  { value: 'mowen',    label: '莫问',     icon: '',   default: true },
  { value: 'badao',    label: '霸刀山庄', icon: '' },
  { value: 'penglai',  label: '蓬莱',     icon: '' },
  { value: 'yantian',  label: '衍天宗',   icon: '' },
  { value: 'yaozong',  label: '药宗',     icon: '' },
];

/** 📐 玩家属性输入字段配置 */
export const PLAYER_FIELDS = [
  { id: 'jichu_shuxing',   label: '基础属性',   min: 0, step: 1 },
  { id: 'jichu_gongji',    label: '基础攻击',   min: 0, step: 1 },
  { id: 'huixin_dengji',   label: '会心等级',   min: 0, step: 1 },
  { id: 'huixin_xiaoguo',  label: '会心效果',   min: 0, step: 1 },
  { id: 'pofang_dengji',   label: '破防等级',   min: 0, step: 1 },
  { id: 'wuqi_shanghai',   label: '武器伤害',   min: 0, step: 1 },
];

/** 🎯 目标（敌对）属性输入字段配置 */
export const HOSTILE_FIELDS = [
  { id: 'waigong_fangyu',  label: '外功防御',     min: 0, step: 1 },
  { id: 'neigong_fangyu',  label: '内功防御',     min: 0, step: 1 },
  { id: 'yujin_dengji',    label: '御劲等级',     min: 0, step: 1 },
  { id: 'huajin_dengji',   label: '化劲等级',     min: 0, step: 1 },
  { id: 'jianshang_bili',  label: '减伤比例(%)',  min: 0, max: 100, step: 0.1 },
];

/** 🎨 UI 界面参数 */
export const UI_CONFIG = {
  toastDuration: 3000,            // Toast 通知显示时长（毫秒）
  animationDuration: 250,         // 动画过渡时长（毫秒）
  debounceDelay: 150,             // 防抖延迟（毫秒）
  numberFormat: {
    locale: 'zh-CN',              // 数字格式化语言
    style: 'decimal',             // 格式风格
    maximumFractionDigits: 2,     // 最多小数位
  },
};

/** 🔌 Tauri 命令名称映射（与 Rust 端 #[tauri::command] 名称对应） */
export const TAURI_COMMANDS = {
  calculate:       'calculate_damage',          // 伤害计算
  save:            'save_config_cmd',           // 保存配置
  load:            'load_config_cmd',           // 加载配置
  loadProfession:  'load_profession_config',    // 按心法加载
  checkUpdate:     'check_update',              // 检查更新
  performUpdate:   'perform_update',            // 执行更新
  forumListFiles:  'forum_list_files',          // 论坛文件列表
  forumDownload:   'forum_download_file',       // 论坛文件下载
};

/** 🌐 论坛服务器地址（默认本地开发服务器） */
export const FORUM_URL = 'http://localhost:8080';

/** 🎯 默认配置值（计算前表单的初始值） */
export const DEFAULT_CONFIG = {
  xinfa: 'mowen',
  player: {
    jichu_shuxing: 0,
    jichu_gongji: 0,
    huixin_dengji: 0,
    huixin_xiaoguo: 0,
    pofang_dengji: 0,
    wuqi_shanghai: 0,
  },
  hostile: {
    waigong_fangyu: 0,
    neigong_fangyu: 0,
    yujin_dengji: 0,
    huajin_dengji: 0,
    jianshang_bili: 0,
  },
  xinfa_config: {
    xinfa_name: '莫问',
    xinfa_nom: 'gengu',
    atk_up: 0,
    pofang_up: 0,
    huixin_up: 0,
  },
};

/** 🌙 主题配置 */
export const THEMES = {
  dark:  { name: 'dark',  label: '🌙 深色' },
  light: { name: 'light', label: '☀️ 浅色' },
};

/** 🗄️ localStorage 存储键名 */
export const STORAGE_KEYS = {
  config: 'jpcg_user_config',       // 用户配置
  theme: 'jpcg_theme',              // 主题偏好
  lastXinfa: 'jpcg_last_xinfa',    // 上次选中的心法
};
