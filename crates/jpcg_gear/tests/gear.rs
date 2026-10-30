use jpcg_gear::*;
use std::path::PathBuf;

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/gear")
}
fn catalog() -> Catalog {
    Catalog::load(&dir()).expect("sample catalog")
}
fn loadout() -> Loadout {
    toml::from_str(&std::fs::read_to_string(dir().join("sample-loadout.toml")).unwrap()).unwrap()
}
fn request() -> OptimizeRequest {
    toml::from_str(&std::fs::read_to_string(dir().join("sample-request.toml")).unwrap()).unwrap()
}
fn select(l: &mut Loadout, slot: Slot, source: Source, tier: Tier) {
    let c = l.choices.iter_mut().find(|c| c.slot == slot).unwrap();
    c.source = source;
    c.tier = tier;
}

// 子进程设置数据目录，避免 edition 2024 下 unsafe set_var 与全局环境竞争；CI 无需外部 env。
fn runtime_child(name: &str) -> bool {
    if std::env::var_os("JPCG_GEAR_TEST_CHILD").is_some() {
        return true;
    }
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name])
        .env("JPCG_GEAR_TEST_CHILD", "1")
        .env("JPCG_DATA_DIR", dir().parent().unwrap())
        .status()
        .unwrap();
    assert!(status.success(), "runtime child failed: {name}");
    false
}

fn evaluate(c: &Composed) -> Result<f64, String> {
    Ok(c.stats.gongji as f64 + c.stats.pofang as f64)
}

