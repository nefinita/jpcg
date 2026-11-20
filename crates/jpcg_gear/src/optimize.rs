use crate::model::*;
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

fn canonical_choice(choice: &Choice, school: &str, own: &str) -> Choice {
    let mut c = choice.clone();
    c.school = (school != own).then(|| school.to_owned());
    c
}

struct FinishProgress<'a>(&'a OptimizeProgress);
impl Drop for FinishProgress<'_> {
    fn drop(&mut self) {
        self.0.done.store(true, Ordering::Relaxed);
    }
}

#[derive(Clone, Copy)]
struct Score {
    value: f64,
    damage: f64,
}
impl Score {
    fn plain(value: f64) -> Self {
        Self { value, damage: 0.0 }
    }
    fn is_finite(self) -> bool {
        self.value.is_finite() && self.damage.is_finite()
    }
}
fn candidate(composed: Composed, score: Score) -> Candidate {
    Candidate {
        composed,
        score: score.value,
        tie_break_damage: score.damage,
    }
}
fn rank(a: &Candidate, b: &Candidate) -> std::cmp::Ordering {
    a.score
        .total_cmp(&b.score)
        .then_with(|| a.tie_break_damage.total_cmp(&b.tie_break_damage))
}
fn better(a: &Candidate, b: &Candidate) -> bool {
    rank(a, b).is_gt()
}
// 伤害专用联合搜索只维护全局最佳，不再构造生存 Pareto。
fn collect_warnings(result: &mut OptimizeResult) {
    result.warnings = result.baseline.composed.warnings.clone();
    if let Some(best) = &result.best {
        for warning in &best.composed.warnings {
            if !result.warnings.contains(warning) {
                result.warnings.push(warning.clone());
            }
        }
    }
}

fn normalize_request_choices(req: &mut OptimizeRequest) {
    let own = &req.loadout.school;
    for choice in req
        .loadout
        .choices
        .iter_mut()
        .chain(&mut req.constraints.locked)
        .chain(&mut req.constraints.banned)
    {
        if choice.school.as_deref() == Some(own) {
            choice.school = None;
        }
    }
}

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
            if !catalog.templates.iter().any(|t| {
                (t.school == loadout.school || t.cross_school)
                    && canonical_choice(&t.choice, &t.school, &loadout.school)
                        == canonical_choice(
                            choice,
                            choice.school.as_deref().unwrap_or(&loadout.school),
                            &loadout.school,
                        )
            }) {
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
    !rank(a, b).is_lt()
        && effective_qixue(&a.composed) >= effective_qixue(&b.composed)
        && x.yujin >= y.yujin
        && x.huajin >= y.huajin
}

/// 每个槽位的候选组合。
/// 只考虑升级装备：`allow_upgraded` 且没有上限约束时，每个槽位只保留升级档（没有升级档则回退全部）。
/// 存在上限约束时保留全部档位——上限可能只有基础档能满足。
fn build_candidates(
    catalog: &Catalog,
    req: &OptimizeRequest,
) -> Result<(Vec<Vec<Choice>>, u64), String> {
    let has_upper_bound = req
        .constraints
        .stats
        .values()
        .any(|bounds| bounds.max.is_some());
    let only_upgraded = req.constraints.allow_upgraded && !has_upper_bound;
    let mut candidates = Vec::new();
    let mut combinations = 1u64;
    for slot in Slot::ALL {
        let all: Vec<Choice> = catalog
            .templates
            .iter()
            .filter(|t| {
                (t.school == req.loadout.school || t.cross_school)
                    && t.choice.slot == slot
                    && req.constraints.permits(&canonical_choice(
                        &t.choice,
                        &t.school,
                        &req.loadout.school,
                    ))
            })
            .map(|t| canonical_choice(&t.choice, &t.school, &req.loadout.school))
            .collect();
        let mut choices = if only_upgraded {
            let upgraded: Vec<Choice> = all
                .iter()
                .filter(|c| c.tier == Tier::Upgraded)
                .cloned()
                .collect();
            if upgraded.is_empty() { all } else { upgraded }
        } else {
            all
        };
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

pub(super) type DecorationAxis = (Option<Slot>, Vec<Option<String>>);

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
    on_progress: impl FnMut(Progress) -> bool,
) -> Result<OptimizeResult, String> {
    optimize_ranked_with(catalog, req, |c| evaluate(c).map(Score::plain), on_progress)
}

fn optimize_ranked_with(
    catalog: &Catalog,
    req: &OptimizeRequest,
    mut evaluate: impl FnMut(&Composed) -> Result<Score, String>,
    mut on_progress: impl FnMut(Progress) -> bool,
) -> Result<OptimizeResult, String> {
    let mut normalized = req.clone();
    normalize_request_choices(&mut normalized);
    let req = &normalized;
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
    let baseline = candidate(composed, baseline_score);
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
        mc_evaluations: 0,
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
        && req.loadout.choices.iter().all(|c| {
            candidates.iter().any(|axis| {
                axis.contains(&canonical_choice(
                    c,
                    c.school.as_deref().unwrap_or(&req.loadout.school),
                    &req.loadout.school,
                ))
            })
        })
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
            let candidate = candidate(composed, score);
            if result.best.as_ref().is_none_or(|b| better(&candidate, b)) {
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
    if !matches!(&req.objective, Objective::Timeline { .. }) {
        result
            .warnings
            .push("加速等级仅合成/约束/导出；当前目标不重排技能时序".into());
    }
    result
        .warnings
        .push("生存Pareto使用气血/御劲/化劲点数，不代表实战承伤".into());
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
/// 正态近似击杀率：总伤害 ≈ N(mean, variance)，P(D ≥ hp)。
fn normal_kill_prob(mean: f64, variance: f64, hp: f64) -> f64 {
    if hp <= 0.0 {
        return 0.0;
    }
    if variance <= 1e-9 {
        return if mean >= hp { 1.0 } else { 0.0 };
    }
    let z = (hp - mean) / variance.sqrt();
    0.5 * erfc(z / std::f64::consts::SQRT_2)
}

fn erfc(x: f64) -> f64 {
    1.0 - erf(x)
}

/// Abramowitz-Stegun 7.1.26；误差 < 1.5e-7，足够做搜索排序。
fn erf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.3275911 * x);
    let y = 1.0
        - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t
            + 0.254829592)
            * t
            * (-x * x).exp();
    sign * y
}

