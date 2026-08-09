use crate::type_set::buff::BuffConfig;
use crate::type_set::coefficient::CoefficientConfig;
use crate::type_set::hostilepile::HostilepileConfig;
use crate::type_set::player::PlayerConfig;
use crate::type_set::skilltype::Skilltype;
use crate::type_set::xinfa::XinfaConfig;

/// 单技能伤害计算器
/// 所有配置均以引用持有，避免逐技能全量 clone（热路径优化）。
pub struct JpcgConfig<'a> {
    player: &'a PlayerConfig,
    hostilepile: &'a HostilepileConfig,
    skilltype: &'a Skilltype,
    xinfa: &'a XinfaConfig,
    buff: &'a BuffConfig,
    coeff: &'a CoefficientConfig,
}

impl<'a> JpcgConfig<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_config(
        playerdata: &'a PlayerConfig,
        hostilepiledata: &'a HostilepileConfig,
        skilltypedata: &'a Skilltype,
        xinfadata: &'a XinfaConfig,
        buff: &'a BuffConfig,
        coeff: &'a CoefficientConfig,
    ) -> JpcgConfig<'a> {
        JpcgConfig {
            player: playerdata,
            hostilepile: hostilepiledata,
            skilltype: skilltypedata,
            xinfa: xinfadata,
            buff,
            coeff,
        }
    }

    fn guo_fangyu(&self) -> u32 {
        let wushifangyu_total = self.skilltype.wushifangyu
            + (self.buff.wushi_fangyu_pct * 1024.0 / 100.0) as u32;
        match self.xinfa.xinfa_nom.as_str() {
            "gengu" | "yuanqi" => self.hostilepile.guo_nfangyu_with(wushifangyu_total, &self.coeff),
            _ => self.hostilepile.guo_wfangyu_with(wushifangyu_total, &self.coeff),
        }
    }

    pub fn guo_huixin(&self) -> f32 {
        let player_crit = self.player.guo_huixin_with(&self.coeff) + self.buff.huixin_pct / 100.0;
        let enemy_crit_reduce = self.hostilepile.guo_yujin_huixin_with(&self.coeff);
        if player_crit >= enemy_crit_reduce {
            player_crit - enemy_crit_reduce
        } else {
            0.0
        }
    }

    fn y_cal(&self) -> u32 {
        let pofang = self.player.guo_pofang_with(&self.coeff)
            + (self.buff.pofang_pct * 1024.0 / 100.0) as u32;
        1024 + pofang - ((1024.0 + pofang as f32) * (self.guo_fangyu() as f32 / 1024.0)) as u32
    }

    fn b_cal(&self) -> u32 {
        self.player.atk_with_buff(self.xinfa.atk_up, self.buff.base_atk_pct).total()
    }

    fn i_cal(&self) -> [u32; 5] {
        let atk = self.b_cal();
        let x = self.skilltype.base_atk()
            + (atk as f32 * self.skilltype.atk_xishu) as u32
            + (self.player.wuqi_shanghai as f32 * self.skilltype.watk_xishu as f32 / 100.0) as u32;
        [0, atk, x, 0, 0]
    }

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
    /// crit_rate = 自身会心率 - 目标御劲减免 + 技能增益
    /// buff.huixin_pct 已在 guo_huixin() 中计入，此处不再重复
    pub fn q_cal(&self) -> DamageResult {
        let i = self.h_cal();
        let crit_rate = self.guo_huixin() + self.skilltype.huixin_up as f32 / 100.0;
        let x = (i[3] as f32 * (1.0 - crit_rate) + i[4] as f32 * crit_rate) as u32;
        DamageResult::new(i, x)
    }

    /// 正向计算伤害 + 反向求导（链式法则）
    /// 对 6 个玩家属性分别计算 ∂Q/∂attr，忽略中间 as u32 截断（连续近似）
    pub fn q_cal_with_derivatives(&self) -> DamageResultWithDerivatives {
        // ---- forward：一次计算全程，复用各段中间结果，避免二次走链 ----
        let y = self.y_cal();
        let i_arr = self.i_cal();
        let g_arr = self.g_cal();
        let h_arr = self.h_cal();
        let crit_rate = self.guo_huixin() + self.skilltype.huixin_up as f32 / 100.0;

        // 与 q_cal() 完全等价的 Q 段计算（不重复调用全链）
        let q = (g_arr[3] as f32 * (1.0 - crit_rate) + h_arr[4] as f32 * crit_rate) as u32;
        let result = DamageResult::new([y, i_arr[1], i_arr[2], g_arr[3], h_arr[4]], q);

        // ---- intermediates (f32, 连续) ----
        let i_hit = i_arr[2] as f32;
        let g_damage = g_arr[3] as f32;
        let h_damage = h_arr[4] as f32;
        let y_val = y as f32;

        let shanghai_buff = 1.0 + self.buff.shanghai_pct / 100.0;
        let hit_up = self.skilltype.hit_up as f32 / 100.0;
        let huajin = self.hostilepile.guo_huajin_with(&self.coeff) as f32;
        let pvp = self.coeff.pvp_global_jianshang;
        let jianshang_bili = self.hostilepile.jianshang_bili as f32 / 100.0;

        let huixiao = self.player.guo_huixinxiaoguo_with(&self.coeff) as f32;
        let yujin_huixiao = self.hostilepile.guo_yujin_huixiao_with(&self.coeff) as f32;
        let buff_huixiao_f = self.buff.huixiao_pct * 1024.0 / 100.0;

        // ---- 公共导数因子 ----
        // dG/dI2（无截断连续近似）
        let dg_di2 = (1.0 + hit_up) * shanghai_buff
            * (y_val / 1024.0)
            * (1.0 - huajin / 1024.0)
            * pvp
            * (1.0 - jianshang_bili);

        // H = G + G * h_factor * yujin_factor
        let h_factor = 0.75 + (huixiao + buff_huixiao_f) / 1024.0
            + self.skilltype.huixiao_up as f32 / 100.0;
        let yujin_factor = 1.0 - yujin_huixiao / 1024.0;
        let dh_dg = 1.0 + h_factor * yujin_factor;

        // dQ/dG = (1-crit) + crit * dH/dG
        let dq_dg = (1.0 - crit_rate) + crit_rate * dh_dg;
        let dq_dh = crit_rate;
        let dq_di2 = dg_di2 * dq_dg;

        // ---- 各属性求导 ----

        // 1. jichu_gongji: B = (jg + js * atk_up) * (1+buff%) + wuqi
        //    dB/d(jg) = 1 + buff%
        let db_d_jg = 1.0 + self.buff.base_atk_pct / 100.0;
        let d_jichu_gongji = dq_di2 * self.skilltype.atk_xishu * db_d_jg;

        // 2. jichu_shuxing: dB/d(js) = atk_up * (1+buff%)
        let db_d_js = self.xinfa.atk_up * (1.0 + self.buff.base_atk_pct / 100.0);
        let d_jichu_shuxing = dq_di2 * self.skilltype.atk_xishu * db_d_js;

        // 3. huixin_dengji: 仅影响会心率
        //    dQ/d(crit) = H - G, d(crit)/d(hd) = 1/huixin_xishu
        let dcrit_d_hd = 1.0 / self.coeff.huixin_xishu;
        let d_huixin_dengji = (h_damage - g_damage) * dcrit_d_hd;

        // 4. huixin_xiaoguo: 仅影响 H 段
        //    huixiao = hx * 1024 / huixiao_xishu
        //    dH/dhuixiao = G * yujin_factor * (1/1024)
        let dhuixiao_d_hx = 1024.0 / self.coeff.huixiao_xishu;
        let dh_dhuixiao = g_damage * yujin_factor / 1024.0;
        let d_huixin_xiaoguo = dq_dh * dh_dhuixiao * dhuixiao_d_hx;

        // 5. pofang_dengji: Y → G → Q
        //    dY/dpofang = 1 - fangyu/1024
        //    dG/dY = I2 * (1+hit_up) * sh_buff / 1024 * ...
        let dpofang_d_pd = 1024.0 / self.coeff.pofang_xishu;
        let fangyu = self.guo_fangyu() as f32;
        let dy_dpofang = 1.0 - fangyu / 1024.0;
        let dg_dy = i_hit * (1.0 + hit_up) * shanghai_buff / 1024.0
            * (1.0 - huajin / 1024.0) * pvp * (1.0 - jianshang_bili);
        let d_pofang_dengji = dq_dg * dg_dy * dy_dpofang * dpofang_d_pd;

        // 6. wuqi_shanghai: 仅走 watk_xishu 路径
        //    I2 += wuqi * watk_xishu / 100
        let di2_d_wq = self.skilltype.watk_xishu as f32 / 100.0;
        let d_wuqi_shanghai = dq_di2 * di2_d_wq;

        DamageResultWithDerivatives {
            result,
            derivatives: DerivativeSet {
                d_jichu_shuxing,
                d_jichu_gongji,
                d_huixin_dengji,
                d_huixin_xiaoguo,
                d_pofang_dengji,
                d_wuqi_shanghai,
            },
        }
    }
}

// ============================================================================
// DamageResult — 五段伤害结果
// ============================================================================

pub struct DamageResult {
    pub y: u32,
    pub i: u32,
    pub b: u32,
    pub g_damage: u32,
    pub h_damage: u32,
    pub q_damage: u32,
}

impl DamageResult {
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

// ============================================================================
// DerivativeSet — 6 属性导数
// ============================================================================

/// 单技能对 6 个玩家属性的偏导 ∂Q/∂attr
#[derive(Debug, Clone)]
pub struct DerivativeSet {
    pub d_jichu_shuxing: f32,
    pub d_jichu_gongji: f32,
    pub d_huixin_dengji: f32,
    pub d_huixin_xiaoguo: f32,
    pub d_pofang_dengji: f32,
    pub d_wuqi_shanghai: f32,
}

/// 带导数的伤害计算结果
pub struct DamageResultWithDerivatives {
    pub result: DamageResult,
    pub derivatives: DerivativeSet,
}
