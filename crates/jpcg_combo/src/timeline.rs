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

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
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
use jpcg_core::type_set::modifier::QixueHook;

/// 已激活奇穴的运行时钩子（带作用技能；targets 为空 = 对所有技能生效）
#[derive(Debug, Clone)]
pub struct ActiveHook {
    pub targets: Vec<SkillKey>,
    pub hook: QixueHook,
}

impl ActiveHook {
    fn target(&self, key: SkillKey) -> bool {
        self.targets.is_empty()
            || self
                .targets
                .iter()
                .any(|(id, sub)| *id == key.0 && (*sub == 0 || *sub == key.1))
    }
}

/// 独立 DOT 数据（覆盖层提供；与本体伤害分开结算）
#[derive(Debug, Clone)]
pub struct DotTickSpec {
    /// 用于逐跳重算的合成技能（dot_flag/interval/duration 已填好）
    pub skill: Skilltype,
    /// 每跳间隔（秒，只用于展示/校验；跳时间按 duration 均分）
    pub interval: f32,
    /// 总持续时长（秒）
    pub duration: f32,
}

/// 带时序数据的技能条目
#[derive(Debug, Clone)]
pub struct TimingSkill {
    pub skill: Skilltype,
    /// 施放获得曲风
    pub qifeng_gain: u8,
    /// 施放消耗全部曲风
    pub spend_all_qifeng: bool,
    /// 要求当前器形（None = 任意器形可用）
    pub requires_instrument: Option<String>,
    /// 要求当前曲风（None = 任意曲风可用）
    pub requires_qifeng: Option<String>,
    /// 该技能占用的 GCD 通道（空 = "default"）
    pub gcd_lane: String,
    /// 施放副作用
    pub on_cast: OnCast,
    /// 独立 DOT（None = 本体带 dot 或无 DOT）
    pub dot_tick: Option<DotTickSpec>,
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
        // 独立 DOT：overlay 给出 dot_duration/dot_interval 时，克隆一份「每跳技能」用于逐跳重算
        let dot_tick = sc.and_then(|s| {
            let duration = s.dot_duration?;
            let interval = s.dot_interval.unwrap_or(0.0);
            if duration <= 0.0 {
                return None;
            }
            let base1 = s.dot_tick_base1.unwrap_or(1);
            let mut tick = resolved.clone();
            tick.dot_flag = 1;
            tick.dot_interval = interval;
            tick.dot_duration = duration;
            tick.dot_up = 0.0;
            tick.base_damage1 = base1;
            tick.base_damage2 = s.dot_tick_base2.unwrap_or(base1);
            tick.atk_xishu = s.dot_tick_xishu.unwrap_or(0.0);
            tick.watk_xishu = 0;
            Some(DotTickSpec {
                skill: tick,
                interval,
                duration,
            })
        });
        TimingSkill {
            skill: resolved,
            dot_tick,
            qifeng_gain: sc.and_then(|s| s.qifeng_gain).unwrap_or(0),
            spend_all_qifeng: sc.and_then(|s| s.spend_all_qifeng).unwrap_or(false),
            requires_instrument: sc.and_then(|s| s.requires_instrument.clone()),
            requires_qifeng: sc.and_then(|s| s.requires_qifeng.clone()),
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
        overrides: Option<jpcg_api::StepOverrideDTO>,
    },
    SwitchForm(String),
    /// 计算开始/结束标记（start = 开始，false = 结束）
    Flag {
        start: bool,
    },
}

impl TimelineAction {
    /// 按技能 id 施放（仅在 (skill_id, sub_id) 唯一时可用）
    pub fn cast(skill_id: u32, sub_id: u32) -> Self {
        TimelineAction::Cast {
            skill_id,
            sub_id,
            skill_name: None,
            overrides: None,
        }
    }

    /// 计算开始/结束标记
    pub fn flag(start: bool) -> Self {
        TimelineAction::Flag { start }
    }