/// 时间轴目标的确定性击杀率近似（总伤 + 方差的正态近似，不跑 MC）。
/// 全量穷举用它做搜索指标，最后 Pareto 候选再用 50k MC 复核。
fn approx_kill_prob(req: &OptimizeRequest, composed: &Composed) -> Result<f64, String> {
    let Objective::Timeline { request, .. } = &req.objective else {
        return Err("近似击杀率只用于时间轴目标".into());
    };
    let hp = if request.hostile.max_hp > 0 {
        request.hostile.max_hp as f64
    } else {
        request.hostile.target_hp as f64
    };
    let mut r = request.clone();
    r.player = player(composed);
    r.samples = 0;
    r.seed = None;
    let result = jpcg_combo::timeline_host::calculate_timeline(r)?;
    Ok(normal_kill_prob(
        result.total_damage,
        result.damage_variance,
        hp,
    ))
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
    let mut prepared = prepare_request(catalog, req)?;
    // 击杀率目标：CPU/GPU 批量 MC 与基准评分必须共用同一组样本（固定 seed）。
    if let Objective::Timeline {
        request, metric, ..
    } = &mut prepared.objective
        && *metric == crate::TimelineMetric::KillProbability
        && request.seed.is_none()
    {
        request.seed = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(42),
        );
    }
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
    let result = optimize_ranked_with(
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
    let mut prepared = req.clone();
    normalize_request_choices(&mut prepared);
    if let Objective::Timeline { request, .. } = &mut prepared.objective {
        request.xinfa = req.config.xinfa_config.clone();
        request.hostile = req.config.hostile.clone();
        request.buff = req.config.buff.clone();
        request.coeff = req.config.coefficient.clone();
        request.value_set = req.config.value_set.clone();
        if matches!(
            &req.objective,
            Objective::Timeline {
                metric: crate::TimelineMetric::KillProbability,
                ..
            }
        ) && request.seed.is_none()
        {
            request.seed = Some(0);
        }
        *request = jpcg_combo::timeline_host::prepare_timeline_from_store(request.clone())?;
    }
    validate_context(catalog, &prepared)?;
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
) -> Result<Score, String> {
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
            Ok(Score::plain(score))
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
            Ok(Score::plain(result.final_kill_prob))
        }
        Objective::Timeline { request, metric } => {
            let mut r = request.clone();
            r.player = crate::player(composed);
            let adjustments = cooldown_adjustments(composed);
            let result =
                jpcg_combo::timeline_host::calculate_timeline_with_cooldowns(r, &adjustments)?;
            Ok(match metric {
                crate::TimelineMetric::TotalDamage => Score::plain(result.total_damage),
                crate::TimelineMetric::WindowDps => Score::plain(result.window_dps),
                crate::TimelineMetric::KillProbability => Score {
                    value: result.final_kill_prob,
                    damage: result.total_damage,
                },
            })
        }
    }
}

fn cooldown_adjustments(composed: &Composed) -> Vec<(u32, f32)> {
    composed
        .cooldowns
        .iter()
        .map(|c| (c.skill_id, c.seconds as f32))
        .collect()
}

// ============ 全核并行调优 ============

pub static PROF_COMPOSE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static PROF_MC: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

struct JointSpace {
    candidates: Vec<Vec<Choice>>,
    axes: Vec<DecorationAxis>,
    base: Loadout,
    combinations: u64,
}

fn affects_damage(stats: &Stats) -> bool {
    [
        stats.zhushuxing,
        stats.liliang,
        stats.shenfa,
        stats.gengu,
        stats.yuanqi,
        stats.gongji,
        stats.huixin,
        stats.huixiao,
        stats.pofang,
        stats.jiasu,
        stats.wuqi_shanghai,
    ]
    .into_iter()
    .any(|value| value != 0)
}

fn has_survival_constraint(req: &OptimizeRequest) -> bool {
    ["tizhi", "qixue", "qixue_additional", "yujin", "huajin"]
        .into_iter()
        .any(|name| req.constraints.stats.contains_key(name))
}

/// 生产搜索与完整筛选基准共用同一份候选空间，避免装饰/档位口径漂移。
/// 伤害专用搜索把纯生存装饰固定到用户当前选项，不再枚举等价进攻输入。
fn joint_space(
    catalog: &Catalog,
    req: &OptimizeRequest,
    damage_only: bool,
) -> Result<JointSpace, String> {
    let (candidates, mut combinations) = build_candidates(catalog, req)?;
    let mut search_base = req.loadout.clone();
    let fold_survival = damage_only && !has_survival_constraint(req);
    // 每个槽位必须选一个附魔，五彩石只要目录里有就必装。
    let mut axes: Vec<DecorationAxis> = Vec::new();
    for slot in Slot::ALL {
        let mut options: Vec<Option<String>> = catalog
            .enchants
            .iter()
            .filter(|e| e.slot == slot)
            .map(|e| Some(e.id.clone()))
            .collect();
        options.sort();
        let defensive = fold_survival
            && options.iter().all(|id| {
                id.as_ref().is_none_or(|id| {
                    catalog
                        .enchants
                        .iter()
                        .find(|e| e.slot == slot && e.id == *id)
                        .is_none_or(|e| !affects_damage(&e.stats))
                })
            });
        if defensive || options.len() == 1 {
            let selected = req
                .loadout
                .enchants
                .get(&slot)
                .filter(|current| options.contains(&Some((*current).clone())))
                .cloned()
                .or_else(|| options.first().cloned().flatten());
            if let Some(id) = selected {
                search_base.enchants.insert(slot, id);
            }
        } else if options.len() > 1 {
            axes.push((Some(slot), options));
        }
    }
    if !catalog.wucai.is_empty() {
        let mut options: Vec<Option<String>> =
            catalog.wucai.iter().map(|w| Some(w.id.clone())).collect();
        options.sort();
        let defensive = fold_survival
            && options.iter().all(|id| {
                id.as_ref().is_none_or(|id| {
                    catalog
                        .wucai
                        .iter()
                        .find(|w| w.id == *id)
                        .is_none_or(|w| !affects_damage(&w.stats))
                })
            });
        if defensive {
            let selected = req
                .loadout
                .wucai
                .as_ref()
                .filter(|current| options.contains(&Some((*current).clone())))
                .cloned()
                .or_else(|| options.first().cloned().flatten());
            search_base.wucai = selected;
        } else {
            axes.push((None, options));
        }
    }
    for (_, options) in &axes {
        combinations = combinations
            .checked_mul(options.len() as u64)
            .ok_or("组合数溢出")?;
    }
    Ok(JointSpace {
        candidates,
        axes,
        base: search_base,
        combinations,
    })
}

