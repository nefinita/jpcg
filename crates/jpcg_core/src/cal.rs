mod atkcal;

use crate::{
    cal::atkcal::atkout,
    io::{TomlConfig, toml_input},
    log::{error, success},
    type_set::{hostilepile::HostilepileConfig, player::PlayerConfig, xinfa::XinfaConfig},
};
use serde::{Serialize};

pub fn start_calculation(
    player: PlayerConfig,
    hostilepile: HostilepileConfig,
    xinfa: XinfaConfig,
) -> Vec<CalculateResult> {
    success("Calculation started!");

    let current_dir = match std::env::current_exe() {
        Ok(path) => path
            .parent()
            .expect("Failed to get parent directory")
            .to_path_buf(),
        Err(e) => {
            error(format!("Failed to get current exe path: {}", e).as_str());
            return vec![CalculateResult::default()];
        }
    };
    let file_path = current_dir
        .join("data")
        .join("pvp36500")
        .join(xinfa.xinfa_name.clone());
    let content = toml_input(file_path.to_str().unwrap());
    let skill_table: TomlConfig = toml::from_str(&content).unwrap();
    call_back(&skill_table, &player, &hostilepile)
}

#[derive(Default, Serialize)]
pub struct CalculateResult {
    pub skill_name: String,
    pub y: u32,
    pub b: u32,
    pub i: u32,
    pub n: u32,
    pub h: u32,
    pub q: u32,
}

impl CalculateResult {
    pub fn new(skill_name: String, y: u32, b: u32, i: u32, n: u32, h: u32, q: u32) -> Self {
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

fn call_back(
    toml_config: &TomlConfig,
    player: &PlayerConfig,
    hostilepile: &HostilepileConfig,
) -> Vec<CalculateResult> {
    let mut results = Vec::new();
    for skill in &toml_config.skill {
        let damage_result = atkout(player, hostilepile, skill, &toml_config.xinfa, "pvp36500");
        let calculate_result = CalculateResult::new(
            skill.skill_name.clone(),
            damage_result.y,
            damage_result.b,
            damage_result.i,
            damage_result.g_damage,
            damage_result.h_damage,
            damage_result.q_damage,
        );
        calculate_result.get_message();
        results.push(calculate_result);
    }
    results
}
