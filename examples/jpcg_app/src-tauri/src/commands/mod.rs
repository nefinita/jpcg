pub mod types;
use types::*;

/// 🔢 执行伤害计算
#[tauri::command]
pub async fn calculate_damage(req: CalculateRequest) -> Result<Vec<SkillResultDTO>, String> {
    
    // 1. 转换前端数据 -> 核心库类型
    let player = req.player.into_core();
    let hostile = req.hostile.into_core();
    
    // 2. 构建心法配置（优先使用传入的扩展配置，否则用默认）
    let xinfa = req.xinfa_config.into_core();
    
    // 3. 调用核心计算接口
    let results = jpcg_core::calculate::start(player, hostile, xinfa);
    
    // 4. 转换结果 -> 前端友好格式
    Ok(results.into_iter().map(SkillResultDTO::from).collect())
}

/// 💾 保存玩家配置
#[tauri::command]
pub fn save_config_cmd(
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
) -> Result<(), String> {
    jpcg_core::save_config::save(
        player.into_core(),
        hostile.into_core(),
        xinfa.into_core(),
    );
    Ok(())
}

/// 📥 加载默认配置
#[tauri::command]
pub fn load_config_cmd() -> Result<types::CalculateRequest, String> {
    let saved = jpcg_core::load_config::default_load();
    
    // 注意：这里需要根据 SaveConfig 的实际结构进行映射
    // 假设 SaveConfig 包含 player/hostile/xinfa 字段
    Ok(types::CalculateRequest {
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
        }
    })
}
    

/// 🎯 按职业加载特定配置（扩展接口）
#[tauri::command]
pub fn load_profession_config(profession: String) -> Result<types::XinfaConfigDTO, String> {
    let toml_cfg = jpcg_core::load_config::show_config(&profession);
    
    // 将 TomlConfig 转换为前端可用的 DTO
    Ok(types::XinfaConfigDTO {
            xinfa_name: toml_cfg.xinfa.xinfa_name,
            xinfa_nom: toml_cfg.xinfa.xinfa_nom,
            atk_up: toml_cfg.xinfa.atk_up,
            pofang_up: toml_cfg.xinfa.pofang_up,
            huixin_up: toml_cfg.xinfa.huixin_up,
    })
}