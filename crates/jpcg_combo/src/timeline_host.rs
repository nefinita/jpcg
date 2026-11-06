// ============================================================================
// timeline_host — 时序计算宿主入口（DTO ↔ timeline 引擎）
//
// 对外：
//   - load_combat(profession)                 读 `data/combat/{门派}.toml`
//   - load_combat_with_skills(profession)     技能池（已套 overlay）+ combat
//   - calculate_timeline(TimelineRequestDTO)  DTO 请求 → 时间轴模拟结果 DTO
// ============================================================================

use jpcg_api::{
    CombatDataDTO, TimelineActionDTO, TimelineEventDTO, TimelineEventKindDTO, TimelineRequestDTO,
    TimelineResultDTO,
};
use jpcg_core::type_set::buff::TimedBuffDef;
use jpcg_core::type_set::combat::{CombatData, FormDef};

use crate::conv;
use crate::timeline::{
    self, TimelineAction, TimelineEvent, TimelineEventKind, TimelineInput, TimingSkill,
};

/// 读取某门派的战斗时序 overlay（文件缺失 = 空数据）
pub fn load_combat(profession: String) -> Result<CombatDataDTO, String> {
    jpcg_core::host::combat::load_combat_data(profession)
}

/// 技能池（已套 overlay）+ 战斗时序数据（前端时间轴一次拿全）
pub fn load_combat_bundle(profession: String) -> Result<jpcg_api::CombatBundleDTO, String> {
    let (skills, combat) = jpcg_core::host::combat::load_combat_with_skills(profession)?;
    Ok(jpcg_api::CombatBundleDTO { skills, combat })
}

