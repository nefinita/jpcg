// ============================================================================
// timeline — #92 时序引擎（形态 / GCD 通道 / 充能 / 读条 / 引导 / dot / 定时 buff）
//
// 设计要点（见 .scratch/combat/design-combat.md）：
//   - 时间量化：16 帧/秒；加速后时长 = floor(基础帧 / (1+率)) / 16（阶梯）。
//   - 加速上限：final = min(sum(受上限来源), 25%) + sum(突破上限来源)。
//   - GCD 用「通道（lane）」隔离：不同形态/双持等各自一条通道，互不阻塞。
//   - 形态由数据驱动：FormDef + 技能 on_cast（切形态 / 额外 GCD 通道 / 锁技能），
//     引擎不写任何门派分支。
//   - dot：施加时快照进攻属性与加速（决定跳数、逐跳时间点、每跳基础伤害），
//     每跳结算时用「当前」目标减伤重算（目标减伤实时）。
//   - 旧 engine.rs（按步累加）保持不动，作为兼容层。
// ============================================================================

use std::collections::HashMap;

use jpcg_core::engine::atkcal::JpcgConfig;
use jpcg_core::type_set::buff::{BuffConfig, TimedBuffDef};
use jpcg_core::type_set::coefficient::CoefficientConfig;
use jpcg_core::type_set::hostilepile::HostilepileConfig;
use jpcg_core::type_set::player::PlayerConfig;
use jpcg_core::type_set::skilltype::Skilltype;
use jpcg_core::type_set::timing;
use jpcg_core::type_set::xinfa::XinfaConfig;

// 形态 / 施放副作用 / 技能键的数据结构由 core 数据层定义（与 data/combat/*.toml 一一对应），
// 时序引擎直接复用，避免两套类型。
pub use jpcg_core::type_set::combat::{CombatData, FormDef, OnCast, SkillKey};

/// 带时序数据的技能条目
#[derive(Debug, Clone)]
pub struct TimingSkill {
    pub skill: Skilltype,
    /// 所属形态 id（None = 任意形态可用）
    pub form: Option<String>,
    /// 该技能占用的 GCD 通道（空 = "default"）
    pub gcd_lane: String,
    /// 施放副作用
    pub on_cast: OnCast,
}

impl TimingSkill {
    /// 从「shuxing 技能数据 + combat overlay」构造时序技能条目：
    /// overlay 的时序字段直接覆盖到技能上（cast_time / cooldown / 充能 / buff_id 等），
    /// 形态与 GCD 通道取 overlay 配置（未配置 = 任意形态 / default 通道）。
    pub fn from_combat(skill: &Skilltype, data: &CombatData) -> Self {
        let sc = data.skill_combat_for(skill);
        let mut resolved = skill.clone();
        if let Some(sc) = sc {
            sc.apply_to(&mut resolved);
        }
        // 没有显式 combat.gcd_lane 时，从技能的 gcd_group 派生稳定通道：
        // 0 = 默认通道；1 = 另一套（双形态）独立通道。避免分组信息被忽略后全部挤进 default。
        let mut gcd_lane = sc.map(|s| s.gcd_lane.clone()).unwrap_or_default();
        if gcd_lane.is_empty() && resolved.gcd_group != 0 {
            gcd_lane = format!("group_{}", resolved.gcd_group);
        }
        TimingSkill {
            skill: resolved,
            form: sc.and_then(|s| s.form.clone()),
            gcd_lane,
            on_cast: sc.map(|s| s.on_cast.clone()).unwrap_or_default(),
        }
    }

    pub fn key(&self) -> SkillKey {
        (self.skill.skill_id, self.skill.sub_id)
    }

    fn lane(&self) -> &str {
        if self.gcd_lane.is_empty() {
            "default"
        } else {
            self.gcd_lane.as_str()
        }
    }
}

/// 模拟动作：施放技能 / 手动切形态
#[derive(Debug, Clone)]
pub enum TimelineAction {
    /// 施放技能。
    /// `(skill_id, sub_id)` 可能对应多个变体（例如莫问「入月·N 层」共用同一对 id），
    /// 此时必须带 `skill_name` 指定具体条目；只给 id 且命中多个变体时该动作会被跳过。
    Cast {
        skill_id: u32,
        sub_id: u32,
        skill_name: Option<String>,
    },
    SwitchForm(String),
}

impl TimelineAction {
    /// 按技能 id 施放（仅在 (skill_id, sub_id) 唯一时可用）
    pub fn cast(skill_id: u32, sub_id: u32) -> Self {
        TimelineAction::Cast {
            skill_id,
            sub_id,
            skill_name: None,
        }
    }

    /// 按技能 id + 名称施放（用于同名 id 的多变体，如「入月·9层」）
    pub fn cast_named(skill_id: u32, sub_id: u32, skill_name: impl Into<String>) -> Self {
        TimelineAction::Cast {
            skill_id,
            sub_id,
            skill_name: Some(skill_name.into()),
        }
    }
}

/// 时序模拟输入
#[derive(Debug, Clone)]
pub struct TimelineInput {
    pub player: PlayerConfig,
    pub hostile: HostilepileConfig,
    pub xinfa: XinfaConfig,
    pub buff: BuffConfig,
    pub coeff: CoefficientConfig,
    pub skills: Vec<TimingSkill>,
    pub forms: Vec<FormDef>,
    /// 定时 buff 定义（技能 `buff_id` 引用）
    pub timed_buffs: Vec<TimedBuffDef>,
    /// 动作序列（按给定顺序，引擎计算最早可执行时刻）
    pub actions: Vec<TimelineAction>,
    /// 模拟窗口（秒）
    pub window: f32,
}

/// 事件类型
#[derive(Debug, Clone)]
pub enum TimelineEventKind {
    /// 开始施放（`hit_time` = 伤害结算时刻；引导/读条技能晚于起手）
    Cast {
        skill_name: String,
        damage: u32,
        hit_time: f32,
    },
    /// dot 单跳结算（`index` 从 1 开始）
    DotTick {
        skill_name: String,
        index: u32,
        damage: u32,
    },
    /// 形态切换
    FormSwitch { from: String, to: String },
    /// buff 生效
    BuffGain { buff_id: String, stacks: u8 },
}

