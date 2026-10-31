#![cfg(feature = "engine")]
use jpcg_api::{
    AttributeConversion, AttributeDamageKind as Kind, AttributeRatio, AttributeRules,
    AttributeTarget as Target, BasicAttribute as Basic,
};
use jpcg_gear::{
    attributes::{PROFESSION_TOML, convert},
    *,
};

const VALUE: &str = "cszj-exp-260908";

fn profile(main: MainStat) -> School {
    let damage = if matches!(main, MainStat::Liliang | MainStat::Shenfa) {
        DamageType::Waigong
    } else {
        DamageType::Neigong
    };
    School {
        id: "mowen".into(),
        zhushuxing: main,
        gongji_type: damage,
        pofang_type: damage,
    }
}

// Deliberately artificial coefficients: tests the pipeline, NOT a level50 game rule.
fn rules(main: Basic, kind: Kind) -> AttributeRules {
    AttributeRules {
        id: "synthetic-50-handfilled-v1".into(),
        level: 50,
        value_set: VALUE.into(),
        source: "synthetic test arithmetic; not game data".into(),
        main_stat: main,
        damage_kind: kind,
        qixue_multiplier: AttributeRatio {
            numerator: 3,
            denominator: 2,
        },
        conversions: vec![
            AttributeConversion {
                from: Basic::Tizhi,
                to: Target::QixueBase,
                numerator: 7,
                denominator: 1,
            },
            AttributeConversion {
                from: main,
                to: Target::GongjiExtra,
                numerator: 5,
                denominator: 2,
            },
            AttributeConversion {
                from: main,
                to: Target::Huixin,
                numerator: 9,
                denominator: 10,
            },
            AttributeConversion {
                from: main,
                to: Target::Huixin,
                numerator: 2,
                denominator: 5,
            },
            AttributeConversion {
                from: Basic::Yuanqi,
                to: Target::GongjiBase,
                numerator: 1,
                denominator: 5,
            },
            AttributeConversion {
                from: Basic::Liliang,
                to: Target::Pofang,
                numerator: 3,
                denominator: 10,
            },
        ],
    }
}

#[test]
fn handfilled_50_rules_cover_all_main_attributes_without_school_constants() {
    for (main, basic, kind) in [
        (MainStat::Liliang, Basic::Liliang, Kind::Physical),
        (MainStat::Shenfa, Basic::Shenfa, Kind::Physical),
        (MainStat::Gengu, Basic::Gengu, Kind::Lunar),
        (MainStat::Yuanqi, Basic::Yuanqi, Kind::Neutral),
    ] {
        let p = profile(main);
        let r = rules(basic, kind);
        let stats = Stats {
            zhushuxing: 1,
            liliang: 100,
            shenfa: 100,
            gengu: 100,
            yuanqi: 100,
            tizhi: 3,
            gongji: 1000,
            huixin: 2000,
            pofang: 3000,
            qixue: 100,
            qixue_additional: 11,
            ..Default::default()
        };
        let out = convert(&stats, &p, VALUE, &r).unwrap();
        assert_eq!(out.main_value(main), 101);
        assert_eq!(out.gongji_extra, 252);
        assert_eq!(out.gongji_base, 1020);
        assert_eq!(out.huixin, 2130);
        assert_eq!(out.pofang, 3030);
        assert_eq!(
            (out.qixue_base, out.qixue_additional, out.qixue),
            (181, 11, 192)
        );
        assert_eq!((out.level, out.value_set.as_str()), (50, VALUE));
        assert!(!out.verified_in_game);
    }
}

#[test]
fn independent_rounding_alias_and_late_hp_stages() {
    let mut r = rules(Basic::Gengu, Kind::Lunar);
    let p = profile(MainStat::Gengu);
    let stats = Stats {
        gengu: 1,
        ..Default::default()
    };
    let out = convert(&stats, &p, VALUE, &r).unwrap();
    assert_eq!(out.huixin, 0); // floor(.9)+floor(.4), never floor(1.3).
    let aliased = Stats {
        zhushuxing: 1,
        gengu: 1,
        ..Default::default()
    };
    let out = convert(&aliased, &p, VALUE, &r).unwrap();
    assert_eq!((out.gengu, out.gongji_extra, out.huixin), (2, 5, 1));
    r.conversions.push(AttributeConversion {
        from: Basic::Tizhi,
        to: Target::QixueAdditional,
        numerator: 2,
        denominator: 1,
    });
    let out = convert(
        &Stats {
            tizhi: 1,
            qixue: 1,
            qixue_additional: 10,
            ..Default::default()
        },
        &p,
        VALUE,
        &r,
    )
    .unwrap();
    assert_eq!(
        (out.qixue_base, out.qixue_additional, out.qixue),
        (12, 12, 24)
    );
}

