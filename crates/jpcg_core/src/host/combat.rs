// ============================================================================
// host::combat — 战斗时序数据 DTO 层（#92）
//
// 对外入口：
//   - load_combat_data(profession)       → `data/combat/{门派}.toml` 的 DTO
//   - load_combat_with_skills(profession)→ 技能池（已套 overlay）+ combat DTO
//   - 本模块同时提供 core 领域类型 ↔ jpcg_api DTO 的双向转换（供 combo 引擎复用）
// ============================================================================

use jpcg_api::{
    BuffEffectDTO, CombatDataDTO, CombatMetaDTO, FormDefDTO, MijiDefDTO, OnCastDTO, QixueDefDTO,
    SkillCombatDTO, SkillModifierDTO, SkillPoolItemDTO, TimedBuffDefDTO,
};

use crate::host::conv::skilltype_to_pool_item;
use crate::store;
use crate::type_set::buff::{BuffEffect, TimedBuffDef};
use crate::type_set::combat::{CombatData, CombatMeta, FormDef, OnCast, SkillCombatDef};
use crate::type_set::modifier::{MijiDef, QixueDef, SkillModifier};
use crate::type_set::skilltype::Skilltype;

/// 加载某门派的战斗时序 overlay（文件缺失 = 空数据，不算错误）
pub fn load_combat_data(profession: String) -> Result<CombatDataDTO, String> {
    let data = store::combat::load_combat(&profession);
    Ok(CombatDataDTO::from(data))
}

/// 加载技能池（已套 combat overlay）与 combat 数据本身（前端一次拿全）
pub fn load_combat_with_skills(
    profession: String,
) -> Result<(Vec<SkillPoolItemDTO>, CombatDataDTO), String> {
    let (skills, data) = store::combat::load_skills_with_combat(&profession)?;
    Ok((
        skills.iter().map(skilltype_to_pool_item).collect(),
        CombatDataDTO::from(data),
    ))
}

// ============ core → DTO ============

impl From<CombatMeta> for CombatMetaDTO {
    fn from(m: CombatMeta) -> Self {
        CombatMetaDTO {
            profession: m.profession,
            version: m.version,
            note: m.note,
        }
    }
}

impl From<FormDef> for FormDefDTO {
    fn from(f: FormDef) -> Self {
        FormDefDTO {
            id: f.id,
            name: f.name,
            gcd_base: f.gcd_base,
            initial: f.initial,
            skills: f.skills,
        }
    }
}

impl From<OnCast> for OnCastDTO {
    fn from(o: OnCast) -> Self {
        OnCastDTO {
            switch_form: o.switch_form,
            gcd_lanes: o.gcd_lanes,
            lock_skills: o.lock_skills,
        }
    }
}

impl From<SkillCombatDef> for SkillCombatDTO {
    fn from(s: SkillCombatDef) -> Self {
        SkillCombatDTO {
            skill_id: s.skill_id,
            sub_id: s.sub_id,
            form: s.form,
            gcd_lane: s.gcd_lane,
            on_cast: s.on_cast.into(),
            cast_time: s.cast_time,
            gcd_base: s.gcd_base,
            gcd_kind: s.gcd_kind,
            cooldown: s.cooldown,
            charges: s.charges,
            recharge: s.recharge,
            cd_haste: s.cd_haste,
            channel: s.channel,
            buff_id: s.buff_id,
        }
    }
}

impl From<BuffEffect> for BuffEffectDTO {
    fn from(e: BuffEffect) -> Self {
        match e {
            BuffEffect::BaseAtkPct(v) => BuffEffectDTO::BaseAtkPct(v),
            BuffEffect::HuixinPct(v) => BuffEffectDTO::HuixinPct(v),
            BuffEffect::HuixiaoPct(v) => BuffEffectDTO::HuixiaoPct(v),
            BuffEffect::PofangPct(v) => BuffEffectDTO::PofangPct(v),
            BuffEffect::WushiFangyuPct(v) => BuffEffectDTO::WushiFangyuPct(v),
            BuffEffect::ShanghaiPct(v) => BuffEffectDTO::ShanghaiPct(v),
            BuffEffect::JiasuPct(v) => BuffEffectDTO::JiasuPct(v),
        }
    }
}

