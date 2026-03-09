use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerConfig {
    jcsx: String,
    jichu_shuxing: u32,
    jichu_gongji: u32,
    huixin_dengji: u32,
    huixin_xiaoguo: u32,
    pofang_dengji: u32,
    wuqi_shanghai: u32,
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
            jcsx: "根骨".to_string(),
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
        32
    }

    pub fn guo_pofang(&self) {}
}

struct AtkConfig {
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
