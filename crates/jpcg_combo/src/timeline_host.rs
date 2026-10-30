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

    let combat = req.combat.map(CombatData::from);
    let skills: Vec<TimingSkill> = req
        .skills
        .iter()
        .map(|dto| {
            let skill = conv::skill_dto_to_skilltype(dto);
            match &combat {
                Some(data) => TimingSkill::from_combat(&skill, data),
                None => TimingSkill {
                    skill,
                    form: None,
                    gcd_lane: String::new(),
                    on_cast: Default::default(),
                },
            }
        })
        .collect();
    let forms: Vec<FormDef> = req.forms.into_iter().map(FormDef::from).collect();
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
    let window = if req.window > 0.0 { req.window } else { 300.0 };

    let result = timeline::simulate(&TimelineInput {
        player,
        hostile,
        xinfa,
        buff,
        coeff,
        skills,
        forms,
        timed_buffs,
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
            } => TimelineEventKindDTO::Cast {
                skill_name,
                damage,
                hit_time,
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
            timed_buffs: Vec::new(),
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
    fn request_roundtrips_through_json() {
        let json = serde_json::to_string(&request()).unwrap();
        let back: TimelineRequestDTO = serde_json::from_str(&json).unwrap();
        let result = calculate_timeline(back).unwrap();
        assert!(result.total_damage > 0.0);
    }
}
