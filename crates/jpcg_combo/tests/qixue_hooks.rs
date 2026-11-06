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
