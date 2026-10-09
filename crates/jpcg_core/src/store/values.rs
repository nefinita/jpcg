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

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

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
            coefficient: Some(coefficient_dto(&CURRENT)),
        },
        constant: CURRENT,
    }
}

/// 等级常数 → 系数 DTO（供前端 seed「系数设置」）
fn coefficient_dto(c: &LevelConstant) -> jpcg_api::CoefficientConfigDTO {
    jpcg_api::CoefficientConfigDTO {
        pofang_xishu: c.pofang_xishu,
        huixin_xishu: c.huixin_xishu,
        huixiao_xishu: c.huixiao_xishu,
        yujin_xishu: c.yujin_xishu,
        yuhui_xishu: c.yuhui_xishu,
        huajin_xishu: c.huajin_xishu,
        fangyu_xishu: c.fangyu_xishu,
        pvp_global_jianshang: c.pvp_global_jianshang,
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

/// 快照文件名必须是纯文件名（拒绝路径分隔符 / `..` / 空名，防目录穿越）
fn is_safe_relative_filename(file: &str) -> bool {
    !file.is_empty() && !file.contains('/') && !file.contains('\\') && file != "." && file != ".."
}

/// 读取并校验单个快照
fn read_snapshot(dir: &Path, file: &str) -> Result<LevelConstant, String> {
    if !is_safe_relative_filename(file) {
        return Err(format!("快照文件名非法（不得含路径分隔符）: {}", file));
    }
    let text = std::fs::read_to_string(dir.join(file))
        .map_err(|e| format!("读取快照 {} 失败: {}", file, e))?;
    let snap: Snapshot =
        toml::from_str(&text).map_err(|e| format!("解析快照 {} 失败: {}", file, e))?;
    snap.to_constant()
}

fn entry_to_dto(
    entry: &IndexEntry,
    index: &IndexFile,
    constant: Option<&LevelConstant>,
) -> ValueSetDTO {
    ValueSetDTO {
        id: entry.id.clone(),
        name: entry.name.clone(),
        level: entry.level,
        is_default: entry.id == index.default,
        source: "data".to_string(),
        available: constant.is_some(),
        coefficient: constant.map(coefficient_dto),
    }
}

// ---------------------------------------------------------------------------
// 内存缓存：避免每次计算都读盘解析（index + 快照）。
// 以 index/快照文件的 mtime 做校验：文件被数据更新改写后，下次读取自动失效——
// 该机制对**每个 cdylib 各自生效**（dynamic 模式下 core 与 combo 各持一份缓存，
// 无需跨库广播失效）。
// ---------------------------------------------------------------------------

struct CacheEntry {
    loaded: LoadedValueSet,
    index_mtime: Option<SystemTime>,
    snapshot_path: Option<PathBuf>,
    snapshot_mtime: Option<SystemTime>,
}

#[derive(Default)]
struct ValueCache {
    loaded: HashMap<(PathBuf, Option<String>), CacheEntry>,
}

fn cache() -> &'static Mutex<ValueCache> {
    static CACHE: OnceLock<Mutex<ValueCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(ValueCache::default()))
}

/// 清空内存缓存（数据更新落盘后调用；mtime 校验亦会自动失效）
pub fn invalidate_cache() {
    if let Ok(mut c) = cache().lock() {
        c.loaded.clear();
    }
}

fn mtime(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).ok().and_then(|m| m.modified().ok())
}

fn cache_valid(e: &CacheEntry, index_mtime: Option<SystemTime>) -> bool {
    if e.index_mtime != index_mtime {
        return false;
    }
    match &e.snapshot_path {
        Some(p) => e.snapshot_mtime == mtime(p),
        None => true,
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
            .map(|s| {
                let constant = read_snapshot(dir, &s.file).ok();
                entry_to_dto(s, &index, constant.as_ref())
            })
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
        Some(dir) => load_cached(&dir, id),
        None => builtin(),
    }
}

/// 从指定目录加载（带 mtime 校验的缓存；显式目录版便于测试）
pub(crate) fn load_cached(dir: &Path, id: Option<&str>) -> LoadedValueSet {
    let key = (dir.to_path_buf(), id.map(str::to_string));
    let index_mtime = mtime(&dir.join(INDEX_FILENAME));

    if let Ok(c) = cache().lock()
        && let Some(e) = c.loaded.get(&key)
        && cache_valid(e, index_mtime)
    {
        return e.loaded.clone();
    }

    let (loaded, snapshot_path) = match load_from(dir, id) {
        Ok((v, p)) => (v, Some(p)),
        Err(e) => {
            crate::log::warn(&format!("数值集加载失败（回退内置兜底）：{}", e));
            (builtin(), None)
        }
    };
    let snapshot_mtime = snapshot_path.as_deref().and_then(mtime);
    let entry = CacheEntry {
        loaded: loaded.clone(),
        index_mtime,
        snapshot_path,
        snapshot_mtime,
    };
    if let Ok(mut c) = cache().lock() {
        c.loaded.insert(key, entry);
    }
    loaded
}

