use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct XinfaConfig {
    pub xinfa_nom: String,
    pub atk_up: f32,
    pub pofang_up: f32,
    pub huixin_up: f32,
}

pub fn get_xinfa_list() -> HashMap<String, XinfaConfig> {
    todo!()
}