/**
 * ============================================================================
 * ui.js — UI 渲染与 DOM 操作工具模块
 * 提供表单渲染、结果展示、通知提示、状态管理等纯 UI 功能。
 * 不直接调用 Tauri API，仅操作 DOM。
 * ============================================================================
 */

import { XINFA_LIST, UI_CONFIG, DEFAULT_CONFIG } from './config.js';

/**
 * 渲染心法下拉选择框
 * @param {HTMLSelectElement} selectEl - 下拉框 DOM 元素
 * @param {string|null} selectedValue - 当前选中的心法值
 */
export function renderXinfaOptions(selectEl, selectedValue = null) {
  if (!selectEl) return;

  // 清空并添加占位选项
  selectEl.innerHTML = '<option value="" disabled selected>请选择心法...</option>';

  const fragment = document.createDocumentFragment();

  XINFA_LIST.forEach((xf) => {
    const opt = document.createElement('option');
    opt.value = xf.value;
    opt.textContent = `${xf.icon} ${xf.label}`;

    // 优先使用传入值，其次使用 default 标记
    if (xf.value === selectedValue || xf.default) {
      opt.selected = true;
    }

    fragment.appendChild(opt);
  });

  selectEl.appendChild(fragment);
}

/**
 * 渲染计算结果表格
 * @param {Array} results - 计算结果数组
 * @param {HTMLElement} containerEl - 表格容器
 * @param {Object|null} comboInfo - 连招信息 { skillNames: string[], critRate: number }
 */
export function renderResults(results, containerEl, comboInfo = null) {
  if (!containerEl) return;

  if (!results || results.length === 0) {
    containerEl.innerHTML = `
      <tr class="empty-row">
        <td colspan="7">
          <div class="empty-state">
            <span class="empty-icon">🎲</span>
            <p>暂无计算结果，请检查输入后重试</p>
          </div>
        </td>
      </tr>
    `;
    return;
  }

  const rows = results
    .map(
      (r, idx) => `
    <tr class="fade-in" style="animation-delay: ${idx * 30}ms" data-skill-type="normal">
      <td class="col-name" title="${r.skill_name}">
        ${_escapeHtml(r.skill_name)}
      </td>
      <td class="col-num">${_formatNumber(r.y)}</td>
      <td class="col-num">${_formatNumber(r.b)}</td>
      <td class="col-num">${_formatNumber(r.i)}</td>
      <td class="col-num">${_formatNumber(r.n)}</td>
      <td class="col-num highlight">${_formatNumber(r.h)}</td>
      <td class="col-num highlight">${_formatNumber(r.q)}</td>
    </tr>
  `
    )
    .join('');

  containerEl.innerHTML = rows;
  _updateStats(results, comboInfo);
}

/**
 * 更新结果统计面板
 * 有连招时显示连招专属数据，否则显示常规统计
 * @param {Array} results - 计算结果
 * @param {Object|null} comboInfo - 连招信息 { skillNames: string[], critRate: number }
 */
function _updateStats(results, comboInfo = null) {
  const statsEl = document.getElementById('result-stats');
  const countEl = document.getElementById('result-count');
  const label1 = document.getElementById('stat-label-1');
  const label2 = document.getElementById('stat-label-2');
  const label3 = document.getElementById('stat-label-3');

  if (!statsEl || !countEl) return;

  const validResults = results.filter((r) => r.q > 0);

  if (validResults.length === 0) {
    statsEl.hidden = true;
    countEl.textContent = '0 个技能';
    return;
  }

  // 检查是否有连招
  const hasCombo = comboInfo && comboInfo.skillNames && comboInfo.skillNames.length > 0;
  const comboResults = hasCombo
    ? validResults.filter((r) => comboInfo.skillNames.includes(r.skill_name))
    : [];

  if (hasCombo && comboResults.length > 0) {
    const totalH = comboResults.reduce((sum, r) => sum + r.h, 0);
    const totalQ = comboResults.reduce((sum, r) => sum + r.q, 0);

    countEl.textContent = `连招 ${comboResults.length} 个技能`;
    if (label1) label1.textContent = '连招会心';
    if (label2) label2.textContent = '连招期望';
    if (label3) label3.textContent = '会心率';

    document.getElementById('stat-max-q').textContent = _formatNumber(totalH);
    document.getElementById('stat-avg-q').textContent = _formatNumber(totalQ);
    document.getElementById('stat-crit-ratio').textContent = `${comboInfo.critRate.toFixed(2)}%`;
    statsEl.hidden = false;
    return;
  }

  // 常规统计
  if (label1) label1.textContent = '最大期望';
  if (label2) label2.textContent = '平均期望';
  if (label3) label3.textContent = '会心占比';

  const maxQ = Math.max(...validResults.map((r) => r.q));
  const avgQ = validResults.reduce((sum, r) => sum + r.q, 0) / validResults.length;
  const critRatio = (validResults.filter((r) => r.h > r.n).length / validResults.length) * 100;

  countEl.textContent = `${validResults.length} 个技能`;
  document.getElementById('stat-max-q').textContent = _formatNumber(maxQ);
  document.getElementById('stat-avg-q').textContent = _formatNumber(avgQ);
  document.getElementById('stat-crit-ratio').textContent = `${critRatio.toFixed(1)}%`;

  statsEl.hidden = false;
  _highlightMaxRow(maxQ);
}

