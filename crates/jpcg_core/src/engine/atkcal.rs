use crate::type_set::buff::BuffConfig;
use crate::type_set::coefficient::CoefficientConfig;
use crate::type_set::hostilepile::HostilepileConfig;
use crate::type_set::player::PlayerConfig;
use crate::type_set::skilltype::Skilltype;
use crate::type_set::xinfa::XinfaConfig;

/// 游戏内向下取整（抢实检测基准）。
/// 正数域内与 `v as u32` 截断语义一致，显式命名以固定各计算步的取整点。
fn truncate(v: f32) -> f32 {
    v as u32 as f32
}

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
        let wushifangyu_total =
            self.skilltype.wushifangyu + (self.buff.wushi_fangyu_pct * 1024.0 / 100.0) as u32;
        match self.xinfa.xinfa_nom.as_str() {
            "gengu" | "yuanqi" => self
                .hostilepile
                .guo_nfangyu_with(wushifangyu_total, &self.coeff),
            _ => self
                .hostilepile
                .guo_wfangyu_with(wushifangyu_total, &self.coeff),
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
        self.player
            .atk_with_buff(self.xinfa.atk_up, self.buff.base_atk_pct)
            .total()
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

        // 游戏实测截断点（顺序与取整策略源自实际检测结果，勿改动逻辑）
        // ① 技能伤害系数×命中加成×伤害增益 → ×破防系数 → 截断
        let mut x = truncate(
            i_hit as f32
                * (1.0 + self.skilltype.hit_up as f32 / 100.0)
                * shanghai_buff
                * (y as f32 / 1024.0),
        );
        // ② × 化劲减免 → 截断
        x = truncate(x * (1.0 - huajin as f32 / 1024.0));
        // ③ × 全局 PVP 减伤 × (1 - 目标减伤比) → 截断输出
        x = x * pvp * (1.0 - self.hostilepile.jianshang_bili as f32 / 100.0);
        let x = x as u32;
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
                * (1.0 - yujin_huixiao as f32 / 1024.0)) as u32;
        [i[0], i[1], i[2], i[3], x]
    }

    /// Q 段: 期望伤害（最终结果）
    /// crit_rate = 自身会心率 - 目标御劲减免 + 技能增益
    /// buff.huixin_pct 已在 guo_huixin() 中计入，此处不再重复
    /// Dot 技能：返回总期望（各跳之和），并填充 dot_jumps 每跳期望
    pub fn q_cal(&self) -> DamageResult {
        let i = self.h_cal();
        let crit_rate = self.guo_huixin() + self.skilltype.huixin_up as f32 / 100.0;
        let base_q = (i[3] as f32 * (1.0 - crit_rate) + i[4] as f32 * crit_rate) as u32;
        let mut result = DamageResult::new(i, base_q);
        let jumps = self.dot_jump_expected(base_q);
        result.dot_jumps = jumps.clone();
        if !jumps.is_empty() {
            result.q_damage = jumps.iter().sum();
        }
        result
    }

    /// Dot 每跳期望（等比递增）：首跳 = 单次期望，第 k 跳 × (1+dot_up)^(k-1)
    /// 非 Dot 技能返回空集合
    fn dot_jump_expected(&self, base_q: u32) -> Vec<u32> {
        let n = self.skilltype.dot_jump_count();
        if n == 0 {
            return Vec::new();
        }
        let up = self.skilltype.dot_up;
        let mut jumps = Vec::with_capacity(n as usize);
        for k in 0..n {
            let factor = if up > 0.0 { (1.0 + up).powi(k as i32) } else { 1.0 };
            jumps.push((base_q as f32 * factor) as u32);
        }
        jumps
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
        let jumps = self.dot_jump_expected(q);
        let total_q = if jumps.is_empty() {
            q
        } else {
            jumps.iter().sum()
        };
        let mut result = DamageResult::new([y, i_arr[1], i_arr[2], g_arr[3], h_arr[4]], total_q);
        result.dot_jumps = jumps;

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
        let dg_di2 = (1.0 + hit_up)
            * shanghai_buff
            * (y_val / 1024.0)
            * (1.0 - huajin / 1024.0)
            * pvp
            * (1.0 - jianshang_bili);

        // H = G + G * h_factor * yujin_factor
        let h_factor =
            0.75 + (huixiao + buff_huixiao_f) / 1024.0 + self.skilltype.huixiao_up as f32 / 100.0;
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
            * (1.0 - huajin / 1024.0)
            * pvp
            * (1.0 - jianshang_bili);
        let d_pofang_dengji = dq_dg * dg_dy * dy_dpofang * dpofang_d_pd;

        // 6. wuqi_shanghai: 仅走 watk_xishu 路径
        //    I2 += wuqi * watk_xishu / 100
        let di2_d_wq = self.skilltype.watk_xishu as f32 / 100.0;
        let d_wuqi_shanghai = dq_di2 * di2_d_wq;

        // ---- Dot 等比和因子 ----
        // 总期望 = Σ 首跳×(1+u)^(k-1)，∂Q/∂attr 整体按等比和缩放
        //（dot_up=0 时退化为跳数倍；非 Dot 技能因子为 1）
        let dot_n = self.skilltype.dot_jump_count() as f32;
        let dot_up = self.skilltype.dot_up;
        let dot_factor = if dot_n == 0.0 {
            1.0
        } else if dot_up == 0.0 {
            dot_n
        } else {
            ((1.0 + dot_up).powi(dot_n as i32) - 1.0) / dot_up
        };

        DamageResultWithDerivatives {
            result,
            derivatives: DerivativeSet {
                d_jichu_shuxing: d_jichu_shuxing * dot_factor,
                d_jichu_gongji: d_jichu_gongji * dot_factor,
                d_huixin_dengji: d_huixin_dengji * dot_factor,
                d_huixin_xiaoguo: d_huixin_xiaoguo * dot_factor,
                d_pofang_dengji: d_pofang_dengji * dot_factor,
                d_wuqi_shanghai: d_wuqi_shanghai * dot_factor,
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
    /// Dot 每跳期望伤害（非 Dot 技能为空；q_damage 为各跳之和）
    pub dot_jumps: Vec<u32>,
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
            dot_jumps: Vec::new(),
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

// ============================================================================
// 金标准基准测试
// 数值取自 2026-08-09 重构前（jpcg_core cd5b694 时代代码）在相同输入下的输出，
// 用于锁定「P1 引用化/P2 截断显式化」前后行为逐位一致。
// ============================================================================
#[cfg(test)]
mod golden_tests {
    use crate::engine::atkcal::JpcgConfig;
    use crate::type_set::buff::BuffConfig;
    use crate::type_set::coefficient::CoefficientConfig;
    use crate::type_set::hostilepile::HostilepileConfig;
    use crate::type_set::player::PlayerConfig;
    use crate::type_set::skilltype::Skilltype;
    use crate::type_set::xinfa::XinfaConfig;

    fn skill(
        name: &str,
        b1: u32,
        b2: u32,
        atk_xishu: f32,
        hit_up: u32,
        watk_xishu: u32,
        wushifangyu: u32,
    ) -> Skilltype {
        Skilltype {
            skill_name: name.to_string(),
            base_damage1: b1,
            base_damage2: b2,
            atk_xishu,
            hit_up,
            watk_xishu,
            wushifangyu,
            ..Skilltype::default()
        }
    }

    fn player() -> PlayerConfig {
        PlayerConfig {
            jcsx: "gengu".into(),
            jichu_shuxing: 18888,
            jichu_gongji: 4666,
            huixin_dengji: 33000,
            huixin_xiaoguo: 22000,
            pofang_dengji: 25000,
            wuqi_shanghai: 2800,
            zuizhong_gongji: 0,
        }
    }

    fn hostile() -> HostilepileConfig {
        HostilepileConfig {
            waigong_fangyu: 21000,
            neigong_fangyu: 21000,
            yujin_dengji: 8500,
            huajin_dengji: 35000,
            jianshang_bili: 35,
            target_hp: 200,
        }
    }

    fn xinfa() -> XinfaConfig {
        XinfaConfig {
            profession: "mowen".into(),
            xinfa_name: "莫问".into(),
            xinfa_nom: "根骨".into(),
            atk_up: 1.96,
            pofang_up: 2.0,
            huixin_up: 0.0,
        }
    }

    fn buff_full() -> BuffConfig {
        BuffConfig {
            base_atk_pct: 10.0,
            huixin_pct: 5.0,
            huixiao_pct: 8.0,
            pofang_pct: 3.0,
            wushi_fangyu_pct: 0.0,
            shanghai_pct: 6.0,
            mode_is_point: false,
        }
    }

    struct Golden {
        y: u32,
        b: u32,
        i: u32,
        n: u32,
        h: u32,
        q: u32,
        d_js: f32,
        d_jg: f32,
        d_hxd: f32,
        d_hxg: f32,
        d_pf: f32,
        d_wq: f32,
    }

    fn assert_golden(tag: &str, sk: &Skilltype, buff: &BuffConfig, g: Golden) {
        let p = player();
        let h = hostile();
        let x = xinfa();
        let c = CoefficientConfig::default();
        let cfg = JpcgConfig::new_with_config(&p, &h, sk, &x, buff, &c);
        let d = cfg.q_cal();
        assert_eq!(d.y, g.y, "{tag}: Y 段不匹配");
        assert_eq!(d.b, g.b, "{tag}: B 段不匹配");
        assert_eq!(d.i, g.i, "{tag}: I 段不匹配");
        assert_eq!(d.g_damage, g.n, "{tag}: N 段不匹配");
        assert_eq!(d.h_damage, g.h, "{tag}: H 段不匹配");
        assert_eq!(d.q_damage, g.q, "{tag}: Q 段不匹配");

        let dr = cfg.q_cal_with_derivatives().derivatives;
        let eps = 1e-6;
        assert!(
            (dr.d_jichu_shuxing - g.d_js).abs() < eps,
            "{tag}: dJS 不匹配: got {} want {}",
            dr.d_jichu_shuxing,
            g.d_js
        );
        assert!(
            (dr.d_jichu_gongji - g.d_jg).abs() < eps,
            "{tag}: dJG 不匹配: got {} want {}",
            dr.d_jichu_gongji,
            g.d_jg
        );
        assert!(
            (dr.d_huixin_dengji - g.d_hxd).abs() < eps,
            "{tag}: dHXD 不匹配: got {} want {}",
            dr.d_huixin_dengji,
            g.d_hxd
        );
        assert!(
            (dr.d_huixin_xiaoguo - g.d_hxg).abs() < eps,
            "{tag}: dHXG 不匹配: got {} want {}",
            dr.d_huixin_xiaoguo,
            g.d_hxg
        );
        assert!(
            (dr.d_pofang_dengji - g.d_pf).abs() < eps,
            "{tag}: dPF 不匹配: got {} want {}",
            dr.d_pofang_dengji,
            g.d_pf
        );
        assert!(
            (dr.d_wuqi_shanghai - g.d_wq).abs() < eps,
            "{tag}: dWQ 不匹配: got {} want {}",
            dr.d_wuqi_shanghai,
            g.d_wq
        );
    }

    #[test]
    fn golden_gong_default() {
        let sk = skill("宫", 160, 200, 2.609375, 0, 0, 0);
        assert_golden(
            "gong_default",
            &sk,
            &BuffConfig::default(),
            Golden {
                y: 975,
                b: 116260,
                i: 44486,
                n: 23524,
                h: 47202,
                q: 26458,
                d_js: 1.163984,
                d_jg: 0.593869,
                d_hxd: 0.119766,
                d_hxg: 0.038300,
                d_pf: 0.105450,
                d_wq: 0.000000,
            },
        );
    }

    /// DOT 每跳等比递增：首跳 = 同配置普通技能 Q（宫 default 金标准 q=26458），
    /// 第 k 跳 × 1.08^k（k=0..5），共 6 跳（18s / 3s）。
    /// 期望由等比公式手工推算，锁定引擎行为；数值待正式服数据校准。
    #[test]
    fn golden_gong_dot() {
        let mut sk = skill("宫(6跳dot)", 160, 200, 2.609375, 0, 0, 0);
        sk.dot_flag = 1;
        sk.dot_interval = 3;
        sk.dot_duration = 18;
        sk.dot_up = 0.08;
        let p = player();
        let h = hostile();
        let x = xinfa();
        let b = BuffConfig::default();
        let c = CoefficientConfig::default();
        let cfg = JpcgConfig::new_with_config(&p, &h, &sk, &x, &b, &c);
        let d = cfg.q_cal();
        let expect_jumps: [u32; 6] = [
            26458,          // k=0: 26458 × 1.08^0
            28574,          // k=1: 26458 × 1.08 = 28574.64
            30860,          // k=2: 26458 × 1.08^2 = 30860.61
            33329,          // k=3: 26458 × 1.08^3 = 33329.46
            35995,          // k=4: 26458 × 1.08^4 = 35995.82
            38875,          // k=5: 26458 × 1.08^5 = 38875.48
        ];
        assert_eq!(d.dot_jumps, expect_jumps, "DOT 每跳不匹配");
        assert_eq!(
            d.q_damage,
            expect_jumps.iter().sum::<u32>(),
            "DOT 总期望不匹配"
        );

        // 导数链 × 等比和因子 Σ_{k=0..5} 1.08^k = (1.08^6-1)/0.08 = 7.3359290...
        let factor = 7.3359290368;
        let dr = cfg.q_cal_with_derivatives().derivatives;
        let want_pairs = [
            (dr.d_jichu_shuxing, 1.163984 * factor),
            (dr.d_jichu_gongji, 0.593869 * factor),
            (dr.d_huixin_dengji, 0.119766 * factor),
            (dr.d_huixin_xiaoguo, 0.038300 * factor),
            (dr.d_pofang_dengji, 0.105450 * factor),
        ];
        for (got, want) in want_pairs {
            assert!(
                (got - want).abs() < 1e-3,
                "DOT 导数不匹配: got {got} want {want}"
            );
        }
        assert_eq!(dr.d_wuqi_shanghai, 0.0);
    }

    #[test]
    fn golden_gong_buff() {
        let sk = skill("宫", 160, 200, 2.609375, 0, 0, 0);
        assert_golden(
            "gong_buff",
            &sk,
            &buff_full(),
            Golden {
                y: 1001,
                b: 127139,
                i: 48655,
                n: 27996,
                h: 58319,
                q: 33269,
                d_js: 1.472244,
                d_jg: 0.751145,
                d_hxd: 0.153377,
                d_hxg: 0.063971,
                d_pf: 0.129154,
                d_wq: 0.000000,
            },
        );
    }

    #[test]
    fn golden_zheng_default() {
        let sk = skill("徵(豪情)", 190, 210, 1.7760416666666667, 20, 0, 0);
        assert_golden(
            "zheng_default",
            &sk,
            &BuffConfig::default(),
            Golden {
                y: 975,
                b: 79208,
                i: 44486,
                n: 19233,
                h: 38592,
                q: 21632,
                d_js: 0.950703,
                d_jg: 0.485052,
                d_hxd: 0.097920,
                d_hxg: 0.031313,
                d_pf: 0.086212,
                d_wq: 0.000000,
            },
        );
    }

    #[test]
    fn golden_zheng_buff() {
        let sk = skill("徵(豪情)", 190, 210, 1.7760416666666667, 20, 0, 0);
        assert_golden(
            "zheng_buff",
            &sk,
            &buff_full(),
            Golden {
                y: 1001,
                b: 86613,
                i: 48655,
                n: 22887,
                h: 47676,
                q: 27198,
                d_js: 1.202480,
                d_jg: 0.613510,
                d_hxd: 0.125385,
                d_hxg: 0.052297,
                d_pf: 0.105583,
                d_wq: 0.000000,
            },
        );
    }

    #[test]
    fn golden_wei_default() {
        let sk = skill("剑·徵(削竹)", 330, 350, 3.15625, 0, 100, 90);
        assert_golden(
            "wei_default",
            &sk,
            &BuffConfig::default(),
            Golden {
                y: 988,
                b: 143548,
                i: 44486,
                n: 29433,
                h: 59059,
                q: 33104,
                d_js: 1.426705,
                d_jg: 0.727911,
                d_hxd: 0.149851,
                d_hxg: 0.047920,
                d_pf: 0.131832,
                d_wq: 0.230625,
            },
        );
    }

    #[test]
    fn golden_wei_buff() {
        let sk = skill("剑·徵(削竹)", 330, 350, 3.15625, 0, 100, 90);
        assert_golden(
            "wei_buff",
            &sk,
            &buff_full(),
            Golden {
                y: 1014,
                b: 156707,
                i: 48655,
                n: 34956,
                h: 72817,
                q: 41540,
                d_js: 1.803926,
                d_jg: 0.920370,
                d_hxd: 0.191504,
                d_hxg: 0.079875,
                d_pf: 0.161185,
                d_wq: 0.265093,
            },
        );
    }
}
