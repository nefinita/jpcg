// ============================================================================
// log — 彩色日志输出工具
// 依赖 colorized crate 实现终端颜色输出。
// 分为 info / warn / error / success 四种级别。
// 日志前缀使用 [CORE_*] 标签 + 对应颜色。
// ============================================================================

use colorized::Color;

/// 信息日志（蓝色背景标签，白色文字）
pub fn info(message: &str) {
    let x = "CORE_INFO".color(colorized::Colors::BlueBg);
    println!("{}    {}", x, message.color(colorized::Colors::WhiteFg));
}

/// 警告日志（黄色背景标签，黄色文字）
pub fn warn(message: &str) {
    let x = "CORE_WARN".color(colorized::Colors::YellowBg);
    println!("{}    {}", x, message.color(colorized::Colors::YellowFg));
}

/// 错误日志（红色背景标签，红色文字，输出到 stderr）
pub fn error(message: &str) {
    let x = "CORE_ERROR".color(colorized::Colors::RedBg);
    eprintln!("{}   {}", x, message.color(colorized::Colors::RedFg));
}

/// 成功日志（绿色背景标签，绿色文字）
pub fn success(message: &str) {
    let x = "CORE_SUCCESS".color(colorized::Colors::GreenBg);
    println!("{} {}", x, message.color(colorized::Colors::GreenFg));
}
