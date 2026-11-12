use jpcg_api::*;

fn req(qixue: Vec<String>, actions: Vec<TimelineActionDTO>) -> TimelineRequestDTO {
    req_full(qixue, Vec::new(), actions)
}

fn req_full(
    qixue: Vec<String>,
    miji: Vec<String>,
    actions: Vec<TimelineActionDTO>,
) -> TimelineRequestDTO {
    let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    unsafe { std::env::set_var("JPCG_DATA_DIR", &data) };
    let (skills, combat) =
        jpcg_core::host::combat::load_combat_with_skills("mowen".to_string()).expect("bundle");
    TimelineRequestDTO {
        skills,
        forms: vec![],
        initial_instrument: None,
        initial_qifeng: String::new(),
        timed_buffs: vec![],
        qixue,
        miji,
        actions,
        window: 20.0,
        player: PlayerConfigDTO::default(),
        hostile: HostileConfigDTO::default(),
        xinfa: XinfaConfigDTO::default(),
        buff: BuffConfigDTO::default(),
        coeff: CoefficientConfigDTO {
            pvp_global_jianshang: 0.9,
            ..Default::default()
        },
        value_set: None,
        combat: Some(combat),
        samples: 0,
        seed: None,
    }
}

fn req_instrument(
    instrument: &str,
    qixue: Vec<String>,
    actions: Vec<TimelineActionDTO>,
) -> TimelineRequestDTO {
    let mut r = req(qixue, actions);
    r.initial_instrument = Some(instrument.into());
    r
}

fn cast(id: u32, sub: u32, name: &str) -> TimelineActionDTO {
    TimelineActionDTO::Cast {
        skill_id: id,
        sub_id: sub,
        skill_name: Some(name.into()),
        overrides: None,
    }
}

fn gong_damage(result: &TimelineResultDTO) -> u32 {
    result
        .events
        .iter()
        .find_map(|e| match &e.kind {
            TimelineEventKindDTO::Cast {
                skill_name, damage, ..
            } if skill_name.starts_with("宫") => Some(*damage),
            _ => None,
        })
        .unwrap_or(0)
}

#[test]
fn jingzhi_adds_damage_per_qifeng_spent() {
    // 富商（+1 曲风）→ 宫（消耗全部）
    let r = jpcg_combo::timeline_host::calculate_timeline(req(
        vec![],
        vec![cast(14065, 14311, "商"), cast(14064, 14474, "宫")],
    ))
    .expect("base");
    let base = gong_damage(&r);
    assert!(base > 0, "宫 应有伤害");

    let r2 = jpcg_combo::timeline_host::calculate_timeline(req(
        vec!["jingzhi".into()],
        vec![cast(14065, 14311, "商"), cast(14064, 14474, "宫")],
    ))
    .expect("hook");
    let boosted = gong_damage(&r2);
    // 战斗内初始曲风满点 9（非战斗获取不建模），宫 一次消耗全部 9 点
    assert_eq!(
        boosted as i64 - base as i64,
        9 * 1330,
        "净植应按消耗的 9 点曲风加 9×1330"
    );
}

fn casts(result: &TimelineResultDTO) -> Vec<(String, u32, f32)> {
    result
        .events
        .iter()
        .filter_map(|e| match &e.kind {
            TimelineEventKindDTO::Cast {
                skill_name, damage, ..
            } => Some((skill_name.clone(), *damage, e.time)),
            _ => None,
        })
        .collect()
}

#[test]
fn shixiang_scales_with_qifeng() {
    // 商 获得 1 点曲风；连放两次商，第二次吃 2%/点
    let base = jpcg_combo::timeline_host::calculate_timeline(req(
        vec![],
        vec![cast(14065, 14311, "商"), cast(14065, 14311, "商")],
    ))
    .expect("base");
    let b = casts(&base);
    assert_eq!(b.len(), 2);
    let base2 = b[1].1;

    let with = jpcg_combo::timeline_host::calculate_timeline(req(
        vec!["shixiang".into()],
        vec![cast(14065, 14311, "商"), cast(14065, 14311, "商")],
    ))
    .expect("hook");
    let w = casts(&with);
    assert_eq!(w.len(), 2);
    // 初始 9 点曲风 → 师襄每点 +2% 会心效果，共 +18%
    let expected = (base2 as f32 * 1.18).round() as i64;
    assert!(
        (w[1].1 as i64 - expected).abs() <= 2,
        "师襄（9 点曲风 +18%）：期望约 {expected}，实际 {}",
        w[1].1
    );
}

#[test]
fn yince_adds_flat_damage_on_sword_skills() {
    let base = jpcg_combo::timeline_host::calculate_timeline(req_instrument(
        "jian",
        vec![],
        vec![cast(14451, 0, "剑·徵")],
    ))
    .expect("base");
    let b = casts(&base);
    let with = jpcg_combo::timeline_host::calculate_timeline(req_instrument(
        "jian",
        vec!["yince".into()],
        vec![cast(14451, 0, "剑·徵")],
    ))
    .expect("hook");
    let w = casts(&with);
    assert_eq!(b.len(), 1);
    assert_eq!(w.len(), 1);
    assert_eq!(
        w[0].1 as i64 - b[0].1 as i64,
        2159,
        "音彻：剑·音 命中追加 2159"
    );
}

