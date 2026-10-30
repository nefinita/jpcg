use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BuffConfig {
    pub base_atk_pct: f32,
    pub huixin_pct: f32,
    pub huixiao_pct: f32,
    pub pofang_pct: f32,
    pub wushi_fangyu_pct: f32,
    pub shanghai_pct: f32,
    pub mode_is_point: bool,
}

impl Default for BuffConfig {
    fn default() -> Self {
        Self {
            base_atk_pct: 0.0,
            huixin_pct: 0.0,
            huixiao_pct: 0.0,
            pofang_pct: 0.0,
            wushi_fangyu_pct: 0.0,
            shanghai_pct: 0.0,
            mode_is_point: false,
        }
    }
}

// ============================================================================
// #92 时序引擎：带持续时间的 buff 定义（buff 列表 / 只提供 buff 的技能）
// ============================================================================

/// 单个 buff 的可叠加效果条目（百分比；`mode_is_point` 全局语义沿用 BuffConfig）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuffEffect {
    /// 基础攻击加成（%）
    BaseAtkPct(f32),
    /// 会心率加成（%）
    HuixinPct(f32),
    /// 会心效果加成（%）
    HuixiaoPct(f32),
    /// 破防加成（%）
    PofangPct(f32),
    /// 无视防御（%）
    WushiFangyuPct(f32),
    /// 伤害加成（%）
    ShanghaiPct(f32),
    /// 加速率加成（百分比，如 2.0 = +2% 加速率；时序引擎按加速率叠加）
    JiasuPct(f32),
}

impl BuffEffect {
    /// 把本条效果按层数累加进运行期 BuffConfig（加速单独走 `jiasu_bonus`）
    pub fn apply(&self, cfg: &mut BuffConfig, stacks: f32) {
        match *self {
            BuffEffect::BaseAtkPct(v) => cfg.base_atk_pct += v * stacks,
            BuffEffect::HuixinPct(v) => cfg.huixin_pct += v * stacks,
            BuffEffect::HuixiaoPct(v) => cfg.huixiao_pct += v * stacks,
            BuffEffect::PofangPct(v) => cfg.pofang_pct += v * stacks,
            BuffEffect::WushiFangyuPct(v) => cfg.wushi_fangyu_pct += v * stacks,
            BuffEffect::ShanghaiPct(v) => cfg.shanghai_pct += v * stacks,
            BuffEffect::JiasuPct(_) => {} // 由调用方单独汇总
        }
    }

    /// 加速率加成（百分比；非加速效果返回 0）
    pub fn jiasu_pct(&self) -> f32 {
        match *self {
            BuffEffect::JiasuPct(v) => v,
            _ => 0.0,
        }
    }
}

/// 带持续时间的 buff 定义（数据在 TOML 中手工填写 / 由技能 buff_id 引用）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TimedBuffDef {
    /// 唯一 id（技能 `buff_id` 引用）
    pub id: String,
    /// 显示名
    pub name: String,
    /// 持续时间（秒；0 = 常驻直至手动移除）
    pub duration: f32,
    /// 最大层数（0/1 = 不可叠加）
    pub max_stacks: u8,
    /// 重复施加时是否刷新持续时间（false = 独立实例，各自计时）
    pub refresh: bool,
    /// 效果列表
    pub effects: Vec<BuffEffect>,
    /// 本 buff 提供的加速是否为「可突破 25% 上限」的来源
    pub ignore_cap: bool,
}

impl TimedBuffDef {
    /// 最大层数（0/1 视作 1）
    pub fn max_stacks(&self) -> u8 {
        if self.max_stacks > 1 {
            self.max_stacks
        } else {
            1
        }
    }

    /// 按层数计算加速率加成（小数）
    pub fn jiasu_rate_bonus(&self, stacks: u8) -> f32 {
        let stacks = stacks.min(self.max_stacks());
        let pct: f32 = self.effects.iter().map(|e| e.jiasu_pct()).sum();
        pct * stacks as f32 / 100.0
    }

    /// 加速率加成是否可突破 25% 上限
    pub fn jiasu_ignore_cap(&self) -> bool {
        self.ignore_cap && self.effects.iter().any(|e| e.jiasu_pct() != 0.0)
    }
}
