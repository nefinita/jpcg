use crate::log::{debug, info};
use crate::type_set::hostilepile::HostilepileConfig;
use crate::type_set::jcsx_set;
use crate::type_set::player::PlayerConfig;
use crate::type_set::skilltype::Skilltype;
use crate::type_set::xinfa;

pub fn atkout(
    playerdata: &PlayerConfig,
    hostilepile: &HostilepileConfig,
    skilltype: &Skilltype,
    xinfadata: &xinfa::XinfaConfig,
    jc: &str,
) -> DamageResult {
    info(format!("计算技能：{}", skilltype.skill_name).as_str());
    let guo_fangyu = match xinfadata.xinfa_nom.as_str() {
        "gengu" | "yuanqi" => hostilepile.guo_nfangyu(skilltype.wushifangyu),
        _ => hostilepile.guo_wfangyu(skilltype.wushifangyu),
    };
    info(format!("使用防御: {}", guo_fangyu).as_str());
    let guo_huixin = if playerdata.guo_huixin() - hostilepile.guo_yujin_huixin() >= 0.0 {
        playerdata.guo_huixin() - hostilepile.guo_yujin_huixin()
    } else {
        0.0
    };
    info(format!("使用会心：{}", guo_huixin).as_str());
    let y = ycal(playerdata.guo_pofang(), guo_fangyu);
    let b_damage = playerdata.atk(0.0).total();
    let i_damage = iatkc(
        skilltype.base_atk(),
        b_damage,
        skilltype.atk_xishu,
        playerdata.wuqi_shanghai,
        skilltype.watk_xishu as f32 / 100.0,
    );
    let g_damage = gatkc(
        i_damage,
        y,
        hostilepile.guo_huajin(),
        skilltype.hit_up as f32 / 100.0,
        hostilepile.jianshang_bili,
    );
    let h_damage = hatkc(
        g_damage,
        playerdata.guo_huixinxiaoguo(),
        skilltype.huixiao_up as f32 / 100.0,
        hostilepile.guo_yujin_huixiao(),
    );
    let q_damage = qatkc(
        g_damage,
        guo_huixin,
        h_damage,
        skilltype.huixin_up as f32 / 100.0,
    );
    debug(
        format!(
            "name:{}, y:{}, b:{}, i:{} g:{}, h:{}, q:{}",
            skilltype.skill_name, y, b_damage, i_damage, g_damage, h_damage, q_damage
        )
        .as_str(),
    );
    DamageResult::new(y, i_damage, b_damage, g_damage, h_damage, q_damage)
}

fn iatkc(base_atk: u32, atk: u32, atk_xishu: f32, watk: u32, watk_xishu: f32) -> u32 {
    base_atk + (atk as f32 * atk_xishu) as u32 + (watk as f32 * watk_xishu) as u32
}

fn gatkc(i_hit: u32, y: u32, guo_huajin: u32, hit_up: f32, jianshang: u32) -> u32 {
    (((((i_hit as f32 * (1.0 + hit_up)) * (y as f32 / 1024.0)) as u32 as f32
        * (1.0 - (guo_huajin as f32 / 1024.0))) as u32 as f32
        * 0.9)
        * (1.0 - jianshang as f32 / 100.0)) as u32
}

fn hatkc(g_damage: u32, guo_huixinxiaoguo: u32, e_huxin_xiaoguo: f32, guo_yujin: u32) -> u32 {
    g_damage
        + (g_damage as f32
            * (0.75 + guo_huixinxiaoguo as f32 / 1024.0 + e_huxin_xiaoguo)
            * (1.0 - guo_yujin as f32 / 1024.0)) as u32
}

fn qatkc(g_damage: u32, guo_huixin: f32, h_damage: u32, e_huixin: f32) -> u32 {
    (g_damage as f32 * (1.0 - (guo_huixin + e_huixin)) + h_damage as f32 * (guo_huixin + e_huixin))
        as u32
}

fn ycal(gpofang: u32, gfangyu: u32) -> u32 {
    1024 + gpofang - ((1024.0 + gpofang as f32) * (gfangyu as f32 / 1024.0)) as u32
}

