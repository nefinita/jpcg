// ============================================================================
// values — 数值集（运行期加载）
//
// 真源：data/values/index.toml + 同目录快照文件（随 data 通道下发）。
// 兜底：本地缺失/损坏时回退内置（jpcg_const::level_constant::CURRENT，
//       即体验服·苍生铸世一测），UI 通过 source="builtin" 可见来源。
//
// 解析策略：白名单严格解析（deny_unknown_fields），换算分母必须 > 0，
//           pvp_global_jianshang ∈ [0,1]；任一不合法即整份回退兜底并告警。
// ============================================================================

use std::path::PathBuf;

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

#[derive(Debug, Deserialize)]
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

/// 已加载的数值集（供计算取系数）
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
        },
        constant: CURRENT,
    }
}

fn read_index() -> Result<(IndexFile, PathBuf), String> {
    let dir = values_dir().ok_or_else(|| "未找到 data/values 目录".to_string())?;
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
    Ok((index, dir))
}

/// 列出可用值集（本地 data 不可用时返回单个内置兜底项）
pub fn list_value_sets() -> Vec<ValueSetDTO> {
    match read_index() {
        Ok((index, _)) => index
            .value_sets
            .iter()
            .map(|s| ValueSetDTO {
                id: s.id.clone(),
                name: s.name.clone(),
                level: s.level,
                is_default: s.id == index.default,
                source: "data".to_string(),
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
    match try_load(id) {
        Ok(v) => v,
        Err(e) => {
            crate::log::warn(&format!("数值集加载失败（回退内置兜底）：{}", e));
            builtin()
        }
    }
}

fn try_load(id: Option<&str>) -> Result<LoadedValueSet, String> {
    let (index, dir) = read_index()?;
    let wanted = id.unwrap_or(&index.default);
    let entry = index
        .value_sets
        .iter()
        .find(|s| s.id == wanted)
        .or_else(|| index.value_sets.iter().find(|s| s.id == index.default))
        .ok_or_else(|| format!("未找到值集: {}", wanted))?;
    let text = std::fs::read_to_string(dir.join(&entry.file))
        .map_err(|e| format!("读取快照 {} 失败: {}", entry.file, e))?;
    let snap: Snapshot =
        toml::from_str(&text).map_err(|e| format!("解析快照 {} 失败: {}", entry.file, e))?;
    let constant = snap.to_constant()?;
    Ok(LoadedValueSet {
        info: ValueSetDTO {
            id: entry.id.clone(),
            name: entry.name.clone(),
            level: entry.level,
            is_default: entry.id == index.default,
            source: "data".to_string(),
        },
        constant,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXP: &str = include_str!("../../../../data/values/cszj-exp-260908.toml");

    #[test]
    fn snapshot_parses_exp_and_validates() {
        let snap: Snapshot = toml::from_str(EXP).expect("exp 快照应可解析");
        assert_eq!(snap.level, 50);
        let c = snap.to_constant().expect("exp 快照应通过校验");
        assert_eq!(c.huixin_xishu, 9512.91);
    }

    #[test]
    fn snapshot_rejects_zero_or_unknown() {
        // 未知字段
        let unknown = format!("{}\nnope = 1\n", EXP);
        assert!(toml::from_str::<Snapshot>(&unknown).is_err());
        // 分母为 0
        let zero = EXP.replace("huixin_xishu  = 9512.91", "huixin_xishu  = 0.0");
        let snap: Snapshot = toml::from_str(&zero).unwrap();
        assert!(snap.to_constant().is_err());
        // pvp 越界
        let bad_pvp = EXP.replace("pvp_global_jianshang = 0.9", "pvp_global_jianshang = 1.5");
        let snap: Snapshot = toml::from_str(&bad_pvp).unwrap();
        assert!(snap.to_constant().is_err());
    }

    #[test]
    fn index_parses_builtin_repo_file() {
        let idx: IndexFile = toml::from_str(include_str!("../../../../data/values/index.toml"))
            .expect("index 应可解析");
        assert_eq!(idx.default, "cszj-exp-260908");
        assert_eq!(idx.value_sets.len(), 2);
    }

    #[test]
    fn load_falls_back_to_builtin_when_no_data() {
        if values_dir().is_some() {
            return; // 有本地 data 时该断言不适用
        }
        let v = load_value_set(Some("does-not-exist"));
        assert_eq!(v.constant.huixin_xishu, CURRENT.huixin_xishu);
        assert_eq!(v.info.source, "builtin");
    }

    #[test]
    fn loads_real_data_when_available() {
        if values_dir().is_none() {
            return; // 无本地 data（默认 CI 环境）
        }
        let sets = list_value_sets();
        assert!(sets.iter().any(|s| s.id == "live-130"));
        assert!(sets.iter().any(|s| s.id == "cszj-exp-260908"));

        let live = load_value_set(Some("live-130"));
        assert_eq!(live.info.source, "data");
        assert_eq!(live.constant, jpcg_const::level_constant::LIVE_130);

        // 未知 id → 回退默认（index.toml 的 default = 体验服一测）
        let dflt = load_value_set(Some("nope"));
        assert_eq!(dflt.constant, CURRENT);
    }
}
