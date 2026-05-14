// ============================================================================
// jpcg_core — 剑心计算核心库
// 提供剑网3 伤害计算的核心数据结构、配置加载与保存、以及计算入口。
// 该库作为底层 crate，被 jpcg_update、jpcg_app (Tauri) 等上层 crate 依赖。
// ============================================================================

// —— 公开模块 ——
pub mod cal;        // 伤害计算引擎
mod io;             // 文件 IO（配置文件读写）
mod log;            // 日志输出工具
pub mod type_set;   // 数据类型定义（玩家、目标、心法、技能等）

// ============================================================================
// load_config — 配置加载模块
// 对外提供从文件加载默认配置（SaveConfig）和按心法加载 TOML 配置的入口。
// ============================================================================
pub mod load_config {
    use crate::io::{SaveConfig, TomlConfig, load_config, load_save_config};

    /// 加载当前保存的完整配置（玩家+目标+心法）
    pub fn default_load() -> SaveConfig {
        load_save_config()
    }

    /// 按心法名称加载对应的 TOML 技能配置表
    /// - `profession`: 心法名称（同时也是 .toml 文件名）
    /// - 返回: 包含心法配置和技能列表的 TomlConfig
    pub fn show_config(profession: &str) -> TomlConfig {
        load_config(profession)
    }
}

// ============================================================================
// save_config — 配置持久化模块
// 将当前玩家配置保存到本地 TOML 文件（saved_config.toml）。
// ============================================================================
pub mod save_config {
    use crate::{
        io::save_config,
        type_set::{hostilepile::HostilepileConfig, player::PlayerConfig},
    };

    /// 持久化保存玩家、目标和心法配置到本地文件
    pub fn save(
        player: PlayerConfig,
        hostilepile: HostilepileConfig,
        xinfa: crate::type_set::xinfa::XinfaConfig,
    ) {
        save_config(player, hostilepile, xinfa);
    }
}

// ============================================================================
// calculate — 伤害计算入口模块
// 封装核心计算引擎（cal），对外提供统一的 start 函数。
// ============================================================================
pub mod calculate {
    use std::io::Error;

    use crate::cal;
    use crate::type_set::{
        hostilepile::HostilepileConfig, player::PlayerConfig, xinfa::XinfaConfig,
    };

    /// 启动伤害计算
    /// - `player`: 玩家属性配置
    /// - `hostilepile`: 目标（敌对）属性配置
    /// - `xinfa`: 心法配置
    /// - 返回: 每个技能的伤害计算结果列表
    pub fn start(
        player: PlayerConfig,
        hostilepile: HostilepileConfig,
        xinfa: XinfaConfig,
    ) -> Result<Vec<cal::CalculateResult>, Error> {
        cal::start_calculation(player, hostilepile, xinfa)
    }
}

// ============================================================================
// FFI (Foreign Function Interface) — 供 C/动态库调用
// 以 #[repr(C)] 布局兼容 C ABI，用于跨语言调用（如 Lua / Unity）。
// 当前主要用于预留扩展，未在 Tauri 主流程中使用。
// ============================================================================

// 引入核心类型，用于 FFI 转换
use crate::type_set::{hostilepile::HostilepileConfig, player::PlayerConfig, xinfa::XinfaConfig};

/// FFI 类型: 不透明指针包装，指向 PlayerConfig
#[repr(C)]
#[derive(Clone)]
pub struct FFIPlayerConfig(*const u8);

/// FFI 类型: 不透明指针包装，指向 HostilepileConfig
#[repr(C)]
#[derive(Clone)]
pub struct FFIHostilepileConfig(*const u8);

/// FFI 类型: 不透明指针包装，指向 XinfaConfig
#[repr(C)]
#[derive(Clone)]
pub struct FFIXinfaConfig(*const u8);

/// FFI 导出函数: 启动伤害计算
/// - 接收三个原始指针（分别指向 PlayerConfig / HostilepileConfig / XinfaConfig）
/// - 返回指向计算结果的裸指针，调用方负责释放
#[unsafe(no_mangle)]
pub extern "C" fn start_calculation(
    player: *const u8,
    hostilepile: *const u8,
    xinfa: *const u8,
) -> *const u8 {
    // 将裸指针解引用为 Rust 引用（调用方保证指针有效）
    let player: &PlayerConfig = unsafe { &*(player as *const PlayerConfig) };
    let hostilepile: &HostilepileConfig = unsafe { &*(hostilepile as *const HostilepileConfig) };
    let xinfa: &XinfaConfig = unsafe { &*(xinfa as *const XinfaConfig) };
    // 执行计算，将结果装箱后返回裸指针
    let result = calculate::start(player.clone(), hostilepile.clone(), xinfa.clone());
    Box::into_raw(Box::new(result)) as *const u8
}
