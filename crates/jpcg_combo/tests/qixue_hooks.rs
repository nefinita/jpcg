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
    // 变徵（14299/18859）默认是 84-92 ×3；洞天替换成 4851 ×3 固定伤害
    let switch = || TimelineActionDTO::Cast {
        skill_id: 14069,
        sub_id: 0,
        skill_name: Some("高山流水".into()),
    };
    let base = jpcg_combo::timeline_host::calculate_timeline(req(
        vec![],
        vec![
            switch(),
            TimelineActionDTO::Cast {
                skill_id: 14299,
                sub_id: 18859,
                skill_name: None,
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
    // 洞天：变徵被替换为 3 跳 4851，每次跳让商/角下一跳立刻生效（提前，不补跳）
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
    // 连续两次洞天（刷新）时，第一次已生效的跳不能被丢掉，序号继续累加
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
            TimelineEventKindDTO::Cast { skill_name, .. } if skill_name.starts_with("宫") => {
                Some(e.time)
            }
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
