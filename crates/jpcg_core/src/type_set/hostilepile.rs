use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct HostilepileConfig {
    pub waigong_fangyu: u32, //外功防御
    pub neigong_fangyu: u32, //内功防御
    pub yujin_dengji: u32,   //御劲等级
    pub huajin_dengji: u32,  //化劲等级
    pub jianshang_bili: u32, //减伤比例
}
