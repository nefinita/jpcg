use serde::{Deserialize, Serialize};

// ========== 前端传输用的 DTO (Data Transfer Object) ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfigDTO {
    pub jcsx: String,
    pub jichu_shuxing: u32,
    pub jichu_gongji: u32,
    pub huixin_dengji: u32,
    pub huixin_xiaoguo: u32,
    pub pofang_dengji: u32,
    pub wuqi_shanghai: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostileConfigDTO {
    pub waigong_fangyu: u32, //外功防御
    pub neigong_fangyu: u32, //内功防御
    pub yujin_dengji: u32,   //御劲等级
    pub huajin_dengji: u32,  //化劲等级
    pub jianshang_bili: u32, //减伤比例
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XinfaConfigDTO {
    pub xinfa_name: String,
    pub xinfa_nom: String,
    pub atk_up: f32,
    pub pofang_up: f32,
    pub huixin_up: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculateRequest {
    pub player: PlayerConfigDTO,
    pub hostile: HostileConfigDTO,
    pub xinfa_config: XinfaConfigDTO, // 可选扩展
}

#[derive(Debug, Serialize)]
pub struct SkillResultDTO {
    pub skill_name: String,
    pub y: u32,
    pub b: u32,
    pub i: u32,
    pub n: u32,
    pub h: u32,
    pub q: u32,
}

// ========== 类型转换实现 ==========

impl PlayerConfigDTO {
    pub fn into_core(self) -> jpcg_core::type_set::player::PlayerConfig {
        jpcg_core::type_set::player::PlayerConfig::new(
            self.jcsx,
            self.jichu_shuxing,
            self.jichu_gongji,
            self.huixin_dengji,
            self.huixin_xiaoguo,
            self.pofang_dengji,
            self.wuqi_shanghai,
        )
    }
}

impl HostileConfigDTO {
    pub fn into_core(self) -> jpcg_core::type_set::hostilepile::HostilepileConfig {
        jpcg_core::type_set::hostilepile::HostilepileConfig::new(
            self.waigong_fangyu,
            self.neigong_fangyu,
            self.yujin_dengji,
            self.huajin_dengji,
            self.jianshang_bili,
        )
    }
}

impl XinfaConfigDTO {
    pub fn into_core(self) -> jpcg_core::type_set::xinfa::XinfaConfig {
        jpcg_core::type_set::xinfa::XinfaConfig::new(
            self.xinfa_name,
            self.xinfa_nom,
            self.atk_up,
            self.pofang_up,
            self.huixin_up,
        )
    }
}

impl From<jpcg_core::cal::CalculateResult> for SkillResultDTO {
    fn from(core: jpcg_core::cal::CalculateResult) -> Self {
        Self {
            skill_name: core.skill_name,
            b: core.b,
            i: core.i,
            n: core.n,
            h: core.h,
            q: core.q,
            y: core.y,
        }
    }
}
