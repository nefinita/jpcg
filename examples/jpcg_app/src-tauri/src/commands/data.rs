use crate::commands::types::SkillPoolItemDTO;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AttributeConfigDocumentDTO {
    pub profession: String,
    pub file_name: String,
    pub content: String,
}

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

#[tauri::command]
pub fn load_attribute_config(profession: String) -> Result<AttributeConfigDocumentDTO, String> {
    let content = jpcg_core::attribute_config_io::read(&profession)?;
    Ok(AttributeConfigDocumentDTO {
        file_name: format!("{}.toml", profession),
        profession,
        content,
    })
}

#[tauri::command]
pub fn save_attribute_config(profession: String, content: String) -> Result<String, String> {
    jpcg_core::attribute_config_io::write(&profession, &content)?;
    Ok(format!("{}.toml", profession))
}