/**
 * 高亮期望伤害最高的技能行（标注为 ultimate 样式）
 * @param {number} maxQ - 最高期望值
 */
function _highlightMaxRow(maxQ) {
  const rows = document.querySelectorAll('#result-body tr[data-skill-type]');
  rows.forEach((row) => {
    const qCell = row.querySelector('.col-num.highlight:last-child');
    if (qCell && _parseNumber(qCell.textContent) === maxQ) {
      row.setAttribute('data-skill-type', 'ultimate');
    }
  });
}

/**
 * 显示 Toast 通知
 * @param {string} message - 通知消息
 * @param {'success'|'error'|'warning'|'info'} type - 通知类型
 * @param {number} duration - 显示时长（毫秒）
 */
export function showToast(message, type = 'info', duration = UI_CONFIG.toastDuration) {
  const container = document.getElementById('toast-container');
  if (!container) return;

  const icons = {
    success: '✅',
    error: '❌',
    warning: '⚠️',
    info: 'ℹ️',
  };

  const toast = document.createElement('div');
  toast.className = `toast ${type} fade-in`;
  toast.innerHTML = `
    <span class="toast-icon">${icons[type] || icons.info}</span>
    <span>${_escapeHtml(message)}</span>
  `;

  container.appendChild(toast);

  // 自动淡出移除
  setTimeout(() => {
    toast.style.animation = 'fadeOut 0.3s ease forwards';
    setTimeout(() => toast.remove(), 300);
  }, duration);
}

/**
 * 更新状态栏消息和状态指示灯
 * @param {string} message - 状态消息
 * @param {'ready'|'loading'|'error'} status - 状态类型
 */
export function updateStatus(message, status = 'ready') {
  const msgEl = document.getElementById('status-message');
  const dotEl = document.getElementById('status-dot');

  if (msgEl) msgEl.textContent = message;
  if (dotEl) {
    dotEl.setAttribute('data-status', status);
  }
}

/**
 * 设置按钮加载状态（显示加载中动画并禁用点击）
 * @param {HTMLElement} btnEl - 按钮元素
 * @param {boolean} loading - 是否加载中
 */
export function setLoading(btnEl, loading = true) {
  if (!btnEl) return;

  if (loading) {
    btnEl.classList.add('loading');
    btnEl.dataset.originalText = btnEl.innerHTML;
    btnEl.disabled = true;
  } else {
    btnEl.classList.remove('loading');
    btnEl.disabled = false;
    if (btnEl.dataset.originalText) {
      btnEl.innerHTML = btnEl.dataset.originalText;
      delete btnEl.dataset.originalText;
    }
  }
}

/**
 * 收集当前表单数据
 * @returns {Object} 包含 xinfa, player, hostile, xinfa_config 的配置对象
 */
