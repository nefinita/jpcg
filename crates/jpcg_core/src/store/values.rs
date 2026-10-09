// ============================================================================
// values — 数值集（运行期加载）
//
// 真源：data/values/index.toml + 同目录快照文件（随 data 通道下发）。
// 兜底：本地缺失/损坏时回退内置（jpcg_const::level_constant::CURRENT，
//       即体验服·苍生铸世一测），UI 通过 source="builtin" 可见来源。
//
// 解析策略：白名单严格解析（deny_unknown_fields），换算分母必须 > 0，
//           pvp_global_jianshang ∈ [0,1]；任一不合法即整体回退并告警。
// 可测试性：核心逻辑均以显式目录参数（`*_from` / `load_with_fallback`）实现，
//           便于用隔离临时目录做确定性回归，而不依赖环境变量或真实 data 是否存在。
// ============================================================================

use std::path::Path;

use jpcg_api::ValueSetDTO;
use jpcg_const::level_constant::{CURRENT, LevelConstant};
use serde::Deserialize;

use super::paths::values_dir;

const INDEX_FILENAME: &str = "index.toml";
/// 内置兜底值集的 id / 显示名（与 data/values/cszj-exp-260908.toml 同源）
pub const BUILTIN_ID: &str = "builtin-cszj-exp-260908";
pub const BUILTIN_NAME: &str = "体验服·苍生铸世一测";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct IndexFile {
    default: String,
    value_sets: Vec<IndexEntry>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct IndexEntry {
    id: String,
    name: String,
    level: u32,
    file: String,
}

/// 快照文件结构（与 LevelConstant 字段一一对应；level 单独存放）
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    level: u32,
    pofang_xishu: f32,
    huixin_xishu: f32,
    huixiao_xishu: f32,
    yujin_xishu: f32,
    yuhui_xishu: f32,
    huajin_xishu: f32,
    fangyu_xishu: f32,
    pvp_global_jianshang: f32,
}

impl Snapshot {
    fn to_constant(&self) -> Result<LevelConstant, String> {
        let denominators = [
            ("pofang_xishu", self.pofang_xishu),
            ("huixin_xishu", self.huixin_xishu),
            ("huixiao_xishu", self.huixiao_xishu),
            ("yujin_xishu", self.yujin_xishu),
            ("yuhui_xishu", self.yuhui_xishu),
            ("huajin_xishu", self.huajin_xishu),
            ("fangyu_xishu", self.fangyu_xishu),
        ];
        for (name, v) in denominators {
            if !(v.is_finite() && v > 0.0) {
                return Err(format!("{} 非法（须为有限正数）: {}", name, v));
            }
        }
        if !(self.pvp_global_jianshang.is_finite()
            && (0.0..=1.0).contains(&self.pvp_global_jianshang))
        {
            return Err(format!(
                "pvp_global_jianshang 非法（须 ∈ [0,1]）: {}",
                self.pvp_global_jianshang
            ));
        }
        if self.level == 0 {
            return Err("level 须为正整数".to_string());
        }
        Ok(LevelConstant {
            pofang_xishu: self.pofang_xishu,
            huixin_xishu: self.huixin_xishu,
            huixiao_xishu: self.huixiao_xishu,
            yujin_xishu: self.yujin_xishu,
            yuhui_xishu: self.yuhui_xishu,
            huajin_xishu: self.huajin_xishu,
            fangyu_xishu: self.fangyu_xishu,
            pvp_global_jianshang: self.pvp_global_jianshang,
        })
    }
}

/// 已加载的数值集（供计算取系数；info 用于上报实际使用的值集）
#[derive(Debug, Clone)]
pub struct LoadedValueSet {
    pub info: ValueSetDTO,
    pub constant: LevelConstant,
}

fn builtin() -> LoadedValueSet {
    LoadedValueSet {
        info: ValueSetDTO {
            id: BUILTIN_ID.to_string(),
            name: BUILTIN_NAME.to_string(),
            level: 0,
            is_default: true,
            source: "builtin".to_string(),
            available: true,
        },
        constant: CURRENT,
    }
}

