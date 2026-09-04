// ============================================================================
// deploy-gen — 更新服务器通道目录编排器
//
// 输入：GitHub Release 资产目录（三平台 app 二进制 + 各平台 dll 集 +
//       data_manifest.toml + channel.txt）、仓库 data 目录、版本号、通道
// 输出：可直接 push 到服务器 downloads 根的 stage 目录：
//   beta  : stage/JPCG_beta/{update.toml, manifest.toml, 二进制,
//           modules/{dll×9, modules_manifest.toml}, data/{data_manifest.toml, 数据}}
//   stable: stage/JPCG/update.toml +
//           stage/JPCG/{version}/{manifest.toml, 二进制, modules/}
//           stage/JPCG/{data_version}/data/{data_manifest.toml, 数据}
//
// 约定（与 jpcg_update 客户端 / server_manifest.md 一致）：
//   - beta：清单在通道根、二进制在根、模块清单 modules/modules_manifest.toml、
//     数据 data/data_manifest.toml
//   - stable：update.toml 在根；每版本 vX.Y.Z 目录放 manifest+二进制+modules；
//     数据放 {data_version}/data
//   - modules_manifest 合并三平台 dll（platform="multi"），客户端按扩展名过滤
// ============================================================================

use clap::Parser;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "deploy-gen", about = "生成更新服务器通道目录")]
struct Args {
    /// 通道: beta | stable
    #[arg(long)]
    channel: String,
    /// 版本号（含 v 前缀），如 v2.1.0-beta.1
    #[arg(long)]
    version: String,
    /// GitHub Release 资产所在目录
    #[arg(long)]
    assets_dir: PathBuf,
    /// 仓库 data 目录（含 shuxing 等；仅拷贝 data_manifest 列出的文件）
    #[arg(long)]
    data_dir: PathBuf,
    /// 输出 stage 根目录
    #[arg(long)]
    output: PathBuf,
}

#[derive(Deserialize)]
struct DataManifest {
    #[serde(default)]
    data_version: Option<String>,
    files: Vec<DataFileEntry>,
}
#[derive(Deserialize)]
struct DataFileEntry {
    path: String,
}

const APP_BINS: [(&str, &str, &str); 3] = [
    ("jpcg-app-darwin-x86_64", "darwin", "x86_64"),
    ("jpcg-app-linux-x86_64", "linux", "x86_64"),
    ("jpcg-app-windows-x86_64.exe", "windows", "x86_64"),
];

/// 各平台 dll 文件名（三平台合并；客户端按扩展名取本平台）
const MODULES: [&str; 9] = [
    "libjpcg_core.dylib",
    "libjpcg_update.dylib",
    "libjpcg_const.dylib",
    "libjpcg_core.so",
    "libjpcg_update.so",
    "libjpcg_const.so",
    "jpcg_core.dll",
    "jpcg_update.dll",
    "jpcg_const.dll",
];

fn sha256_hex(path: &Path) -> String {
    let mut f = fs::File::open(path).expect("open file for hashing");
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 1 << 16];
    loop {
        let n = f.read(&mut buf).expect("read file");
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    hex::encode(hasher.finalize())
}

fn copy_asset(src: &Path, dst: &Path) {
    fs::create_dir_all(dst.parent().expect("dst parent")).expect("create dst dir");
    fs::copy(src, dst).expect("copy asset");
}

fn module_version(name: &str, app_version: &str) -> String {
    // core / update 模块版本与 app 同源；const 为独立版本（反查其 Cargo.toml）
    if name.contains("const") {
        read_const_version()
    } else {
        app_version.trim_start_matches('v').to_string()
    }
}

/// 读取 jpcg_const 独立版本号（130.3.日期）
fn read_const_version() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/jpcg_const/Cargo.toml");
    let text = fs::read_to_string(&path).expect("read jpcg_const/Cargo.toml");
    text.lines()
        .find(|l| l.starts_with("version"))
        .and_then(|l| l.split('\"').nth(1).map(|s| s.to_string()))
        .expect("jpcg_const version")
}

