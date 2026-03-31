use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct HostilepileConfig {
    pub waigong_fangyu: u32, //外功防御
    pub neigong_fangyu: u32, //内功防御
    pub yujin_dengji: u32,   //御劲等级
    pub huajin_dengji: u32,  //化劲等级
    pub jianshang_bili: u32, //减伤比例
}

impl HostilepileConfig {
    pub fn new(
        waigong_fangyu: u32,
        neigong_fangyu: u32,
        yujin_dengji: u32,
        huajin_dengji: u32,
        jianshang_bili: u32,
    ) -> Self {
        Self {
            waigong_fangyu,
            neigong_fangyu,
            yujin_dengji,
            huajin_dengji,
            jianshang_bili,
        }
    }

    pub fn default() -> Self {
        Self {
            waigong_fangyu: 100,
            neigong_fangyu: 100,
            yujin_dengji: 1,
            huajin_dengji: 1,
            jianshang_bili: 10,
        }
    }

    pub fn guo_wfangyu(&self, guo_wsfangyu: u32) -> u32 {
        ((self.waigong_fangyu as f32 * (1.0 - guo_wsfangyu as f32 / 1024.0)) * 1024.0
            / (self.waigong_fangyu as f32 * (1.0 - guo_wsfangyu as f32 / 1024.0) + 126007.2))
            as u32
    }

    pub fn guo_nfangyu(&self, guo_wsfangyu: u32) -> u32 {
        ((self.neigong_fangyu as f32 * (1.0 - guo_wsfangyu as f32 / 1024.0)) * 1024.0
            / (self.neigong_fangyu as f32 * (1.0 - guo_wsfangyu as f32 / 1024.0) + 126007.2))
            as u32
    }

    pub fn guo_huajin(&self) -> u32 {
        ((self.huajin_dengji as f32 / (self.huajin_dengji as f32 + 30115.8) + 102.0 / 1024.0)
            * 1024.0) as u32
    }

    pub fn guo_yujin_huixiao(&self) -> u32 {
        (self.yujin_dengji as f32 * 1024.0 / 55123.2) as u32
    }

    pub fn guo_yujin_huixin(&self) -> f32 {
        self.yujin_dengji as f32 / 197703.0
    }
}
