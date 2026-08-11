use crate::cal::atkcal::{DerivativeSet, JpcgConfig};
use crate::type_set::{
    buff::BuffConfig, coefficient::CoefficientConfig,
    hostilepile::HostilepileConfig, player::PlayerConfig, skilltype::Skilltype,
    xinfa::XinfaConfig,
};
use serde::Serialize;

/// 单个属性对单个技能的导数
#[derive(Debug, Clone, Serialize)]
pub struct SkillDerivative {
    pub skill_name: String,
    pub derivative: f32,
}

/// 单个属性的全技能求导结果
#[derive(Debug, Clone, Serialize)]
pub struct DerivativeEntry {
    pub attr_name: String,
    pub attr_id: String,
    pub current_value: f32,
    pub total_derivative: f32,
    pub per_skill: Vec<SkillDerivative>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CritVsPofang {
    pub better: String,
    pub huixin_total: f32,
    pub pofang_total: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopAttr {
    pub attr_name: String,
    pub attr_id: String,
    pub total_derivative: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct OptimizeRecommendation {
    pub crit_vs_pofang: CritVsPofang,
    pub top3: Vec<TopAttr>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DerivativesOutput {
    pub derivatives: Vec<DerivativeEntry>,
    pub recommendation: OptimizeRecommendation,
}

const ATTR_META: [(&str, &str, fn(&DerivativeSet) -> f32, fn(&PlayerConfig) -> f32); 6] = [
    ("基础属性",   "jichu_shuxing",   |d| d.d_jichu_shuxing,   |p| p.jichu_shuxing as f32),
    ("基础攻击",   "jichu_gongji",    |d| d.d_jichu_gongji,    |p| p.jichu_gongji as f32),
    ("会心等级",   "huixin_dengji",   |d| d.d_huixin_dengji,   |p| p.huixin_dengji as f32),
    ("会心效果",   "huixin_xiaoguo",  |d| d.d_huixin_xiaoguo,  |p| p.huixin_xiaoguo as f32),
    ("破防等级",   "pofang_dengji",   |d| d.d_pofang_dengji,   |p| p.pofang_dengji as f32),
    ("武器伤害",   "wuqi_shanghai",   |d| d.d_wuqi_shanghai,   |p| p.wuqi_shanghai as f32),
];

pub fn compute_derivatives(
    player: &PlayerConfig,
    hostile: &HostilepileConfig,
    buff: &BuffConfig,
    coeff: &CoefficientConfig,
    xinfa: &XinfaConfig,
    skills: &[Skilltype],
) -> DerivativesOutput {
    // 每属性: 收集各技能导数
    let mut per_attr: Vec<Vec<SkillDerivative>> = (0..6).map(|_| Vec::with_capacity(skills.len())).collect();
    let mut attr_totals = [0.0f32; 6];

    for skill in skills {
        let config = JpcgConfig::new_with_config(
            player.clone(),
            hostile.clone(),
            skill.clone(),
            xinfa.clone(),
            buff.clone(),
            coeff.clone(),
        );
        let dwd = config.q_cal_with_derivatives();

        for i in 0..6 {
            let d = (ATTR_META[i].2)(&dwd.derivatives);
            per_attr[i].push(SkillDerivative {
                skill_name: skill.skill_name.clone(),
                derivative: d,
            });
            attr_totals[i] += d;
        }
    }

    let mut derivatives: Vec<DerivativeEntry> = ATTR_META
        .iter()
        .enumerate()
        .map(|(i, (name, id, _, get_cur))| DerivativeEntry {
            attr_name: name.to_string(),
            attr_id: id.to_string(),
            current_value: get_cur(player),
            total_derivative: attr_totals[i],
            per_skill: per_attr[i].clone(),
        })
        .collect();

    derivatives.sort_by(|a, b| b.total_derivative.partial_cmp(&a.total_derivative).unwrap_or(std::cmp::Ordering::Equal));

    let huixin_total = attr_totals[2];
    let pofang_total = attr_totals[4];

    let better = if huixin_total >= pofang_total {
        "会心等级"
    } else {
        "破防等级"
    };

    let top3: Vec<TopAttr> = derivatives
        .iter()
        .take(3)
        .map(|d| TopAttr {
            attr_name: d.attr_name.clone(),
            attr_id: d.attr_id.clone(),
            total_derivative: d.total_derivative,
        })
        .collect();

    DerivativesOutput {
        derivatives,
        recommendation: OptimizeRecommendation {
            crit_vs_pofang: CritVsPofang {
                better: better.to_string(),
                huixin_total,
                pofang_total,
            },
            top3,
        },
    }
}
