use colorized::Color;

pub fn info(message: &str) {
    let x = "CORE_INFO".color(colorized::Colors::BlueBg);
    println!("{}    {}", x, message.color(colorized::Colors::WhiteFg));
}

pub fn warn(message: &str) {
    let x = "CORE_WARN".color(colorized::Colors::YellowBg);
    println!("{}    {}", x, message.color(colorized::Colors::YellowFg));
}

pub fn error(message: &str) {
    let x = "CORE_ERROR".color(colorized::Colors::RedBg);
    eprintln!("{}   {}", x, message.color(colorized::Colors::RedFg));
}

pub fn debug(message: &str) {
    let x = "CORE_DEBUG";
    println!("{}   {}", x, message);
}

pub fn success(message: &str) {
    let x = "CORE_SUCCESS".color(colorized::Colors::GreenBg);
    println!(
        "{} {}", x,
        message.color(colorized::Colors::GreenFg)
    );
}
