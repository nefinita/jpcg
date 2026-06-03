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
            target_hp: saved.hostilepile.target_hp,
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
    })
}

#[tauri::command]
pub fn list_professions_cmd() -> Result<Vec<XinfaSummaryDTO>, String> {
    let list = jpcg_core::profession_list::list_available();
    Ok(list.into_iter().map(|s| XinfaSummaryDTO {
        value: s.value,
        label: s.label,
        nom: s.nom,
        version_label: s.version_label,
    }).collect())
}

#[tauri::command]
pub fn load_profession_config(profession: String) -> Result<XinfaConfigDTO, String> {
    let toml_cfg = jpcg_core::load_config::show_config(&profession);

    Ok(XinfaConfigDTO {
        profession: toml_cfg.xinfa.profession,
        xinfa_name: toml_cfg.xinfa.xinfa_name,
        xinfa_nom: toml_cfg.xinfa.xinfa_nom,
        atk_up: toml_cfg.xinfa.atk_up,
        pofang_up: toml_cfg.xinfa.pofang_up,
        huixin_up: toml_cfg.xinfa.huixin_up,
    })
}
