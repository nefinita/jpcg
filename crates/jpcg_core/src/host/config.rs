// ============================================================================
// host::config — 玩家配置读写与门派列表入口
// ============================================================================

use jpcg_api::{
    BuffConfigDTO, CoefficientConfigDTO, HostileConfigDTO, PlayerConfigDTO, XinfaConfigDTO,
    XinfaSummaryDTO,
};

use crate::store;

/// 保存配置（saved_config.toml）
pub fn save_config(player: PlayerConfigDTO, hostilepile: HostileConfigDTO, xinfa: XinfaConfigDTO) {
    let player_core = crate::type_set::player::PlayerConfig::new(
        player.jcsx,
        player.jichu_shuxing,
        player.jichu_gongji,
        player.huixin_dengji,
        player.huixin_xiaoguo,
        player.pofang_dengji,
        player.wuqi_shanghai,
    );
    let hostile_core = crate::type_set::hostilepile::HostilepileConfig {
        waigong_fangyu: hostilepile.waigong_fangyu,
        neigong_fangyu: hostilepile.neigong_fangyu,
        yujin_dengji: hostilepile.yujin_dengji,
        huajin_dengji: hostilepile.huajin_dengji,
        jianshang_bili: hostilepile.jianshang_bili,
        target_hp: hostilepile.target_hp,
        max_hp: hostilepile.max_hp,
        current_hp: hostilepile.current_hp,
    };
    let xinfa_core = crate::type_set::xinfa::XinfaConfig::new(
        xinfa.profession,
        xinfa.xinfa_name,
        xinfa.xinfa_nom,
        xinfa.atk_up,
        xinfa.pofang_up,
        xinfa.huixin_up,
    );
    store::save_config(player_core, hostile_core, xinfa_core);
}

/// 加载默认配置（saved_config.toml，无则默认）
pub fn load_config() -> jpcg_api::ConfigDataDTO {
    let saved = store::load_save_config();
    jpcg_api::ConfigDataDTO {
        player: PlayerConfigDTO {
            jcsx: saved.player.jcsx,
            jichu_shuxing: saved.player.jichu_shuxing,
            jichu_gongji: saved.player.jichu_gongji,
            huixin_dengji: saved.player.huixin_dengji,
            huixin_xiaoguo: saved.player.huixin_xiaoguo,
            pofang_dengji: saved.player.pofang_dengji,
            wuqi_shanghai: saved.player.wuqi_shanghai,
        },
        hostile: HostileConfigDTO {
            waigong_fangyu: saved.hostilepile.waigong_fangyu,
            neigong_fangyu: saved.hostilepile.neigong_fangyu,
            yujin_dengji: saved.hostilepile.yujin_dengji,
            huajin_dengji: saved.hostilepile.huajin_dengji,
            jianshang_bili: saved.hostilepile.jianshang_bili,
            target_hp: saved.hostilepile.target_hp,
            max_hp: saved.hostilepile.max_hp,
            current_hp: saved.hostilepile.current_hp,
        },
        xinfa_config: XinfaConfigDTO {
            profession: saved.xinfa.profession,
            xinfa_name: saved.xinfa.xinfa_name,
            xinfa_nom: saved.xinfa.xinfa_nom,
            atk_up: saved.xinfa.atk_up,
            pofang_up: saved.xinfa.pofang_up,
            huixin_up: saved.xinfa.huixin_up,
        },
        buff: BuffConfigDTO {
            base_atk_pct: saved.buff.base_atk_pct,
            huixin_pct: saved.buff.huixin_pct,
            huixiao_pct: saved.buff.huixiao_pct,
            pofang_pct: saved.buff.pofang_pct,
            wushi_fangyu_pct: saved.buff.wushi_fangyu_pct,
            shanghai_pct: saved.buff.shanghai_pct,
            mode_is_point: saved.buff.mode_is_point,
        },
        coefficient: CoefficientConfigDTO {
            pofang_xishu: saved.coefficient.pofang_xishu,
            huixin_xishu: saved.coefficient.huixin_xishu,
            huixiao_xishu: saved.coefficient.huixiao_xishu,
            huajin_xishu: saved.coefficient.huajin_xishu,
            fangyu_xishu: saved.coefficient.fangyu_xishu,
            pvp_global_jianshang: saved.coefficient.pvp_global_jianshang,
        },
    }
}

/// 可用门派列表
pub fn list_professions() -> Vec<XinfaSummaryDTO> {
    store::list_available_professions()
        .into_iter()
        .map(Into::into)
        .collect()
}
