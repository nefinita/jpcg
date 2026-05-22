// ============================================================================
// jpcg_core — 剑心计算核心库
// 提供剑网3 伤害计算的核心数据结构、配置加载与保存、以及计算入口。
// 该库作为底层 crate，被 jpcg_update、jpcg_app (Tauri) 等上层 crate 依赖。
// ============================================================================

pub mod cal;
mod io;
mod log;
pub mod type_set;
pub mod ffi;

pub mod load_config {
    use crate::io::{SaveConfig, TomlConfig, load_config, load_save_config};

    pub fn default_load() -> SaveConfig {
        load_save_config()
    }

    pub fn show_config(profession: &str) -> TomlConfig {
        load_config(profession)
    }
}

pub mod save_config {
    use crate::io::save_config;
    use crate::type_set::{hostilepile::HostilepileConfig, player::PlayerConfig};

    pub fn save(
        player: PlayerConfig,
        hostilepile: HostilepileConfig,
        xinfa: crate::type_set::xinfa::XinfaConfig,
    ) {
        save_config(player, hostilepile, xinfa);
    }
}

pub mod calculate {
    use std::io::Error;

    use crate::cal;
    use crate::type_set::{
        hostilepile::HostilepileConfig, player::PlayerConfig, xinfa::XinfaConfig,
    };

    pub fn start(
        player: PlayerConfig,
        hostilepile: HostilepileConfig,
        xinfa: XinfaConfig,
    ) -> Result<Vec<cal::CalculateResult>, Error> {
        cal::start_calculation(player, hostilepile, xinfa)
    }
}
