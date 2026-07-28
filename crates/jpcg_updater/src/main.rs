use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        eprintln!("用法: jpcg_updater <父进程PID> <旧程序路径> <新程序路径> <工作目录>");
        std::process::exit(1);
    }

    let parent_pid: u32 = match args[1].parse() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("无效的 PID '{}': {}", args[1], e);
            std::process::exit(1);
        }
    };
    let old_path = &args[2];
    let new_path = &args[3];
    let workdir = &args[4];

    // 等待父进程退出
    eprintln!("等待父进程 (PID {}) 退出...", parent_pid);
    loop {
        if !process_exists(parent_pid) {
            break;
        }
        sleep(Duration::from_millis(300));
    }
    eprintln!("父进程已退出，继续执行...");

    // 额外等待确保文件句柄释放
    sleep(Duration::from_secs(1));

    // 替换旧二进制
    let old = Path::new(old_path);
    let new = Path::new(new_path);

    if !new.exists() {
        eprintln!("新程序文件不存在: {}", new.display());
        std::process::exit(1);
    }

    if old.exists() {
        eprintln!("删除旧程序: {}", old.display());
        std::fs::remove_file(old).ok();
    }

    eprintln!("替换程序: {} -> {}", new.display(), old.display());
    std::fs::rename(new, old).or_else(|_| {
        std::fs::copy(new, old).and_then(|_| {
            std::fs::remove_file(new).ok();
            Ok(())
        })
    }).expect("替换程序失败");

    // 设置可执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(old, std::fs::Permissions::from_mode(0o755))
            .expect("设置可执行权限失败");
    }

    eprintln!("启动新版本: {}", old_path);
    Command::new(old_path)
        .current_dir(workdir)
        .spawn()
        .ok();

    eprintln!("更新完成，更新器退出。");
}

/// 检查指定 PID 的进程是否存在
fn process_exists(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        Command::new("tasklist")
            .args(&["/FI", &format!("PID eq {}", pid), "/NH"])
            .output()
            .map(|o| {
                let out = String::from_utf8_lossy(&o.stdout);
                out.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }
}
