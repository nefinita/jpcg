use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

macro_rules! gear_enum {
    ($name:ident { $($value:ident),+ }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($value),+ }
    }
}
gear_enum!(Source {
    Mingjian,
    Weiwang,
    Baowu
});
gear_enum!(Tier { Base, Upgraded });
gear_enum!(Slot {
    Tou,
    Yi,
    Yao,
    Shou,
    Ku,
    Xie,
    Wuqi,
    Anqi,
    Xianglian,
    Yaozhui,
    Jie1,
    Jie2
});
gear_enum!(Green {
    Pofang,
    Huixin,
    Jiasu,
    Yujin
});
gear_enum!(MainStat {
    Liliang,
    Shenfa,
    Gengu,
    Yuanqi
});
gear_enum!(DamageType { Neigong, Waigong });
gear_enum!(SetKind { Armor, Accessory });

impl Slot {
    pub const ALL: [Self; 12] = [
        Self::Tou,
        Self::Yi,
        Self::Yao,
        Self::Shou,
        Self::Ku,
        Self::Xie,
        Self::Wuqi,
        Self::Anqi,
        Self::Xianglian,
        Self::Yaozhui,
        Self::Jie1,
        Self::Jie2,
    ];
    pub fn accessory(self) -> bool {
        matches!(
            self,
            Self::Xianglian | Self::Yaozhui | Self::Jie1 | Self::Jie2
        )
    }
    pub fn armor(self) -> bool {
        matches!(
            self,
            Self::Tou | Self::Yi | Self::Yao | Self::Shou | Self::Xie
        )
    }
}

/// 全为显式点数；不从绿字方向、装分或其他部位推导。省略项为零，拒绝拼错字段。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Stats {
    pub zhushuxing: u32,
    pub gongji: u32,
    pub qixue: u32,
    pub huixin: u32,
    pub huixiao: u32,
    pub pofang: u32,
    pub jiasu: u32,
    pub yujin: u32,
    pub huajin: u32,
    pub wuqi_shanghai: u32,
}
impl Stats {
    pub fn add(&mut self, rhs: &Self) -> Result<(), String> {
        macro_rules! add { ($($f:ident),+) => { $(self.$f = self.$f.checked_add(rhs.$f)
            .ok_or_else(|| format!("属性溢出: {}", stringify!($f)))?;)+ } }
        add!(
            zhushuxing,
            gongji,
            qixue,
            huixin,
            huixiao,
            pofang,
            jiasu,
            yujin,
            huajin,
            wuqi_shanghai
        );
        Ok(())
    }
    pub fn value(&self, name: &str) -> Result<u32, String> {
        Ok(match name {
            "zhushuxing" => self.zhushuxing,
            "gongji" => self.gongji,
            "qixue" => self.qixue,
            "huixin" => self.huixin,
            "huixiao" => self.huixiao,
            "pofang" => self.pofang,
            "jiasu" => self.jiasu,
            "yujin" => self.yujin,
            "huajin" => self.huajin,
            "wuqi_shanghai" => self.wuqi_shanghai,
            _ => return Err(format!("未知属性: {name}")),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Version {
    pub schema: u32,
    pub season: u32,
    pub revision: String,
    pub value_set: String,
    /// 必须明确声明样例；正式数据需有可追溯来源。
    pub sample: bool,
    pub provenance: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct School {
    pub id: String,
    pub zhushuxing: MainStat,
    pub gongji_type: DamageType,
    pub pofang_type: DamageType,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub slot: Slot,
    pub source: Source,
    pub tier: Tier,
    /// 非饰品固定为 0；饰品为 1 或 2，独立于绿字。
    #[serde(default)]
    pub variant: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Template {
    pub school: String,
    pub choice: Choice,
    pub green: Green,
    pub stats: Stats,
    pub set: Option<String>,
    pub zhuangfen: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Override {
    pub school: String,
    pub choice: Choice,
    pub green: Option<Green>,
    /// 完整替换属性而非增量，御劲高攻击按此显式录入。
    pub stats: Option<Stats>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cooldown {
    pub skill_id: u32,
    pub seconds: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetTier {
    pub tier: Tier,
    pub bonus2: Stats,
    pub bonus3: Option<Stats>,
    #[serde(default)]
    pub cooldown4: Vec<Cooldown>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GearSet {
    pub id: String,
    pub source: Source,
    pub kind: SetKind,
    pub tiers: Vec<SetTier>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub version: Version,
    pub schools: Vec<School>,
    pub templates: Vec<Template>,
    pub sets: Vec<GearSet>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Loadout {
    pub school: String,
    pub revision: String,
    pub value_set: String,
    /// 裸装基础点数，不能传入已经含装备的总面板。
    #[serde(default)]
    pub base: Stats,
    pub choices: Vec<Choice>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveBonus {
    pub set: String,
    pub pieces: usize,
    pub threshold: usize,
    pub tier: Tier,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composed {
    pub loadout: Loadout,
    pub profile: School,
    pub stats: Stats,
    pub active_bonuses: Vec<ActiveBonus>,
    pub cooldowns: Vec<Cooldown>,
    pub warnings: Vec<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    pub min: Option<u32>,
    pub max: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Constraints {
    pub sources: Vec<Source>,
    pub allow_upgraded: bool,
    pub locked: Vec<Choice>,
    pub banned: Vec<Choice>,
    pub required_sources: BTreeMap<Slot, Source>,
    pub banned_sources: BTreeMap<Slot, Vec<Source>>,
    pub stats: BTreeMap<String, Bounds>,
    pub allow_cross_source_2plus2: bool,
}
impl Default for Constraints {
    fn default() -> Self {
        Self {
            sources: vec![Source::Mingjian, Source::Weiwang, Source::Baowu],
            allow_upgraded: true,
            locked: vec![],
            banned: vec![],
            required_sources: BTreeMap::new(),
            banned_sources: BTreeMap::new(),
            stats: BTreeMap::new(),
            allow_cross_source_2plus2: true,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Objective {
    /// 指定技能 q 的非负加权和，不把技能池隐式当作一套连招。
    Expected { weights: BTreeMap<String, f64> },
    KillProbability {
        steps: Vec<jpcg_api::ComboStepDTO>,
        samples: u32,
        seed: u64,
        #[serde(default)]
        allow_unapplied_cd: bool,
    },
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptimizeRequest {
    pub loadout: Loadout,
    pub config: jpcg_api::CalculateRequest,
    pub objective: Objective,
    #[serde(default)]
    pub constraints: Constraints,
    /// 枚举叶子上限，包含违反硬约束的组合；不虚报全局最优。
    pub max_combinations: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub composed: Composed,
    pub score: f64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Progress {
    pub visited: u64,
    pub evaluated: u64,
    pub combinations: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizeResult {
    pub best: Option<Candidate>,
    pub baseline: Candidate,
    pub delta: Option<f64>,
    /// 目标值 vs 气血/御劲/化劲三维生存偏序，无杜撰的生存权重。
    pub pareto: Vec<Candidate>,
    pub progress: Progress,
    pub complete: bool,
    pub stop_reason: String,
    pub warnings: Vec<String>,
}

/// 桌面合成响应：保留完整配装元数据，伤害属性转换复用 player() 唯一实现。
#[derive(Debug, Serialize, Deserialize)]
pub struct Composition {
    pub composed: Composed,
    pub player: jpcg_api::PlayerConfigDTO,
}
