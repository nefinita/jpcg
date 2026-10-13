// ============================================================================
// host::values — 数值集查询与取值入口（静态直调 / FFI 共用）
//
// 数值真源：data/values/index.toml + 同目录快照（随 data 通道下发）；
// 本地不可用时回退内置兜底（体验服·苍生铸世一测）。
// 单技能/求导/连招均经 `value_set_constant` 取值，保证跨入口一致。
// ============================================================================

use jpcg_api::ValueSetDTO;
use jpcg_const::level_constant::LevelConstant;

/// 列出可用数值集（含默认标记、来源 data/builtin、逐项可用性）
pub fn list_value_sets() -> Vec<ValueSetDTO> {
    crate::store::values::list_value_sets()
}

/// 解析指定值集最终使用的等级常数（含回退；单技能/连招统一入口）
pub fn value_set_constant(id: Option<&str>) -> LevelConstant {
    crate::store::values::load_value_set(id).constant
}

/// 解析指定值集**实际使用**的信息（用于向调用方上报回退结果：
/// 例如请求 live-130 但快照损坏时，返回 source="builtin" 的兜底值集）
pub fn resolve_value_set(id: Option<&str>) -> ValueSetDTO {
    crate::store::values::load_value_set(id).info
}
