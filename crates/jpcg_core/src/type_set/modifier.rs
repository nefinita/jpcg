// ============================================================================
// modifier —— 技能秘籍 / 奇穴（#92 时序引擎）
//
// 设计口径（见 issue #92）：
//   - 秘籍：挂在**默认技能**上的修正项。激活后直接参与该技能伤害/时间计算，
//     不新增「同名技能 + jihuoqixue」重复条目。
//   - 奇穴：激活后把默认技能**替换**为独立的奇穴技能条目（`replacement`），
//     并可附带修正项；替换后技能名/ID 取 replacement 自身。
//   - 数据全部**手工填入**（`*.manual.toml` 或 data/manual/），
//     由 json-to-toml 全量重生成数据时只合并不覆盖。
//   - `Skilltype::jihuoqixue` 字段仅为旧数据兼容保留，新数据不再使用。
// ============================================================================

use serde::{Deserialize, Serialize};

use crate::type_set::skilltype::Skilltype;

/// 单条技能修正项（作用在 Skilltype 上）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillModifier {
    /// 基础伤害 +N（加在 base_damage1/base_damage2 上）
    BaseDamageAdd(u32),
    /// 基础伤害 ×(1 + v/100)（负值 = 降低）
    BaseDamageMul(f32),
    /// 攻击系数 +v（直接相加）
    AtkXishuAdd(f32),
    /// 增伤 +v 个百分点
    HitUpAdd(u32),
    /// 会心率 +v 个百分点
    HuixinUpAdd(u32),
    /// 会心效果 +v 个百分点
    HuixiaoUpAdd(u32),
    /// 无视防御 +v（1024 制）
    WushifangyuAdd(u32),
    /// 读条时长 ×(1 + v/100)（负值 = 缩短）
    CastTimeMul(f32),
    /// 调息时间 ×(1 + v/100)（负值 = 缩短）
    CooldownMul(f32),
    /// Dot 持续时长 ×(1 + v/100)
    DotDurationMul(f32),
    /// Dot 每跳间隔 ×(1 + v/100)
    DotIntervalMul(f32),
}

impl SkillModifier {
    /// 就地修改技能
    pub fn apply(&self, skill: &mut Skilltype) {
        match *self {
            SkillModifier::BaseDamageAdd(v) => {
                skill.base_damage1 = skill.base_damage1.saturating_add(v);
                skill.base_damage2 = skill.base_damage2.saturating_add(v);
            }
            SkillModifier::BaseDamageMul(v) => {
                let k = 1.0 + v / 100.0;
                skill.base_damage1 = ((skill.base_damage1 as f32) * k) as u32;
                skill.base_damage2 = ((skill.base_damage2 as f32) * k) as u32;
            }
            SkillModifier::AtkXishuAdd(v) => skill.atk_xishu += v,
            SkillModifier::HitUpAdd(v) => skill.hit_up = skill.hit_up.saturating_add(v),
            SkillModifier::HuixinUpAdd(v) => skill.huixin_up = skill.huixin_up.saturating_add(v),
            SkillModifier::HuixiaoUpAdd(v) => skill.huixiao_up = skill.huixiao_up.saturating_add(v),
            SkillModifier::WushifangyuAdd(v) => {
                skill.wushifangyu = skill.wushifangyu.saturating_add(v)
            }
            SkillModifier::CastTimeMul(v) => {
                skill.cast_time = (skill.cast_time * (1.0 + v / 100.0)).max(0.0)
            }
            SkillModifier::CooldownMul(v) => {
                skill.cooldown = (skill.cooldown * (1.0 + v / 100.0)).max(0.0)
            }
            SkillModifier::DotDurationMul(v) => {
                skill.dot_duration = (skill.dot_duration * (1.0 + v / 100.0)).max(0.0)
            }
            SkillModifier::DotIntervalMul(v) => {
                skill.dot_interval = (skill.dot_interval * (1.0 + v / 100.0)).max(0.0)
            }
        }
    }
}

/// 秘籍定义（多条秘籍可同时挂在同一技能上，全部生效即同时累加）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MijiDef {
    /// 唯一 id
    pub id: String,
    /// 秘籍名称（如「<技能名>·人偶图绝章」）
    pub name: String,
    /// 所属技能 id
    pub skill_id: u32,
    /// 所属子技能 id（0 = 不限）
    pub sub_id: u32,
    /// 修正项列表
    pub modifiers: Vec<SkillModifier>,
}