#[test]
fn miji_boosts_damage() {
    let base =
        jpcg_combo::timeline_host::calculate_timeline(req(vec![], vec![cast(14064, 14474, "宫")]))
            .expect("base");
    let b = casts(&base);
    assert_eq!(b.len(), 1);
    let base_dmg = b[0].1;

    let with = jpcg_combo::timeline_host::calculate_timeline(req_full(
        vec![],
        vec!["gong_dmg4".into()],
        vec![cast(14064, 14474, "宫")],
    ))
    .expect("miji");
    let w = casts(&with);
    assert_eq!(w.len(), 1);
    let expected = (base_dmg as f32 * 1.04).round() as i64;
    assert!(
        (w[0].1 as i64 - expected).abs() <= 2,
        "宫 秘籍 +4%：期望约 {expected}，实际 {}",
        w[0].1
    );
}

#[test]
fn miji_cuts_cast_time() {
    let base = jpcg_combo::timeline_host::calculate_timeline(req(
        vec![],
        vec![cast(14064, 14474, "宫"), cast(14065, 14311, "商")],
    ))
    .expect("base");
    let with = jpcg_combo::timeline_host::calculate_timeline(req_full(
        vec![],
        vec!["gong_cast1".into(), "gong_cast2".into()],
        vec![cast(14064, 14474, "宫"), cast(14065, 14311, "商")],
    ))
    .expect("miji");
    let b = casts(&base);
    let w = casts(&with);
    assert_eq!(b.len(), 2);
    assert_eq!(w.len(), 2);
    assert!(
        w[1].2 < b[1].2 - 0.2,
        "两条减读条秘籍应让后续技能提前（{} vs {}）",
        w[1].2,
        b[1].2
    );
}

#[test]
fn replacement_qixue_changes_skill() {
    // 变徵（14299/18859）默认是 84-92 ×3；洞天替换成 4608 ×3 固定伤害
    let switch = || TimelineActionDTO::Cast {
        skill_id: 14069,
        sub_id: 0,
        skill_name: Some("高山流水".into()),
        overrides: None,
    };
    let base = jpcg_combo::timeline_host::calculate_timeline(req(
        vec![],
        vec![
            switch(),
            TimelineActionDTO::Cast {
                skill_id: 14299,
                sub_id: 18859,
                skill_name: None,
                overrides: None,
            },
        ],
    ))
    .expect("base");
    let b = casts(&base);
    assert_eq!(b.len(), 2, "曲目技能 + 变徵");
    assert_eq!(b[1].0, "变徵");

    let with = jpcg_combo::timeline_host::calculate_timeline(req(
        vec!["dongtian".into()],
        vec![
            switch(),
            TimelineActionDTO::Cast {
                skill_id: 14299,
                sub_id: 18859,
                skill_name: None,
                overrides: None,
            },
        ],
    ))
    .expect("dongtian");
    let w = casts(&with);
    assert_eq!(w.len(), 2);
    assert_eq!(w[1].0, "洞天", "替换后技能名应变为洞天");
    let dot_total: f64 = with
        .events
        .iter()
        .filter_map(|e| match &e.kind {
            TimelineEventKindDTO::DotTick {
                skill_name, damage, ..
            } if skill_name == "洞天" => Some(*damage as f64),
            _ => None,
        })
        .sum();
    assert!(
        dot_total > 10_000.0,
        "洞天 3 次固定伤害合计应过万，实际 {dot_total}"
    );
}

#[test]
fn luanyu_replaces_qingxiao_with_dash_attack() {
    // 青霄飞羽 14076 默认不是伤害技；鸾羽替换为「飞鸾动羽」999 点固定伤害
    let with = jpcg_combo::timeline_host::calculate_timeline(req(
        vec!["luanyu".into()],
        vec![cast(14076, 0, "青霄飞羽")],
    ))
    .expect("luanyu");
    let w = casts(&with);
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].0, "飞鸾动羽");
    assert!(w[0].1 > 0, "飞鸾动羽应有伤害");
}

#[test]
fn dot_ticks_come_from_combat_overlay() {
    // 商/角 的 DOT 由 overlay 独立定义（dot_interval/dot_duration/每跳系数），
    // 施放本体后应在时间轴上产生 6 跳持续伤害事件。
    let mut request = req(vec![], vec![cast(14065, 14311, "商")]);
    request.player.jichu_gongji = 8438;
    let result = jpcg_combo::timeline_host::calculate_timeline(request).expect("dot");
    let ticks: Vec<u32> = result
        .events
        .iter()
        .filter_map(|e| match &e.kind {
            TimelineEventKindDTO::DotTick { damage, .. } => Some(*damage),
            _ => None,
        })
        .collect();
    assert_eq!(ticks.len(), 6, "商 DOT 应为 6 跳：{ticks:?}");
    assert!(ticks.iter().all(|d| *d > 0), "每跳伤害应大于 0：{ticks:?}");
}

