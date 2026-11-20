// ============================================================================
// timeline_host — 时序计算宿主入口（DTO ↔ timeline 引擎）
//
// 对外：
//   - load_combat(profession)                 读 `data/{门派}/combat.toml`
//   - load_combat_with_skills(profession)     技能池（已套 overlay）+ combat
//   - calculate_timeline(TimelineRequestDTO)  DTO 请求 → 时间轴模拟结果 DTO
// ============================================================================

use jpcg_api::{
    CombatDataDTO, TimelineActionDTO, TimelineEventDTO, TimelineEventKindDTO, TimelineRequestDTO,
    TimelineResultDTO, TimelineStepDTO, TimelineStepSwitchDTO,
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

/// 桌面/FFI 入口：展示 DTO 不能作为计算定义，使用本机完整数据并校验选择。
pub fn calculate_timeline_from_store(req: TimelineRequestDTO) -> Result<TimelineResultDTO, String> {
    calculate_timeline(prepare_timeline_from_store(req)?)
}

/// 搜索开始时冻结完整定义，候选评估只替换玩家属性，不反复读取展示 DTO。
pub fn prepare_timeline_from_store(
    mut req: TimelineRequestDTO,
) -> Result<TimelineRequestDTO, String> {
    let profession = req.xinfa.profession.clone();
    let bundle = load_combat_bundle(profession.clone())?;
    validate_selection(&bundle.combat, &req.qixue, &req.miji)?;
    validate_flags(&req.actions)?;
    // 同一格的多个效果必须一起启用，外部调用者不能只传半组。
    req.qixue = bundle
        .combat
        .qixue
        .iter()
        .filter(|q| {
            req.qixue.contains(&q.id) || q.group_ids.iter().any(|id| req.qixue.contains(id))
        })
        .flat_map(|q| {
            if q.group_ids.is_empty() {
                vec![q.id.clone()]
            } else {
                q.group_ids.clone()
            }
        })
        .collect();
    req.skills = bundle.skills;
    let combat = load_combat(profession)?;
    req.forms = combat.form.clone();
    req.timed_buffs = combat.buff.clone();
    req.combat = Some(combat);
    Ok(req)
}

/// 计算开始/结束 flag 各最多一个，且开始必须早于结束。
fn validate_flags(actions: &[TimelineActionDTO]) -> Result<(), String> {
    let mut start: Option<usize> = None;
    let mut end: Option<usize> = None;
    for (i, a) in actions.iter().enumerate() {
        if let TimelineActionDTO::Flag { flag } = a {
            match flag.as_str() {
                "start" => {
                    if start.is_some() {
                        return Err("计算开始 flag 只能有一个".into());
                    }
                    start = Some(i);
                }
                "end" => {
                    if end.is_some() {
                        return Err("计算结束 flag 只能有一个".into());
                    }
                    end = Some(i);
                }
                other => return Err(format!("未知 flag：{other}")),
            }
        }
    }
    if let (Some(s), Some(e)) = (start, end)
        && s >= e
    {
        return Err("计算开始 flag 必须早于计算结束 flag".into());
    }
    Ok(())
}

/// 按后端生成的格子计数；同格多个效果 id 不重复占槽。
fn validate_selection(
    data: &CombatDataDTO,
    qixue: &[String],
    miji: &[String],
) -> Result<(), String> {
    use std::collections::{HashMap, HashSet};
    let mut selected = HashSet::new();
    for id in qixue {
        let index = data
            .qixue
            .iter()
            .position(|q| q.id == *id || q.group_ids.contains(id))
            .ok_or_else(|| format!("未知奇穴：{id}"))?;
        if data.qixue[index].shelved {
            return Err(format!("奇穴暂未建模：{id}"));
        }
        selected.insert(index);
    }
    let mut slots = HashSet::new();
    let mut mixed = 0;
    for index in selected {
        let q = &data.qixue[index];
        if q.slot >= data.meta.mixed_slot_from {
            mixed += 1;
        } else if !slots.insert(q.slot) {
            return Err(format!("第 {} 重只能选择一个奇穴", q.slot));
        }
    }
    if mixed > data.meta.mixed_qixue_cap {
        return Err("混选奇穴超过上限".into());
    }
    let mut counts = HashMap::new();
    for id in miji.iter().collect::<HashSet<_>>() {
        let m = data
            .miji
            .iter()
            .find(|m| m.id == *id)
            .ok_or_else(|| format!("未知秘籍：{id}"))?;
        let count = counts.entry((m.skill_id, m.sub_id)).or_insert(0usize);
        *count += 1;
        if *count > usize::from(data.meta.miji_per_skill_cap) {
            return Err(format!("技能 {} 的秘籍超过上限", m.skill_id));
        }
    }
    Ok(())
}

/// 组装时间轴输入（DTO → 引擎输入），供模拟与批量 MC 共用。
pub fn build_timeline_input(req: TimelineRequestDTO) -> Result<timeline::TimelineInput, String> {
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
                    dot_tick: None,
                    gcd_lane: String::new(),
                    on_cast: Default::default(),
                },
            };
            ts.skill =
                jpcg_core::type_set::modifier::resolve_skill(&ts.skill, &miji_refs, &qixue_refs);
            // 独立 DOT 也要吃秘籍/奇穴修正（殊曲缩短间隔、冻泉加跳数等），
            // 但 DOT 的每跳技能是合成条目，需要单独 resolve 后回写时长/间隔。
            if let Some(spec) = &mut ts.dot_tick {
                let resolved = jpcg_core::type_set::modifier::resolve_skill(
                    &spec.skill,
                    &miji_refs,
                    &qixue_refs,
                );
                spec.interval = resolved.dot_interval;
                spec.duration = resolved.dot_duration;
                spec.skill = resolved;
            }
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
                overrides,
            } => TimelineAction::Cast {
                skill_id,
                sub_id,
                skill_name,
                overrides,
            },
            TimelineActionDTO::SwitchForm { form } => TimelineAction::SwitchForm(form),
            TimelineActionDTO::Flag { flag } => TimelineAction::Flag {
                start: flag == "start",
            },
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

    Ok(TimelineInput {
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
        samples: req.samples,
        seed: req.seed,
    })
}

