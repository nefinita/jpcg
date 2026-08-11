// ============================================================================
// ffi — 跨语言 C ABI 接口（同步化）
// jpcg_update 的公开 API 均为 async；FFI 层用内嵌 tokio runtime 同步化，
// 供动态模式被 jpcg_core dlopen 或外部程序调用。
// 协议：JSON 进 → JSON 出，错误经 jpcg_update_last_error 获取。
// ============================================================================

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

static LAST_ERROR: std::sync::Mutex<Option<CString>> = std::sync::Mutex::new(None);

fn set_last_error(msg: &str) {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = Some(CString::new(msg).unwrap_or_default());
    }
}

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    let rt = tokio::runtime::Runtime::new().expect("创建 tokio runtime 失败");
    rt.block_on(f)
}

/// 当前 FFI 协议版本
#[unsafe(no_mangle)]
pub extern "C" fn jpcg_update_abi_version() -> u32 {
    1
}

/// 返回最近一次错误信息
#[unsafe(no_mangle)]
pub extern "C" fn jpcg_update_last_error() -> *mut c_char {
    let msg = match LAST_ERROR.lock() {
        Ok(guard) => guard.as_ref().map(|c| c.to_str().unwrap_or("").to_string()),
        Err(_) => None,
    };
    match CString::new(msg.as_deref().unwrap_or("")) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// 释放由本模块分配的 C 字符串
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_update_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

fn cstring_out(s: &str) -> *mut c_char {
    match CString::new(s) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

unsafe fn cstr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }.to_str().ok().map(|s| s.to_string())
}

/// 检查更新（同步封装）
/// request: JSON `{"base_path":".","beta":false,"force":false}`
/// 返回: JSON 序列化的 UpdateCheckResult
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_update_check(request: *const c_char) -> *mut c_char {
    let result = std::panic::catch_unwind(|| {
        let req = unsafe { cstr_to_string(request) }.unwrap_or_default();
        #[derive(serde::Deserialize)]
        struct CheckRequest {
            base_path: Option<String>,
            beta: Option<bool>,
            force: Option<bool>,
        }
        let parsed: CheckRequest = match serde_json::from_str(&req) {
            Ok(p) => p,
            Err(e) => {
                set_last_error(&format!("请求解析失败: {}", e));
                return std::ptr::null_mut();
            }
        };
        let base_path = std::path::PathBuf::from(parsed.base_path.unwrap_or_else(|| ".".to_string()));
        match block_on(crate::check_updates(&base_path, parsed.beta.unwrap_or(false), parsed.force.unwrap_or(false))) {
            Ok(result) => match serde_json::to_string(&result) {
                Ok(s) => cstring_out(&s),
                Err(e) => {
                    set_last_error(&format!("响应序列化失败: {}", e));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                set_last_error(&format!("检查更新失败: {}", e));
                std::ptr::null_mut()
            }
        }
    });
    match result {
        Ok(ptr) => ptr,
        Err(_) => {
            set_last_error("内部 panic（已通过 catch_unwind 拦截）");
            std::ptr::null_mut()
        }
    }
}

/// 获取应用更新信息（同步封装）
/// request: JSON `{"base_path":".","beta":false,"force":false}`
/// 返回: JSON 序列化的 AppUpdateInfo 或 "null"
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_update_fetch_app_info(request: *const c_char) -> *mut c_char {
    let result = std::panic::catch_unwind(|| {
        let req = unsafe { cstr_to_string(request) }.unwrap_or_default();
        #[derive(serde::Deserialize)]
        struct CheckRequest {
            base_path: Option<String>,
            beta: Option<bool>,
            force: Option<bool>,
        }
        let parsed: CheckRequest = match serde_json::from_str(&req) {
            Ok(p) => p,
            Err(e) => {
                set_last_error(&format!("请求解析失败: {}", e));
                return std::ptr::null_mut();
            }
        };
        let base_path = std::path::PathBuf::from(parsed.base_path.unwrap_or_else(|| ".".to_string()));
        match block_on(crate::fetch_app_update_info(&base_path, parsed.beta.unwrap_or(false), parsed.force.unwrap_or(false))) {
            Ok(Some(info)) => match serde_json::to_string(&info) {
                Ok(s) => cstring_out(&s),
                Err(e) => {
                    set_last_error(&format!("响应序列化失败: {}", e));
                    std::ptr::null_mut()
                }
            },
            Ok(None) => cstring_out("null"),
            Err(e) => {
                set_last_error(&format!("获取更新信息失败: {}", e));
                std::ptr::null_mut()
            }
        }
    });
    match result {
        Ok(ptr) => ptr,
        Err(_) => {
            set_last_error("内部 panic（已通过 catch_unwind 拦截）");
            std::ptr::null_mut()
        }
    }
}

/// 计算文件 SHA256（同步封装）
/// request: JSON `{"path":"/abs/path"}`
/// 返回: JSON `{"hash":"..."}`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_update_file_sha256(request: *const c_char) -> *mut c_char {
    let result = std::panic::catch_unwind(|| {
        let req = unsafe { cstr_to_string(request) }.unwrap_or_default();
        #[derive(serde::Deserialize)]
        struct ShaRequest {
            path: String,
        }
        let parsed: ShaRequest = match serde_json::from_str(&req) {
            Ok(p) => p,
            Err(e) => {
                set_last_error(&format!("请求解析失败: {}", e));
                return std::ptr::null_mut();
            }
        };
        match block_on(crate::calculate_file_sha256(std::path::Path::new(&parsed.path))) {
            Ok(hash) => match serde_json::json!({"hash": hash}).to_string() {
                s => cstring_out(&s),
            },
            Err(e) => {
                set_last_error(&format!("计算哈希失败: {}", e));
                std::ptr::null_mut()
            }
        }
    });
    match result {
        Ok(ptr) => ptr,
        Err(_) => {
            set_last_error("内部 panic（已通过 catch_unwind 拦截）");
            std::ptr::null_mut()
        }
    }
}