fn read_index(dir: &Path) -> Result<IndexFile, String> {
    let text = std::fs::read_to_string(dir.join(INDEX_FILENAME))
        .map_err(|e| format!("读取 {} 失败: {}", INDEX_FILENAME, e))?;
    let index: IndexFile =
        toml::from_str(&text).map_err(|e| format!("解析 {} 失败: {}", INDEX_FILENAME, e))?;
    if index.value_sets.is_empty() {
        return Err("index.toml 未定义任何值集".to_string());
    }
    if !index.value_sets.iter().any(|s| s.id == index.default) {
        return Err(format!(
            "index.toml 的 default 未在 value_sets 中: {}",
            index.default
        ));
    }
    Ok(index)
}

/// 读取并校验单个快照
fn read_snapshot(dir: &Path, file: &str) -> Result<LevelConstant, String> {
    let text = std::fs::read_to_string(dir.join(file))
        .map_err(|e| format!("读取快照 {} 失败: {}", file, e))?;
    let snap: Snapshot =
        toml::from_str(&text).map_err(|e| format!("解析快照 {} 失败: {}", file, e))?;
    snap.to_constant()
}

fn entry_to_dto(entry: &IndexEntry, index: &IndexFile, available: bool) -> ValueSetDTO {
    ValueSetDTO {
        id: entry.id.clone(),
        name: entry.name.clone(),
        level: entry.level,
        is_default: entry.id == index.default,
        source: "data".to_string(),
        available,
    }
}

/// 列出可用值集（同时逐项校验快照可用性；索引不可用时返回内置兜底项）
pub fn list_value_sets() -> Vec<ValueSetDTO> {
    match values_dir() {
        Some(dir) => list_from(&dir),
        None => vec![builtin().info],
    }
}

fn list_from(dir: &Path) -> Vec<ValueSetDTO> {
    match read_index(dir) {
        Ok(index) => index
            .value_sets
            .iter()
            .map(|s| entry_to_dto(s, &index, read_snapshot(dir, &s.file).is_ok()))
            .collect(),
        Err(e) => {
            crate::log::warn(&format!("数值集清单不可用（回退内置兜底）：{}", e));
            vec![builtin().info]
        }
    }
}

/// 加载指定数值集；`id = None` 用 index.toml 的 default；
/// 任何失败（目录/文件/解析/校验）都回退内置兜底（体验服一测）。
pub fn load_value_set(id: Option<&str>) -> LoadedValueSet {
    match values_dir() {
        Some(dir) => load_with_fallback(&dir, id),
        None => builtin(),
    }
}

/// 从指定目录加载，失败回退内置兜底（显式目录版，便于测试）
pub(crate) fn load_with_fallback(dir: &Path, id: Option<&str>) -> LoadedValueSet {
    match load_from(dir, id) {
        Ok(v) => v,
        Err(e) => {
            crate::log::warn(&format!("数值集加载失败（回退内置兜底）：{}", e));
            builtin()
        }
    }
}