impl From<TimedBuffDef> for TimedBuffDefDTO {
    fn from(b: TimedBuffDef) -> Self {
        TimedBuffDefDTO {
            id: b.id,
            name: b.name,
            duration: b.duration,
            max_stacks: b.max_stacks,
            refresh: b.refresh,
            ignore_cap: b.ignore_cap,
            effects: b.effects.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<SkillModifier> for SkillModifierDTO {
    fn from(m: SkillModifier) -> Self {
        match m {
            SkillModifier::BaseDamageAdd(v) => SkillModifierDTO::BaseDamageAdd(v),
            SkillModifier::BaseDamageMul(v) => SkillModifierDTO::BaseDamageMul(v),
            SkillModifier::AtkXishuAdd(v) => SkillModifierDTO::AtkXishuAdd(v),
            SkillModifier::HitUpAdd(v) => SkillModifierDTO::HitUpAdd(v),
            SkillModifier::HuixinUpAdd(v) => SkillModifierDTO::HuixinUpAdd(v),
            SkillModifier::HuixiaoUpAdd(v) => SkillModifierDTO::HuixiaoUpAdd(v),
            SkillModifier::WushifangyuAdd(v) => SkillModifierDTO::WushifangyuAdd(v),
            SkillModifier::CastTimeMul(v) => SkillModifierDTO::CastTimeMul(v),
            SkillModifier::CooldownMul(v) => SkillModifierDTO::CooldownMul(v),
            SkillModifier::DotDurationMul(v) => SkillModifierDTO::DotDurationMul(v),
            SkillModifier::DotIntervalMul(v) => SkillModifierDTO::DotIntervalMul(v),
        }
    }
}

impl From<MijiDef> for MijiDefDTO {
    fn from(m: MijiDef) -> Self {
        MijiDefDTO {
            id: m.id,
            name: m.name,
            skill_id: m.skill_id,
            sub_id: m.sub_id,
            modifiers: m.modifiers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<CombatData> for CombatDataDTO {
    fn from(d: CombatData) -> Self {
        CombatDataDTO {
            meta: d.meta.into(),
            form: d.form.into_iter().map(Into::into).collect(),
            skill_combat: d.skill_combat.into_iter().map(Into::into).collect(),
            buff: d.buff.into_iter().map(Into::into).collect(),
            miji: d.miji.into_iter().map(Into::into).collect(),
            qixue: d.qixue.into_iter().map(qixue_to_dto).collect(),
        }
    }
}

fn qixue_to_dto(q: QixueDef) -> QixueDefDTO {
    QixueDefDTO {
        id: q.id,
        name: q.name,
        slot: q.slot,
        replaces_skill_id: q.replaces_skill_id,
        replaces_sub_id: q.replaces_sub_id,
        replacement: q.replacement.as_ref().map(skilltype_to_pool_item),
        modifiers: q.modifiers.into_iter().map(Into::into).collect(),
    }
}

// ============ DTO → core ============

impl From<FormDefDTO> for FormDef {
    fn from(f: FormDefDTO) -> Self {
        FormDef {
            id: f.id,
            name: f.name,
            gcd_base: f.gcd_base,
            initial: f.initial,
            skills: f.skills,
        }
    }
}

impl From<OnCastDTO> for OnCast {
    fn from(o: OnCastDTO) -> Self {
        OnCast {
            switch_form: o.switch_form,
            gcd_lanes: o.gcd_lanes,
            lock_skills: o.lock_skills,
        }
    }
}

impl From<SkillCombatDTO> for SkillCombatDef {
    fn from(s: SkillCombatDTO) -> Self {
        SkillCombatDef {
            skill_id: s.skill_id,
            sub_id: s.sub_id,
            form: s.form,
            gcd_lane: s.gcd_lane,
            on_cast: s.on_cast.into(),
            cast_time: s.cast_time,
            gcd_base: s.gcd_base,
            gcd_kind: s.gcd_kind,
            cooldown: s.cooldown,
            charges: s.charges,
            recharge: s.recharge,
            cd_haste: s.cd_haste,
            channel: s.channel,
            buff_id: s.buff_id,
        }
    }
}

impl From<BuffEffectDTO> for BuffEffect {
    fn from(e: BuffEffectDTO) -> Self {
        match e {
            BuffEffectDTO::BaseAtkPct(v) => BuffEffect::BaseAtkPct(v),
            BuffEffectDTO::HuixinPct(v) => BuffEffect::HuixinPct(v),
            BuffEffectDTO::HuixiaoPct(v) => BuffEffect::HuixiaoPct(v),
            BuffEffectDTO::PofangPct(v) => BuffEffect::PofangPct(v),
            BuffEffectDTO::WushiFangyuPct(v) => BuffEffect::WushiFangyuPct(v),
            BuffEffectDTO::ShanghaiPct(v) => BuffEffect::ShanghaiPct(v),
            BuffEffectDTO::JiasuPct(v) => BuffEffect::JiasuPct(v),
        }
    }
}

impl From<TimedBuffDefDTO> for TimedBuffDef {
    fn from(b: TimedBuffDefDTO) -> Self {
        TimedBuffDef {
            id: b.id,
            name: b.name,
            duration: b.duration,
            max_stacks: b.max_stacks,
            refresh: b.refresh,
            effects: b.effects.into_iter().map(Into::into).collect(),
            ignore_cap: b.ignore_cap,
        }
    }
}

impl From<SkillModifierDTO> for SkillModifier {
    fn from(m: SkillModifierDTO) -> Self {
        match m {
            SkillModifierDTO::BaseDamageAdd(v) => SkillModifier::BaseDamageAdd(v),
            SkillModifierDTO::BaseDamageMul(v) => SkillModifier::BaseDamageMul(v),
            SkillModifierDTO::AtkXishuAdd(v) => SkillModifier::AtkXishuAdd(v),
            SkillModifierDTO::HitUpAdd(v) => SkillModifier::HitUpAdd(v),
            SkillModifierDTO::HuixinUpAdd(v) => SkillModifier::HuixinUpAdd(v),
            SkillModifierDTO::HuixiaoUpAdd(v) => SkillModifier::HuixiaoUpAdd(v),
            SkillModifierDTO::WushifangyuAdd(v) => SkillModifier::WushifangyuAdd(v),
            SkillModifierDTO::CastTimeMul(v) => SkillModifier::CastTimeMul(v),
            SkillModifierDTO::CooldownMul(v) => SkillModifier::CooldownMul(v),
            SkillModifierDTO::DotDurationMul(v) => SkillModifier::DotDurationMul(v),
            SkillModifierDTO::DotIntervalMul(v) => SkillModifier::DotIntervalMul(v),
        }
    }
}

/// 技能池条目 → 领域技能（奇穴 replacement 等场景）
pub fn pool_item_to_skilltype(s: &SkillPoolItemDTO) -> Skilltype {
    super::calc::skill_dto_to_skilltype(s)
}

impl From<QixueDefDTO> for QixueDef {
    fn from(q: QixueDefDTO) -> Self {
        QixueDef {
            id: q.id,
            name: q.name,
            slot: q.slot,
            replaces_skill_id: q.replaces_skill_id,
            replaces_sub_id: q.replaces_sub_id,
            replacement: q.replacement.as_ref().map(pool_item_to_skilltype),
            modifiers: q.modifiers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<MijiDefDTO> for MijiDef {
    fn from(m: MijiDefDTO) -> Self {
        MijiDef {
            id: m.id,
            name: m.name,
            skill_id: m.skill_id,
            sub_id: m.sub_id,
            modifiers: m.modifiers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<CombatMetaDTO> for CombatMeta {
    fn from(m: CombatMetaDTO) -> Self {
        CombatMeta {
            profession: m.profession,
            version: m.version,
            note: m.note,
        }
    }
}

impl From<CombatDataDTO> for CombatData {
    fn from(d: CombatDataDTO) -> Self {
        CombatData {
            meta: CombatMeta {
                profession: d.meta.profession,
                version: d.meta.version,
                note: d.meta.note,
            },
            form: d.form.into_iter().map(Into::into).collect(),
            skill_combat: d.skill_combat.into_iter().map(Into::into).collect(),
            buff: d.buff.into_iter().map(Into::into).collect(),
            miji: d.miji.into_iter().map(Into::into).collect(),
            qixue: d.qixue.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combat_data_roundtrip_through_dto() {
        let data = CombatData {
            meta: CombatMeta {
                profession: "mowen".into(),
                version: "v1".into(),
                note: "n".into(),
            },
            form: vec![FormDef {
                id: "single".into(),
                name: "单持".into(),
                gcd_base: 1.5,
                initial: true,
                skills: vec![(1, 0)],
            }],
            skill_combat: vec![SkillCombatDef {
                skill_id: 1,
                sub_id: 0,
                form: Some("single".into()),
                gcd_lane: "single".into(),
                on_cast: OnCast {
                    switch_form: Some("dual".into()),
                    gcd_lanes: vec!["dual".into()],
                    lock_skills: vec![(2, 0)],
                },
                cast_time: Some(2.0),
                ..Default::default()
            }],
            buff: vec![TimedBuffDef {
                id: "b".into(),
                name: "buff".into(),
                duration: 10.0,
                max_stacks: 2,
                refresh: true,
                effects: vec![BuffEffect::JiasuPct(10.0)],
                ignore_cap: true,
            }],
            miji: vec![MijiDef {
                id: "m".into(),
                name: "秘籍".into(),
                skill_id: 1,
                sub_id: 0,
                modifiers: vec![SkillModifier::HitUpAdd(3)],
            }],
            qixue: Vec::new(),
        };
        let dto = CombatDataDTO::from(data.clone());
        assert_eq!(dto.form[0].id, "single");
        assert_eq!(dto.skill_combat[0].on_cast.lock_skills, vec![(2, 0)]);
        let back = CombatData::from(dto);
        // DTO 往返后与原始数据一致（逐字段比对关键项；CombatData 未实现 PartialEq）
        assert_eq!(back.form[0].gcd_base, 1.5);
        assert!(back.buff[0].ignore_cap);
        assert_eq!(back.miji[0].modifiers.len(), 1);
    }
}
