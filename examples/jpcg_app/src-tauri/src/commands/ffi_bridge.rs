// ============================================================================
// commands::ffi_bridge — 动态模式桥接层（feature = "dynamic"）
// dlopen libjpcg_core.{dylib,so,dll}，经 jpcg_call(handle, method, json) 调用，
// 进度/退出/updater 路径经 jpcg_set_host_events 回调表注入。
// 静态模式（feature = "static"）不编译本模块，命令直调 jpcg_core::host。
// ============================================================================

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::path::PathBuf;
use std::sync::OnceLock;

use jpcg_api::HostEventsTable;
use libloading::{Library, Symbol};
use serde::Serialize;
use serde::de::DeserializeOwned;

type JpcgCallFn = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> *mut c_char;
type JpcgSetEventsFn = unsafe extern "C" fn(*const HostEventsTable) -> std::os::raw::c_int;
type JpcgFreeStringFn = unsafe extern "C" fn(*mut c_char);
type JpcgLastErrorFn = unsafe extern "C" fn() -> *mut c_char;
type JpcgHandleCreateFn = unsafe extern "C" fn(*const c_char) -> *mut c_void;
type JpcgHandleFreeFn = unsafe extern "C" fn(*mut c_void);
type JpcgVersionFn = unsafe extern "C" fn() -> *mut c_char;

#[cfg(target_os = "windows")]
const CORE_LIB_NAME: &str = "jpcg_core.dll";
#[cfg(target_os = "macos")]
const CORE_LIB_NAME: &str = "libjpcg_core.dylib";
#[cfg(all(unix, not(target_os = "macos")))]
const CORE_LIB_NAME: &str = "libjpcg_core.so";

#[cfg(target_os = "windows")]
const CONST_LIB_NAME: &str = "jpcg_const.dll";
#[cfg(target_os = "macos")]
const CONST_LIB_NAME: &str = "libjpcg_const.dylib";
#[cfg(all(unix, not(target_os = "macos")))]
const CONST_LIB_NAME: &str = "libjpcg_const.so";

#[cfg(target_os = "windows")]
const UPDATE_LIB_NAME: &str = "jpcg_update.dll";
#[cfg(target_os = "macos")]
const UPDATE_LIB_NAME: &str = "libjpcg_update.dylib";
#[cfg(all(unix, not(target_os = "macos")))]
const UPDATE_LIB_NAME: &str = "libjpcg_update.so";

struct CoreLib {
    _lib: Library,
    handle: *mut c_void,
    jpcg_call: JpcgCallFn,
    jpcg_handle_free: JpcgHandleFreeFn,
    jpcg_free_string: JpcgFreeStringFn,
    jpcg_last_error: JpcgLastErrorFn,
    jpcg_core_version: JpcgVersionFn,
}

// JpcgCallFn 等均无 Send 限制（仅函数指针/裸指针），断言安全
unsafe impl Send for CoreLib {}
unsafe impl Sync for CoreLib {}

impl Drop for CoreLib {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { (self.jpcg_handle_free)(self.handle) };
            self.handle = std::ptr::null_mut();
        }
    }
}

static CORE: OnceLock<Result<CoreLib, String>> = OnceLock::new();

fn core_lib_path() -> Result<PathBuf, String> {
    resolve_lib_path(CORE_LIB_NAME)
}

/// 按名称在标准位置搜索库（exe 同目录 modules/ → exe 同目录 → target → JPCG_CORE_LIB env）
fn resolve_lib_path(lib_name: &str) -> Result<PathBuf, String> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("modules").join(lib_name));
            candidates.push(dir.join(lib_name));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        for dir in ["debug", "release"] {
            candidates.push(cwd.join("target").join(dir).join(lib_name));
        }
    }
    if let Ok(p) = std::env::var("JPCG_CORE_LIB") {
        candidates.push(PathBuf::from(p));
    }
    candidates.into_iter().find(|p| p.exists()).ok_or_else(|| {
        format!(
            "找不到 {}（动态模式），可设置 JPCG_CORE_LIB 环境变量指定路径",
            lib_name
        )
    })
}

