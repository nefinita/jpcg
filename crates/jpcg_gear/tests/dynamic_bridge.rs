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
            root.join("target/debug").join(format!(
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
    let catalog: jpcg_gear::Catalog = gear_bridge::call("catalog", &serde_json::json!({})).unwrap();
    assert!(catalog.version.sample);
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/gear/sample-loadout.toml"),
    )
    .unwrap();
    let loadout: jpcg_gear::Loadout = toml::from_str(&text).unwrap();
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
