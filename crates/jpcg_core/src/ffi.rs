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
    let result = crate::calculate::start(player.clone(), hostilepile.clone(), xinfa.clone());
    Box::into_raw(Box::new(result)) as *const u8
}
