/**
 * ============================================================================
 * app.js — 主应用逻辑
 * 应用初始化、事件绑定、自动更新检查、用户交互逻辑。
 * 作为所有模块的编排者，协调 config / api / ui 三大模块。
 * ============================================================================
 */

import { XINFA_LIST, STORAGE_KEYS, THEMES, FORUM_URL } from './config.js';
import {
  calculateDamage, saveConfig, loadConfig, logger, isTauri,
  checkUpdate, performUpdate, listenUpdateProgress,
  forumListFiles, forumDownloadFile,
} from './api.js';
import {
  renderXinfaOptions, renderResults, showToast, updateStatus,
  setLoading, collectFormData, fillFormData, clearForm, _formatNumber,
} from './ui.js';

console.log('🟢 app.js 已加载，等待 DOM...');

// ============================================================================
// 入口: DOM 就绪后初始化应用
// ============================================================================

document.addEventListener('DOMContentLoaded', async () => {
  console.log('🟢 DOMContentLoaded 触发');
  try {
    await initApp();
  } catch (err) {
    console.error('❌ 应用初始化失败:', err);
    showToast('应用启动异常，请查看控制台', 'error');
  }
});

// ============================================================================
// initApp — 应用初始化
// 1. 获取所有 DOM 元素
// 2. 绑定按钮事件
// 3. 初始化 UI
// 4. 自动检查更新
// ============================================================================

