use crate::commands::types::*;

#[tauri::command]
pub fn save_config_cmd(
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
) -> Result<(), String> {
    jpcg_core::save_config::save(player.into_core(), hostile.into_core(), xinfa.into_core());
    Ok(())
}

#[tauri::command]
pub fn load_config_cmd() -> Result<CalculateRequest, String> {
    let saved = jpcg_core::load_config::default_load();

    Ok(CalculateRequest {
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
        },
        xinfa_config: XinfaConfigDTO {
            xinfa_name: saved.xinfa.xinfa_name,
            xinfa_nom: saved.xinfa.xinfa_nom,
            atk_up: saved.xinfa.atk_up,
            pofang_up: saved.xinfa.pofang_up,
            huixin_up: saved.xinfa.huixin_up,
        },
    })
}

#[tauri::command]
pub fn load_profession_config(profession: String) -> Result<XinfaConfigDTO, String> {
    let toml_cfg = jpcg_core::load_config::show_config(&profession);

    Ok(XinfaConfigDTO {
        xinfa_name: toml_cfg.xinfa.xinfa_name,
        xinfa_nom: toml_cfg.xinfa.xinfa_nom,
        atk_up: toml_cfg.xinfa.atk_up,
        pofang_up: toml_cfg.xinfa.pofang_up,
        huixin_up: toml_cfg.xinfa.huixin_up,
    })
}
