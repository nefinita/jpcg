use crate::log::{error, info, warn};
use crate::type_set::skilltype::Skilltype;
use crate::type_set::xinfa::XinfaConfig;
use crate::type_set::{hostilepile, player, skilltype, xinfa};
use serde::{Deserialize, Serialize};

pub fn toml_input(profession: &str) -> String {
    let file_path = format!("{}.toml", profession);
    info(&format!("Loading config from: {}", file_path));
    match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            error(&format!("Failed to read config file: {}", e));
            "".into()
        }
    }
}

#[derive(Default, Deserialize)]
pub struct TomlConfig {
    pub xinfa: xinfa::XinfaConfig,
    pub skill: Vec<skilltype::Skilltype>,
}

#[derive(Default)]
pub struct Config {
    pub player: player::PlayerConfig,
    pub hostilepile: hostilepile::HostilepileConfig,
    pub data: TomlConfig,
}

impl Config {
    pub fn load(x: SaveConfig, fs: &str) -> Self {
        let data = load_config(fs);
        Config {
            player: x.player,
            hostilepile: x.hostilepile,
            data,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SaveConfig {
    pub xinfa: xinfa::XinfaConfig,
    pub player: player::PlayerConfig,
    pub hostilepile: hostilepile::HostilepileConfig,
}

pub fn load_config(profession: &str) -> TomlConfig {
    let current_dir = match std::env::current_exe() {
        Ok(path) => path
            .parent()
            .expect("Failed to get parent directory")
            .to_path_buf(),
        Err(e) => {
            error(&format!("Failed to get current exe path: {}", e));
            return TomlConfig::default();
        }
    };
    let file_path = current_dir.join("data").join("pvp36500").join(profession);
    let content = toml_input(file_path.to_str().unwrap());
    toml::from_str(&content).expect("Failed to parse TOML")
}

pub fn save_config(player: player::PlayerConfig, hostilepile: hostilepile::HostilepileConfig, xinfa: xinfa::XinfaConfig) {
    //SaveConfig 结构体转换为 TOML 字符串
    let save_config = SaveConfig {
        player,
        hostilepile,
        xinfa,
    };
    let x = toml::to_string(&save_config).unwrap();
    match std::fs::write("saved_config.toml", x) {
        Ok(_) => info("Configuration saved successfully."),
        Err(e) => error(&format!("Failed to save configuration: {}", e)),
    }
}

pub fn load_save_config() -> SaveConfig {
    let content = toml_input("saved_config");
    let x: SaveConfig = match toml::from_str(&content) {
        Ok(data) => data,
        Err(_) => {
            warn("No saved configuration found, using default values.");
            SaveConfig::default()
        }
    };
    x
}
