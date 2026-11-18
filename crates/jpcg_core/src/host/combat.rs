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
    QixueHookDTO, SkillCombatDTO, SkillModifierDTO, SkillPoolItemDTO, TimedBuffDefDTO,
};

use crate::host::conv::skilltype_to_pool_item;
use crate::store;
use crate::type_set::buff::{BuffEffect, TimedBuffDef};
use crate::type_set::combat::{CombatData, CombatMeta, FormDef, OnCast, SkillCombatDef};
use crate::type_set::modifier::{MijiDef, QixueDef, QixueHook, SkillModifier};
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
    let dto = enrich_combat_dto(&skills, CombatDataDTO::from(data))?;
    Ok((skills.iter().map(skilltype_to_pool_item).collect(), dto))
}

/// 面板整理（全部在后端完成，前端只渲染）：
/// - 秘籍 → 技能按 (skill_id, sub_id) 精确配对，sub_id=0 仅在唯一父技能时展开，歧义/缺失直接报错；
/// - 奇穴按槽位生成显示分组标签，同名多效果（冻泉/冻泉·宫）合并为一格；
/// - 每条奇穴给 group_ids（点击时一起开关）。
fn enrich_combat_dto(
    skills: &[Skilltype],
    mut dto: CombatDataDTO,
) -> Result<CombatDataDTO, String> {
    use std::collections::BTreeMap;

    // ---- 秘籍配对 ----
    let mut parents: std::collections::HashMap<u32, Vec<(u32, u32)>> =
        std::collections::HashMap::new();
    for s in skills {
        parents
            .entry(s.skill_id)
            .or_default()
            .push((s.skill_id, s.sub_id));
    }
    for m in &dto.miji {
        let list = parents.get(&m.skill_id).cloned().unwrap_or_default();
        if list.is_empty() {
            return Err(format!(
                "秘籍 `{}` 引用的技能 {} 不在技能表中",
                m.id, m.skill_id
            ));
        }
        if m.sub_id == 0 && list.len() > 1 {
            return Err(format!(
                "秘籍 `{}` 使用 sub_id=0，但技能 {} 有 {} 个变体，无法确定挂载目标",
                m.id,
                m.skill_id,
                list.len()
            ));
        }
    }
    for sc in &mut dto.skill_combat {
        sc.miji_ids = dto
            .miji
            .iter()
            .filter(|m| {
                let core: MijiDef = (*m).clone().into();
                core.targets_skill(sc.skill_id, sc.sub_id)
            })
            .map(|m| m.id.clone())
            .collect();
    }

    // ---- 奇穴分组 + 同名合并 ----
    let slot_label = |slot: u8| -> String {
        const NAMES: [&str; 6] = ["一", "二", "三", "四", "五", "六"];
        if slot >= 1 && slot <= 6 {
            format!("第{}重", NAMES[slot as usize - 1])
        } else {
            "混选".to_string()
        }
    };
    let original = std::mem::take(&mut dto.qixue);
    let mut merged: Vec<jpcg_api::QixueDefDTO> = Vec::new();
    let mut index: BTreeMap<(u8, String), usize> = BTreeMap::new();
    for q in original {
        let base = q.name.split('·').next().unwrap_or(&q.name).to_string();
        let key = (q.slot, base.clone());
        if let Some(&i) = index.get(&key) {
            merged[i].group_ids.push(q.id);
            continue;
        }
        let mut q2 = q;
        q2.name = base;
        q2.group_label = slot_label(q2.slot);
        q2.group_ids = vec![q2.id.clone()];
        index.insert(key, merged.len());
        merged.push(q2);
    }
    dto.qixue = merged;
    // 门控技能：把 jihuoqixue（可能是名称或 id）解析成 qixue id，前端只做 id 匹配
    for sc in &mut dto.skill_combat {
        if let Some(gate) = &sc.jihuoqixue {
            let found = dto
                .qixue
                .iter()
                .find(|q| q.name == *gate || q.id == *gate)
                .map(|q| q.id.clone());
            sc.jihuoqixue_id = found;
        }
    }
    Ok(dto)
}

// ============ core → DTO ============

impl From<CombatMeta> for CombatMetaDTO {
    fn from(m: CombatMeta) -> Self {
        CombatMetaDTO {
            profession: m.profession,
            version: m.version,
            note: m.note,
            miji_per_skill_cap: jpcg_api::default_miji_cap(),
            mixed_qixue_cap: jpcg_api::default_mixed_cap(),
            mixed_slot_from: jpcg_api::default_mixed_slot_from(),
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
            switch_instrument: o.switch_instrument,
            switch_qifeng: o.switch_qifeng,
            gcd_lanes: o.gcd_lanes,
            lock_skills: o.lock_skills,
            advance_dot_targets: o.advance_dot_targets,
            advance_dot_ticks: o.advance_dot_ticks,
        }
    }
}

