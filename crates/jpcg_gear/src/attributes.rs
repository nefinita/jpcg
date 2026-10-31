//! 装备原始点数 → 玩家面板。独立整数实现；规则/证据边界见 docs/attribute-engine.md。
//! 没有规则时维持旧版语义；不根据心法名称猜系数，也不把示例装备当游戏数据。
use crate::model::MainStat;
#[cfg(feature = "engine")]
use crate::model::{DamageType, School, Stats};
use serde::{Deserialize, Serialize};

pub const PROFESSION_TOML: &str = "profession-toml";
#[cfg(feature = "engine")]
use jpcg_api::{AttributeDamageKind, AttributeRules, AttributeTarget, BasicAttribute};

/// 无战斗增益的派生面板；攻击区分可吃基础攻击增益的部分和心法额外转换。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributePanel {
    pub rule_id: String,
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub value_set: String,
    pub source: String,
    pub verified_in_game: bool,
    #[serde(default)]
    pub damage_kind: String,
    pub tizhi: u32,
    pub liliang: u32,
    pub shenfa: u32,
    pub gengu: u32,
    pub yuanqi: u32,
    pub gongji_base: u32,
    pub gongji_extra: u32,
    pub gongji: u32,
    pub huixin: u32,
    pub pofang: u32,
    pub qixue: u32,
    #[serde(default)]
    pub qixue_base: u32,
    #[serde(default)]
    pub qixue_additional: u32,
}

#[cfg(feature = "engine")]
fn sum(values: &[u32]) -> Result<u32, String> {
    values.iter().try_fold(0u32, |a, b| {
        a.checked_add(*b).ok_or("派生属性溢出 u32".into())
    })
}

/// 每个转换阶段独立向下取整。u64 中间量防止先乘后除溢出，最终越界报错。
#[cfg(feature = "engine")]
fn scale(value: u32, numerator: u32, denominator: u32) -> Result<u32, String> {
    if denominator == 0 {
        return Err("换算分母不能为0".into());
    }
    u32::try_from(u64::from(value) * u64::from(numerator) / u64::from(denominator))
        .map_err(|_| "派生属性溢出 u32".into())
}

/// 每次请求读取选中职业同一TOML的规则；无隐式心法系数或等级回退。
#[cfg(feature = "engine")]
pub(crate) fn load_rules(
    loadout: &crate::model::Loadout,
) -> Result<Option<AttributeRules>, String> {
    match loadout.attribute_rules.as_deref() {
        None => Ok(None),
        Some(PROFESSION_TOML) => {
            let rules = jpcg_core::skill_editor::load_attribute_rules(&loadout.school)?;
            let value = jpcg_core::host::values::resolve_value_set(Some(&loadout.value_set));
            if rules.value_set != loadout.value_set
                || value.id != loadout.value_set
                || !value.available
                || value.source != "data"
                || value.level != rules.level
            {
                return Err(
                    "职业换算表的value_set/level必须与可用的所选数值集一致，禁止回退或混用50/130级"
                        .into(),
                );
            }
            Ok(Some(rules))
        }
        Some(id) => Err(format!(
            "不再支持内置历史规则 {id}；请将已验证规则填写到职业TOML并选择 profession-toml"
        )),
    }
}