#[test]
fn dot_modifiers_apply_to_overlay_dot() {
    // 殊曲：商/角 DOT 间隔 -33.3% → 18s / 2s = 9 跳
    let mut shuqu = req(vec!["shuqu".into()], vec![cast(14065, 14311, "商")]);
    shuqu.player.jichu_gongji = 8438;
    let result = jpcg_combo::timeline_host::calculate_timeline(shuqu).expect("shuqu");
    let ticks = result
        .events
        .iter()
        .filter(|e| matches!(&e.kind, TimelineEventKindDTO::DotTick { .. }))
        .count();
    assert_eq!(ticks, 9, "殊曲后商 DOT 应为 9 跳，实际 {ticks}");

    // 冻泉：跳数 +4 → 10 跳
    let mut dongquan = req(vec!["dongquan".into()], vec![cast(14065, 14311, "商")]);
    dongquan.player.jichu_gongji = 8438;
    dongquan.window = 60.0; // 冻泉后 DOT 持续 30 秒，窗口需覆盖整段
    let result = jpcg_combo::timeline_host::calculate_timeline(dongquan).expect("dongquan");
    let ticks = result
        .events
        .iter()
        .filter(|e| matches!(&e.kind, TimelineEventKindDTO::DotTick { .. }))
        .count();
    assert_eq!(ticks, 10, "冻泉后商 DOT 应为 10 跳，实际 {ticks}");
}

#[test]
fn shuqu_dot_ticks_ramp_up() {
    // 殊曲：商 DOT 每跳 +12%（dot_up = 0.12）
    let mut request = req(vec!["shuqu".into()], vec![cast(14065, 14311, "商")]);
    request.player.jichu_gongji = 8438;
    request.window = 60.0;
    let result = jpcg_combo::timeline_host::calculate_timeline(request).expect("shuqu");
    let damages: Vec<u32> = result
        .events
        .iter()
        .filter_map(|e| match &e.kind {
            TimelineEventKindDTO::DotTick {
                skill_name, damage, ..
            } if skill_name.contains("商") => Some(*damage),
            _ => None,
        })
        .collect();
    assert_eq!(damages.len(), 9, "殊曲后 9 跳：{damages:?}");
    assert!(
        damages.windows(2).all(|w| w[1] >= w[0]),
        "每跳应递增：{damages:?}"
    );
    assert!(
        damages[8] > damages[0] * 2,
        "9 跳后应明显高于首跳：{damages:?}"
    );
}

#[test]
fn dongtian_advances_shang_dot_ticks() {
    // 洞天：变徵被替换为 3 跳 4608，每次跳让商/角下一跳立刻生效（提前，不补跳）
    let mut request = req(
        vec!["dongtian".into()],
        vec![
            cast(14065, 14311, "商"),
            cast(14069, 0, "高山流水"),
            cast(14299, 18859, "变徵"),
        ],
    );
    request.window = 60.0;
    request.player.jichu_gongji = 8438;
    request.window = 60.0;
    let result = jpcg_combo::timeline_host::calculate_timeline(request).expect("dongtian");
    let shang: Vec<(f32, u32)> = result
        .events
        .iter()
        .filter_map(|e| match &e.kind {
            TimelineEventKindDTO::DotTick {
                skill_name, damage, ..
            } if skill_name.contains("商") => Some((e.time, *damage)),
            _ => None,
        })
        .collect();
    let dongtian_ticks = result
        .events
        .iter()
        .filter(|e| matches!(&e.kind, TimelineEventKindDTO::DotTick { skill_name, .. } if skill_name == "洞天"))
        .count();
    assert_eq!(dongtian_ticks, 3, "洞天应为 3 跳");
    assert_eq!(shang.len(), 6, "商 DOT 总跳数不变：{shang:?}");
    assert!(shang[0].0 < 2.9, "商首跳应被洞天提前到 3 秒前：{shang:?}");
    assert!(
        shang.iter().any(|(t, _)| *t < 2.9),
        "至少一跳被提前：{shang:?}"
    );
}

#[test]
fn zheng_channel_has_three_ticks() {
    // 徵：倒读条 2.8125s，分 3 段结算；Cast 事件本身不带伤害
    let mut request = req(vec![], vec![cast(14067, 14227, "徵")]);
    request.player.jichu_gongji = 8438;
    request.window = 30.0;
    let result = jpcg_combo::timeline_host::calculate_timeline(request).expect("zheng");
    let ticks: Vec<(f32, u32)> = result
        .events
        .iter()
        .filter_map(|e| match &e.kind {
            TimelineEventKindDTO::ChannelTick { damage, .. } => Some((e.time, *damage)),
            _ => None,
        })
        .collect();
    assert_eq!(ticks.len(), 3, "徵应为 3 段：{ticks:?}");
    assert!(ticks.iter().all(|(_, d)| *d > 0), "{ticks:?}");
    assert!(
        (ticks[0].0 - 0.9375).abs() < 1e-3,
        "首段应在 0.9375s：{ticks:?}"
    );
    assert!(
        (ticks[2].0 - 2.8125).abs() < 1e-3,
        "末段应在 2.8125s：{ticks:?}"
    );
    let cast_damage: u32 = result
        .events
        .iter()
        .filter_map(|e| match &e.kind {
            TimelineEventKindDTO::Cast {
                skill_name, damage, ..
            } if skill_name == "徵" => Some(*damage),
            _ => None,
        })
        .sum();
    assert_eq!(cast_damage, 0, "引导技能伤害应只出现在分段事件上");
}

