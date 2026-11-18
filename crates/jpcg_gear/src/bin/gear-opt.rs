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
    /// 列出本机 CPU/wgpu 后端，供无 GUI 的性能测试准确选择设备。
    Backends,
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
        /// 仅运行指定秒数后正常取消并输出已完成结果；用于短轮测速。
        #[arg(long)]
        seconds: Option<u64>,
    },
    /// stdin JSON 请求 → stdout JSON 信封；失败退出码1。
    Api,
}
static INTERRUPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[cfg(unix)]
extern "C" fn interrupt_signal(_: libc::c_int) {
    INTERRUPTED.store(true, std::sync::atomic::Ordering::Relaxed);
}
#[cfg(windows)]
unsafe extern "system" fn interrupt_console(kind: u32) -> i32 {
    if kind <= 1 {
        INTERRUPTED.store(true, std::sync::atomic::Ordering::Relaxed);
        1
    } else {
        0
    }
}
#[cfg(windows)]
#[link(name = "Kernel32")]
unsafe extern "system" {
    fn SetConsoleCtrlHandler(
        handler: Option<unsafe extern "system" fn(u32) -> i32>,
        add: i32,
    ) -> i32;
}
struct InterruptGuard {
    #[cfg(unix)]
    previous: (libc::sighandler_t, libc::sighandler_t),
}
impl InterruptGuard {
    fn install() -> Result<Self, String> {
        INTERRUPTED.store(false, std::sync::atomic::Ordering::Relaxed);
        #[cfg(unix)]
        {
            // 处理器仅写无锁原子标记，不分配、不触碰Rust锁或引用。
            let first = unsafe {
                libc::signal(
                    libc::SIGINT,
                    interrupt_signal as *const () as libc::sighandler_t,
                )
            };
            if first == libc::SIG_ERR {
                return Err("无法安装SIGINT处理器".into());
            }
            let second = unsafe {
                libc::signal(
                    libc::SIGTERM,
                    interrupt_signal as *const () as libc::sighandler_t,
                )
            };
            if second == libc::SIG_ERR {
                unsafe {
                    libc::signal(libc::SIGINT, first);
                }
                return Err("无法安装SIGTERM处理器".into());
            }
            Ok(Self {
                previous: (first, second),
            })
        }
        #[cfg(windows)]
        {
            if unsafe { SetConsoleCtrlHandler(Some(interrupt_console), 1) } == 0 {
                return Err("无法安装控制台中断处理器".into());
            }
            Ok(Self {})
        }
        #[cfg(not(any(unix, windows)))]
        {
            Ok(Self {})
        }
    }
}
impl Drop for InterruptGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            libc::signal(libc::SIGINT, self.previous.0);
            libc::signal(libc::SIGTERM, self.previous.1);
        }
        #[cfg(windows)]
        unsafe {
            SetConsoleCtrlHandler(Some(interrupt_console), 0);
        }
    }
}

fn read<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Result<T, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&raw).map_err(|e| format!("{}: {e}", path.display()))
}
fn run() -> Result<bool, String> {
    let args = Args::parse();
    let catalog = Catalog::load(&args.catalog)?;
    let (output, success) = match args.command {
        Command::Backends => (
            serde_json::to_value(jpcg_compute::list_backends()).map_err(|e| e.to_string())?,
            true,
        ),
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
        Command::Joint { request, seconds } => {
            let raw = std::fs::read_to_string(&request)
                .map_err(|e| format!("{}: {e}", request.display()))?;
            let req: OptimizeRequest = toml::from_str(&raw)
                .or_else(|_| serde_json::from_str(&raw))
                .map_err(|e| e.to_string())?;
            let progress = std::sync::Arc::new(jpcg_gear::OptimizeProgress::new());
            let reporter = progress.clone();
            if seconds == Some(0) {
                progress.cancel();
            }
            let _interrupts = InterruptGuard::install()?;
            let (finish, finished) = std::sync::mpsc::channel();
            let handle = std::thread::spawn(move || {
                let start = std::time::Instant::now();
                let mut last_report = start;
                loop {
                    if finished.recv_timeout(std::time::Duration::from_millis(100))
                        != Err(std::sync::mpsc::RecvTimeoutError::Timeout)
                    {
                        break;
                    }
                    if INTERRUPTED.load(std::sync::atomic::Ordering::Relaxed)
                        || seconds.is_some_and(|limit| start.elapsed().as_secs() >= limit)
                    {
                        reporter.cancel();
                    }
                    if last_report.elapsed().as_secs() >= 10 {
                        eprintln!(
                            "progress {}% (visited {}, evaluated {}, mc {})",
                            reporter.percent(),
                            reporter.visited.load(std::sync::atomic::Ordering::Relaxed),
                            reporter
                                .evaluated
                                .load(std::sync::atomic::Ordering::Relaxed),
                            reporter
                                .mc_evaluations
                                .load(std::sync::atomic::Ordering::Relaxed)
                        );
                        last_report = std::time::Instant::now();
                    }
                }
            });
            let result = jpcg_gear::optimize_joint(&catalog, &req, &progress);
            let _ = finish.send(());
            let _ = handle.join();
            let result = result?;
            let success = result.best.is_some() || result.stop_reason == "cancelled";
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
