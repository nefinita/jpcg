// ============================================================================
// types — 前端-后端通信的数据传输对象 (DTO)
// 所有通过 Tauri invoke 传输的数据结构在此定义。
// 包含配置类 DTO 和它们到核心库类型的转换实现。
// ============================================================================

use serde::{Deserialize, Serialize};

// ============================================================================
// 玩家配置 DTO（对应前端表单中的玩家属性输入）
// ============================================================================

/// 玩家属性 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfigDTO {
    pub jcsx: String,            // 基础属性类型（根骨/力道/身法/元气）
    pub jichu_shuxing: u32,     // 基础属性值
    pub jichu_gongji: u32,      // 基础攻击力
    pub huixin_dengji: u32,     // 会心等级
    pub huixin_xiaoguo: u32,    // 会心效果等级
    pub pofang_dengji: u32,     // 破防等级
    pub wuqi_shanghai: u32,     // 武器伤害
}

/// 目标（敌方）属性 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostileConfigDTO {
    pub waigong_fangyu: u32, // 外功防御
    pub neigong_fangyu: u32, // 内功防御
    pub yujin_dengji: u32,   // 御劲等级
    pub huajin_dengji: u32,  // 化劲等级
    pub jianshang_bili: u32, // 减伤比例（百分比）
}

/// 心法配置 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XinfaConfigDTO {
    pub xinfa_name: String,   // 心法名称
    pub xinfa_nom: String,    // 基础属性类型
    pub atk_up: f32,          // 攻击力加成
    pub pofang_up: f32,       // 破防加成
    pub huixin_up: f32,       // 会心加成
}

/// 计算请求（完整输入）
#[derive(Debug, Serialize, Deserialize)]
pub struct CalculateRequest {
    pub player: PlayerConfigDTO,       // 玩家属性
    pub hostile: HostileConfigDTO,     // 目标属性
    pub xinfa_config: XinfaConfigDTO,  // 心法配置
}

/// 技能计算结果 DTO（对应前端表格的 7 列）
#[derive(Debug, Serialize)]
pub struct SkillResultDTO {
    pub skill_name: String, // 技能名称
    pub y: u32,             // 破防系数段
    pub b: u32,             // 基础攻击段
    pub i: u32,             // 技能基础段
    pub n: u32,             // 普通命中段
    pub h: u32,             // 会心段
    pub q: u32,             // 期望值段
}

// ============================================================================
// DTO → 核心库类型转换
// ============================================================================

impl PlayerConfigDTO {
    /// 将前端 DTO 转换为核心库的 PlayerConfig
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
    /// 将前端 DTO 转换为核心库的 HostilepileConfig
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
    /// 将前端 DTO 转换为核心库的 XinfaConfig
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

/// 核心库计算结果 → 前端 DTO
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