/// 时间轴事件
#[derive(Debug, Clone)]
pub struct TimelineEvent {
    pub time: f32,
    pub kind: TimelineEventKind,
}

/// 时序模拟结果
#[derive(Debug, Clone, Default)]
pub struct TimelineResult {
    pub events: Vec<TimelineEvent>,
    pub total_damage: f64,
    pub window_dps: f64,
    /// 窗口内未能施放的动作数（GCD/CD/形态不满足或超出窗口）
    pub skipped_actions: u32,
}

// ============ 内部运行态 ============

/// 充能状态：剩余层数 / 上限 / 恢复间隔 / 下一层恢复时刻（满层 = +∞）
#[derive(Debug, Clone, Copy)]
struct ChargeState {
    left: u32,
    max: u32,
    interval: f32,
    next: f32,
}

#[derive(Debug, Clone)]
struct ActiveBuff {
    def_index: usize,
    stacks: u8,
    expires: f32,
}

/// dot 施加时快照（进攻侧），每跳按当前 hostile 重算
#[derive(Debug, Clone)]
struct DotInstance {
    skill_name: String,
    skill: Skilltype,
    /// 施加快照：玩家 / 心法 / buff / 系数
    player: PlayerConfig,
    xinfa: XinfaConfig,
    buff: BuffConfig,
    coeff: CoefficientConfig,
    /// 每跳时间点（绝对时间，施加时按当时加速率算好）
    tick_times: Vec<f32>,
}

impl DotInstance {
    fn per_tick_damage(&self, hostile: &HostilepileConfig, index: usize) -> u32 {
        let cfg = JpcgConfig::new_with_config(
            &self.player,
            hostile,
            &self.skill,
            &self.xinfa,
            &self.buff,
            &self.coeff,
        );
        let damage = cfg.q_cal();
        damage.dot_jumps.get(index).copied().unwrap_or(0)
    }
}

struct Sim<'a> {
    input: &'a TimelineInput,
    events: Vec<TimelineEvent>,
    dots: Vec<DotInstance>,
    /// 各 GCD 通道就绪时刻
    lane_ready: HashMap<String, f32>,
    /// 各技能就绪时刻（按下标记录，避免同名 id 变体共享状态；有充能技能不使用）
    skill_ready: HashMap<usize, f32>,
    /// 充能（按下标记录）
    charges: HashMap<usize, ChargeState>,
    /// 当前形态 id（空 = 无形态概念）
    current_form: String,
    /// 活跃定时 buff
    active_buffs: Vec<ActiveBuff>,
    school_gcd_secs: f32,
}

impl<'a> Sim<'a> {
    fn new(input: &'a TimelineInput) -> Self {
        let school_gcd_secs = input.xinfa.gcd_base_secs();
        let current_form = input
            .forms
            .iter()
            .find(|f| f.initial)
            .or_else(|| input.forms.first())
            .map(|f| f.id.clone())
            .unwrap_or_default();
        Sim {
            input,
            events: Vec::new(),
            dots: Vec::new(),
            lane_ready: HashMap::new(),
            skill_ready: HashMap::new(),
            charges: HashMap::new(),
            current_form,
            active_buffs: Vec::new(),
            school_gcd_secs,
        }
    }

    /// 某时刻的加速率（受上限 / 突破上限分开汇总）
    fn jiasu_at(&self, t: f32) -> f32 {
        let mut capped = self.input.player.guo_jiasu_with(&self.input.coeff);
        let mut uncapped = 0.0f32;
        for a in &self.active_buffs {
            if a.expires > t {
                let def = &self.input.timed_buffs[a.def_index];
                let bonus = def.jiasu_rate_bonus(a.stacks);
                if def.jiasu_ignore_cap() {
                    uncapped += bonus;
                } else {
                    capped += bonus;
                }
            }
        }
        timing::combine_jiasu(capped, uncapped)
    }

    /// 活跃 buff 汇总成运行期 BuffConfig（伤害计算用）
    fn buff_at(&self, t: f32) -> BuffConfig {
        let mut cfg = self.input.buff.clone();
        for a in &self.active_buffs {
            if a.expires > t {
                let def = &self.input.timed_buffs[a.def_index];
                for e in &def.effects {
                    e.apply(&mut cfg, a.stacks as f32);
                }
            }
        }
        cfg
    }

