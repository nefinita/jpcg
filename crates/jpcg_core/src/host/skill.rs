// ============================================================================
// host::skill — 技能编辑器入口
// ============================================================================

use jpcg_api::SkillEditorDataDTO;

use crate::store;

use super::conv::toml_to_editor_data;

/// 加载心法技能数据（技能编辑器用）
pub fn load_skill_data(profession: String) -> Result<SkillEditorDataDTO, String> {
    let toml_cfg = store::load_config(&profession);
    Ok(toml_to_editor_data(&toml_cfg))
}

/// 保存心法技能数据（技能编辑器用）
pub fn save_skill_data(
    profession: String,
    data: SkillEditorDataDTO,
) -> Result<(), String> {
    let xinfa = crate::type_set::xinfa::XinfaConfig::new(
        data.xinfa.profession,
        data.xinfa.xinfa_name,
        data.xinfa.xinfa_nom,
        data.xinfa.atk_up,
        data.xinfa.pofang_up,
        data.xinfa.huixin_up,
    );
    let skills = data.skills.into_iter().map(Into::into).collect();
    let version = data.version.map(Into::into);
    store::save_skill_toml(&profession, store::TomlConfig {
        xinfa,
        skill: skills,
        version,
    })
}

/// 技能池条目（连招编辑器下拉）
pub fn load_skill_pool(profession: String) -> Vec<jpcg_api::SkillPoolItemDTO> {
    let toml_cfg = store::load_config(&profession);
    toml_cfg
        .skill
        .into_iter()
        .map(|s| jpcg_api::SkillPoolItemDTO {
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
        })
        .collect()
}