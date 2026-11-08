use crate::model::*;
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

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
        for (set, min) in &self.required_sets {
            if !catalog.sets.iter().any(|s| s.id == *set) {
                return Err(format!("必须套装不存在：{set}"));
            }
            if *min == 0 || *min > 6 {
                return Err(format!("必须套装件数超出范围：{set}={min}"));
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
        for (set, min) in &self.required_sets {
            if composed.set_counts.get(set).copied().unwrap_or(0) < *min {
                return Ok(false);
            }
        }
        for (name, bounds) in &self.stats {
            let value = effective_value(composed, name)?;
            if bounds.min.is_some_and(|min| value < min)
                || bounds.max.is_some_and(|max| value > max)
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// 规则模式的约束使用派生点数；旧模式仍为原始合计。
fn effective_value(composed: &Composed, name: &str) -> Result<u32, String> {
    if let Some(p) = &composed.panel {
        match name {
            "qixue" => return Ok(p.qixue),
            "gongji" => return Ok(p.gongji),
            "huixin" => return Ok(p.huixin),
            "pofang" => return Ok(p.pofang),
            "zhushuxing" => return Ok(p.main_value(composed.profile.zhushuxing)),
            "gengu" => return Ok(p.gengu),
            "liliang" => return Ok(p.liliang),
            "shenfa" => return Ok(p.shenfa),
            "yuanqi" => return Ok(p.yuanqi),
            _ => (),
        }
    }
    composed.stats.value(name)
}

fn effective_qixue(composed: &Composed) -> u32 {
    composed
        .panel
        .as_ref()
        .map_or(composed.stats.qixue, |p| p.qixue)
}

fn dominates(a: &Candidate, b: &Candidate) -> bool {
    let x = &a.composed.stats;
    let y = &b.composed.stats;
    a.score >= b.score
        && effective_qixue(&a.composed) >= effective_qixue(&b.composed)
        && x.yujin >= y.yujin
        && x.huajin >= y.huajin
}

fn stats_ge(a: &Stats, b: &Stats) -> bool {
    a.zhushuxing >= b.zhushuxing
        && a.tizhi >= b.tizhi
        && a.liliang >= b.liliang
        && a.shenfa >= b.shenfa
        && a.gengu >= b.gengu
        && a.yuanqi >= b.yuanqi
        && a.gongji >= b.gongji
        && a.qixue >= b.qixue
        && a.qixue_additional >= b.qixue_additional
        && a.huixin >= b.huixin
        && a.huixiao >= b.huixiao
        && a.pofang >= b.pofang
        && a.jiasu >= b.jiasu
        && a.yujin >= b.yujin
        && a.huajin >= b.huajin
        && a.wuqi_shanghai >= b.wuqi_shanghai
}

fn template_total(t: &Template) -> Stats {
    let mut total = t.stats.clone();
    for extra in &t.sockets {
        if total.add(extra).is_err() {
            // 溢出视为不可支配（保守）
            return Stats {
                zhushuxing: u32::MAX,
                ..Stats::default()
            };
        }
    }
    total
}

/// 每个槽位的候选组合；剪掉「被同槽/同来源/同变体的升级档逐项 >= 支配」的基础档。
/// 避免预算截断时 odometer 前缀把基础档当成最优先、结果反而更差。
fn build_candidates(
    catalog: &Catalog,
    req: &OptimizeRequest,
) -> Result<(Vec<Vec<Choice>>, u64), String> {
    let mut droppable: BTreeSet<(Slot, Source, u8)> = BTreeSet::new();
    // 有上限时，属性更高不代表可行性更好，不能删去基础档。
    let has_upper_bound = req
        .constraints
        .stats
        .values()
        .any(|bounds| bounds.max.is_some());
    for up in &catalog.templates {
        if has_upper_bound
            || up.school != req.loadout.school
            || up.choice.tier != Tier::Upgraded
            || !req.constraints.permits(&up.choice)
        {
            continue;
        }
        let base = catalog.templates.iter().find(|b| {
            b.school == up.school
                && b.choice.slot == up.choice.slot
                && b.choice.source == up.choice.source
                && b.choice.variant == up.choice.variant
                && b.choice.tier == Tier::Base
        });
        if let Some(base) = base
            && stats_ge(&template_total(up), &template_total(base))
        {
            droppable.insert((up.choice.slot, up.choice.source, up.choice.variant));
        }
    }
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
            .filter(|t| {
                !(t.choice.tier == Tier::Base
                    && droppable.contains(&(t.choice.slot, t.choice.source, t.choice.variant)))
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
    Ok((candidates, combinations))
}

// 附魔/五彩石与模板共同构成搜索空间，硬约束只检查完整配装。
// None 槽位代表五彩石；选项 None 代表不装备。预算覆盖全部维度。
type DecorationAxis = (Option<Slot>, Vec<Option<String>>);
fn decoration_axes(
    catalog: &Catalog,
    combinations: &mut u64,
) -> Result<Vec<DecorationAxis>, String> {
    let mut axes = Vec::new();
    for slot in Slot::ALL {
        let mut options = vec![None];
        options.extend(
            catalog
                .enchants
                .iter()
                .filter(|e| e.slot == slot)
                .map(|e| Some(e.id.clone())),
        );
        if options.len() > 1 {
            axes.push((Some(slot), options));
        }
    }
    if !catalog.wucai.is_empty() {
        let mut options = vec![None];
        options.extend(catalog.wucai.iter().map(|w| Some(w.id.clone())));
        axes.push((None, options));
    }
    for (_, options) in &axes {
        *combinations = combinations
            .checked_mul(options.len() as u64)
            .ok_or("组合数溢出")?;
    }
    Ok(axes)
}
fn decode_loadout(
    mut flat: u64,
    candidates: &[Vec<Choice>],
    axes: &[DecorationAxis],
    base: &Loadout,
) -> Loadout {
    let mut loadout = base.clone();
    for (slot, options) in axes {
        let choice = options[(flat % options.len() as u64) as usize].clone();
        flat /= options.len() as u64;
        match slot {
            Some(slot) => match choice {
                Some(id) => {
                    loadout.enchants.insert(*slot, id);
                }
                None => {
                    loadout.enchants.remove(slot);
                }
            },
            None => loadout.wucai = choice,
        }
    }
    loadout.choices = decode_flat(flat, candidates);
    loadout
}

/// 组合遍历下标：空间不超预算时全遍历；否则确定性均匀抽样（splitmix64），避免前缀偏差。
fn combination_indices(combinations: u64, budget: u64) -> Vec<u64> {
    if combinations <= budget {
        return (0..combinations).collect();
    }
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let mut seen = std::collections::HashSet::with_capacity(budget as usize);
    let mut out = Vec::with_capacity(budget as usize);
    while (out.len() as u64) < budget {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        let idx = z % combinations;
        if seen.insert(idx) {
            out.push(idx);
        }
    }
    out
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
    // 一次搜索固定同一职业规则，避免中途改文件使候选口径漂移。
    let rules = crate::attributes::load_rules(&req.loadout)?;
    let composed = catalog.compose_validated(&req.loadout, rules.as_ref())?;
    let baseline_score = evaluate(&composed)?;
    if !baseline_score.is_finite() {
        return Err("目标值必须有限".into());
    }
    let baseline = Candidate {
        score: baseline_score,
        composed,
    };
    let (candidates, mut combinations) = build_candidates(catalog, req)?;
    let axes = decoration_axes(catalog, &mut combinations)?;
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
    let visited = combination_indices(combinations, req.max_combinations);
    result.complete = combinations <= req.max_combinations;
    result.stop_reason = if result.complete {
        "exhausted"
    } else {
        "budget_exhausted"
    }
    .into();
    for flat in visited {
        if !on_progress(result.progress) {
            result.stop_reason = "cancelled".into();
            result.complete = false;
            break;
        }
        let loadout = decode_loadout(flat, &candidates, &axes, &req.loadout);
        result.progress.visited += 1;
        // 某些五维附魔不适用于旧属性规则；其余候选仍应搜索。
        let Ok(composed) = catalog.compose_validated(&loadout, rules.as_ref()) else {
            continue;
        };
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

/// 唯一的原始/派生面板到玩家适配器；显式标记心法额外攻击以避免 core 二次换算。
pub fn player(composed: &Composed) -> jpcg_api::PlayerConfigDTO {
    let stats = &composed.stats;
    jpcg_api::PlayerConfigDTO {
        jcsx: main_name(composed.profile.zhushuxing).into(),
        jichu_shuxing: composed.panel.as_ref().map_or(stats.zhushuxing, |p| {
            p.main_value(composed.profile.zhushuxing)
        }),
        jichu_gongji: composed
            .panel
            .as_ref()
            .map_or(stats.gongji, |p| p.gongji_base),
        zhuanhua_gongji: composed.panel.as_ref().map(|p| p.gongji_extra),
        huixin_dengji: composed.panel.as_ref().map_or(stats.huixin, |p| p.huixin),
        huixin_xiaoguo: stats.huixiao,
        pofang_dengji: composed.panel.as_ref().map_or(stats.pofang, |p| p.pofang),
        wuqi_shanghai: stats.wuqi_shanghai,
        jiasu_dengji: stats.jiasu,
    }
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
    let prepared = prepare_request(catalog, req)?;
    let req = &prepared;
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
    let result = optimize_with(
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
    )?;
    Ok(result)
}

fn prepare_request(catalog: &Catalog, req: &OptimizeRequest) -> Result<OptimizeRequest, String> {
    catalog.validate()?;
    req.constraints.validate(catalog, &req.loadout)?;
    if req.max_combinations == 0 || req.max_combinations > 1_000_000 {
        return Err("max_combinations 必须为1..=1000000".into());
    }
    validate_context(catalog, req)?;
    let mut prepared = req.clone();
    if let Objective::Timeline { request, .. } = &mut prepared.objective {
        request.xinfa = req.config.xinfa_config.clone();
        request.hostile = req.config.hostile.clone();
        request.buff = req.config.buff.clone();
        request.coeff = req.config.coefficient.clone();
        request.value_set = req.config.value_set.clone();
        *request = jpcg_combo::timeline_host::prepare_timeline_from_store(request.clone())?;
    }
    Ok(prepared)
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
        Objective::Timeline { request, metric } => {
            if request.actions.is_empty() || request.actions.len() > 256 || request.samples > 50_000
            {
                return Err("时间轴需1..256个动作及不超过50000次采样".into());
            }
            if *metric == crate::TimelineMetric::KillProbability {
                if request.samples == 0 {
                    return Err("击杀率目标需 samples > 0".into());
                }
                if request.hostile.target_hp == 0 && request.hostile.max_hp == 0 {
                    return Err("击杀率目标需目标血量".into());
                }
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
    let config = request(&req.config, composed);
    match &req.objective {
        Objective::Expected { weights } => {
            // 直接复用 core 同一个 q_cal，避免每候选重复加载技能及 host 的 stdout 日志。
            use jpcg_core::type_set::{
                buff::BuffConfig, coefficient::CoefficientConfig, hostilepile::HostilepileConfig,
                player::PlayerConfig,
            };
            let player = PlayerConfig::from(config.player.clone());
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
        Objective::Timeline { request, metric } => {
            let mut r = request.clone();
            r.player = crate::player(composed);
            let result = jpcg_combo::timeline_host::calculate_timeline(r)?;
            Ok(match metric {
                crate::TimelineMetric::TotalDamage => result.total_damage,
                crate::TimelineMetric::WindowDps => result.window_dps,
                crate::TimelineMetric::KillProbability => result.final_kill_prob,
            })
        }
    }
}

// ============ 全核并行调优 ============

/// 并行调优进度：原子字段供 Tauri 主线程轮询并发事件（worker 内不 emit）。
#[derive(Default)]
pub struct OptimizeProgress {
    pub visited: AtomicU64,
    pub evaluated: AtomicU64,
    pub total: AtomicU64,
    pub cancel: AtomicBool,
    pub done: AtomicBool,
}

impl OptimizeProgress {
    pub fn new() -> Self {
        Self::default()
    }

    /// 0..=100 的整数百分比（分母为实际要遍历的组合数，已按预算裁剪）。
    pub fn percent(&self) -> u64 {
        let total = self.total.load(Ordering::Relaxed).max(1);
        (self.visited.load(Ordering::Relaxed).saturating_mul(100) / total).min(100)
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

fn decode_flat(mut flat: u64, candidates: &[Vec<Choice>]) -> Vec<Choice> {
    let mut idx = vec![0usize; candidates.len()];
    for i in (0..candidates.len()).rev() {
        let len = candidates[i].len() as u64;
        idx[i] = (flat % len) as usize;
        flat /= len;
    }
    candidates
        .iter()
        .zip(idx)
        .map(|(c, i)| c[i].clone())
        .collect()
}

fn armor2_count(catalog: &Catalog, composed: &Composed) -> usize {
    composed
        .active_bonuses
        .iter()
        .filter(|b| {
            b.threshold == 2
                && catalog
                    .sets
                    .iter()
                    .any(|s| s.id == b.set && s.kind == SetKind::Armor)
        })
        .count()
}

/// 全核并行有界优化：候选按 flat index 并行评估，线程本地缓存去重；进度/取消经原子共享。
pub fn optimize_parallel(
    catalog: &Catalog,
    req: &OptimizeRequest,
    progress: &OptimizeProgress,
) -> Result<OptimizeResult, String> {
    let prepared = prepare_request(catalog, req)?;
    let req = &prepared;
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
    let rules = crate::attributes::load_rules(&req.loadout)?;
    let baseline_composed = catalog.compose_validated(&req.loadout, rules.as_ref())?;
    let baseline_score = score(
        req,
        &baseline_composed,
        &skill_table.skill,
        &skill_table.xinfa,
    )?;
    let baseline = Candidate {
        score: baseline_score,
        composed: baseline_composed,
    };
    let (candidates, mut combinations) = build_candidates(catalog, req)?;
    let axes = decoration_axes(catalog, &mut combinations)?;
    let indices = combination_indices(combinations, req.max_combinations);
    progress
        .total
        .store(indices.len() as u64, Ordering::Relaxed);
    progress.visited.store(0, Ordering::Relaxed);
    progress.evaluated.store(0, Ordering::Relaxed);
    progress.done.store(false, Ordering::Relaxed);
    progress.cancel.store(false, Ordering::Relaxed);

    let baseline_armor2 = armor2_count(catalog, &baseline.composed);
    let mut result = OptimizeResult {
        best: None,
        baseline: baseline.clone(),
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
    let shared: Mutex<(Option<Candidate>, Vec<Candidate>)> =
        Mutex::new((result.best.clone(), result.pareto.clone()));

    indices
        .into_par_iter()
        .map_init(
            BTreeMap::<String, f64>::new,
            |cache, flat| -> Result<(), String> {
                if progress.cancel.load(Ordering::Relaxed) {
                    return Ok(());
                }
                progress.visited.fetch_add(1, Ordering::Relaxed);
                let loadout = decode_loadout(flat, &candidates, &axes, &req.loadout);
                let Ok(composed) = catalog.compose_validated(&loadout, rules.as_ref()) else {
                    return Ok(());
                };
                let armor2 = armor2_count(catalog, &composed);
                if !(req.constraints.allow_cross_source_2plus2 || armor2 <= 1) {
                    return Ok(());
                }
                if !req.constraints.satisfied(&composed)? {
                    return Ok(());
                }
                let key = serde_json::to_string(&(&composed.stats, &composed.cooldowns))
                    .map_err(|e| e.to_string())?;
                let value = if let Some(v) = cache.get(&key) {
                    *v
                } else {
                    let v = score(req, &composed, &skill_table.skill, &skill_table.xinfa)?;
                    cache.insert(key, v);
                    progress.evaluated.fetch_add(1, Ordering::Relaxed);
                    v
                };
                let candidate = Candidate {
                    score: value,
                    composed,
                };
                let mut guard = shared.lock().map_err(|_| "并行状态锁污染".to_string())?;
                if guard.0.as_ref().is_none_or(|b| value > b.score) {
                    guard.0 = Some(candidate.clone());
                }
                if !guard.1.iter().any(|p| dominates(p, &candidate)) {
                    guard.1.retain(|p| !dominates(&candidate, p));
                    guard.1.push(candidate);
                }
                Ok(())
            },
        )
        .try_for_each(|r| r)?;

    let (best, pareto) = shared
        .into_inner()
        .map_err(|_| "并行状态锁污染".to_string())?;
    result.best = best;
    result.pareto = pareto;
    result.progress = Progress {
        visited: progress.visited.load(Ordering::Relaxed),
        evaluated: progress.evaluated.load(Ordering::Relaxed),
        combinations,
    };
    let cancelled = progress.cancel.load(Ordering::Relaxed);
    result.complete = combinations <= req.max_combinations && !cancelled;
    result.stop_reason = if cancelled {
        "cancelled".into()
    } else if result.complete {
        "exhausted".into()
    } else {
        "budget_exhausted".into()
    };
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
    progress.done.store(true, Ordering::Relaxed);
    Ok(result)
}