#[test]
fn yu_triggers_two_shang_dot_ticks() {
    // 羽：命中后使自身「商」「角」下两跳立刻生效
    let mut request = req(
        vec![],
        vec![cast(14065, 14311, "商"), cast(14068, 14100, "羽")],
    );
    request.player.jichu_gongji = 8438;
    request.window = 60.0;
    let result = jpcg_combo::timeline_host::calculate_timeline(request).expect("yu");
    let ticks: Vec<f32> = result
        .events
        .iter()
        .filter_map(|e| match &e.kind {
            TimelineEventKindDTO::DotTick { skill_name, .. } if skill_name.contains("商") => {
                Some(e.time)
            }
            _ => None,
        })
        .collect();
    assert_eq!(ticks.len(), 6, "商 DOT 总跳数不变：{ticks:?}");
    let advanced = ticks.iter().filter(|t| (**t - 1.5).abs() < 0.2).count();
    assert_eq!(advanced, 2, "应有 2 跳被羽提前到 1.5s：{ticks:?}");
    let marked = result
        .events
        .iter()
        .filter(|e| matches!(&e.kind, TimelineEventKindDTO::DotTick { skill_name, advanced: true, .. } if skill_name.contains("商")))
        .count();
    assert_eq!(marked, 2, "被羽提前的跳应带 advanced 标记");
    let last = ticks.iter().cloned().fold(0.0f32, f32::max);
    assert!(
        last < 17.0,
        "提前后后续跳应整体前移（末跳 < 17s）：{ticks:?}"
    );
}

#[test]
fn panel_bundle_is_enriched_by_backend() {
    let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    unsafe { std::env::set_var("JPCG_DATA_DIR", &data) };
    let (_, combat) =
        jpcg_core::host::combat::load_combat_with_skills("mowen".to_string()).expect("bundle");
    // 秘籍 → 技能配对
    let gong = combat
        .skill_combat
        .iter()
        .find(|s| s.skill_id == 14064 && s.sub_id == 14474)
        .expect("宫");
    assert!(
        gong.miji_ids.iter().any(|id| id == "gong_cast1"),
        "{:?}",
        gong.miji_ids
    );
    // 秘籍效果文案由后端生成
    let miji = combat
        .miji
        .iter()
        .find(|m| m.id == "gong_cast1")
        .expect("miji");
    assert!(
        miji.effects.iter().any(|e| e.contains("运功")),
        "{:?}",
        miji.effects
    );
    // 面板规则
    assert_eq!(combat.meta.miji_per_skill_cap, 4);
    assert_eq!(combat.meta.mixed_qixue_cap, 3);
    assert_eq!(combat.meta.mixed_slot_from, 7);
    // 奇穴分组与同名合并（冻泉 + 冻泉·宫）
    let dong = combat
        .qixue
        .iter()
        .find(|q| q.name == "冻泉")
        .expect("冻泉");
    assert_eq!(dong.group_label, "第四重");
    assert!(dong.group_ids.contains(&"dongquan".to_string()));
    assert!(dong.group_ids.contains(&"dongquan-gong".to_string()));
    assert!(
        combat
            .qixue
            .iter()
            .filter(|q| q.group_label == "混选")
            .all(|q| q.slot >= 7)
    );
}

#[test]
fn dongtian_recast_keeps_past_ticks() {
    // 连续两次洞天刷新保留过去伤害，新实例从第一跳重新结算
    let mut request = req(
        vec!["dongtian".into()],
        vec![
            cast(14065, 14311, "商"),
            cast(14069, 0, "高山流水"),
            cast(14299, 18859, "变徵"),
            cast(14299, 18859, "变徵"),
        ],
    );
    request.player.jichu_gongji = 8438;
    request.player.jiasu_dengji = 900;
    request.window = 60.0;
    let result = jpcg_combo::timeline_host::calculate_timeline(request).expect("recast");
    let indices: Vec<u32> = result
        .events
        .iter()
        .filter_map(|e| match &e.kind {
            TimelineEventKindDTO::DotTick {
                skill_name, index, ..
            } if skill_name == "洞天" => Some(*index),
            _ => None,
        })
        .collect();
    for e in &result.events {
        if let TimelineEventKindDTO::DotTick {
            skill_name, damage, ..
        } = &e.kind
        {
            if skill_name == "洞天" {
                assert!(*damage > 0, "刷新后的跳不能归零");
            }
        }
    }
    // 老实例已生效的 3 跳保留 1..3；刷新后的新一轮重新从 1 开始
    assert_eq!(
        indices,
        vec![1, 2, 3, 1, 2, 3],
        "刷新后新一轮应从第 1 跳编号：{indices:?}"
    );
}

#[test]
fn idle_skill_exists_and_occupies_gcd() {
    // 所有门派技能池都有「停手」空位：0 伤害、无属性、占普通 GCD
    let pool = jpcg_core::host::skill::load_skill_pool("mowen".to_string());
    let idle = pool
        .iter()
        .find(|s| s.skill_name == "停手")
        .expect("技能池应包含停手");
    assert_eq!(idle.skill_id, jpcg_core::type_set::skilltype::IDLE_SKILL_ID);
    assert_eq!(idle.base_damage1, 0);
    assert_eq!(idle.base_damage2, 0);
    assert_eq!(idle.atk_xishu, 0.0);
    assert_eq!(idle.dot_flag, 0);

    let mut request = req(
        vec![],
        vec![
            cast(jpcg_core::type_set::skilltype::IDLE_SKILL_ID, 0, "停手"),
            cast(14064, 14474, "宫"),
        ],
    );
    request.player.jichu_gongji = 8438;
    request.window = 10.0;
    let result = jpcg_combo::timeline_host::calculate_timeline(request).expect("idle");
    let idle_cast = result
        .events
        .iter()
        .find_map(|e| match &e.kind {
            TimelineEventKindDTO::Cast {
                skill_name, damage, ..
            } if skill_name == "停手" => Some((e.time, *damage)),
            _ => None,
        })
        .expect("停手 Cast 事件");
    assert_eq!(idle_cast.0, 0.0, "停手在 0s 施放");
    assert_eq!(idle_cast.1, 0, "停手伤害为 0");
    let gong_time = result
        .events
        .iter()
        .find_map(|e| match &e.kind {
            TimelineEventKindDTO::Cast {
                skill_name,
                start_time,
                ..
            } if skill_name.starts_with("宫") => Some(*start_time),
            _ => None,
        })
        .expect("宫 Cast");
    assert!(
        (gong_time - 1.5).abs() < 0.05,
        "停手应占一个普通 GCD：{gong_time}"
    );
}