impl From<SkillCombatDef> for SkillCombatDTO {
    fn from(s: SkillCombatDef) -> Self {
        SkillCombatDTO {
            skill_id: s.skill_id,
            sub_id: s.sub_id,
            requires_instrument: s.requires_instrument,
            requires_qifeng: s.requires_qifeng,
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
            channel_ticks: s.channel_ticks,
            buff_id: s.buff_id,
            jihuoqixue: s.jihuoqixue,
            jihuoqixue_id: None,
            qifeng_gain: s.qifeng_gain,
            spend_all_qifeng: s.spend_all_qifeng,
            miji_ids: Vec::new(),
            dot_interval: s.dot_interval,
            dot_duration: s.dot_duration,
            dot_tick_base1: s.dot_tick_base1,
            dot_tick_base2: s.dot_tick_base2,
            dot_tick_xishu: s.dot_tick_xishu,
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
            SkillModifier::CastTimeAdd(v) => SkillModifierDTO::CastTimeAdd(v),
            SkillModifier::CooldownMul(v) => SkillModifierDTO::CooldownMul(v),
            SkillModifier::DotDurationMul(v) => SkillModifierDTO::DotDurationMul(v),
            SkillModifier::DotIntervalMul(v) => SkillModifierDTO::DotIntervalMul(v),
            SkillModifier::CooldownAdd(v) => SkillModifierDTO::CooldownAdd(v),
            SkillModifier::DotTicksAdd(v) => SkillModifierDTO::DotTicksAdd(v),
            SkillModifier::DotUpAdd(v) => SkillModifierDTO::DotUpAdd(v),
        }
    }
}

/// 修正项 → 简短效果文案（后端统一生成，前端不翻译）
fn modifier_effect(m: &SkillModifier) -> String {
    let pct = |v: f32| format!("{}%", (v * 10.0).round() / 10.0);
    match *m {
        SkillModifier::BaseDamageAdd(v) => format!("基础伤害+{v}"),
        SkillModifier::BaseDamageMul(v) => format!("基础伤害×{v}"),
        SkillModifier::AtkXishuAdd(v) => format!("系数+{v}"),
        SkillModifier::HitUpAdd(v) => format!("伤害+{v}%"),
        SkillModifier::HuixinUpAdd(v) => format!("会心+{v}%"),
        SkillModifier::HuixiaoUpAdd(v) => format!("会效+{v}%"),
        SkillModifier::WushifangyuAdd(v) => {
            format!("无视{}%防御", ((v as f32 / 1024.0) * 100.0).round())
        }
        SkillModifier::CastTimeMul(v) => format!("运功×{v}"),
        SkillModifier::CastTimeAdd(v) if v < 0.0 => format!("运功{v}秒"),
        SkillModifier::CastTimeAdd(v) => format!("运功+{v}秒"),
        SkillModifier::CooldownMul(v) if v < 0.0 => format!("调息-{}秒", -v),
        SkillModifier::CooldownMul(v) => format!("调息+{v}秒"),
        SkillModifier::DotDurationMul(v) => format!("持续×{v}"),
        SkillModifier::DotIntervalMul(v) => format!("间隔{}%", pct(v)),
        SkillModifier::CooldownAdd(v) if v < 0.0 => format!("调息-{}秒", -v),
        SkillModifier::CooldownAdd(v) => format!("调息+{v}秒"),
        SkillModifier::DotTicksAdd(v) => format!("跳数{:+}", v),
        SkillModifier::DotUpAdd(v) => format!("每跳+{}%", (v * 100.0).round()),
    }
}

impl From<MijiDef> for MijiDefDTO {
    fn from(m: MijiDef) -> Self {
        let effects = m.modifiers.iter().map(modifier_effect).collect();
        MijiDefDTO {
            id: m.id,
            name: m.name,
            skill_id: m.skill_id,
            sub_id: m.sub_id,
            applies_to: m.applies_to,
            modifiers: m.modifiers.into_iter().map(Into::into).collect(),
            effects,
        }
    }
}