    fn form(&self, id: &str) -> Option<&'a FormDef> {
        self.input.forms.iter().find(|f| f.id == id)
    }

    /// 技能在当前形态下是否可选
    fn form_allows(&self, sk: &TimingSkill) -> bool {
        if self.input.forms.is_empty() {
            return true;
        }
        match &sk.form {
            Some(form_id) => *form_id == self.current_form,
            None => true,
        }
    }

    /// 通道默认 GCD 基础（形态 > 心法/门派）
    fn lane_base(&self, lane: &str) -> f32 {
        self.form(lane)
            .map(|f| f.gcd_base_secs(self.school_gcd_secs))
            .unwrap_or(self.school_gcd_secs)
    }

    /// 某通道的 GCD 时长（秒，含加速与 25% 封顶）
    fn lane_gcd(&self, lane: &str, rate: f32) -> f32 {
        // 注意：rate 已由 `jiasu_at` 按「25% 封顶 + 突破上限」合并，这里不能再夹一次，
        // 否则可突破上限的加速会被二次截断。
        timing::quantize_secs(self.lane_base(lane), rate)
    }

    /// 技能自身占用的 GCD 时长：技能 `gcd_base` 覆盖形态/心法基础；
    /// `gcd_kind=2`（固定 GCD）不受加速影响。
    fn skill_gcd(&self, sk: &TimingSkill, rate: f32) -> f32 {
        if !sk.skill.occupies_gcd() {
            return 0.0;
        }
        let base = if sk.skill.gcd_base > 0.0 {
            sk.skill.gcd_base
        } else {
            self.lane_base(sk.lane())
        };
        if sk.skill.gcd_kind == 2 {
            base
        } else {
            timing::quantize_secs(base, rate)
        }
    }

    /// 按下标结算已恢复的充能层数，并推进下一层恢复时刻
    fn settle_charges(&mut self, index: usize, at: f32) {
        let Some(state) = self.charges.get_mut(&index) else {
            return;
        };
        if state.interval <= 0.0 {
            state.left = state.max;
            state.next = f32::INFINITY;
            return;
        }
        while state.left < state.max && state.next <= at {
            state.left += 1;
            state.next += state.interval;
        }
        if state.left >= state.max {
            state.left = state.max;
            state.next = f32::INFINITY;
        }
    }

    /// (skill_id, sub_id) 命中的所有技能下标（同名 id 的多个变体可能不止一个）
    fn indices_for_key(&self, key: SkillKey) -> Vec<usize> {
        self.input
            .skills
            .iter()
            .enumerate()
            .filter(|(_, s)| s.skill.skill_id == key.0 && s.skill.sub_id == key.1)
            .map(|(i, _)| i)
            .collect()
    }

    /// 解析施放目标：id 唯一时直接命中；命中多个变体时必须用名称消歧，否则返回 None（跳过）。
    fn resolve_skill(&self, skill_id: u32, sub_id: u32, name: Option<&str>) -> Option<usize> {
        let candidates: Vec<usize> = self
            .input
            .skills
            .iter()
            .enumerate()
            .filter(|(_, s)| s.skill.skill_id == skill_id && s.skill.sub_id == sub_id)
            .map(|(i, _)| i)
            .collect();
        match candidates.as_slice() {
            [] => None,
            [only] => Some(*only),
            _ => {
                let name = name?;
                let mut named = candidates
                    .iter()
                    .copied()
                    .filter(|&i| self.input.skills[i].skill.skill_name == name);
                match (named.next(), named.next()) {
                    (Some(i), None) => Some(i),
                    _ => None,
                }
            }
        }
    }

    /// 技能最早可施放时刻（形态已校验；GCD 通道 + CD/充能）
    fn earliest_start(&self, index: usize) -> f32 {
        let sk = &self.input.skills[index];
        let mut t: f32 = 0.0;
        // CD / 充能
        if let Some(&ready) = self.skill_ready.get(&index) {
            t = t.max(ready);
        }
        if let Some(state) = self.charges.get(&index)
            && state.left == 0
        {
            t = t.max(state.next);
        }
        // GCD 通道
        if sk.skill.occupies_gcd() {
            t = t.max(*self.lane_ready.get(sk.lane()).unwrap_or(&0.0));
        }
        t
    }

    /// 执行一次施放
    fn cast(&mut self, sk_index: usize, start: f32) -> f32 {
        let sk = &self.input.skills[sk_index];
        let rate = self.jiasu_at(start);
        let cast_time = sk.skill.cast_time_with_haste(rate);
        let hit_time = start + cast_time;
        let buff_cfg = self.buff_at(start);
        let damage = JpcgConfig::new_with_config(
            &self.input.player,
            &self.input.hostile,
            &sk.skill,
            &self.input.xinfa,
            &buff_cfg,
            &self.input.coeff,
        )
        .q_cal();

        // DOT 技能的 q_damage 已是全部跳数之和，伤害由 DotTick 事件按实际跳点计入；
        // 这里只记起手事件，避免与 dot 跳点重复累加。
        let direct_damage = if damage.dot_jumps.is_empty() {
            damage.q_damage
        } else {
            0
        };
        self.events.push(TimelineEvent {
            time: start,
            kind: TimelineEventKind::Cast {
                skill_name: sk.skill.skill_name.clone(),
                damage: direct_damage,
                hit_time,
            },
        });

        // ---- dot：进攻侧快照（伤害/加速/跳数），逐跳排在时间轴上 ----
        if !damage.dot_jumps.is_empty() {
            let ticks = damage.dot_jumps.len() as u32;
            let total = timing::quantize_secs(sk.skill.dot_duration, rate);
            let stamp = |i: u32| hit_time + total * i as f32 / ticks as f32;
            self.dots.push(DotInstance {
                skill_name: sk.skill.skill_name.clone(),
                skill: sk.skill.clone(),
                player: self.input.player.clone(),
                xinfa: self.input.xinfa.clone(),
                buff: buff_cfg.clone(),
                coeff: self.input.coeff.clone(),
                tick_times: (1..=ticks).map(stamp).collect(),
            });
        }

        // ---- 占用 GCD 通道（自身通道 + 副作用额外通道） ----
        // 自身通道取技能自己的 gcd_base / 固定 GCD 语义；副作用通道按通道默认值。
        let own_gcd = self.skill_gcd(sk, rate);
        if sk.skill.occupies_gcd() {
            let now = self.lane_ready.get(sk.lane()).copied().unwrap_or(0.0);
            self.lane_ready
                .insert(sk.lane().to_string(), now.max(start + own_gcd));
        }
        for lane in &sk.on_cast.gcd_lanes {
            let g = self.lane_gcd(lane, rate);
            let now = self.lane_ready.get(lane).copied().unwrap_or(0.0);
            self.lane_ready.insert(lane.clone(), now.max(start + g));
        }
        // 锁技能（如断云把孤锋破浪顶进 GCD）；同名 id 的所有变体一起锁
        for lk in &sk.on_cast.lock_skills {
            for index in self.indices_for_key(*lk) {
                let now = self.skill_ready.get(&index).copied().unwrap_or(0.0);
                self.skill_ready.insert(index, now.max(start + own_gcd));
            }
        }

        // ---- CD / 充能 ----
        let max_charges = sk.skill.max_charges();
        if max_charges > 1 {
            self.settle_charges(sk_index, start);
            if let Some(state) = self.charges.get_mut(&sk_index) {
                let was_full = state.left >= state.max;
                state.left = state.left.saturating_sub(1);
                if was_full {
                    // 从满层消耗第一层时，恢复计时从这一刻开始
                    state.next = if state.interval > 0.0 {
                        start + state.interval
                    } else {
                        f32::INFINITY
                    };
                }
            } else {
                let interval = sk.skill.recharge.max(0.0);
                self.charges.insert(
                    sk_index,
                    ChargeState {
                        left: max_charges - 1,
                        max: max_charges,
                        interval,
                        next: if interval > 0.0 {
                            start + interval
                        } else {
                            f32::INFINITY
                        },
                    },
                );
            }
        } else if sk.skill.cooldown > 0.0 {
            let cd = sk.skill.cooldown_with_haste(rate);
            let now = self.skill_ready.get(&sk_index).copied().unwrap_or(0.0);
            self.skill_ready.insert(sk_index, now.max(start + cd));
        }

        // ---- 定时 buff（只提供 buff 的技能） ----
        if !sk.skill.buff_id.is_empty()
            && let Some(def_index) = self
                .input
                .timed_buffs
                .iter()
                .position(|b| b.id == sk.skill.buff_id)
        {
            let def = &self.input.timed_buffs[def_index];
            let max = def.max_stacks();
            let refresh = def.refresh;
            let buff_id = def.id.clone();
            // duration=0 表示常驻（不随时间失效）
            let expires = if def.duration > 0.0 {
                start + def.duration
            } else {
                f32::INFINITY
            };
            let mut applied_stacks: Option<u8> = None;
            if refresh {
                // refresh=true：叠加层数并刷新持续时间（已有实例合并为一层）
                if let Some(a) = self
                    .active_buffs
                    .iter_mut()
                    .find(|a| a.def_index == def_index && a.expires > start)
                {
                    if a.stacks < max {
                        a.stacks += 1;
                        applied_stacks = Some(a.stacks);
                    }
                    a.expires = expires;
                }
            }
            if applied_stacks.is_none() {
                // refresh=false：每个实例独立计时（受总层数上限约束）
                let active_count = self
                    .active_buffs
                    .iter()
                    .filter(|a| a.def_index == def_index && a.expires > start)
                    .count();
                let can_push = if refresh {
                    true
                } else {
                    active_count < max as usize
                };
                if can_push {
                    self.active_buffs.push(ActiveBuff {
                        def_index,
                        stacks: 1,
                        expires,
                    });
                    applied_stacks = Some(1);
                }
            }
            if let Some(stacks) = applied_stacks {
                self.events.push(TimelineEvent {
                    time: start,
                    kind: TimelineEventKind::BuffGain { buff_id, stacks },
                });
            }
        }

        // ---- 形态切换 ----
        if let Some(to) = &sk.on_cast.switch_form
            && *to != self.current_form
            && self.form(to).is_some()
        {
            let from = self.current_form.clone();
            self.current_form = to.clone();
            self.events.push(TimelineEvent {
                time: start,
                kind: TimelineEventKind::FormSwitch {
                    from,
                    to: to.clone(),
                },
            });
        }

        hit_time
    }

    fn run(mut self) -> TimelineResult {
        let mut now: f32 = 0.0;
        let mut skipped = 0u32;

        for action in &self.input.actions {
            match action {
                TimelineAction::SwitchForm(to) => {
                    if self.form(to).is_some() && *to != self.current_form {
                        let from = self.current_form.clone();
                        self.current_form = to.clone();
                        self.events.push(TimelineEvent {
                            time: now,
                            kind: TimelineEventKind::FormSwitch {
                                from,
                                to: to.clone(),
                            },
                        });
                    } else {
                        skipped += 1;
                    }
                }
                TimelineAction::Cast {
                    skill_id,
                    sub_id,
                    skill_name,
                } => {
                    let Some(index) = self.resolve_skill(*skill_id, *sub_id, skill_name.as_deref())
                    else {
                        // id 无法唯一确定具体变体（且未提供 skill_name）→ 跳过，避免静默取错条目
                        skipped += 1;
                        continue;
                    };
                    // 先按当前时刻结算充能，再判断能否施放
                    self.settle_charges(index, now);
                    if !self.form_allows(&self.input.skills[index]) {
                        skipped += 1;
                        continue;
                    }
                    let start = self.earliest_start(index).max(now);
                    if start > self.input.window {
                        skipped += 1;
                        continue;
                    }
                    now = self.cast(index, start);
                }
            }
        }

        // ---- dot 逐跳结算（每跳用当前 hostile 重算减伤） ----
        for dot in &self.dots {
            for (i, &t) in dot.tick_times.iter().enumerate() {
                if t > self.input.window {
                    continue;
                }
                let dmg = dot.per_tick_damage(&self.input.hostile, i);
                self.events.push(TimelineEvent {
                    time: t,
                    kind: TimelineEventKind::DotTick {
                        skill_name: dot.skill_name.clone(),
                        index: (i + 1) as u32,
                        damage: dmg,
                    },
                });
            }
        }

        self.events.sort_by(|a, b| a.time.total_cmp(&b.time));

        let mut total = 0f64;
        for e in &self.events {
            match &e.kind {
                // 读条/引导的伤害在命中时刻结算：只有命中落在窗口内才计入总伤
                TimelineEventKind::Cast {
                    damage, hit_time, ..
                } => {
                    if *hit_time <= self.input.window {
                        total += *damage as f64;
                    }
                }
                TimelineEventKind::DotTick { damage, .. } => total += *damage as f64,
                _ => {}
            }
        }
        let window_dps = if self.input.window > 0.0 {
            total / self.input.window as f64
        } else {
            0.0
        };

        TimelineResult {
            events: self.events,
            total_damage: total,
            window_dps,
            skipped_actions: skipped,
        }
    }
}

