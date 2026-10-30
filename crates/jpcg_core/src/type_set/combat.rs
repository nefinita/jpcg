// ============================================================================
// combat — 战斗时序数据层（#92 时序引擎）
//
// 数据来源分两层：
//   `data/shuxing/{门派}.toml`  json-to-toml 生成（伤害/技能基础数据，不手改）
//   `data/combat/{门派}.toml`   **手工 overlay**（形态 / GCD 通道 / 施放副作用 /
//                               定时 buff / 秘籍 / 奇穴；常改的东西放这里，免编译）
//
// overlay 只新增或覆盖时序相关字段，伤害数值仍以 shuxing 数据为准。
// 解析后先 `validate()` 做结构校验，再用 `validate_against_skills()` 校验技能引用。
// ============================================================================

use serde::{Deserialize, Serialize};

use crate::type_set::buff::TimedBuffDef;
use crate::type_set::modifier::{MijiDef, QixueDef, SkillModifier};
use crate::type_set::skilltype::Skilltype;

/// 技能唯一键（skill_id, sub_id）
pub type SkillKey = (u32, u32);

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}

/// 形态定义（`[[form]]`）
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FormDef {
    /// 稳定 id（引擎与前端都引用；如 "single" / "dual"），必填
    pub id: String,
    /// 显示名
    pub name: String,
    /// 该形态的基础 GCD（秒；0 = 用门派默认）
    pub gcd_base: f32,
    /// 是否初始形态（至多一个；都不标则以第一条为准）
    pub initial: bool,
    /// 该形态可用技能（空 = 不限）
    pub skills: Vec<SkillKey>,
}

impl FormDef {
    /// 该形态的基础 GCD（秒；未配置回退门派默认）
    pub fn gcd_base_secs(&self, school_gcd_secs: f32) -> f32 {
        if self.gcd_base > 0.0 {
            self.gcd_base
        } else {
            school_gcd_secs
        }
    }
}

/// 技能施放副作用（`[[skill_combat.on_cast]]`）
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OnCast {
    /// 施放后切换到的形态 id
    pub switch_form: Option<String>,
    /// 本次施放额外占用的 GCD 通道（如刀宗断云同时顶住双持通道）
    pub gcd_lanes: Vec<String>,
    /// 施放后直接把指定技能顶进 GCD（把该技能置为不可用至本次 GCD 结束）
    pub lock_skills: Vec<SkillKey>,
}

/// 单个技能的时序 overlay（`[[skill_combat]]`）
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SkillCombatDef {
    /// 技能 id
    pub skill_id: u32,
    /// 子技能 id（0 = 主技能）
    pub sub_id: u32,
    /// 所属形态 id（None = 任意形态可用）
    pub form: Option<String>,
    /// 占用的 GCD 通道（空 = "default"）
    pub gcd_lane: String,
    /// 施放副作用
    pub on_cast: OnCast,
    /// ---- 以下为可选覆盖（None = 沿用 shuxing 数据） ----
    /// 读条时长（秒；0 = 瞬发）
    pub cast_time: Option<f32>,
    /// 基础 GCD（秒；0 = 门派默认）
    pub gcd_base: Option<f32>,
    /// GCD 行为：0 普通 / 1 不占 GCD / 2 固定（不吃加速）
    pub gcd_kind: Option<u8>,
    /// 调息时间（秒）
    pub cooldown: Option<f32>,
    /// 充能层数（0/1 = 无充能）
    pub charges: Option<u8>,
    /// 充能恢复间隔（秒）
    pub recharge: Option<f32>,
    /// 调息是否吃加速
    pub cd_haste: Option<bool>,
    /// 是否引导技能
    pub channel: Option<bool>,
    /// 施放后施加的定时 buff id（空 = 不施加）
    pub buff_id: Option<String>,
}

impl SkillCombatDef {
    /// 是否作用于给定技能
    pub fn targets(&self, skill: &Skilltype) -> bool {
        self.skill_id == skill.skill_id && self.sub_id == skill.sub_id
    }

    /// 把 overlay 的时序字段套到技能上（None 字段保持原值）
    pub fn apply_to(&self, skill: &mut Skilltype) {
        if let Some(v) = self.cast_time {
            skill.cast_time = v.max(0.0);
        }
        if let Some(v) = self.gcd_base {
            skill.gcd_base = v.max(0.0);
        }
        if let Some(v) = self.gcd_kind {
            skill.gcd_kind = v;
        }
        if let Some(v) = self.cooldown {
            skill.cooldown = v.max(0.0);
        }
        if let Some(v) = self.charges {
            skill.charges = v;
        }
        if let Some(v) = self.recharge {
            skill.recharge = v.max(0.0);
        }
        if let Some(v) = self.cd_haste {
            skill.cd_haste = v;
        }
        if let Some(v) = self.channel {
            skill.channel = v;
        }
        if let Some(v) = &self.buff_id {
            skill.buff_id = v.clone();
        }
    }
}

