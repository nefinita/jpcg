//! 装备原始点数 → 玩家面板。独立整数实现；规则/证据边界见 docs/attribute-engine.md。
//! 没有规则时维持旧版语义；不根据心法名称猜系数，也不把示例装备当游戏数据。
use crate::model::{DamageType, MainStat, School, Stats};
use serde::{Deserialize, Serialize};

pub const MOWEN_130: &str = "jx3-130-mowen-20260814";
pub const SOURCE: &str =
    "https://github.com/IcyTide/Generator/tree/21f472e211581ac1c1a7fa5ea2ff9199df6151cb";

/// 无战斗增益的派生面板；攻击区分可吃基础攻击增益的部分和心法额外转换。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributePanel {
    pub rule_id: String,
    pub source: String,
    pub verified_in_game: bool,
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
}

fn sum(values: &[u32]) -> Result<u32, String> {
    values.iter().try_fold(0u32, |a, b| {
        a.checked_add(*b).ok_or("派生属性溢出 u32".into())
    })
}

/// 每个转换阶段独立向下取整。u64 中间量防止先乘后除溢出，最终越界报错。
fn scale(value: u32, numerator: u32, denominator: u32) -> Result<u32, String> {
    u32::try_from(u64::from(value) * u64::from(numerator) / u64::from(denominator))
        .map_err(|_| "派生属性溢出 u32".into())
}

/// 第一条明确支持的历史规则：130 级莫问阴性攻击/会心/破防、体质/气血。
/// stats 是转换前的平铺值；zhushuxing 为主属性增量别名，与 gengu 相加一次。
/// 不自动添加等级裸装值。调用者必须提供尚未经过基础属性/心法转换的平铺值。
pub fn convert(
    stats: &Stats,
    profile: &School,
    value_set: &str,
    rule_id: &str,
) -> Result<AttributePanel, String> {
    if rule_id != MOWEN_130 {
        return Err(format!("未知属性规则: {rule_id}"));
    }
    if value_set != "live-130"
        || profile.id != "mowen"
        || profile.zhushuxing != MainStat::Gengu
        || profile.gongji_type != DamageType::Neigong
        || profile.pofang_type != DamageType::Neigong
    {
        return Err("jx3-130-mowen-20260814 仅支持 live-130 的莫问阴性属性".into());
    }
    let gengu = sum(&[stats.gengu, stats.zhushuxing])?;
    let gongji_base = sum(&[stats.gongji, scale(stats.yuanqi, 181, 1000)?])?;
    let gongji_extra = scale(gengu, 2007, 1024)?;
    let huixin = sum(&[stats.huixin, scale(gengu, 9, 10)?, scale(gengu, 410, 1024)?])?;
    let pofang = sum(&[stats.pofang, scale(stats.yuanqi, 3, 10)?])?;
    let life_base = sum(&[stats.qixue, scale(stats.tizhi, 10, 1)?])?;
    let qixue = scale(life_base, 1331, 1024)?;
    Ok(AttributePanel {
        rule_id: rule_id.into(),
        source: SOURCE.into(),
        verified_in_game: false,
        tizhi: stats.tizhi,
        liliang: stats.liliang,
        shenfa: stats.shenfa,
        gengu,
        yuanqi: stats.yuanqi,
        gongji_base,
        gongji_extra,
        gongji: sum(&[gongji_base, gongji_extra])?,
        huixin,
        pofang,
        qixue,
    })
}

#[cfg(feature = "engine")]
pub(crate) fn validate_legacy(stats: &Stats) -> Result<(), String> {
    if [
        stats.tizhi,
        stats.liliang,
        stats.shenfa,
        stats.gengu,
        stats.yuanqi,
    ]
    .iter()
    .any(|v| *v != 0)
    {
        return Err("体质及具名基础属性需要显式 attribute_rules，旧模式不能忽略这些输入".into());
    }
    Ok(())
}
