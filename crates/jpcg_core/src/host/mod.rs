// ============================================================================
// host — 统一宿主 API（JSON 契约层）
// 所有对外业务入口集中在 host：输入输出均为 jpcg_api DTO 类型。
// 两种消费方式：
//   - 静态模式：Tauri 壳直接调用本模块函数（Rust 直调，零 JSON 开销）
//   - 动态模式：FFI 层（ffi.rs）将 JSON 反序列化为 DTO 后调用本模块
// 金标准测试穿透 host 层，保证双模式数值一致。
// ============================================================================

pub mod calc;
pub mod combo;
pub mod config;
pub mod conv;
pub mod skill;
#[cfg(feature = "net")]
pub mod update;