#[test]
fn invalid_metadata_ratio_and_overflow_are_errors() {
    let p = profile(MainStat::Gengu);
    let r = rules(Basic::Gengu, Kind::Lunar);
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
            yuanqi: 10,
            ..Default::default()
        },
        Stats {
            huixin: u32::MAX,
            gengu: 10,
            ..Default::default()
        },
        Stats {
            gengu: u32::MAX,
            zhushuxing: 1,
            ..Default::default()
        },
        Stats {
            qixue: 1,
            qixue_additional: u32::MAX,
            ..Default::default()
        },
    ] {
        assert!(convert(&stats, &p, VALUE, &r).unwrap_err().contains("溢出"));
    }
    for field in [
        "level",
        "source",
        "main_stat",
        "damage_kind",
        "qixue_multiplier",
        "conversions",
    ] {
        let mut bad = r.clone();
        match field {
            "level" => bad.level = 0,
            "source" => bad.source.clear(),
            "main_stat" => bad.main_stat = Basic::Shenfa,
            "damage_kind" => bad.damage_kind = Kind::Physical,
            "qixue_multiplier" => bad.qixue_multiplier.denominator = 0,
            _ => bad.conversions[0].denominator = 0,
        }
        assert!(
            convert(&Stats::default(), &p, VALUE, &bad).is_err(),
            "{field}"
        );
    }
    assert!(convert(&Stats::default(), &p, "live-130", &r).is_err());
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
fn explicit_zero_rules_do_not_invent_generic_coefficients() {
    let mut r = rules(Basic::Gengu, Kind::Poison);
    r.conversions.clear();
    r.qixue_multiplier = AttributeRatio {
        numerator: 1,
        denominator: 1,
    };
    let out = convert(
        &Stats {
            gengu: 100,
            tizhi: 100,
            yuanqi: 100,
            qixue: 2,
            ..Default::default()
        },
        &profile(MainStat::Gengu),
        VALUE,
        &r,
    )
    .unwrap();
    assert_eq!(
        (out.gongji, out.huixin, out.pofang, out.qixue),
        (0, 0, 0, 2)
    );
}

#[test]
fn old_inputs_remain_legacy_and_historical_ids_cannot_silently_mix_versions() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/gear");
    let c = Catalog::load(&root).unwrap();
    let mut l: Loadout =
        toml::from_str(&std::fs::read_to_string(root.join("sample-loadout.toml")).unwrap())
            .unwrap();
    let out = host::compose(&c, &l).unwrap();
    assert!(out.composed.panel.is_none());
    assert_eq!(out.player.zhuanhua_gongji, None);
    assert_eq!(out.player.jiasu_dengji, out.composed.stats.jiasu);
    l.base.tizhi = 1;
    assert!(c.compose(&l).unwrap_err().contains("attribute_rules"));
    l.attribute_rules = Some("jx3-130-mowen-20260814".into());
    assert!(c.compose(&l).unwrap_err().contains("不再支持"));
}

