// ============================================================================
// engine — 剑网3 伤害计算引擎
// 根据玩家属性、目标防御属性、心法配置、技能数据，
// 逐一计算每个技能的多段伤害（Y/B/I/N/H/Q）。
// 核心公式参考剑网3 现行版本伤害机制。
// ============================================================================

pub mod atkcal;
pub mod derivatives;
pub mod kill_prob;

use std::io::Error;

use crate::{
    engine::atkcal::JpcgConfig,
    log::{error, success},
    store::{TomlConfig, data_dir, toml_input},
    type_set::{
        buff::BuffConfig, coefficient::CoefficientConfig, hostilepile::HostilepileConfig,
        player::PlayerConfig, xinfa::XinfaConfig,
    },
};
use serde::Serialize;

/// 启动完整伤害计算
/// 1. 定位可执行文件所在目录下的 data/shuxing/{profession}.toml
/// 2. 解析 TOML 获得技能列表
/// 3. 对每个技能调用单技能伤害计算
///
/// # 参数
/// - `player`: 玩家属性（基础属性、攻击、会心、破防、武器伤害）
/// - `hostilepile`: 目标属性（外内防、御劲、化劲、减伤）
/// - `xinfa`: 心法配置（心法 key、心法名、根骨/元气、攻击/破防/会心加成）
///
/// # 返回
/// - `Ok(Vec<CalculateResult>)`: 每个技能的各段伤害结果
/// - `Err`: 路径定位或 TOML 解析失败
pub fn start_calculation(
    player: PlayerConfig,
    hostilepile: HostilepileConfig,
    xinfa: XinfaConfig,
) -> Result<Vec<CalculateResult>, Error> {
    start_calculation_with_config(
        player,
        hostilepile,
        xinfa,
        &BuffConfig::default(),
        &CoefficientConfig::default(),
    )
}

pub fn start_calculation_with_config(
    player: PlayerConfig,
    hostilepile: HostilepileConfig,
    xinfa: XinfaConfig,
    buff: &BuffConfig,
    coeff: &CoefficientConfig,
) -> Result<Vec<CalculateResult>, Error> {
    success("Calculation started!");

    // ---- 步骤1: 定位心法数据文件路径 ----
    let dir = match data_dir() {
        Some(d) => d,
        None => {
            error("无法获取数据文件目录");
            return Err(Error::other("无法获取数据文件目录"));
        }
    };

    let file_path = dir.join(xinfa.profession.clone());
    let file_path_str = match file_path.to_str() {
        Some(s) => s.to_string(),
        None => {
            error("配置文件路径包含非法 UTF-8 字符");
            return Err(Error::other("配置文件路径包含非法 UTF-8 字符"));
        }
    };

    // ---- 步骤3: 读取 TOML 内容并解析 ----
    let skill_table: TomlConfig = match toml_input(&file_path_str) {
        // 文件不存在时返回 None，使用默认空技能表
        None => TomlConfig::default(),
        Some(content) => match toml::from_str(content.as_str()) {
            Ok(config) => config,
            Err(e) => {
                error(format!("心法技能 TOML 解析失败: {}", e).as_str());
                return Err(Error::other(format!("心法技能 TOML 解析失败: {}", e)));
            }
        },
    };

    // ---- 步骤4: 逐技能计算伤害 ----
    Ok(call_back(&skill_table, &player, &hostilepile, buff, coeff))
}

// ============================================================================
// CalculateResult — 单技能计算结果
// 包含技能名称及各段伤害数值，对应前端表格 7 列。
// ============================================================================

/// 单技能完整的伤害计算结果
#[derive(Default, Serialize)]
pub struct CalculateResult {
    pub skill_name: String, // 技能名称
    pub y: u32,             // 破防系数计算结果（Y 段）
    pub b: u32,             // 基础攻击力（B 段）
    pub i: u32,             // 技能基础伤害（I 段）
    pub n: u32,             // 普通命中伤害（N 段，常规命中）
    pub h: u32,             // 会心伤害（H 段）
    pub q: u32,             // 期望伤害（Q 段，考虑会心概率的加权值）
    /// Dot 每跳期望伤害（非 Dot 技能为空；q 为各跳之和）
    #[serde(default)]
    pub dot_jumps: Vec<u32>,
    /// 无质（伤害固定 = 期望 Q，含会心加权）
    #[serde(default)]
    pub has_critical_strike: bool,
    /// 真实伤害（数据源 custom_damage_base 标签，无视防御减免）
    #[serde(default)]
    pub zhenshishanghai: u32,
    /// 追加真伤系数（已损失生命值 × 系数，连招中动态结算，单技能面板满血为 0）
    #[serde(default)]
    pub lost_hp_zhenshishanghai: f32,
}

impl CalculateResult {
    /// 构造完整的计算结算
    #[allow(clippy::too_many_arguments)]
    pub fn new(skill_name: String, y: u32, b: u32, i: u32, n: u32, h: u32, q: u32) -> Self {
        CalculateResult {
            skill_name,
            y,
            b,
            i,
            n,
            h,
            q,
            dot_jumps: Vec::new(),
            has_critical_strike: false,
            zhenshishanghai: 0,
            lost_hp_zhenshishanghai: 0.0,
        }
    }

    /// 打印当前伤害计算结果到日志（Dot 技能逐跳一行）
    pub fn get_message(&self) {
        success(&format!(
            "技能: {}, Y: {}, B: {}, I: {}, N: {}, H: {}, Q: {}",
            self.skill_name, self.y, self.b, self.i, self.n, self.h, self.q
        ));
        for (k, j) in self.dot_jumps.iter().enumerate() {
            success(&format!("DOT 第{}跳: {}", k + 1, j));
        }
    }
}

// ============================================================================
// call_back — 遍历技能表执行计算
// 读取 TomlConfig 中的技能列表，对每个技能实例化 JpcgConfig 并计算伤害。
// ============================================================================

/// 回调遍历函数: 将 TOML 配置中的每个技能送入计算引擎
///
/// # 参数
/// - `toml_config`: 从 TOML 文件解析出的心法+技能配置
/// - `player`: 玩家属性
/// - `hostilepile`: 目标属性
///
/// # 返回
/// 所有技能的伤害结果列表
fn call_back(
    toml_config: &TomlConfig,
    player: &PlayerConfig,
    hostilepile: &HostilepileConfig,
    buff: &BuffConfig,
    coeff: &CoefficientConfig,
) -> Vec<CalculateResult> {
    let mut results = Vec::new();
    for skill in &toml_config.skill {
        let damage_result = JpcgConfig::new_with_config(
            player,
            hostilepile,
            skill,
            &toml_config.xinfa,
            buff,
            coeff,
        )
        .q_cal();

        // 将 5 段伤害数组映射为 CalculateResult
        let mut calculate_result = CalculateResult::new(
            skill.skill_name.clone(),
            damage_result.y,        // Y: 破防系数段
            damage_result.b,        // B: 基础攻击段
            damage_result.i,        // I: 技能基础段
            damage_result.g_damage, // N: 普通命中段
            damage_result.h_damage, // H: 会心段
            damage_result.q_damage, // Q: 期望值段
        );
        calculate_result.dot_jumps = damage_result.dot_jumps;
        calculate_result.has_critical_strike = skill.has_critical_strike;
        calculate_result.zhenshishanghai = skill.zhenshishanghai;
        calculate_result.lost_hp_zhenshishanghai = skill.lost_hp_zhenshishanghai;
        calculate_result.get_message();
        results.push(calculate_result);
    }
    results
}
