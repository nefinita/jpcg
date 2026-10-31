#![cfg(feature = "engine")]
use jpcg_gear::{Catalog, Composition, Loadout, host};

fn sample() -> (Catalog, Loadout) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/gear");
    let catalog = Catalog::load(&dir).unwrap();
    let loadout =
        toml::from_str(&std::fs::read_to_string(dir.join("sample-loadout.toml")).unwrap()).unwrap();
    (catalog, loadout)
}
#[test]
fn desktop_catalog_and_import_export_keep_composition() {
    let (catalog, loadout) = sample();
    let output = host::dispatch(&catalog, "catalog", "{}").unwrap();
    assert_eq!(output["version"]["revision"], "synthetic-v1");
    let input = serde_json::to_string(&loadout).unwrap();
    let composed = host::dispatch(&catalog, "compose", &input).unwrap();
    let composition: Composition = serde_json::from_value(composed.clone()).unwrap();
    assert_eq!(
        composition.player.jichu_gongji,
        composition.composed.stats.gongji
    );
    assert_eq!(composition.player.jcsx, "根骨");
    let text = host::dispatch(&catalog, "export", &input).unwrap();
    let imported = host::dispatch(
        &catalog,
        "import",
        &serde_json::json!({"toml":text}).to_string(),
    )
    .unwrap();
    assert_eq!(imported, composed);
    assert!(host::dispatch(&catalog, "catalog", r#"{"path":"/tmp"}"#).is_err());
    assert!(host::dispatch(&catalog, "import", r#"{"toml":"bad","path":"/tmp"}"#).is_err());
    assert!(host::dispatch(&catalog, "unknown", "{}").is_err());
    let mut stale = loadout;
    stale.revision = "old".into();
    assert!(host::dispatch(&catalog, "compose", &serde_json::to_string(&stale).unwrap()).is_err());
}