fn core() -> Result<&'static CoreLib, String> {
    CORE.get_or_init(|| {
        let path = core_lib_path()?;
        let lib = unsafe { Library::new(&path) }
            .map_err(|e| format!("加载 {} 失败: {}", path.display(), e))?;
        unsafe {
            let jpcg_call: Symbol<'_, JpcgCallFn> =
                lib.get(b"jpcg_call").map_err(|e| e.to_string())?;
            let free: Symbol<'_, JpcgFreeStringFn> =
                lib.get(b"jpcg_free_string").map_err(|e| e.to_string())?;
            let last_err: Symbol<'_, JpcgLastErrorFn> =
                lib.get(b"jpcg_last_error").map_err(|e| e.to_string())?;
            let create: Symbol<'_, JpcgHandleCreateFn> =
                lib.get(b"jpcg_handle_create").map_err(|e| e.to_string())?;
            let free_handle: Symbol<'_, JpcgHandleFreeFn> =
                lib.get(b"jpcg_handle_free").map_err(|e| e.to_string())?;
            let version: Symbol<'_, JpcgVersionFn> =
                lib.get(b"jpcg_core_version").map_err(|e| e.to_string())?;
            let jpcg_call = *jpcg_call;
            let jpcg_free_string = *free;
            let jpcg_last_error = *last_err;
            let jpcg_handle_create = *create;
            let jpcg_handle_free = *free_handle;
            let jpcg_core_version = *version;
            let session_c = CString::new("{}").map_err(|e| e.to_string())?;
            let handle = jpcg_handle_create(session_c.as_ptr());
            Ok(CoreLib {
                _lib: lib,
                handle,
                jpcg_call,
                jpcg_handle_free,
                jpcg_free_string,
                jpcg_last_error,
                jpcg_core_version,
            })
        }
    })
    .as_ref()
    .map_err(|e| e.clone())
}

/// 调用 core 业务方法（请求对象序列化 + 响应反序列化）
pub fn call<T: Serialize, R: DeserializeOwned>(method: &str, req: &T) -> Result<R, String> {
    let req_json = serde_json::to_string(req).map_err(|e| e.to_string())?;
    call_json(method, &req_json)
}

/// 调用 core 业务方法（无请求体）
pub fn call_no_args<R: DeserializeOwned>(method: &str) -> Result<R, String> {
    call_json(method, "{}")
}

/// 调用 core 业务方法（原始 JSON 请求体）
pub fn call_json<R: DeserializeOwned>(method: &str, req_json: &str) -> Result<R, String> {
    let core = core()?;
    let method_c = CString::new(method).map_err(|e| e.to_string())?;
    let req_c = CString::new(req_json).map_err(|e| e.to_string())?;
    let resp = unsafe { (core.jpcg_call)(core.handle, method_c.as_ptr(), req_c.as_ptr()) };
    if resp.is_null() {
        let err_c = unsafe { (core.jpcg_last_error)() };
        let err = if err_c.is_null() {
            "core 返回空错误".to_string()
        } else {
            unsafe { CStr::from_ptr(err_c) }
                .to_string_lossy()
                .into_owned()
        };
        unsafe { (core.jpcg_free_string)(err_c) };
        return Err(err);
    }
    let s = unsafe { CStr::from_ptr(resp) }
        .to_string_lossy()
        .into_owned();
    unsafe { (core.jpcg_free_string)(resp) };
    serde_json::from_str(&s).map_err(|e| format!("响应解析失败: {}", e))
}

/// 读取 core dll 的模块版本号
pub fn core_version() -> Option<String> {
    let core = core().ok()?;
    let p = unsafe { (core.jpcg_core_version)() };
    if p.is_null() {
        return None;
    }
    let s = unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned();
    unsafe { (core.jpcg_free_string)(p) };
    Some(s)
}

/// 读取指定库的版本号（const / update），getter 返回的串用该库 free 符号释放
pub fn lib_version(lib_name: &str, getter: &[u8], freer: &[u8]) -> Option<String> {
    let path = resolve_lib_path(lib_name).ok()?;
    let lib = unsafe { Library::new(&path) }.ok()?;
    unsafe {
        let get: Symbol<'_, JpcgVersionFn> = lib.get(getter).ok()?;
        let free: Symbol<'_, JpcgFreeStringFn> = lib.get(freer).ok()?;
        let p = get();
        if p.is_null() {
            return None;
        }
        let s = unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned();
        free(p);
        Some(s)
    }
}