/// 版本/元信息（`[meta]`）
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CombatMeta {
    /// 门派（与文件名一致；缺省时由加载器补）
    pub profession: String,
    /// 数据版本（可选，用于排查「数据是哪一版」）
    pub version: String,
    /// 备注
    pub note: String,
}

/// `data/combat/{门派}.toml` 顶层结构
/// （不能 derive PartialEq：`QixueDef.replacement` 含 `Skilltype`，后者未实现）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CombatData {
    pub meta: CombatMeta,
    /// `[[form]]`
    pub form: Vec<FormDef>,
    /// `[[skill_combat]]`
    pub skill_combat: Vec<SkillCombatDef>,
    /// `[[buff]]`
    pub buff: Vec<TimedBuffDef>,
    /// `[[miji]]`
    pub miji: Vec<MijiDef>,
    /// `[[qixue]]`
    pub qixue: Vec<QixueDef>,
}

impl CombatData {
    /// 结构校验（不依赖技能表）：id 唯一、引用非空、数值合法
    pub fn validate(&self) -> Result<(), String> {
        let mut errors: Vec<String> = Vec::new();

        // ---- form ----
        let mut seen_forms: Vec<&str> = Vec::new();
        let mut initial_count = 0usize;
        for f in &self.form {
            if f.id.trim().is_empty() {
                errors.push("form: 存在空 id".into());
            } else if seen_forms.contains(&f.id.as_str()) {
                errors.push(format!("form: id 重复 `{}`", f.id));
            } else {
                seen_forms.push(f.id.as_str());
            }
            if f.initial {
                initial_count += 1;
            }
            if !f.gcd_base.is_finite() || f.gcd_base < 0.0 {
                errors.push(format!("form `{}`: gcd_base 非法（{}）", f.id, f.gcd_base));
            }
        }
        if initial_count > 1 {
            errors.push(format!(
                "form: 初始形态标了 {initial_count} 个（应至多 1 个）"
            ));
        }

        // ---- skill_combat ----
        let mut seen_skills: Vec<SkillKey> = Vec::new();
        for sc in &self.skill_combat {
            if sc.skill_id == 0 && sc.sub_id == 0 {
                errors.push("skill_combat: skill_id/sub_id 均为 0".into());
            }
            let key = (sc.skill_id, sc.sub_id);
            if seen_skills.contains(&key) {
                errors.push(format!(
                    "skill_combat: 技能 {}:{} 重复配置",
                    key.0, key.1
                ));
            } else {
                seen_skills.push(key);
            }
            if is_zero_u32(&sc.skill_id) {
                errors.push(format!(
                    "skill_combat: sub_id {} 的 skill_id 为 0",
                    sc.sub_id
                ));
            }
            if let Some(f) = &sc.form
                && !f.trim().is_empty()
                && !seen_forms.contains(&f.as_str())
            {
                errors.push(format!(
                    "skill_combat {}:{}: 引用了不存在的形态 `{}`",
                    sc.skill_id, sc.sub_id, f
                ));
            }
            if let Some(t) = sc.cast_time
                && (!t.is_finite() || t < 0.0)
            {
                errors.push(format!("skill_combat {}:{}: cast_time 非法", sc.skill_id, sc.sub_id));
            }
            if let Some(c) = sc.cooldown
                && (!c.is_finite() || c < 0.0)
            {
                errors.push(format!("skill_combat {}:{}: cooldown 非法", sc.skill_id, sc.sub_id));
            }
            if let Some(r) = sc.recharge
                && (!r.is_finite() || r < 0.0)
            {
                errors.push(format!("skill_combat {}:{}: recharge 非法", sc.skill_id, sc.sub_id));
            }
            if let Some(to) = &sc.on_cast.switch_form
                && !to.trim().is_empty()
                && !seen_forms.contains(&to.as_str())
            {
                errors.push(format!(
                    "skill_combat {}:{}: on_cast 切到不存在的形态 `{}`",
                    sc.skill_id, sc.sub_id, to
                ));
            }
        }

        // ---- buff ----
        let mut seen_buffs: Vec<&str> = Vec::new();
        for b in &self.buff {
            if b.id.trim().is_empty() {
                errors.push("buff: 存在空 id".into());
            } else if seen_buffs.contains(&b.id.as_str()) {
                errors.push(format!("buff: id 重复 `{}`", b.id));
            } else {
                seen_buffs.push(b.id.as_str());
            }
            if !b.duration.is_finite() || b.duration < 0.0 {
                errors.push(format!("buff `{}`: duration 非法", b.id));
            }
            if b.max_stacks == 0 {
                errors.push(format!("buff `{}`: max_stacks 应 >= 1", b.id));
            }
        }

        // ---- miji / qixue ----
        let mut seen_miji: Vec<&str> = Vec::new();
        for m in &self.miji {
            if m.id.trim().is_empty() {
                errors.push("miji: 存在空 id".into());
            } else if seen_miji.contains(&m.id.as_str()) {
                errors.push(format!("miji: id 重复 `{}`", m.id));
            } else {
                seen_miji.push(m.id.as_str());
            }
            if m.skill_id == 0 {
                errors.push(format!("miji `{}`: skill_id 为 0", m.id));
            }
        }
        let mut seen_qixue: Vec<&str> = Vec::new();
        for q in &self.qixue {
            if q.id.trim().is_empty() {
                errors.push("qixue: 存在空 id".into());
            } else if seen_qixue.contains(&q.id.as_str()) {
                errors.push(format!("qixue: id 重复 `{}`", q.id));
            } else {
                seen_qixue.push(q.id.as_str());
            }
            if q.replaces_skill_id == 0 {
                errors.push(format!("qixue `{}`: replaces_skill_id 为 0", q.id));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "data/combat 校验失败（{} 处）：\n- {}",
                errors.len(),
                errors.join("\n- ")
            ))
        }
    }

    /// 引用校验：形态可用技能 / skill_combat / on_cast 锁技能都必须存在于技能表
    pub fn validate_against_skills(&self, skills: &[Skilltype]) -> Result<(), String> {
        let exists = |key: SkillKey| {
            skills
                .iter()
                .any(|s| s.skill_id == key.0 && s.sub_id == key.1)
        };
        let mut errors: Vec<String> = Vec::new();
        for f in &self.form {
            for key in &f.skills {
                if !exists(*key) {
                    errors.push(format!(
                        "form `{}`: 技能 {}:{} 不在技能表中",
                        f.id, key.0, key.1
                    ));
                }
            }
        }
        for sc in &self.skill_combat {
            let key = (sc.skill_id, sc.sub_id);
            if !exists(key) {
                errors.push(format!(
                    "skill_combat: 技能 {}:{} 不在技能表中",
                    key.0, key.1
                ));
            }
            for lock in &sc.on_cast.lock_skills {
                if !exists(*lock) {
                    errors.push(format!(
                        "skill_combat {}:{}: lock_skills 指向不存在的技能 {}:{}",
                        key.0, key.1, lock.0, lock.1
                    ));
                }
            }
        }
        for m in &self.miji {
            if !skills
                .iter()
                .any(|s| s.skill_id == m.skill_id && (m.sub_id == 0 || s.sub_id == m.sub_id))
            {
                errors.push(format!(
                    "miji `{}`: 技能 {}:{} 不在技能表中",
                    m.id, m.skill_id, m.sub_id
                ));
            }
        }
        for q in &self.qixue {
            if !skills.iter().any(|s| {
                s.skill_id == q.replaces_skill_id
                    && (q.replaces_sub_id == 0 || s.sub_id == q.replaces_sub_id)
            }) {
                errors.push(format!(
                    "qixue `{}`: 被替换技能 {}:{} 不在技能表中",
                    q.id, q.replaces_skill_id, q.replaces_sub_id
                ));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "data/combat 技能引用校验失败（{} 处）：\n- {}",
                errors.len(),
                errors.join("\n- ")
            ))
        }
    }

    /// 按技能键取时序 overlay
    pub fn skill_combat_for(&self, skill: &Skilltype) -> Option<&SkillCombatDef> {
        self.skill_combat.iter().find(|s| s.targets(skill))
    }

    /// 按 id 取形态
    pub fn form_by_id(&self, id: &str) -> Option<&FormDef> {
        self.form.iter().find(|f| f.id == id)
    }

    /// 按 id 取 buff
    pub fn buff_by_id(&self, id: &str) -> Option<&TimedBuffDef> {
        self.buff.iter().find(|b| b.id == id)
    }

    /// 把 overlay 套到整张技能表上（返回新表；原表不变）
    pub fn apply_overlay(&self, skills: &[Skilltype]) -> Vec<Skilltype> {
        skills
            .iter()
            .map(|s| {
                let mut out = s.clone();
                if let Some(sc) = self.skill_combat_for(s) {
                    sc.apply_to(&mut out);
                }
                out
            })
            .collect()
    }
}

