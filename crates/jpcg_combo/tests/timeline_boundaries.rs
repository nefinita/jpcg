use jpcg_api::*;
use jpcg_combo::timeline_host::calculate_timeline;

fn cast() -> TimelineActionDTO {
    TimelineActionDTO::Cast {
        skill_id: 1,
        sub_id: 0,
        skill_name: Some("测试".into()),
        overrides: None,
    }
}
fn flag(name: &str) -> TimelineActionDTO {
    TimelineActionDTO::Flag { flag: name.into() }
}
fn request() -> TimelineRequestDTO {
    TimelineRequestDTO {
        skills: vec![SkillPoolItemDTO {
            skill_id: 1,
            skill_name: "测试".into(),
            base_damage1: 100,
            base_damage2: 100,
            ..Default::default()
        }],
        forms: vec![],
        timed_buffs: vec![],
        actions: vec![cast()],
        window: 10.0,
        player: Default::default(),
        hostile: Default::default(),
        xinfa: Default::default(),
        buff: Default::default(),
        coeff: CoefficientConfigDTO {
            pvp_global_jianshang: 1.0,
            ..Default::default()
        },
        value_set: None,
        combat: None,
        qixue: vec![],
        miji: vec![],
        initial_instrument: None,
        initial_qifeng: String::new(),
        samples: 100,
        seed: Some(1),
    }
}
#[test]
fn dot_start_does_not_deal_an_extra_monte_carlo_hit() {
    let mut req = request();
    req.skills[0].dot_flag = 1;
    req.skills[0].dot_duration = 3.0;
    req.skills[0].dot_interval = 3.0;
    let total = calculate_timeline(req.clone()).unwrap().total_damage;
    assert!(total > 0.0);
    req.hostile.target_hp = total as u32 + 1;
    assert_eq!(
        calculate_timeline(req.clone()).unwrap().final_kill_prob,
        0.0
    );
    req.hostile.target_hp = total as u32;
    assert_eq!(calculate_timeline(req).unwrap().final_kill_prob, 1.0);
}
#[test]
fn lost_hp_damage_is_settled_in_expected_and_sampled_paths() {
    let mut req = request();
    let base = calculate_timeline(req.clone()).unwrap().total_damage;
    req.hostile.target_hp = (base * 1.5) as u32;
    assert_eq!(
        calculate_timeline(req.clone()).unwrap().final_kill_prob,
        0.0
    );
    req.skills[0].lost_hp_zhenshishanghai = 1.0;
    let result = calculate_timeline(req).unwrap();
    assert_eq!(result.total_damage, base * 2.0);
    assert_eq!(result.final_kill_prob, 1.0);
}
#[test]
fn adjacent_and_trailing_flags_are_resolved() {
    let mut req = request();
    req.hostile.target_hp = 1;
    req.actions = vec![flag("start"), flag("end"), cast()];
    let result = calculate_timeline(req.clone()).unwrap();
    assert_eq!(result.kill_start, Some(0.0));
    assert_eq!(result.kill_end, Some(0.0));
    assert_eq!(result.final_kill_prob, 0.0);
    req.actions = vec![cast(), flag("end")];
    assert!(calculate_timeline(req.clone()).unwrap().kill_end.is_some());
    req.actions = vec![flag("start"), flag("end")];
    let result = calculate_timeline(req).unwrap();
    assert_eq!(result.kill_start, result.kill_end);
    assert_eq!(result.final_kill_prob, 0.0);
}

