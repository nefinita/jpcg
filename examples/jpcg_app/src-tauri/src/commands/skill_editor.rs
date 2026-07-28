use crate::commands::types::*;

#[tauri::command]
pub fn load_skill_data(profession: String) -> Result<SkillEditorDataDTO, String> {
    let toml_cfg = jpcg_core::skill_editor::load_skills(&profession);
    Ok(SkillEditorDataDTO {
        xinfa: XinfaConfigDTO {
            profession: toml_cfg.xinfa.profession,
            xinfa_name: toml_cfg.xinfa.xinfa_name,
            xinfa_nom: toml_cfg.xinfa.xinfa_nom,
            atk_up: toml_cfg.xinfa.atk_up,
            pofang_up: toml_cfg.xinfa.pofang_up,
            huixin_up: toml_cfg.xinfa.huixin_up,
        },
        version: toml_cfg.version.map(|v| v.into()),
        skills: toml_cfg.skill.into_iter().map(|s| s.into()).collect(),
    })
}

#[tauri::command]
pub fn save_skill_data(profession: String, data: SkillEditorDataDTO) -> Result<(), String> {
    jpcg_core::skill_editor::save_skills(
        &profession,
        data.xinfa.into_core(),
        data.skills.into_iter().map(|s| s.into()).collect(),
        data.version.map(|v| v.into()),
    )
}
