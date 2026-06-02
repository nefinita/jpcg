// ============================================================================
// atkcal — 单技能伤害计算器
// 实现剑网3 伤害计算的核心公式链:
//   Y(破防系数) → B(基础攻击) → I(技能基础) → G(普通命中) → H(会心) → Q(期望)
// 每一段计算依赖前一段的结果，最终输出 5 段伤害。
// ============================================================================

use crate::type_set::buff::BuffConfig;
use crate::type_set::coefficient::CoefficientConfig;
use crate::type_set::hostilepile::HostilepileConfig;
use crate::type_set::player::PlayerConfig;
use crate::type_set::skilltype::Skilltype;
use crate::type_set::xinfa::XinfaConfig;

/// 单技能伤害计算器
/// 聚合玩家属性、目标防御、技能数据、心法加成四个维度的输入，
/// 通过链式调用逐步计算各段伤害。
pub struct JpcgConfig {
    player: PlayerConfig,           // 玩家属性（攻击、会心、破防等）
    hostilepile: HostilepileConfig, // 目标属性（防御、御劲、化劲、减伤）
    skilltype: Skilltype,           // 当前技能数据（系数、增伤等）
    xinfa: XinfaConfig,             // 心法加成（根骨/元气、破防/会心百分比）
    buff: BuffConfig,               // 阵眼/奇穴增益
    coeff: CoefficientConfig,       // 可配置系数
}

impl JpcgConfig {
    /// 创建单技能计算器实例
    pub fn new(
        playerdata: PlayerConfig,
        hostilepiledata: HostilepileConfig,
        skilltypedata: Skilltype,
        xinfadata: XinfaConfig,
    ) -> JpcgConfig {
        JpcgConfig {
            player: playerdata,
            hostilepile: hostilepiledata,
            skilltype: skilltypedata,
            xinfa: xinfadata,
            buff: BuffConfig::default(),
            coeff: CoefficientConfig::default(),
        }
    }

    /// 创建带增益和系数配置的计算器实例
    pub fn new_with_config(
        playerdata: PlayerConfig,
        hostilepiledata: HostilepileConfig,
        skilltypedata: Skilltype,
        xinfadata: XinfaConfig,
        buff: BuffConfig,
        coeff: CoefficientConfig,
    ) -> JpcgConfig {
        let mut config = Self::new(playerdata, hostilepiledata, skilltypedata, xinfadata);
        config.buff = buff;
        config.coeff = coeff;
        config
    }

    /// 计算目标的实际防御系数（含无视防御 + 增益无视防御）
    fn guo_fangyu(&self) -> u32 {
        let wushifangyu_total = self.skilltype.wushifangyu
            + (self.buff.wushi_fangyu_pct * 1024.0 / 100.0) as u32;
        match self.xinfa.xinfa_nom.as_str() {
            "gengu" | "yuanqi" => self.hostilepile.guo_nfangyu_with(wushifangyu_total, &self.coeff),
            _ => self.hostilepile.guo_wfangyu_with(wushifangyu_total, &self.coeff),
        }
    }

    /// 计算实际会心率（玩家会心率 - 目标御劲会心减免 + 增益会心）
    pub fn guo_huixin(&self) -> f32 {
        let player_crit = self.player.guo_huixin_with(&self.coeff) + self.buff.huixin_pct / 100.0;
        let enemy_crit_reduce = self.hostilepile.guo_yujin_huixin_with(&self.coeff);
        if player_crit >= enemy_crit_reduce {
            player_crit - enemy_crit_reduce
        } else {
            0.0
        }
    }

    /// Y 段: 破防系数
    fn y_cal(&self) -> u32 {
        let pofang = self.player.guo_pofang_with(&self.coeff)
            + (self.buff.pofang_pct * 1024.0 / 100.0) as u32;
        1024 + pofang - ((1024.0 + pofang as f32) * (self.guo_fangyu() as f32 / 1024.0)) as u32
    }

