//! gear 模型仅作 serde 契约；dynamic 壳不链接 gear/core/combo 引擎。
use libloading::Library;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    ffi::{CStr, CString, c_char},
    path::PathBuf,
    sync::OnceLock,
};

type Call = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_char;
type Free = unsafe extern "C" fn(*mut c_char);
struct Gear {
    _lib: Library,
    call: Call,
    free: Free,
}
static GEAR: OnceLock<Result<Gear, String>> = OnceLock::new();

fn load() -> Result<&'static Gear, String> {
    GEAR.get_or_init(|| {
        let name = format!(
            "{}jpcg_gear{}",
            std::env::consts::DLL_PREFIX,
            std::env::consts::DLL_SUFFIX
        );
        let mut paths = Vec::new();
        if let Some(path) = std::env::var_os("JPCG_GEAR_LIB") {
            paths.push(PathBuf::from(path));
        }
        if let Ok(exe) = std::env::current_exe()
            && let Some(dir) = exe.parent()
        {
            paths.extend([dir.join("modules").join(&name), dir.join(&name)]);
        }
        if let Ok(cwd) = std::env::current_dir() {
            paths.extend([
                cwd.join("target/debug").join(&name),
                cwd.join("target/release").join(&name),
            ]);
        }
        #[cfg(test)]
        if let Some(root) = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .find(|p| p.join("data/gear").is_dir())
        {
            paths.push(root.join("target/debug").join(&name));
        }
        let path = paths
            .into_iter()
            .find(|p| p.is_file())
            .ok_or("找不到配装动态库，可设置 JPCG_GEAR_LIB")?;
        let lib = unsafe { Library::new(path) }.map_err(|e| e.to_string())?;
        unsafe {
            let abi = lib
                .get::<unsafe extern "C" fn() -> u32>(b"jpcg_gear_abi_version")
                .map_err(|e| e.to_string())?;
            if abi() != 1 {
                return Err("配装动态库 ABI 不兼容".into());
            }
            let call = *lib
                .get::<Call>(b"jpcg_gear_call")
                .map_err(|e| e.to_string())?;
            let free = *lib
                .get::<Free>(b"jpcg_gear_free_string")
                .map_err(|e| e.to_string())?;
            Ok(Gear {
                _lib: lib,
                call,
                free,
            })
        }
    })
    .as_ref()
    .map_err(Clone::clone)
}

pub fn call<T: Serialize, R: DeserializeOwned>(method: &str, input: &T) -> Result<R, String> {
    let gear = load()?;
    let method = CString::new(method).map_err(|e| e.to_string())?;
    let input = CString::new(serde_json::to_string(input).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let response = unsafe { (gear.call)(method.as_ptr(), input.as_ptr()) };
    if response.is_null() {
        return Err("配装动态库返回空响应".into());
    }
    let text = unsafe { CStr::from_ptr(response) }
        .to_string_lossy()
        .into_owned();
    unsafe {
        (gear.free)(response);
    }
    let envelope: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    if envelope["status"] != "ok" {
        return Err(envelope["message"]
            .as_str()
            .unwrap_or("配装调用失败")
            .into());
    }
    serde_json::from_value(envelope["result"].clone()).map_err(|e| e.to_string())
}