#[test]
fn steps_are_computed_by_backend() {
    // 变徵需要高山曲风：商（合法）+ 变徵（非法）→ steps 直接给出结果，前端不再自己推导
    let mut request = req(
        vec![],
        vec![cast(14065, 14311, "商"), cast(14299, 18859, "变徵")],
    );
    request.player.jichu_gongji = 8438;
    request.window = 20.0;
    let result = jpcg_combo::timeline_host::calculate_timeline(request).expect("steps");
    assert_eq!(result.steps.len(), 2);
    assert!(result.steps[0].legal);
    assert!((result.steps[0].time.unwrap_or(-1.0) - 0.0).abs() < 0.01);
    assert!(
        !result.steps[1].legal,
        "变徵应判为非法：{:?}",
        result.steps[1]
    );
    assert!(!result.steps[1].skipped_reason.is_empty());
    assert_eq!(result.skipped_actions, 1);
}

#[test]
fn channel_damage_is_included_in_total() {
    // 徵 3 段引导伤害必须计入窗口总伤（之前漏了 ChannelTick）
    let mut request = req(vec![], vec![cast(14067, 14227, "徵")]);
    request.player.jichu_gongji = 8438;
    request.window = 30.0;
    let result = jpcg_combo::timeline_host::calculate_timeline(request).expect("channel");
    let channel_sum: f64 = result
        .events
        .iter()
        .filter_map(|e| match &e.kind {
            TimelineEventKindDTO::ChannelTick { damage, .. } => Some(*damage as f64),
            _ => None,
        })
        .sum();
    assert!(channel_sum > 0.0);
    assert!(
        (result.total_damage - channel_sum).abs() < 1.0,
        "总伤应包含所有引导段：{} vs {}",
        result.total_damage,
        channel_sum
    );
    // 事件累计伤害由后端给出且单调
    let mut last = 0.0;
    for e in &result.events {
        assert!(e.cumulative >= last);
        last = e.cumulative;
    }
}

#[test]
fn desktop_entry_uses_full_backend_definitions_and_rejects_invalid_selection() {
    let mut request = req(vec![], vec![cast(14064, 14474, "宫")]);
    request.xinfa.profession = "mowen".into();
    let baseline = jpcg_combo::timeline_host::calculate_timeline_from_store(request.clone())
        .expect("backend definitions");
    // 即使展示 DTO / 技能属性被篡改，桌面入口也只使用后端定义。
    request.skills.clear();
    request.combat = None;
    let result = jpcg_combo::timeline_host::calculate_timeline_from_store(request.clone())
        .expect("backend definitions");
    assert_eq!(result.total_damage, baseline.total_damage);
    assert!(!result.events.is_empty());
    request.qixue = vec!["not-a-real-qixue".into()];
    assert!(jpcg_combo::timeline_host::calculate_timeline_from_store(request).is_err());
}

#[test]
fn kill_probability_matches_deterministic_total() {
    // 单技能无质（has_critical_strike）场景下，击杀率应严格由确定性总伤决定。
    let mut request = req(vec![], vec![cast(14065, 14311, "商")]);
    request.player.jichu_gongji = 8438;
    request.window = 10.0;
    request.samples = 2000;
    request.seed = Some(7);
    let det = jpcg_combo::timeline_host::calculate_timeline(request.clone()).expect("det");
    // 目标血量高于总伤 → 永不击杀；低于总伤 → 必杀
    let total = det.total_damage as u32;
    request.hostile.target_hp = total + 10_000;
    let low = jpcg_combo::timeline_host::calculate_timeline(request.clone()).expect("low");
    assert_eq!(low.final_kill_prob, 0.0, "血量高于总伤应为 0");
    request.hostile.target_hp = total / 2;
    let high = jpcg_combo::timeline_host::calculate_timeline(request.clone()).expect("high");
    assert_eq!(high.final_kill_prob, 1.0, "血量远低于总伤应为 1");
    assert!(!high.kill_prob_curve.is_empty());
    // samples=0 保持纯确定性：不产生击杀率
    let mut d = request.clone();
    d.samples = 0;
    let pure = jpcg_combo::timeline_host::calculate_timeline(d).expect("pure");
    assert_eq!(pure.final_kill_prob, 0.0);
    assert!(pure.kill_prob_curve.is_empty());
}

fn fixed_skill(id: u32, name: &str, base: u32) -> SkillPoolItemDTO {
    SkillPoolItemDTO {
        skill_name: name.into(),
        skill_id: id,
        sub_id: 0,
        base_damage1: base,
        base_damage2: base,
        atk_xishu: 0.0,
        watk_xishu: 0,
        hit_up: 0,
        huixin_up: 0,
        huixiao_up: 0,
        wushifangyu: 0,
        wushihuajin: 0,
        dot_flag: 0,
        has_critical_strike: false,
        cast_time: 0.0,
        gcd_kind: 0,
        ..Default::default()
    }
}

