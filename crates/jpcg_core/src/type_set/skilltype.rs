use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Skilltype {
    pub skill_name: String,        //名字
    pub skill_id: u32,             //技能ID
    pub sub_id: u32,               //子ID
    pub group: u8,                 //套路组编号
    pub weapon_request: u8,        //需要武器编号
    pub design_effect: u8,         //技能生效方式（直接伤害，Dot等）
    pub kind_type: u8,             //技能类型(外功、毒性内功)
    pub cast_mode: u8,             //释放方式（群攻、单体等）
    pub guaranteed_hit: bool,      //必定命中标签（true,false）
    pub has_critical_strike: bool, //无质标签（true,false）
    pub effect_type: u8,           //技能效果标签（有害0，有益1）
    pub jihuoqixue: String,        //激活奇穴
    pub base_damage1: u32,             //基本伤害
    pub base_damage2: u32,             //基本伤害2
    pub atk_xishu: f32,            //伤害系数
    pub watk_xishu: u32,           //武器伤害系数
    pub hit_up: u32,               //增伤乘区
    pub huixin_up: u32,            //额外会心
    pub huixiao_up: u32,           //额外会效
    pub wushifangyu: u32,          //无视防御
    pub wushihuajin: u32,          //无视化劲
    pub wushijianshang: u32,       //无视减伤
    pub zhenshishanghai: u32,      //真实伤害
    pub dot_flag: u8,              //dot标签
    pub dot_num: u8,               //dot次数
    pub dot_up: f32,               //dot递增
}

impl Skilltype {
    pub fn base_atk(&self) -> u32 {
        (self.base_damage1 + self.base_damage2) / 2
    }
}