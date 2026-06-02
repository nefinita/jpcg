use crate::commands::types::SkillPoolItemDTO;

#[tauri::command]
pub fn load_skill_pool(profession: String) -> Vec<SkillPoolItemDTO> {
    let toml_cfg = jpcg_core::load_config::show_config(&profession);
    toml_cfg.skill.into_iter().map(|s| SkillPoolItemDTO {
        skill_name: s.skill_name,
        skill_id: s.skill_id,
        base_damage1: s.base_damage1,
        base_damage2: s.base_damage2,
        atk_xishu: s.atk_xishu,
        watk_xishu: s.watk_xishu,
        hit_up: s.hit_up,
        huixin_up: s.huixin_up,
        huixiao_up: s.huixiao_up,
        wushifangyu: s.wushifangyu,
        wushihuajin: s.wushihuajin,
        dot_flag: s.dot_flag,
    }).collect()
}
