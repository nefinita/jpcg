// js/app.js (替换或覆盖原有初始化逻辑)
import { XINFA_LIST, STORAGE_KEYS, THEMES } from './config.js';
import { calculateDamage, saveConfig, loadConfig, logger, isTauri } from './api.js';
import {
  renderXinfaOptions, renderResults, showToast, updateStatus, 
  setLoading, collectFormData, fillFormData, clearForm
} from './ui.js';

console.log('🟢 app.js 已加载，等待 DOM...');

document.addEventListener('DOMContentLoaded', async () => {
  console.log('🟢 DOMContentLoaded 触发');
  try {
    await initApp();
  } catch (err) {
    console.error('❌ 应用初始化失败:', err);
    showToast('应用启动异常，请查看控制台', 'error');
  }
});

async function initApp() {
  // 1. 安全获取 DOM
  const getEl = (id) => {
    const el = document.getElementById(id);
    if (!el) console.warn(`⚠️ 未找到元素: #${id}`);
    return el;
  };

  const DOM = {
    xinfaSelect: getEl('xinfa-select'),
    resultBody: getEl('result-body'),
    configStatus: getEl('config-status'),
    btnCalc: getEl('btn-calculate'),
    btnSave: getEl('btn-save'),
    btnLoad: getEl('btn-load'),
    btnClear: getEl('btn-clear'),
    btnExport: getEl('btn-export'),
    btnTheme: getEl('theme-toggle'),
    inputs: {
      player: ['jichu_shuxing', 'jichu_gongji', 'huixin_dengji', 'huixin_xiaoguo', 'pofang_dengji', 'wuqi_shanghai'].map(getEl),
      hostile: ['waigong_fangyu', 'neigong_fangyu', 'yujin_dengji', 'huajin_dengji', 'jianshang_bili'].map(getEl)
    }
  };

  if (!DOM.xinfaSelect || !DOM.btnCalc) {
    console.error('❌ 关键元素缺失，请检查 HTML ID 是否匹配');
    return;
  }

  // 2. 绑定事件（带空值保护）
  const bind = (el, event, handler) => el?.addEventListener(event, handler);

  bind(DOM.xinfaSelect, 'change', (e) => {
    console.log('🧘 心法切换:', e.target.value);
    localStorage.setItem(STORAGE_KEYS.lastXinfa, e.target.value);
  });

  bind(DOM.btnCalc, 'click', async () => {
    console.log('🧮 点击计算按钮');
    DOM.btnCalc.disabled = true;
    DOM.btnCalc.textContent = '计算中...';
    try {
      const config = collectFormData();
      if (!config.xinfa) throw new Error('请先选择心法');
      const results = await calculateDamage(config);
      renderResults(results, DOM.resultBody);
      showToast('✅ 计算完成', 'success');
    } catch (err) {
      console.error('计算失败:', err);
      showToast(err.message || '计算失败', 'error');
    } finally {
      DOM.btnCalc.disabled = false;
      DOM.btnCalc.innerHTML = '<span class="btn-icon">🧮</span> 开始计算';
    }
  });

  bind(DOM.btnSave, 'click', async () => {
    console.log('💾 点击保存');
    try {
      await saveConfig(collectFormData());
      showToast('已保存', 'success');
    } catch (e) { showToast(e.message, 'error'); }
  });

  bind(DOM.btnLoad, 'click', async () => {
    console.log('📥 点击加载');
    try {
      const cfg = await loadConfig();
      if (cfg) { fillFormData(cfg); showToast('已加载', 'success'); }
      else showToast('无保存记录', 'warning');
    } catch (e) { showToast(e.message, 'error'); }
  });

  bind(DOM.btnClear, 'click', () => {
    console.log('🗑️ 点击清空');
    clearForm();
    showToast('已清空', 'info');
  });

  bind(DOM.btnExport, 'click', () => showToast('导出功能开发中...', 'info'));
  bind(DOM.btnTheme, 'click', () => {
    const next = document.documentElement.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
    document.documentElement.setAttribute('data-theme', next);
    showToast(`主题: ${next}`, 'info');
  });

  // 3. 初始化 UI
  renderXinfaOptions(DOM.xinfaSelect, localStorage.getItem(STORAGE_KEYS.lastXinfa));
  console.log('✅ 初始化完成，按钮已激活');
}