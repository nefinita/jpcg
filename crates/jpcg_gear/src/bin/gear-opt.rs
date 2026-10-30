use clap::{Parser, Subcommand};
use jpcg_gear::{Catalog, Loadout, OptimizeRequest};
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "JPCG 配装模板校验/合成/有界优化；样例数据不代表游戏装备")]
struct Args {
    #[arg(long, default_value = "data/gear")]
    catalog: PathBuf,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Validate,
    Compose {
        #[arg(long)]
        loadout: PathBuf,
    },
    Optimize {
        #[arg(long)]
        request: PathBuf,
    },
    /// stdin JSON 请求 → stdout JSON 信封；失败退出码1。
    Api,
}
fn read<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Result<T, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&raw).map_err(|e| format!("{}: {e}", path.display()))
}
fn run() -> Result<bool, String> {
    let args = Args::parse();
    let catalog = Catalog::load(&args.catalog)?;
    let (output, success) = match args.command {
        Command::Validate => (
            serde_json::json!({"version": catalog.version, "templates": catalog.templates.len()}),
            true,
        ),
        Command::Compose { loadout } => {
            let loadout: Loadout = read(&loadout)?;
            (
                serde_json::to_value(catalog.compose(&loadout)?).map_err(|e| e.to_string())?,
                true,
            )
        }
        Command::Optimize { request } => {
            let req: OptimizeRequest = read(&request)?;
            let result = jpcg_gear::optimize(&catalog, &req, |_| true)?;
            let success = result.best.is_some();
            (
                serde_json::to_value(result).map_err(|e| e.to_string())?,
                success,
            )
        }
        Command::Api => {
            use std::io::Read;
            let mut input = String::new();
            std::io::stdin()
                .read_to_string(&mut input)
                .map_err(|e| e.to_string())?;
            let response = jpcg_gear::call(&catalog, &input);
            let success = matches!(response, jpcg_gear::ApiResponse::Ok { .. });
            (
                serde_json::to_value(response).map_err(|e| e.to_string())?,
                success,
            )
        }
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?
    );
    Ok(success)
}
fn main() -> std::process::ExitCode {
    match run() {
        Ok(true) => std::process::ExitCode::SUCCESS,
        Ok(false) => std::process::ExitCode::FAILURE,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