impl From<CombatData> for CombatDataDTO {
    fn from(d: CombatData) -> Self {
        CombatDataDTO {
            meta: d.meta.into(),
            form: d.form.into_iter().map(Into::into).collect(),
            qifeng: d.qifeng,
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
        applies_to: q.applies_to,
        grants_skills: q.grants_skills,
        hooks: q.hooks.iter().map(hook_to_dto).collect(),
        shelved: q.shelved,
        replacement: q.replacement.as_ref().map(skilltype_to_pool_item),
        modifiers: q.modifiers.into_iter().map(Into::into).collect(),
        group_label: String::new(),
        group_ids: Vec::new(),
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
            switch_instrument: o.switch_instrument,
            switch_qifeng: o.switch_qifeng,
            gcd_lanes: o.gcd_lanes,
            lock_skills: o.lock_skills,
            advance_dot_targets: o.advance_dot_targets,
            advance_dot_ticks: o.advance_dot_ticks,
        }
    }
}

impl From<SkillCombatDTO> for SkillCombatDef {
    fn from(s: SkillCombatDTO) -> Self {
        SkillCombatDef {
            skill_id: s.skill_id,
            sub_id: s.sub_id,
            requires_instrument: s.requires_instrument,
            requires_qifeng: s.requires_qifeng,
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
            channel_ticks: s.channel_ticks,
            buff_id: s.buff_id,
            jihuoqixue: s.jihuoqixue,
            qifeng_gain: s.qifeng_gain,
            spend_all_qifeng: s.spend_all_qifeng,
            dot_interval: s.dot_interval,
            dot_duration: s.dot_duration,
            dot_tick_base1: s.dot_tick_base1,
            dot_tick_base2: s.dot_tick_base2,
            dot_tick_xishu: s.dot_tick_xishu,
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
            SkillModifierDTO::CastTimeAdd(v) => SkillModifier::CastTimeAdd(v),
            SkillModifierDTO::CooldownMul(v) => SkillModifier::CooldownMul(v),
            SkillModifierDTO::DotDurationMul(v) => SkillModifier::DotDurationMul(v),
            SkillModifierDTO::DotIntervalMul(v) => SkillModifier::DotIntervalMul(v),
            SkillModifierDTO::CooldownAdd(v) => SkillModifier::CooldownAdd(v),
            SkillModifierDTO::DotTicksAdd(v) => SkillModifier::DotTicksAdd(v),
            SkillModifierDTO::DotUpAdd(v) => SkillModifier::DotUpAdd(v),
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
            applies_to: q.applies_to,
            grants_skills: q.grants_skills,
            hooks: q.hooks.into_iter().map(QixueHook::from).collect(),
            shelved: q.shelved,
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
            applies_to: m.applies_to,
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
            qifeng: d.qifeng,
            skill_combat: d.skill_combat.into_iter().map(Into::into).collect(),
            buff: d.buff.into_iter().map(Into::into).collect(),
            miji: d.miji.into_iter().map(Into::into).collect(),
            qixue: d.qixue.into_iter().map(Into::into).collect(),
        }
    }
}

fn hook_to_dto(h: &QixueHook) -> QixueHookDTO {
    match *h {
        QixueHook::FlatDamageOnCast {
            damage,
            per_qifeng_spent,
        } => QixueHookDTO::FlatDamageOnCast {
            damage,
            per_qifeng_spent,
        },
        QixueHook::DamagePctPerQifeng { pct } => QixueHookDTO::DamagePctPerQifeng { pct },
        QixueHook::AccelerateDot { ref targets } => QixueHookDTO::AccelerateDot {
            targets: targets.clone(),
        },
    }
}

impl From<QixueHookDTO> for QixueHook {
    fn from(h: QixueHookDTO) -> Self {
        match h {
            QixueHookDTO::FlatDamageOnCast {
                damage,
                per_qifeng_spent,
            } => QixueHook::FlatDamageOnCast {
                damage,
                per_qifeng_spent,
            },
            QixueHookDTO::DamagePctPerQifeng { pct } => QixueHook::DamagePctPerQifeng { pct },
            QixueHookDTO::AccelerateDot { targets } => QixueHook::AccelerateDot { targets },
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
            qifeng: vec!["yangchun".into(), "gaoshan".into()],
            skill_combat: vec![SkillCombatDef {
                skill_id: 1,
                sub_id: 0,
                requires_instrument: Some("single".into()),
                requires_qifeng: None,
                gcd_lane: "single".into(),
                on_cast: OnCast {
                    switch_instrument: Some("dual".into()),
                    switch_qifeng: None,
                    gcd_lanes: vec!["dual".into()],
                    lock_skills: vec![(2, 0)],
                    advance_dot_targets: Vec::new(),
                    advance_dot_ticks: 0,
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
                applies_to: Vec::new(),
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