struct ScoringBatch {
    metas: Vec<Composed>,
    pending: Vec<(jpcg_compute::CandidateMc, f64)>,
    leaves: u64,
}
struct StopWorker(Arc<std::sync::atomic::AtomicBool>);
impl Drop for StopWorker {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

fn prepare_scoring_batch(
    catalog: &Catalog,
    req: &OptimizeRequest,
    rules: Option<&jpcg_api::AttributeRules>,
    candidates: &[Vec<Choice>],
    axes: &[DecorationAxis],
    base: &Loadout,
    indices: Vec<u64>,
    template: &jpcg_combo::timeline_host::TimelineTemplate,
    progress: &OptimizeProgress,
) -> Result<ScoringBatch, String> {
    let leaves = indices.len() as u64;
    let metas = indices
        .into_par_iter()
        .map(|flat| -> Result<Option<Composed>, String> {
            if progress.cancel.load(Ordering::Relaxed) {
                return Ok(None);
            }
            let loadout = decode_loadout(flat, candidates, axes, base);
            let Ok(c) = catalog.compose_validated(&loadout, rules) else {
                return Ok(None);
            };
            if (!req.constraints.allow_cross_source_2plus2 && armor2_count(catalog, &c) > 1)
                || !req.constraints.satisfied(&c)?
            {
                return Ok(None);
            }
            Ok(Some(c))
        })
        .collect::<Result<Vec<_>, String>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    // 防御装饰已固定在用户配置，保留的每个候选都独立评分，不做进攻缓存去重。
    let pending = metas
        .par_iter()
        .map_init(
            || template.clone(),
            |tpl, c| {
                tpl.set_loadout(player(c), &cooldown_adjustments(c))?;
                let mc = tpl.mc_input()?;
                Ok((
                    jpcg_compute::CandidateMc {
                        events: mc.events,
                        hp0: mc.hp0,
                    },
                    mc.total_damage,
                ))
            },
        )
        .collect::<Result<Vec<_>, String>>()?;
    Ok(ScoringBatch {
        metas,
        pending,
        leaves,
    })
}

/// 全量联合穷举：装备候选 × 必选附魔/五彩石，多核并行。
/// 不抽样、不使用预算；`complete = true` 表示整个空间已经遍历或证明无解。
pub fn optimize_joint(
    catalog: &Catalog,
    req: &OptimizeRequest,
    progress: &OptimizeProgress,
) -> Result<OptimizeResult, String> {
    progress.done.store(false, Ordering::Relaxed);
    let _finished = FinishProgress(progress);
    let mut prepared = prepare_request(catalog, req)?;
    // 批量 MC 与基线必须使用同一组样本；未指定种子时也固定本次搜索的口径。
    if let Objective::Timeline { request, metric } = &mut prepared.objective
        && *metric == crate::TimelineMetric::KillProbability
        && request.seed.is_none()
    {
        request.seed = Some(0);
    }
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
    // 击杀率目标走「按后端的批量精确 MC」（CPU AVX-512 / wgpu），其余指标走原有逐候选评分。
    let kill_mode = matches!(
        &req.objective,
        Objective::Timeline {
            metric: crate::TimelineMetric::KillProbability,
            ..
        }
    );
    let baseline_composed = catalog.compose_validated(&req.loadout, rules.as_ref())?;
    let baseline_score = score(
        req,
        &baseline_composed,
        &skill_table.skill,
        &skill_table.xinfa,
    )?;
    let baseline = candidate(baseline_composed, baseline_score);
    let JointSpace {
        candidates,
        axes,
        base: search_base,
        combinations,
    } = joint_space(catalog, req, kill_mode)?;
    let total = combinations;
    progress.total.store(total, Ordering::Relaxed);
    progress.visited.store(0, Ordering::Relaxed);
    progress.evaluated.store(0, Ordering::Relaxed);
    progress.mc_evaluations.store(0, Ordering::Relaxed);
    progress.done.store(false, Ordering::Relaxed);

    let mut search =
        crate::joint_search::JointSearch::new(catalog, req, &candidates, &axes, &search_base)?;

    if !baseline.score.is_finite() || !baseline.tie_break_damage.is_finite() {
        return Err("目标值必须有限".into());
    }
    let baseline_armor2 = armor2_count(catalog, &baseline.composed);
    let mut result = OptimizeResult {
        best: None,
        baseline: baseline.clone(),
        delta: None,
        pareto: vec![],
        progress: Progress {
            visited: 0,
            evaluated: 0,
            combinations: total,
        },
        complete: false,
        stop_reason: String::new(),
        warnings: Vec::new(),
        mc_evaluations: 0,
    };
    // 基线始终用于比较；仅完整佩戴必选装饰的基线可作为联合搜索结果。
    let baseline_decorated = catalog.enchants.iter().all(|e| {
        req.loadout.enchants.get(&e.slot).is_some_and(|id| {
            catalog
                .enchants
                .iter()
                .any(|option| option.slot == e.slot && option.id == *id)
        })
    }) && (catalog.wucai.is_empty() || req.loadout.wucai.is_some());
    if baseline_decorated
        && req
            .loadout
            .choices
            .iter()
            .all(|c| req.constraints.permits(c))
        && (req.constraints.allow_cross_source_2plus2 || baseline_armor2 <= 1)
        && req.constraints.satisfied(&result.baseline.composed)?
        && req.loadout.choices.iter().all(|c| {
            candidates.iter().any(|axis| {
                axis.contains(&canonical_choice(
                    c,
                    c.school.as_deref().unwrap_or(&req.loadout.school),
                    &req.loadout.school,
                ))
            })
        })
    {
        result.best = Some(result.baseline.clone());
        result.pareto.push(result.baseline.clone());
    }
    if kill_mode {
        // 击杀率目标：按选中后端的批量精确 MC（CPU AVX-512 / wgpu）逐批评分。
        let backend = req.compute.clone().unwrap_or_else(|| "cpu".to_string());
        if backend != "cpu" && !backend.starts_with("wgpu:") {
            return Err(format!("非法计算后端: {backend}"));
        }
        let gpu = if let Some(idx) = backend.strip_prefix("wgpu:") {
            let index: usize = idx
                .parse()
                .map_err(|_| format!("非法计算后端: {backend}"))?;
            Some(jpcg_compute::WgpuCompute::new(index)?)
        } else {
            None
        };
        let multi = match &req.objective {
            Objective::Timeline { request, .. } => request,
            _ => unreachable!(),
        };
        let samples = multi.samples.max(1);
        let seed = multi.seed.unwrap_or(0);
        // 与玩家无关的时间轴部分只构建一次；每个线程克隆一份模板，逐候选只替换 player。
        let template = jpcg_combo::timeline_host::TimelineTemplate::new(multi.clone())?;
        let initial_best = result.best.clone();
        // 搜索游标轻量串行分发；两个无缓存准备 worker 与 CPU/GPU 评分并行。
        // 消费端按序号归并，worker 完成先后不影响最佳方案的确定性。
        let best = std::thread::scope(|scope| -> Result<_, String> {
            const PREPARE_WORKERS: usize = 2;
            let (raw_tx, raw_rx) = std::sync::mpsc::sync_channel(PREPARE_WORKERS * 2);
            let raw_rx = Arc::new(Mutex::new(raw_rx));
            let (tx, rx) = std::sync::mpsc::sync_channel(PREPARE_WORKERS * 2);
            let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let worker_stop = Arc::clone(&stop);
            let _stop_worker = StopWorker(stop);
            scope.spawn(move || {
                let mut sequence = 0usize;
                loop {
                    if progress.cancel.load(Ordering::Relaxed)
                        || worker_stop.load(Ordering::Relaxed)
                    {
                        break;
                    }
                    let batch = search.next_batch(32768, progress);
                    if batch.is_empty() || raw_tx.send((sequence, batch)).is_err() {
                        break;
                    }
                    sequence += 1;
                }
            });
            for _ in 0..PREPARE_WORKERS {
                let raw_rx = Arc::clone(&raw_rx);
                let tx = tx.clone();
                let template = &template;
                let candidates = &candidates;
                let axes = &axes;
                let base = &search_base;
                let rules = rules.as_ref();
                scope.spawn(move || {
                    loop {
                        let received = raw_rx
                            .lock()
                            .map_err(|_| "候选队列锁污染".to_string())
                            .and_then(|rx| rx.recv().map_err(|_| "候选队列已关闭".into()));
                        let Ok((sequence, batch)) = received else {
                            break;
                        };
                        let prepared = prepare_scoring_batch(
                            catalog, req, rules, candidates, axes, base, batch, template, progress,
                        );
                        let failed = prepared.is_err();
                        if tx.send((sequence, prepared)).is_err() || failed {
                            break;
                        }
                    }
                });
            }
            drop(tx);
            let mut best = initial_best;
            let mut next_sequence = 0usize;
            let mut queued = BTreeMap::new();
            for (sequence, batch) in &rx {
                queued.insert(sequence, batch);
                while let Some(batch) = queued.remove(&next_sequence) {
                    next_sequence += 1;
                    let batch = batch?;
                    if progress.cancel.load(Ordering::Relaxed) {
                        break;
                    }
                    let count = batch.metas.len() as u64;
                    let (inputs, damages): (Vec<_>, Vec<_>) = batch.pending.into_iter().unzip();
                    let kills = if inputs.is_empty() {
                        Vec::new()
                    } else if let Some(gpu) = &gpu {
                        gpu.run(&inputs, samples, seed)?
                    } else {
                        jpcg_compute::run_cpu(&inputs, samples, seed)
                    };
                    for (composed, (damage, kill)) in
                        batch.metas.into_iter().zip(damages.into_iter().zip(kills))
                    {
                        let value = Score {
                            value: kill as f64 / samples as f64,
                            damage,
                        };
                        if !value.is_finite() {
                            return Err("目标值必须有限".into());
                        }
                        let candidate = candidate(composed, value);
                        if best.as_ref().is_none_or(|b| better(&candidate, b)) {
                            best = Some(candidate);
                        }
                    }
                    progress
                        .mc_evaluations
                        .fetch_add(inputs.len() as u64, Ordering::Relaxed);
                    progress.evaluated.fetch_add(count, Ordering::Relaxed);
                    progress.visited.fetch_add(batch.leaves, Ordering::Relaxed);
                }
            }
            Ok(best)
        })?;
        result.best = best;
        result.pareto = result.best.iter().cloned().collect();
        result.delta = result
            .best
            .as_ref()
            .map(|b| b.score - result.baseline.score);
        result.progress = Progress {
            visited: progress.visited.load(Ordering::Relaxed),
            evaluated: progress.evaluated.load(Ordering::Relaxed),
            combinations: total,
        };
        result.complete =
            result.progress.visited >= total && !progress.cancel.load(Ordering::Relaxed);
        result.stop_reason = if progress.cancel.load(Ordering::Relaxed) {
            "cancelled".into()
        } else {
            "exhausted".into()
        };
        result.mc_evaluations = progress.mc_evaluations.load(Ordering::Relaxed);
        collect_warnings(&mut result);
        if !has_survival_constraint(req) {
            result
                .warnings
                .push("伤害专用搜索：纯生存附魔固定为当前配置，未生成生存 Pareto".into());
        }
        return Ok(result);
    }

    let shared: Mutex<(Option<Candidate>, Vec<Candidate>)> =
        Mutex::new((result.best.clone(), result.pareto.clone()));
    loop {
        let batch = search.next_batch(32768, progress);
        if batch.is_empty() {
            break;
        }
        batch
            .into_par_iter()
            .map_init(
                BTreeMap::<String, Score>::new,
                |cache, flat| -> Result<(), String> {
                    if progress.cancel.load(Ordering::Relaxed) {
                        return Ok(());
                    }
                    progress.visited.fetch_add(1, Ordering::Relaxed);
                    let loadout = decode_loadout(flat, &candidates, &axes, &search_base);
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
                        let v = if kill_mode {
                            Score::plain(approx_kill_prob(req, &composed)?)
                        } else {
                            score(req, &composed, &skill_table.skill, &skill_table.xinfa)?
                        };
                        cache.insert(key, v);
                        progress.evaluated.fetch_add(1, Ordering::Relaxed);
                        v
                    };
                    if !value.is_finite() {
                        return Err("目标值必须有限".into());
                    }
                    let candidate = candidate(composed, value);
                    let mut guard = shared.lock().map_err(|_| "并行状态锁污染".to_string())?;
                    if guard.0.as_ref().is_none_or(|b| better(&candidate, b)) {
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
    }

    let (best, pareto) = shared
        .into_inner()
        .map_err(|_| "并行状态锁污染".to_string())?;
    let visited = progress.visited.load(Ordering::Relaxed);
    result.best = best;
    result.delta = result
        .best
        .as_ref()
        .map(|b| b.score - result.baseline.score);
    result.pareto = pareto;
    result.progress = Progress {
        visited,
        evaluated: progress.evaluated.load(Ordering::Relaxed),
        combinations: total,
    };
    result.complete = visited >= total && !progress.cancel.load(Ordering::Relaxed);
    result.stop_reason = if progress.cancel.load(Ordering::Relaxed) {
        "cancelled".into()
    } else {
        "exhausted".into()
    };
    collect_warnings(&mut result);
    progress.done.store(true, Ordering::Relaxed);
    Ok(result)
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
    progress.done.store(false, Ordering::Relaxed);
    let _finished = FinishProgress(progress);
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
    let baseline = candidate(baseline_composed, baseline_score);
    let (candidates, mut combinations) = build_candidates(catalog, req)?;
    let axes = decoration_axes(catalog, &mut combinations)?;
    let indices = combination_indices(combinations, req.max_combinations);
    progress
        .total
        .store(indices.len() as u64, Ordering::Relaxed);
    progress.visited.store(0, Ordering::Relaxed);
    progress.evaluated.store(0, Ordering::Relaxed);
    progress.mc_evaluations.store(0, Ordering::Relaxed);
    progress.done.store(false, Ordering::Relaxed);

    if !baseline.score.is_finite() || !baseline.tie_break_damage.is_finite() {
        return Err("目标值必须有限".into());
    }
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
        mc_evaluations: 0,
    };
    if req
        .loadout
        .choices
        .iter()
        .all(|c| req.constraints.permits(c))
        && (req.constraints.allow_cross_source_2plus2 || baseline_armor2 <= 1)
        && req.constraints.satisfied(&result.baseline.composed)?
        && req.loadout.choices.iter().all(|c| {
            candidates.iter().any(|axis| {
                axis.contains(&canonical_choice(
                    c,
                    c.school.as_deref().unwrap_or(&req.loadout.school),
                    &req.loadout.school,
                ))
            })
        })
    {
        result.best = Some(result.baseline.clone());
        result.pareto.push(result.baseline.clone());
    }
    let shared: Mutex<(Option<Candidate>, Vec<Candidate>)> =
        Mutex::new((result.best.clone(), result.pareto.clone()));

    indices
        .into_par_iter()
        .map_init(
            BTreeMap::<String, Score>::new,
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
                if !value.is_finite() {
                    return Err("目标值必须有限".into());
                }
                let candidate = candidate(composed, value);
                let mut guard = shared.lock().map_err(|_| "并行状态锁污染".to_string())?;
                if guard.0.as_ref().is_none_or(|b| better(&candidate, b)) {
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
    collect_warnings(&mut result);
    progress.done.store(true, Ordering::Relaxed);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erf_matches_known_values() {
        assert!(erf(0.0).abs() < 1e-7);
        assert!((erf(1.0) - 0.842_700_8).abs() < 1e-5);
        assert!((erf(-1.0) + 0.842_700_8).abs() < 1e-5);
    }

    #[test]
    fn normal_kill_prob_matches_known_values() {
        // 均值 100、σ=10、目标 100 → 50%
        assert!((normal_kill_prob(100.0, 100.0, 100.0) - 0.5).abs() < 1e-6);
        // 目标远低于均值 → 接近 1；远高于 → 接近 0
        assert!(normal_kill_prob(100.0, 100.0, 50.0) > 0.999);
        assert!(normal_kill_prob(100.0, 100.0, 150.0) < 0.001);
        // 方差 0 → 阈值判定
        assert_eq!(normal_kill_prob(100.0, 0.0, 99.0), 1.0);
        assert_eq!(normal_kill_prob(100.0, 0.0, 101.0), 0.0);
    }
}

#[cfg(test)]
mod joint_search_tests {
    use super::*;
    use crate::joint_search::JointSearch;

    fn fixture() -> (Catalog, OptimizeRequest) {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/synthetic/gear");
        let catalog = Catalog::load(&dir).unwrap();
        let req =
            toml::from_str(&std::fs::read_to_string(dir.join("sample-request.toml")).unwrap())
                .unwrap();
        (catalog, req)
    }

    #[test]
    fn joint_pruning_matches_brute_force_with_mixed_tiers_and_haste_bonuses() {
        let (mut catalog, mut req) = fixture();
        req.constraints.sources = vec![Source::Mingjian, Source::Weiwang];
        req.constraints.allow_upgraded = true;
        req.constraints
            .required_sets
            .insert("sample-mingjian".into(), 4);
        // 套装、镶嵌、固定附魔、可选附魔和五彩石都参与加速上下界。
        for template in &mut catalog.templates {
            let slot = Slot::ALL
                .iter()
                .position(|s| *s == template.choice.slot)
                .unwrap() as u32;
            template.stats.jiasu = if template.choice.source == Source::Mingjian {
                3 + slot * 3
            } else {
                9 + slot
            };
            template.sockets.push(Stats {
                jiasu: 2,
                ..Default::default()
            });
        }
        for set in &mut catalog.sets {
            for tier in &mut set.tiers {
                tier.bonus2.jiasu = if tier.tier == Tier::Base { 11 } else { 23 };
                if let Some(bonus) = &mut tier.bonus3 {
                    bonus.jiasu = 17;
                }
            }
        }
        catalog.wucai.clear();
        for (id, haste) in [("slow", 0), ("fast", 31)] {
            catalog.wucai.push(WucaiDef {
                id: id.into(),
                name: id.into(),
                stats: Stats {
                    jiasu: haste,
                    ..Default::default()
                },
            });
            catalog.enchants.push(EnchantDef {
                slot: Slot::Tou,
                id: id.into(),
                name: id.into(),
                stats: Stats {
                    jiasu: haste,
                    ..Default::default()
                },
            });
        }
        catalog.enchants.push(EnchantDef {
            slot: Slot::Yi,
            id: "fixed".into(),
            name: "fixed".into(),
            stats: Stats {
                jiasu: 7,
                ..Default::default()
            },
        });
        req.loadout.base.jiasu = 5;
        req.loadout.enchants.insert(Slot::Yi, "fixed".into());
        // 限制其余部位，保留五个防具的来源与混档分支。
        req.constraints.locked = req
            .loadout
            .choices
            .iter()
            .filter(|c| !c.slot.armor())
            .cloned()
            .collect();
        let axes = vec![
            (
                Some(Slot::Tou),
                vec![Some("slow".into()), Some("fast".into())],
            ),
            (None, vec![Some("slow".into()), Some("fast".into())]),
        ];
        for allow_cross in [false, true] {
            req.constraints.allow_cross_source_2plus2 = allow_cross;
            for (min, max) in [
                (Some(0), Some(500)),
                (Some(130), Some(160)),
                (Some(200), Some(200)),
                (Some(200), Some(230)),
                (Some(200), None),
                (None, Some(180)),
            ] {
                req.constraints
                    .stats
                    .insert("jiasu".into(), Bounds { min, max });
                let (candidates, combinations) = build_candidates(&catalog, &req).unwrap();
                let total = combinations * 4;
                let expected: BTreeSet<_> = (0..total)
                    .filter(|&flat| {
                        let l = decode_loadout(flat, &candidates, &axes, &req.loadout);
                        let c = catalog.compose_validated(&l, None).unwrap();
                        req.constraints.satisfied(&c).unwrap()
                            && (allow_cross || armor2_count(&catalog, &c) <= 1)
                    })
                    .collect();
                let progress = OptimizeProgress::new();
                let mut search =
                    JointSearch::new(&catalog, &req, &candidates, &axes, &req.loadout).unwrap();
                let mut actual = BTreeSet::new();
                loop {
                    let batch = search.next_batch(7, &progress);
                    if batch.is_empty() {
                        break;
                    }
                    progress
                        .visited
                        .fetch_add(batch.len() as u64, Ordering::Relaxed);
                    for flat in batch {
                        assert!(actual.insert(flat), "重复候选");
                    }
                }
                assert_eq!(expected, actual, "加速范围 {min:?}..{max:?}");
                assert_eq!(progress.visited.load(Ordering::Relaxed), total);
            }
        }
    }

    #[test]
    fn impossible_constraints_skip_billions_of_decoration_leaves() {
        let (catalog, mut req) = fixture();
        req.constraints.sources = vec![Source::Mingjian, Source::Weiwang];
        req.constraints
            .required_sets
            .insert("sample-mingjian".into(), 4);
        // 锁两个防具为威望：剩余三件再怎么选也凑不齐竞技场四件套。
        req.constraints.locked = req
            .loadout
            .choices
            .iter()
            .filter(|c| [Slot::Tou, Slot::Yi].contains(&c.slot))
            .cloned()
            .collect();
        let (candidates, combinations) = build_candidates(&catalog, &req).unwrap();
        let axes = vec![(None, vec![None; 1_000]); 3];
        let progress = OptimizeProgress::new();
        let mut search =
            JointSearch::new(&catalog, &req, &candidates, &axes, &req.loadout).unwrap();
        assert!(search.next_batch(1, &progress).is_empty());
        assert_eq!(
            progress.visited.load(Ordering::Relaxed),
            combinations * 1_000_000_000
        );
        assert!(progress.visited.load(Ordering::Relaxed) > 1_000_000_000);

        req.constraints.required_sets.clear();
        req.constraints.stats.insert(
            "jiasu".into(),
            Bounds {
                min: Some(u32::MAX),
                max: None,
            },
        );
        let progress = OptimizeProgress::new();
        let mut search =
            JointSearch::new(&catalog, &req, &candidates, &axes, &req.loadout).unwrap();
        assert!(search.next_batch(1, &progress).is_empty());
        assert_eq!(
            progress.visited.load(Ordering::Relaxed),
            combinations * 1_000_000_000
        );
    }

    #[test]
    fn joint_enumeration_can_cancel_and_resume_between_batches() {
        let (catalog, req) = fixture();
        let (candidates, total) = build_candidates(&catalog, &req).unwrap();
        let progress = OptimizeProgress::new();
        let mut search = JointSearch::new(&catalog, &req, &candidates, &[], &req.loadout).unwrap();
        let first = search.next_batch(1, &progress);
        assert_eq!(first.len(), 1);
        progress.cancel();
        assert!(search.next_batch(10, &progress).is_empty());
        // 只有显式恢复此测试游标时清除取消；优化入口不自动清除用户取消。
        progress.cancel.store(false, Ordering::Relaxed);
        let rest = search.next_batch(total as usize, &progress);
        assert_eq!(rest.len() as u64 + 1, total);
        assert!(!rest.contains(&first[0]));
    }
}

#[cfg(test)]
mod joint_bench {
    use super::*;

    /// 仅测候选准备/硬约束筛选，不含时间轴与 MC；显式运行以免拖慢日常回归。
    #[test]
    #[ignore = "cargo test -p jpcg_gear --release joint_prefilter_benchmark -- --ignored --nocapture"]
    fn joint_prefilter_benchmark() {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/synthetic/gear");
        let mut catalog = Catalog::load(&dir).unwrap();
        let mut req: OptimizeRequest =
            toml::from_str(&std::fs::read_to_string(dir.join("sample-request.toml")).unwrap())
                .unwrap();
        req.constraints.sources = vec![Source::Mingjian, Source::Weiwang];
        req.constraints
            .required_sets
            .insert("sample-mingjian".into(), 4);
        req.constraints.stats.clear();
        req.constraints.stats.insert(
            "jiasu".into(),
            Bounds {
                min: Some(80),
                max: Some(85),
            },
        );
        catalog.wucai.clear();
        for t in &mut catalog.templates {
            t.stats.jiasu = 0;
        }
        let mut axes = Vec::new();
        for slot in [Slot::Tou, Slot::Yi, Slot::Shou, Slot::Wuqi] {
            let mut options = Vec::new();
            for haste in [0, 10, 20, 30] {
                let id = format!("{slot:?}-{haste}");
                catalog.enchants.push(EnchantDef {
                    slot,
                    id: id.clone(),
                    name: id.clone(),
                    stats: Stats {
                        jiasu: haste,
                        ..Default::default()
                    },
                });
                options.push(Some(id));
            }
            axes.push((Some(slot), options));
        }
        let (candidates, gear_total) = build_candidates(&catalog, &req).unwrap();
        let total = gear_total * 256;
        let start = std::time::Instant::now();
        let mut reference = BTreeSet::new();
        for flat in 0..total {
            let loadout = decode_loadout(flat, &candidates, &axes, &req.loadout);
            let composed = catalog.compose_validated(&loadout, None).unwrap();
            if req.constraints.satisfied(&composed).unwrap() {
                reference.insert(flat);
            }
        }
        let old_elapsed = start.elapsed();
        let start = std::time::Instant::now();
        let progress = OptimizeProgress::new();
        let mut search =
            crate::joint_search::JointSearch::new(&catalog, &req, &candidates, &axes, &req.loadout)
                .unwrap();
        let mut actual = BTreeSet::new();
        loop {
            let batch = search.next_batch(1024, &progress);
            if batch.is_empty() {
                break;
            }
            progress
                .visited
                .fetch_add(batch.len() as u64, Ordering::Relaxed);
            for flat in batch {
                let loadout = decode_loadout(flat, &candidates, &axes, &req.loadout);
                let composed = catalog.compose_validated(&loadout, None).unwrap();
                assert!(
                    req.constraints.satisfied(&composed).unwrap(),
                    "flat={flat} haste={} sets={:?}",
                    composed.stats.jiasu,
                    composed.set_counts
                );
                actual.insert(flat);
            }
        }
        let new_elapsed = start.elapsed();
        assert_eq!(reference, actual);
        assert_eq!(progress.visited.load(Ordering::Relaxed), total);
        eprintln!(
            "prefilter: raw={total} composed={} old_ms={:.3} new_ms={:.3} speedup={:.2}x",
            actual.len(),
            old_elapsed.as_secs_f64() * 1000.0,
            new_elapsed.as_secs_f64() * 1000.0,
            old_elapsed.as_secs_f64() / new_elapsed.as_secs_f64()
        );
    }
}

#[cfg(test)]
mod current_config_filter {
    use super::*;

    /// 只遍历约束筛选器，不运行时间轴或 MC。请求路径由测试进程环境显式指定。
    #[test]
    #[ignore = "set JPCG_FILTER_REQUEST and JPCG_DATA_DIR; run with --release --ignored --nocapture"]
    fn full_current_config_filter() {
        let path = std::env::var("JPCG_FILTER_REQUEST").unwrap();
        let req: OptimizeRequest =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let catalog = crate::host::catalog(&req.loadout.school).unwrap();
        let req = prepare_request(&catalog, &req).unwrap();
        assert!(
            req.constraints.stats.keys().all(|k| k == "jiasu"),
            "此基准仅覆盖提前筛选的约束"
        );
        let rules = crate::attributes::load_rules(&req.loadout).unwrap();
        let baseline = catalog
            .compose_validated(&req.loadout, rules.as_ref())
            .unwrap();
        let JointSpace {
            candidates,
            axes,
            base,
            combinations: total,
        } = joint_space(&catalog, &req, false).unwrap();
        let decoration_count: u64 = axes
            .iter()
            .map(|(_, options)| options.len() as u64)
            .product();
        let gear_count = total / decoration_count;
        eprintln!(
            "raw={total} gear={gear_count} decorations={decoration_count} baseline_haste={}",
            baseline.stats.jiasu
        );

        // 独立核对：装饰加速分布用整数 DP；每种装备单独调用正式合成器判定套装与加速。
        // 不依赖 JointSearch 的分支可达范围或下标生成实现。
        let oracle_start = std::time::Instant::now();
        let mut distribution = BTreeMap::from([(0u32, 1u64)]);
        let mut first_decoration_haste = 0;
        for (slot, options) in &axes {
            let values: Vec<u32> = options
                .iter()
                .map(|id| match (slot, id) {
                    (Some(slot), Some(id)) => {
                        catalog
                            .enchants
                            .iter()
                            .find(|e| e.slot == *slot && e.id == *id)
                            .unwrap()
                            .stats
                            .jiasu
                    }
                    (None, Some(id)) => {
                        catalog
                            .wucai
                            .iter()
                            .find(|w| w.id == *id)
                            .unwrap()
                            .stats
                            .jiasu
                    }
                    (_, None) => 0,
                })
                .collect();
            first_decoration_haste += values[0];
            let mut next = BTreeMap::new();
            for (haste, count) in distribution {
                for value in &values {
                    *next.entry(haste + value).or_insert(0u64) += count;
                }
            }
            distribution = next;
        }
        assert_eq!(distribution.values().sum::<u64>(), decoration_count);
        let bounds = req
            .constraints
            .stats
            .get("jiasu")
            .cloned()
            .unwrap_or_default();
        let mut expected = 0u64;
        let mut gear_matching_sets = 0u64;
        for gear in 0..gear_count {
            let loadout = decode_loadout(gear * decoration_count, &candidates, &axes, &base);
            let composed = catalog.compose_validated(&loadout, rules.as_ref()).unwrap();
            if req
                .constraints
                .required_sets
                .iter()
                .any(|(set, min)| composed.set_counts.get(set).copied().unwrap_or(0) < *min)
                || (!req.constraints.allow_cross_source_2plus2
                    && armor2_count(&catalog, &composed) > 1)
            {
                continue;
            }
            gear_matching_sets += 1;
            let gear_haste = composed.stats.jiasu - first_decoration_haste;
            for (decoration_haste, count) in &distribution {
                let haste = gear_haste + decoration_haste;
                if bounds.min.is_none_or(|min| haste >= min)
                    && bounds.max.is_none_or(|max| haste <= max)
                {
                    expected += count;
                }
            }
        }
        let oracle_ms = oracle_start.elapsed().as_secs_f64() * 1000.0;
        eprintln!(
            "oracle: gear_matching_sets={gear_matching_sets} feasible={expected} elapsed_ms={oracle_ms:.3}"
        );

        let start = std::time::Instant::now();
        let progress = OptimizeProgress::new();
        progress.total.store(total, Ordering::Relaxed);
        let mut search =
            crate::joint_search::JointSearch::new(&catalog, &req, &candidates, &axes, &base)
                .unwrap();
        let mut feasible = 0u64;
        let mut sampled = 0u64;
        let mut last_report = std::time::Instant::now();
        loop {
            let batch = search.next_batch(32768, &progress);
            if batch.is_empty() {
                break;
            }
            feasible += batch.len() as u64;
            progress
                .visited
                .fetch_add(batch.len() as u64, Ordering::Relaxed);
            // 每批首尾使用正式合成器复核，其余叶子仍全部由生产筛选器实际产出计数。
            for flat in [batch[0], batch[batch.len() - 1]] {
                let loadout = decode_loadout(flat, &candidates, &axes, &base);
                let composed = catalog.compose_validated(&loadout, rules.as_ref()).unwrap();
                assert!(req.constraints.satisfied(&composed).unwrap());
                assert!(
                    req.constraints.allow_cross_source_2plus2
                        || armor2_count(&catalog, &composed) <= 1
                );
                sampled += 1;
            }
            if last_report.elapsed().as_secs() >= 2 {
                eprintln!(
                    "filter {:.2}% visited={} feasible={feasible} elapsed={:.2}s",
                    progress.visited.load(Ordering::Relaxed) as f64 * 100.0 / total as f64,
                    progress.visited.load(Ordering::Relaxed),
                    start.elapsed().as_secs_f64()
                );
                last_report = std::time::Instant::now();
            }
        }
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(progress.visited.load(Ordering::Relaxed), total);
        assert_eq!(feasible, expected, "生产筛选与独立 DP/合成计数不一致");
        let report = serde_json::json!({
            "request_file": path, "raw_combinations": total, "gear_combinations": gear_count,
            "decoration_combinations": decoration_count, "gear_matching_sets": gear_matching_sets,
            "feasible": feasible, "pruned": total-feasible, "visited": total,
            "verified_compositions": sampled, "filter_elapsed_ms": elapsed_ms,
            "oracle_elapsed_ms": oracle_ms, "oracle_feasible": expected,
            "complete": true, "timeline_evaluated": 0, "mc_evaluated": 0,
            "baseline_haste": baseline.stats.jiasu,
            "gear_axes": candidates.iter().enumerate().map(|(i, c)| serde_json::json!({"slot":Slot::ALL[i],"options":c.len()})).collect::<Vec<_>>(),
            "decoration_axes": axes.iter().map(|(slot,options)| serde_json::json!({"slot":slot,"options":options.len()})).collect::<Vec<_>>()
        });
        let output = std::env::var("JPCG_FILTER_REPORT")
            .unwrap_or_else(|_| "/tmp/jpcg-current-filter-result.json".into());
        std::fs::write(output, serde_json::to_string_pretty(&report).unwrap()).unwrap();
        eprintln!("{}", serde_json::to_string_pretty(&report).unwrap());
    }
}

#[cfg(test)]
mod audit_regressions {
    use super::*;
    fn fixture() -> (Catalog, OptimizeRequest) {
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/synthetic/gear");
        (
            Catalog::load(&root).unwrap(),
            toml::from_str(&std::fs::read_to_string(root.join("sample-request.toml")).unwrap())
                .unwrap(),
        )
    }
    #[test]
    fn cross_school_candidates_can_be_selected_locked_and_do_not_count_as_own_set() {
        let (mut catalog, mut req) = fixture();
        let mut foreign = catalog
            .templates
            .iter()
            .find(|t| {
                t.choice.slot == Slot::Tou
                    && t.choice.source == Source::Weiwang
                    && t.choice.tier == Tier::Base
            })
            .unwrap()
            .clone();
        foreign.school = "foreign".into();
        foreign.cross_school = true;
        foreign.stats.gongji += 1000;
        let selected = canonical_choice(&foreign.choice, &foreign.school, &req.loadout.school);
        catalog.templates.push(foreign);
        req.constraints.locked.push(selected.clone());
        let out = optimize_with(&catalog, &req, |c| Ok(c.stats.gongji as f64), |_| true).unwrap();
        let best = out.best.unwrap();
        assert!(best.composed.loadout.choices.contains(&selected));
        assert_eq!(best.composed.set_counts["sample-weiwang"], 4);
    }
    #[test]
    fn probability_ranking_has_priority_over_any_damage_tie_break() {
        let (catalog, req) = fixture();
        let c = catalog.compose(&req.loadout).unwrap();
        let low = candidate(
            c.clone(),
            Score {
                value: 25000.0 / 50000.0,
                damage: 30_000_000.0,
            },
        );
        let high = candidate(
            c.clone(),
            Score {
                value: 25001.0 / 50000.0,
                damage: 1_000_000.0,
            },
        );
        assert!(better(&high, &low));
        let tied = candidate(
            c,
            Score {
                value: high.score,
                damage: 2_000_000.0,
            },
        );
        assert!(better(&tied, &high));
        assert!(high.score <= 1.0);
    }
    #[test]
    fn damage_only_locks_defensive_enchant_to_user_choice_but_respects_constraints() {
        let (mut catalog, mut req) = fixture();
        req.constraints.stats.clear();
        for (id, huajin) in [("def-a", 10), ("def-b", 20)] {
            catalog.enchants.push(EnchantDef {
                slot: Slot::Tou,
                id: id.into(),
                name: id.into(),
                stats: Stats {
                    huajin,
                    ..Default::default()
                },
            });
        }
        req.loadout.enchants.insert(Slot::Tou, "def-b".into());
        let full = joint_space(&catalog, &req, false).unwrap();
        assert_eq!(
            full.axes
                .iter()
                .filter(|(slot, _)| *slot == Some(Slot::Tou))
                .count(),
            1
        );
        let damage = joint_space(&catalog, &req, true).unwrap();
        assert!(!damage.axes.iter().any(|(slot, _)| *slot == Some(Slot::Tou)));
        assert_eq!(damage.base.enchants[&Slot::Tou], "def-b");
        assert_eq!(damage.combinations * 2, full.combinations);
        assert_eq!(damage.axes.len() + 1, full.axes.len());
        req.constraints.stats.insert(
            "huajin".into(),
            Bounds {
                min: Some(1),
                max: None,
            },
        );
        assert_eq!(joint_space(&catalog, &req, true).unwrap().axes.len(), 2);
        catalog.enchants[0].stats.huixin = 1;
        req.constraints.stats.clear();
        assert_eq!(joint_space(&catalog, &req, true).unwrap().axes.len(), 2);
    }
    #[test]
    fn early_cancellation_is_not_reset_and_errors_mark_done() {
        let (catalog, mut req) = fixture();
        req.max_combinations = 0;
        let progress = OptimizeProgress::new();
        progress.cancel();
        assert!(optimize_joint(&catalog, &req, &progress).is_err());
        assert!(progress.cancel.load(Ordering::Relaxed));
        assert!(progress.done.load(Ordering::Relaxed));
    }
}
