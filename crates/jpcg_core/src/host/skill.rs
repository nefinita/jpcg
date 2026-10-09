// ============================================================================
// host::skill — 技能编辑器入口
// ============================================================================

use jpcg_api::SkillEditorDataDTO;

use crate::store;

use super::conv::{self, toml_to_editor_data};

/// 加载心法技能数据（技能编辑器用）
pub fn load_skill_data(profession: String) -> Result<SkillEditorDataDTO, String> {
    let toml_cfg = store::load_config(&profession);
    Ok(toml_to_editor_data(&toml_cfg))
}

/// 保存心法技能数据（技能编辑器用）
pub fn save_skill_data(profession: String, data: SkillEditorDataDTO) -> Result<(), String> {
    let xinfa = conv::xinfa_from_dto(data.xinfa);
    let skills = data.skills.into_iter().map(Into::into).collect();
    let version = data.version.map(Into::into);
    store::save_skill_toml(
        &profession,
        store::TomlConfig {
            xinfa,
            skill: skills,
            version,
        },
    )
}

/// 技能池条目（连招编辑器下拉）
pub fn load_skill_pool(profession: String) -> Vec<jpcg_api::SkillPoolItemDTO> {
    store::load_config(&profession)
        .skill
        .iter()
        .map(conv::skilltype_to_pool_item)
        .collect()
}
