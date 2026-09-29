// ============================================================================
// host::values — 数值集查询入口（静态直调 / FFI 共用）
//
// 数值真源：data/values/index.toml + 同目录快照（随 data 通道下发）；
// 本地不可用时回退内置兜底（体验服·苍生铸世一测）。
// ============================================================================

use jpcg_api::ValueSetDTO;

/// 列出可用数值集（含默认标记与来源 data/builtin）
pub fn list_value_sets() -> Vec<ValueSetDTO> {
    crate::store::values::list_value_sets()
}