/// 运行时序模拟
pub fn simulate(input: &TimelineInput) -> TimelineResult {
    Sim::new(input).run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jpcg_core::type_set::buff::BuffEffect;

    fn player() -> PlayerConfig {
        PlayerConfig {
            jcsx: "gengu".into(),
            jichu_shuxing: 21371,
            jichu_gongji: 64329,
            huixin_dengji: 61877,
            huixin_xiaoguo: 2925,
            pofang_dengji: 109160,
            wuqi_shanghai: 0,
            zuizhong_gongji: 0,
            jiasu_dengji: 0,
        }
    }

    fn hostile() -> HostilepileConfig {
        HostilepileConfig {
            waigong_fangyu: 15176,
            neigong_fangyu: 21388,
            yujin_dengji: 5047,
            huajin_dengji: 59402,
            jianshang_bili: 0,
            target_hp: 2_000_000,
            max_hp: 0,
            current_hp: 0,
        }
    }

    fn xinfa() -> XinfaConfig {
        XinfaConfig {
            profession: "mowen".into(),
            xinfa_name: "莫问".into(),
            xinfa_nom: "根骨".into(),
            atk_up: 1.96,
            pofang_up: 2.0,
            huixin_up: 0.0,
            gcd_base: 0.0,
        }
    }

    fn skill(id: u32, sub: u32, name: &str) -> Skilltype {
        Skilltype {
            skill_name: name.into(),
            skill_id: id,
            sub_id: sub,
            base_damage1: 100,
            base_damage2: 120,
            atk_xishu: 1.0,
            ..Skilltype::default()
        }
    }

    fn timing(skill: Skilltype, lane: &str) -> TimingSkill {
        TimingSkill {
            skill,
            form: None,
            gcd_lane: lane.into(),
            on_cast: OnCast::default(),
        }
    }

    fn base_input(
        skills: Vec<TimingSkill>,
        actions: Vec<TimelineAction>,
        window: f32,
    ) -> TimelineInput {
        TimelineInput {
            player: player(),
            hostile: hostile(),
            xinfa: xinfa(),
            buff: BuffConfig::default(),
            coeff: CoefficientConfig::default(),
            skills,
            forms: Vec::new(),
            timed_buffs: Vec::new(),
            actions,
            window,
        }
    }

    fn cast_times(result: &TimelineResult) -> Vec<f32> {
        result
            .events
            .iter()
            .filter(|e| matches!(e.kind, TimelineEventKind::Cast { .. }))
            .map(|e| e.time)
            .collect()
    }

    #[test]
    fn single_gcd_chain_advances_by_1_5s() {
        // 同一条 GCD 通道：两次瞬发技能应相隔 1.5 秒
        let skills = vec![timing(skill(1, 0, "A"), "default")];
        let input = base_input(
            skills,
            vec![TimelineAction::cast(1, 0), TimelineAction::cast(1, 0)],
            10.0,
        );
        let result = simulate(&input);
        assert_eq!(cast_times(&result), vec![0.0, 1.5]);
    }

    #[test]
    fn independent_gcd_lanes_do_not_block_each_other() {
        // 双形态：两条通道各自 1.5s / 1.0s，交替施放互不阻塞
        let a = timing(skill(1, 0, "单持技能"), "single");
        let b = TimingSkill {
            form: Some("dual".into()),
            ..timing(skill(2, 0, "双持技能"), "dual")
        };
        let mut input = base_input(
            vec![a, b],
            vec![TimelineAction::cast(1, 0), TimelineAction::cast(2, 0)],
            10.0,
        );
        input.forms = vec![
            FormDef {
                id: "single".into(),
                name: "单持".into(),
                gcd_base: 1.5,
                initial: true,
                skills: vec![(1, 0)],
            },
            FormDef {
                id: "dual".into(),
                name: "双持".into(),
                gcd_base: 1.0,
                initial: false,
                skills: vec![(2, 0)],
            },
        ];
        let mut result = simulate(&input);
        // 双持技能不可在单持形态施放 → 被跳过
        assert_eq!(result.skipped_actions, 1);
        assert_eq!(cast_times(&result), vec![0.0]);

        // 让单持技能施放后切双持，则双持技能可以在 0 秒（独立通道）后立即施放
        let a2 = TimingSkill {
            on_cast: OnCast {
                switch_form: Some("dual".into()),
                ..OnCast::default()
            },
            ..timing(skill(1, 0, "切形态技能"), "single")
        };
        result = simulate(&TimelineInput {
            skills: vec![a2, timing(skill(2, 0, "双持技能"), "dual")],
            actions: vec![TimelineAction::cast(1, 0), TimelineAction::cast(2, 0)],
            ..input.clone()
        });
        assert_eq!(result.skipped_actions, 0);
        assert_eq!(cast_times(&result).len(), 2);
    }

    #[test]
    fn on_cast_extra_lane_blocks_other_form_skills() {
        // 刀宗：断云势（单持）额外占用双持通道 → 双持技能被顶到 1.5s
        let duanyun = TimingSkill {
            on_cast: OnCast {
                switch_form: Some("dual".into()),
                gcd_lanes: vec!["dual".into()],
                lock_skills: vec![(2, 0)],
            },
            ..timing(skill(1, 0, "断云势"), "single")
        };
        let gufeng = TimingSkill {
            form: Some("dual".into()),
            ..timing(skill(2, 0, "孤锋破浪"), "dual")
        };
        let mut input = base_input(
            vec![duanyun, gufeng],
            vec![TimelineAction::cast(1, 0), TimelineAction::cast(2, 0)],
            10.0,
        );
        input.forms = vec![
            FormDef {
                id: "single".into(),
                name: "单持".into(),
                gcd_base: 1.5,
                initial: true,
                skills: vec![(1, 0)],
            },
            FormDef {
                id: "dual".into(),
                name: "双持".into(),
                gcd_base: 1.0,
                initial: false,
                skills: vec![(2, 0)],
            },
        ];
        let result = simulate(&input);
        assert_eq!(result.skipped_actions, 0);
        let times = cast_times(&result);
        assert_eq!(times[0], 0.0);
        // 孤锋破浪同时吃断云的 GCD（1.5s）→ 最早 1.5 秒
        assert!((times[1] - 1.5).abs() < 1e-4, "实际 {}", times[1]);
    }

    #[test]
    fn cast_time_uses_haste_quantization() {
        // 读条 1.5 秒 + 20% 加速：floor(24/1.2)=20 帧 = 1.25 秒
        let mut sk = skill(1, 0, "读条");
        sk.cast_time = 1.5;
        let input = base_input(
            vec![timing(sk, "default")],
            vec![TimelineAction::cast(1, 0)],
            10.0,
        );
        let result = simulate(&input);
        let hit = result
            .events
            .iter()
            .find_map(|e| match &e.kind {
                TimelineEventKind::Cast { hit_time, .. } => Some(*hit_time),
                _ => None,
            })
            .unwrap();
        assert_eq!(hit, 1.5, "无加速时读条 1.5 秒");

        let mut input2 = input.clone();
        input2.player.jiasu_dengji = 2024; // 20.02%
        input2.coeff.jiasu_xishu = 101.079;
        let result2 = simulate(&input2);
        let hit2 = result2
            .events
            .iter()
            .find_map(|e| match &e.kind {
                TimelineEventKind::Cast { hit_time, .. } => Some(*hit_time),
                _ => None,
            })
            .unwrap();
        assert_eq!(
            hit2, 1.1875,
            "20.02% 加速后读条 floor(24/1.2002)=19 帧 = 1.1875 秒"
        );
    }

    #[test]
    fn dot_ticks_are_scheduled_on_timeline() {
        // 18 秒 9 跳 dot：逐跳排在施加后 2s..18s（无加速），总伤害 = q_cal 的 dot_jumps 之和
        let mut sk = skill(1, 0, "dot");
        sk.dot_flag = 1;
        sk.dot_interval = 2.0;
        sk.dot_duration = 18.0;
        sk.dot_up = 0.0;
        let input = base_input(
            vec![timing(sk.clone(), "default")],
            vec![TimelineAction::cast(1, 0)],
            30.0,
        );
        let result = simulate(&input);
        let ticks: Vec<f32> = result
            .events
            .iter()
            .filter(|e| matches!(e.kind, TimelineEventKind::DotTick { .. }))
            .map(|e| e.time)
            .collect();
        assert_eq!(ticks.len(), 9);
        assert!((ticks[0] - 2.0).abs() < 1e-4);
        assert!((ticks[8] - 18.0).abs() < 1e-4);
        // 伤害：q_cal 的 dot 总和 = 9 跳之和
        let expected = JpcgConfig::new_with_config(
            &input.player,
            &input.hostile,
            &sk,
            &input.xinfa,
            &BuffConfig::default(),
            &input.coeff,
        )
        .q_cal();
        let expected_sum: f64 = expected.dot_jumps.iter().map(|d| *d as f64).sum();
        let tick_sum: f64 = result
            .events
            .iter()
            .filter_map(|e| match &e.kind {
                TimelineEventKind::DotTick { damage, .. } => Some(*damage as f64),
                _ => None,
            })
            .sum();
        assert!((expected_sum - tick_sum).abs() < 1.0);
    }

    #[test]
    fn charges_gate_recasts() {
        // 2 层充能、8 秒恢复：前两次可 0 / 1.5 施放，第三次要等 8 秒
        let mut sk = skill(1, 0, "充能技能");
        sk.charges = 2;
        sk.recharge = 8.0;
        let input = base_input(
            vec![timing(sk, "default")],
            vec![
                TimelineAction::cast(1, 0),
                TimelineAction::cast(1, 0),
                TimelineAction::cast(1, 0),
            ],
            20.0,
        );
        let result = simulate(&input);
        let times = cast_times(&result);
        assert_eq!(times.len(), 3);
        assert_eq!(times[0], 0.0);
        assert_eq!(times[1], 1.5);
        assert!((times[2] - 8.0).abs() < 1e-4, "实际 {}", times[2]);
    }

    #[test]
    fn haste_cap_applies_to_gcd_but_uncapped_buff_breaks_25_percent() {
        // 25% 封顶：加速等级 2024（20.02%）→ GCD 1.5 → floor(24/1.2002)=19 帧 = 1.1875 秒
        let mut input = base_input(
            vec![timing(skill(1, 0, "A"), "default")],
            vec![TimelineAction::cast(1, 0), TimelineAction::cast(1, 0)],
            10.0,
        );
        input.player.jiasu_dengji = 2024;
        input.coeff.jiasu_xishu = 101.079;
        let result = simulate(&input);
        assert_eq!(cast_times(&result)[1], 1.1875);

        // 突破上限的 buff：+10% 额外加速率 → 1.5 / 1.30 = 18.46 → 18 帧 = 1.125 秒
        input.timed_buffs = vec![TimedBuffDef {
            id: "jiasu_buff".into(),
            name: "突破加速".into(),
            duration: 100.0,
            max_stacks: 1,
            refresh: true,
            effects: vec![BuffEffect::JiasuPct(10.0)],
            ignore_cap: true,
        }];
        let mut buff_skill = skill(2, 0, "上加速");
        buff_skill.effect_type = 1;
        buff_skill.buff_id = "jiasu_buff".into();
        input.skills.push(timing(buff_skill, "default"));
        input.actions = vec![
            TimelineAction::cast(2, 0),
            TimelineAction::cast(1, 0),
            TimelineAction::cast(1, 0),
        ];
        let result = simulate(&input);
        let times = cast_times(&result);
        // 第一次上 buff 本身占 GCD（1.5s），随后两次技能在 1.5 与 1.5+1.125 秒
        assert_eq!(times.len(), 3);
        assert!(
            (times[2] - times[1] - 1.125).abs() < 1e-4,
            "{} vs {}",
            times[2],
            times[1]
        );
    }

    #[test]
    fn builds_timeline_from_combat_overlay() {
        // 数据层 → 时序引擎：data/combat TOML 里的 form / skill_combat 能被直接消费
        const COMBAT: &str = r#"
[meta]
profession = "mowen"

[[form]]
id = "single"
name = "单持"
gcd_base = 1.5
initial = true
skills = [[1, 0]]

[[form]]
id = "dual"
name = "双持"
gcd_base = 1.0

[[skill_combat]]
skill_id = 1
sub_id = 0
form = "single"
gcd_lane = "single"
on_cast = { switch_form = "dual", gcd_lanes = ["dual"], lock_skills = [[2, 0]] }

[[skill_combat]]
skill_id = 2
sub_id = 0
form = "dual"
gcd_lane = "dual"
cast_time = 2.0
"#;
        let data: CombatData = toml::from_str(COMBAT).unwrap();
        data.validate().unwrap();
        let raw = vec![skill(1, 0, "断云势"), skill(2, 0, "孤锋破浪")];
        data.validate_against_skills(&raw).unwrap();
        let skills: Vec<TimingSkill> = raw
            .iter()
            .map(|s| TimingSkill::from_combat(s, &data))
            .collect();
        let input = TimelineInput {
            skills,
            forms: data.form.clone(),
            actions: vec![TimelineAction::cast(1, 0), TimelineAction::cast(2, 0)],
            ..base_input(Vec::new(), Vec::new(), 10.0)
        };
        let result = simulate(&input);
        assert_eq!(result.skipped_actions, 0);
        let times = cast_times(&result);
        assert_eq!(times[0], 0.0);
        // 孤锋破浪读条 2.0 秒 + 被断云锁到 1.5 秒 → 起手 1.5、伤害在 3.5 结算
        assert!((times[1] - 1.5).abs() < 1e-4, "实际 {}", times[1]);
        let hit = result
            .events
            .iter()
            .find_map(|e| match &e.kind {
                TimelineEventKind::Cast { hit_time, .. } if *hit_time > 1.6 => Some(*hit_time),
                _ => None,
            })
            .unwrap();
        assert!((hit - 3.5).abs() < 1e-4, "实际 {hit}");
    }

    #[test]
    fn manual_form_switch_action() {
        let a = TimingSkill {
            form: Some("a".into()),
            ..timing(skill(1, 0, "A"), "a")
        };
        let b = TimingSkill {
            form: Some("b".into()),
            ..timing(skill(2, 0, "B"), "b")
        };
        let mut input = base_input(
            vec![a, b],
            vec![
                TimelineAction::SwitchForm("b".into()),
                TimelineAction::cast(2, 0),
            ],
            10.0,
        );
        input.forms = vec![
            FormDef {
                id: "a".into(),
                name: "A".into(),
                gcd_base: 1.5,
                initial: true,
                skills: vec![(1, 0)],
            },
            FormDef {
                id: "b".into(),
                name: "B".into(),
                gcd_base: 1.0,
                initial: false,
                skills: vec![(2, 0)],
            },
        ];
        let result = simulate(&input);
        assert_eq!(result.skipped_actions, 0);
        assert_eq!(cast_times(&result), vec![0.0]);
    }

    #[test]
    fn dot_damage_counted_by_ticks_not_double() {
        // 18 秒 9 跳 dot：总伤 = 逐跳之和（起手事件不再重复计入 q_damage 总和）
        let mut sk = skill(1, 0, "dot");
        sk.dot_flag = 1;
        sk.dot_interval = 2.0;
        sk.dot_duration = 18.0;
        let input = base_input(
            vec![timing(sk, "default")],
            vec![TimelineAction::cast(1, 0)],
            30.0,
        );
        let result = simulate(&input);
        let tick_sum: f64 = result
            .events
            .iter()
            .filter_map(|e| match &e.kind {
                TimelineEventKind::DotTick { damage, .. } => Some(*damage as f64),
                _ => None,
            })
            .sum();
        assert!(tick_sum > 0.0);
        assert!((result.total_damage - tick_sum).abs() < 1.0);
        // 起手事件本身不记直接伤害
        let cast_damage: u32 = result
            .events
            .iter()
            .filter_map(|e| match &e.kind {
                TimelineEventKind::Cast { damage, .. } => Some(*damage),
                _ => None,
            })
            .sum();
        assert_eq!(cast_damage, 0);
    }

    #[test]
    fn dot_only_counts_ticks_inside_short_window() {
        // 窗口 1 秒、首跳在 2 秒：窗口伤害应为 0
        let mut sk = skill(1, 0, "dot");
        sk.dot_flag = 1;
        sk.dot_interval = 2.0;
        sk.dot_duration = 18.0;
        let input = base_input(
            vec![timing(sk, "default")],
            vec![TimelineAction::cast(1, 0)],
            1.0,
        );
        let result = simulate(&input);
        assert!(
            result
                .events
                .iter()
                .any(|e| matches!(e.kind, TimelineEventKind::Cast { .. }))
        );
        assert!(
            !result
                .events
                .iter()
                .any(|e| matches!(e.kind, TimelineEventKind::DotTick { .. }))
        );
        assert_eq!(result.total_damage, 0.0);
        assert_eq!(result.window_dps, 0.0);
    }

    #[test]
    fn ambiguous_skill_key_is_skipped_unless_named() {
        // 入月 1/9 层共用 (14870, 44386)，只给 id 时跳过；带名称时选中正确变体
        let mut one = skill(14870, 44386, "入月·1层");
        one.atk_xishu = 0.45833334;
        let mut nine = skill(14870, 44386, "入月·9层");
        nine.atk_xishu = 4.125;
        let skills = vec![timing(one, "default"), timing(nine, "default")];

        let ambiguous = base_input(
            skills.clone(),
            vec![TimelineAction::cast(14870, 44386)],
            10.0,
        );
        let r = simulate(&ambiguous);
        assert_eq!(r.skipped_actions, 1);
        assert!(r.events.is_empty());

        let named = base_input(
            skills,
            vec![TimelineAction::cast_named(14870, 44386, "入月·9层")],
            10.0,
        );
        let r2 = simulate(&named);
        assert_eq!(r2.skipped_actions, 0);
        let name = r2
            .events
            .iter()
            .find_map(|e| match &e.kind {
                TimelineEventKind::Cast { skill_name, .. } => Some(skill_name.clone()),
                _ => None,
            })
            .unwrap();
        assert_eq!(name, "入月·9层");
    }

    #[test]
    fn charges_recover_on_continuing_timeline() {
        // 2 层充能、8 秒恢复、1.5s GCD：5 次施放 = 0 / 1.5 / 8 / 16 / 24
        let mut sk = skill(1, 0, "充能技能");
        sk.charges = 2;
        sk.recharge = 8.0;
        let input = base_input(
            vec![timing(sk, "default")],
            vec![
                TimelineAction::cast(1, 0),
                TimelineAction::cast(1, 0),
                TimelineAction::cast(1, 0),
                TimelineAction::cast(1, 0),
                TimelineAction::cast(1, 0),
            ],
            30.0,
        );
        let result = simulate(&input);
        assert_eq!(result.skipped_actions, 0);
        assert_eq!(cast_times(&result), vec![0.0, 1.5, 8.0, 16.0, 24.0]);
    }

    #[test]
    fn skill_gcd_base_overrides_form_and_fixed_gcd_ignores_haste() {
        // 技能自带 gcd_base=1.0：在 1.5s 心法下两次施放间隔 1.0s
        let mut sk = skill(1, 0, "自带 GCD");
        sk.gcd_base = 1.0;
        let input = base_input(
            vec![timing(sk, "default")],
            vec![TimelineAction::cast(1, 0), TimelineAction::cast(1, 0)],
            10.0,
        );
        assert_eq!(cast_times(&simulate(&input)), vec![0.0, 1.0]);

        // gcd_kind=2 固定 GCD：20.02% 加速下仍为 1.5s
        let mut fixed = skill(2, 0, "固定 GCD");
        fixed.gcd_kind = 2;
        let mut input2 = base_input(
            vec![timing(fixed, "default")],
            vec![TimelineAction::cast(2, 0), TimelineAction::cast(2, 0)],
            10.0,
        );
        input2.player.jiasu_dengji = 2024;
        input2.coeff.jiasu_xishu = 101.079;
        assert_eq!(cast_times(&simulate(&input2)), vec![0.0, 1.5]);
    }

    #[test]
    fn cast_straddling_window_counts_damage_only_when_hit_inside() {
        // 2 秒读条、窗口 1 秒：起手事件保留，但命中在 2 秒 → 窗口伤害 0
        let mut sk = skill(1, 0, "长读条");
        sk.cast_time = 2.0;
        let input = base_input(
            vec![timing(sk, "default")],
            vec![TimelineAction::cast(1, 0)],
            1.0,
        );
        let result = simulate(&input);
        let hit = result
            .events
            .iter()
            .find_map(|e| match &e.kind {
                TimelineEventKind::Cast { hit_time, .. } => Some(*hit_time),
                _ => None,
            })
            .unwrap();
        assert_eq!(hit, 2.0);
        assert_eq!(result.total_damage, 0.0);
    }

    #[test]
    fn permanent_buff_affects_later_casts() {
        // duration=0 常驻加速 buff：后续技能 GCD 按 25% 加速量化（1.1875s）
        let mut buff_skill = skill(9, 0, "开爆发");
        buff_skill.effect_type = 1;
        buff_skill.buff_id = "burst".into();
        let mut input = base_input(
            vec![
                timing(buff_skill, "default"),
                timing(skill(1, 0, "输出"), "default"),
            ],
            vec![
                TimelineAction::cast(9, 0),
                TimelineAction::cast(1, 0),
                TimelineAction::cast(1, 0),
            ],
            10.0,
        );
        input.timed_buffs = vec![TimedBuffDef {
            id: "burst".into(),
            name: "爆发".into(),
            duration: 0.0,
            max_stacks: 1,
            refresh: true,
            effects: vec![BuffEffect::JiasuPct(25.0)],
            ignore_cap: false,
        }];
        let result = simulate(&input);
        let times = cast_times(&result);
        assert_eq!(times.len(), 3);
        // 起手 buff 技能无读条，占 1.5s GCD；第二次输出在 1.5s，第三次在其后 1.1875s
        assert!((times[1] - 1.5).abs() < 1e-4, "实际 {:?}", times);
        assert!((times[2] - 2.6875).abs() < 1e-4, "实际 {:?}", times);
    }

    #[test]
    fn non_refresh_buff_instances_expire_independently() {
        // refresh=false：duration=3、max_stacks=2，t=0 / 1.5 各施加一次
        // → t=2.9 两层都活着（+20%），t=3 第一层到期只剩一层（+10%）
        let mut buff_skill = skill(9, 0, "上护甲");
        buff_skill.effect_type = 1;
        buff_skill.buff_id = "armor".into();
        let mut input = base_input(vec![timing(buff_skill, "default")], vec![], 10.0);
        input.timed_buffs = vec![TimedBuffDef {
            id: "armor".into(),
            name: "护甲".into(),
            duration: 3.0,
            max_stacks: 2,
            refresh: false,
            effects: vec![BuffEffect::BaseAtkPct(10.0)],
            ignore_cap: false,
        }];
        let mut sim = Sim::new(&input);
        sim.cast(0, 0.0);
        sim.cast(0, 1.5);
        assert_eq!(sim.active_buffs.len(), 2, "两层应为两个独立实例");
        assert!((sim.buff_at(2.9).base_atk_pct - 20.0).abs() < 1e-4);
        assert!(
            (sim.buff_at(3.0).base_atk_pct - 10.0).abs() < 1e-4,
            "第一层应在 t=3 到期，只剩第二层（实际 {}）",
            sim.buff_at(3.0).base_atk_pct
        );
        assert!((sim.buff_at(4.5).base_atk_pct - 0.0).abs() < 1e-4);
    }

    #[test]
    fn gcd_group_derives_independent_lanes_without_overlay() {
        // 无 combat overlay 时，gcd_group=0 / 1 应各自成为独立通道 → 两个瞬发都在 0 秒
        let mut a = skill(1, 0, "A");
        a.gcd_group = 0;
        let mut b = skill(2, 0, "B");
        b.gcd_group = 1;
        let data = CombatData::default();
        let skills = vec![
            TimingSkill::from_combat(&a, &data),
            TimingSkill::from_combat(&b, &data),
        ];
        assert_eq!(skills[0].lane(), "default");
        assert_eq!(skills[1].lane(), "group_1");
        let input = base_input(
            skills,
            vec![TimelineAction::cast(1, 0), TimelineAction::cast(2, 0)],
            10.0,
        );
        let result = simulate(&input);
        assert_eq!(cast_times(&result), vec![0.0, 0.0]);
    }
}
