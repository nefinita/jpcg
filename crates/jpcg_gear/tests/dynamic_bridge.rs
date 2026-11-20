// 直接编译桌面实际 bridge 文件；此测试可在 model-only 特性下运行，证明不需静态引擎。
#[path = "../../../examples/jpcg_app/src-tauri/src/commands/gear_bridge.rs"]
mod gear_bridge;

#[test]
fn desktop_bridge_subprocess() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "desktop_bridge_child"])
        .env("JPCG_DATA_DIR", root.join("data"))
        .env(
            "JPCG_GEAR_LIB",
            root.join("target")
                .join(if cfg!(debug_assertions) {
                    "debug"
                } else {
                    "release"
                })
                .join(format!(
                    "{}jpcg_gear{}",
                    std::env::consts::DLL_PREFIX,
                    std::env::consts::DLL_SUFFIX
                )),
        )
        .env("JPCG_GEAR_BRIDGE_CHILD", "1")
        .status()
        .unwrap();
    assert!(status.success());
}
#[test]
fn desktop_bridge_child() {
    if std::env::var_os("JPCG_GEAR_BRIDGE_CHILD").is_none() {
        return;
    }
    let catalog: jpcg_gear::Catalog =
        gear_bridge::call("catalog", &serde_json::json!({"profession":"mowen"})).unwrap();
    assert_eq!(catalog.version.revision, "live-50-20261106");
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/mowen/gear/sample-loadout.toml"),
    )
    .unwrap();
    // 随附真实装备样例已显式选择职业 TOML 换算，不再依赖调用方补字段。
    let loadout: jpcg_gear::Loadout = toml::from_str(&text).unwrap();
    let prepared: jpcg_gear::Loadout = gear_bridge::call(
        "prepare_loadout",
        &serde_json::json!({"profession":"mowen","loadout":loadout}),
    )
    .unwrap();
    assert_eq!(prepared.choices.len(), 12);
    assert_eq!(
        prepared.attribute_rules.as_deref(),
        Some(jpcg_gear::attributes::PROFESSION_TOML)
    );
    let composition: jpcg_gear::Composition = gear_bridge::call("compose", &loadout).unwrap();
    assert_eq!(composition.player.jcsx, "根骨");
    let exported: String = gear_bridge::call("export", &loadout).unwrap();
    let imported: jpcg_gear::Composition =
        gear_bridge::call("import", &serde_json::json!({"toml":exported})).unwrap();
    assert_eq!(composition.composed.stats, imported.composed.stats);
    assert!(
        gear_bridge::call::<_, serde_json::Value>("catalog", &serde_json::json!({"path":"/tmp"}))
            .is_err()
    );
}

#[test]
fn desktop_joint_cancel_subprocess() {
    if std::env::var_os("JPCG_GEAR_BRIDGE_CHILD").is_some() {
        return;
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "desktop_joint_cancel_child"])
        .env("JPCG_DATA_DIR", root.join("data"))
        .env(
            "JPCG_GEAR_LIB",
            root.join("target")
                .join(if cfg!(debug_assertions) {
                    "debug"
                } else {
                    "release"
                })
                .join(format!(
                    "{}jpcg_gear{}",
                    std::env::consts::DLL_PREFIX,
                    std::env::consts::DLL_SUFFIX
                )),
        )
        .env("JPCG_GEAR_BRIDGE_CHILD", "1")
        .status()
        .unwrap();
    assert!(status.success());
}
#[test]
fn desktop_joint_cancel_child() {
    if std::env::var_os("JPCG_GEAR_BRIDGE_CHILD").is_none() {
        return;
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let request: jpcg_gear::OptimizeRequest = toml::from_str(
        &std::fs::read_to_string(root.join("data/mowen/gear/sample-request.toml")).unwrap(),
    )
    .unwrap();
    let id: jpcg_gear::OptimizeJobId = gear_bridge::call("optimize_joint_start", &request).unwrap();
    let _: serde_json::Value = gear_bridge::call("optimize_joint_cancel", &id).unwrap();
    let started = std::time::Instant::now();
    loop {
        let status: jpcg_gear::OptimizeJobStatus =
            gear_bridge::call("optimize_joint_poll", &id).unwrap();
        if status.done {
            assert!(status.error.is_none(), "{:?}", status.error);
            let result = status.result.unwrap();
            assert!(!result.complete);
            assert_eq!(result.stop_reason, "cancelled");
            break;
        }
        assert!(started.elapsed().as_secs() < 15, "cancel did not finish");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(
        gear_bridge::call::<_, jpcg_gear::OptimizeJobStatus>("optimize_joint_poll", &id).is_err()
    );
}
