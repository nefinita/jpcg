//! 联合穷举的轻量分支定界：先装备、后装饰，只为可能满足约束的叶子合成面板。
use super::model::*;
use super::optimize::DecorationAxis;
use std::sync::atomic::Ordering;

struct OptionData {
    flat: u64,
    haste: u64,
    set: Option<usize>,
    upgraded: bool,
}
struct Node {
    depth: usize,
    flat: u64,
    haste: u64,
    counts: std::sync::Arc<Vec<usize>>,
    upgraded: std::sync::Arc<Vec<usize>>,
    bonus: Option<u64>,
}

pub(super) struct JointSearch<'a> {
    axes: Vec<Vec<OptionData>>,
    gear_axes: usize,
    sets: &'a [GearSet],
    required: Vec<usize>,
    allow_cross: bool,
    haste: Bounds,
    remaining_sets: Vec<Vec<usize>>,
    min_haste: Vec<u64>,
    max_haste: Vec<u64>,
    leaves: Vec<u64>,
    bonus_max: u64,
    stack: Vec<Node>,
}

impl<'a> JointSearch<'a> {
    pub(super) fn new(
        catalog: &'a Catalog,
        req: &OptimizeRequest,
        candidates: &[Vec<Choice>],
        decorations: &[DecorationAxis],
        base: &Loadout,
    ) -> Result<Self, String> {
        let mut axes = Vec::new();
        let mut stride = 1u64;
        // 保留 decode_loadout 的混合进制下标，遍历顺序可以独立调整。
        let mut decoration_data = Vec::new();
        let mut fixed_haste = u64::from(base.base.jiasu);
        for (slot, id) in &base.enchants {
            if !decorations.iter().any(|(s, _)| *s == Some(*slot)) {
                let e = catalog
                    .enchants
                    .iter()
                    .find(|e| e.slot == *slot && e.id == *id)
                    .ok_or("附魔不存在")?;
                fixed_haste += u64::from(e.stats.jiasu);
            }
        }
        if !decorations.iter().any(|(s, _)| s.is_none())
            && let Some(id) = &base.wucai
        {
            fixed_haste += u64::from(
                catalog
                    .wucai
                    .iter()
                    .find(|w| w.id == *id)
                    .ok_or("五彩石不存在")?
                    .stats
                    .jiasu,
            );
        }
        for (slot, options) in decorations {
            let mut data = Vec::new();
            for (i, id) in options.iter().enumerate() {
                let haste = match (slot, id) {
                    (Some(slot), Some(id)) => {
                        catalog
                            .enchants
                            .iter()
                            .find(|e| e.slot == *slot && e.id == *id)
                            .ok_or("附魔不存在")?
                            .stats
                            .jiasu
                    }
                    (None, Some(id)) => {
                        catalog
                            .wucai
                            .iter()
                            .find(|w| w.id == *id)
                            .ok_or("五彩石不存在")?
                            .stats
                            .jiasu
                    }
                    (_, None) => 0,
                };
                data.push(OptionData {
                    flat: i as u64 * stride,
                    haste: u64::from(haste),
                    set: None,
                    upgraded: false,
                });
            }
            stride = stride
                .checked_mul(options.len() as u64)
                .ok_or("组合数溢出")?;
            decoration_data.push(data);
        }
        // 装饰轴从前往后解码，装备轴由 decode_flat 从后往前解码。
        let mut gear_strides = vec![0; candidates.len()];
        for i in (0..candidates.len()).rev() {
            gear_strides[i] = stride;
            stride = stride
                .checked_mul(candidates[i].len() as u64)
                .ok_or("组合数溢出")?;
        }
        for (choices, gear_stride) in candidates.iter().zip(gear_strides) {
            let mut data = Vec::new();
            for (i, choice) in choices.iter().enumerate() {
                let school = choice.school.as_deref().unwrap_or(&base.school);
                let t = catalog
                    .templates
                    .iter()
                    .find(|t| {
                        t.school == school
                            && t.choice.slot == choice.slot
                            && t.choice.source == choice.source
                            && t.choice.tier == choice.tier
                            && t.choice.variant == choice.variant
                    })
                    .ok_or("模板不存在")?;
                let haste = u64::from(t.stats.jiasu)
                    + t.sockets.iter().map(|s| u64::from(s.jiasu)).sum::<u64>();
                let set = if school == base.school {
                    t.set
                        .as_ref()
                        .and_then(|id| catalog.sets.iter().position(|s| s.id == *id))
                } else {
                    None
                };
                data.push(OptionData {
                    flat: i as u64 * gear_stride,
                    haste,
                    set,
                    upgraded: choice.tier == Tier::Upgraded,
                });
            }
            axes.push(data);
        }
        // 套装槽位优先，使四件套失败时尚未进入饰品/附魔的笛卡尔积。
        let mut order: Vec<usize> = (0..candidates.len()).collect();
        order.sort_by_key(|&i| !candidates[i][0].slot.armor());
        let mut gear: Vec<Option<Vec<OptionData>>> = axes.into_iter().map(Some).collect();
        let mut axes = Vec::new();
        for i in order {
            axes.push(gear[i].take().ok_or("重复槽位")?);
        }
        let gear_axes = axes.len();
        axes.extend(decoration_data);
        let n = axes.len();
        let mut remaining_sets = vec![vec![0; catalog.sets.len()]; n + 1];
        let mut min_haste = vec![0; n + 1];
        let mut max_haste = vec![0; n + 1];
        let mut leaves = vec![1u64; n + 1];
        for d in (0..n).rev() {
            remaining_sets[d] = remaining_sets[d + 1].clone();
            for (s, remaining) in remaining_sets[d].iter_mut().enumerate() {
                *remaining += usize::from(axes[d].iter().any(|o| o.set == Some(s)));
            }
            min_haste[d] = min_haste[d + 1] + axes[d].iter().map(|o| o.haste).min().unwrap_or(0);
            max_haste[d] = max_haste[d + 1] + axes[d].iter().map(|o| o.haste).max().unwrap_or(0);
            leaves[d] = leaves[d + 1]
                .checked_mul(axes[d].len() as u64)
                .ok_or("组合数溢出")?;
        }
        let bonus_max = catalog
            .sets
            .iter()
            .map(|s| {
                let b2 = s
                    .tiers
                    .iter()
                    .map(|t| u64::from(t.bonus2.jiasu))
                    .max()
                    .unwrap_or(0);
                let b3 = if s.kind == SetKind::Accessory {
                    s.tiers
                        .iter()
                        .filter_map(|t| t.bonus3.as_ref())
                        .map(|b| u64::from(b.jiasu))
                        .max()
                        .unwrap_or(0)
                } else {
                    0
                };
                b2 + b3
            })
            .sum();
        Ok(Self {
            axes,
            gear_axes,
            sets: &catalog.sets,
            required: catalog
                .sets
                .iter()
                .map(|s| {
                    req.constraints
                        .required_sets
                        .get(&s.id)
                        .copied()
                        .unwrap_or(0)
                })
                .collect(),
            allow_cross: req.constraints.allow_cross_source_2plus2,
            haste: req
                .constraints
                .stats
                .get("jiasu")
                .cloned()
                .unwrap_or_default(),
            remaining_sets,
            min_haste,
            max_haste,
            leaves,
            bonus_max,
            stack: vec![Node {
                depth: 0,
                flat: 0,
                haste: fixed_haste,
                counts: std::sync::Arc::new(vec![0; catalog.sets.len()]),
                upgraded: std::sync::Arc::new(vec![0; catalog.sets.len()]),
                bonus: None,
            }],
        })
    }

