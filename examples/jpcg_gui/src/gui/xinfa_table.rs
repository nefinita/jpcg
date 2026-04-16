#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Xinfa {
    YinLong,  // 隐龙诀
    WuFang,   // 无方
    CangJian, // 藏剑山庄
    BinXin,   // 七秀坊
    HuaJian,  // 万花谷
    AoXue,    // 天策府
    YiJin,    // 少林寺
    ZiXia,    // 紫霞功
    TaiXu,    // 太虚
    DuJing,   // 五毒教
    JingYu,   // 唐门
    TianLuo,  //
    FenYing,  // 明教
    XiaoChen, // 丐帮
    FenShan,  // 苍云
    Mowen,    // 长歌门
    BeiAo,    // 霸刀山庄
    LingHai,  // 蓬莱
    TaiXuan,  // 衍天宗
    YouLuo,   // 楼
}

impl Xinfa {
    pub const ALL: [Xinfa; 20] = [
        Xinfa::YinLong,
        Xinfa::WuFang,
        Xinfa::CangJian,
        Xinfa::BinXin,
        Xinfa::HuaJian,
        Xinfa::AoXue,
        Xinfa::YiJin,
        Xinfa::ZiXia,
        Xinfa::TaiXu,
        Xinfa::DuJing,
        Xinfa::JingYu,
        Xinfa::TianLuo,
        Xinfa::FenYing,
        Xinfa::XiaoChen,
        Xinfa::FenShan,
        Xinfa::Mowen,
        Xinfa::BeiAo,
        Xinfa::LingHai,
        Xinfa::TaiXuan,
        Xinfa::YouLuo,
    ];

    pub fn from_string(s: &str) -> Self {
        match s {
            "隐龙诀" => Xinfa::YinLong,
            "无方" => Xinfa::WuFang,
            "藏剑山庄" => Xinfa::CangJian,
            "冰心" => Xinfa::BinXin,
            "花间游" => Xinfa::HuaJian,
            "傲血战意" => Xinfa::AoXue,
            "易筋经" => Xinfa::YiJin,
            "紫霞功" => Xinfa::ZiXia,
            "太虚剑意" => Xinfa::TaiXu,
            "毒经" => Xinfa::DuJing,
            "惊羽诀" => Xinfa::JingYu,
            "天罗诡道" => Xinfa::TianLuo,
            "焚影圣诀" => Xinfa::FenYing,
            "笑尘诀" => Xinfa::XiaoChen,
            "分山劲" => Xinfa::FenShan,
            "莫问" => Xinfa::Mowen,
            "北傲诀" => Xinfa::BeiAo,
            "凌海诀" => Xinfa::LingHai,
            "太玄经" => Xinfa::TaiXuan,
            "幽罗引" => Xinfa::YouLuo,
            _ => {
                crate::log::error(format!("未知的心法: {}", s).as_str());
                Xinfa::Mowen // 默认返回莫问
            }
        }
    }
}

impl std::fmt::Display for Xinfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Xinfa::YinLong => write!(f, "隐龙诀"),
            Xinfa::WuFang => write!(f, "无方"),
            Xinfa::CangJian => write!(f, "藏剑山庄"),
            Xinfa::BinXin => write!(f, "冰心诀"),
            Xinfa::HuaJian => write!(f, "花间游"),
            Xinfa::AoXue => write!(f, "傲血战意"),
            Xinfa::YiJin => write!(f, "易筋经"),
            Xinfa::ZiXia => write!(f, "紫霞功"),
            Xinfa::TaiXu => write!(f, "太虚剑意"),
            Xinfa::DuJing => write!(f, "毒经"),
            Xinfa::JingYu => write!(f, "惊羽诀"),
            Xinfa::TianLuo => write!(f, "天罗诡道"),
            Xinfa::FenYing => write!(f, "焚影圣诀"),
            Xinfa::XiaoChen => write!(f, "笑尘诀"),
            Xinfa::FenShan => write!(f, "分山劲"),
            Xinfa::Mowen => write!(f, "莫问"),
            Xinfa::BeiAo => write!(f, "北傲诀"),
            Xinfa::LingHai => write!(f, "凌海诀"),
            Xinfa::TaiXuan => write!(f, "太玄经"),
            Xinfa::YouLuo => write!(f, "幽罗引"),
        }
    }
}
