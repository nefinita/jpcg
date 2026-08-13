import { useState, useEffect, useCallback } from "react";
import type { SkillEditorDataDTO, SkillEditorItemDTO, XinfaSummaryDTO } from "../types";
import * as api from "../api/commands";
import styles from "./SkillEditorPage.module.css";

interface Props {
  addToast?: (msg: string, type?: "success" | "error" | "warning" | "info") => void;
}

const EMPTY_SKILL = (): SkillEditorItemDTO => ({
  skill_name: "",
  skill_id: 0,
  sub_id: 0,
  group: 0,
  weapon_request: 0,
  design_effect: 0,
  kind_type: 0,
  cast_mode: 0,
  guaranteed_hit: false,
  has_critical_strike: false,
  effect_type: 0,
  jihuoqixue: "",
  base_damage1: 0,
  base_damage2: 0,
  atk_xishu: 0,
  watk_xishu: 0,
  hit_up: 0,
  huixin_up: 0,
  huixiao_up: 0,
  wushifangyu: 0,
  wushihuajin: 0,
  wushijianshang: 0,
  zhenshishanghai: 0,
  dot_flag: 0,
  dot_interval: 0,
  dot_duration: 0,
  dot_up: 0,
});

const NOM_OPTIONS = ["根骨", "力道", "身法", "元气"];