pub fn no_hatkc(
    playerdata: &PlayerConfig,
    hostilepile: &HostilepileConfig,
    skilltype: &Skilltype,
    xinfadata: &xinfa::XinfaConfig,
    jc: &str,
) -> DamageResult {
    let guo_fangyu = match xinfadata.xinfa_nom.as_str() {
        "gengu" | "yuanqi" => hostilepile.guo_nfangyu(skilltype.wushifangyu),
        _ => hostilepile.guo_wfangyu(skilltype.wushifangyu),
    };
    //debug(format!("使用防御: {}", guo_fangyu).as_str());
    let guo_huixin = if playerdata.guo_huixin() - hostilepile.guo_yujin_huixin() >= 0.0 {
        playerdata.guo_huixin() - hostilepile.guo_yujin_huixin()
    } else {
        0.0
    };
    //debug(format!("使用会心：{}", guo_huixin).as_str());
    let y = ycal(playerdata.guo_pofang(), guo_fangyu);
    let b_damage = playerdata.atk(0.0).total();
    let i_damage = iatkc(
        skilltype.base_atk(),
        b_damage,
        skilltype.atk_xishu,
        playerdata.wuqi_shanghai,
        skilltype.watk_xishu as f32 / 100.0,
    );
    let g_damage = {
        (((((i_damage as f32 * (1.0 + skilltype.hit_up as f32 / 100.0)) * (y as f32 / 1024.0))
            as u32 as f32
            * (1.0 - (hostilepile.guo_huajin() as f32 / 1024.0))) as u32 as f32
            * (1.0 + guo_huixin) as u32 as f32
            + (1.0 + playerdata.guo_huixinxiaoguo() as f32 / 1024.0) as u32 as f32 * 0.9)
            * (1.0 - hostilepile.jianshang_bili as f32 / 100.0)) as u32
    };
    //debug(format!("y:{}, b:{}, i:{}, g:{}", y, b_damage, i_damage, g_damage).as_str());
    DamageResult::new(y, i_damage, b_damage, g_damage, g_damage, g_damage)
}

pub struct DamageResult {
    pub y: u32,
    pub i: u32,
    pub b: u32,
    pub g_damage: u32,
    pub h_damage: u32,
    pub q_damage: u32,
}

impl DamageResult {
    pub fn new(
        y: u32,
        i: u32,
        b: u32,
        g_damage: u32,
        h_damage: u32,
        q_damage: u32,
    ) -> DamageResult {
        DamageResult {
            y,
            i,
            b,
            g_damage,
            h_damage,
            q_damage,
        }
    }
}

//对gatk求全微分
pub fn cal_attr_change(
    playerdata: &PlayerConfig,
    hostilepile: &HostilepileConfig,
    skilltype: &Skilltype,
    xinfadata: &xinfa::XinfaConfig,
    jc: &str,
) {
    let base_gatk = atkout(playerdata, hostilepile, skilltype, xinfadata, jc).q_damage;

    //基础攻击微分
    let mut playerdata_delta = playerdata.clone();
    playerdata_delta.jichu_gongji += 849;
    let new_gatk = atkout(&playerdata_delta, hostilepile, skilltype, xinfadata, jc).q_damage;
    let diff1 = new_gatk - base_gatk;

    //基础属性微分
    let mut playerdata_delta = playerdata.clone();
    playerdata_delta.jichu_shuxing += 359;
    playerdata_delta.pofang_dengji += (359.0 * xinfadata.pofang_up) as u32;
    let new_gatk = atkout(&playerdata_delta, hostilepile, skilltype, xinfadata, jc).q_damage;
    let diff2 = new_gatk - base_gatk;

    //破防微分
    let mut playerdata_delta = playerdata.clone();
    playerdata_delta.pofang_dengji += 2801;
    let new_gatk = atkout(&playerdata_delta, hostilepile, skilltype, xinfadata, jc).q_damage;
    let diff3 = new_gatk - base_gatk;

    //会心微分
    let mut playerdata_delta = playerdata.clone();
    playerdata_delta.huixin_dengji += 2801;
    let new_gatk = atkout(&playerdata_delta, hostilepile, skilltype, xinfadata, jc).q_damage;
    let diff4 = new_gatk - base_gatk;

    //会效微分
    let mut playerdata_delta = playerdata.clone();
    playerdata_delta.huixin_xiaoguo += 2801;
    let new_gatk = atkout(&playerdata_delta, hostilepile, skilltype, xinfadata, jc).q_damage;
    let diff5 = new_gatk - base_gatk;
    info(format!("计算技能：{} : 基础攻击微分：{}，基础属性微分: {}，破防微分: {}，会心微分: {}，会效微分: {}。", skilltype.skill_name, diff1, diff2, diff3, diff4, diff5).as_str());
}
