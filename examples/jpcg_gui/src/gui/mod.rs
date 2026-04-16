mod drug_table;
mod message;
mod qixue_table;
mod xinfa_table;
mod yanxi_table;

use iced::widget::{
    Space, button, column, container, image, pick_list, row, scrollable, space, text, text_input,
};
use iced::*;
use jpcg_core::type_set::hostilepile::HostilepileConfig;
use jpcg_core::type_set::player::PlayerConfig;
use jpcg_core::type_set::xinfa::XinfaConfig;
use message::Message;
use xinfa_table::Xinfa;

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
    pub fn view(&self) -> Element<'_, Message> {
        container(column![
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
                .spacing(20),
            ]
            .spacing(20),
            column![
                row![
                    space().width(20),
                    text("奇穴调整区（开发中）").size(24),
                    space().width(100)
                ]
                .spacing(20),
                row![text("流派选择",)],
                row![
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                ],
                row![
                    space().width(20),
                    text("奇穴1"),
                    text("奇穴2"),
                    text("奇穴3"),
                    text("奇穴4"),
                    text("奇穴5"),
                    text("奇穴6"),
                    text("奇穴7"),
                    text("奇穴8"),
                    text("奇穴9")
                ]
                .spacing(30),
                row![space().width(20), text("buff选择区(开发中)")].spacing(20),
                row![
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                ]
                .spacing(20),
                row![
                    space().width(20),
                    text("宴席1"),
                    space().width(20),
                    text("宴席2"),
                    space().width(20),
                    text("小药1"),
                    space().width(20),
                    text("小药2"),
                    space().width(20),
                    text("小药3"),
                    space().width(20),
                    text("小药4"),
                    space().width(20),
                    text("小药5"),
                ]
                .spacing(20),
                row![
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                    space().width(20),
                    image("assets/test.png").width(50).height(50),
                ]
                .spacing(20),
                row![
                    space().width(20),
                    text("阵眼"),
                    space().width(20),
                    text("增益1"),
                    space().width(20),
                    text("增益2"),
                    space().width(20),
                    text("增益3"),
                    space().width(20),
                    text("增益4"),
                    space().width(20),
                    text("增益5"),
                    space().width(20),
                    text("增益6"),
                ]
                .spacing(20),
            ]
            .spacing(20)
        ])
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
                Xinfa::from_string(&x.xinfa.xinfa_name);
                self.jcsx = x.player.jichu_shuxing.to_string();
                self.jcgj = x.player.jichu_gongji.to_string();
                self.hxdj = x.player.huixin_dengji.to_string();
                self.hxxg = x.player.huixin_xiaoguo.to_string();
                self.pfdj = x.player.pofang_dengji.to_string();
                self.wqsh = x.player.wuqi_shanghai.to_string();
                self.wgfy = x.hostilepile.waigong_fangyu.to_string();
                self.ngfy = x.hostilepile.neigong_fangyu.to_string();
                self.yjdj = x.hostilepile.yujin_dengji.to_string();
                self.hjdj = x.hostilepile.huajin_dengji.to_string();
                self.jsbl = x.hostilepile.jianshang_bili.to_string();
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
