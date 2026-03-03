use colorized::Color;

pub fn info(message: &str) {
    println!("[CORE_INFO] {}", message.color(colorized::Colors::BlueFg));
}

pub fn warn(message: &str) {
    println!("[CORE_WARN] {}", message.color(colorized::Colors::YellowFg));
}

pub fn error(message: &str) {
    eprintln!("[CORE_ERROR] {}", message.color(colorized::Colors::RedFg));
}

pub fn debug(message: &str) {
    println!("[CORE_DEBUG] {}", message);
}

pub fn success(message: &str) {
    println!(
        "[CORE_SUCCESS] {}",
        message.color(colorized::Colors::GreenFg)
    );
}
