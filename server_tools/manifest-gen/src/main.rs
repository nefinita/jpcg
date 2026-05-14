use clap::Parser;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, about = "生成 data_manifest.toml 供更新模块使用", disable_version_flag = true)]
struct Args {
    #[arg(short, long, default_value = "./data")]
    data_dir: PathBuf,

    #[arg(short, long)]
    version: String,

    #[arg(short, long, default_value = "data_manifest.toml")]
    output: PathBuf,
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

fn main() {
    let args = Args::parse();

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
        data_version: args.version,
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
