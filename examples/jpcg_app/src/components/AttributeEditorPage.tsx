import {
  useCallback,
  useDeferredValue,
  useEffect,
  useMemo,
  useState,
} from "react";
import {
  CheckCircle2,
  CopyPlus,
  FilePenLine,
  ListTree,
  Plus,
  Save,
  Search,
  Settings2,
  Trash2,
} from "lucide-react";
import * as api from "../api/commands";
import type { Toast } from "../hooks/useToast";
import type { AttributeConfigDocumentDTO, XinfaSummaryDTO } from "../types";
import {
  createDefaultDraft,
  createSkill,
  duplicateSkill,
  type AttributeDraft,
  type SkillDraft,
  type XinfaDraft,
} from "../features/attribute-editor/model";
import {
  parseAttributeToml,
  serializeAttributeToml,
} from "../features/attribute-editor/toml";
import {
  NumberField,
  SelectField,
  TextField,
  ToggleField,
} from "./AttributeEditorFields";
import styles from "./AttributeEditorPage.module.css";

interface Props {
  addToast: (message: string, type?: Toast["type"], duration?: number) => number;
  setStatus: (message: string) => void;
  currentXinfa: string;
}

const DESIGN_EFFECT_OPTIONS = [[0, "直接伤害"], [1, "Dot"]] as const;
const KIND_TYPE_OPTIONS = [[0, "外功"], [1, "阴性"], [2, "混元"], [3, "毒性"], [4, "阳性"]] as const;
const CAST_MODE_OPTIONS = [[0, "单体"], [1, "群体"]] as const;
const EFFECT_TYPE_OPTIONS = [[0, "有害"], [1, "有益"]] as const;

function TomlCode({ source }: { source: string }) {
  const lines = source.trimEnd().split(/\r?\n/);
  return (
    <div className={styles.codeView} aria-label="TOML 预览">
      {lines.map((line, index) => {
        const divider = line.indexOf("=");
        const isSection = /^\[{1,2}.*\]{1,2}$/.test(line);
        const key = divider >= 0 ? line.slice(0, divider).trimEnd() : "";
        const value = divider >= 0 ? line.slice(divider + 1).trimStart() : "";
        const valueClass = value.startsWith('"')
          ? styles.codeString
          : value === "true" || value === "false"
            ? styles.codeBoolean
            : styles.codeNumber;

        return (
          <div className={styles.codeLine} key={`${index}-${line}`}>
            <span className={styles.lineNumber}>{index + 1}</span>
            <span className={styles.codeContent}>
              {isSection ? (
                <span className={styles.codeSection}>{line}</span>
              ) : divider >= 0 ? (
                <>
                  <span className={styles.codeKey}>{key}</span>
                  {" = "}
                  <span className={valueClass}>{value}</span>
                </>
              ) : (
                line || " "
              )}
            </span>
          </div>
        );
      })}
    </div>
  );
}

