mod download; // 导入 download 模块

const UPDATE_BASE_URL: &str = "https://nefinita-ai.com/updates/JPCG/";
const BETA_BASE_URL: &str = "https://nefinita-ai.com/updates/JPCG_beta/";
const CURRENT_DIR: &str = ".";

use clap::Parser;
use std::env;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    // Define your command-line arguments here
    #[arg(long)]
    force_check: bool,
    #[arg(long)]
    target_os: Option<String>,
    #[arg(long)]
    target_arch: Option<String>,
    #[arg(short = 'b', long = "beta")]
    beta: bool, // 用户明确指定使用 beta 通道
}

pub fn all_updates() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running all updates...");
    let args = Args::parse();
    let app_dir = Path::new(CURRENT_DIR);
    let base_path = app_dir.canonicalize()?;
    let use_beta_channel = args.beta || {
        match dow {
            
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