/// 时间轴模拟（DTO 请求 → 结果 DTO）
pub fn calculate_timeline(req: TimelineRequestDTO) -> Result<TimelineResultDTO, String> {
    calculate_timeline_with_cooldowns(req, &[])
}

/// 装备 CD 调整在 overlay/秘籍/奇穴之后应用，供基线和批量搜索共用。
pub fn calculate_timeline_with_cooldowns(
    req: TimelineRequestDTO,
    adjustments: &[(u32, f32)],
) -> Result<TimelineResultDTO, String> {
    let mut input = build_timeline_input(req)?;
    apply_cooldowns(&mut input, adjustments)?;
    Ok(TimelineResultDTO::from(timeline::simulate(&input)))
}

fn apply_cooldowns(
    input: &mut timeline::TimelineInput,
    adjustments: &[(u32, f32)],
) -> Result<(), String> {
    for &(skill_id, seconds) in adjustments {
        if !seconds.is_finite() || seconds < 0.0 {
            return Err("装备CD调整必须为有限非负秒数".into());
        }
        for skill in &mut input.skills {
            if skill.skill.skill_id == skill_id {
                skill.skill.cooldown = (skill.skill.cooldown - seconds).max(0.0);
                if skill.skill.recharge > 0.0 {
                    skill.skill.recharge = (skill.skill.recharge - seconds).max(0.0);
                }
            }
        }
    }
    Ok(())
}

/// 批量 MC 输入：窗口内伤害事件（按结算时刻排序，与引擎内 MC 同序）+ 初始血量 + 确定性总伤。
pub struct TimelineMcInput {
    pub events: Vec<crate::kill_mc::McEvent>,
    pub hp0: f64,
    pub total_damage: f64,
}

/// 与玩家无关的时间轴模板：构建一次，之后只替换 player 复用（批量 MC 用）。
#[derive(Clone)]
pub struct TimelineTemplate {
    input: timeline::TimelineInput,
    cooldown_bases: Vec<(f32, f32)>,
}

impl TimelineTemplate {
    pub fn new(req: TimelineRequestDTO) -> Result<Self, String> {
        let mut input = build_timeline_input(req)?;
        // 批量路径只算确定性部分（伤害骰/总伤），MC 由 CPU/wgpu 后端统一跑，避免重复 50k 采样。
        input.samples = 0;
        input.seed = None;
        let cooldown_bases = input
            .skills
            .iter()
            .map(|s| (s.skill.cooldown, s.skill.recharge))
            .collect();
        Ok(Self {
            input,
            cooldown_bases,
        })
    }

    /// 替换候选玩家（配装合成出的 player 面板）。
    pub fn set_player(&mut self, player: jpcg_api::PlayerConfigDTO) {
        self.input.player = jpcg_core::type_set::player::PlayerConfig::from(player);
    }