#[test]
fn sample_roundtrips_and_maps_school() {
    let c = catalog();
    let l = loadout();
    let a = c.compose(&l).unwrap();
    assert_eq!(c.templates.len(), 56);
    assert_eq!(a.stats.gongji, 40000 + 11 * 20 + 30 + 13);
    assert_eq!(a.stats.qixue, 101200);
    assert_eq!(a.stats.huajin, 7);
    assert_eq!(a.stats.yujin, 15 + 9);
    assert_eq!(a.cooldowns.len(), 1);
    assert_eq!(a.active_bonuses.len(), 4); // 防具2/4、饰品2/3
    assert_eq!(player(&a).jcsx, "根骨");
    assert_eq!(a.profile.gongji_type, DamageType::Neigong);
    let back: Loadout = toml::from_str(&toml::to_string_pretty(&l).unwrap()).unwrap();
    assert_eq!(c.compose(&back).unwrap().stats, a.stats);
    assert!(a.warnings.iter().any(|s| s.contains("样例")));
}
#[test]
fn cross_source_2plus2_stacks_once_and_pants_do_not_count() {
    let c = catalog();
    let mut l = loadout();
    for slot in [Slot::Tou, Slot::Shou] {
        select(&mut l, slot, Source::Mingjian, Tier::Base);
    }
    select(&mut l, Slot::Xie, Source::Baowu, Tier::Base);
    let out = c.compose(&l).unwrap();
    assert_eq!(
        out.active_bonuses
            .iter()
            .filter(|b| b.threshold == 2)
            .count(),
        3
    ); // +饰品
    assert!(out.cooldowns.is_empty());
    assert_eq!(
        out.active_bonuses
            .iter()
            .find(|b| b.set == "sample-weiwang")
            .unwrap()
            .pieces,
        2
    );
    select(&mut l, Slot::Ku, Source::Mingjian, Tier::Base);
    let changed = c.compose(&l).unwrap();
    assert_eq!(
        changed
            .active_bonuses
            .iter()
            .find(|b| b.set == "sample-mingjian")
            .unwrap()
            .pieces,
        2
    );
}
#[test]
fn four_same_source_never_gives_two_two_piece_rewards() {
    let c = catalog();
    let out = c.compose(&loadout()).unwrap();
    assert_eq!(
        out.active_bonuses
            .iter()
            .filter(|b| b.set == "sample-weiwang" && b.threshold == 2)
            .count(),
        1
    );
}
#[test]
fn mixed_upgrade_thresholds_are_independent() {
    let c = catalog();
    let mut l = loadout();
    for slot in [Slot::Tou, Slot::Yi, Slot::Xianglian, Slot::Yaozhui] {
        select(&mut l, slot, Source::Weiwang, Tier::Upgraded);
    }
    let out = c.compose(&l).unwrap();
    for b in &out.active_bonuses {
        assert_eq!(
            b.tier,
            if b.threshold == 2 {
                Tier::Upgraded
            } else {
                Tier::Base
            }
        );
    }
    assert_eq!(out.stats.huajin, 14);
    assert_eq!(out.stats.yujin, 24);
    assert_eq!(out.cooldowns[0].seconds, 1.0);
    select(&mut l, Slot::Jie1, Source::Weiwang, Tier::Upgraded);
    assert_eq!(c.compose(&l).unwrap().stats.yujin, 33);
}
#[test]
fn accessory_variants_mix_freely() {
    let c = catalog();
    let a = c.compose(&loadout()).unwrap();
    let mut l = loadout();
    l.choices
        .iter_mut()
        .find(|c| c.slot == Slot::Jie2)
        .unwrap()
        .variant = 2;
    let b = c.compose(&l).unwrap();
    assert_eq!(a.stats.huixin, b.stats.huixin + 15);
    assert_eq!(a.stats.pofang + 15, b.stats.pofang);
    assert_eq!(a.stats.huajin, b.stats.huajin);
}
#[test]
fn invalid_catalog_rules_are_rejected() {
    type Mutation = Box<dyn Fn(&mut Catalog)>;
    let mutations: Vec<Mutation> = vec![
        Box::new(|c| c.templates.push(c.templates[0].clone())),
        Box::new(|c| c.templates[0].choice.variant = 1),
        Box::new(|c| c.templates[0].set = Some("missing".into())),
        Box::new(|c| c.templates[0].school = "missing".into()),
        Box::new(|c| {
            c.templates
                .iter_mut()
                .find(|t| t.choice.slot == Slot::Ku)
                .unwrap()
                .set = Some("sample-mingjian".into())
        }),
        Box::new(|c| {
            c.templates
                .iter_mut()
                .find(|t| t.choice.source == Source::Baowu)
                .unwrap()
                .choice
                .tier = Tier::Upgraded
        }),
        Box::new(|c| {
            c.templates
                .iter_mut()
                .find(|t| t.choice.slot.accessory())
                .unwrap()
                .choice
                .source = Source::Mingjian
        }),
        Box::new(|c| c.templates[0].green = Green::Yujin),
        Box::new(|c| c.sets.push(c.sets[0].clone())),
        Box::new(|c| c.sets[0].source = Source::Baowu),
        Box::new(|c| c.sets[0].tiers[0].cooldown4.clear()),
        Box::new(|c| c.sets[0].tiers[0].bonus2 = Stats::default()),
        Box::new(|c| c.sets[2].tiers[0].bonus2.huajin = 0),
        Box::new(|c| c.sets[2].tiers[0].bonus3.as_mut().unwrap().yujin = 0),
        Box::new(|c| c.version.schema = 2),
        Box::new(|c| c.templates.retain(|t| t.choice.slot != Slot::Anqi)),
        Box::new(|c| {
            c.templates
                .retain(|t| !(t.choice.slot == Slot::Jie1 && t.choice.variant == 2))
        }),
    ];
    for mutate in mutations {
        let mut c = catalog();
        mutate(&mut c);
        assert!(c.validate().is_err());
    }
}
#[test]
fn compose_rejects_missing_duplicate_stale_and_overflow() {
    let c = catalog();
    let mut l = loadout();
    l.choices.pop();
    assert!(c.compose(&l).is_err());
    let mut l = loadout();
    l.choices[0] = l.choices[1].clone();
    assert!(c.compose(&l).is_err());
    let mut l = loadout();
    l.revision = "old".into();
    assert!(c.compose(&l).is_err());
    let mut l = loadout();
    l.value_set = "other".into();
    assert!(c.compose(&l).is_err());
    let mut l = loadout();
    l.base.gongji = u32::MAX;
    assert!(c.compose(&l).is_err());
}
#[test]
fn overrides_apply_explicit_higher_attack_and_versions_are_checked() {
    let temp = tempfile::tempdir().unwrap();
    for file in [
        "templates.toml",
        "sets.toml",
        "schools.toml",
        "overrides.toml",
    ] {
        std::fs::copy(dir().join(file), temp.path().join(file)).unwrap();
    }
    let raw = std::fs::read_to_string(temp.path().join("overrides.toml"))
        .unwrap()
        .replace("overrides = []", "");
    let extra = r#"
[[overrides]]
school="mowen"
[overrides.choice]
slot="shou"
source="weiwang"
tier="base"
variant=0
[overrides.stats]
gongji=100
qixue=100
yujin=15
zhushuxing=10
"#;
    std::fs::write(temp.path().join("overrides.toml"), format!("{raw}{extra}")).unwrap();
    let overridden = Catalog::load(temp.path()).unwrap();
    assert_eq!(
        overridden.compose(&loadout()).unwrap().stats.gongji,
        catalog().compose(&loadout()).unwrap().stats.gongji + 70
    );
    std::fs::write(
        temp.path().join("schools.toml"),
        std::fs::read_to_string(dir().join("schools.toml"))
            .unwrap()
            .replace("synthetic-v1", "old"),
    )
    .unwrap();
    assert!(Catalog::load(temp.path()).is_err());
}
#[test]
fn optimizer_obeys_hard_constraints_and_finds_exhaustive_best() {
    let c = catalog();
    let mut r = request();
    r.constraints.locked.push(
        r.loadout
            .choices
            .iter()
            .find(|c| c.slot == Slot::Jie1)
            .unwrap()
            .clone(),
    );
    r.constraints.stats.insert(
        "huajin".into(),
        Bounds {
            min: Some(7),
            max: Some(7),
        },
    );
    let out = optimize_with(&c, &r, evaluate, |_| true).unwrap();
    assert!(out.complete);
    assert_eq!(out.progress.visited, 8);
    assert_eq!(out.progress.evaluated, 8);
    let best = out.best.unwrap();
    assert!(best.score >= out.baseline.score);
    assert!(
        best.composed
            .loadout
            .choices
            .iter()
            .all(|c| c.source == Source::Weiwang && c.tier == Tier::Base)
    );
    assert_eq!(best.composed.stats.pofang, r.loadout.base.pofang + 45);
    assert_eq!(out.pareto.len(), 1);
}
#[test]
fn infeasible_budget_and_cancellation_are_explicit() {
    let c = catalog();
    let mut r = request();
    r.constraints.stats.insert(
        "huajin".into(),
        Bounds {
            min: Some(1000),
            max: None,
        },
    );
    let out = optimize_with(&c, &r, evaluate, |_| true).unwrap();
    assert!(out.complete);
    assert!(out.best.is_none());
    r.max_combinations = 1;
    let out = optimize_with(&c, &r, evaluate, |_| true).unwrap();
    assert!(!out.complete);
    assert_eq!(out.stop_reason, "budget_exhausted");
    let out = optimize_with(&c, &r, evaluate, |_| false).unwrap();
    assert_eq!(out.progress.visited, 0);
    assert_eq!(out.stop_reason, "cancelled");
}
#[test]
fn conflicting_constraints_fail_before_search() {
    let c = catalog();
    let mut r = request();
    r.constraints
        .required_sources
        .insert(Slot::Tou, Source::Baowu);
    assert!(optimize_with(&c, &r, evaluate, |_| true).is_err());
    let mut r = request();
    r.constraints.stats.insert("typo".into(), Bounds::default());
    assert!(optimize_with(&c, &r, evaluate, |_| true).is_err());
    let mut r = request();
    r.constraints.locked = vec![r.loadout.choices[0].clone(); 2];
    assert!(optimize_with(&c, &r, evaluate, |_| true).is_err());
    let mut r = request();
    r.constraints.banned.push(r.loadout.choices[0].clone());
    assert!(optimize_with(&c, &r, evaluate, |_| true).is_err());
}
#[test]
fn cross_source_constraint_does_not_count_accessory_set() {
    let c = catalog();
    let mut r = request();
    r.constraints.allow_cross_source_2plus2 = false;
    assert!(
        optimize_with(&c, &r, evaluate, |_| true)
            .unwrap()
            .best
            .is_some()
    );
    r.constraints.sources = vec![Source::Mingjian, Source::Weiwang, Source::Baowu];
    for slot in Slot::ALL {
        let mut choice = r
            .loadout
            .choices
            .iter()
            .find(|c| c.slot == slot)
            .unwrap()
            .clone();
        if [Slot::Tou, Slot::Shou].contains(&slot) {
            choice.source = Source::Mingjian;
        }
        if slot == Slot::Xie {
            choice.source = Source::Baowu;
        }
        r.constraints.locked.push(choice);
    }
    assert!(
        optimize_with(&c, &r, evaluate, |_| true)
            .unwrap()
            .best
            .is_none()
    );
}
#[test]
fn json_api_has_strict_input_and_error_envelopes() {
    let c = catalog();
    assert!(matches!(
        call(&c, r#"{"method":"validate"}"#),
        ApiResponse::Ok { .. }
    ));
    for input in [
        r#"{"method":"compose","loadout":{}}"#,
        r#"{"method":"validate","typo":1}"#,
        r#"{"method":"unknown"}"#,
        "bad",
    ] {
        assert!(matches!(call(&c, input), ApiResponse::Error { .. }));
    }
    assert!(toml::from_str::<Stats>("gongjii=1").is_err());
}
#[test]
fn expected_objective_matches_existing_core_and_is_deterministic() {
    if !runtime_child("expected_objective_matches_existing_core_and_is_deterministic") {
        return;
    }
    let c = catalog();
    let r = request();
    let a = optimize(&c, &r, |_| true).unwrap();
    let b = optimize(&c, &r, |_| true).unwrap();
    assert_eq!(
        serde_json::to_value(&a).unwrap(),
        serde_json::to_value(&b).unwrap()
    );
    assert!(a.complete);
    assert!(a.delta.unwrap() >= 0.0);
    let best = a.best.unwrap();
    let mut config = r.config;
    config.player = player(&best.composed);
    let core = jpcg_core::host::calc::calculate(config).unwrap();
    assert_eq!(
        best.score,
        core.iter().find(|s| s.skill_name == "徵").unwrap().q as f64
    );
}
#[test]
fn kill_probability_matches_existing_seeded_combo_and_cd_is_never_silent() {
    if !runtime_child("kill_probability_matches_existing_seeded_combo_and_cd_is_never_silent") {
        return;
    }
    let c = catalog();
    let mut r = request();
    r.config.hostile.target_hp = 40000;
    r.objective = Objective::KillProbability {
        steps: vec![jpcg_api::ComboStepDTO {
            skill: jpcg_api::SkillPoolItemDTO {
                skill_name: "synthetic-strike".into(),
                base_damage1: 10000,
                base_damage2: 10000,
                atk_xishu: 1.0,
                ..Default::default()
            },
            overrides: None,
        }],
        samples: 1000,
        seed: 42,
        allow_unapplied_cd: false,
    };
    assert!(optimize(&c, &r, |_| true).unwrap_err().contains("CD"));
    if let Objective::KillProbability {
        allow_unapplied_cd, ..
    } = &mut r.objective
    {
        *allow_unapplied_cd = true;
    }
    let a = optimize(&c, &r, |_| true).unwrap();
    let b = optimize(&c, &r, |_| true).unwrap();
    assert_eq!(
        serde_json::to_value(&a).unwrap(),
        serde_json::to_value(&b).unwrap()
    );
    let Objective::KillProbability {
        steps,
        samples,
        seed,
        ..
    } = r.objective
    else {
        panic!()
    };
    let out = jpcg_combo::host::calculate_combo(
        steps,
        player(&a.baseline.composed),
        r.config.hostile,
        r.config.xinfa_config,
        r.config.buff,
        r.config.coefficient,
        r.config.value_set,
        jpcg_combo::engine::ComboConfig {
            samples,
            seed: Some(seed),
        },
    )
    .unwrap();
    assert_eq!(a.baseline.score, out.final_kill_prob);
    assert!(a.baseline.score > 0.0 && a.baseline.score < 1.0);
    assert!(a.warnings.iter().any(|s| s.contains("CD")));
}
#[test]
fn mismatched_context_and_unsafe_numeric_inputs_are_rejected() {
    if !runtime_child("mismatched_context_and_unsafe_numeric_inputs_are_rejected") {
        return;
    }
    let c = catalog();
    let mut r = request();
    r.config.value_set = Some("missing".into());
    assert!(optimize(&c, &r, |_| true).is_err());
    let mut r = request();
    r.config.xinfa_config.xinfa_nom = "力道".into();
    assert!(optimize(&c, &r, |_| true).is_err());
    let mut r = request();
    r.config.xinfa_config.atk_up = 999.0;
    assert!(optimize(&c, &r, |_| true).is_err());
    let mut r = request();
    r.config.hostile.huajin_dengji = u32::MAX;
    assert!(optimize(&c, &r, |_| true).is_err());
}

#[test]
fn cli_stdout_is_json_and_errors_exit_nonzero() {
    let binary = env!("CARGO_BIN_EXE_gear-opt");
    let output = std::process::Command::new(binary)
        .args([
            "--catalog",
            dir().to_str().unwrap(),
            "optimize",
            "--request",
            dir().join("sample-request.toml").to_str().unwrap(),
        ])
        .env("JPCG_DATA_DIR", dir().parent().unwrap())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["complete"], true);
    assert_eq!(result["progress"]["visited"], 16);
    use std::io::Write;
    let mut process = std::process::Command::new(binary)
        .args(["--catalog", dir().to_str().unwrap(), "api"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    process
        .stdin
        .take()
        .unwrap()
        .write_all(b"{\"method\":\"validate\",\"typo\":1}")
        .unwrap();
    let output = process.wait_with_output().unwrap();
    assert!(!output.status.success());
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["status"], "error");
}

#[test]
fn pareto_keeps_damage_survival_tradeoff_and_removes_dominated_results() {
    let mut c = catalog();
    // 合成样例：某项链属性套2用气血换攻击，不引入实战生存公式。
    let t = c
        .templates
        .iter_mut()
        .find(|t| {
            t.choice.slot == Slot::Xianglian && t.choice.tier == Tier::Base && t.choice.variant == 2
        })
        .unwrap();
    t.stats.gongji += 100;
    t.stats.qixue = 0;
    let r = request();
    let out = optimize_with(&c, &r, |v| Ok(v.stats.gongji as f64), |_| true).unwrap();
    assert_eq!(out.pareto.len(), 2);
    assert_ne!(
        out.pareto[0].composed.stats.qixue,
        out.pareto[1].composed.stats.qixue
    );
}
