//! 各心法 TOML 的属性换算规则契约。这里只定义数据结构，不内置游戏系数。
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BasicAttribute {
    Tizhi,
    Liliang,
    Shenfa,
    Gengu,
    Yuanqi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeDamageKind {
    Physical,
    Solar,
    Lunar,
    Neutral,
    Poison,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeTarget {
    GongjiBase,
    GongjiExtra,
    Huixin,
    Pofang,
    QixueBase,
    QixueAdditional,
}

/// 显式有理数；分子为零代表维护者明确配置为零，不提供任何默认分子或分母。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributeRatio {
    pub numerator: u32,
    pub denominator: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributeConversion {
    pub from: BasicAttribute,
    pub to: AttributeTarget,
    pub numerator: u32,
    pub denominator: u32,
}

/// 心法自带的固定点数加成（如「内功攻击力提高 2282 点」），直接加到目标属性。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributeConstant {
    pub to: AttributeTarget,
    pub value: u32,
}

/// 位于 data/shuxing/{profession}.toml 的 [attributes] 表；全部字段必须显式提供。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributeRules {
    pub id: String,
    pub level: u32,
    pub value_set: String,
    pub source: String,
    pub main_stat: BasicAttribute,
    pub damage_kind: AttributeDamageKind,
    pub qixue_multiplier: AttributeRatio,
    pub conversions: Vec<AttributeConversion>,
    /// 心法固定加成；旧数据缺省为空
    #[serde(default)]
    pub constants: Vec<AttributeConstant>,
}
