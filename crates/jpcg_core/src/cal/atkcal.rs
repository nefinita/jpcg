// ============================================================================
// atkcal — 单技能伤害计算器
// 实现剑网3 伤害计算的核心公式链:
//   Y(破防系数) → B(基础攻击) → I(技能基础) → G(普通命中) → H(会心) → Q(期望)
// 每一段计算依赖前一段的结果，最终输出 5 段伤害。
// ============================================================================

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
        }
    }

    /// 计算目标的实际防御系数（含无视防御）
    /// 根据心法根骨/元气属性区分外功防御和内功防御路径
    fn guo_fangyu(&self) -> u32 {
        match self.xinfa.xinfa_nom.as_str() {
            // 根骨/元气职业走内功防御公式
            "gengu" | "yuanqi" => self.hostilepile.guo_nfangyu(self.skilltype.wushifangyu),
            // 其他（力道/身法）走外功防御公式
            _ => self.hostilepile.guo_wfangyu(self.skilltype.wushifangyu),
        }
    }

    /// 计算实际会心率（玩家会心率 - 目标御劲会心减免）
    /// 返回值区间 [0.0, ~1.0]（百分比的小数表示）
    fn guo_huixin(&self) -> f32 {
        if self.player.guo_huixin() >= self.hostilepile.guo_yujin_huixin() {
            self.player.guo_huixin() - self.hostilepile.guo_yujin_huixin()
        } else {
            0.0
        }
    }

    /// Y 段: 破防系数
    /// = 1024 + 破防等级 - (1024 + 破防等级) × 防御系数 / 1024
    /// 反映攻击穿透目标防御后的有效伤害比例（以 1024 为基准）：
    /// - 值 >1024 表示破防高于防御，有加成
    /// - 值 <1024 表示防御高于破防，有衰减
    fn y_cal(&self) -> u32 {
        1024 + self.player.guo_pofang()
            - ((1024.0 + self.player.guo_pofang() as f32) * (self.guo_fangyu() as f32 / 1024.0))
                as u32
    }

    /// B 段: 基础攻击力
    /// = 基础攻击 × (1 + 心法攻击加成) + 武器伤害
    fn b_cal(&self) -> u32 {
        self.player.atk(self.xinfa.atk_up).total()
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

    /// G 段: 普通命中伤害（考虑防御、化劲、减伤后）
    /// 计算链: I段 → 增伤乘区 → 破防系数 → 化劲减免 → 减伤减免 → 90% 系数
    /// 输出 [Y, atk_sum, I_damage, g_damage, 0]
    fn g_cal(&self) -> [u32; 5] {
        let i = self.i_cal();
        let y = self.y_cal();
        let i_hit = i[2];
        // 命中伤害 = I段 × (1+增伤) × (Y/1024) × (1-化劲) × 0.9 × (1-减伤)
        let x = (((((i_hit as f32 * (1.0 + self.skilltype.hit_up as f32 / 100.0))
            * (y as f32 / 1024.0)) as u32 as f32
            * (1.0 - (self.hostilepile.guo_huajin() as f32 / 1024.0))) as u32
            as f32
            * 0.9)
            * (1.0 - self.hostilepile.jianshang_bili as f32 / 100.0)) as u32;
        [y, i[1], i[2], x, 0]
    }

    /// H 段: 会心伤害
    /// = G段普通命中伤害 + G段 × (0.75 + 会效/1024 + 技能会效加成/100) × (1 - 御劲会效减免/1024)
    /// 输出 [Y, atk, base_damage, G_damage, H_damage]
    fn h_cal(&self) -> [u32; 5] {
        let i = self.g_cal();
        let g_damage = i[3];
        // 会心增伤基数为 0.75（75%），加上会效和技能加成，扣除目标御劲会效减免
        let x = g_damage
            + (g_damage as f32
                * (0.75
                    + self.player.guo_huixinxiaoguo() as f32 / 1024.0
                    + self.skilltype.huixiao_up as f32 / 100.0)
                * (1.0 - self.hostilepile.guo_yujin_huixiao() as f32 / 1024.0))
                as u32;
        [i[0], i[1], i[2], i[3], x]
    }

    /// Q 段: 期望伤害（最终结果）
    /// = G段 × (1 - 实际会心率) + H段 × 实际会心率
    /// 反映考虑会心概率后的长期平均伤害
    pub fn q_cal(&self) -> DamageResult {
        let i = self.h_cal();
        // 期望伤害 = 非会心概率 × 普通伤害 + 会心概率 × 会心伤害
        let x = (i[3] as f32
            * (1.0 - (self.guo_huixin() + self.skilltype.huixin_up as f32 / 100.0))
            + i[4] as f32 * (self.player.guo_huixin() + self.skilltype.huixin_up as f32 / 100.0))
            as u32;
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