#[test]
fn profession_toml_50_flows_through_host_optimizer_core_and_roundtrip() {
    // Process isolation protects core value-set caches and all parallel tests from env mutation.
    if std::env::var_os("JPCG_ATTR_50_CHILD").is_none() {
        assert!(
            std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "profession_toml_50_flows_through_host_optimizer_core_and_roundtrip"
                ])
                .env("JPCG_ATTR_50_CHILD", "1")
                .status()
                .unwrap()
                .success()
        );
        return;
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let temp = tempfile::tempdir().unwrap();
    for dir in ["gear", "shuxing", "values"] {
        std::fs::create_dir(temp.path().join(dir)).unwrap();
    }
    for f in ["index.toml", "cszj-exp-260908.toml", "live-130.toml"] {
        std::fs::copy(
            root.join("values").join(f),
            temp.path().join("values").join(f),
        )
        .unwrap();
    }
    for f in [
        "schools.toml",
        "templates.toml",
        "sets.toml",
        "overrides.toml",
    ] {
        let text = std::fs::read_to_string(root.join("gear").join(f))
            .unwrap()
            .replace("live-130", VALUE);
        std::fs::write(temp.path().join("gear").join(f), text).unwrap();
    }
    let base = std::fs::read_to_string(root.join("shuxing/mowen.toml"))
        .unwrap()
        .replace("level = 130", "level = 50");
    let mut school: toml::Value = toml::from_str(&base).unwrap();
    let mut r = rules(Basic::Gengu, Kind::Lunar);
    school
        .as_table_mut()
        .unwrap()
        .insert("attributes".into(), toml::Value::try_from(&r).unwrap());
    let file = temp.path().join("shuxing/mowen.toml");
    std::fs::write(&file, toml::to_string(&school).unwrap()).unwrap();
    // SAFETY: subprocess runs only this one test; no other threads access environment.
    unsafe {
        std::env::set_var("JPCG_DATA_DIR", temp.path());
    }
    let c = host::catalog().unwrap();
    let mut req: OptimizeRequest = toml::from_str(
        &std::fs::read_to_string(root.join("gear/sample-request.toml"))
            .unwrap()
            .replace("live-130", VALUE),
    )
    .unwrap();
    req.loadout.attribute_rules = Some(PROFESSION_TOML.into());
    req.loadout.base.tizhi = 100;
    req.loadout.base.yuanqi = 1001;
    req.loadout.base.qixue_additional = 123;
    let out = host::compose(&c, &req.loadout).unwrap();
    let panel = out.composed.panel.as_ref().unwrap();
    let raw = &out.composed.stats;
    assert_eq!(panel.qixue_base, (raw.qixue + raw.tizhi * 7) * 3 / 2);
    assert_eq!(panel.qixue, panel.qixue_base + 123);
    assert_eq!(out.player.huixin_dengji, panel.huixin);
    assert_eq!(out.player.zhuanhua_gongji, Some(panel.gongji_extra));
    let core = jpcg_core::type_set::player::PlayerConfig::from(out.player.clone());
    assert_eq!(core.atk_with_buff(999.0, 0.0).total(), panel.gongji);
    let text = host::dispatch(&c, "export", &serde_json::to_string(&req.loadout).unwrap()).unwrap();
    let restored =
        host::dispatch(&c, "import", &serde_json::json!({"toml":text}).to_string()).unwrap();
    assert_eq!(restored, serde_json::to_value(&out).unwrap());
    req.constraints.stats.insert(
        "qixue".into(),
        Bounds {
            min: Some(panel.qixue),
            max: None,
        },
    );
    req.constraints.stats.insert(
        "huixin".into(),
        Bounds {
            min: Some(raw.huixin + 1),
            max: None,
        },
    );
    let result = optimize(&c, &req, |_| true).unwrap();
    let best = result.best.unwrap();
    req.config.player = player(&best.composed);
    let damage = jpcg_core::host::calc::calculate(req.config).unwrap();
    assert_eq!(
        best.score,
        damage.iter().find(|s| s.skill_name == "徵").unwrap().q as f64
    );
    // Editing only the profession TOML changes the next composition, no rebuild/global constant.
    r.id = "synthetic-edited-v2".into();
    r.conversions[2].numerator = 19;
    school
        .as_table_mut()
        .unwrap()
        .insert("attributes".into(), toml::Value::try_from(&r).unwrap());
    std::fs::write(&file, toml::to_string(&school).unwrap()).unwrap();
    let changed = host::compose(&c, &req.loadout).unwrap();
    assert!(changed.player.huixin_dengji > out.player.huixin_dengji);
    assert_eq!(changed.composed.panel.unwrap().rule_id, r.id);
    // A 130-level selected snapshot cannot consume the same 50-level profession rule.
    let mut wrong = c.clone();
    wrong.version.value_set = "live-130".into();
    req.loadout.value_set = "live-130".into();
    assert!(
        host::compose(&wrong, &req.loadout)
            .unwrap_err()
            .contains("数值集")
    );
    req.loadout.value_set = VALUE.into();
    school.as_table_mut().unwrap().remove("attributes");
    std::fs::write(&file, toml::to_string(&school).unwrap()).unwrap();
    assert!(host::compose(&c, &req.loadout).is_err());
}