    /// 按技能 id + 名称施放（用于同名 id 的多变体，如「入月·9层」）
    pub fn cast_named(skill_id: u32, sub_id: u32, skill_name: impl Into<String>) -> Self {
        TimelineAction::Cast {
            skill_id,
            sub_id,
            skill_name: Some(skill_name.into()),
            overrides: None,
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
    /// 初始器形 id（空 = 取 forms 里标记 initial 的，或第一条）
    pub initial_instrument: Option<String>,
    /// 初始曲风（空 = 不限曲风）
    pub initial_qifeng: String,
    /// 定时 buff 定义（技能 `buff_id` 引用）
    pub timed_buffs: Vec<TimedBuffDef>,
    /// 动作序列（按给定顺序，引擎计算最早可执行时刻）
    pub actions: Vec<TimelineAction>,
    /// 已激活的奇穴（id 或名称；技能 `jihuoqixue` 命中才可用）
    pub active_qixue: Vec<String>,
    /// 已激活奇穴的运行时钩子
    pub hooks: Vec<ActiveHook>,
    /// 模拟窗口（秒）
    pub window: f32,
    /// 击杀率蒙特卡洛样本数（0 = 不跑 MC，保持纯确定性）
    pub samples: u32,
    /// 击杀率随机种子（None = 每次随机；固定便于回归）
    pub seed: Option<u64>,
}

/// 单个伤害事件的会心分布（供击杀率蒙特卡洛掷点；确定性总伤用 `q`）。
#[derive(Debug, Clone, Copy)]
pub struct DamageDice {
    pub g: u32,
    pub h: u32,
    /// 会心率 0..=1
    pub rate: f32,
    /// 无质：伤害固定 = 期望 q，不掷会心
    pub fixed: bool,
    pub q: u32,
    pub lost_hp_factor: f32,
}

impl DamageDice {
    fn roll(self, rng: &mut impl Rng) -> u32 {
        if self.fixed {
            self.q
        } else if self.rate <= 0.0 || self.g == self.h {
            self.g
        } else if self.rate >= 1.0 {
            self.h
        } else if rng.random::<f64>() < self.rate as f64 {
            self.h
        } else {
            self.g
        }
    }
}

/// 事件类型
#[derive(Debug, Clone)]
pub enum TimelineEventKind {
    /// 开始施放（`hit_time` = 伤害结算时刻；引导/读条技能晚于起手）
    Cast {
        skill_name: String,
        damage: u32,
        hit_time: f32,
        /// 本次施放占用的 GCD 秒数（不占 GCD 为 0）
        gcd_secs: f32,
        /// 该 GCD 通道就绪时刻
        gcd_ready_at: f32,
        dice: DamageDice,
    },
    /// 引导/倒读条单段结算（`index` 从 1 开始；如徵 3 段）
    ChannelTick {
        skill_name: String,
        index: u32,
        damage: u32,
        dice: DamageDice,
    },
    /// dot 单跳结算（`index` 从 1 开始）
    DotTick {
        skill_name: String,
        index: u32,
        damage: u32,
        /// 是否被洞天/羽提前（UI 可标注）
        advanced: bool,
        dice: DamageDice,
    },
    /// 器形切换（琴 / 剑）
    FormSwitch { from: String, to: String },
    /// 曲风切换（0 秒状态事件）
    QifengSwitch { from: String, to: String },
    /// buff 生效
    BuffGain { buff_id: String, stacks: u8 },
}

/// 时间轴事件
#[derive(Debug, Clone)]
pub struct TimelineEvent {
    pub time: f32,
    pub kind: TimelineEventKind,
}

/// 单个动作的结算结果（供前端直接渲染，不再由前端推导）
#[derive(Debug, Clone, Default)]
pub struct TimelineStep {
    /// 该动作实际施放/切换的时刻；被跳过时为 None
    pub time: Option<f32>,
    pub gcd_secs: f32,
    pub gcd_ready_at: f32,
    /// 该动作是否成功执行（形态/曲风/调息/窗口等都算）
    pub legal: bool,
    /// 动作执行前的器形 / 曲风状态
    pub instrument: String,
    pub qifeng: String,
    /// 被跳过时的原因（成功为空）
    pub skipped_reason: String,
    /// 非法时缺少的器形 / 曲风要求
    pub required_instrument: Option<String>,
    pub required_qifeng: Option<String>,
    /// 本动作执行前发生的器形/曲风切换事件
    pub switches: Vec<TimelineStepSwitch>,
}

#[derive(Debug, Clone)]
pub struct TimelineStepSwitch {
    /// "form" / "qifeng"
    pub switch_kind: String,
    pub from: String,
    pub to: String,
    pub time: f32,
}

/// 时序模拟结果
#[derive(Debug, Clone, Default)]
pub struct TimelineResult {
    pub window: f32,
    pub events: Vec<TimelineEvent>,
    /// 每个动作一个步骤结果（与输入 actions 一一对应）
    pub steps: Vec<TimelineStep>,
    pub total_damage: f64,
    pub window_dps: f64,
    /// 窗口内未能施放的动作数（GCD/CD/形态不满足或超出窗口）
    pub skipped_actions: u32,
    /// 序列执行完后的器形 / 曲风
    pub final_instrument: String,
    pub final_qifeng: String,
    /// 窗口内击杀概率（0..1；samples=0 或未提供血量时为 0）
    pub final_kill_prob: f64,
    /// 累计击杀概率曲线：(动作序号 1..=n, 概率 0..1)
    pub kill_prob_curve: Vec<(usize, f64)>,
    /// 击杀率测量窗口起止（flag 解析后的时刻；None = 未提供该 flag）
    pub kill_start: Option<f32>,
    pub kill_end: Option<f32>,
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
    applied_at: f32,
    order: usize,
    key: SkillKey,
    skill_name: String,
    skill: Skilltype,
    /// 施加快照：玩家 / 心法 / buff / 系数
    player: PlayerConfig,
    xinfa: XinfaConfig,
    buff: BuffConfig,
    coeff: CoefficientConfig,
    /// 每跳时间点（绝对时间，施加时按当时加速率算好）
    tick_times: Vec<f32>,
    /// 该跳是否被「下一跳立刻生效」（洞天/羽）提前过
    advanced: Vec<bool>,
    /// 展示用的跳序号（刷新后新一轮从 1 重新编号；老实例已生效的跳保留原序号）
    tick_labels: Vec<u32>,
    /// 跳间隔（秒，施加时算好；提前后按它重新排后续跳）
    interval: f32,
    target_reduction: Option<u32>,
}

impl DotInstance {
    fn tick_dice(&self, hostile: &HostilepileConfig, index: usize) -> DamageDice {
        let mut hostile = hostile.clone();
        if let Some(value) = self.target_reduction {
            hostile.jianshang_bili = value;
        }
        let calc = JpcgConfig::new_with_config(
            &self.player,
            &hostile,
            &self.skill,
            &self.xinfa,
            &self.buff,
            &self.coeff,
        );
        let damage = calc.q_cal();
        let crit_rate = (calc.guo_huixin() + self.skill.huixin_up as f32 / 100.0).clamp(0.0, 1.0);
        let fixed = self.skill.has_critical_strike;
        if damage.dot_jumps.is_empty() {
            DamageDice {
                g: damage.g_damage,
                h: damage.h_damage,
                rate: crit_rate,
                fixed,
                q: damage.q_damage,
                lost_hp_factor: self.skill.lost_hp_zhenshishanghai,
            }
        } else {
            let base_q = damage.dot_jumps.first().copied().unwrap_or(1);
            let jq = damage.dot_jumps.get(index).copied().unwrap_or(0);
            let ratio = if base_q > 0 {
                jq as f32 / base_q as f32
            } else {
                1.0
            };
            DamageDice {
                g: (damage.g_damage as f32 * ratio) as u32,
                h: (damage.h_damage as f32 * ratio) as u32,
                rate: crit_rate,
                fixed,
                q: jq,
                lost_hp_factor: self.skill.lost_hp_zhenshishanghai,
            }
        }
    }
}

struct Sim<'a> {
    input: &'a TimelineInput,
    /// 曲风点数（0~9）
    qifeng_points: u8,
    events: Vec<TimelineEvent>,
    dots: Vec<DotInstance>,
    /// 各 GCD 通道就绪时刻
    lane_ready: HashMap<String, f32>,
    /// 各技能就绪时刻（按下标记录，避免同名 id 变体共享状态；有充能技能不使用）
    skill_ready: HashMap<usize, f32>,
    /// 充能（按下标记录）
    charges: HashMap<usize, ChargeState>,
    /// 当前器形 id（空 = 无器形概念）
    instrument: String,
    /// 当前曲风 id（空 = 无曲风概念）
    qifeng_id: String,
    /// 活跃定时 buff
    active_buffs: Vec<ActiveBuff>,
    /// DOT 提前事件（时间、动作顺序、目标技能键、可选来源实例）。
    accel_events: Vec<(f32, usize, Vec<SkillKey>, Option<usize>)>,
    dot_operation_order: usize,
    school_gcd_secs: f32,
}

impl<'a> Sim<'a> {
    fn new(input: &'a TimelineInput) -> Self {
        let school_gcd_secs = input.xinfa.gcd_base_secs();
        let instrument = input
            .initial_instrument
            .clone()
            .or_else(|| {
                input
                    .forms
                    .iter()
                    .find(|f| f.initial)
                    .or_else(|| input.forms.first())
                    .map(|f| f.id.clone())
            })
            .unwrap_or_default();
        Sim {
            input,
            qifeng_points: MAX_QIFENG,
            events: Vec::new(),
            dots: Vec::new(),
            lane_ready: HashMap::new(),
            skill_ready: HashMap::new(),
            charges: HashMap::new(),
            instrument,
            qifeng_id: input.initial_qifeng.clone(),
            active_buffs: Vec::new(),
            accel_events: Vec::new(),
            dot_operation_order: 0,
            school_gcd_secs,
        }
    }

