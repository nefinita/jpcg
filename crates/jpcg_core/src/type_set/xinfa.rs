use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct XinfaConfig {
    pub xinfa_name: String,
    pub xinfa_nom: String,
    pub atk_up: f32,
    pub pofang_up: f32,
    pub huixin_up: f32,
}

impl XinfaConfig {
    pub fn new(
        xinfa_name: String,
        xinfa_nom: String,
        atk_up: f32,
        pofang_up: f32,
        huixin_up: f32,
    ) -> Self {
        Self {
            xinfa_name,
            xinfa_nom,
            atk_up,
            pofang_up,
            huixin_up,
        }
    }
    pub fn default() -> Self {
        Self {
            xinfa_name: "莫问".to_string(),
            xinfa_nom: "根骨".to_string(),
            atk_up: 0.0,
            pofang_up: 0.0,
            huixin_up: 0.0,
        }
    }
}

pub fn get_xinfa_list() -> HashMap<String, XinfaConfig> {
    todo!()
}
