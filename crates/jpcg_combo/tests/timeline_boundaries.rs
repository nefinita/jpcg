use jpcg_api::*;
use jpcg_combo::timeline_host::calculate_timeline;

fn cast() -> TimelineActionDTO {
    TimelineActionDTO::Cast {
        skill_id: 1,
        sub_id: 0,
        skill_name: Some("测试".into()),
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