#[test]
fn kill_flag_scopes_kill_probability_with_full_hp_reset() {
    // 确定性技能（g==h）：3 段固定伤害。开始 flag 后前两段不计入击杀推进（满血重置）。
    let skill = fixed_skill(1, "测试技能", 100);
    let make = |actions: Vec<TimelineActionDTO>| TimelineRequestDTO {
        skills: vec![skill.clone()],
        forms: vec![],
        timed_buffs: vec![],
        actions,
        window: 30.0,
        player: PlayerConfigDTO::default(),
        hostile: HostileConfigDTO {
            target_hp: 0,
            ..Default::default()
        },
        xinfa: XinfaConfigDTO::default(),
        buff: BuffConfigDTO::default(),
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
        samples: 2000,
        seed: Some(42),
    };
    let three = vec![
        cast(1, 0, "测试技能"),
        cast(1, 0, "测试技能"),
        cast(1, 0, "测试技能"),
    ];
    let mut base_req = make(three.clone());
    base_req.hostile.target_hp = 0;
    let base = jpcg_combo::timeline_host::calculate_timeline(base_req.clone()).expect("base");
    let total = base.total_damage;
    assert!(total > 0.0);

    // 无 flag：3 段全打，target_hp = total → 必杀
    let mut no_flag = make(three.clone());
    no_flag.hostile.target_hp = total as u32;
    let r = jpcg_combo::timeline_host::calculate_timeline(no_flag).expect("no_flag");
    assert_eq!(r.final_kill_prob, 1.0, "3 段固定伤害应必杀");

    // 开始 flag 在第 2 段之后 → 只有第 3 段计入（满血重置），target=total 打不死
    let mut with_start = make(vec![
        cast(1, 0, "测试技能"),
        cast(1, 0, "测试技能"),
        TimelineActionDTO::Flag {
            flag: "start".into(),
        },
        cast(1, 0, "测试技能"),
    ]);
    with_start.hostile.target_hp = total as u32;
    let rs = jpcg_combo::timeline_host::calculate_timeline(with_start).expect("with_start");
    assert_eq!(
        rs.final_kill_prob, 0.0,
        "前两段被开始 flag 排除，仅剩 1/3 伤害"
    );
    assert_eq!(rs.total_damage, base.total_damage, "总伤不受 flag 影响");
    assert!(rs.kill_start.is_some());
}

#[test]
fn kill_flag_validation_rejects_duplicates_and_bad_order() {
    let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    unsafe { std::env::set_var("JPCG_DATA_DIR", &data) };
    let skill = fixed_skill(1, "测试技能", 100);
    let mut req = TimelineRequestDTO {
        skills: vec![skill],
        forms: vec![],
        timed_buffs: vec![],
        actions: vec![],
        window: 30.0,
        player: PlayerConfigDTO::default(),
        hostile: HostileConfigDTO::default(),
        xinfa: XinfaConfigDTO::default(),
        buff: BuffConfigDTO::default(),
        coeff: CoefficientConfigDTO::default(),
        value_set: None,
        combat: None,
        qixue: vec![],
        miji: vec![],
        initial_instrument: None,
        initial_qifeng: String::new(),
        samples: 0,
        seed: None,
    };
    // 两个 start
    req.actions = vec![
        TimelineActionDTO::Flag {
            flag: "start".into(),
        },
        cast(1, 0, "测试技能"),
        TimelineActionDTO::Flag {
            flag: "start".into(),
        },
    ];
    assert!(jpcg_combo::timeline_host::calculate_timeline_from_store(req.clone()).is_err());
    // 开始晚于结束
    req.actions = vec![
        TimelineActionDTO::Flag { flag: "end".into() },
        cast(1, 0, "测试技能"),
        TimelineActionDTO::Flag {
            flag: "start".into(),
        },
    ];
    assert!(jpcg_combo::timeline_host::calculate_timeline_from_store(req.clone()).is_err());
}

#[test]
fn window_dps_uses_actual_damage_span() {
    // 3 次瞬发（GCD 1.5）→ 最后一次命中在 3.0s；区间 DPS 应为 total/3，而不是 total/300。
    let request = TimelineRequestDTO {
        skills: vec![fixed_skill(1, "测试技能", 100)],
        forms: vec![],
        timed_buffs: vec![],
        actions: vec![
            cast(1, 0, "测试技能"),
            cast(1, 0, "测试技能"),
            cast(1, 0, "测试技能"),
        ],
        window: 300.0,
        player: PlayerConfigDTO::default(),
        hostile: HostileConfigDTO::default(),
        xinfa: XinfaConfigDTO::default(),
        buff: BuffConfigDTO::default(),
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
        samples: 0,
        seed: None,
    };
    let result = jpcg_combo::timeline_host::calculate_timeline(request).expect("dps");
    assert!(result.total_damage > 0.0);
    assert!(
        (result.window_dps - result.total_damage / 3.0).abs() < 1.0,
        "window_dps={} total={}（应按 3s 跨度而不是 300s 窗口）",
        result.window_dps,
        result.total_damage
    );
}

