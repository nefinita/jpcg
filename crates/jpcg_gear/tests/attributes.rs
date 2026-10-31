#![cfg(feature = "engine")]
use jpcg_gear::{
    attributes::{MOWEN_130, convert},
    *,
};

fn sample() -> (Catalog, Loadout, OptimizeRequest) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/gear");
    (
        Catalog::load(&dir).unwrap(),
        toml::from_str(&std::fs::read_to_string(dir.join("sample-loadout.toml")).unwrap()).unwrap(),
        toml::from_str(&std::fs::read_to_string(dir.join("sample-request.toml")).unwrap()).unwrap(),
    )
}

#[test]
fn pinned_rule_arithmetic_fixtures_are_not_game_panel_claims() {
    let (catalog, _, _) = sample();
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/attributes-130-mowen.json")).unwrap();
    for case in fixtures["cases"].as_array().unwrap() {
        let stats: Stats = serde_json::from_value(case["input"].clone()).unwrap();
        let panel = convert(&stats, &catalog.schools[0], "live-130", MOWEN_130).unwrap();
        assert_eq!(
            serde_json::json!([
                panel.gongji_base,
                panel.gongji_extra,
                panel.huixin,
                panel.pofang,
                panel.qixue
            ]),
            case["expected"],
            "{}",
            case["name"]
        );
        assert!(!panel.verified_in_game);
        assert!(
            panel
                .source
                .contains(fixtures["source_commit"].as_str().unwrap())
        );
    }
}

#[test]
fn aliases_sum_once_and_each_conversion_truncates_independently() {
    let (c, _, _) = sample();
    let stats = Stats {
        zhushuxing: 1,
        gengu: 1,
        ..Default::default()
    };
    let p = convert(&stats, &c.schools[0], "live-130", MOWEN_130).unwrap();
    assert_eq!((p.gengu, p.huixin, p.gongji_extra), (2, 1, 3));
    let p = convert(
        &Stats {
            gengu: 1,
            ..Default::default()
        },
        &c.schools[0],
        "live-130",
        MOWEN_130,
    )
    .unwrap();
    assert_eq!(p.huixin, 0); // floor(.9)+floor(410/1024), not floor(.9+410/1024)=1
    let p = convert(
        &Stats {
            yuanqi: 1000,
            ..Default::default()
        },
        &c.schools[0],
        "live-130",
        MOWEN_130,
    )
    .unwrap();
    assert_eq!((p.gongji_base, p.pofang), (181, 300));
}

#[test]
fn unsupported_context_and_overflow_are_errors_not_fallbacks() {
    let (c, mut l, _) = sample();
    let profile = &c.schools[0];
    assert!(convert(&Stats::default(), profile, "cszj-exp-260908", MOWEN_130).is_err());
    assert!(convert(&Stats::default(), profile, "live-130", "latest").is_err());
    let mut other = profile.clone();
    other.id = "bingxin".into();
    assert!(convert(&Stats::default(), &other, "live-130", MOWEN_130).is_err());
    for stats in [
        Stats {
            gengu: u32::MAX,
            ..Default::default()
        },
        Stats {
            tizhi: u32::MAX,
            ..Default::default()
        },
        Stats {
            qixue: u32::MAX,
            ..Default::default()
        },
        Stats {
            gongji: u32::MAX,
            yuanqi: 1000,
            ..Default::default()
        },
        Stats {
            huixin: u32::MAX,
            gengu: 1000,
            ..Default::default()
        },
        Stats {
            gengu: u32::MAX,
            zhushuxing: 1,
            ..Default::default()
        },
    ] {
        assert!(
            convert(&stats, profile, "live-130", MOWEN_130)
                .unwrap_err()
                .contains("溢出")
        );
    }
    l.base.tizhi = 1;
    assert!(c.compose(&l).unwrap_err().contains("attribute_rules"));
    l.attribute_rules = Some("unknown".into());
    assert!(c.compose(&l).unwrap_err().contains("未知"));
    let mut s = Stats {
        tizhi: u32::MAX,
        ..Default::default()
    };
    assert!(
        s.add(&Stats {
            tizhi: 1,
            ..Default::default()
        })
        .is_err()
    );
}

