use serde::{Deserialize, Serialize};

use crate::log::error;
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerConfig {
    jcsx: String,
    pub jichu_shuxing: u32,
    pub jichu_gongji: u32,
    pub huixin_dengji: u32,
    pub huixin_xiaoguo: u32,
    pub pofang_dengji: u32,
    pub wuqi_shanghai: u32,
}

impl PlayerConfig {
    pub fn new(
        jcsx: String,
        jichu_shuxing: u32,
        jichu_gongji: u32,
        huixin_dengji: u32,
        huixin_xiaoguo: u32,
        pofang_dengji: u32,
        wuqi_shanghai: u32,
    ) -> Self {
        Self {
            jcsx,
            jichu_shuxing,
            jichu_gongji,
            huixin_dengji,
            huixin_xiaoguo,
            pofang_dengji,
            wuqi_shanghai,
        }
    }

    pub fn default() -> Self {
        Self {
            jcsx: "gengu".to_string(),
            jichu_shuxing: 100,
            jichu_gongji: 200,
            huixin_dengji: 1,
            huixin_xiaoguo: 10,
            pofang_dengji: 1,
            wuqi_shanghai: 50,
        }
    }

    pub fn atk(&self, atk_up: f32) -> AtkConfig {
        let base = (self.jichu_gongji + self.jichu_shuxing) as f32 * (1.0 + atk_up);
        AtkConfig {
            base: base as u32,
            extra: self.wuqi_shanghai,
        }
    }

    fn jcsx_to_atk(&self) -> u32 {
        match self.jcsx.as_str() {
            "gengu" => self.jichu_shuxing * 2,
            "yuanqi" => self.jichu_shuxing * 2,
            "lidao" => self.jichu_shuxing * 2,
            "shenfa" => self.jichu_shuxing * 2,
            _ => {
                error(format!("未知的基础属性: {}", self.jcsx).as_str());
                0
            }
        }
    }

    pub fn guo_pofang(&self) -> u32 {
        ((self.pofang_dengji * 1024) as f32 / 225957.6) as u32
    }
    pub fn guo_huixinxiaoguo(&self) -> u32 {
        (self.huixin_xiaoguo as f32 * 1024.0 / 72844.2) as u32
    }

    pub fn guo_huixin(&self) -> f32 {
        self.huixin_dengji as f32 / 197703.0
    }
}

pub struct AtkConfig {
    base: u32,
    extra: u32,
}

impl AtkConfig {
    pub fn total(&self) -> u32 {
        self.base + self.extra
    }

    pub fn atk_base_show(&self) -> u32 {
        self.base
    }

    pub fn atk_extra_show(&self) -> u32 {
        self.extra
    }
}