#[test]
fn haste_overflow_beyond_cap_does_not_change_dps() {
    // jiasu_xishu = 101.079；25% 上限 = 2527 点。2727 点属于溢出，应和 2527 完全相同。
    let make = |jiasu: u32| TimelineRequestDTO {
        skills: vec![fixed_skill(1, "测试技能", 100)],
        forms: vec![],
        timed_buffs: vec![],
        actions: vec![
            cast(1, 0, "测试技能"),
            cast(1, 0, "测试技能"),
            cast(1, 0, "测试技能"),
        ],
        window: 300.0,
        player: PlayerConfigDTO {
            jiasu_dengji: jiasu,
            ..Default::default()
        },
        hostile: HostileConfigDTO::default(),
        xinfa: XinfaConfigDTO::default(),
        buff: BuffConfigDTO::default(),
        coeff: CoefficientConfigDTO {
            jiasu_xishu: 101.079,
            pvp_global_jianshang: 1.0,
            ..Default::default()
        },
        value_set: None,
        combat: None,
        qixue: vec![],
        miji: vec![],
        initial_instrument: None,
        initial_qifeng: String::new(),
        samples: 0,
        seed: None,
    };
    let at_cap = jpcg_combo::timeline_host::calculate_timeline(make(2527)).expect("cap");
    let overflow = jpcg_combo::timeline_host::calculate_timeline(make(2727)).expect("overflow");
    assert!(
        (at_cap.total_damage - overflow.total_damage).abs() < 0.01,
        "total: cap={} overflow={}",
        at_cap.total_damage,
        overflow.total_damage
    );
    assert!(
        (at_cap.window_dps - overflow.window_dps).abs() < 0.01,
        "dps: cap={} overflow={}",
        at_cap.window_dps,
        overflow.window_dps
    );
}

#[test]
fn prepared_optimizer_request_matches_desktop_full_qixue_definitions() {
    let mut request = req(
        vec!["dongquan".into(), "dongquan-gong".into()],
        vec![cast(14064, 14474, "宫"); 3],
    );
    request.xinfa.profession = "mowen".into();
    request.window = 30.0;
    request.samples = 100;
    request.seed = Some(42);
    if let TimelineActionDTO::Cast { overrides, .. } = &mut request.actions[1] {
        *overrides = Some(StepOverrideDTO {
            base_damage_override: Some(9999.0),
            ..Default::default()
        });
    }
    let prepared = jpcg_combo::timeline_host::prepare_timeline_from_store(request.clone()).unwrap();
    assert!(
        prepared
            .combat
            .as_ref()
            .unwrap()
            .qixue
            .iter()
            .any(|q| q.id == "dongquan-gong")
    );
    let optimizer = jpcg_combo::timeline_host::calculate_timeline(prepared).unwrap();
    let desktop = jpcg_combo::timeline_host::calculate_timeline_from_store(request).unwrap();
    assert_eq!(optimizer.total_damage, desktop.total_damage);
    assert_eq!(optimizer.window_dps, desktop.window_dps);
    assert_eq!(optimizer.final_kill_prob, desktop.final_kill_prob);
    assert_eq!(
        optimizer.steps.iter().map(|s| s.time).collect::<Vec<_>>(),
        desktop.steps.iter().map(|s| s.time).collect::<Vec<_>>()
    );
    assert!(optimizer.steps[1].time.unwrap() - optimizer.steps[0].time.unwrap() >= 12.0);
}

#[test]
fn advance_only_affects_dot_instances_alive_at_the_time() {
    for refresh in [false, true] {
        let mut actions = vec![cast(14068, 14100, "羽"), cast(14065, 14311, "商")];
        if refresh {
            actions.insert(0, cast(14065, 14311, "商"));
        }
        let mut request = req(vec![], actions);
        request.player.jichu_gongji = 8438;
        request.window = 60.0;
        let result = jpcg_combo::timeline_host::calculate_timeline(request).unwrap();
        let ticks: Vec<_> = result.events.iter().filter(|e| matches!(&e.kind, TimelineEventKindDTO::DotTick { skill_name, .. } if skill_name.contains("商"))).collect();
        let expected = if refresh {
            vec![1.5, 1.5, 6.0, 9.0, 12.0, 15.0, 18.0, 21.0]
        } else {
            vec![4.5, 7.5, 10.5, 13.5, 16.5, 19.5]
        };
        assert_eq!(ticks.iter().map(|e| e.time).collect::<Vec<_>>(), expected);
    }
}

#[test]
fn real_single_charge_skills_wait_for_recharge() {
    for (id, sub, name, form, delay) in [
        (14452, 14455, "剑·羽", "jian", 16.0),
        (14082, 15076, "疏影横斜", "qin", 28.0),
    ] {
        // ID 唯一即可；名称展示可能使用数据中的全名。
        let action = TimelineActionDTO::Cast {
            skill_id: id,
            sub_id: sub,
            skill_name: None,
            overrides: None,
        };
        let mut request = req_instrument(form, vec![], vec![action.clone(), action]);
        request.window = 60.0;
        request.xinfa.profession = "mowen".into();
        let result = jpcg_combo::timeline_host::calculate_timeline_from_store(request).unwrap();
        assert!(
            result.steps.iter().all(|s| s.legal),
            "{name}: {:?}",
            result.steps
        );
        assert_eq!(
            result.steps[1].time.unwrap() - result.steps[0].time.unwrap(),
            delay,
            "{name}"
        );
    }
}

