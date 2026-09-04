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
    //攻击方玩家
    player: &'a PlayerConfig,
    //木桩（被攻击的玩家）
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
        // 根骨/元气职业为内功（数据文件 xinfa_nom 为中文，测试兼容英文）
        let is_neigong = matches!(
            self.xinfa.xinfa_nom.as_str(),
            "根骨" | "元气" | "gengu" | "yuanqi"
        );
        if is_neigong {
            self.hostilepile
                .guo_nfangyu_with(wushifangyu_total, self.coeff)
        } else {
            self.hostilepile
                .guo_wfangyu_with(wushifangyu_total, self.coeff)
        }
    }

    pub fn guo_huixin(&self) -> f32 {
        let player_crit = self.player.guo_huixin_with(self.coeff) + self.buff.huixin_pct / 100.0;
        let enemy_crit_reduce = self.hostilepile.guo_yujin_huixin_with(self.coeff);
        if player_crit >= enemy_crit_reduce {
            player_crit - enemy_crit_reduce
        } else {
            0.0
        }
    }

    fn y_cal(&self) -> u32 {
        let pofang = self.player.guo_pofang_with(self.coeff)
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
        let huajin = self.hostilepile.guo_huajin_with(self.coeff);
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
        let huixiao = self.player.guo_huixinxiaoguo_with(self.coeff);
        let yujin_huixiao = self.hostilepile.guo_yujin_huixiao_with(self.coeff);
        let buff_huixiao = self.buff.huixiao_pct * 1024.0 / 100.0;
        let x = g_damage
            + (g_damage as f32
                * (0.75
                    + (huixiao as f32 + buff_huixiao) / 1024.0
                    + self.skilltype.huixiao_up as f32 / 100.0)
                * (1.0 - yujin_huixiao as f32 / 1024.0)) as u32;
        [i[0], i[1], i[2], i[3], x]
    }

    /// 追加真伤公式（core 实现，combo 传入 hp）：
    /// 目标已损失生命 = max_hp - 结算后当前血量（封顶 max_hp），× lost_hp_zhenshishanghai 系数。
    /// 语义 A（结算后）：current_hp_after 为本步伤害扣完后剩余血量。
    pub fn lost_hp_append(&self, max_hp: u32, current_hp_after: u32) -> f64 {
        if self.skilltype.lost_hp_zhenshishanghai <= 0.0 {
            return 0.0;
        }
        let lost = (max_hp as f64 - current_hp_after as f64).max(0.0);
        lost * self.skilltype.lost_hp_zhenshishanghai as f64
    }

    /// 期望通道 + 可选 hp：q_cal() 基础上结算追加真伤。
    /// 未提供 hp（0）时真伤恒 0（调用方负责 target_hp 满血回退语义）。
    pub fn q_cal_with_hp(&self, current_hp: Option<u32>, max_hp: Option<u32>) -> DamageResult {
        let mut result = self.q_cal();
        if let (Some(current), Some(max)) = (current_hp, max_hp) {
            let after = current.saturating_sub(result.q_damage);
            result.lost_hp_zhenshi_damage = self.lost_hp_append(max, after);
        }
        result
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
            let factor = if up > 0.0 {
                (1.0 + up).powi(k as i32)
            } else {
                1.0
            };
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
        let huajin = self.hostilepile.guo_huajin_with(self.coeff) as f32;
        let pvp = self.coeff.pvp_global_jianshang;
        let jianshang_bili = self.hostilepile.jianshang_bili as f32 / 100.0;

        let huixiao = self.player.guo_huixinxiaoguo_with(self.coeff) as f32;
        let yujin_huixiao = self.hostilepile.guo_yujin_huixiao_with(self.coeff) as f32;
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
    /// 追加真伤（目标已损失生命 × 系数，无视防御，确定性）。
    /// 仅显式提供 hp（max/current）时结算；否则恒 0（由连招层维护回退语义）。
    pub lost_hp_zhenshi_damage: f64,
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
            lost_hp_zhenshi_damage: 0.0,
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
// 输入为真实面板属性（2026-08-13 用户提供：基础 21371/攻击 64329/会心 61877/
// 会效 2925/破防 109160，目标外防 15176/内防 21388/御劲 5047/化劲 59402，pvp 0.9），
// 期望值由引擎输出回填（examples/python_demo/quick_calc_test.py 全量一致），
// 锁定计算行为；待用户木桩实测后校准关键值。
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
            jichu_shuxing: 21371,
            jichu_gongji: 64329,
            huixin_dengji: 61877,
            huixin_xiaoguo: 2925,
            pofang_dengji: 109160,
            wuqi_shanghai: 0,
            zuizhong_gongji: 0,
        }
    }

    fn hostile() -> HostilepileConfig {
        HostilepileConfig {
            waigong_fangyu: 15176,
            neigong_fangyu: 21388,
            yujin_dengji: 5047,
            huajin_dengji: 59402,
            jianshang_bili: 0,
            target_hp: 2_000_000,
            max_hp: 0,
            current_hp: 0,
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
        let dr = cfg.q_cal_with_derivatives().derivatives;
        assert_eq!(d.y, g.y, "{tag}: Y 段不匹配");
        assert_eq!(d.b, g.b, "{tag}: B 段不匹配");
        assert_eq!(d.i, g.i, "{tag}: I 段不匹配");
        assert_eq!(d.g_damage, g.n, "{tag}: N 段不匹配");
        assert_eq!(d.h_damage, g.h, "{tag}: H 段不匹配");
        assert_eq!(d.q_damage, g.q, "{tag}: Q 段不匹配");

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
                y: 1299,
                b: 277337,
                i: 106216,
                n: 75138,
                h: 129108,
                q: 90651,
                d_js: 1.6717374,
                d_jg: 0.85292727,
                d_hxd: 0.27298522,
                d_hxg: 0.26957446,
                d_pf: 0.27055234,
                d_wq: 0.000000,
            },
        );
    }

    /// DOT 金标准（真实属性）：普通 6 跳每跳相等（18s/3s），疏曲 9 跳等比 1.12（18s/2s）。
    /// 期望值由引擎回填（quick_calc_test.py 输出一致），待木桩实测校准。
    #[test]
    fn golden_shang_dot() {
        let mut sk = skill("商（dot）", 58, 58, 0.20833333, 0, 0, 0);
        sk.dot_flag = 1;
        sk.dot_interval = 3.0;
        sk.dot_duration = 18.0;
        let p = player();
        let h = hostile();
        let x = xinfa();
        let b = BuffConfig::default();
        let c = CoefficientConfig::default();
        let cfg = JpcgConfig::new_with_config(&p, &h, &sk, &x, &b, &c);
        let d = cfg.q_cal();
        assert_eq!(d.dot_jumps.len(), 6, "普通 dot 应为 6 跳");
        assert!(
            d.dot_jumps.iter().all(|j| *j == d.dot_jumps[0]),
            "非递增条目每跳应相等: {:?}",
            d.dot_jumps
        );
        assert_eq!(
            d.q_damage,
            d.dot_jumps.iter().sum::<u32>(),
            "总期望 = Σ 每跳"
        );
    }

    #[test]
    fn golden_shang_dot_shuqu() {
        let mut sk = skill("商（dot）疏曲", 58, 58, 0.20833333, 0, 0, 0);
        sk.dot_flag = 1;
        sk.dot_interval = 2.0;
        sk.dot_duration = 18.0;
        sk.dot_up = 0.12;
        let p = player();
        let h = hostile();
        let x = xinfa();
        let b = BuffConfig::default();
        let c = CoefficientConfig::default();
        let cfg = JpcgConfig::new_with_config(&p, &h, &sk, &x, &b, &c);
        let d = cfg.q_cal();
        assert_eq!(d.dot_jumps.len(), 9, "疏曲 dot 应为 9 跳");
        let first = d.dot_jumps[0];
        for (k, j) in d.dot_jumps.iter().enumerate() {
            let expect = (first as f32 * 1.12_f32.powi(k as i32)) as u32;
            assert!(
                (*j as i64 - expect as i64).abs() <= 1,
                "第{}跳 {j} != {expect} (±1)",
                k + 1
            );
        }
        assert_eq!(
            d.q_damage,
            d.dot_jumps.iter().sum::<u32>(),
            "总期望 = Σ 每跳"
        );
    }

    #[test]
    fn golden_gong_buff() {
        let sk = skill("宫", 160, 200, 2.609375, 0, 0, 0);
        assert_golden(
            "gong_buff",
            &sk,
            &buff_full(),
            Golden {
                y: 1325,
                b: 305051,
                i: 116837,
                n: 89359,
                h: 160043,
                q: 113211,
                d_js: 2.0878966,
                d_jg: 1.0652533,
                d_hxd: 0.35752618,
                d_hxg: 0.3763607,
                d_pf: 0.33125117,
                d_wq: 0.000000,
            },
        );
    }

    #[test]
    fn golden_zheng_default() {
        let sk = skill("徵(豪情)", 190, 210, 1.776_041_6, 20, 0, 0);
        assert_golden(
            "zheng_default",
            &sk,
            &BuffConfig::default(),
            Golden {
                y: 1299,
                b: 188844,
                i: 106216,
                n: 61395,
                h: 105494,
                q: 74071,
                d_js: 1.3654193,
                d_jg: 0.69664246,
                d_hxd: 0.2230568,
                d_hxg: 0.22026837,
                d_pf: 0.22106907,
                d_wq: 0.000000,
            },
        );
    }

    #[test]
    fn golden_zheng_buff() {
        let sk = skill("徵(豪情)", 190, 210, 1.776_041_6, 20, 0, 0);
        assert_golden(
            "zheng_buff",
            &sk,
            &buff_full(),
            Golden {
                y: 1325,
                b: 207707,
                i: 116837,
                n: 73012,
                h: 130766,
                q: 92501,
                d_js: 1.7053239,
                d_jg: 0.8700632,
                d_hxd: 0.29212505,
                d_hxg: 0.3075107,
                d_pf: 0.27065578,
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
                y: 1315,
                b: 335584,
                i: 106216,
                n: 92039,
                h: 158149,
                q: 111042,
                d_js: 2.0470083,
                d_jg: 1.044392,
                d_hxd: 0.33439046,
                d_hxg: 0.33021063,
                d_pf: 0.3314853,
                d_wq: 0.33089647,
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
                y: 1341,
                b: 369106,
                i: 116837,
                n: 109429,
                h: 195989,
                q: 138638,
                d_js: 2.555976,
                d_jg: 1.3040693,
                d_hxd: 0.43782845,
                d_hxg: 0.4608912,
                d_pf: 0.4058407,
                d_wq: 0.37560952,
            },
        );
    }

    /// 无质金标准：伤害固定为期望 Q（含会心加权），与普通技能输出完全一致。
    /// 相依（莫问无质技能）数据以 has_critical_strike = true 标记。
    #[test]
    fn golden_gong_wuzhi() {
        let mut sk = skill("宫", 160, 200, 2.609375, 0, 0, 0);
        sk.has_critical_strike = true;
        assert_golden(
            "gong_wuzhi",
            &sk,
            &BuffConfig::default(),
            Golden {
                y: 1299,
                b: 277337,
                i: 106216,
                n: 75138,
                h: 129108,
                q: 90651,
                d_js: 1.6717374,
                d_jg: 0.85292727,
                d_hxd: 0.27298522,
                d_hxg: 0.26957446,
                d_pf: 0.27055234,
                d_wq: 0.000000,
            },
        );
    }
}
