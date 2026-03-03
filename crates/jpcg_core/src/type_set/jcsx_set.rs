use std::collections::HashMap;

use crate::log::{error, info, warn};
use crate::io::toml_input;

fn data_load() {
    let content = toml_input("data/atk_config");
    info(format!("Loaded config content:\n{}", content).as_str());
}

pub struct JcsxConfig {
    pub jcsx_name: String,
    pub jcsx_atk: f32,
    pub jcsx_pofang: f32,
    pub jcsx_huixin: f32,
}

pub fn get_jcsx_list() -> HashMap<String, JcsxConfig> {
    todo!()
}