/// 通用分阶段整数换算。所有转换系数由职业TOML提供，等级不决定隐式常数。
#[cfg(feature = "engine")]
pub fn convert(
    stats: &Stats,
    profile: &School,
    value_set: &str,
    rules: &AttributeRules,
) -> Result<AttributePanel, String> {
    let main = match rules.main_stat {
        BasicAttribute::Liliang => MainStat::Liliang,
        BasicAttribute::Shenfa => MainStat::Shenfa,
        BasicAttribute::Gengu => MainStat::Gengu,
        BasicAttribute::Yuanqi => MainStat::Yuanqi,
        BasicAttribute::Tizhi => return Err("当前配装主属性不能为体质".into()),
    };
    let kind = match rules.damage_kind {
        AttributeDamageKind::Physical => "physical",
        AttributeDamageKind::Solar => "solar",
        AttributeDamageKind::Lunar => "lunar",
        AttributeDamageKind::Neutral => "neutral",
        AttributeDamageKind::Poison => "poison",
    };
    let damage = if kind == "physical" {
        DamageType::Waigong
    } else {
        DamageType::Neigong
    };
    if rules.id.trim().is_empty()
        || rules.source.trim().is_empty()
        || rules.level == 0
        || rules.value_set != value_set
        || profile.zhushuxing != main
        || profile.gongji_type != damage
        || profile.pofang_type != damage
        || rules.qixue_multiplier.denominator == 0
    {
        return Err("职业换算表缺少来源/版本，或主属性/通道/数值集/分母不匹配".into());
    }
    let mut basics = stats.clone();
    let major = match main {
        MainStat::Liliang => &mut basics.liliang,
        MainStat::Shenfa => &mut basics.shenfa,
        MainStat::Gengu => &mut basics.gengu,
        MainStat::Yuanqi => &mut basics.yuanqi,
    };
    *major = sum(&[*major, stats.zhushuxing])?;
    let mut gongji_base = stats.gongji;
    let mut gongji_extra = 0;
    let mut huixin = stats.huixin;
    let mut pofang = stats.pofang;
    let mut life_base = stats.qixue;
    let mut qixue_additional = stats.qixue_additional;
    for term in &rules.conversions {
        let value = match term.from {
            BasicAttribute::Tizhi => basics.tizhi,
            BasicAttribute::Liliang => basics.liliang,
            BasicAttribute::Shenfa => basics.shenfa,
            BasicAttribute::Gengu => basics.gengu,
            BasicAttribute::Yuanqi => basics.yuanqi,
        };
        let target = match term.to {
            AttributeTarget::GongjiBase => &mut gongji_base,
            AttributeTarget::GongjiExtra => &mut gongji_extra,
            AttributeTarget::Huixin => &mut huixin,
            AttributeTarget::Pofang => &mut pofang,
            AttributeTarget::QixueBase => &mut life_base,
            AttributeTarget::QixueAdditional => &mut qixue_additional,
        };
        *target = sum(&[*target, scale(value, term.numerator, term.denominator)?])?;
    }
    let qixue_base = scale(
        life_base,
        rules.qixue_multiplier.numerator,
        rules.qixue_multiplier.denominator,
    )?;
    Ok(AttributePanel {
        rule_id: rules.id.clone(),
        level: rules.level,
        value_set: value_set.into(),
        source: rules.source.clone(),
        verified_in_game: false,
        damage_kind: kind.into(),
        tizhi: basics.tizhi,
        liliang: basics.liliang,
        shenfa: basics.shenfa,
        gengu: basics.gengu,
        yuanqi: basics.yuanqi,
        gongji_base,
        gongji_extra,
        gongji: sum(&[gongji_base, gongji_extra])?,
        huixin,
        pofang,
        qixue: sum(&[qixue_base, qixue_additional])?,
        qixue_base,
        qixue_additional,
    })
}

#[cfg(feature = "engine")]
pub(crate) fn validate_legacy(stats: &Stats) -> Result<(), String> {
    if [
        stats.qixue_additional,
        stats.tizhi,
        stats.liliang,
        stats.shenfa,
        stats.gengu,
        stats.yuanqi,
    ]
    .iter()
    .any(|v| *v != 0)
    {
        return Err(
            "体质、具名基础属性及额外气血上限需要显式 attribute_rules，旧模式不能忽略这些输入"
                .into(),
        );
    }
    Ok(())
}

impl AttributePanel {
    pub fn main_value(&self, main: MainStat) -> u32 {
        match main {
            MainStat::Liliang => self.liliang,
            MainStat::Shenfa => self.shenfa,
            MainStat::Gengu => self.gengu,
            MainStat::Yuanqi => self.yuanqi,
        }
    }
}