fn main() {
    let args = Args::parse();
    if args.channel != "beta" && args.channel != "stable" {
        eprintln!("channel 必须是 beta 或 stable");
        std::process::exit(1);
    }

    // 资产存在性
    let assets = &args.assets_dir;
    for (name, _, _) in APP_BINS {
        if !assets.join(name).is_file() {
            eprintln!("缺少资产: {}", assets.join(name).display());
            std::process::exit(1);
        }
    }
    for m in MODULES {
        if !assets.join(m).is_file() {
            eprintln!("缺少资产: {}", assets.join(m).display());
            std::process::exit(1);
        }
    }
    let dm_asset = assets.join("data_manifest.toml");
    if !dm_asset.is_file() {
        eprintln!("缺少资产: data_manifest.toml");
        std::process::exit(1);
    }
    let dm_text = fs::read_to_string(&dm_asset).expect("read data_manifest.toml");
    let dm: DataManifest = toml::from_str(&dm_text).expect("parse data_manifest.toml");
    let data_version = dm
        .data_version
        .clone()
        .unwrap_or_else(|| args.version.clone());

    let stage = &args.output;
    fs::remove_dir_all(stage).ok();
    let ver = args.version.trim_start_matches('v');

    let root: PathBuf; // 通道根（stage 下）
    let data_rel: PathBuf; // 数据文件相对通道根的目录
    if args.channel == "beta" {
        root = stage.join("JPCG_beta");
        data_rel = Path::new("data").to_path_buf();
        fs::create_dir_all(root.join("modules")).expect("mkdir modules");
        fs::create_dir_all(root.join("data")).expect("mkdir data");
    } else {
        root = stage.join("JPCG");
        let vdir = root.join(format!("v{ver}"));
        fs::create_dir_all(vdir.join("modules")).expect("mkdir modules");
        data_rel = root.join(&data_version).join("data");
        fs::create_dir_all(&data_rel).expect("mkdir data");
        // 版本目录内含 manifest + 二进制 + modules
        for (name, _, _) in APP_BINS {
            copy_asset(&assets.join(name), &vdir.join(name));
        }
        for m in MODULES {
            copy_asset(&assets.join(m), &vdir.join("modules").join(m));
        }
        write_manifest_toml(&vdir.join("manifest.toml"), ver);
        write_modules_manifest(
            &vdir.join("modules").join("modules_manifest.toml"),
            &vdir.join("modules"),
            &args.version,
        );
    }

    if args.channel == "beta" {
        // 二进制放根
        for (name, _, _) in APP_BINS {
            copy_asset(&assets.join(name), &root.join(name));
        }
        for m in MODULES {
            copy_asset(&assets.join(m), &root.join("modules").join(m));
        }
        write_manifest_toml(&root.join("manifest.toml"), ver);
        write_modules_manifest(
            &root.join("modules").join("modules_manifest.toml"),
            &root.join("modules"),
            &args.version,
        );
        write_update_toml(&root.join("update.toml"), ver, &data_version);
    }

    // stable：根 update.toml 指向最新版本
    if args.channel == "stable" {
        write_update_toml(&root.join("update.toml"), ver, &data_version);
    }

    // 数据：按 data_manifest 列出的文件拷贝（data_manifest.toml 本身放 data 根）
    copy_asset(&dm_asset, &root.join(&data_rel).join("data_manifest.toml"));
    let data_src = &args.data_dir;
    for f in &dm.files {
        let src = data_src.join(&f.path);
        let dst = root.join(&data_rel).join(&f.path);
        if !src.is_file() {
            eprintln!("data 清单引用的文件缺失: {}", src.display());
            std::process::exit(1);
        }
        copy_asset(&src, &dst);
    }

    println!(
        "deploy-gen OK: channel={} version={} -> {}",
        args.channel,
        args.version,
        stage.display()
    );
}

fn write_update_toml(path: &Path, version: &str, data_version: &str) {
    let major = version.split('.').next().unwrap_or("0");
    fs::write(
        path,
        format!(
            "version = \"v{}\"\nmajor_version = {}\ndata_version = \"{}\"\n",
            version, major, data_version
        ),
    )
    .expect("write update.toml");
}

fn write_manifest_toml(path: &Path, version: &str) {
    let mut out = String::from(format!("version = \"v{}\"\nmajor_version = 2\n\n", version));
    for (name, os, arch) in APP_BINS {
        let parent = path.parent().expect("parent");
        let hash = sha256_hex(&parent.join(name));
        out.push_str("[[binaries]]\n");
        out.push_str(&format!(
            "path = \"{name}\"\nos = \"{os}\"\narch = \"{arch}\"\n"
        ));
        out.push_str(&format!("hash = \"{hash}\"\nhash_type = \"SHA256\"\n\n"));
    }
    fs::write(path, out).expect("write manifest.toml");
}

fn write_modules_manifest(manifest_path: &Path, modules_dir: &Path, version: &str) {
    let mut out = String::from(format!(
        "modules_version = \"{version}\"\nplatform = \"multi\"\n\n"
    ));
    for name in MODULES {
        let p = modules_dir.join(name);
        if !p.is_file() {
            eprintln!("模块缺失: {}", p.display());
            std::process::exit(1);
        }
        let meta = fs::metadata(&p).expect("module metadata");
        out.push_str("[[files]]\n");
        out.push_str(&format!("name = \"{name}\"\n"));
        out.push_str(&format!(
            "version = \"{}\"\n",
            module_version(name, version)
        ));
        out.push_str(&format!("hash = \"{}\"\n", sha256_hex(&p)));
        out.push_str("hash_type = \"SHA256\"\n");
        out.push_str(&format!("size = {}\n\n", meta.len()));
    }
    fs::write(manifest_path, out).expect("write modules_manifest.toml");
}