export default function SkillEditorPage({ addToast }: Props) {
  const [profession, setProfession] = useState("mowen");
  const [professionOptions, setProfessionOptions] = useState<XinfaSummaryDTO[]>([]);
  const [data, setData] = useState<SkillEditorDataDTO | null>(null);
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [showNewDialog, setShowNewDialog] = useState(false);
  const [newForm, setNewForm] = useState({
    profession: "",
    xinfa_name: "",
    xinfa_nom: "根骨",
    atk_up: "1.96",
    pofang_up: "2.0",
    huixin_up: "0",
    version_level: "130",
    version_season: "3",
  });

  useEffect(() => {
    api.listProfessions().then(setProfessionOptions).catch(() => {});
  }, []);

  useEffect(() => {
    if (professionOptions.length > 0) return;
    api.listProfessions().then(setProfessionOptions).catch(() => {});
  }, [professionOptions.length]);

  const loadData = useCallback(async (prof: string) => {
    setLoading(true);
    try {
      const result = await api.loadSkillData(prof);
      setData(result);
      setSelectedIndex(null);
    } catch (err) {
      addToast?.(String(err), "error");
    } finally {
      setLoading(false);
    }
  }, [addToast]);

  useEffect(() => {
    loadData(profession);
  }, [profession, loadData]);

  const handleSave = useCallback(async () => {
    if (!data) return;
    setSaving(true);
    try {
      await api.saveSkillData(profession, data);
      addToast?.("技能数据已保存", "success");
      api.listProfessions().then(setProfessionOptions).catch(() => {});
    } catch (err) {
      addToast?.(String(err), "error");
    } finally {
      setSaving(false);
    }
  }, [profession, data, addToast]);

  const handleAdd = useCallback(() => {
    if (!data) return;
    const newSkill = EMPTY_SKILL();
    const newSkills = [...data.skills, newSkill];
    setData({ ...data, skills: newSkills });
    setSelectedIndex(newSkills.length - 1);
  }, [data]);

  const handleDelete = useCallback(() => {
    if (data == null || selectedIndex == null) return;
    const newSkills = data.skills.filter((_, i) => i !== selectedIndex);
    setData({ ...data, skills: newSkills });
    setSelectedIndex(newSkills.length > 0 ? Math.min(selectedIndex, newSkills.length - 1) : null);
  }, [data, selectedIndex]);

  const handleNewCreate = useCallback(() => {
    const code = newForm.profession.trim().toLowerCase();
    if (!code) { addToast?.("请输入心法代码", "warning"); return; }
    if (!/^[a-z][a-z0-9_]*$/.test(code)) { addToast?.("心法代码只能包含小写字母、数字、下划线，且以字母开头", "warning"); return; }
    if (professionOptions.some((x) => x.value === code)) { addToast?.("该心法代码已存在", "warning"); return; }
    const name = newForm.xinfa_name.trim() || code;
    const nom = newForm.xinfa_nom || "根骨";
    const newData: SkillEditorDataDTO = {
      xinfa: {
        profession: code,
        xinfa_name: name,
        xinfa_nom: nom,
        atk_up: Number(newForm.atk_up) || 0,
        pofang_up: Number(newForm.pofang_up) || 0,
        huixin_up: Number(newForm.huixin_up) || 0,
      },
      version: {
        level: Number(newForm.version_level) || 130,
        season: Number(newForm.version_season) || 1,
        modified: Math.floor(Date.now() / 1000),
      },
      skills: [],
    };
    setData(newData);
    setProfession(code);
    setSelectedIndex(null);
    setShowNewDialog(false);
    addToast?.("新建配置已创建，添加技能后保存", "info");
  }, [newForm, professionOptions, addToast]);

  const updateSkill = useCallback((index: number, field: string, value: unknown) => {
    if (!data) return;
    const newSkills = data.skills.map((s, i) =>
      i === index ? { ...s, [field]: value } : s,
    );
    setData({ ...data, skills: newSkills });
  }, [data]);

  const selectedSkill = data != null && selectedIndex != null ? data.skills[selectedIndex] : null;

  const renderField = (label: string, field: keyof SkillEditorItemDTO, type: "number" | "text" | "checkbox" = "number", opts?: { min?: number; step?: number }) => {
    if (!data || selectedIndex == null || !selectedSkill) return null;
    const idx = selectedIndex;
    const value = selectedSkill[field];
    return (
      <div className={styles.field}>
        <label className={styles.fieldLabel}>{label}</label>
        {type === "checkbox" ? (
          <input
            className={styles.checkbox}
            type="checkbox"
            checked={!!value}
            onChange={(e) => updateSkill(idx, field, e.target.checked)}
          />
        ) : type === "text" ? (
          <input
            className={styles.input}
            type="text"
            value={String(value as string)}
            onChange={(e) => updateSkill(idx, field, e.target.value)}
          />
        ) : (
          <input
            className={styles.input}
            type="number"
            min={opts?.min ?? 0}
            step={opts?.step ?? (field === "atk_xishu" || field === "dot_up" ? 0.000001 : 1)}
            value={value as number}
            onChange={(e) => updateSkill(idx, field, e.target.value === "" ? 0 : Number(e.target.value))}
          />
        )}
      </div>
    );
  };

  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <select
            className={styles.select}
            value={profession}
            onChange={(e) => setProfession(e.target.value)}
          >
            {professionOptions.map((x) => (
              <option key={x.value} value={x.value}>{x.label}（{x.nom}）</option>
            ))}
          </select>
          {data && (
            <span className={styles.versionInfo}>
              版本: {data.version ? `${data.version.level}级第${data.version.season}赛季` : "无版本信息"}
            </span>
          )}
        </div>
        <button className={styles.btn} onClick={() => setShowNewDialog(true)}>新建</button>
        <button className={`${styles.btn} ${styles.btnPrimary}`} onClick={handleSave} disabled={saving || !data}>
          {saving ? "保存中..." : "保存"}
        </button>
      </div>

      <div className={styles.body}>
        <div className={styles.listPanel}>
          <div className={styles.listHeader}>
            <span className={styles.listTitle}>技能列表（{data?.skills.length ?? 0}）</span>
          </div>
          <div className={styles.list}>
            {loading ? (
              <div className={styles.empty}>加载中...</div>
            ) : !data || data.skills.length === 0 ? (
              <div className={styles.empty}>暂无技能数据</div>
            ) : (
              data.skills.map((s, i) => (
                <button
                  key={i}
                  className={`${styles.listItem} ${selectedIndex === i ? styles.listItemActive : ""}`}
                  onClick={() => setSelectedIndex(i)}
                >
                  <span className={styles.listItemName}>{s.skill_name || "(未命名)"}</span>
                  <span className={styles.listItemId}>#{s.skill_id}</span>
                </button>
              ))
            )}
          </div>
          <div className={styles.listActions}>
            <button className={styles.btn} onClick={handleAdd} disabled={!data}>+ 添加</button>
            <button className={`${styles.btn} ${styles.btnDanger}`} onClick={handleDelete} disabled={selectedIndex == null}>
              删除
            </button>
          </div>
        </div>

        <div className={styles.detailPanel}>
          {!selectedSkill ? (
            <div className={styles.empty}>选择一个技能以编辑</div>
          ) : (() => {
            const idx = selectedIndex!;
            return (
            <div className={styles.detailForm}>
              <details className={styles.details} open>
                <summary className={styles.sectionTitle}>基础信息</summary>
                <div className={styles.grid}>
                  {renderField("技能名称", "skill_name", "text")}
                  {renderField("技能ID", "skill_id")}
                  {renderField("子ID", "sub_id")}
                  {renderField("套路组", "group")}
                  {renderField("激活奇穴", "jihuoqixue", "text")}
                </div>
              </details>

              <details className={styles.details} open>
                <summary className={styles.sectionTitle}>伤害系数</summary>
                <div className={styles.grid}>
                  {renderField("基础伤害(最小)", "base_damage1")}
                  {renderField("基础伤害(最大)", "base_damage2")}
                  {renderField("攻击力系数", "atk_xishu", "number", { step: 0.000001 })}
                  {renderField("武器伤害系数%", "watk_xishu")}
                  {renderField("所需武器", "weapon_request")}
                </div>
              </details>

              <details className={styles.details}>
                <summary className={styles.sectionTitle}>增益/穿透</summary>
                <div className={styles.grid}>
                  {renderField("增伤乘区%", "hit_up")}
                  {renderField("额外会心%", "huixin_up")}
                  {renderField("额外会效%", "huixiao_up")}
                  {renderField("无视防御(1024制)", "wushifangyu")}
                  {renderField("无视化劲", "wushihuajin")}
                  {renderField("无视减伤", "wushijianshang")}
                  {renderField("真实伤害", "zhenshishanghai")}
                </div>
              </details>

              <details className={styles.details}>
                <summary className={styles.sectionTitle}>标签/Dot</summary>
                <div className={styles.grid}>
                  <div className={styles.field}>
                    <label className={styles.fieldLabel}>生效方式</label>
                    <select className={styles.input} value={selectedSkill.design_effect} onChange={(e) => updateSkill(idx, "design_effect", Number(e.target.value))}>
                      <option value={0}>未指定</option>
                      <option value={1}>直接伤害</option>
                      <option value={2}>持续伤害Dot</option>
                      <option value={3}>治疗</option>
                    </select>
                  </div>
                  <div className={styles.field}>
                    <label className={styles.fieldLabel}>伤害类型</label>
                    <select className={styles.input} value={selectedSkill.kind_type} onChange={(e) => updateSkill(idx, "kind_type", Number(e.target.value))}>
                      <option value={0}>外功</option>
                      <option value={1}>毒性内功</option>
                      <option value={2}>混元内功</option>
                      <option value={3}>阳性内功</option>
                      <option value={4}>阴性内功</option>
                    </select>
                  </div>
                  <div className={styles.field}>
                    <label className={styles.fieldLabel}>释放方式</label>
                    <select className={styles.input} value={selectedSkill.cast_mode} onChange={(e) => updateSkill(idx, "cast_mode", Number(e.target.value))}>
                      <option value={0}>单体</option>
                      <option value={1}>群攻</option>
                      <option value={2}>扇形</option>
                      <option value={3}>矩形</option>
                    </select>
                  </div>
                  {renderField("效果类型", "effect_type")}
                  {renderField("必然命中", "guaranteed_hit", "checkbox")}
                  {renderField("可暴击", "has_critical_strike", "checkbox")}
                </div>
                <div className={styles.divider} />
                <div className={styles.grid}>
                  <div className={styles.field}>
                    <label className={styles.fieldLabel}>Dot标签</label>
                    <select className={styles.input} value={selectedSkill.dot_flag} onChange={(e) => updateSkill(idx, "dot_flag", Number(e.target.value))}>
                      <option value={0}>非Dot</option>
                      <option value={1}>Dot</option>
                    </select>
                  </div>
                  {renderField("每跳间隔(秒)", "dot_interval")}
                  {renderField("持续时长(秒)", "dot_duration")}
                  {renderField("Dot递增系数", "dot_up", "number", { step: 0.000001 })}
                </div>
              </details>
            </div>
          );})()}
        </div>
      </div>

      {showNewDialog && (
        <div className={styles.overlay} onClick={() => setShowNewDialog(false)}>
          <div className={styles.dialog} onClick={(e) => e.stopPropagation()}>
            <div className={styles.dialogTitle}>新建心法配置</div>
            <div className={styles.dialogBody}>
              <div className={styles.dialogField}>
                <label className={styles.dialogLabel}>心法代码 *</label>
                <input className={styles.input} type="text" placeholder="如: xinxinfa"
                  value={newForm.profession}
                  onChange={(e) => setNewForm({ ...newForm, profession: e.target.value })} />
                <span className={styles.dialogHint}>将作为文件名，仅支持小写字母、数字、下划线</span>
              </div>
              <div className={styles.dialogField}>
                <label className={styles.dialogLabel}>心法名称</label>
                <input className={styles.input} type="text" placeholder="如: 新心法"
                  value={newForm.xinfa_name}
                  onChange={(e) => setNewForm({ ...newForm, xinfa_name: e.target.value })} />
              </div>
              <div className={styles.dialogField}>
                <label className={styles.dialogLabel}>主属性</label>
                <select className={styles.input} value={newForm.xinfa_nom}
                  onChange={(e) => setNewForm({ ...newForm, xinfa_nom: e.target.value })}>
                  {NOM_OPTIONS.map((n) => <option key={n} value={n}>{n}</option>)}
                </select>
              </div>
              <div className={styles.dialogRow}>
                <div className={styles.dialogField}>
                  <label className={styles.dialogLabel}>攻击系数</label>
                  <input className={styles.input} type="number" step="0.01"
                    value={newForm.atk_up}
                    onChange={(e) => setNewForm({ ...newForm, atk_up: e.target.value })} />
                </div>
                <div className={styles.dialogField}>
                  <label className={styles.dialogLabel}>破防系数</label>
                  <input className={styles.input} type="number" step="0.01"
                    value={newForm.pofang_up}
                    onChange={(e) => setNewForm({ ...newForm, pofang_up: e.target.value })} />
                </div>
                <div className={styles.dialogField}>
                  <label className={styles.dialogLabel}>会心系数</label>
                  <input className={styles.input} type="number" step="0.01"
                    value={newForm.huixin_up}
                    onChange={(e) => setNewForm({ ...newForm, huixin_up: e.target.value })} />
                </div>
              </div>
              <div className={styles.dialogRow}>
                <div className={styles.dialogField}>
                  <label className={styles.dialogLabel}>版本等级</label>
                  <input className={styles.input} type="number"
                    value={newForm.version_level}
                    onChange={(e) => setNewForm({ ...newForm, version_level: e.target.value })} />
                </div>
                <div className={styles.dialogField}>
                  <label className={styles.dialogLabel}>赛季</label>
                  <input className={styles.input} type="number"
                    value={newForm.version_season}
                    onChange={(e) => setNewForm({ ...newForm, version_season: e.target.value })} />
                </div>
              </div>
            </div>
            <div className={styles.dialogActions}>
              <button className={styles.btn} onClick={() => setShowNewDialog(false)}>取消</button>
              <button className={`${styles.btn} ${styles.btnPrimary}`} onClick={handleNewCreate}>创建</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
