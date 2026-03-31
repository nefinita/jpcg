use colorized::Color;

pub fn info(message: &str) {
    println!("[GUI_INFO] {}", message.color(colorized::Colors::BlueFg));
}

pub fn warn(message: &str) {
    println!("[GUI_WARN] {}", message.color(colorized::Colors::YellowFg));
}

pub fn error(message: &str) {
    eprintln!("[GUI_ERROR] {}", message.color(colorized::Colors::RedFg));
}

pub fn debug(message: &str) {
    println!("[GUI_DEBUG] {}", message);
}

pub fn success(message: &str) {
    println!(
        "[GUI_SUCCESS] {}",
        message.color(colorized::Colors::GreenFg)
    );
}