export default function AttributeEditorPage({ addToast, setStatus, currentXinfa }: Props) {
  const [draft, setDraft] = useState<AttributeDraft>(createDefaultDraft);
  const [selectedSkillId, setSelectedSkillId] = useState(() => draft.skills[0].row_id);
  const [search, setSearch] = useState("");
  const [professions, setProfessions] = useState<XinfaSummaryDTO[]>([]);
  const [profession, setProfession] = useState(currentXinfa);
  const [fileName, setFileName] = useState("");
  const [savedSource, setSavedSource] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const deferredSearch = useDeferredValue(search);

  const selectedSkill = useMemo(
    () => draft.skills.find((skill) => skill.row_id === selectedSkillId) ?? draft.skills[0],
    [draft.skills, selectedSkillId],
  );
  const filteredSkills = useMemo(() => {
    const query = deferredSearch.trim().toLocaleLowerCase();
    return query
      ? draft.skills.filter((skill) => skill.skill_name.toLocaleLowerCase().includes(query))
      : draft.skills;
  }, [deferredSearch, draft.skills]);
  const tomlSource = useMemo(() => serializeAttributeToml(draft), [draft]);
  const lineCount = useMemo(() => tomlSource.trimEnd().split(/\r?\n/).length, [tomlSource]);
  const isDirty = savedSource !== "" && tomlSource !== savedSource;

  const applyDocument = useCallback((document: AttributeConfigDocumentDTO) => {
    const nextDraft = parseAttributeToml(document.content);
    const normalizedSource = serializeAttributeToml(nextDraft);
    setDraft(nextDraft);
    setSelectedSkillId(nextDraft.skills[0].row_id);
    setSearch("");
    setProfession(document.profession);
    setFileName(document.file_name);
    setSavedSource(normalizedSource);
  }, []);

  useEffect(() => {
    let active = true;

    async function initialize() {
      setLoading(true);
      setStatus("正在读取属性配置...");
      try {
        const options = await api.listProfessions();
        if (!active) return;
        setProfessions(options);
        const target = options.some((item) => item.value === currentXinfa)
          ? currentXinfa
          : options[0]?.value;
        if (!target) throw new Error("没有可读取的心法配置");
        const document = await api.loadAttributeConfig(target);
        if (!active) return;
        applyDocument(document);
        setStatus(`已读取 ${document.file_name}`);
      } catch (error) {
        if (!active) return;
        const message = error instanceof Error ? error.message : String(error);
        setStatus("属性配置读取失败");
        addToast(`读取失败：${message}`, "error", 4500);
      } finally {
        if (active) setLoading(false);
      }
    }

    void initialize();
    return () => {
      active = false;
    };
  }, [addToast, applyDocument, currentXinfa, setStatus]);

  const loadProfession = useCallback(async (nextProfession: string) => {
    setLoading(true);
    setStatus("正在读取属性配置...");
    try {
      const document = await api.loadAttributeConfig(nextProfession);
      applyDocument(document);
      setStatus(`已读取 ${document.file_name}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setStatus("属性配置读取失败");
      addToast(`读取失败：${message}`, "error", 4500);
    } finally {
      setLoading(false);
    }
  }, [addToast, applyDocument, setStatus]);

  const updateXinfa = useCallback((patch: Partial<XinfaDraft>) => {
    setDraft((current) => ({ ...current, xinfa: { ...current.xinfa, ...patch } }));
  }, []);

  const updateSkill = useCallback((patch: Partial<SkillDraft>) => {
    setDraft((current) => ({
      ...current,
      skills: current.skills.map((skill) =>
        skill.row_id === selectedSkillId ? { ...skill, ...patch } : skill,
      ),
    }));
  }, [selectedSkillId]);

  const handleAddSkill = useCallback(() => {
    const skill = createSkill({ skill_name: `新技能 ${draft.skills.length + 1}` });
    setDraft((current) => ({ ...current, skills: [...current.skills, skill] }));
    setSelectedSkillId(skill.row_id);
    setSearch("");
    setStatus("已添加技能");
    addToast("已添加技能", "success");
  }, [addToast, draft.skills.length, setStatus]);

  const handleDuplicateSkill = useCallback(() => {
    const copy = duplicateSkill(selectedSkill);
    const index = draft.skills.findIndex((skill) => skill.row_id === selectedSkill.row_id);
    setDraft((current) => {
      const skills = [...current.skills];
      skills.splice(index + 1, 0, copy);
      return { ...current, skills };
    });
    setSelectedSkillId(copy.row_id);
    setStatus("已复制技能");
    addToast("已复制技能", "success");
  }, [addToast, draft.skills, selectedSkill, setStatus]);

  const handleDeleteSkill = useCallback(() => {
    if (draft.skills.length === 1) {
      addToast("配置中至少需要保留一个技能", "warning");
      return;
    }
    const index = draft.skills.findIndex((skill) => skill.row_id === selectedSkill.row_id);
    const nextSkills = draft.skills.filter((skill) => skill.row_id !== selectedSkill.row_id);
    setDraft((current) => ({ ...current, skills: nextSkills }));
    setSelectedSkillId(nextSkills[Math.max(0, index - 1)].row_id);
    setStatus("已删除技能");
    addToast("已删除技能", "success");
  }, [addToast, draft.skills, selectedSkill.row_id, setStatus]);

  const handleProfessionChange = useCallback((nextProfession: string) => {
    if (isDirty && !window.confirm("当前修改尚未保存，确定切换配置吗？")) return;
    void loadProfession(nextProfession);
  }, [isDirty, loadProfession]);

  const handleSave = useCallback(async () => {
    if (!profession || loading || saving) return;
    setSaving(true);
    setStatus("正在保存属性配置...");
    try {
      const savedFileName = await api.saveAttributeConfig(profession, tomlSource);
      setFileName(savedFileName);
      setSavedSource(tomlSource);
      setStatus(`已保存 ${savedFileName}`);
      addToast(`已保存 ${savedFileName}`, "success");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setStatus("属性配置保存失败");
      addToast(`保存失败：${message}`, "error", 4500);
    } finally {
      setSaving(false);
    }
  }, [addToast, loading, profession, saving, setStatus, tomlSource]);

  return (
    <div className={styles.page} aria-busy={loading || saving}>
      <div className={styles.pageToolbar}>
        <div className={styles.pageHeading}>
          <h1>属性配置</h1>
          <div className={styles.fileMeta}>
            <span className={`${styles.syncDot} ${isDirty ? styles.syncDotDirty : ""}`} />
            {fileName || "正在读取配置"} · {draft.skills.length} 个技能 · {isDirty ? "未保存" : "已保存"}
          </div>
        </div>
        <div className={styles.toolbarActions}>
          <select
            className={styles.configSelect}
            value={profession}
            aria-label="选择已有心法配置"
            disabled={loading || saving}
            onChange={(event) => handleProfessionChange(event.target.value)}
          >
            {professions.map((item) => (
              <option key={item.value} value={item.value}>{item.label} · {item.value}</option>
            ))}
          </select>
          <button
            className={styles.primaryButton}
            type="button"
            disabled={loading || saving || !fileName || !isDirty}
            onClick={() => void handleSave()}
          >
            <Save size={15} />
            <span>{saving ? "保存中..." : "保存"}</span>
          </button>
        </div>
      </div>

      <div className={styles.workspace}>
        <aside className={styles.leftPanel}>
          <section className={styles.xinfaSection}>
            <h2 className={styles.panelTitle}><Settings2 size={14} />心法信息</h2>
            <div className={styles.xinfaFields}>
              <TextField label="心法名称" value={draft.xinfa.xinfa_name} span onChange={(value) => updateXinfa({ xinfa_name: value })} />
              <TextField label="基础属性" value={draft.xinfa.xinfa_nom} span onChange={(value) => updateXinfa({ xinfa_nom: value })} />
              <NumberField label="攻击倍率" value={draft.xinfa.atk_up} step={0.01} onChange={(value) => updateXinfa({ atk_up: value })} />
              <NumberField label="破防倍率" value={draft.xinfa.pofang_up} step={0.01} onChange={(value) => updateXinfa({ pofang_up: value })} />
            </div>
          </section>

          <section className={styles.skillsSection}>
            <div className={styles.listHeading}>
              <h2 className={styles.panelTitle}><ListTree size={14} />技能列表</h2>
              <button className={styles.iconButton} type="button" title="添加技能" aria-label="添加技能" onClick={handleAddSkill}>
                <Plus size={16} />
              </button>
            </div>
            <label className={styles.searchBox}>
              <Search size={14} />
              <input type="search" value={search} placeholder="搜索技能" aria-label="搜索技能" onChange={(event) => setSearch(event.target.value)} />
            </label>
            <div className={styles.skillList}>
              {filteredSkills.length === 0 ? (
                <div className={styles.emptyList}>没有匹配的技能</div>
              ) : filteredSkills.map((skill) => (
                <button
                  className={`${styles.skillRow} ${skill.row_id === selectedSkill.row_id ? styles.skillRowActive : ""}`}
                  type="button"
                  key={skill.row_id}
                  onClick={() => setSelectedSkillId(skill.row_id)}
                >
                  <span className={styles.skillRowText}>
                    <strong>{skill.skill_name || "未命名技能"}</strong>
                    <small>{skill.jihuoqixue ? `奇穴 · ${skill.jihuoqixue}` : `ID ${skill.skill_id} · 子ID ${skill.sub_id}`}</small>
                  </span>
                  {skill.dot_flag === 1 ? <span className={styles.dotMark}>DOT</span> : null}
                </button>
              ))}
            </div>
            <div className={styles.listFooter}><span>{draft.skills.length} 个技能</span><span>按文件顺序</span></div>
          </section>
        </aside>

        <section className={styles.inspectorPanel}>
          <div className={styles.inspectorTitlebar}>
            <div className={styles.inspectorName}><FilePenLine size={16} /><h2>{selectedSkill.skill_name || "未命名技能"}</h2></div>
            <div className={styles.inspectorActions}>
              <button className={styles.iconButton} type="button" title="复制技能" aria-label="复制技能" onClick={handleDuplicateSkill}><CopyPlus size={15} /></button>
              <button className={styles.iconButton} type="button" title="删除技能" aria-label="删除技能" onClick={handleDeleteSkill}><Trash2 size={15} /></button>
            </div>
          </div>

          <div className={styles.inspectorScroll}>
            <section className={styles.formSection}>
              <h3>基础信息</h3>
              <div className={styles.formGrid}>
                <TextField label="技能名称" value={selectedSkill.skill_name} span onChange={(value) => updateSkill({ skill_name: value })} />
                <NumberField label="技能 ID" value={selectedSkill.skill_id} max={65535} onChange={(value) => updateSkill({ skill_id: value })} />
                <NumberField label="子 ID" value={selectedSkill.sub_id} max={65535} onChange={(value) => updateSkill({ sub_id: value })} />
                <NumberField label="套路组编号" value={selectedSkill.group} max={255} onChange={(value) => updateSkill({ group: value })} />
                <NumberField label="武器编号" value={selectedSkill.weapon_request} max={255} onChange={(value) => updateSkill({ weapon_request: value })} />
                <TextField label="激活奇穴" value={selectedSkill.jihuoqixue} span onChange={(value) => updateSkill({ jihuoqixue: value })} />
              </div>
            </section>

            <section className={styles.formSection}>
              <h3>释放与效果</h3>
              <div className={styles.formGrid}>
                <SelectField label="技能生效方式" value={selectedSkill.design_effect} options={DESIGN_EFFECT_OPTIONS} onChange={(value) => updateSkill({ design_effect: value })} />
                <SelectField label="技能类型" value={selectedSkill.kind_type} options={KIND_TYPE_OPTIONS} onChange={(value) => updateSkill({ kind_type: value })} />
                <SelectField label="释放方式" value={selectedSkill.cast_mode} options={CAST_MODE_OPTIONS} onChange={(value) => updateSkill({ cast_mode: value })} />
                <SelectField label="技能效果" value={selectedSkill.effect_type} options={EFFECT_TYPE_OPTIONS} onChange={(value) => updateSkill({ effect_type: value })} />
              </div>
              <div className={styles.toggleGrid}>
                <ToggleField label="必定命中" value={selectedSkill.guaranteed_hit} onChange={(value) => updateSkill({ guaranteed_hit: value })} />
                <ToggleField label="可以暴击" value={selectedSkill.has_critical_strike} onChange={(value) => updateSkill({ has_critical_strike: value })} />
                <ToggleField label="DOT 伤害" value={selectedSkill.dot_flag === 1} onChange={(value) => updateSkill({ dot_flag: value ? 1 : 0 })} />
              </div>
            </section>

            <section className={styles.formSection}>
              <h3>伤害参数</h3>
              <div className={styles.formGrid}>
                <NumberField label="基础伤害 1" value={selectedSkill.base_damage1} onChange={(value) => updateSkill({ base_damage1: value })} />
                <NumberField label="基础伤害 2" value={selectedSkill.base_damage2} onChange={(value) => updateSkill({ base_damage2: value })} />
                <NumberField label="伤害系数" value={selectedSkill.atk_xishu} step={0.000001} onChange={(value) => updateSkill({ atk_xishu: value })} />
                <NumberField label="武器伤害系数" value={selectedSkill.watk_xishu} onChange={(value) => updateSkill({ watk_xishu: value })} />
                <NumberField label="增伤" value={selectedSkill.hit_up} onChange={(value) => updateSkill({ hit_up: value })} />
                <NumberField label="额外会心" value={selectedSkill.huixin_up} onChange={(value) => updateSkill({ huixin_up: value })} />
                <NumberField label="额外会效" value={selectedSkill.huixiao_up} onChange={(value) => updateSkill({ huixiao_up: value })} />
                <NumberField label="DOT 增益" value={selectedSkill.dot_up} step={0.01} disabled={selectedSkill.dot_flag !== 1} onChange={(value) => updateSkill({ dot_up: value })} />
              </div>
            </section>

            <section className={styles.formSection}>
              <h3>穿透与减免</h3>
              <div className={styles.formGrid}>
                <NumberField label="无视防御 (%)" value={selectedSkill.wushifangyu} max={100} onChange={(value) => updateSkill({ wushifangyu: value })} />
                <NumberField label="无视化劲 (%)" value={selectedSkill.wushihuajin} max={100} onChange={(value) => updateSkill({ wushihuajin: value })} />
                <NumberField label="无视减伤 (%)" value={selectedSkill.wushijianshang} max={100} onChange={(value) => updateSkill({ wushijianshang: value })} />
                <NumberField label="真实伤害 (%)" value={selectedSkill.zhenshishanghai} max={100} onChange={(value) => updateSkill({ zhenshishanghai: value })} />
              </div>
            </section>
          </div>
        </section>

        <aside className={styles.previewPanel}>
          <div className={styles.previewTitlebar}>
            <h2>TOML 预览</h2>
          </div>
          <TomlCode source={tomlSource} />
          <div className={styles.previewStatus}>
            <CheckCircle2 size={13} />
            <span>{isDirty ? "更改等待保存" : "内容已保存"}</span>
            <span>{lineCount} 行</span>
          </div>
        </aside>
      </div>
    </div>
  );
}
