use iced::widget::{
    Space, button, column, container, pick_list, row, scrollable, space, text, text_input,
};
use iced::*;
use jpcg_core::type_set::hostilepile::HostilepileConfig;
use jpcg_core::type_set::player::PlayerConfig;
use jpcg_core::type_set::xinfa::XinfaConfig;

#[derive(Default)]
pub struct Counter {
    // 心法选择
    pub selected_xinfa: Option<Xinfa>,

    // 玩家属性s
    pub jcsx: String, // 基础属性
    pub jcgj: String, // 基础攻击
    pub hxdj: String, // 会心等级
    pub hxxg: String, // 会心效果
    pub pfdj: String, // 破防等级
    pub wqsh: String, // 武器伤害

    // 目标属性
    pub wgfy: String, // 外功防御
    pub ngfy: String, // 内功防御
    pub yjdj: String, // 御劲等级
    pub hjdj: String, // 化劲等级
    pub jsbl: String, // 减伤倍率

    skill_table1: String,
    skill_table2: String,
    skill_table3: String,
    skill_table4: String,
    skill_table5: String,
}

impl Counter {
    pub fn new() -> Self {
        Counter {
            selected_xinfa: Some(Xinfa::Mowen),
            jcsx: String::new(),
            jcgj: String::new(),
            hxdj: String::new(),
            hxxg: String::new(),
            pfdj: String::new(),
            wqsh: String::new(),
            wgfy: String::new(),
            ngfy: String::new(),
            yjdj: String::new(),
            hjdj: String::new(),
            jsbl: String::new(),
            skill_table1: String::new(),
            skill_table2: String::new(),
            skill_table3: String::new(),
            skill_table4: String::new(),
            skill_table5: String::new(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(
            row![
                column![
                    row![
                        Space::width(space(), 370.0),
                        text("属性填写区").size(30).width(300.0),
                    ]
                    .spacing(20),
                    row![
                        Space::width(space(), 20.0),
                        column![
                            pick_list(
                                &Xinfa::ALL[..],
                                self.selected_xinfa.clone(),
                                Message::XinfaSelected
                            )
                            .placeholder("选择心法"),
                            button("    开始计算")
                                .on_press(Message::Calculator)
                                .width(110.0)
                                .height(40.0),
                            button("    保存配置")
                                .on_press(Message::SaveConfig)
                                .width(110.0)
                                .height(40.0),
                            button("    加载配置")
                                .on_press(Message::LoadConfig)
                                .width(110.0)
                                .height(40.0),
                            button("        清空")
                                .on_press(Message::ClearConfig)
                                .width(110.0)
                                .height(40.0),
                        ]
                        .spacing(20),
                        column![
                            row![
                                "基础属性：",
                                text_input("", &self.jcsx)
                                    .on_input(Message::JcsxChanged)
                                    .width(150.0)
                            ]
                            .spacing(20),
                            row![
                                "基础攻击：",
                                text_input("", &self.jcgj)
                                    .on_input(Message::JcgjChanged)
                                    .width(150.0)
                            ]
                            .spacing(20),
                            row![
                                "会心等级：",
                                text_input("", &self.hxdj)
                                    .on_input(Message::HxdjChanged)
                                    .width(150.0)
                            ]
                            .spacing(20),
                            row![
                                "会心效果：",
                                text_input("", &self.hxxg)
                                    .on_input(Message::HxxgChanged)
                                    .width(150.0)
                            ]
                            .spacing(20),
                            row![
                                "破防等级：",
                                text_input("", &self.pfdj)
                                    .on_input(Message::PfdjChanged)
                                    .width(150.0)
                            ]
                            .spacing(20),
                            row![
                                "武器伤害：",
                                text_input("", &self.wqsh)
                                    .on_input(Message::WqshChanged)
                                    .width(150.0)
                            ]
                            .spacing(20),
                        ]
                        .spacing(20),
                        column![
                            row![Space::width(space(), 50.0), text("被攻击目标属性").size(24)]
                                .spacing(20),
                            row![
                                "外功防御：",
                                text_input("", &self.wgfy)
                                    .on_input(Message::WgfyChanged)
                                    .width(150.0)
                            ]
                            .spacing(20),
                            row![
                                "内功防御：",
                                text_input("", &self.ngfy)
                                    .on_input(Message::NgfyChanged)
                                    .width(150.0)
                            ]
                            .spacing(20),
                            row![
                                "御劲等级：",
                                text_input("", &self.yjdj)
                                    .on_input(Message::YjdjChanged)
                                    .width(150.0)
                            ]
                            .spacing(20),
                            row![
                                "化劲等级：",
                                text_input("", &self.hjdj)
                                    .on_input(Message::HjdjChanged)
                                    .width(150.0)
                            ]
                            .spacing(20),
                            row![
                                "减伤倍率：",
                                text_input("", &self.jsbl)
                                    .on_input(Message::JsblChanged)
                                    .width(150.0)
                            ]
                            .spacing(20),
                        ]
                        .spacing(20),
                    ]
                    .spacing(20),
                ]
                .spacing(20),
                column![
                    row![Space::width(space(), 240.0), text("计算结果").size(30)],
                    scrollable(row![
                        text(self.skill_table1.clone()).width(200.0),
                        text(self.skill_table2.clone()).width(180.0),
                        text(self.skill_table3.clone()).width(180.0),
                        text(self.skill_table4.clone()).width(180.0),
                        text(self.skill_table5.clone()).width(180.0),
                    ])
                    .width(920.0)
                    .height(300.0)
                    .spacing(20)
                ]
                .width(920.0)
                .spacing(20),
                row![Space::new()],
            ]
            .spacing(20),
        )
        .center_x(Fill)
        .center_y(Fill)
        .into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::XinfaSelected(xinfa) => {
                self.selected_xinfa = Some(xinfa);
            }
            Message::JcsxChanged(jcsx) => {
                self.jcsx = jcsx;
            }
            Message::JcgjChanged(jcgj) => {
                self.jcgj = jcgj;
            }
            Message::HxdjChanged(hxdj) => {
                self.hxdj = hxdj;
            }
            Message::HxxgChanged(hxxg) => {
                self.hxxg = hxxg;
            }
            Message::PfdjChanged(pfdj) => {
                self.pfdj = pfdj;
            }
            Message::WqshChanged(wqsh) => {
                self.wqsh = wqsh;
            }
            Message::WgfyChanged(wgfy) => {
                self.wgfy = wgfy;
            }
            Message::NgfyChanged(ngfy) => {
                self.ngfy = ngfy;
            }
            Message::YjdjChanged(yjdj) => {
                self.yjdj = yjdj;
            }
            Message::HjdjChanged(hjdj) => {
                self.hjdj = hjdj;
            }
            Message::JsblChanged(jsbl) => {
                self.jsbl = jsbl;
            }
            Message::Calculator => {
                //开始计算
                let player = PlayerConfig::new(
                    self.selected_xinfa.as_ref().unwrap().to_string(),
                    self.jcsx.parse().unwrap_or(0),
                    self.jcgj.parse().unwrap_or(0),
                    self.hxdj.parse().unwrap_or(0),
                    self.hxxg.parse().unwrap_or(0),
                    self.pfdj.parse().unwrap_or(0),
                    self.wqsh.parse().unwrap_or(0),
                );

                let hostilepile = HostilepileConfig::new(
                    self.wgfy.parse().unwrap_or(0),
                    self.ngfy.parse().unwrap_or(0),
                    self.yjdj.parse().unwrap_or(0),
                    self.hjdj.parse().unwrap_or(0),
                    self.jsbl.parse().unwrap_or(0),
                );

                let xinfa = XinfaConfig::new(
                    self.selected_xinfa.as_ref().unwrap().to_string(),
                    self.selected_xinfa.as_ref().unwrap().to_string(),
                    0.0,
                    0.0,
                    0.0,
                );

                let skill_out: Vec<jpcg_core::cal::CalculateResult> =
                    jpcg_core::calculate::start(player, hostilepile, xinfa);

                self.skill_table1.clear();
                self.skill_table2.clear();
                self.skill_table3.clear();
                self.skill_table4.clear();
                self.skill_table5.clear();

                for n in skill_out {
                    let mut pj1: String = String::new();
                    let mut pj2: String = String::new();
                    let mut pj3: String = String::new();
                    let mut pj4: String = String::new();
                    let mut pj5: String = String::new();

                    pj1.push_str("｜ ");
                    pj1.push_str(n.skill_name.as_str());
                    pj1.push('\n');
                    self.skill_table1.push_str(pj1.as_str());

                    pj2.push_str(" ｜ 普通伤害：");
                    pj2.push_str(n.n.to_string().as_str());
                    pj2.push('\n');
                    self.skill_table2.push_str(pj2.as_str());

                    pj3.push_str(" ｜ 会心伤害：");
                    pj3.push_str(n.h.to_string().as_str());
                    pj3.push('\n');
                    self.skill_table3.push_str(pj3.as_str());

                    pj4.push_str(" ｜ 期望伤害：");
                    pj4.push_str(n.q.to_string().as_str());
                    pj4.push('\n');
                    self.skill_table4.push_str(pj4.as_str());

                    pj5.push_str(" ｜");
                    pj5.push('\n');
                    self.skill_table5.push_str(pj5.as_str());
                }
            }
            Message::SaveConfig => {
                //保存配置
                jpcg_core::save_config::save(
                    PlayerConfig::new(
                        self.selected_xinfa
                            .as_ref()
                            .unwrap_or(&Xinfa::Mowen)
                            .to_string(),
                        self.jcsx.parse().unwrap_or(0),
                        self.jcgj.parse().unwrap_or(0),
                        self.hxdj.parse().unwrap_or(0),
                        self.hxxg.parse().unwrap_or(0),
                        self.pfdj.parse().unwrap_or(0),
                        self.wqsh.parse().unwrap_or(0),
                    ),
                    HostilepileConfig::new(
                        self.wgfy.parse().unwrap_or(0),
                        self.ngfy.parse().unwrap_or(0),
                        self.yjdj.parse().unwrap_or(0),
                        self.hjdj.parse().unwrap_or(0),
                        self.jsbl.parse().unwrap_or(0),
                    ),
                    XinfaConfig::new(
                        self.selected_xinfa
                            .as_ref()
                            .unwrap_or(&Xinfa::Mowen)
                            .to_string(),
                        "gengu".to_string(),
                        0.0,
                        0.0,
                        0.0,
                    ),
                );
            }
            Message::LoadConfig => {
                let x = jpcg_core::load_config::default_load();
                self.selected_xinfa = Some(Xinfa::from_string(&x.xinfa.xinfa_name)); //默认心法
            }
            Message::ClearConfig => {
                //清空配置
                self.jcsx.clear();
                self.jcgj.clear();
                self.hxdj.clear();
                self.hxxg.clear();
                self.pfdj.clear();
                self.wqsh.clear();

                self.wgfy.clear();
                self.ngfy.clear();
                self.yjdj.clear();
                self.hjdj.clear();
                self.jsbl.clear();

                self.skill_table1.clear();
                self.skill_table2.clear();
                self.skill_table3.clear();
                self.skill_table4.clear();
                self.skill_table5.clear();
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Xinfa {
    YinLongJue,         // 隐龙诀
    Wufang,             // 无方
    CangJianShanZhuang, // 藏剑山庄
    QiXiuFang,          // 七秀坊
    WanHuaGu,           // 万花谷
    TianCeFu,           // 天策府
    ShaoLinSi,          // 少林寺
    ChunYangGuan,       // 纯阳观
    WuDouJiao,          // 五毒教
    TangMen,            // 唐门
    MingJiao,           // 明教
    GaiBang,            // 丐帮
    CangYun,            // 苍云
    Mowen,              // 长歌门
    BaDaoShanZhuang,    // 霸刀山庄
    PengLai,            // 蓬莱
    YanTianZong,        // 衍天宗
    YaoZong,            // 药宗
}

impl Xinfa {
    pub const ALL: [Xinfa; 18] = [
        Xinfa::YinLongJue,
        Xinfa::Wufang,
        Xinfa::CangJianShanZhuang,
        Xinfa::QiXiuFang,
        Xinfa::WanHuaGu,
        Xinfa::TianCeFu,
        Xinfa::ShaoLinSi,
        Xinfa::ChunYangGuan,
        Xinfa::WuDouJiao,
        Xinfa::TangMen,
        Xinfa::MingJiao,
        Xinfa::GaiBang,
        Xinfa::CangYun,
        Xinfa::Mowen,
        Xinfa::BaDaoShanZhuang,
        Xinfa::PengLai,
        Xinfa::YanTianZong,
        Xinfa::YaoZong,
    ];

    pub fn from_string(s: &str) -> Self {
        match s {
            "隐龙诀" => Xinfa::YinLongJue,
            "无方" => Xinfa::Wufang,
            "藏剑山庄" => Xinfa::CangJianShanZhuang,
            "七秀坊" => Xinfa::QiXiuFang,
            "万花谷" => Xinfa::WanHuaGu,
            "天策府" => Xinfa::TianCeFu,
            "少林寺" => Xinfa::ShaoLinSi,
            "纯阳观" => Xinfa::ChunYangGuan,
            "五毒教" => Xinfa::WuDouJiao,
            "唐门" => Xinfa::TangMen,
            "明教" => Xinfa::MingJiao,
            "丐帮" => Xinfa::GaiBang,
            "苍云" => Xinfa::CangYun,
            "莫问" => Xinfa::Mowen,
            "霸刀山庄" => Xinfa::BaDaoShanZhuang,
            "蓬莱" => Xinfa::PengLai,
            "衍天宗" => Xinfa::YanTianZong,
            "药宗" => Xinfa::YaoZong,
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
            Xinfa::YinLongJue => write!(f, "隐龙诀"),
            Xinfa::Wufang => write!(f, "无方"),
            Xinfa::CangJianShanZhuang => write!(f, "藏剑山庄"),
            Xinfa::QiXiuFang => write!(f, "七秀坊"),
            Xinfa::WanHuaGu => write!(f, "万花谷"),
            Xinfa::TianCeFu => write!(f, "天策府"),
            Xinfa::ShaoLinSi => write!(f, "少林寺"),
            Xinfa::ChunYangGuan => write!(f, "纯阳观"),
            Xinfa::WuDouJiao => write!(f, "五毒教"),
            Xinfa::TangMen => write!(f, "唐门"),
            Xinfa::MingJiao => write!(f, "明教"),
            Xinfa::GaiBang => write!(f, "丐帮"),
            Xinfa::CangYun => write!(f, "苍云"),
            Xinfa::Mowen => write!(f, "莫问"),
            Xinfa::BaDaoShanZhuang => write!(f, "霸刀山庄"),
            Xinfa::PengLai => write!(f, "蓬莱"),
            Xinfa::YanTianZong => write!(f, "衍天宗"),
            Xinfa::YaoZong => write!(f, "药宗"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    XinfaSelected(Xinfa),
    JcsxChanged(String),
    JcgjChanged(String),
    HxdjChanged(String),
    HxxgChanged(String),
    PfdjChanged(String),
    WqshChanged(String),
    WgfyChanged(String),
    NgfyChanged(String),
    YjdjChanged(String),
    HjdjChanged(String),
    JsblChanged(String),
    Calculator,
    SaveConfig,
    LoadConfig,
    ClearConfig,
}