fn load_from(dir: &Path, id: Option<&str>) -> Result<LoadedValueSet, String> {
    let index = read_index(dir)?;
    let wanted = id.unwrap_or(&index.default);
    // 指定 id 未命中时回退默认项；默认项也缺失才报错
    let entry = index
        .value_sets
        .iter()
        .find(|s| s.id == wanted)
        .or_else(|| index.value_sets.iter().find(|s| s.id == index.default))
        .ok_or_else(|| format!("未找到值集: {}", wanted))?;
    let constant = read_snapshot(dir, &entry.file)?;
    Ok(LoadedValueSet {
        info: entry_to_dto(entry, &index, true),
        constant,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::paths::values_dir_from_env;
    use std::path::{Path, PathBuf};

    const EXP: &str = include_str!("../../../../data/values/cszj-exp-260908.toml");
    const LIVE: &str = include_str!("../../../../data/values/live-130.toml");
    const INDEX: &str = include_str!("../../../../data/values/index.toml");

    /// 隔离临时目录：`root/values/` 写入 index + 两份快照，Drop 时清理
    struct TempData {
        root: PathBuf,
        values: PathBuf,
    }
    impl TempData {
        fn new(tag: &str) -> Self {
            let root =
                std::env::temp_dir().join(format!("jpcg-values-{}-{}", tag, std::process::id()));
            let _ = std::fs::remove_dir_all(&root);
            let values = root.join("values");
            std::fs::create_dir_all(&values).expect("mkdir temp");
            std::fs::write(values.join("index.toml"), INDEX).expect("write index");
            std::fs::write(values.join("cszj-exp-260908.toml"), EXP).expect("write exp");
            std::fs::write(values.join("live-130.toml"), LIVE).expect("write live");
            Self { root, values }
        }
        fn values(&self) -> &Path {
            &self.values
        }
        fn root(&self) -> &Path {
            &self.root
        }
    }
    impl Drop for TempData {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn lists_with_availability_and_default() {
        let t = TempData::new("list");
        let sets = list_from(t.values());
        assert_eq!(sets.len(), 2);
        let exp = sets.iter().find(|s| s.id == "cszj-exp-260908").unwrap();
        assert!(exp.is_default && exp.available && exp.source == "data");
        let live = sets.iter().find(|s| s.id == "live-130").unwrap();
        assert!(!live.is_default && live.available);
    }

    #[test]
    fn loads_default_and_explicit_set() {
        let t = TempData::new("load");
        assert_eq!(load_from(t.values(), None).unwrap().constant, CURRENT);
        assert_eq!(
            load_from(t.values(), Some("live-130")).unwrap().constant,
            jpcg_const::level_constant::LIVE_130
        );
        // 未知 id → 回退默认项（体验服一测），而非报错
        assert_eq!(
            load_from(t.values(), Some("nope")).unwrap().constant,
            CURRENT
        );
    }

    #[test]
    fn missing_snapshot_marks_unavailable_and_falls_back() {
        let t = TempData::new("missing");
        std::fs::remove_file(t.values().join("live-130.toml")).unwrap();

        // 列表：live-130 仍列出但 available=false
        let sets = list_from(t.values());
        assert!(!sets.iter().find(|s| s.id == "live-130").unwrap().available);
        assert!(
            sets.iter()
                .find(|s| s.id == "cszj-exp-260908")
                .unwrap()
                .available
        );

        // 加载：回退内置兜底，且可观察（source=builtin）
        let v = load_with_fallback(t.values(), Some("live-130"));
        assert_eq!(v.info.source, "builtin");
        assert_eq!(v.constant, CURRENT);
    }

    #[test]
    fn invalid_snapshot_marks_unavailable_and_falls_back() {
        let t = TempData::new("invalid");
        // 分母置 0（校验失败）
        let bad = LIVE.replace("huixin_xishu  = 197703.0", "huixin_xishu  = 0.0");
        std::fs::write(t.values().join("live-130.toml"), bad).unwrap();

        let sets = list_from(t.values());
        assert!(!sets.iter().find(|s| s.id == "live-130").unwrap().available);
        let v = load_with_fallback(t.values(), Some("live-130"));
        assert_eq!(v.info.source, "builtin");
    }

    #[test]
    fn unknown_snapshot_field_rejected() {
        let t = TempData::new("unknown");
        let extra = format!("{}\nbogus = 1\n", EXP);
        std::fs::write(t.values().join("cszj-exp-260908.toml"), extra).unwrap();
        assert!(
            !list_from(t.values())
                .iter()
                .find(|s| s.id == "cszj-exp-260908")
                .unwrap()
                .available
        );
    }

    #[test]
    fn env_dir_accepts_data_root_and_shuxing() {
        let t = TempData::new("env");
        // JPCG_DATA_DIR 指向数据根
        assert_eq!(
            values_dir_from_env(t.root()),
            Some(t.values().to_path_buf())
        );
        // JPCG_DATA_DIR 直接指向 <root>/shuxing → 取同级 values
        let shuxing = t.root().join("shuxing");
        std::fs::create_dir_all(&shuxing).unwrap();
        assert_eq!(
            values_dir_from_env(&shuxing),
            Some(t.values().to_path_buf())
        );
    }
}