    /// 每个候选先恢复模板 CD，防止上一个候选的套装效果泄漏。
    pub fn set_loadout(
        &mut self,
        player: jpcg_api::PlayerConfigDTO,
        adjustments: &[(u32, f32)],
    ) -> Result<(), String> {
        self.set_player(player);
        for (skill, &(cooldown, recharge)) in self.input.skills.iter_mut().zip(&self.cooldown_bases)
        {
            skill.skill.cooldown = cooldown;
            skill.skill.recharge = recharge;
        }
        apply_cooldowns(&mut self.input, adjustments)
    }

    /// 用当前玩家跑确定性 MC 输入（窗口内 McEvent + hp0 + 总伤）。
    pub fn mc_input(&self) -> Result<TimelineMcInput, String> {
        mc_input_from_input(&self.input)
    }
}

/// 时间轴 → 批量 MC 输入（供 CPU/wgpu 后端跑精确 50k MC）。
pub fn timeline_mc_input(req: TimelineRequestDTO) -> Result<TimelineMcInput, String> {
    mc_input_from_input(&build_timeline_input(req)?)
}

fn mc_input_from_input(input: &timeline::TimelineInput) -> Result<TimelineMcInput, String> {
    let hp0 = if input.hostile.max_hp > 0 {
        input.hostile.max_hp as f64
    } else {
        input.hostile.target_hp as f64
    };
    let result = timeline::simulate(input);
    let mut timed: Vec<(f32, crate::kill_mc::McEvent)> = Vec::new();
    for e in &result.visible_events {
        let (time, dice) = match &e.kind {
            TimelineEventKind::Cast { hit_time, dice, .. } => (*hit_time, *dice),
            TimelineEventKind::ChannelTick { dice, .. } => (e.time, *dice),
            TimelineEventKind::DotTick { dice, .. } => (e.time, *dice),
            _ => continue,
        };
        let (g, h, rate) = if dice.fixed {
            (dice.q, dice.q, 1.0_f32)
        } else {
            (dice.g, dice.h, dice.rate)
        };
        timed.push((
            time,
            crate::kill_mc::McEvent {
                g,
                h,
                rate,
                lost: dice.lost_hp_factor,
            },
        ));
    }
    timed.sort_by(|a, b| a.0.total_cmp(&b.0));
    Ok(TimelineMcInput {
        events: timed.into_iter().map(|(_, ev)| ev).collect(),
        hp0,
        total_damage: result.total_damage,
    })
}

impl From<timeline::TimelineResult> for TimelineResultDTO {
    fn from(r: timeline::TimelineResult) -> Self {
        // 伤害日志按实际命中排序，起手另存 start_time；只展示测量窗口内的事件。
        let settlement = |e: &TimelineEvent| match e.kind {
            TimelineEventKind::Cast { hit_time, .. } => hit_time,
            _ => e.time,
        };
        let mut events = r.visible_events;
        events.sort_by(|a, b| settlement(a).total_cmp(&settlement(b)));
        let mut cumulative = 0.0;
        let events = events
            .into_iter()
            .map(|e| {
                cumulative += event_damage(&e.kind);
                let time = settlement(&e);
                let mut dto = event_to_dto(e);
                dto.time = time;
                dto.cumulative = cumulative;
                dto
            })
            .collect();
        let steps = r
            .steps
            .into_iter()
            .map(|step| TimelineStepDTO {
                time: step.time,
                gcd_secs: step.gcd_secs,
                gcd_ready_at: step.gcd_ready_at,
                legal: step.legal,
                instrument: step.instrument,
                qifeng: step.qifeng,
                skill_name: step.skill_name,
                skipped_reason: step.skipped_reason,
                required_instrument: step.required_instrument,
                required_qifeng: step.required_qifeng,
                switches: step
                    .switches
                    .into_iter()
                    .map(|sw| TimelineStepSwitchDTO {
                        switch_kind: sw.switch_kind,
                        from: sw.from,
                        to: sw.to,
                        time: sw.time,
                    })
                    .collect(),
            })
            .collect();
        TimelineResultDTO {
            events,
            steps,
            total_damage: r.total_damage,
            damage_variance: r.damage_variance,
            window_dps: r.window_dps,
            skipped_actions: r.skipped_actions,
            final_instrument: r.final_instrument,
            final_qifeng: r.final_qifeng,
            final_kill_prob: r.final_kill_prob,
            kill_prob_curve: r.kill_prob_curve,
            kill_start: r.kill_start,
            kill_end: r.kill_end,
        }
    }
}

fn event_damage(kind: &TimelineEventKind) -> f64 {
    match kind {
        TimelineEventKind::Cast { damage, .. } => *damage as f64,
        TimelineEventKind::ChannelTick { damage, .. } => *damage as f64,
        TimelineEventKind::DotTick { damage, .. } => *damage as f64,
        _ => 0.0,
    }
}

