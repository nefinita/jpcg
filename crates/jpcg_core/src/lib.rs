// ============================================================================
// jpcg_core - JX3 PVP 计算核心库
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

pub mod combo_io {
    use crate::io;
    use crate::type_set::combo::ComboPreset;

    pub fn list_presets() -> Vec<String> {
        io::list_combo_presets()
    }

    pub fn load_preset(name: &str) -> Option<ComboPreset> {
        io::load_combo_preset(name)
    }

    pub fn save_preset(preset: &ComboPreset) -> Result<(), String> {
        io::save_combo_preset(preset)
    }

    pub fn delete_preset(name: &str) -> Result<(), String> {
        io::delete_combo_preset(name)
    }
}

pub mod config_io {
    use crate::io;

    pub fn export_config() -> Result<String, String> {
        io::export_config_toml()
    }

    pub fn import_config(toml_str: &str) -> Result<(), String> {
        io::import_config_toml(toml_str)
    }
}

pub mod profession_list {
    use crate::type_set::xinfa::XinfaSummary;

    pub fn list_available() -> Vec<XinfaSummary> {
        crate::io::list_available_professions()
    }
}

pub mod skill_editor {
    use crate::io::{self, TomlConfig};
    use crate::type_set::skilltype::Skilltype;
    use crate::type_set::xinfa::{VersionInfo, XinfaConfig};

    pub fn load_skills(profession: &str) -> TomlConfig {
        io::load_config(profession)
    }

    pub fn save_skills(
        profession: &str,
        xinfa: XinfaConfig,
        skills: Vec<Skilltype>,
        version: Option<VersionInfo>,
    ) -> Result<(), String> {
        let config = TomlConfig { xinfa, skill: skills, version };
        io::save_skill_toml(profession, config)
    }
}

pub mod derivatives {
    use crate::cal;
    use crate::type_set::{
        buff::BuffConfig, coefficient::CoefficientConfig,
        hostilepile::HostilepileConfig, player::PlayerConfig, skilltype::Skilltype,
        xinfa::XinfaConfig,
    };

    pub fn compute_derivatives(
        player: &PlayerConfig,
        hostile: &HostilepileConfig,
        buff: &BuffConfig,
        coeff: &CoefficientConfig,
        xinfa: &XinfaConfig,
        skills: &[Skilltype],
    ) -> cal::derivatives::DerivativesOutput {
        cal::derivatives::compute_derivatives(player, hostile, buff, coeff, xinfa, skills)
    }
}

pub mod calculate {
    use std::io::Error;

    use crate::cal;
    use crate::type_set::{
        buff::BuffConfig, coefficient::CoefficientConfig,
        hostilepile::HostilepileConfig, player::PlayerConfig, xinfa::XinfaConfig,
    };

    pub fn start(
        player: PlayerConfig,
        hostilepile: HostilepileConfig,
        xinfa: XinfaConfig,
    ) -> Result<Vec<cal::CalculateResult>, Error> {
        cal::start_calculation(player, hostilepile, xinfa)
    }

    pub fn start_with_config(
        player: PlayerConfig,
        hostilepile: HostilepileConfig,
        xinfa: XinfaConfig,
        buff: &BuffConfig,
        coeff: &CoefficientConfig,
    ) -> Result<Vec<cal::CalculateResult>, Error> {
        cal::start_calculation_with_config(player, hostilepile, xinfa, buff, coeff)
    }

    pub fn start_combo(
        skills: &[crate::type_set::skilltype::Skilltype],
        player: &PlayerConfig,
        hostilepile: &HostilepileConfig,
        xinfa: &XinfaConfig,
        buff: &BuffConfig,
        coeff: &CoefficientConfig,
    ) -> cal::kill_prob::ComboResult {
        cal::kill_prob::calculate_combo(skills, player, hostilepile, xinfa, buff, coeff)
    }
}