async function initApp() {
  // ---- 步骤1: 安全获取 DOM 元素 ----
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
    btnForumSidebar: getEl('btn-forum-sidebar'),
    btnComboSidebar: getEl('btn-combo-sidebar'),
    sidebar: getEl('sidebar'),
    forumPanel: getEl('forum-panel'),
    forumClose: getEl('forum-close'),
    comboPanel: getEl('combo-panel'),
    comboClose: getEl('combo-close'),
    comboSequence: getEl('combo-sequence'),
    comboPoolList: getEl('combo-pool-list'),
    comboClear: getEl('combo-clear'),
    forumRefresh: getEl('forum-refresh'),
    forumSearch: getEl('forum-search'),
    forumFileList: getEl('forum-file-list'),
    forumPagination: {
      prev: getEl('page-prev'),
      next: getEl('page-next'),
      info: getEl('page-info'),
    },
    btnUpdate: getEl('btn-update'),
    updateProgress: getEl('update-progress'),
    updateProgressFill: getEl('update-progress-fill'),
    updateProgressText: getEl('update-progress-text'),
    inputs: {
      player: [
        'jichu_shuxing', 'jichu_gongji', 'huixin_dengji',
        'huixin_xiaoguo', 'pofang_dengji', 'wuqi_shanghai',
      ].map(getEl),
      hostile: [
        'waigong_fangyu', 'neigong_fangyu', 'yujin_dengji',
        'huajin_dengji', 'jianshang_bili',
      ].map(getEl),
    },
  };

  // 关键元素缺失时中止初始化
  if (!DOM.xinfaSelect || !DOM.btnCalc) {
    console.error('❌ 关键元素缺失，请检查 HTML ID 是否匹配');
    return;
  }

  // ---- 步骤2: 事件绑定辅助函数 ----
  const bind = (el, event, handler) => el?.addEventListener(event, handler);

  // ---- 事件绑定: 心法切换 ----
  bind(DOM.xinfaSelect, 'change', (e) => {
    console.log('🧘 心法切换:', e.target.value);
    localStorage.setItem(STORAGE_KEYS.lastXinfa, e.target.value);
  });

  // ---- 事件绑定: 计算按钮 ----
  // 发起伤害计算，try/catch/finally 确保按钮状态恢复
  bind(DOM.btnCalc, 'click', async () => {
    console.log('🧮 点击计算按钮');
    DOM.btnCalc.disabled = true;
    DOM.btnCalc.textContent = '计算中...';
    try {
      const config = collectFormData();
      if (!config.xinfa) throw new Error('请先选择心法');
      const results = await calculateDamage(config);

      lastResults = results;

      const hxDj = Number(config.player.huixin_dengji) || 0;
      const yjDj = Number(config.hostile.yujin_dengji) || 0;
      lastCritRate = Math.max(0, (hxDj - yjDj) / 197703) * 100;

      renderResults(results, DOM.resultBody, { skillNames: comboSkillNames, critRate: lastCritRate });
      renderComboPool();
      showToast('计算完成', 'success');
    } catch (err) {
      console.error('计算失败:', err);
      showToast(err.message || '计算失败', 'error');
    } finally {
      DOM.btnCalc.disabled = false;
      DOM.btnCalc.innerHTML = '<span class="btn-icon">🧮</span> 开始计算';
    }
  });

  // ---- 事件绑定: 保存按钮 ----
  bind(DOM.btnSave, 'click', async () => {
    console.log('💾 点击保存');
    try {
      await saveConfig(collectFormData());
      showToast('已保存', 'success');
    } catch (e) {
      showToast(e.message, 'error');
    }
  });

  // ---- 事件绑定: 加载按钮 ----
  bind(DOM.btnLoad, 'click', async () => {
    console.log('📥 点击加载');
    try {
      const cfg = await loadConfig();
      if (cfg) {
        fillFormData(cfg);
        showToast('已加载', 'success');
      } else showToast('无保存记录', 'warning');
    } catch (e) {
      showToast(e.message, 'error');
    }
  });

  // ---- 事件绑定: 清空按钮 ----
  bind(DOM.btnClear, 'click', () => {
    console.log('🗑️ 点击清空');
    clearForm();
    showToast('已清空', 'info');
  });

  // ---- 事件绑定: 导出按钮（预留） ----
  bind(DOM.btnExport, 'click', () => showToast('导出功能开发中...', 'info'));

  // ---- 事件绑定: 主题切换 ----
  bind(DOM.btnTheme, 'click', () => {
    const next =
      document.documentElement.getAttribute('data-theme') === 'dark'
        ? 'light'
        : 'dark';
    document.documentElement.setAttribute('data-theme', next);
    showToast(`主题: ${next}`, 'info');
  });

  // ---- 事件绑定: 更新按钮 ----
  // 点击后依次执行：检查 → 展示结果 → 用户确认 → 下载 → 安装
  bind(DOM.btnUpdate, 'click', async () => {
    // 防止重复点击
    if (DOM.btnUpdate.dataset.updating === 'true') return;
    DOM.btnUpdate.dataset.updating = 'true';
    DOM.btnUpdate.disabled = true;
    DOM.btnUpdate.innerHTML = '<span class="btn-icon">⏳</span> 检查中...';
    DOM.updateProgress.hidden = false;
    DOM.updateProgressFill.style.width = '0%';
    DOM.updateProgressText.textContent = '正在检查更新...';

    try {
      // 检查更新（force=true 确保重新检查）
      const result = await checkUpdate(false, true);

      // 无可用更新
      if (!result.has_data_update && !result.has_app_update) {
        showToast('已是最新，无需更新', 'success');
        return;
      }

      // 构造确认对话框消息
      let msg = '';
      if (result.has_data_update) {
        const ver = result.latest_data_version || '';
        msg = `发现数据更新 (${ver})，共 ${result.data_files_to_update.length} 个文件:\n${result.data_files_to_update.join('\n')}\n\n是否下载？`;
      }
      if (result.has_app_update) {
        msg = `发现新版本 ${result.latest_app_version}，是否更新？`;
      }

      // 用户确认
      if (!confirm(msg)) {
        showToast('已取消更新', 'info');
        return;
      }

      // 监听进度事件并更新 UI
      DOM.btnUpdate.innerHTML = '<span class="btn-icon">⬇️</span> 下载中...';
      DOM.updateProgressText.textContent = '开始下载...';

      const unlisten = listenUpdateProgress(({ stage, message, progress, file }) => {
        if (stage === 'downloading') {
          const pct = Math.round(progress * 100);
          DOM.updateProgressFill.style.width = `${pct}%`;
          DOM.updateProgressText.textContent = file ? `${file} (${pct}%)` : message;
        } else if (stage === 'installing') {
          DOM.updateProgressFill.style.width = `${Math.round(progress * 100)}%`;
          DOM.updateProgressText.textContent = message;
        } else if (stage === 'done') {
          DOM.updateProgressFill.style.width = '100%';
          DOM.updateProgressText.textContent = '更新完成';
          showToast('✅ ' + message, 'success');
        }
      });

      // 执行下载
      await performUpdate(false, result);
      unlisten();

      DOM.updateProgressFill.style.width = '100%';
      DOM.updateProgressText.textContent = '更新完成';
      showToast('更新完成！', 'success');
    } catch (e) {
      console.error('更新失败:', e);
      showToast('更新失败: ' + e.message, 'error');
      DOM.updateProgressText.textContent = '更新失败';
    } finally {
      // 恢复按钮状态
      DOM.btnUpdate.dataset.updating = 'false';
      DOM.btnUpdate.disabled = false;
      DOM.btnUpdate.innerHTML = '<span class="btn-icon">🔄</span> 更新';
      setTimeout(() => {
        DOM.updateProgress.hidden = true;
      }, 5000);
    }
  });

  // ---- 论坛分页状态 ----
  let forumPage = 1;
  const FORUM_PAGE_SIZE = 10;
  let forumFiles = [];

  // ---- 连招状态 ----
  let comboSkillNames = [];
  let lastResults = [];
  let lastCritRate = 0;

  function renderForumTable() {
    const search = (DOM.forumSearch?.value || '').toLowerCase();
    const filtered = forumFiles.filter(f => f.name.toLowerCase().includes(search));
    const totalPages = Math.max(1, Math.ceil(filtered.length / FORUM_PAGE_SIZE));
    forumPage = Math.min(forumPage, totalPages);
    const start = (forumPage - 1) * FORUM_PAGE_SIZE;
    const page = filtered.slice(start, start + FORUM_PAGE_SIZE);

    if (!page.length) {
      DOM.forumFileList.innerHTML = '<tr><td colspan="4" class="empty">暂无文件</td></tr>';
    } else {
      DOM.forumFileList.innerHTML = page.map(f => `
        <tr>
          <td>${f.name}</td>
          <td class="col-size">${(f.size / 1024).toFixed(1)} KB</td>
          <td class="col-time">${f.modified}</td>
          <td class="col-action">
            <button class="btn-download" data-file="${f.name}">下载</button>
          </td>
        </tr>
      `).join('');
    }

    DOM.forumPagination.info.textContent = `第 ${forumPage} / ${totalPages} 页`;
    DOM.forumPagination.prev.disabled = forumPage <= 1;
    DOM.forumPagination.next.disabled = forumPage >= totalPages;
  }

  async function loadForumFiles() {
    DOM.forumFileList.innerHTML = '<tr><td colspan="4" class="empty">加载中...</td></tr>';
    try {
      forumFiles = await forumListFiles(FORUM_URL);
      forumPage = 1;
      renderForumTable();
    } catch (e) {
      DOM.forumFileList.innerHTML = '<tr><td colspan="4" class="empty">加载失败: ' + (e.message || '未知错误') + '</td></tr>';
      showToast('加载论坛文件列表失败', 'error');
    }
  }

  async function handleForumDownload(filename) {
    try {
      const msg = await forumDownloadFile(filename, FORUM_URL);
      showToast(msg, 'success');
    } catch (e) {
      showToast('下载失败: ' + (e.message || '未知错误'), 'error');
    }
  }

  // ========================================================================
  // 侧栏切换逻辑（分页模式）
  // ========================================================================

  function openSidebar(panel) {
    DOM.sidebar.classList.remove('collapsed');
    DOM.forumPanel.hidden = panel !== 'forum';
    DOM.comboPanel.hidden = panel !== 'combo';
    DOM.btnForumSidebar.classList.toggle('active', panel === 'forum');
    DOM.btnComboSidebar.classList.toggle('active', panel === 'combo');
    if (panel === 'forum') loadForumFiles();
    if (panel === 'combo') renderComboPool();
  }

  function closeSidebar() {
    DOM.sidebar.classList.add('collapsed');
    DOM.btnForumSidebar.classList.remove('active');
    DOM.btnComboSidebar.classList.remove('active');
  }

  function toggleSidebar(panel) {
    const isOpen = !DOM.sidebar.classList.contains('collapsed');
    const isSamePanel =
      (panel === 'forum' && DOM.btnForumSidebar.classList.contains('active')) ||
      (panel === 'combo' && DOM.btnComboSidebar.classList.contains('active'));
    if (isOpen && isSamePanel) {
      closeSidebar();
    } else {
      openSidebar(panel);
    }
  }

  // ---- 活动栏按钮 ----
  bind(DOM.btnForumSidebar, 'click', () => toggleSidebar('forum'));
  bind(DOM.btnComboSidebar, 'click', () => toggleSidebar('combo'));

  // ---- 各页关闭按钮 ----
  bind(DOM.forumClose, 'click', closeSidebar);
  bind(DOM.comboClose, 'click', closeSidebar);

  // ========================================================================
  // 连招逻辑
  // ========================================================================

  function addToCombo(skillName) {
    if (!comboSkillNames.includes(skillName)) {
      comboSkillNames.push(skillName);
      renderComboSequence();
      renderComboPool();
      if (lastResults.length > 0) {
        renderResults(lastResults, DOM.resultBody, { skillNames: comboSkillNames, critRate: lastCritRate });
      }
    }
  }

  function removeFromCombo(skillName) {
    comboSkillNames = comboSkillNames.filter(s => s !== skillName);
    renderComboSequence();
    renderComboPool();
    if (lastResults.length > 0) {
      renderResults(lastResults, DOM.resultBody, { skillNames: comboSkillNames, critRate: lastCritRate });
    }
  }

  function renderComboSequence() {
    if (!DOM.comboSequence) return;
    if (comboSkillNames.length === 0) {
      DOM.comboSequence.innerHTML = '<span class="combo-empty">暂无技能，请从下方添加</span>';
      return;
    }
    DOM.comboSequence.innerHTML = comboSkillNames.map((name, idx) => `
      <div class="combo-chip">
        <span class="combo-idx">${idx + 1}</span>
        <span class="combo-name">${name}</span>
        <button class="combo-remove" data-skill="${name}" title="移除">✕</button>
      </div>
    `).join('');

    DOM.comboSequence.querySelectorAll('.combo-remove').forEach(btn => {
      btn.addEventListener('click', () => removeFromCombo(btn.dataset.skill));
    });
  }

  function renderComboPool() {
    if (!DOM.comboPoolList) return;
    if (lastResults.length === 0) {
      DOM.comboPoolList.innerHTML = '<span class="combo-empty">请先进行计算以获取技能列表</span>';
      return;
    }
    DOM.comboPoolList.innerHTML = lastResults.map(r => {
      const inCombo = comboSkillNames.includes(r.skill_name);
      return `
        <div class="combo-pool-skill">
          <button class="combo-add ${inCombo ? 'in-combo' : ''}" data-skill="${r.skill_name}" title="${inCombo ? '已添加' : '加入连招'}">
            ${inCombo ? '✓' : '+'}
          </button>
          <span class="combo-name">${r.skill_name}</span>
          <span class="combo-h">${_formatNumber(r.h)}</span>
        </div>
      `;
    }).join('');

    DOM.comboPoolList.querySelectorAll('.combo-add').forEach(btn => {
      btn.addEventListener('click', () => {
        if (btn.dataset.skill) {
          if (comboSkillNames.includes(btn.dataset.skill)) {
            removeFromCombo(btn.dataset.skill);
          } else {
            addToCombo(btn.dataset.skill);
          }
        }
      });
    });
  }

  bind(DOM.comboClear, 'click', () => {
    comboSkillNames = [];
    renderComboSequence();
    renderComboPool();
    if (lastResults.length > 0) {
      renderResults(lastResults, DOM.resultBody, { skillNames: [], critRate: lastCritRate });
    }
  });

  // ---- 论坛事件 ----
  bind(DOM.forumRefresh, 'click', loadForumFiles);

  bind(DOM.forumSearch, 'input', () => {
    forumPage = 1;
    renderForumTable();
  });

  bind(DOM.forumPagination.prev, 'click', () => {
    if (forumPage > 1) { forumPage--; renderForumTable(); }
  });

  bind(DOM.forumPagination.next, 'click', () => {
    forumPage++;
    renderForumTable();
  });

  bind(DOM.forumFileList, 'click', (e) => {
    const btn = e.target.closest('.btn-download');
    if (btn) handleForumDownload(btn.dataset.file);
  });

  // ---- 步骤3: 初始化 UI ----
  renderXinfaOptions(DOM.xinfaSelect, localStorage.getItem(STORAGE_KEYS.lastXinfa));

  // ---- 步骤4: 启动时自动检查更新 ----
  autoCheckUpdate(DOM);

  console.log('✅ 初始化完成，按钮已激活');
}

// ============================================================================
// autoCheckUpdate — 启动时自动检查更新
// 仅在 Tauri 环境中运行。若发现更新，通过 Toast 通知用户，
// 并让「更新」按钮脉冲闪烁以吸引注意。
// ============================================================================

async function autoCheckUpdate(DOM) {
  if (!isTauri()) return;
  try {
    console.log('🔍 启动自动检查更新...');
    const result = await checkUpdate(false, false);

    if (result.has_data_update) {
      const ver = result.latest_data_version || '';
      const files = result.data_files_to_update || [];
      showToast(
        `发现数据更新 (${ver})，${files.length} 个文件待更新。点击「更新」按钮下载`,
        'info',
        8000
      );
      DOM.btnUpdate.style.animation = 'pulse 2s infinite';
    }
    if (result.has_app_update) {
      showToast(
        `发现新版本 ${result.latest_app_version}，点击「更新」按钮升级`,
        'info',
        8000
      );
      DOM.btnUpdate.style.animation = 'pulse 2s infinite';
    }
  } catch (e) {
    console.warn('自动检查更新失败（可能网络不可用）:', e);
  }
}