impl MijiDef {
    /// 是否作用于给定技能
    pub fn targets(&self, skill: &Skilltype) -> bool {
        self.skill_id == skill.skill_id && (self.sub_id == 0 || self.sub_id == skill.sub_id)
    }
}

/// 奇穴定义（把默认技能替换为奇穴技能条目）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct QixueDef {
    /// 唯一 id
    pub id: String,
    /// 奇穴名称
    pub name: String,
    /// 奇穴槽位（1~N，仅展示/约束用）
    pub slot: u8,
    /// 被替换的默认技能 id
    pub replaces_skill_id: u32,
    /// 被替换的子技能 id（0 = 不限）
    pub replaces_sub_id: u32,
    /// 替换后的独立技能条目（None = 不替换，仅施加 modifiers）
    pub replacement: Option<Skilltype>,
    /// 附加修正项（作用于替换后技能；无 replacement 时作用于默认技能）
    pub modifiers: Vec<SkillModifier>,
}

impl QixueDef {
    /// 是否替换/修改给定技能
    pub fn targets(&self, skill: &Skilltype) -> bool {
        self.replaces_skill_id == skill.skill_id
            && (self.replaces_sub_id == 0 || self.replaces_sub_id == skill.sub_id)
    }
}

/// 对技能应用一串修正项（返回新技能，原技能不变）
pub fn apply_modifiers(skill: &Skilltype, modifiers: &[SkillModifier]) -> Skilltype {
    let mut out = skill.clone();
    for m in modifiers {
        m.apply(&mut out);
    }
    out
}

/// 按已激活的秘籍 / 奇穴解析出实际参与计算的技能条目。
///
/// 规则：
/// 1. 先按激活的奇穴做替换（同技能多处命中时以列表中靠后者为准）；
/// 2. 再累加所有命中秘籍的修正项，以及奇穴自身的修正项（作用于替换后技能）。
pub fn resolve_skill(
    skill: &Skilltype,
    active_miji: &[&MijiDef],
    active_qixue: &[&QixueDef],
) -> Skilltype {
    let mut resolved = skill.clone();
    for q in active_qixue {
        if let (true, Some(rep)) = (q.targets(skill), q.replacement.as_ref()) {
            resolved = rep.clone();
        }
    }
    let mut modifiers: Vec<SkillModifier> = Vec::new();
    for m in active_miji {
        if m.targets(skill) {
            modifiers.extend(m.modifiers.iter().cloned());
        }
    }
    for q in active_qixue {
        if q.targets(skill) {
            modifiers.extend(q.modifiers.iter().cloned());
        }
    }
    apply_modifiers(&resolved, &modifiers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill() -> Skilltype {
        Skilltype {
            skill_name: "测试".into(),
            skill_id: 100,
            base_damage1: 100,
            base_damage2: 100,
            atk_xishu: 1.0,
            cast_time: 2.0,
            ..Default::default()
        }
    }

    #[test]
    fn miji_modifiers_apply_without_adding_new_skill() {
        let miji = MijiDef {
            id: "m1".into(),
            name: "秘籍".into(),
            skill_id: 100,
            sub_id: 0,
            modifiers: vec![SkillModifier::BaseDamageAdd(20), SkillModifier::CastTimeMul(-10.0)],
        };
        let out = resolve_skill(&skill(), &[&miji], &[]);
        assert_eq!(out.base_damage1, 120);
        assert_eq!(out.base_damage2, 120);
        assert!((out.cast_time - 1.8).abs() < 1e-6);
        assert_eq!(out.skill_name, "测试");
    }

    #[test]
    fn qixue_replaces_skill_entry() {
        let rep = Skilltype {
            skill_name: "奇穴版".into(),
            skill_id: 200,
            base_damage1: 300,
            base_damage2: 300,
            ..Default::default()
        };
        let qixue = QixueDef {
            id: "q1".into(),
            name: "奇穴".into(),
            slot: 1,
            replaces_skill_id: 100,
            replaces_sub_id: 0,
            replacement: Some(rep),
            modifiers: vec![SkillModifier::HitUpAdd(10)],
        };
        let out = resolve_skill(&skill(), &[], &[&qixue]);
        assert_eq!(out.skill_name, "奇穴版");
        assert_eq!(out.skill_id, 200);
        assert_eq!(out.base_damage1, 300);
        assert_eq!(out.hit_up, 10);
    }
}
