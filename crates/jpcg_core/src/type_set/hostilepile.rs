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
}