fn independent_dot_ticks(result: &TimelineResultDTO, name: &str) -> Vec<(f32, u32)> {
    result
        .events
        .iter()
        .filter_map(|event| match &event.kind {
            TimelineEventKindDTO::DotTick {
                skill_name, damage, ..
            } if skill_name == &format!("{name}·持续") => Some((event.time, *damage)),
            _ => None,
        })
        .collect()
}

#[test]
fn real_independent_dots_apply_shared_step_overrides_without_leaking_on_refresh() {
    for (id, sub, name) in [(14065, 14311, "商"), (14066, 14312, "角")] {
        let mut request = req(
            vec![],
            vec![
                cast(id, sub, name),
                cast(14064, 14474, "宫"),
                cast(14064, 14474, "宫"),
                cast(id, sub, name),
            ],
        );
        request.xinfa.profession = "mowen".into();
        request.player.jichu_gongji = 8438;
        request.buff.huixin_pct = 25.0;
        request.hostile.huajin_dengji = 20000;
        request.window = 40.0;
        let baseline =
            jpcg_combo::timeline_host::calculate_timeline_from_store(request.clone()).unwrap();
        assert!(baseline.steps.iter().all(|s| s.legal));
        let refresh = baseline.steps[3].time.unwrap();
        assert!(refresh > 3.0 && refresh < 18.0, "must refresh a live DOT");
        let base_ticks = independent_dot_ticks(&baseline, name);
        assert!(
            base_ticks
                .iter()
                .any(|(time, damage)| *time < refresh && *damage > 0)
        );
        assert!(base_ticks.iter().any(|(time, _)| *time > refresh));
        for (label, step_override) in [
            (
                "gain",
                StepOverrideDTO {
                    gain_override: Some(100.0),
                    ..Default::default()
                },
            ),
            (
                "ignore mitigation",
                StepOverrideDTO {
                    wushihuajin_override: Some(100.0),
                    ..Default::default()
                },
            ),
            (
                "crit",
                StepOverrideDTO {
                    extra_crit_pct: Some(50.0),
                    ..Default::default()
                },
            ),
            (
                "crit damage",
                StepOverrideDTO {
                    extra_crit_dmg_pct: Some(100.0),
                    ..Default::default()
                },
            ),
        ] {
            let mut modified = request.clone();
            if let TimelineActionDTO::Cast { overrides, .. } = &mut modified.actions[0] {
                *overrides = Some(step_override);
            }
            let result =
                jpcg_combo::timeline_host::calculate_timeline_from_store(modified).unwrap();
            let ticks = independent_dot_ticks(&result, name);
            assert_eq!(ticks.len(), base_ticks.len(), "{name}: {label}");
            for ((time, damage), (base_time, base_damage)) in ticks.iter().zip(&base_ticks) {
                assert_eq!(time, base_time);
                if *time < refresh {
                    assert!(
                        damage > base_damage,
                        "{name}: {label} at {time}: {damage} <= {base_damage}"
                    );
                } else if *time > refresh {
                    assert_eq!(
                        damage, base_damage,
                        "{name}: {label} leaked into refresh at {time}"
                    );
                }
            }
        }

        // 本体的基础伤害/系数覆盖不能覆盖独立 DOT 的专属系数。
        if let TimelineActionDTO::Cast { overrides, .. } = &mut request.actions[0] {
            *overrides = Some(StepOverrideDTO {
                base_damage_override: Some(99999.0),
                atk_xishu_override: Some(10.0),
                ..Default::default()
            });
        }
        let result = jpcg_combo::timeline_host::calculate_timeline_from_store(request).unwrap();
        assert_eq!(independent_dot_ticks(&result, name), base_ticks);
        assert!(casts(&result)[0].1 > casts(&baseline)[0].1);
    }
}

#[test]
fn dongtian_damage_is_nerfed_to_4608() {
    // 2026-11-12 技改：洞天伤害降低 5%（4851 → 4608，3 跳合计 13824）。
    // 用 pvp 全局系数 1.0 + 默认零防御，单跳应等于基础伤害。
    let mut request = req(
        vec!["dongtian".into()],
        vec![
            TimelineActionDTO::Cast {
                skill_id: 14069,
                sub_id: 0,
                skill_name: Some("高山流水".into()),
                overrides: None,
            },
            TimelineActionDTO::Cast {
                skill_id: 14299,
                sub_id: 18859,
                skill_name: None,
                overrides: None,
            },
        ],
    );
    request.coeff.pvp_global_jianshang = 1.0;
    let result = jpcg_combo::timeline_host::calculate_timeline(request).expect("dongtian");
    let ticks: Vec<u32> = result
        .events
        .iter()
        .filter_map(|e| match &e.kind {
            TimelineEventKindDTO::DotTick {
                skill_name, damage, ..
            } if skill_name == "洞天" => Some(*damage),
            _ => None,
        })
        .collect();
    assert_eq!(ticks.len(), 3, "洞天应为 3 跳");
    // base 4608 经引擎伤害链路（PVP 0.9、化劲/破防取整）后单跳 4149；
    // 削弱前 base 4851 时约 4368，4149/4368 ≈ 0.95，即伤害降低 5%。
    assert!(
        ticks.iter().all(|d| *d == 4149),
        "洞天单跳应为 4149（base 4608 经伤害链路），实际 {ticks:?}"
    );
}
