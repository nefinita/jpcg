pub mod cal;
mod io;
mod log;
pub mod type_set;

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
    use crate::{
        io::save_config,
        type_set::{hostilepile::HostilepileConfig, player::PlayerConfig},
    };

    pub fn save(player: PlayerConfig, hostilepile: HostilepileConfig) {
        save_config(player, hostilepile);
    }
}

pub mod calculate {
    use crate::cal::{CalculateResult, start_calculation};
    use crate::type_set::{
        hostilepile::HostilepileConfig, player::PlayerConfig, xinfa::XinfaConfig,
    };
    pub fn start(
        player: PlayerConfig,
        hostilepile: HostilepileConfig,
        xinfa: XinfaConfig,
    ) -> Vec<CalculateResult> {
        start_calculation(player, hostilepile, xinfa)
    }
}