    fn exact_bonus(&self, node: &Node) -> Option<u64> {
        let mut bonus = 0u64;
        for (s, set) in self.sets.iter().enumerate() {
            for threshold in [2, 3] {
                if node.counts[s] < threshold || (threshold == 3 && set.kind == SetKind::Armor) {
                    continue;
                }
                let tier = if node.upgraded[s] >= threshold {
                    Tier::Upgraded
                } else {
                    Tier::Base
                };
                // 不完整的奖励数据仍交由正式合成器拒绝。
                let effect = set.tiers.iter().find(|t| t.tier == tier)?;
                bonus += u64::from(if threshold == 2 {
                    effect.bonus2.jiasu
                } else {
                    effect.bonus3.as_ref().map_or(0, |b| b.jiasu)
                });
            }
        }
        Some(bonus)
    }

    fn possible(&self, node: &Node) -> bool {
        let d = node.depth;
        if d <= self.gear_axes
            && self
                .required
                .iter()
                .enumerate()
                .any(|(s, &min)| node.counts[s] + self.remaining_sets[d][s] < min)
        {
            return false;
        }
        if d <= self.gear_axes
            && !self.allow_cross
            && self
                .sets
                .iter()
                .enumerate()
                .filter(|(s, set)| set.kind == SetKind::Armor && node.counts[*s] >= 2)
                .count()
                > 1
        {
            return false;
        }
        let (bonus_min, bonus_max) = node
            .bonus
            .map_or((0, self.bonus_max), |bonus| (bonus, bonus));
        !self
            .haste
            .min
            .is_some_and(|min| node.haste + self.max_haste[d] + bonus_max < u64::from(min))
            && !self
                .haste
                .max
                .is_some_and(|max| node.haste + self.min_haste[d] + bonus_min > u64::from(max))
    }

    /// 流式产出，不预扫描全空间。剪枝按覆盖叶子数计入 visited；评分叶子由调用方计入。
    pub(super) fn next_batch(&mut self, limit: usize, progress: &OptimizeProgress) -> Vec<u64> {
        let mut out = Vec::new();
        while out.len() < limit && !progress.cancel.load(Ordering::Relaxed) {
            let Some(mut node) = self.stack.pop() else {
                break;
            };
            if node.depth == self.gear_axes && node.bonus.is_none() {
                node.bonus = self.exact_bonus(&node);
            }
            if !self.possible(&node) {
                progress
                    .visited
                    .fetch_add(self.leaves[node.depth], Ordering::Relaxed);
            } else if node.depth == self.axes.len() {
                out.push(node.flat);
            } else {
                for option in self.axes[node.depth].iter().rev() {
                    let mut counts = node.counts.clone();
                    let mut upgraded = node.upgraded.clone();
                    if let Some(s) = option.set {
                        std::sync::Arc::make_mut(&mut counts)[s] += 1;
                        std::sync::Arc::make_mut(&mut upgraded)[s] += usize::from(option.upgraded);
                    }
                    self.stack.push(Node {
                        depth: node.depth + 1,
                        flat: node.flat + option.flat,
                        haste: node.haste + option.haste,
                        counts,
                        upgraded,
                        bonus: node.bonus,
                    });
                }
            }
        }
        out
    }
}