#[test]
fn raw_base_templates_and_sets_convert_once_through_host_and_toml() {
    let (mut c, mut l, _) = sample();
    l.attribute_rules = Some(MOWEN_130.into());
    l.base.tizhi = 100;
    l.base.yuanqi = 1000;
    c.templates.iter_mut().for_each(|t| t.stats.tizhi = 2);
    c.sets[0].tiers.iter_mut().for_each(|t| t.bonus2.tizhi = 3);
    let out = host::compose(&c, &l).unwrap();
    let raw = &out.composed.stats;
    assert!(raw.tizhi >= 124);
    let panel = out.composed.panel.as_ref().unwrap();
    assert_eq!(
        panel.qixue,
        ((u64::from(raw.qixue) + 10 * u64::from(raw.tizhi)) * 1331 / 1024) as u32
    );
    assert_eq!(out.player.huixin_dengji, panel.huixin);
    assert_eq!(out.player.jiasu_dengji, raw.jiasu);
    assert_eq!(out.player.zhuanhua_gongji, Some(panel.gongji_extra));
    let player = jpcg_core::type_set::player::PlayerConfig::from(out.player.clone());
    assert_eq!(player.atk_with_buff(999.0, 0.0).total(), panel.gongji);
    assert_eq!(
        player.atk_with_buff(999.0, 25.0).total(),
        (panel.gongji_base as f32 * 1.25) as u32 + panel.gongji_extra
    );
    let input = serde_json::to_string(&l).unwrap();
    let exported = host::dispatch(&c, "export", &input).unwrap();
    let imported = host::dispatch(
        &c,
        "import",
        &serde_json::json!({"toml":exported}).to_string(),
    )
    .unwrap();
    assert_eq!(imported, serde_json::to_value(out).unwrap());
}

#[test]
fn hp_constraints_use_derived_health_without_changing_legacy_constraints() {
    let (c, _, mut req) = sample();
    req.loadout.attribute_rules = Some(MOWEN_130.into());
    req.loadout.base.tizhi = 100;
    req.constraints.stats.insert(
        "qixue".into(),
        Bounds {
            min: Some(130_000),
            max: None,
        },
    );
    let out = optimize_with(&c, &req, |_| Ok(1.0), |_| true).unwrap();
    let best = out.best.unwrap();
    assert!(best.composed.stats.qixue < 130_000);
    assert!(best.composed.panel.as_ref().unwrap().qixue >= 130_000);
    req.loadout.attribute_rules = None;
    req.loadout.base.tizhi = 0;
    assert!(
        optimize_with(&c, &req, |_| Ok(1.0), |_| true)
            .unwrap()
            .best
            .is_none()
    );
}

#[test]
fn optimizer_converted_candidate_matches_core_host() {
    if std::env::var_os("JPCG_ATTR_TEST_CHILD").is_none() {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "optimizer_converted_candidate_matches_core_host"])
            .env("JPCG_ATTR_TEST_CHILD", "1")
            .env(
                "JPCG_DATA_DIR",
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data"),
            )
            .status()
            .unwrap();
        assert!(status.success());
        return;
    }
    let (c, _, mut req) = sample();
    req.loadout.attribute_rules = Some(MOWEN_130.into());
    req.loadout.base.tizhi = 100;
    req.loadout.base.yuanqi = 1001;
    req.config.buff.base_atk_pct = 25.0;
    let out = optimize(&c, &req, |_| true).unwrap();
    let best = out.best.unwrap();
    req.config.player = player(&best.composed);
    let core = jpcg_core::host::calc::calculate(req.config).unwrap();
    assert_eq!(
        best.score,
        core.iter().find(|s| s.skill_name == "徵").unwrap().q as f64
    );
}

#[test]
fn converted_constraints_cannot_pass_a_critical_cap_using_raw_rating() {
    let (c, _, mut req) = sample();
    req.loadout.attribute_rules = Some(MOWEN_130.into());
    let baseline = c.compose(&req.loadout).unwrap();
    let raw = baseline.stats.huixin;
    assert!(baseline.panel.unwrap().huixin > raw + 1000);
    req.constraints.stats.insert(
        "huixin".into(),
        Bounds {
            min: None,
            max: Some(raw + 1000),
        },
    );
    assert!(
        optimize_with(&c, &req, |_| Ok(1.0), |_| true)
            .unwrap()
            .best
            .is_none()
    );
    req.constraints.stats.insert(
        "huixin".into(),
        Bounds {
            min: Some(raw + 1000),
            max: None,
        },
    );
    assert!(
        optimize_with(&c, &req, |_| Ok(1.0), |_| true)
            .unwrap()
            .best
            .is_some()
    );
}

