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

use crate::type_set::{hostilepile::HostilepileConfig, player::PlayerConfig, xinfa::XinfaConfig};

#[repr(C)]
#[derive(Clone)]
pub struct FFIPlayerConfig(*const u8);

#[repr(C)]
#[derive(Clone)]
pub struct FFIHostilepileConfig(*const u8);

#[repr(C)]
#[derive(Clone)]
pub struct FFIXinfaConfig(*const u8);

#[unsafe(no_mangle)]
pub extern "C" fn start_calculation(
    player: *const u8,
    hostilepile: *const u8,
    xinfa: *const u8,
) -> *const u8 {
    let player: &PlayerConfig = unsafe { &*(player as *const PlayerConfig) };
    let hostilepile: &HostilepileConfig = unsafe { &*(hostilepile as *const HostilepileConfig) };
    let xinfa: &XinfaConfig = unsafe { &*(xinfa as *const XinfaConfig) };
    //输出*const u8
    let result = calculate::start(player.clone(), hostilepile.clone(), xinfa.clone());
    Box::into_raw(Box::new(result)) as *const u8
}