    /// B 段: 基础攻击力（含阵眼增益）
    fn b_cal(&self) -> u32 {
        self.player.atk_with_buff(self.xinfa.atk_up, self.buff.base_atk_pct).total()
    }

    /// I 段: 技能基础伤害
    /// = 技能基础攻击 + B段 × 技能伤害系数 + 武器伤害 × 武器系数 / 100
    /// 输出 [0, atk_sum, skill_damage, 0, 0] 原始面板伤害
    fn i_cal(&self) -> [u32; 5] {
        let atk = self.b_cal();
        let x = self.skilltype.base_atk()
            + (atk as f32 * self.skilltype.atk_xishu) as u32
            + (self.player.wuqi_shanghai as f32 * self.skilltype.watk_xishu as f32 / 100.0) as u32;
        [0, atk, x, 0, 0]
    }

    /// G 段: 普通命中伤害
    fn g_cal(&self) -> [u32; 5] {
        let i = self.i_cal();
        let y = self.y_cal();
        let i_hit = i[2];
        let shanghai_buff = 1.0 + self.buff.shanghai_pct / 100.0;
        let huajin = self.hostilepile.guo_huajin_with(&self.coeff);
        let pvp = self.coeff.pvp_global_jianshang;
        let x = (((((i_hit as f32 * (1.0 + self.skilltype.hit_up as f32 / 100.0) * shanghai_buff)
            * (y as f32 / 1024.0)) as u32 as f32
            * (1.0 - (huajin as f32 / 1024.0))) as u32
            as f32
            * pvp)
            * (1.0 - self.hostilepile.jianshang_bili as f32 / 100.0)) as u32;
        [y, i[1], i[2], x, 0]
    }

    /// H 段: 会心伤害
    fn h_cal(&self) -> [u32; 5] {
        let i = self.g_cal();
        let g_damage = i[3];
        let huixiao = self.player.guo_huixinxiaoguo_with(&self.coeff);
        let yujin_huixiao = self.hostilepile.guo_yujin_huixiao_with(&self.coeff);
        let buff_huixiao = self.buff.huixiao_pct * 1024.0 / 100.0;
        let x = g_damage
            + (g_damage as f32
                * (0.75
                    + (huixiao as f32 + buff_huixiao) / 1024.0
                    + self.skilltype.huixiao_up as f32 / 100.0)
                * (1.0 - yujin_huixiao as f32 / 1024.0))
                as u32;
        [i[0], i[1], i[2], i[3], x]
    }

    /// Q 段: 期望伤害（最终结果）
    pub fn q_cal(&self) -> DamageResult {
        let i = self.h_cal();
        let crit_rate = self.guo_huixin() + self.skilltype.huixin_up as f32 / 100.0
            + self.buff.huixin_pct / 100.0;
        let x = (i[3] as f32 * (1.0 - crit_rate) + i[4] as f32 * crit_rate) as u32;
        DamageResult::new(i, x)
    }
}

// ============================================================================
// DamageResult — 五段伤害结果结构
// 将 [Y, atk, base_damage, G_damage, H_damage] 数组映射为具名字段。
// ============================================================================

/// 五段伤害输出结果
pub struct DamageResult {
    pub y: u32,        // Y: 破防系数
    pub i: u32,        // I: 基础攻击总计
    pub b: u32,        // B: 技能基础伤害
    pub g_damage: u32, // G: 普通命中伤害（经过防御、化劲、减伤衰减后）
    pub h_damage: u32, // H: 会心伤害（包含会效加成）
    pub q_damage: u32, // Q: 期望伤害（会心与非会心的加权平均）
}

impl DamageResult {
    /// 从 5 段计算数组构造结果
    /// `i` 数组结构: [破防系数, 攻击力合计, 技能基础伤害, 普通命中伤害, 会心伤害]
    /// `x`: 最终期望伤害
    pub fn new(i: [u32; 5], x: u32) -> DamageResult {
        DamageResult {
            y: i[0],
            i: i[1],
            b: i[2],
            g_damage: i[3],
            h_damage: i[4],
            q_damage: x,
        }
    }
}
