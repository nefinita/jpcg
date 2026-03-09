use crate::{
    log::success,
    type_set::{hostilepile::HostilepileConfig, player::PlayerConfig, xinfa::XinfaConfig},
};

pub fn start_calculation(
    player: PlayerConfig,
    hostilepile: HostilepileConfig,
    xinfa: XinfaConfig,
) -> Vec<CalculateResult> {
    success("Calculation started!");
    // 这里可以调用具体的计算逻辑
    vec![
        CalculateResult::new("技能1".to_string(), 100, 2000, 300, 400, 500, 600),
        CalculateResult::new("技能2".to_string(), 150, 2500, 350, 450, 550, 650),
    ]
}

pub struct CalculateResult {
    skill_name: String,
    y: u16,
    b: u32,
    i: u32,
    n: u32,
    h: u32,
    q: u32,
}

impl CalculateResult {
    pub fn new(skill_name: String, y: u16, b: u32, i: u32, n: u32, h: u32, q: u32) -> Self {
        CalculateResult {
            skill_name,
            y,
            b,
            i,
            n,
            h,
            q,
        }
    }

    pub fn get_message(&self) {
        success(&format!(
            "技能: {}, Y: {}, B: {}, I: {}, N: {}, H: {}, Q: {}",
            self.skill_name, self.y, self.b, self.i, self.n, self.h, self.q
        ));
    }
}