#[test]
fn three_more_school_snapshots_cover_all_primary_attributes() {
    use jpcg_gear::attributes::{AOXUE_130, FENSHAN_130, HUAJIAN_130};
    // Expected values executed independently in pinned upstream BaseAttribute.
    for (id, main, damage, rule, expected) in [
        (
            "10026",
            MainStat::Liliang,
            DamageType::Waigong,
            AOXUE_130,
            [10163, 1801, 20900, 30580],
        ),
        (
            "10390",
            MainStat::Shenfa,
            DamageType::Waigong,
            FENSHAN_130,
            [10163, 1881, 20900, 30300],
        ),
        (
            "10021",
            MainStat::Yuanqi,
            DamageType::Neigong,
            HUAJIAN_130,
            [10181, 2032, 20900, 30500],
        ),
    ] {
        let profile = School {
            id: id.into(),
            zhushuxing: main,
            gongji_type: damage,
            pofang_type: damage,
        };
        let stats = Stats {
            tizhi: 1001,
            liliang: 1001,
            shenfa: 1001,
            gengu: 1001,
            yuanqi: 1001,
            gongji: 10000,
            huixin: 20000,
            pofang: 30000,
            qixue: 100000,
            qixue_additional: 123,
            ..Default::default()
        };
        let p = convert(&stats, &profile, "live-130", rule).unwrap();
        assert_eq!(
            [p.gongji_base, p.gongji_extra, p.huixin, p.pofang],
            expected,
            "{id}"
        );
        assert_eq!(
            (p.qixue_base, p.qixue_additional, p.qixue),
            (142991, 123, 143114)
        );
        assert_eq!(p.main_value(main), 1001);
        assert_eq!(
            p.damage_kind,
            if damage == DamageType::Waigong {
                "physical"
            } else {
                "neutral"
            }
        );
        // Same final main value supplied wholly through legacy alias must match named attribute.
        let mut aliased = stats.clone();
        match main {
            MainStat::Liliang => aliased.liliang = 0,
            MainStat::Shenfa => aliased.shenfa = 0,
            MainStat::Yuanqi => aliased.yuanqi = 0,
            _ => unreachable!(),
        }
        aliased.zhushuxing = 1001;
        assert_eq!(convert(&aliased, &profile, "live-130", rule).unwrap(), p);
        for v in [1, 3] {
            let tiny = Stats {
                liliang: v,
                shenfa: v,
                gengu: v,
                yuanqi: v,
                tizhi: v,
                ..Default::default()
            };
            let p = convert(&tiny, &profile, "live-130", rule).unwrap();
            assert_eq!(p.huixin, if v == 1 { 0 } else { 2 });
            assert_eq!(p.pofang, 0);
            assert_eq!(
                p.gongji,
                if id == "10021" {
                    if v == 1 { 2 } else { 6 }
                } else if v == 1 {
                    1
                } else {
                    5
                }
            );
        }
        let mut wrong = profile.clone();
        wrong.zhushuxing = MainStat::Gengu;
        assert!(convert(&stats, &wrong, "live-130", rule).is_err());
    }
}

#[test]
fn extra_health_is_added_after_base_multiplier_and_checked_for_overflow() {
    let (c, mut l, _) = sample();
    let stats = Stats {
        qixue: 1,
        qixue_additional: 10,
        ..Default::default()
    };
    let p = convert(&stats, &c.schools[0], "live-130", MOWEN_130).unwrap();
    assert_eq!((p.qixue_base, p.qixue), (1, 11)); // not floor(11*1331/1024)=14
    let too_large = Stats {
        qixue: 1,
        qixue_additional: u32::MAX,
        ..Default::default()
    };
    assert!(convert(&too_large, &c.schools[0], "live-130", MOWEN_130).is_err());
    l.base.qixue_additional = 1;
    assert!(c.compose(&l).unwrap_err().contains("attribute_rules"));
}

#[test]
fn additional_school_compose_host_preserves_type_and_primary_attribute() {
    use jpcg_gear::attributes::{AOXUE_130, FENSHAN_130, HUAJIAN_130};
    for (id, main, damage, rule) in [
        ("10026", MainStat::Liliang, DamageType::Waigong, AOXUE_130),
        ("10390", MainStat::Shenfa, DamageType::Waigong, FENSHAN_130),
        ("10021", MainStat::Yuanqi, DamageType::Neigong, HUAJIAN_130),
    ] {
        // Explicitly synthetic mechanics fixture, no new game equipment catalog.
        let (mut c, mut l, _) = sample();
        c.schools[0] = School {
            id: id.into(),
            zhushuxing: main,
            gongji_type: damage,
            pofang_type: damage,
        };
        for t in &mut c.templates {
            t.school = id.into();
        }
        l.school = id.into();
        l.attribute_rules = Some(rule.into());
        l.base.qixue_additional = 123;
        let out = host::compose(&c, &l).unwrap();
        let p = out.composed.panel.as_ref().unwrap();
        assert_eq!(out.player.jichu_shuxing, p.main_value(main));
        assert!(out.player.jichu_shuxing > 0);
        assert_eq!(out.player.huixin_dengji, p.huixin);
        assert_eq!(out.player.pofang_dengji, p.pofang);
        assert_eq!(out.player.zhuanhua_gongji, Some(p.gongji_extra));
        let core = jpcg_core::type_set::player::PlayerConfig::from(out.player);
        assert_eq!(core.atk_with_buff(999.0, 0.0).total(), p.gongji);
        let back: Loadout = toml::from_str(&toml::to_string(&l).unwrap()).unwrap();
        assert_eq!(c.compose(&back).unwrap().panel, out.composed.panel);
    }
}