    /// 施加 DOT；同一技能重复施放时「刷新」：保留已经生效的跳，追加新的跳（序号继续），
    /// 而不是丢弃旧实例（否则刷新前已经打出的跳会从日志里消失）。
    fn push_dot(
        &mut self,
        now: f32,
        sid: u32,
        sub: u32,
        name: String,
        skill: Skilltype,
        times: Vec<f32>,
        buff: BuffConfig,
        interval: f32,
        target_reduction: Option<u32>,
    ) {
        if times.is_empty() {
            return;
        }
        let ticks = times.len();
        let order = self.dot_operation_order;
        self.dot_operation_order += 1;
        self.dots.push(DotInstance {
            applied_at: now,
            order,
            key: (sid, sub),
            skill_name: name,
            skill,
            player: self.input.player.clone(),
            xinfa: self.input.xinfa.clone(),
            buff,
            coeff: self.input.coeff.clone(),
            tick_times: times,
            advanced: vec![false; ticks],
            tick_labels: (1..=ticks as u32).collect(),
            interval,
            target_reduction,
        });
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

    /// 技能在当前器形 / 曲风下是否可选
    fn skill_allowed(&self, sk: &TimingSkill) -> bool {
        if let Some(inst) = &sk.requires_instrument
            && !inst.trim().is_empty()
            && !self.input.forms.is_empty()
            && *inst != self.instrument
        {
            return false;
        }
        if let Some(q) = &sk.requires_qifeng
            && !q.trim().is_empty()
            && *q != self.qifeng_id
        {
            return false;
        }
        true
    }

    /// 通道默认 GCD 基础（具名器形通道 > 当前器形 > 心法/门派）
    fn lane_base(&self, lane: &str) -> f32 {
        self.form(lane)
            .or_else(|| self.form(&self.instrument))
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
            .filter(|(_, s)| {
                s.skill.skill_id == skill_id
                    && s.skill.sub_id == sub_id
                    && (s.skill.jihuoqixue.is_empty()
                        || self
                            .input
                            .active_qixue
                            .iter()
                            .any(|q| q == &s.skill.jihuoqixue))
            })
            .map(|(i, _)| i)
            .collect();
        match candidates.as_slice() {
            [] => None,
            [only] => Some(*only),
            _ => {
                let name = name?;
                let named: Vec<usize> = candidates
                    .iter()
                    .copied()
                    .filter(|&i| self.input.skills[i].skill.skill_name == name)
                    .collect();
                // 同 (id, sub) 的重复条目（如多等级变体）视为同一技能，取第一条
                let first = *named.first()?;
                let key = self.input.skills[first].key();
                if named.iter().all(|&i| self.input.skills[i].key() == key) {
                    Some(first)
                } else {
                    None
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
    fn cast(
        &mut self,
        sk_index: usize,
        start: f32,
        overrides: Option<&jpcg_api::StepOverrideDTO>,
    ) -> f32 {
        // 每次施放独立副本，不改变技能目录或后续同技能的冷却身份。
        let mut cast_skill = self.input.skills[sk_index].clone();
        let mut buff_cfg = self.buff_at(start);
        let mut hostile = self.input.hostile.clone();
        let target_reduction = overrides
            .and_then(|o| o.jianshang_bili_override)
            .map(|v| v.clamp(0.0, 100.0) as u32);
        if let Some(value) = target_reduction {
            hostile.jianshang_bili = value;
        }
        if let Some(o) = overrides {
            let st = &mut cast_skill.skill;
            if let Some(v) = o.base_damage_override {
                st.base_damage1 = v as u32;
                st.base_damage2 = v as u32;
            }
            if let Some(v) = o.atk_xishu_override {
                st.atk_xishu = v;
            }
            if let Some(v) = o.extra_atk_pct {
                buff_cfg.base_atk_pct += v;
            }
            // 共享修饰同时作用于本体与独立 DOT 的本次快照；
            // 基础伤害/攻击系数仅覆盖本体，保留 DOT 自己的每跳参数。
            for st in std::iter::once(&mut cast_skill.skill)
                .chain(cast_skill.dot_tick.as_mut().map(|spec| &mut spec.skill))
            {
                if let Some(v) = o.wushihuajin_override {
                    st.wushihuajin = v as u32;
                }
                if let Some(v) = o.extra_crit_pct {
                    st.huixin_up = v as u32;
                }
                if let Some(v) = o.extra_crit_dmg_pct {
                    st.huixiao_up = v as u32;
                }
                if let Some(v) = o.gain_override {
                    st.hit_up = v.max(0.0) as u32;
                }
            }
        }
        let sk = &cast_skill;
        let rate = self.jiasu_at(start);
        let cast_time = sk.skill.cast_time_with_haste(rate);
        let hit_time = start + cast_time;
        let calc = JpcgConfig::new_with_config(
            &self.input.player,
            &hostile,
            &sk.skill,
            &self.input.xinfa,
            &buff_cfg,
            &self.input.coeff,
        );
        let damage = calc.q_cal();
        let crit_rate = (calc.guo_huixin() + sk.skill.huixin_up as f32 / 100.0).clamp(0.0, 1.0);
        let fixed = sk.skill.has_critical_strike;

        // DOT 技能的 q_damage 已是全部跳数之和，伤害由 DotTick 事件按实际跳点计入；
        // 这里只记起手事件，避免与 dot 跳点重复累加。
        let base_direct = if damage.dot_jumps.is_empty() {
            damage.q_damage
        } else {
            0
        };

        // ---- 曲风与奇穴钩子（第 3 层）：按施放前曲风与本次消耗结算 ----
        let qifeng_before = self.qifeng_points;
        let key = sk.key();
        let spent = if sk.spend_all_qifeng {
            qifeng_before as f32
        } else {
            0.0
        };
        let mut flat_bonus = 0.0f32;
        let mut pct_bonus = 0.0f32;
        for h in &self.input.hooks {
            if !h.target(key) {
                continue;
            }
            match h.hook {
                QixueHook::FlatDamageOnCast {
                    damage,
                    per_qifeng_spent,
                } => {
                    flat_bonus += if per_qifeng_spent {
                        damage * spent
                    } else {
                        damage
                    };
                }
                QixueHook::DamagePctPerQifeng { pct } => {
                    pct_bonus += pct * qifeng_before as f32;
                }
                QixueHook::AccelerateDot { .. } => {}
            }
        }
        let direct_damage =
            ((base_direct as f32) * (1.0 + pct_bonus / 100.0) + flat_bonus).max(0.0) as u32;
        // 引导/倒读条：伤害按段结算（Cast 事件本身不带伤害）
        let channel_ticks = sk.skill.channel_ticks.max(1) as u32;
        let channel_cast = sk.skill.channel && channel_ticks > 1;
        let per_tick_damage = ((base_direct as f32) * (1.0 + pct_bonus / 100.0)).max(0.0) as u32;
        let g_final =
            ((damage.g_damage as f32) * (1.0 + pct_bonus / 100.0) + flat_bonus).max(0.0) as u32;
        let h_final =
            ((damage.h_damage as f32) * (1.0 + pct_bonus / 100.0) + flat_bonus).max(0.0) as u32;
        let g_tick = ((damage.g_damage as f32) * (1.0 + pct_bonus / 100.0)).max(0.0) as u32;
        let h_tick = ((damage.h_damage as f32) * (1.0 + pct_bonus / 100.0)).max(0.0) as u32;
        // 施放后更新曲风：先结清消耗，再加获得（上限 9）
        self.qifeng_points = if sk.spend_all_qifeng {
            sk.qifeng_gain.min(MAX_QIFENG)
        } else {
            (self.qifeng_points + sk.qifeng_gain).min(MAX_QIFENG)
        };
        let own_gcd = self.skill_gcd(sk, rate);
        let gcd_secs = if sk.skill.occupies_gcd() {
            own_gcd
        } else {
            0.0
        };
        let gcd_ready_at = start + gcd_secs;
        self.events.push(TimelineEvent {
            time: start,
            kind: TimelineEventKind::Cast {
                skill_name: sk.skill.skill_name.clone(),
                damage: if channel_cast { 0 } else { direct_damage },
                hit_time,
                gcd_secs,
                gcd_ready_at,
                dice: if channel_cast {
                    DamageDice {
                        g: 0,
                        h: 0,
                        rate: 0.0,
                        fixed: true,
                        q: 0,
                        lost_hp_factor: 0.0,
                    }
                } else {
                    DamageDice {
                        g: if damage.dot_jumps.is_empty() {
                            g_final
                        } else {
                            flat_bonus.max(0.0) as u32
                        },
                        h: if damage.dot_jumps.is_empty() {
                            h_final
                        } else {
                            flat_bonus.max(0.0) as u32
                        },
                        rate: crit_rate,
                        fixed,
                        q: direct_damage,
                        lost_hp_factor: if damage.dot_jumps.is_empty() {
                            sk.skill.lost_hp_zhenshishanghai
                        } else {
                            0.0
                        },
                    }
                },
            },
        });

        // ---- 引导分段：每段在其结算时刻输出（首段吃一次性固定加成） ----
        if channel_cast {
            let cast_secs = cast_time.max(0.0);
            for k in 1..=channel_ticks {
                let t = start + cast_secs * k as f32 / channel_ticks as f32;
                let bonus = if k == 1 {
                    flat_bonus.max(0.0) as u32
                } else {
                    0
                };
                let tick_g = if k == 1 {
                    g_tick + flat_bonus.max(0.0) as u32
                } else {
                    g_tick
                };
                let tick_h = if k == 1 {
                    h_tick + flat_bonus.max(0.0) as u32
                } else {
                    h_tick
                };
                let tick_q = per_tick_damage + bonus;
                self.events.push(TimelineEvent {
                    time: t,
                    kind: TimelineEventKind::ChannelTick {
                        skill_name: sk.skill.skill_name.clone(),
                        index: k,
                        damage: tick_q,
                        dice: DamageDice {
                            g: tick_g,
                            h: tick_h,
                            rate: crit_rate,
                            fixed,
                            q: tick_q,
                            lost_hp_factor: sk.skill.lost_hp_zhenshishanghai,
                        },
                    },
                });
            }
        }

        // ---- dot：进攻侧快照（伤害/加速/跳数），逐跳排在时间轴上 ----
        // 先保存独立快照，在时间顺序回放阶段截断旧实例的未来跳。
        let mut raw_dot_source = None;
        if !damage.dot_jumps.is_empty() {
            raw_dot_source = Some(self.dots.len());
            let ticks = damage.dot_jumps.len() as u32;
            let total = timing::quantize_secs(sk.skill.dot_duration, rate);
            let stamp = |i: u32| hit_time + total * i as f32 / ticks as f32;
            self.push_dot(
                hit_time,
                sk.skill.skill_id,
                sk.skill.sub_id,
                sk.skill.skill_name.clone(),
                sk.skill.clone(),
                (1..=ticks).map(stamp).collect(),
                buff_cfg.clone(),
                total / ticks as f32,
                target_reduction,
            );
        }

        // ---- 独立 DOT：逐跳按合成技能结算（与本体伤害分开） ----
        if let Some(spec) = &sk.dot_tick {
            let tick_cfg = JpcgConfig::new_with_config(
                &self.input.player,
                &hostile,
                &spec.skill,
                &self.input.xinfa,
                &buff_cfg,
                &self.input.coeff,
            );
            let jumps = tick_cfg.q_cal().dot_jumps;
            if !jumps.is_empty() {
                let ticks = jumps.len() as u32;
                let total = timing::quantize_secs(spec.duration, rate);
                let stamp = |i: u32| hit_time + total * i as f32 / ticks as f32;
                self.push_dot(
                    hit_time,
                    sk.skill.skill_id,
                    sk.skill.sub_id,
                    format!("{}·持续", sk.skill.skill_name),
                    spec.skill.clone(),
                    (1..=ticks).map(stamp).collect(),
                    buff_cfg.clone(),
                    total / ticks as f32,
                    target_reduction,
                );
            }
        }

        // ---- 羽类：施放时让目标 DOT 的「下 N 跳」立刻生效 ----
        if !sk.on_cast.advance_dot_targets.is_empty() {
            let times = sk.on_cast.advance_dot_ticks.max(1) as usize;
            for target in &sk.on_cast.advance_dot_targets {
                for _ in 0..times {
                    self.accel_events.push((
                        hit_time,
                        self.dot_operation_order,
                        vec![*target],
                        None,
                    ));
                    self.dot_operation_order += 1;
                }
            }
        }

        // ---- 洞天类奇穴：本技能的每次 DOT 跳都让目标 DOT「下一跳立刻生效」 ----
        if !damage.dot_jumps.is_empty() {
            let ticks = damage.dot_jumps.len() as u32;
            let total = timing::quantize_secs(sk.skill.dot_duration, rate);
            for h in &self.input.hooks {
                if !h.target(key) {
                    continue;
                }
                if let QixueHook::AccelerateDot { targets } = &h.hook {
                    for i in 1..=ticks {
                        let t = hit_time + total * i as f32 / ticks as f32;
                        self.accel_events.push((
                            t,
                            self.dot_operation_order,
                            targets.clone(),
                            raw_dot_source,
                        ));
                        self.dot_operation_order += 1;
                    }
                }
            }
        }

        // ---- 占用 GCD 通道（自身通道 + 副作用额外通道） ----
        // 自身通道取技能自己的 gcd_base / 固定 GCD 语义；副作用通道按通道默认值。
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
        // 旧数据可能用 charges=1 表示普通冷却；只有显式 recharge 才启用单充能。
        if max_charges > 1 || (sk.skill.charges == 1 && sk.skill.recharge > 0.0) {
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

        // ---- 器形切换 ----
        if let Some(to) = &sk.on_cast.switch_instrument
            && *to != self.instrument
            && self.form(to).is_some()
        {
            let from = self.instrument.clone();
            self.instrument = to.clone();
            self.events.push(TimelineEvent {
                time: start,
                kind: TimelineEventKind::FormSwitch {
                    from,
                    to: to.clone(),
                },
            });
        }
        // ---- 曲风切换（0 秒状态事件） ----
        if let Some(to) = &sk.on_cast.switch_qifeng
            && *to != self.qifeng_id
        {
            let from = self.qifeng_id.clone();
            self.qifeng_id = to.clone();
            self.events.push(TimelineEvent {
                time: start,
                kind: TimelineEventKind::QifengSwitch {
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
        let mut steps: Vec<TimelineStep> = Vec::new();
        let mut start_flag_time: Option<f32> = None;
        let mut end_flag_time: Option<f32> = None;
        // 待吸附的 flag：(step 下标, 是否开始)；时刻由下一个施放的起手决定
        let mut pending_flags: Vec<(usize, bool)> = Vec::new();

        for action in &self.input.actions {
            let ev_start = self.events.len();
            let instrument_before = self.instrument.clone();
            let qifeng_before = self.qifeng_id.clone();
            let mut legal = true;
            let mut skipped_reason = String::new();
            let mut required_instrument: Option<String> = None;
            let mut required_qifeng: Option<String> = None;

            match action {
                TimelineAction::SwitchForm(to) => {
                    if self.form(to).is_some() && *to != self.instrument {
                        let from = self.instrument.clone();
                        self.instrument = to.clone();
                        self.events.push(TimelineEvent {
                            time: now,
                            kind: TimelineEventKind::FormSwitch {
                                from,
                                to: to.clone(),
                            },
                        });
                    } else {
                        legal = false;
                        skipped_reason = "器形不存在或已经处于该器形".to_string();
                        skipped += 1;
                    }
                }
                TimelineAction::Flag { start } => {
                    // flag 不是技能：不占 GCD、不产生事件；时刻吸附到下一个施放的起手
                    pending_flags.push((steps.len(), *start));
                }
                TimelineAction::Cast {
                    skill_id,
                    sub_id,
                    skill_name,
                    overrides,
                } => {
                    if let Some(index) =
                        self.resolve_skill(*skill_id, *sub_id, skill_name.as_deref())
                    {
                        // 先按当前时刻结算充能，再判断能否施放
                        self.settle_charges(index, now);
                        if !self.skill_allowed(&self.input.skills[index]) {
                            legal = false;
                            skipped_reason = "当前器形/曲风不满足技能要求".to_string();
                            required_instrument =
                                self.input.skills[index].requires_instrument.clone();
                            required_qifeng = self.input.skills[index].requires_qifeng.clone();
                            skipped += 1;
                        } else {
                            let start = self.earliest_start(index).max(now);
                            if start > self.input.window {
                                legal = false;
                                skipped_reason = "超出模拟窗口".to_string();
                                skipped += 1;
                            } else {
                                for (flag_step_idx, is_start) in pending_flags.drain(..) {
                                    if is_start {
                                        if start_flag_time.is_none() {
                                            start_flag_time = Some(start);
                                        }
                                    } else if end_flag_time.is_none() {
                                        end_flag_time = Some(start);
                                    }
                                    steps[flag_step_idx].time = Some(start);
                                }
                                now = self.cast(index, start, overrides.as_ref());
                            }
                        }
                    } else {
                        // id 无法唯一确定具体变体（且未提供 skill_name）→ 跳过，避免静默取错条目
                        legal = false;
                        skipped_reason = "技能 id 无法唯一确定变体".to_string();
                        skipped += 1;
                    }
                }
            }

            // 从本动作新产生的事件里提取时间 / GCD / 切换，供前端直接渲染
            let mut step = TimelineStep {
                time: None,
                gcd_secs: 0.0,
                gcd_ready_at: 0.0,
                legal,
                instrument: instrument_before,
                qifeng: qifeng_before,
                skipped_reason,
                required_instrument,
                required_qifeng,
                switches: Vec::new(),
            };
            for e in &self.events[ev_start..] {
                match &e.kind {
                    TimelineEventKind::Cast {
                        hit_time,
                        gcd_secs,
                        gcd_ready_at,
                        ..
                    } => {
                        step.time = Some(*hit_time);
                        step.gcd_secs = *gcd_secs;
                        step.gcd_ready_at = *gcd_ready_at;
                    }
                    TimelineEventKind::FormSwitch { from, to } => {
                        step.switches.push(TimelineStepSwitch {
                            switch_kind: "form".to_string(),
                            from: from.clone(),
                            to: to.clone(),
                            time: e.time,
                        });
                    }
                    TimelineEventKind::QifengSwitch { from, to } => {
                        step.switches.push(TimelineStepSwitch {
                            switch_kind: "qifeng".to_string(),
                            from: from.clone(),
                            to: to.clone(),
                            time: e.time,
                        });
                    }
                    _ => {}
                }
            }
            steps.push(step);
        }

        // 只放了「计算开始」、没放「计算结束」时，测量窗口结束于最后一个动作（紧跟上一个技能），
        // 而不是固定模拟窗口；否则连招结束后 DOT 会继续跳完，击杀率被高估。
        if end_flag_time.is_none() && start_flag_time.is_some() && !steps.is_empty() {
            end_flag_time = Some(now.min(self.input.window));
        }

        // 没有后续施放时，末尾标记落在当前动作结束处，不隐式扩展到 300 秒。
        for (index, is_start) in pending_flags {
            let time = now.min(self.input.window);
            if is_start {
                start_flag_time = Some(time);
            } else {
                end_flag_time = Some(time);
            }
            steps[index].time = Some(time);
        }

        // 按时间及同刻动作顺序回放施加/刷新/提前，保留每次施加的独立快照。
        // kind=true 为施加，false 为提前；先发生的羽不能作用于未来的商。
        let mut operations: Vec<(f32, usize, bool, usize)> = self
            .dots
            .iter()
            .enumerate()
            .map(|(i, d)| (d.applied_at, d.order, true, i))
            .collect();
        operations.extend(
            self.accel_events
                .iter()
                .enumerate()
                .map(|(i, e)| (e.0, e.1, false, i)),
        );
        operations.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let mut active: HashMap<SkillKey, usize> = HashMap::new();
        let mut consumed: HashMap<usize, usize> = HashMap::new();
        for (time, _, apply, index) in operations {
            if apply {
                let key = self.dots[index].key;
                if let Some(old) = active.insert(key, index) {
                    let dot = &mut self.dots[old];
                    let keep = dot
                        .tick_times
                        .iter()
                        .take_while(|t| **t <= time + 1e-4)
                        .count();
                    dot.tick_times.truncate(keep);
                    dot.tick_labels.truncate(keep);
                    dot.advanced.truncate(keep);
                }
                continue;
            }
            let (_, _, targets, source) = &self.accel_events[index];
            // 被刷新掉的旧实例不再发出未来触发事件。
            if source.is_some_and(|i| active.get(&self.dots[i].key) != Some(&i)) {
                continue;
            }
            for (&(id, sub), &di) in &active {
                if !targets
                    .iter()
                    .any(|(target, variant)| *target == id && (*variant == 0 || *variant == sub))
                {
                    continue;
                }
                let dot = &mut self.dots[di];
                let mut start = *consumed.get(&di).unwrap_or(&0);
                while start < dot.tick_times.len() && dot.tick_times[start] <= time + 1e-4 {
                    start += 1;
                }
                if start >= dot.tick_times.len() {
                    continue;
                }
                for (offset, tick) in dot.tick_times.iter_mut().enumerate().skip(start) {
                    *tick = time + dot.interval * (offset - start) as f32;
                }
                dot.advanced[start] = true;
                consumed.insert(di, start + 1);
            }
        }

        // ---- dot 逐跳结算（每跳用当前 hostile 重算减伤） ----
        for dot in &self.dots {
            for (i, &t) in dot.tick_times.iter().enumerate() {
                if t > self.input.window {
                    continue;
                }
                let dice = dot.tick_dice(&self.input.hostile, i);
                self.events.push(TimelineEvent {
                    time: t,
                    kind: TimelineEventKind::DotTick {
                        skill_name: dot.skill_name.clone(),
                        index: *dot.tick_labels.get(i).unwrap_or(&(i as u32 + 1)),
                        damage: dice.q,
                        advanced: *dot.advanced.get(i).unwrap_or(&false),
                        dice,
                    },
                });
            }
        }

        self.events.sort_by(|a, b| a.time.total_cmp(&b.time));

        // 期望通道也逐次扣血结算追加真伤；实际命中时间决定顺序。
        if let Some(max_hp) = full_hp(&self.input.hostile) {
            let mut hp = if self.input.hostile.max_hp > 0 && self.input.hostile.current_hp > 0 {
                (self.input.hostile.current_hp as f64).min(max_hp)
            } else {
                max_hp
            };
            let mut hits: Vec<(usize, f32)> = self
                .events
                .iter()
                .enumerate()
                .filter_map(|(i, e)| {
                    let time = match &e.kind {
                        TimelineEventKind::Cast { hit_time, .. } => *hit_time,
                        TimelineEventKind::ChannelTick { .. }
                        | TimelineEventKind::DotTick { .. } => e.time,
                        _ => return None,
                    };
                    (time <= self.input.window).then_some((i, time))
                })
                .collect();
            hits.sort_by(|a, b| a.1.total_cmp(&b.1));
            for (i, _) in hits {
                match &mut self.events[i].kind {
                    TimelineEventKind::Cast { damage, dice, .. }
                    | TimelineEventKind::ChannelTick { damage, dice, .. }
                    | TimelineEventKind::DotTick { damage, dice, .. } => {
                        hp = (hp - *damage as f64).max(0.0);
                        let extra = jpcg_core::engine::atkcal::lost_hp_append_damage(
                            dice.lost_hp_factor,
                            max_hp,
                            hp,
                        ) as u32;
                        *damage = damage.saturating_add(extra);
                        hp = (hp - extra as f64).max(0.0);
                    }
                    _ => {}
                }
            }
        }
        let mut total = 0f64;
        // 区间 DPS 的分母是「实际伤害跨度」，不是固定模拟窗口（否则 10 秒的连招会被 300 秒稀释）。
        let mut last_damage_time = 0.0f32;
        for e in &self.events {
            match &e.kind {
                // 读条/引导的伤害在命中时刻结算：只有命中落在窗口内才计入总伤
                TimelineEventKind::Cast {
                    damage, hit_time, ..
                } => {
                    if *hit_time <= self.input.window {
                        total += *damage as f64;
                        if *damage > 0 {
                            last_damage_time = last_damage_time.max(*hit_time);
                        }
                    }
                }
                TimelineEventKind::ChannelTick { damage, .. } => {
                    if e.time <= self.input.window {
                        total += *damage as f64;
                        if *damage > 0 {
                            last_damage_time = last_damage_time.max(e.time);
                        }
                    }
                }
                TimelineEventKind::DotTick { damage, .. } => {
                    total += *damage as f64;
                    if *damage > 0 {
                        last_damage_time = last_damage_time.max(e.time);
                    }
                }
                _ => {}
            }
        }
        // 至少 1 秒，避免单次瞬发除零；正常连招取最后一个伤害事件时刻。
        let span = last_damage_time.max(1.0);
        let window_dps = total / span as f64;

        // ---- 击杀率蒙特卡洛：在 [start,end] 内独立结算（开始时刻满血重置） ----
        let kill_start = start_flag_time.unwrap_or(0.0);
        let kill_end = end_flag_time.unwrap_or(self.input.window);
        let mut final_kill_prob = 0.0f64;
        let mut kill_prob_curve: Vec<(usize, f64)> = Vec::new();
        if self.input.samples > 0 && !steps.is_empty() {
            if let Some(hp0) = full_hp(&self.input.hostile) {
                let mut damage_events: Vec<(f32, DamageDice)> = self
                    .events
                    .iter()
                    .filter_map(|e| match &e.kind {
                        TimelineEventKind::Cast { hit_time, dice, .. } => Some((*hit_time, *dice)),
                        TimelineEventKind::ChannelTick { dice, .. } => Some((e.time, *dice)),
                        TimelineEventKind::DotTick { dice, .. } => Some((e.time, *dice)),
                        _ => None,
                    })
                    .filter(|(t, _)| {
                        kill_end > kill_start && *t >= kill_start - 1e-4 && *t <= kill_end + 1e-4
                    })
                    .collect();
                damage_events.sort_by(|a, b| a.0.total_cmp(&b.0));
                let n = self.input.samples.max(1) as usize;
                let mut rng = match self.input.seed {
                    Some(seed) => StdRng::seed_from_u64(seed),
                    None => StdRng::from_os_rng(),
                };
                let mut kills_at = vec![0u64; steps.len()];
                for _ in 0..n {
                    let mut cur = hp0;
                    let mut killed: Option<f32> = None;
                    for (t, dice) in &damage_events {
                        cur = (cur - dice.roll(&mut rng) as f64).max(0.0);
                        let extra = jpcg_core::engine::atkcal::lost_hp_append_damage(
                            dice.lost_hp_factor,
                            hp0,
                            cur,
                        );
                        cur = (cur - extra).max(0.0);
                        if cur <= 0.0 {
                            killed = Some(*t);
                            break;
                        }
                    }
                    if let Some(kt) = killed {
                        let idx = steps
                            .iter()
                            .position(|s| s.time.is_some_and(|tm| tm >= kt))
                            .unwrap_or(steps.len() - 1);
                        kills_at[idx] += 1;
                    }
                }
                let mut acc = 0u64;
                for (i, _) in steps.iter().enumerate() {
                    acc += kills_at[i];
                    kill_prob_curve.push((i + 1, acc as f64 / n as f64));
                }
                final_kill_prob = kill_prob_curve.last().map(|(_, p)| *p).unwrap_or(0.0);
            }
        }

        let final_instrument = self.instrument.clone();
        let final_qifeng = self.qifeng_id.clone();
        TimelineResult {
            events: self.events,
            steps,
            final_instrument,
            final_qifeng,
            window: self.input.window,
            total_damage: total,
            window_dps,
            skipped_actions: skipped,
            final_kill_prob,
            kill_prob_curve,
            kill_start: start_flag_time,
            kill_end: end_flag_time,
        }
    }
}

/// 运行时序模拟
/// 战斗内曲风初始值：非战斗获取不建模，进场按满点 9 处理。
pub const MAX_QIFENG: u8 = 9;

/// 击杀率测量起始满血量：max_hp>0 → max；否则 target_hp → target_hp；都 0 → 无击杀语义。
fn full_hp(hostile: &HostilepileConfig) -> Option<f64> {
    if hostile.max_hp > 0 {
        Some(hostile.max_hp as f64)
    } else if hostile.target_hp > 0 {
        Some(hostile.target_hp as f64)
    } else {
        None
    }
}

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
            zhuanhua_gongji: None,
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
            qifeng_gain: 0,
            spend_all_qifeng: false,
            requires_instrument: None,
            requires_qifeng: None,
            gcd_lane: lane.into(),
            on_cast: OnCast::default(),
            dot_tick: None,
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
            initial_instrument: None,
            initial_qifeng: String::new(),
            timed_buffs: Vec::new(),
            active_qixue: Vec::new(),
            hooks: Vec::new(),
            actions,
            window,
            samples: 0,
            seed: None,
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
        let casts: Vec<_> = result
            .events
            .iter()
            .filter_map(|e| match &e.kind {
                TimelineEventKind::Cast {
                    gcd_secs,
                    gcd_ready_at,
                    ..
                } => Some((*gcd_secs, *gcd_ready_at)),
                _ => None,
            })
            .collect();
        assert_eq!(casts, vec![(1.5, 1.5), (1.5, 3.0)]);
    }

    #[test]
    fn independent_gcd_lanes_do_not_block_each_other() {
        // 双形态：两条通道各自 1.5s / 1.0s，交替施放互不阻塞
        let a = timing(skill(1, 0, "单持技能"), "single");
        let b = TimingSkill {
            requires_instrument: Some("dual".into()),
            requires_qifeng: None,
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
                switch_instrument: Some("dual".into()),
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
                switch_instrument: Some("dual".into()),
                switch_qifeng: None,
                gcd_lanes: vec!["dual".into()],
                lock_skills: vec![(2, 0)],
                advance_dot_targets: Vec::new(),
                advance_dot_ticks: 0,
            },
            ..timing(skill(1, 0, "断云势"), "single")
        };
        let gufeng = TimingSkill {
            requires_instrument: Some("dual".into()),
            requires_qifeng: None,
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
requires_instrument = "single"
gcd_lane = "single"
on_cast = { switch_instrument = "dual", gcd_lanes = ["dual"], lock_skills = [[2, 0]] }

[[skill_combat]]
skill_id = 2
sub_id = 0
requires_instrument = "dual"
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
            requires_instrument: Some("a".into()),
            requires_qifeng: None,
            ..timing(skill(1, 0, "A"), "a")
        };
        let b = TimingSkill {
            requires_instrument: Some("b".into()),
            requires_qifeng: None,
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
        sim.cast(0, 0.0, None);
        sim.cast(0, 1.5, None);
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