/// 各模块版本（动态模式）
pub fn module_versions() -> (String, String, String) {
    let core = core_version().unwrap_or_default();
    let r#const = lib_version(
        CONST_LIB_NAME,
        b"jpcg_const_version",
        b"jpcg_const_free_string",
    )
    .unwrap_or_default();
    let update = lib_version(
        UPDATE_LIB_NAME,
        b"jpcg_update_version",
        b"jpcg_update_free_string",
    )
    .unwrap_or_default();
    (core, r#const, update)
}

// ============================================================================
// 宿主事件回调（Tauri → core dll）
// ============================================================================

static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
static UPDATER_PATH_C: OnceLock<CString> = OnceLock::new();

fn store_app(app: &tauri::AppHandle) {
    let _ = APP.set(app.clone());
}

extern "C" fn on_progress_cb(event_json: *const c_char) {
    use tauri::Emitter;
    if let Some(app) = APP.get() {
        if !event_json.is_null() {
            if let Ok(s) = unsafe { CStr::from_ptr(event_json) }.to_str() {
                if let Ok(event) = serde_json::from_str::<jpcg_update::UpdateProgressEvent>(s) {
                    let _ = app.emit("update-progress", event);
                }
            }
        }
    }
}

extern "C" fn request_exit_cb() -> std::os::raw::c_int {
    if let Some(app) = APP.get() {
        app.exit(0);
        0
    } else {
        -1
    }
}

extern "C" fn updater_path_cb() -> *const c_char {
    if let Ok(p) = std::env::var("JPCG_UPDATER_PATH") {
        let c = UPDATER_PATH_C.get_or_init(|| CString::new(p).unwrap_or_default());
        return c.as_ptr();
    }
    std::ptr::null()
}

/// 注册宿主事件回调（幂等；perform_update / perform_app_update 前调用）
pub fn register_host_events(app: &tauri::AppHandle) -> Result<(), String> {
    static REGISTERED: OnceLock<()> = OnceLock::new();
    if REGISTERED.get().is_some() {
        return Ok(());
    }
    store_app(app);

    let path = core_lib_path()?;
    let lib = unsafe { Library::new(&path) }
        .map_err(|e| format!("加载 {} 失败: {}", path.display(), e))?;
    let result = unsafe {
        let set_events: Symbol<'_, JpcgSetEventsFn> = lib
            .get(b"jpcg_set_host_events")
            .map_err(|e| e.to_string())?;
        let table = HostEventsTable {
            on_progress: Some(on_progress_cb),
            request_exit: Some(request_exit_cb),
            updater_path: Some(updater_path_cb),
        };
        set_events(&table)
    };
    drop(lib);
    if result != 0 {
        return Err("注册宿主事件失败".to_string());
    }
    let _ = REGISTERED.set(());
    Ok(())
}

// ============================================================================
// 动态模式端到端冒烟测试（需先 cargo build -p jpcg_core 生成 dylib）
// ============================================================================

#[cfg(test)]
mod ffi_bridge_tests {
    use super::*;

    fn workspace_dylib() -> Option<PathBuf> {
        // 测试 CWD 为 src-tauri 目录；dylib 在 workspace target/debug 下
        for rel in ["../../target/debug", "../../../target/debug"] {
            let p = std::path::Path::new(rel).join(CORE_LIB_NAME);
            if p.exists() {
                return Some(p);
            }
        }
        None
    }

    #[test]
    fn dynamic_call_roundtrip() {
        let Some(lib) = workspace_dylib() else {
            eprintln!("跳过：未找到 {}", CORE_LIB_NAME);
            return;
        };
        unsafe { std::env::set_var("JPCG_CORE_LIB", &lib) };
        let out: Vec<jpcg_api::XinfaSummaryDTO> =
            call_no_args("list_professions").expect("动态调用失败");
        assert!(out.is_empty(), "测试环境无数据目录，应返回空数组");

        let err = call_json::<serde_json::Value>("no_such_method", "{}").expect_err("应返回错误");
        assert!(err.contains("未知方法"), "错误信息: {}", err);
    }
}
