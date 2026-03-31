use jpcg_core::calculate::start;
use jpcg_core::load_config::default_load;
use jpcg_core::save_config::save;
use jpcg_core::type_set::hostilepile::HostilepileConfig;
use jpcg_core::type_set::player::PlayerConfig;
use jpcg_core::type_set::xinfa::XinfaConfig;
fn main() {
    let config = default_load();
    let result = start(
        PlayerConfig::default(),
        HostilepileConfig::default(),
        XinfaConfig::default(),
    );
    save(
        PlayerConfig::default(),
        HostilepileConfig::default(),
        XinfaConfig::default(),
    );
}