/// 时间轴模拟（DTO 请求 → 结果 DTO）
pub fn calculate_timeline(req: TimelineRequestDTO) -> Result<TimelineResultDTO, String> {
    let (player, hostile, xinfa, buff, coeff) = super::host::into_core(
        req.player,
        req.hostile,
        req.xinfa,
        req.buff,
        req.coeff,
        req.value_set,
    );

    // overlay 提供的形态定义可以作为默认值；调用方显式给了 forms 则以调用方为准
    let default_forms: Vec<jpcg_api::FormDefDTO> = req
        .combat
        .as_ref()
        .map(|c| c.form.clone())
        .unwrap_or_default();
    let combat = req.combat.map(CombatData::from);
    // 激活的奇穴：先按 id 取出定义，后面逐个技能套 modifiers
    let active_qixue: Vec<jpcg_core::type_set::modifier::QixueDef> = combat
        .as_ref()
        .map(|data| {
            data.qixue
                .iter()
                .filter(|q| req.qixue.iter().any(|id| id == &q.id))
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let qixue_refs: Vec<&jpcg_core::type_set::modifier::QixueDef> = active_qixue.iter().collect();
    // 已激活秘籍：按 id 取定义，逐个技能套修正项
    let active_miji: Vec<jpcg_core::type_set::modifier::MijiDef> = combat
        .as_ref()
        .map(|data| {
            data.miji
                .iter()
                .filter(|m| req.miji.iter().any(|id| id == &m.id))
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let miji_refs: Vec<&jpcg_core::type_set::modifier::MijiDef> = active_miji.iter().collect();
    let skills: Vec<TimingSkill> = req
        .skills
        .iter()
        .map(|dto| {
            let skill = conv::skill_dto_to_skilltype(dto);
            // 先套 overlay 时序字段（cast_time/cooldown 等以 combat 为准），
            // 再套秘籍/奇穴修正项，避免定值覆盖掉秘籍的读条/调息修正。
            let mut ts = match &combat {
                Some(data) => TimingSkill::from_combat(&skill, data),
                None => TimingSkill {
                    skill,
                    qifeng_gain: 0,
                    spend_all_qifeng: false,
                    requires_instrument: None,
                    requires_qifeng: None,
                    gcd_lane: String::new(),
                    on_cast: Default::default(),
                },
            };
            ts.skill =
                jpcg_core::type_set::modifier::resolve_skill(&ts.skill, &miji_refs, &qixue_refs);
            ts
        })
        .collect();
    let forms: Vec<FormDef> = if req.forms.is_empty() {
        default_forms.into_iter().map(FormDef::from).collect()
    } else {
        req.forms.into_iter().map(FormDef::from).collect()
    };
    let timed_buffs: Vec<TimedBuffDef> = req
        .timed_buffs
        .into_iter()
        .map(TimedBuffDef::from)
        .collect();
    let actions: Vec<TimelineAction> = req
        .actions
        .into_iter()
        .map(|a| match a {
            TimelineActionDTO::Cast {
                skill_id,
                sub_id,
                skill_name,
            } => match skill_name {
                Some(name) => TimelineAction::cast_named(skill_id, sub_id, name),
                None => TimelineAction::cast(skill_id, sub_id),
            },
            TimelineActionDTO::SwitchForm { form } => TimelineAction::SwitchForm(form),
        })
        .collect();
    // 门控用的激活奇穴集合：请求里的 id + 对应奇穴名（技能 `jihuoqixue` 可能写 id 或名称）
    let mut combat_qixue_names: Vec<String> = req.qixue.clone();
    if let Some(data) = combat.as_ref() {
        for q in data.qixue.iter() {
            if req.qixue.iter().any(|id| id == &q.id) {
                combat_qixue_names.push(q.name.clone());
            }
        }
    }
    // 第 3 层钩子：把激活奇穴的 hooks 转成带作用技能的 ActiveHook
    let hooks: Vec<timeline::ActiveHook> = active_qixue
        .iter()
        .flat_map(|q| {
            let targets: Vec<timeline::SkillKey> = if !q.applies_to.is_empty() {
                q.applies_to.clone()
            } else if q.replaces_skill_id != 0 {
                vec![(q.replaces_skill_id, q.replaces_sub_id)]
            } else {
                Vec::new()
            };
            q.hooks
                .iter()
                .cloned()
                .map(move |hook| timeline::ActiveHook {
                    targets: targets.clone(),
                    hook,
                })
        })
        .collect();
    let window = if req.window > 0.0 { req.window } else { 300.0 };

    let result = timeline::simulate(&TimelineInput {
        player,
        hostile,
        xinfa,
        buff,
        coeff,
        skills,
        forms,
        initial_instrument: req.initial_instrument,
        initial_qifeng: req.initial_qifeng,
        timed_buffs,
        active_qixue: combat_qixue_names,
        hooks,
        actions,
        window,
    });

    Ok(TimelineResultDTO::from(result))
}

impl From<timeline::TimelineResult> for TimelineResultDTO {
    fn from(r: timeline::TimelineResult) -> Self {
        TimelineResultDTO {
            events: r.events.into_iter().map(event_to_dto).collect(),
            total_damage: r.total_damage,
            window_dps: r.window_dps,
            skipped_actions: r.skipped_actions,
        }
    }
}

fn event_to_dto(e: TimelineEvent) -> TimelineEventDTO {
    TimelineEventDTO {
        time: e.time,
        kind: match e.kind {
            TimelineEventKind::Cast {
                skill_name,
                damage,
                hit_time,
                gcd_secs,
                gcd_ready_at,
            } => TimelineEventKindDTO::Cast {
                skill_name,
                damage,
                hit_time,
                gcd_secs,
                gcd_ready_at,
            },
            TimelineEventKind::DotTick {
                skill_name,
                index,
                damage,
            } => TimelineEventKindDTO::DotTick {
                skill_name,
                index,
                damage,
            },
            TimelineEventKind::QifengSwitch { from, to } => {
                TimelineEventKindDTO::QifengSwitch { from, to }
            }
            TimelineEventKind::FormSwitch { from, to } => {
                TimelineEventKindDTO::FormSwitch { from, to }
            }
            TimelineEventKind::BuffGain { buff_id, stacks } => {
                TimelineEventKindDTO::BuffGain { buff_id, stacks }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jpcg_api::{
        BuffConfigDTO, CoefficientConfigDTO, HostileConfigDTO, PlayerConfigDTO, SkillPoolItemDTO,
        XinfaConfigDTO,
    };

    fn request() -> TimelineRequestDTO {
        TimelineRequestDTO {
            skills: vec![SkillPoolItemDTO {
                skill_name: "测试技能".into(),
                skill_id: 1,
                base_damage1: 1000,
                base_damage2: 1000,
                atk_xishu: 1.0,
                cast_time: 1.5,
                gcd_kind: 0,
                ..Default::default()
            }],
            forms: Vec::new(),
            initial_instrument: None,
            initial_qifeng: String::new(),
            timed_buffs: Vec::new(),
            qixue: Vec::new(),
            miji: Vec::new(),
            actions: vec![
                TimelineActionDTO::Cast {
                    skill_id: 1,
                    sub_id: 0,
                    skill_name: None,
                },
                TimelineActionDTO::Cast {
                    skill_id: 1,
                    sub_id: 0,
                    skill_name: None,
                },
            ],
            window: 10.0,
            player: PlayerConfigDTO {
                jcsx: "gengu".into(),
                jichu_shuxing: 21371,
                jichu_gongji: 64329,
                huixin_dengji: 61877,
                huixin_xiaoguo: 2925,
                pofang_dengji: 109160,
                ..Default::default()
            },
            hostile: HostileConfigDTO::default(),
            xinfa: XinfaConfigDTO {
                profession: "mowen".into(),
                gcd_base: 1.5,
                ..Default::default()
            },
            buff: BuffConfigDTO::default(),
            coeff: CoefficientConfigDTO {
                pvp_global_jianshang: 0.9,
                ..Default::default()
            },
            value_set: None,
            combat: None,
        }
    }

    #[test]
    fn timeline_request_through_dto_returns_events() {
        let result = calculate_timeline(request()).expect("模拟应成功");
        assert_eq!(result.skipped_actions, 0);
        assert!(result.total_damage > 0.0);
        assert!(
            result
                .events
                .iter()
                .any(|e| matches!(e.kind, TimelineEventKindDTO::Cast { .. }))
        );
        // 两个 GCD 技能：第二发应落在 1.5s 之后
        let casts: Vec<f32> = result
            .events
            .iter()
            .filter(|e| matches!(e.kind, TimelineEventKindDTO::Cast { .. }))
            .map(|e| e.time)
            .collect();
        assert_eq!(casts.len(), 2);
        assert!((casts[0] - 0.0).abs() < 1e-4, "起手应在 0s: {:?}", casts);
        assert!(
            (casts[1] - 1.5).abs() < 1e-4,
            "第二发应在 1.5s: {:?}",
            casts
        );
    }

    #[test]
    fn converted_attack_reaches_timeline_damage_without_second_conversion() {
        let mut converted = request();
        converted.player.jichu_gongji = 1001;
        converted.player.jichu_shuxing = 100;
        converted.player.zhuanhua_gongji = Some(251);
        converted.xinfa.atk_up = 99.0;
        converted.buff.base_atk_pct = 12.5;
        let mut manual = request();
        // 手算：floor(1001 * 1.125) + 251 = 1377。
        manual.player.jichu_gongji = 1377;
        manual.player.jichu_shuxing = 0;
        let converted = calculate_timeline(converted).unwrap();
        let manual = calculate_timeline(manual).unwrap();
        assert!(converted.total_damage > 0.0);
        assert_eq!(converted.total_damage, manual.total_damage);
    }

    #[test]
    fn request_roundtrips_through_json() {
        let json = serde_json::to_string(&request()).unwrap();
        let back: TimelineRequestDTO = serde_json::from_str(&json).unwrap();
        let result = calculate_timeline(back).unwrap();
        assert!(result.total_damage > 0.0);
    }
}
