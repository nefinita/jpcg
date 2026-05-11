/**
 * UI 渲染 & DOM 操作工具模块
 */

import { XINFA_LIST, UI_CONFIG, DEFAULT_CONFIG } from './config.js';

/**
 * 渲染心法下拉选项
 */
export function renderXinfaOptions(selectEl, selectedValue = null) {
  if (!selectEl) return;
  
  // 清空并添加默认项
  selectEl.innerHTML = '<option value="" disabled selected>请选择心法...</option>';
  
  const fragment = document.createDocumentFragment();
  
  XINFA_LIST.forEach(xf => {
    const opt = document.createElement('option');
    opt.value = xf.value;
    opt.textContent = `${xf.icon} ${xf.label}`;
    
    if (xf.value === selectedValue || xf.default) {
      opt.selected = true;
    }
    
    fragment.appendChild(opt);
  });
  
  selectEl.appendChild(fragment);
}

/**
 * 渲染计算结果表格
 */
export function renderResults(results, containerEl) {
  if (!containerEl) return;
  
  // 空状态
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
  
  // 渲染数据行
  const rows = results.map((r, idx) => `
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
  `).join('');
  
  containerEl.innerHTML = rows;
  
  // 更新统计信息
  _updateStats(results);
}

/**
 * 更新结果统计面板
 */
function _updateStats(results) {
  const statsEl = document.getElementById('result-stats');
  const countEl = document.getElementById('result-count');
  
  if (!statsEl || !countEl) return;
  
  const validResults = results.filter(r => r.q > 0);
  
  if (validResults.length === 0) {
    statsEl.hidden = true;
    countEl.textContent = '0 个技能';
    return;
  }
  
  // 计算统计值
  const maxQ = Math.max(...validResults.map(r => r.q));
  const avgQ = validResults.reduce((sum, r) => sum + r.q, 0) / validResults.length;
  const critRatio = validResults.filter(r => r.h > r.n).length / validResults.length * 100;
  
  // 更新 DOM
  countEl.textContent = `${validResults.length} 个技能`;
  document.getElementById('stat-max-q').textContent = _formatNumber(maxQ);
  document.getElementById('stat-avg-q').textContent = _formatNumber(avgQ);
  document.getElementById('stat-crit-ratio').textContent = `${critRatio.toFixed(1)}%`;
  
  statsEl.hidden = false;
  
  // 高亮最高期望行
  _highlightMaxRow(maxQ);
}

/**
 * 高亮期望伤害最高的技能行
 */
function _highlightMaxRow(maxQ) {
  const rows = document.querySelectorAll('#result-body tr[data-skill-type]');
  rows.forEach(row => {
    const qCell = row.querySelector('.col-num.highlight:last-child');
    if (qCell && _parseNumber(qCell.textContent) === maxQ) {
      row.setAttribute('data-skill-type', 'ultimate');
    }
  });
}

/**
 * 显示 Toast 通知
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
  
  // 自动移除
  setTimeout(() => {
    toast.style.animation = 'fadeOut 0.3s ease forwards';
    setTimeout(() => toast.remove(), 300);
  }, duration);
}

/**
 * 更新状态栏
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
 * 设置按钮加载状态
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
 * 收集表单数据
 */
export function collectFormData() {
  const getVal = (id) => {
    const el = document.getElementById(id);
    return el ? el.value.trim() : '';
  };
  
  return {
    xinfa: document.getElementById('xinfa-select')?.value || DEFAULT_CONFIG.xinfa,
    player: {
	jcsx : "gengu",
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
 * 填充表单数据
 */
export function fillFormData(config) {
  if (!config) return;
  
  // 心法
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
  
  // 心法扩展配置
  if (config.xinfa_config) {
    Object.assign(DEFAULT_CONFIG.xinfa_config, config.xinfa_config);
  }
}

/**
 * 清空表单
 */
export function clearForm() {
  // 重置心法
  const select = document.getElementById('xinfa-select');
  if (select) {
    const defaultXinfa = XINFA_LIST.find(x => x.default)?.value || 'mowen';
    select.value = defaultXinfa;
  }
  
  // 清空所有输入框
  document.querySelectorAll('.form-input').forEach(el => {
    el.value = '';
  });
  
  // 清空结果
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
  
  // 隐藏统计
  const statsEl = document.getElementById('result-stats');
  if (statsEl) statsEl.hidden = true;
}

/**
 * 工具函数：数字格式化
 */
function _formatNumber(num) {
  if (num === null || num === undefined) return '-';
  return Number(num).toLocaleString(UI_CONFIG.numberFormat.locale, {
    ...UI_CONFIG.numberFormat,
    minimumFractionDigits: num % 1 === 0 ? 0 : 2,
  });
}

/**
 * 工具函数：解析带格式的数值
 */
function _parseNumber(str) {
  if (!str) return 0;
  return Number(str.replace(/,/g, '')) || 0;
}

/**
 * 工具函数：HTML 转义
 */
function _escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

/**
 * 工具函数：防抖
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
 * 工具函数：节流
 */
export function throttle(func, limit = 250) {
  let inThrottle;
  return function(...args) {
    if (!inThrottle) {
      func.apply(this, args);
      inThrottle = true;
      setTimeout(() => inThrottle = false, limit);
    }
  };
}