fn event_to_dto(e: TimelineEvent) -> TimelineEventDTO {
    TimelineEventDTO {
        time: e.time,
        cumulative: 0.0,
        kind: match e.kind {
            TimelineEventKind::Cast {
                skill_name,
                damage,
                hit_time,
                gcd_secs,
                gcd_ready_at,
                ..
            } => TimelineEventKindDTO::Cast {
                start_time: e.time,
                skill_name,
                damage,
                hit_time,
                gcd_secs,
                gcd_ready_at,
            },
            TimelineEventKind::ChannelTick {
                skill_name,
                index,
                damage,
                ..
            } => TimelineEventKindDTO::ChannelTick {
                skill_name,
                index,
                damage,
            },
            TimelineEventKind::DotTick {
                skill_name,
                index,
                damage,
                advanced,
                ..
            } => TimelineEventKindDTO::DotTick {
                skill_name,
                index,
                damage,
                advanced,
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
                    overrides: None,
                },
                TimelineActionDTO::Cast {
                    skill_id: 1,
                    sub_id: 0,
                    skill_name: None,
                    overrides: None,
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
            samples: 0,
            seed: None,
        }
    }

    #[test]
    fn rejects_invalid_selections_and_counts_grouped_effects_once() {
        use jpcg_api::{CombatMetaDTO, MijiDefDTO, QixueDefDTO};
        let mut data = CombatDataDTO {
            meta: CombatMetaDTO {
                mixed_slot_from: 7,
                mixed_qixue_cap: 1,
                miji_per_skill_cap: 1,
                ..Default::default()
            },
            qixue: vec![
                QixueDefDTO {
                    id: "a".into(),
                    slot: 1,
                    ..Default::default()
                },
                QixueDefDTO {
                    id: "b".into(),
                    slot: 1,
                    ..Default::default()
                },
                QixueDefDTO {
                    id: "c".into(),
                    slot: 7,
                    group_ids: vec!["c".into(), "c2".into()],
                    ..Default::default()
                },
                QixueDefDTO {
                    id: "d".into(),
                    slot: 8,
                    ..Default::default()
                },
            ],
            miji: vec![
                MijiDefDTO {
                    id: "m1".into(),
                    skill_id: 1,
                    ..Default::default()
                },
                MijiDefDTO {
                    id: "m2".into(),
                    skill_id: 1,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let ids = |items: &[&str]| items.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(validate_selection(&data, &ids(&["a", "b"]), &[]).is_err());
        assert!(validate_selection(&data, &ids(&["c", "d"]), &[]).is_err());
        assert!(validate_selection(&data, &ids(&["c", "c2"]), &[]).is_ok());
        assert!(validate_selection(&data, &ids(&["unknown"]), &[]).is_err());
        assert!(validate_selection(&data, &[], &ids(&["m1", "m2"])).is_err());
        assert!(validate_selection(&data, &[], &ids(&["unknown"])).is_err());
        data.qixue[0].shelved = true;
        assert!(validate_selection(&data, &ids(&["a"]), &[]).is_err());
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
            .filter_map(|e| match e.kind {
                TimelineEventKindDTO::Cast { start_time, .. } => Some(start_time),
                _ => None,
            })
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

#[cfg(test)]
mod equipment_cooldown_tests {
    use super::*;
    fn input() -> timeline::TimelineInput {
        let req: TimelineRequestDTO = serde_json::from_value(serde_json::json!({
            "skills":[{"skill_id":1,"cooldown":30.0,"charges":2,"recharge":30.0}],
            "player":{},"hostile":{},"xinfa":{},"buff":{},"coeff":{}
        }))
        .unwrap();
        build_timeline_input(req).unwrap()
    }
    #[test]
    fn equipment_reduction_applies_to_prepared_skills_and_resets_between_candidates() {
        let input = input();
        let bases = input
            .skills
            .iter()
            .map(|s| (s.skill.cooldown, s.skill.recharge))
            .collect();
        let mut template = TimelineTemplate {
            input,
            cooldown_bases: bases,
        };
        template
            .set_loadout(Default::default(), &[(1, 15.0)])
            .unwrap();
        assert_eq!(template.input.skills[0].skill.cooldown, 15.0);
        assert_eq!(template.input.skills[0].skill.recharge, 15.0);
        template.set_loadout(Default::default(), &[]).unwrap();
        assert_eq!(template.input.skills[0].skill.cooldown, 30.0);
        assert_eq!(template.input.skills[0].skill.recharge, 30.0);
        template
            .set_loadout(Default::default(), &[(1, 100.0)])
            .unwrap();
        assert_eq!(template.input.skills[0].skill.cooldown, 0.0);
    }
}