#[test]
fn overrides_are_per_cast_and_do_not_reset_skill_cooldown() {
    let mut req = request();
    req.skills[0].cooldown = 4.0;
    req.skills[0].charges = 1; // 兼容旧单次/普通冷却数据：未配置 recharge。
    req.actions = vec![cast(), cast(), cast()];
    let original = calculate_timeline(req.clone()).unwrap();
    if let TimelineActionDTO::Cast { overrides, .. } = &mut req.actions[1] {
        *overrides = Some(StepOverrideDTO {
            base_damage_override: Some(1000.0),
            ..Default::default()
        });
    }
    let changed = calculate_timeline(req).unwrap();
    let damage = |r: &TimelineResultDTO| {
        r.events
            .iter()
            .filter_map(|e| match e.kind {
                TimelineEventKindDTO::Cast { damage, .. } => Some(damage),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let a = damage(&original);
    let b = damage(&changed);
    assert_eq!(a[0], b[0]);
    assert!(b[1] > a[1]);
    assert_eq!(a[2], b[2]);
    assert_eq!(
        changed.steps.iter().map(|s| s.time).collect::<Vec<_>>(),
        vec![Some(0.0), Some(4.0), Some(8.0)]
    );
}

#[test]
fn cumulative_uses_only_hits_inside_window_in_hit_order() {
    // 读条跨窗口：已起手但尚未命中，不应出现在伤害日志。
    let mut req = request();
    req.window = 1.0;
    req.skills[0].cast_time = 2.0;
    let result = calculate_timeline(req.clone()).unwrap();
    assert_eq!(result.total_damage, 0.0);
    assert!(result.events.iter().all(|e| e.cumulative == 0.0));
    // 引导跨窗口：仅前两段计入；后端日志最终累计与总伤一致。
    req.skills[0].channel = true;
    req.combat = Some(serde_json::from_value(serde_json::json!({"skill_combat": [{"skill_id":1,"sub_id":0,"channel":true,"channel_ticks":4}]})).unwrap());
    let result = calculate_timeline(req).unwrap();
    assert!(result.total_damage > 0.0);
    assert_eq!(
        result
            .events
            .iter()
            .filter(|e| matches!(e.kind, TimelineEventKindDTO::ChannelTick { .. }))
            .count(),
        2
    );
    assert!(result.events.iter().all(|e| e.time <= 1.0));
    assert_eq!(
        result.events.last().map(|e| e.cumulative).unwrap_or(0.0),
        result.total_damage
    );
    // 在读条中间有 DOT 命中：DOT 不能提前计入尚未结束的读条伤害。
    let mut req = request();
    let mut dot = req.skills[0].clone();
    dot.skill_id = 2;
    dot.skill_name = "持续".into();
    dot.dot_flag = 1;
    dot.dot_interval = 1.0;
    dot.dot_duration = 4.0;
    req.skills.push(dot);
    req.skills[0].cast_time = 2.0;
    req.actions = vec![
        TimelineActionDTO::Cast {
            skill_id: 2,
            sub_id: 0,
            skill_name: None,
            overrides: None,
        },
        cast(),
    ];
    let result = calculate_timeline(req).unwrap();
    let mut sum = 0.0;
    for e in &result.events {
        let d = match e.kind {
            TimelineEventKindDTO::Cast {
                damage, hit_time, ..
            } => {
                assert_eq!(e.time, hit_time);
                damage
            }
            TimelineEventKindDTO::DotTick { damage, .. }
            | TimelineEventKindDTO::ChannelTick { damage, .. } => damage,
            _ => 0,
        };
        sum += d as f64;
        assert_eq!(e.cumulative, sum);
        if e.time < 3.5 {
            assert!(!matches!(e.kind,TimelineEventKindDTO::Cast{damage,..} if damage>0));
        }
    }
    assert!(result.events.windows(2).all(|w| w[0].time <= w[1].time));
    // 总伤只统计测量窗口内的伤害；窗口外事件仍在 events 里（供展示/调试）。
    let kill_start = result.kill_start.unwrap_or(0.0);
    let kill_end = result.kill_end.unwrap_or(0.0);
    let window_sum: f64 = result
        .events
        .iter()
        .filter(|e| e.time >= kill_start - 1e-4 && e.time <= kill_end + 1e-4)
        .map(|e| match e.kind {
            TimelineEventKindDTO::Cast { damage, .. }
            | TimelineEventKindDTO::DotTick { damage, .. }
            | TimelineEventKindDTO::ChannelTick { damage, .. } => damage as f64,
            _ => 0.0,
        })
        .sum();
    assert_eq!(window_sum, result.total_damage);
}

#[test]
fn step_reduction_gain_and_ignore_mitigation_have_effect_without_leaking() {
    for dot in [false, true] {
        let mut req = request();
        if dot {
            req.skills[0].dot_flag = 1;
            req.skills[0].dot_duration = 2.0;
            req.skills[0].dot_interval = 1.0;
        }
        let baseline = calculate_timeline(req.clone()).unwrap().total_damage;
        if let TimelineActionDTO::Cast { overrides, .. } = &mut req.actions[0] {
            *overrides = Some(StepOverrideDTO {
                jianshang_bili_override: Some(50.0),
                ..Default::default()
            });
        }
        assert!(calculate_timeline(req.clone()).unwrap().total_damage <= baseline * 0.5);
        if let TimelineActionDTO::Cast { overrides, .. } = &mut req.actions[0] {
            *overrides = Some(StepOverrideDTO {
                wushihuajin_override: Some(100.0),
                ..Default::default()
            });
        }
        assert!(calculate_timeline(req).unwrap().total_damage > baseline);
    }
    let mut req = request();
    req.buff.shanghai_pct = 100.0;
    let baseline = calculate_timeline(req.clone()).unwrap().total_damage;
    if let TimelineActionDTO::Cast { overrides, .. } = &mut req.actions[0] {
        *overrides = Some(StepOverrideDTO {
            gain_override: Some(100.0),
            ..Default::default()
        });
    }
    assert_eq!(
        calculate_timeline(req).unwrap().total_damage,
        baseline * 2.0,
        "技能增益不能覆盖全局伤害 buff"
    );
}

#[test]
fn kill_curve_only_covers_flag_window() {
    // 步骤：cast / 开始 / cast / 结束 / cast
    // 曲线只应覆盖 [开始 flag, 结束 flag] 之间的步骤（1-based 2..4），窗口外不出点。
    let mut req = request();
    // 需要目标血量才会跑击杀率 MC（full_hp 非空）。
    req.hostile.target_hp = 1_000_000;
    // 步骤：cast / 开始 / cast / cast / 结束 / cast
    req.actions = vec![cast(), flag("start"), cast(), cast(), flag("end"), cast()];
    let result = calculate_timeline(req).expect("timeline");
    assert!(result.kill_prob_curve.len() >= 2, "曲线不应为空");
    // 窗口内的动作 step：第 3、4 步（1-based）。
    assert_eq!(result.kill_prob_curve.first().unwrap().0, 3);
    assert_eq!(result.kill_prob_curve.last().unwrap().0, 4);
}

#[test]
fn end_flag_excludes_cast_at_same_time() {
    // 结束标记下方的第二招（同一 GCD 起点）不应计入窗口。
    let mut one = request();
    one.hostile.target_hp = 0;
    let single = calculate_timeline(one).expect("single").total_damage;

    let mut req = request();
    req.actions = vec![cast(), flag("end"), cast()];
    req.hostile.target_hp = single as u32 + 10;
    let result = calculate_timeline(req).expect("timeline");
    assert_eq!(
        result.total_damage, single,
        "结束标记后的第二招不应计入总伤"
    );
    assert_eq!(result.final_kill_prob, 0.0, "只有第一招伤害，打不死目标");
}

#[test]
fn end_flag_keeps_previous_cast_at_same_time() {
    // 上一招 2s 读条，命中与下一招起点同为 2s；结束标记在两者之间，上一招的合法命中必须保留。
    let mut one = request();
    one.skills[0].cast_time = 2.0;
    one.hostile.target_hp = 0;
    let single = calculate_timeline(one).expect("single").total_damage;
    assert!(single > 0.0);

    let mut req = request();
    req.skills[0].cast_time = 2.0;
    req.actions = vec![cast(), flag("end"), cast()];
    req.hostile.target_hp = single as u32 + 10;
    let result = calculate_timeline(req).expect("timeline");
    assert_eq!(result.total_damage, single, "上一招的命中必须保留");
    assert_eq!(result.final_kill_prob, 0.0);
}

#[test]
fn end_flag_before_first_instant_cast_excludes_it() {
    let mut req = request();
    req.actions = vec![flag("end"), cast()];
    req.hostile.target_hp = 1;
    let result = calculate_timeline(req).expect("timeline");
    assert_eq!(result.total_damage, 0.0, "结束标记之后的瞬发不应计入");
    assert_eq!(result.final_kill_prob, 0.0);
}

#[test]
fn adjacent_start_end_is_empty_window() {
    let mut req = request();
    req.actions = vec![flag("start"), flag("end"), cast()];
    req.hostile.target_hp = 1;
    let result = calculate_timeline(req).expect("timeline");
    assert_eq!(result.total_damage, 0.0);
    assert_eq!(result.final_kill_prob, 0.0);
}
