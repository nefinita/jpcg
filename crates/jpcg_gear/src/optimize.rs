use crate::model::*;
use std::collections::{BTreeMap, BTreeSet};

impl Constraints {
    fn validate(&self, catalog: &Catalog, loadout: &Loadout) -> Result<(), String> {
        if self.sources.is_empty() {
            return Err("允许来源不能为空".into());
        }
        for (name, bounds) in &self.stats {
            Stats::default().value(name)?;
            if let (Some(min), Some(max)) = (bounds.min, bounds.max)
                && min > max
            {
                return Err(format!("{name} 下限大于上限"));
            }
        }
        let mut slots = BTreeSet::new();
        for choice in &self.locked {
            if !slots.insert(choice.slot) {
                return Err("锁定槽位重复".into());
            }
        }
        for choice in self.locked.iter().chain(&self.banned) {
            if !catalog
                .templates
                .iter()
                .any(|t| t.school == loadout.school && &t.choice == choice)
            {
                return Err("锁定/禁用的模板不存在".into());
            }
        }
        Ok(())
    }
    fn permits(&self, c: &Choice) -> bool {
        self.sources.contains(&c.source)
            && (self.allow_upgraded || c.tier == Tier::Base)
            && !self.banned.contains(c)
            && self
                .locked
                .iter()
                .all(|locked| locked.slot != c.slot || locked == c)
            && self
                .required_sources
                .get(&c.slot)
                .is_none_or(|s| *s == c.source)
            && self
                .banned_sources
                .get(&c.slot)
                .is_none_or(|s| !s.contains(&c.source))
    }
    fn satisfied(&self, composed: &Composed) -> Result<bool, String> {
        for (name, bounds) in &self.stats {
            let value = composed.stats.value(name)?;
            if bounds.min.is_some_and(|min| value < min)
                || bounds.max.is_some_and(|max| value > max)
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

fn dominates(a: &Candidate, b: &Candidate) -> bool {
    let x = &a.composed.stats;
    let y = &b.composed.stats;
    a.score >= b.score && x.qixue >= y.qixue && x.yujin >= y.yujin && x.huajin >= y.huajin
}

/// 有界穷举：小空间给全局最优证明；大空间明确报告截断。回调返回 false 可取消。
/// 遍历固定槽位/来源/档位顺序，所有硬约束在目标函数前检查。基线可不满足新约束。
pub fn optimize_with(
    catalog: &Catalog,
    req: &OptimizeRequest,
    mut evaluate: impl FnMut(&Composed) -> Result<f64, String>,
    mut on_progress: impl FnMut(Progress) -> bool,
) -> Result<OptimizeResult, String> {
    catalog.validate()?;
    req.constraints.validate(catalog, &req.loadout)?;
    if req.max_combinations == 0 || req.max_combinations > 1_000_000 {
        return Err("max_combinations 必须为1..=1000000".into());
    }
    let composed = catalog.compose_validated(&req.loadout)?;
    let baseline_score = evaluate(&composed)?;
    if !baseline_score.is_finite() {
        return Err("目标值必须有限".into());
    }
    let baseline = Candidate {
        score: baseline_score,
        composed,
    };
    let mut candidates = Vec::new();
    let mut combinations = 1u64;
    for slot in Slot::ALL {
        let mut choices: Vec<_> = catalog
            .templates
            .iter()
            .filter(|t| {
                t.school == req.loadout.school
                    && t.choice.slot == slot
                    && req.constraints.permits(&t.choice)
            })
            .map(|t| t.choice.clone())
            .collect();
        choices.sort();
        if choices.is_empty() {
            return Err(format!("槽位 {slot:?} 无候选（锁定或来源约束冲突）"));
        }
        combinations = combinations
            .checked_mul(choices.len() as u64)
            .ok_or("组合数溢出")?;
        candidates.push(choices);
    }
    let mut result = OptimizeResult {
        best: None,
        baseline,
        delta: None,
        pareto: vec![],
        progress: Progress {
            visited: 0,
            evaluated: 0,
            combinations,
        },
        complete: false,
        stop_reason: String::new(),
        warnings: Vec::new(),
    };
    let baseline_armor2 = result
        .baseline
        .composed
        .active_bonuses
        .iter()
        .filter(|b| {
            b.threshold == 2
                && catalog
                    .sets
                    .iter()
                    .any(|s| s.id == b.set && s.kind == SetKind::Armor)
        })
        .count();
    if req
        .loadout
        .choices
        .iter()
        .all(|c| req.constraints.permits(c))
        && (req.constraints.allow_cross_source_2plus2 || baseline_armor2 <= 1)
        && req.constraints.satisfied(&result.baseline.composed)?
    {
        result.best = Some(result.baseline.clone());
        result.pareto.push(result.baseline.clone());
    }
    let mut indices = [0usize; 12];
    loop {
        if !on_progress(result.progress) {
            result.stop_reason = "cancelled".into();
            break;
        }
        if result.progress.visited >= req.max_combinations {
            result.stop_reason = "budget_exhausted".into();
            break;
        }
        let mut loadout = req.loadout.clone();
        loadout.choices = candidates
            .iter()
            .zip(indices)
            .map(|(v, i)| v[i].clone())
            .collect();
        let composed = catalog.compose_validated(&loadout)?;
        result.progress.visited += 1;
        let armor2 = composed
            .active_bonuses
            .iter()
            .filter(|b| {
                b.threshold == 2
                    && catalog
                        .sets
                        .iter()
                        .any(|s| s.id == b.set && s.kind == SetKind::Armor)
            })
            .count();
        // 跨来源叠加作为选择约束：不通过关闭一组奖励篡改已选装备。
        if (req.constraints.allow_cross_source_2plus2 || armor2 <= 1)
            && req.constraints.satisfied(&composed)?
        {
            let score = evaluate(&composed)?;
            if !score.is_finite() {
                return Err("目标值必须有限".into());
            }
            result.progress.evaluated += 1;
            let candidate = Candidate { composed, score };
            if result.best.as_ref().is_none_or(|b| score > b.score) {
                result.best = Some(candidate.clone());
            }
            if !result.pareto.iter().any(|p| dominates(p, &candidate)) {
                result.pareto.retain(|p| !dominates(&candidate, p));
                result.pareto.push(candidate);
            }
        }
        let mut digit = 12;
        while digit > 0 {
            digit -= 1;
            indices[digit] += 1;
            if indices[digit] < candidates[digit].len() {
                break;
            }
            indices[digit] = 0;
        }
        if indices.iter().all(|i| *i == 0) {
            result.complete = true;
            result.stop_reason = "exhausted".into();
            break;
        }
    }
    result.delta = result
        .best
        .as_ref()
        .map(|b| b.score - result.baseline.score);
    result.warnings = result.baseline.composed.warnings.clone();
    if let Some(best) = &result.best {
        for warning in &best.composed.warnings {
            if !result.warnings.contains(warning) {
                result.warnings.push(warning.clone());
            }
        }
    }
    result.warnings.push("加速等级仅合成/约束/导出；当前目标不重排技能时序。生存Pareto使用气血/御劲/化劲点数，不代表实战承伤".into());
    if !result.complete {
        result
            .warnings
            .push("搜索未遍历全部组合，不保证全局最优；Pareto仅覆盖已评估候选".into());
    }
    on_progress(result.progress);
    Ok(result)
}

/// 保持现有 DTO 不变，纯转换可供 #91 静态集成使用。
pub fn player(composed: &Composed) -> jpcg_api::PlayerConfigDTO {
    let stats = &composed.stats;
    jpcg_api::PlayerConfigDTO {
        jcsx: main_name(composed.profile.zhushuxing).into(),
        jichu_shuxing: stats.zhushuxing,
        jichu_gongji: stats.gongji,
        huixin_dengji: stats.huixin,
        huixin_xiaoguo: stats.huixiao,
        pofang_dengji: stats.pofang,
        wuqi_shanghai: stats.wuqi_shanghai,
        jiasu_dengji: stats.jiasu,
    }
}

/// 保留 DTO 的加速等级；PlayerConfig::new 为兼容旧调用仍将该字段初始化为 0。
fn core_player(p: &jpcg_api::PlayerConfigDTO) -> jpcg_core::type_set::player::PlayerConfig {
    let mut player = jpcg_core::type_set::player::PlayerConfig::new(
        p.jcsx.clone(),
        p.jichu_shuxing,
        p.jichu_gongji,
        p.huixin_dengji,
        p.huixin_xiaoguo,
        p.pofang_dengji,
        p.wuqi_shanghai,
    );
    player.jiasu_dengji = p.jiasu_dengji;
    player
}
fn main_name(stat: MainStat) -> &'static str {
    match stat {
        MainStat::Liliang => "力道",
        MainStat::Shenfa => "身法",
        MainStat::Gengu => "根骨",
        MainStat::Yuanqi => "元气",
    }
}
fn request(config: &jpcg_api::CalculateRequest, composed: &Composed) -> jpcg_api::CalculateRequest {
    jpcg_api::CalculateRequest {
        player: player(composed),
        hostile: config.hostile.clone(),
        xinfa_config: config.xinfa_config.clone(),
        buff: config.buff.clone(),
        coefficient: config.coefficient.clone(),
        value_set: config.value_set.clone(),
    }
}

pub fn optimize(
    catalog: &Catalog,
    req: &OptimizeRequest,
    on_progress: impl FnMut(Progress) -> bool,
) -> Result<OptimizeResult, String> {
    catalog.validate()?;
    validate_context(catalog, req)?;
    let skill_table = jpcg_core::skill_editor::load_skills(&req.loadout.school);
    let x = &skill_table.xinfa;
    let dto = &req.config.xinfa_config;
    if skill_table.skill.is_empty()
        || x.xinfa_nom != dto.xinfa_nom
        || x.atk_up != dto.atk_up
        || x.pofang_up != dto.pofang_up
        || x.huixin_up != dto.huixin_up
    {
        return Err("技能数据缺失或心法系数与现有计算器数据不一致".into());
    }
    let mut cache = BTreeMap::new();
    optimize_with(
        catalog,
        req,
        |composed| {
            // CD状态也属于缓存键；不能把未应用CD与纯属性方案混淆。
            let key = serde_json::to_string(&(&composed.stats, &composed.cooldowns))
                .map_err(|e| e.to_string())?;
            if let Some(value) = cache.get(&key) {
                return Ok(*value);
            }
            let value = score(req, composed, &skill_table.skill, &skill_table.xinfa)?;
            cache.insert(key, value);
            Ok(value)
        },
        on_progress,
    )
}

fn validate_context(catalog: &Catalog, req: &OptimizeRequest) -> Result<(), String> {
    let config = &req.config;
    let b = &config.buff;
    let c = &config.coefficient;
    let x = &config.xinfa_config;
    if [
        b.base_atk_pct,
        b.huixin_pct,
        b.huixiao_pct,
        b.pofang_pct,
        b.wushi_fangyu_pct,
        b.shanghai_pct,
        c.pofang_xishu,
        c.huixin_xishu,
        c.huixiao_xishu,
        c.yujin_xishu,
        c.yuhui_xishu,
        c.huajin_xishu,
        c.fangyu_xishu,
        c.pvp_global_jianshang,
        x.atk_up,
        x.pofang_up,
        x.huixin_up,
    ]
    .iter()
    .any(|v| !v.is_finite())
    {
        return Err("计算配置包含非有限数值".into());
    }

    let profile = catalog
        .schools
        .iter()
        .find(|s| s.id == req.loadout.school)
        .ok_or("未知门派")?;
    if config.xinfa_config.profession != req.loadout.school
        || config.value_set.as_deref() != Some(&catalog.version.value_set)
        || config.xinfa_config.xinfa_nom != main_name(profile.zhushuxing)
    {
        return Err("计算配置的门派/主属性/显式数值集与模板不一致".into());
    }
    let is_internal = matches!(profile.zhushuxing, MainStat::Gengu | MainStat::Yuanqi);
    if is_internal != (profile.gongji_type == DamageType::Neigong) {
        return Err("门派主属性与攻击类型不匹配".into());
    }
    let value_set = jpcg_core::host::values::list_value_sets()
        .into_iter()
        .find(|v| v.id == catalog.version.value_set && v.available)
        .ok_or("指定数值集不可用（禁止静默回退）")?;
    if value_set.source != "data" {
        return Err("优化需可追溯的 data/values 数值集".into());
    }
    // core 的部分旧换算仍使用 u32 × 1024；在适配器入口明确拒绝越界。
    if config.hostile.waigong_fangyu > u32::MAX / 1024
        || config.hostile.neigong_fangyu > u32::MAX / 1024
        || config.hostile.huajin_dengji > u32::MAX / 1024
        || config.hostile.yujin_dengji > u32::MAX / 1024
    {
        return Err("目标等级超出当前core安全换算范围".into());
    }
    match &req.objective {
        Objective::Expected { weights } => {
            if weights.is_empty() || weights.values().any(|v| !v.is_finite() || *v <= 0.0) {
                return Err("期望伤害需明确指定技能及有限正权重".into());
            }
        }
        Objective::KillProbability { steps, samples, .. } => {
            if steps.is_empty() || steps.len() > 256 || *samples == 0 || *samples > 50_000 {
                return Err("击杀率需1..256步及1..50000采样".into());
            }
            if config.hostile.target_hp == 0 && config.hostile.max_hp == 0 {
                return Err("击杀率需目标气血".into());
            }
        }
    }
    Ok(())
}

fn score(
    req: &OptimizeRequest,
    composed: &Composed,
    skills: &[jpcg_core::type_set::skilltype::Skilltype],
    xinfa: &jpcg_core::type_set::xinfa::XinfaConfig,
) -> Result<f64, String> {
    if composed.stats.pofang > u32::MAX / 1024 {
        return Err("破防超出当前core安全换算范围".into());
    }
    let config = request(&req.config, composed);
    match &req.objective {
        Objective::Expected { weights } => {
            // 直接复用 core 同一个 q_cal，避免每候选重复加载技能及 host 的 stdout 日志。
            use jpcg_core::type_set::{
                buff::BuffConfig, coefficient::CoefficientConfig, hostilepile::HostilepileConfig,
            };
            let player = core_player(&config.player);
            let hostile: HostilepileConfig = serde_json::from_value(
                serde_json::to_value(&config.hostile).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            let buff: BuffConfig = serde_json::from_value(
                serde_json::to_value(&config.buff).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            let base = jpcg_core::host::values::value_set_constant(config.value_set.as_deref());
            let coeff = CoefficientConfig::from_dto(&config.coefficient, &base);
            let mut score = 0.0;
            for (name, weight) in weights {
                let matches: Vec<_> = skills.iter().filter(|s| &s.skill_name == name).collect();
                if matches.len() != 1 {
                    return Err(format!("技能 {name} 不存在或名称不唯一"));
                }
                let damage = jpcg_core::engine::atkcal::JpcgConfig::new_with_config(
                    &player, &hostile, matches[0], xinfa, &buff, &coeff,
                )
                .q_cal();
                score += damage.q_damage as f64 * weight;
            }
            Ok(score)
        }
        Objective::KillProbability {
            steps,
            samples,
            seed,
            allow_unapplied_cd,
        } => {
            if !composed.cooldowns.is_empty() && !allow_unapplied_cd {
                return Err("连招引擎未应用4件套CD；请换约束或明确 allow_unapplied_cd=true".into());
            }
            let result = jpcg_combo::host::calculate_combo(
                steps.clone(),
                config.player,
                config.hostile,
                config.xinfa_config,
                config.buff,
                config.coefficient,
                config.value_set,
                jpcg_combo::engine::ComboConfig {
                    samples: *samples,
                    seed: Some(*seed),
                },
            )?;
            Ok(result.final_kill_prob)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::core_player;

    #[test]
    fn expected_damage_adapter_preserves_acceleration_rating() {
        // 构造器默认加速为0，适配器必须保留显式值，也兼容旧配置的0值。
        for rating in [0, 12345] {
            let dto = jpcg_api::PlayerConfigDTO {
                jcsx: "根骨".into(),
                jiasu_dengji: rating,
                ..Default::default()
            };
            assert_eq!(core_player(&dto).jiasu_dengji, rating);
        }
    }
}
