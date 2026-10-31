//! 装备原始点数 → 玩家面板。独立整数实现；规则/证据边界见 docs/attribute-engine.md。
//! 没有规则时维持旧版语义；不根据心法名称猜系数，也不把示例装备当游戏数据。
use crate::model::MainStat;
#[cfg(feature = "engine")]
use crate::model::{DamageType, School, Stats};
use serde::{Deserialize, Serialize};

pub const MOWEN_130: &str = "jx3-130-mowen-20260814";
pub const AOXUE_130: &str = "jx3-130-aoxue-20260814";
pub const FENSHAN_130: &str = "jx3-130-fenshan-20260814";
pub const HUAJIAN_130: &str = "jx3-130-huajian-20260814";
pub const SOURCE: &str =
    "https://github.com/IcyTide/Generator/tree/21f472e211581ac1c1a7fa5ea2ff9199df6151cb";

/// 无战斗增益的派生面板；攻击区分可吃基础攻击增益的部分和心法额外转换。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributePanel {
    pub rule_id: String,
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
    u32::try_from(u64::from(value) * u64::from(numerator) / u64::from(denominator))
        .map_err(|_| "派生属性溢出 u32".into())
}

/// 固定130级快照的四条已核对规则。输入为转换前平铺值；主属性增量别名
/// 加到 profile 指定的具名属性，再执行通用转换和心法额外转换，不自动补裸装值。
#[cfg(feature = "engine")]
pub fn convert(
    stats: &Stats,
    profile: &School,
    value_set: &str,
    rule_id: &str,
) -> Result<AttributePanel, String> {
    // 只独立记录必要数学常数；每条来源/排除范围见文档，未复制上游代码或数据表。
    let (school, main, kind, atk_extra, crit_extra, pofang_extra) = match rule_id {
        MOWEN_130 => ("mowen", MainStat::Gengu, "lunar", 2007, 410, 0),
        AOXUE_130 => ("10026", MainStat::Liliang, "physical", 1843, 0, 287),
        FENSHAN_130 => ("10390", MainStat::Shenfa, "physical", 1925, 0, 0),
        HUAJIAN_130 => ("10021", MainStat::Yuanqi, "neutral", 2079, 0, 205),
        _ => return Err(format!("未知属性规则: {rule_id}")),
    };
    let damage_type = if kind == "physical" {
        DamageType::Waigong
    } else {
        DamageType::Neigong
    };
    if value_set != "live-130"
        || profile.id != school
        || profile.zhushuxing != main
        || profile.gongji_type != damage_type
        || profile.pofang_type != damage_type
    {
        return Err(format!(
            "{rule_id} 仅支持 live-130 心法{school}的{kind}属性及对应主属性"
        ));
    }
    let mut liliang = stats.liliang;
    let mut shenfa = stats.shenfa;
    let mut gengu = stats.gengu;
    let mut yuanqi = stats.yuanqi;
    let major = match main {
        MainStat::Liliang => &mut liliang,
        MainStat::Shenfa => &mut shenfa,
        MainStat::Gengu => &mut gengu,
        MainStat::Yuanqi => &mut yuanqi,
    };
    *major = sum(&[*major, stats.zhushuxing])?;
    let major = *major;
    let (generic_attack, generic_crit, generic_pofang) = if kind == "physical" {
        (
            scale(liliang, 163, 1000)?,
            scale(shenfa, 9, 10)?,
            scale(liliang, 3, 10)?,
        )
    } else {
        (
            scale(yuanqi, 181, 1000)?,
            scale(gengu, 9, 10)?,
            scale(yuanqi, 3, 10)?,
        )
    };
    let gongji_base = sum(&[stats.gongji, generic_attack])?;
    let gongji_extra = scale(major, atk_extra, 1024)?;
    let huixin = sum(&[stats.huixin, generic_crit, scale(major, crit_extra, 1024)?])?;
    let pofang = sum(&[
        stats.pofang,
        generic_pofang,
        scale(major, pofang_extra, 1024)?,
    ])?;
    let life_base = sum(&[stats.qixue, scale(stats.tizhi, 10, 1)?])?;
    let qixue_base = scale(life_base, 1331, 1024)?;
    let qixue = sum(&[qixue_base, stats.qixue_additional])?;
    Ok(AttributePanel {
        rule_id: rule_id.into(),
        source: SOURCE.into(),
        verified_in_game: false,
        damage_kind: kind.into(),
        tizhi: stats.tizhi,
        liliang,
        shenfa,
        gengu,
        yuanqi,
        gongji_base,
        gongji_extra,
        gongji: sum(&[gongji_base, gongji_extra])?,
        huixin,
        pofang,
        qixue,
        qixue_base,
        qixue_additional: stats.qixue_additional,
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