/// 对单个技能应用一串修正项（转调 modifier，便于数据层直接使用）
pub fn modify_skill(skill: &Skilltype, modifiers: &[SkillModifier]) -> Skilltype {
    crate::type_set::modifier::apply_modifiers(skill, modifiers)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[meta]
profession = "mowen"
version = "test-1"

[[form]]
id = "single"
name = "单持"
gcd_base = 1.5
initial = true
skills = [[101, 0]]

[[form]]
id = "dual"
name = "双持"
gcd_base = 1.0

[[skill_combat]]
skill_id = 101
sub_id = 0
form = "single"
gcd_lane = "single"

[[skill_combat]]
skill_id = 102
sub_id = 0
form = "dual"
gcd_lane = "dual"
buff_id = "jiasu_10"
on_cast = { switch_form = "dual", gcd_lanes = ["dual"], lock_skills = [[102, 0]] }

[[buff]]
id = "jiasu_10"
name = "疾风"
duration = 12.0
max_stacks = 1
refresh = true
ignore_cap = true
effects = [{ jiasu_pct = 10.0 }]

[[miji]]
id = "miji_101_1"
name = "断云·人偶图绝章"
skill_id = 101
modifiers = [{ hit_up_add = 3 }]

[[qixue]]
id = "qixue_101"
name = "奇穴·孤锋"
slot = 1
replaces_skill_id = 101
modifiers = [{ atk_xishu_add = 0.2 }]
"#;

    fn parse() -> CombatData {
        toml::from_str(SAMPLE).expect("sample should parse")
    }

    fn skill(id: u32, sub: u32) -> Skilltype {
        Skilltype {
            skill_id: id,
            sub_id: sub,
            ..Skilltype::default()
        }
    }

    #[test]
    fn parses_overlay_document() {
        let data = parse();
        assert_eq!(data.meta.profession, "mowen");
        assert_eq!(data.form.len(), 2);
        assert!(data.form[0].initial);
        assert_eq!(data.form[0].skills, vec![(101, 0)]);
        assert_eq!(data.skill_combat.len(), 2);
        assert_eq!(
            data.skill_combat[1].on_cast.switch_form.as_deref(),
            Some("dual")
        );
        assert_eq!(data.buff.len(), 1);
        assert!(data.buff[0].ignore_cap);
        assert!(matches!(data.buff[0].effects[0], crate::type_set::buff::BuffEffect::JiasuPct(_)));
        assert_eq!(data.miji[0].modifiers.len(), 1);
        assert_eq!(data.qixue[0].slot, 1);
    }

    #[test]
    fn unknown_field_is_rejected() {
        let bad = SAMPLE.replace("gcd_base = 1.5", "gcd_base = 1.5\nunknown_key = 1");
        let err = toml::from_str::<CombatData>(&bad).unwrap_err().to_string();
        assert!(err.contains("unknown_key"), "实际错误：{err}");
    }

    #[test]
    fn validates_and_applies_overlay() {
        let data = parse();
        data.validate().unwrap();
        let skills = vec![skill(101, 0), skill(102, 0)];
        data.validate_against_skills(&skills).unwrap();
        // 以 101 技能验证 on_cast / 形态字段来源
        let sc = data.skill_combat_for(&skills[0]).unwrap();
        assert_eq!(sc.gcd_lane, "single");
        assert_eq!(data.form_by_id("dual").unwrap().gcd_base, 1.0);
        // 覆盖字段：修改 cast_time 后 apply_overlay 生效
        let mut data2 = data.clone();
        data2.skill_combat[0].cast_time = Some(2.5);
        let out = data2.apply_overlay(&skills);
        assert_eq!(out[0].cast_time, 2.5);
        assert_eq!(out[1].cast_time, 0.0);
    }

    #[test]
    fn rejects_duplicate_and_broken_refs() {
        let mut data = parse();
        data.skill_combat.push(data.skill_combat[0].clone());
        data.form[0].id = String::new();
        data.skill_combat[0].form = Some("nope".into());
        let err = data.validate().unwrap_err();
        assert!(err.contains("重复配置"), "{err}");
        assert!(err.contains("空 id"), "{err}");
        assert!(err.contains("不存在的形态 `nope`"), "{err}");
    }

    #[test]
    fn rejects_missing_skill_refs() {
        let data = parse();
        let skills = vec![skill(101, 0)]; // 缺 102 与 lock 目标
        let err = data.validate_against_skills(&skills).unwrap_err();
        assert!(err.contains("102"), "{err}");
        assert!(err.contains("lock_skills"), "{err}");
    }

    #[test]
    fn multiple_initial_forms_rejected() {
        let mut data = parse();
        data.form[1].initial = true;
        let err = data.validate().unwrap_err();
        assert!(err.contains("初始形态"), "{err}");
    }

    #[test]
    fn template_file_parses() {
        let text = include_str!("../../../../data/combat/_template.toml");
        let data: CombatData = toml::from_str(text).expect("模板必须能解析");
        data.validate().expect("模板必须通过结构校验");
        assert_eq!(data.meta.profession, "mowen");
        assert_eq!(data.form.len(), 1);
        assert_eq!(data.skill_combat.len(), 1);
        assert_eq!(data.skill_combat[0].skill_id, 14064);
    }
}
