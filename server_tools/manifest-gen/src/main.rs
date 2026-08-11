use clap::Parser;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    author,
    about = "生成 data_manifest.toml / modules_manifest.toml 供更新模块使用",
    disable_version_flag = true
)]
struct Args {
    #[arg(short, long, default_value = "./data")]
    data_dir: PathBuf,

    #[arg(short, long)]
    version: String,

    #[arg(short, long, default_value = "data_manifest.toml")]
    output: PathBuf,

    /// 模块库（dll）目录；提供后额外生成 modules_manifest.toml
    #[arg(long)]
    modules_dir: Option<PathBuf>,

    #[arg(long, default_value = "modules_manifest.toml")]
    modules_output: PathBuf,

    /// 目标平台（默认按本机 OS 推断：darwin / linux / windows）
    #[arg(long)]
    platform: Option<String>,
}

#[derive(Serialize)]
struct DataManifest {
    data_version: String,
    files: Vec<DataFileEntry>,
}

#[derive(Serialize)]
struct DataFileEntry {
    path: String,
    hash: String,
    hash_type: String,
}

#[derive(Serialize)]
struct ModulesManifest {
    modules_version: String,
    platform: String,
    files: Vec<ModulesFileEntry>,
}

#[derive(Serialize)]
struct ModulesFileEntry {
    name: String,
    hash: String,
    hash_type: String,
    size: u64,
}

fn detect_platform() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        "linux" => "linux",
        "windows" => "windows",
        other => other,
    }
}

fn main() {
    let args = Args::parse();

    let platform = args.platform.clone().unwrap_or_else(|| detect_platform().to_string());

    if !args.data_dir.is_dir() {
        eprintln!("错误: '{}' 不是有效的目录", args.data_dir.display());
        std::process::exit(1);
    }

    let mut files = Vec::new();
    walk_dir(&args.data_dir, &args.data_dir, &mut files).unwrap_or_else(|e| {
        eprintln!("扫描目录失败: {}", e);
        std::process::exit(1);
    });

    files.sort_by(|a, b| a.path.cmp(&b.path));

    let manifest = DataManifest {
        data_version: args.version.clone(),
        files,
    };

    let toml_str = toml::to_string_pretty(&manifest).unwrap_or_else(|e| {
        eprintln!("序列化为 TOML 失败: {}", e);
        std::process::exit(1);
    });

    fs::write(&args.output, &toml_str).unwrap_or_else(|e| {
        eprintln!("写入输出文件 '{}' 失败: {}", args.output.display(), e);
        std::process::exit(1);
    });

    println!(
        "✅ 已生成: {} (共 {} 个文件)",
        args.output.display(),
        manifest.files.len()
    );

    if let Some(ref modules_dir) = args.modules_dir {
        if !modules_dir.is_dir() {
            eprintln!("错误: '{}' 不是有效的目录", modules_dir.display());
            std::process::exit(1);
        }

        let mut modules = Vec::new();
        walk_modules_dir(modules_dir, &mut modules).unwrap_or_else(|e| {
            eprintln!("扫描模块目录失败: {}", e);
            std::process::exit(1);
        });
        modules.sort_by(|a, b| a.name.cmp(&b.name));

        let modules_manifest = ModulesManifest {
            modules_version: args.version,
            platform,
            files: modules,
        };

        let modules_toml = toml::to_string_pretty(&modules_manifest).unwrap_or_else(|e| {
            eprintln!("序列化为 TOML 失败: {}", e);
            std::process::exit(1);
        });

        fs::write(&args.modules_output, &modules_toml).unwrap_or_else(|e| {
            eprintln!("写入输出文件 '{}' 失败: {}", args.modules_output.display(), e);
            std::process::exit(1);
        });

        println!(
            "✅ 已生成: {} (共 {} 个模块，平台 {})",
            args.modules_output.display(),
            modules_manifest.files.len(),
            modules_manifest.platform
        );
    }
}

fn walk_dir(
    root: &PathBuf,
    dir: &PathBuf,
    files: &mut Vec<DataFileEntry>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            walk_dir(root, &path, files)?;
        } else if path.is_file() {
            let rel_path = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");

            let hash = calc_sha256(&path)?;

            files.push(DataFileEntry {
                path: rel_path,
                hash,
                hash_type: "SHA256".to_string(),
            });
        }
    }
    Ok(())
}

/// 模块目录：仅哈希一层文件名（不递归）
fn walk_modules_dir(
    dir: &PathBuf,
    files: &mut Vec<ModulesFileEntry>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let hash = calc_sha256(&path)?;
        let size = fs::metadata(&path)?.len();
        files.push(ModulesFileEntry {
            name,
            hash,
            hash_type: "SHA256".to_string(),
            size,
        });
    }
    Ok(())
}

fn calc_sha256(path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(hex::encode(hasher.finalize()))
}