export function collectFormData() {
  const getVal = (id) => {
    const el = document.getElementById(id);
    return el ? el.value.trim() : '';
  };

  return {
    xinfa: document.getElementById('xinfa-select')?.value || DEFAULT_CONFIG.xinfa,
    player: {
      jcsx: 'gengu',
      jichu_shuxing: getVal('jichu_shuxing'),
      jichu_gongji: getVal('jichu_gongji'),
      huixin_dengji: getVal('huixin_dengji'),
      huixin_xiaoguo: getVal('huixin_xiaoguo'),
      pofang_dengji: getVal('pofang_dengji'),
      wuqi_shanghai: getVal('wuqi_shanghai'),
    },
    hostile: {
      waigong_fangyu: getVal('waigong_fangyu'),
      neigong_fangyu: getVal('neigong_fangyu'),
      yujin_dengji: getVal('yujin_dengji'),
      huajin_dengji: getVal('huajin_dengji'),
      jianshang_bili: getVal('jianshang_bili'),
    },
    xinfa_config: { ...DEFAULT_CONFIG.xinfa_config },
  };
}

/**
 * 用配置数据填充表单
 * @param {Object} config - 配置数据对象
 */
export function fillFormData(config) {
  if (!config) return;

  // 心法选择
  if (config.xinfa) {
    const select = document.getElementById('xinfa-select');
    if (select) select.value = config.xinfa;
  }

  // 玩家属性
  if (config.player) {
    Object.entries(config.player).forEach(([key, val]) => {
      const el = document.getElementById(key);
      if (el && val !== undefined) {
        el.value = val;
      }
    });
  }

  // 目标属性
  if (config.hostile) {
    Object.entries(config.hostile).forEach(([key, val]) => {
      const el = document.getElementById(key);
      if (el && val !== undefined) {
        el.value = val;
      }
    });
  }

  // 心法扩展配置（合并到默认配置）
  if (config.xinfa_config) {
    Object.assign(DEFAULT_CONFIG.xinfa_config, config.xinfa_config);
  }
}

/**
 * 清空表单（重置为初始状态）
 */
export function clearForm() {
  // 重置心法选择
  const select = document.getElementById('xinfa-select');
  if (select) {
    const defaultXinfa = XINFA_LIST.find((x) => x.default)?.value || 'mowen';
    select.value = defaultXinfa;
  }

  // 清空所有输入框
  document.querySelectorAll('.form-input').forEach((el) => {
    el.value = '';
  });

  // 重置结果表格为初始空状态
  const resultBody = document.getElementById('result-body');
  if (resultBody) {
    resultBody.innerHTML = `
      <tr class="empty-row">
        <td colspan="7">
          <div class="empty-state">
            <span class="empty-icon">🎲</span>
            <p>填写属性后点击「开始计算」查看结果</p>
          </div>
        </td>
      </tr>
    `;
  }

  // 隐藏统计面板
  const statsEl = document.getElementById('result-stats');
  if (statsEl) statsEl.hidden = true;
}

/**
 * ============================================================================
 * 工具函数
 * ============================================================================
 */

/**
 * 数字格式化（添加千分位分隔符）
 * @param {number} num - 待格式化数字
 * @returns {string} 格式化后的字符串
 */
export function _formatNumber(num) {
  if (num === null || num === undefined) return '-';
  return Number(num).toLocaleString(UI_CONFIG.numberFormat.locale, {
    ...UI_CONFIG.numberFormat,
    minimumFractionDigits: num % 1 === 0 ? 0 : 2,
  });
}

/**
 * 解析带千分位分隔符的字符串为数字
 * @param {string} str - 格式化后的数字字符串
 * @returns {number}
 */
function _parseNumber(str) {
  if (!str) return 0;
  return Number(str.replace(/,/g, '')) || 0;
}

/**
 * HTML 转义（防止 XSS）
 * @param {string} text - 原始文本
 * @returns {string} 转义后的安全 HTML
 */
function _escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

/**
 * 防抖函数
 * @param {Function} func - 要执行的函数
 * @param {number} wait - 等待时间（毫秒）
 * @returns {Function} 防抖包装后的函数
 */
export function debounce(func, wait = UI_CONFIG.debounceDelay) {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

/**
 * 节流函数
 * @param {Function} func - 要执行的函数
 * @param {number} limit - 时间限制（毫秒）
 * @returns {Function} 节流包装后的函数
 */
export function throttle(func, limit = 250) {
  let inThrottle;
  return function (...args) {
    if (!inThrottle) {
      func.apply(this, args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}
