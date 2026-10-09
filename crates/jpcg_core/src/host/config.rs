// ============================================================================
// host::config — 玩家配置读写与门派列表入口
// DTO ↔ core 转换统一委托 host::conv（单一来源）。
// ============================================================================

use jpcg_api::{ConfigDataDTO, HostileConfigDTO, PlayerConfigDTO, XinfaConfigDTO, XinfaSummaryDTO};

use crate::store;

use super::conv;

/// 保存配置（saved_config.toml）
pub fn save_config(player: PlayerConfigDTO, hostilepile: HostileConfigDTO, xinfa: XinfaConfigDTO) {
    store::save_config(
        conv::player_from_dto(player),
        conv::hostile_from_dto(hostilepile),
        conv::xinfa_from_dto(xinfa),
    );
}

/// 加载默认配置（saved_config.toml，无则默认）
pub fn load_config() -> ConfigDataDTO {
    let saved = store::load_save_config();
    ConfigDataDTO {
        player: conv::player_to_dto(saved.player),
        hostile: conv::hostile_to_dto(saved.hostilepile),
        xinfa_config: conv::xinfa_to_dto(saved.xinfa),
        buff: conv::buff_to_dto(saved.buff),
        coefficient: conv::coefficient_to_dto(saved.coefficient),
    }
}

/// 可用门派列表
pub fn list_professions() -> Vec<XinfaSummaryDTO> {
    store::list_available_professions()
        .into_iter()
        .map(Into::into)
        .collect()
}
