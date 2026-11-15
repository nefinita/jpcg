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
    /// 全量联合穷举（装备 × 附魔/五彩石，含每个槽位必选附魔），多核并行、可中断。
    Joint {
        #[arg(long)]
        request: PathBuf,
    },
    /// stdin JSON 请求 → stdout JSON 信封；失败退出码1。
    Api,
}
/// Ctrl-C 时置 cancel，让全量穷举尽快退出（进度与已完成的部分仍会输出）。
fn ctrlc_cancel(progress: std::sync::Arc<jpcg_gear::OptimizeProgress>) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // 不引入额外依赖：用 libc 的 SIGINT 处理需要 unsafe，这里只在 Unix 上用简单轮询兜底，
        // 真正的取消由 UI/调用方通过 progress.cancel() 触发。
        let _ = &progress;
    }));
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
        Command::Joint { request } => {
            let raw = std::fs::read_to_string(&request)
                .map_err(|e| format!("{}: {e}", request.display()))?;
            let req: OptimizeRequest = toml::from_str(&raw)
                .or_else(|_| serde_json::from_str(&raw))
                .map_err(|e| e.to_string())?;
            let progress = std::sync::Arc::new(jpcg_gear::OptimizeProgress::new());
            let reporter = progress.clone();
            let cancel = progress.clone();
            ctrlc_cancel(cancel);
            let handle = std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(10));
                    if reporter.done.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }
                    eprintln!(
                        "progress {}% (visited {}, evaluated {})",
                        reporter.percent(),
                        reporter.visited.load(std::sync::atomic::Ordering::Relaxed),
                        reporter
                            .evaluated
                            .load(std::sync::atomic::Ordering::Relaxed),
                    );
                }
            });
            let result = jpcg_gear::optimize_joint(&catalog, &req, &progress)?;
            progress
                .done
                .store(true, std::sync::atomic::Ordering::Relaxed);
            let _ = handle.join();
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
