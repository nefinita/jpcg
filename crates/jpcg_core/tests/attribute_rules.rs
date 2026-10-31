//! 在子进程中验证真实编辑器保存路径，避免测试线程共享 JPCG_DATA_DIR。
#[test]
fn editor_entrypoints_preserve_profession_rules_subprocess() {
    let root = std::env::temp_dir().join(format!("jpcg-attribute-editor-{}", std::process::id()));
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "editor_entrypoints_child"])
        .env("JPCG_ATTRIBUTE_EDITOR_CHILD", "1")
        .env("JPCG_DATA_DIR", &root)
        .status()
        .unwrap();
    let _ = std::fs::remove_dir_all(root);
    assert!(status.success());
}

#[test]
fn editor_entrypoints_child() {
    if std::env::var_os("JPCG_ATTRIBUTE_EDITOR_CHILD").is_none() {
        return;
    }
    let root = std::path::PathBuf::from(std::env::var_os("JPCG_DATA_DIR").unwrap());
    let dir = root.join("shuxing");
    std::fs::create_dir_all(&dir).unwrap();
    // 全部数字是合成的测试数据，不声称为任何心法的真实规则。
    let rules = jpcg_api::AttributeRules {
        id: "synthetic-editor".into(),
        level: 50,
        value_set: "synthetic-50".into(),
        source: "integration test only; no game data".into(),
        main_stat: jpcg_api::BasicAttribute::Gengu,
        damage_kind: jpcg_api::AttributeDamageKind::Lunar,
        qixue_multiplier: jpcg_api::AttributeRatio {
            numerator: 1,
            denominator: 1,
        },
        conversions: vec![],
    };
    jpcg_core::host::skill::save_skill_data(
        "mowen".into(),
        jpcg_api::SkillEditorDataDTO {
            xinfa: jpcg_api::XinfaConfigDTO {
                xinfa_nom: "根骨".into(),
                ..Default::default()
            },
            attributes: Some(rules.clone()),
            version: Some(jpcg_api::VersionInfoDTO {
                level: 50,
                ..Default::default()
            }),
            ..Default::default()
        },
    )
    .unwrap();
    let mut dto = jpcg_core::host::skill::load_skill_data("mowen".into()).unwrap();
    assert_eq!(dto.attributes.as_ref(), Some(&rules));
    // 老 JSON 请求缺少 attributes（不以 null 删除规则）。
    let mut legacy_json = serde_json::to_value(&dto).unwrap();
    legacy_json.as_object_mut().unwrap().remove("attributes");
    dto = serde_json::from_value(legacy_json).unwrap();
    dto.xinfa.xinfa_name = "legacy edit".into();
    jpcg_core::host::skill::save_skill_data("mowen".into(), dto).unwrap();
    assert_eq!(
        jpcg_core::skill_editor::load_attribute_rules("mowen").unwrap(),
        rules
    );
    let cfg = jpcg_core::skill_editor::load_skills("mowen");
    assert_eq!(cfg.xinfa.xinfa_name, "legacy edit");
    jpcg_core::skill_editor::save_skills("mowen", cfg.xinfa, cfg.skill, cfg.version).unwrap();
    assert_eq!(
        jpcg_core::skill_editor::load_attribute_rules("mowen").unwrap(),
        rules
    );
    let mut new_dto = jpcg_core::host::skill::load_skill_data("mowen".into()).unwrap();
    new_dto.attributes.as_mut().unwrap().id = "explicit-new-revision".into();
    jpcg_core::host::skill::save_skill_data("mowen".into(), new_dto).unwrap();
    assert_eq!(
        jpcg_core::skill_editor::load_attribute_rules("mowen")
            .unwrap()
            .id,
        "explicit-new-revision"
    );
}