/// 从指定目录加载，失败回退内置兜底（显式目录版，仅供测试）
#[cfg(test)]
pub(crate) fn load_with_fallback(dir: &Path, id: Option<&str>) -> LoadedValueSet {
    match load_from(dir, id) {
        Ok((v, _)) => v,
        Err(e) => {
            crate::log::warn(&format!("数值集加载失败（回退内置兜底）：{}", e));
            builtin()
        }
    }
}

/// 加载指定值集，返回 (值集, 实际使用的快照文件路径)
fn load_from(dir: &Path, id: Option<&str>) -> Result<(LoadedValueSet, PathBuf), String> {
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
    let snapshot_path = dir.join(&entry.file);
    Ok((
        LoadedValueSet {
            info: entry_to_dto(entry, &index, Some(&constant)),
            constant,
        },
        snapshot_path,
    ))
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
    fn dto_carries_coefficient_when_available() {
        let t = TempData::new("coeff");
        let sets = list_from(t.values());
        let exp = sets.iter().find(|s| s.id == "cszj-exp-260908").unwrap();
        assert!(exp.available);
        assert_eq!(
            exp.coefficient.as_ref().map(|c| c.huixin_xishu),
            Some(CURRENT.huixin_xishu)
        );
        // 快照缺失 → 不可用且无系数
        std::fs::remove_file(t.values().join("live-130.toml")).unwrap();
        let sets = list_from(t.values());
        let live = sets.iter().find(|s| s.id == "live-130").unwrap();
        assert!(!live.available && live.coefficient.is_none());
    }

    #[test]
    fn loads_default_and_explicit_set() {
        let t = TempData::new("load");
        assert_eq!(load_from(t.values(), None).unwrap().0.constant, CURRENT);
        assert_eq!(
            load_from(t.values(), Some("live-130")).unwrap().0.constant,
            jpcg_const::level_constant::LIVE_130
        );
        // 未知 id → 回退默认项（体验服一测），而非报错
        assert_eq!(
            load_from(t.values(), Some("nope")).unwrap().0.constant,
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

    #[test]
    fn rejects_path_traversal_in_index() {
        let t = TempData::new("traversal");
        // 目录外写一份“陷阱”快照（huixin=9999）
        std::fs::write(
            t.root().join("outside.toml"),
            "level = 1\npofang_xishu = 1.0\nhuixin_xishu = 9999.0\nhuixiao_xishu = 1.0\nyujin_xishu = 1.0\nyuhui_xishu = 1.0\nhuajin_xishu = 1.0\nfangyu_xishu = 1.0\npvp_global_jianshang = 0.9\n",
        )
        .unwrap();
        std::fs::write(
            t.values().join("index.toml"),
            "default = \"esc\"\n[[value_sets]]\nid = \"esc\"\nname = \"esc\"\nlevel = 1\nfile = \"../outside.toml\"\n",
        )
        .unwrap();

        // 列表：该顶标记为不可用
        let sets = list_from(t.values());
        assert!(!sets.iter().find(|s| s.id == "esc").unwrap().available);
        // 加载：回退内置兜底，绝不读到目录外
        let v = load_with_fallback(t.values(), Some("esc"));
        assert_eq!(v.info.source, "builtin");
        assert_ne!(v.constant.huixin_xishu, 9999.0);
    }

    #[test]
    fn cache_reloads_after_snapshot_change() {
        let t = TempData::new("cache");
        assert_eq!(
            load_cached(t.values(), Some("live-130")).constant,
            jpcg_const::level_constant::LIVE_130
        );
        // 改写同一快照文件（同 id、同路径）——mtime 变化应使缓存失效
        std::thread::sleep(std::time::Duration::from_millis(20));
        let modified = LIVE.replace("huixin_xishu  = 197703.0", "huixin_xishu  = 111.0");
        std::fs::write(t.values().join("live-130.toml"), modified).unwrap();
        assert_eq!(
            load_cached(t.values(), Some("live-130"))
                .constant
                .huixin_xishu,
            111.0
        );
    }
}
