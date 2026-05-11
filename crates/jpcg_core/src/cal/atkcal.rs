use crate::type_set::hostilepile::HostilepileConfig;
use crate::type_set::player::PlayerConfig;
use crate::type_set::skilltype::Skilltype;
use crate::type_set::xinfa::XinfaConfig;

pub struct JpcgConfig {
    player: PlayerConfig,
    hostilepile: HostilepileConfig,
    skilltype: Skilltype,
    xinfa: XinfaConfig,
}

impl JpcgConfig {
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

    fn guo_fangyu(&self) -> u32 {
        match self.xinfa.xinfa_nom.as_str() {
            "gengu" | "yuanqi" => self.hostilepile.guo_nfangyu(self.skilltype.wushifangyu),
            _ => self.hostilepile.guo_wfangyu(self.skilltype.wushifangyu),
        }
    }

    fn guo_huixin(&self) -> f32 {
        if self.player.guo_huixin() >= self.hostilepile.guo_yujin_huixin() {
            self.player.guo_huixin() - self.hostilepile.guo_yujin_huixin()
        } else {
            0.0
        }
    }

    fn y_cal(&self) -> u32 {
        1024 + self.player.guo_pofang()
            - ((1024.0 + self.player.guo_pofang() as f32) * (self.guo_fangyu() as f32 / 1024.0))
                as u32
    }

    fn b_cal(&self) -> u32 {
        self.player.atk(0.0).total()
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
        let x = (((((i_hit as f32 * (1.0 + self.skilltype.hit_up as f32 / 100.0))
            * (y as f32 / 1024.0)) as u32 as f32
            * (1.0 - (self.hostilepile.guo_huajin() as f32 / 1024.0))) as u32
            as f32
            * 0.9)
            * (1.0 - self.hostilepile.jianshang_bili as f32 / 100.0)) as u32;
        [y, i[1], i[2], x, 0]
    }

    fn h_cal(&self) -> [u32; 5] {
        let i = self.g_cal();
        let g_damage = i[3];
        let x = g_damage
            + (g_damage as f32
                * (0.75
                    + self.player.guo_huixinxiaoguo() as f32 / 1024.0
                    + self.skilltype.huixiao_up as f32 / 100.0)
                * (1.0 - self.hostilepile.guo_yujin_huixiao() as f32 / 1024.0))
                as u32;
        [i[0], i[1], i[2], i[3], x]
    }

    pub fn q_cal(&self) -> DamageResult {
        let i = self.h_cal();
        let x = (i[3] as f32
            * (1.0 - (self.guo_huixin() + self.skilltype.huixin_up as f32 / 100.0))
            + i[4] as f32 * (self.player.guo_huixin() + self.skilltype.huixin_up as f32 / 100.0))
            as u32;
        DamageResult::new(i, x)
    }
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
