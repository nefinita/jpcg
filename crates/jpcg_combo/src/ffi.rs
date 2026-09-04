// ============================================================================
// ffi — 连招引擎跨语言 C ABI 接口（句柄 + JSON，协议与 jpcg_core 一致）
// 供动态模式 Tauri 壳（dlopen libjpcg_combo）与外部程序调用。
// 协议约定：
//   - 所有输入输出均为 UTF-8 JSON 字符串（经 CString 传递，调用方释放）
//   - jpcg_combo_handle_create / jpcg_combo_handle_free / jpcg_combo_call /
//     jpcg_combo_last_error / jpcg_combo_free_string
//   - 实现通过 catch_unwind 防护，panic 不跨 FFI 边界
// ============================================================================

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::COMBO_VERSION;
use crate::engine::ComboConfig;

/// 当前 FFI 协议版本
pub const ABI_VERSION: u32 = 1;

static LAST_ERROR: std::sync::Mutex<Option<CString>> = std::sync::Mutex::new(None);

/// 会话句柄（当前仅占位，业务输入在调用时传入）
pub struct ComboHandle {
    _session_config: String,
}

/// 构造 C 字符串（调用方需 jpcg_combo_free_string 释放）
fn cstring_out(s: &str) -> *mut c_char {
    match CString::new(s) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn set_last_error(e: &str) {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = CString::new(e).ok();
    }
}

unsafe fn cstr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(|s| s.to_string())
}

/// 业务方法分发：method + request_json → response_json
fn dispatch(method: &str, request: &str) -> Result<String, String> {
    match method {
        "calculate_combo" => {
            #[derive(serde::Deserialize)]
            struct ComboRequest {
                steps: Vec<jpcg_api::ComboStepDTO>,
                player: jpcg_api::PlayerConfigDTO,
                hostile: jpcg_api::HostileConfigDTO,
                xinfa: jpcg_api::XinfaConfigDTO,
                buff: jpcg_api::BuffConfigDTO,
                coefficient: jpcg_api::CoefficientConfigDTO,
            }
            let req: ComboRequest =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            let out = crate::host::calculate_combo(
                req.steps,
                req.player,
                req.hostile,
                req.xinfa,
                req.buff,
                req.coefficient,
                ComboConfig::default(),
            )?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "save_combo_preset" => {
            #[derive(serde::Deserialize)]
            struct ComboPresetReq {
                name: String,
                steps: Vec<jpcg_api::ComboStepDTO>,
            }
            let req: ComboPresetReq =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            crate::host::save_combo_preset(req.name, req.steps)?;
            serde_json::to_string(&()).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "list_combo_presets" => {
            let out = crate::host::list_combo_presets();
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "load_combo_preset" => {
            #[derive(serde::Deserialize)]
            struct ComboPresetReq {
                name: String,
            }
            let req: ComboPresetReq =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            let out = crate::host::load_combo_preset(req.name)?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "delete_combo_preset" => {
            #[derive(serde::Deserialize)]
            struct ComboPresetReq {
                name: String,
            }
            let req: ComboPresetReq =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            crate::host::delete_combo_preset(req.name)?;
            serde_json::to_string(&()).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "export_config" => {
            let out = crate::host::export_config()?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "import_config" => {
            #[derive(serde::Deserialize)]
            struct ImportConfigReq {
                toml_str: String,
            }
            let req: ImportConfigReq =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            crate::host::import_config(req.toml_str)?;
            serde_json::to_string(&()).map_err(|e| format!("响应序列化失败: {}", e))
        }
        _ => Err(format!("未知方法: {}", method)),
    }
}

/// 统一调用入口
///
/// # Safety
/// - `handle` 必须来自 jpcg_combo_handle_create（或 null，null 时以空会话执行）
/// - `method` / `request_json` 必须为 null 结尾的 UTF-8 字符串
/// - 返回的字符串必须由 jpcg_combo_free_string 释放
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_combo_call(
    handle: *mut ComboHandle,
    method: *const c_char,
    request_json: *const c_char,
) -> *mut c_char {
    let _ = handle; // 会话占用，业务输入在调用时传入
    let method = match unsafe { cstr_to_string(method) } {
        Some(m) => m,
        None => {
            set_last_error("method 为 null");
            return std::ptr::null_mut();
        }
    };
    let request = unsafe { cstr_to_string(request_json) }.unwrap_or_default();
    let result = std::panic::catch_unwind(|| dispatch(&method, &request));
    match result {
        Ok(Ok(out)) => cstring_out(&out),
        Ok(Err(e)) => {
            set_last_error(&e);
            std::ptr::null_mut()
        }
        Err(_) => {
            set_last_error("panic 已捕获");
            std::ptr::null_mut()
        }
    }
}

/// 创建会话句柄（session_config 当前仅存储）
///
/// # Safety
/// - `session_config` 必须为 null 结尾的 UTF-8 字符串（可 null）
/// - 返回的句柄必须由 jpcg_combo_handle_free 释放
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_combo_handle_create(
    session_config: *const c_char,
) -> *mut ComboHandle {
    let config = unsafe { cstr_to_string(session_config) }.unwrap_or_default();
    Box::into_raw(Box::new(ComboHandle {
        _session_config: config,
    }))
}

/// 释放会话句柄（null 安全）
///
/// # Safety
/// - `handle` 必须来自 jpcg_combo_handle_create 或为 null
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_combo_handle_free(handle: *mut ComboHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

/// 释放引擎返回的字符串（null 安全）
///
/// # Safety
/// - `s` 必须为 jpcg_combo_call 返回的指针或为 null
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_combo_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

/// 读取上次错误信息（jpcg_combo_call 返回 null 后调用），返回字符串需 jpcg_combo_free_string 释放
///
/// # Safety
/// - 返回指针需由 jpcg_combo_free_string 释放（可为 null）
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_combo_last_error() -> *mut c_char {
    let s = match LAST_ERROR.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => None,
    };
    match s {
        Some(c) => c.into_raw(),
        None => std::ptr::null_mut(),
    }
}

/// 返回 FFI 协议版本号（版本协商用）
#[unsafe(no_mangle)]
pub extern "C" fn jpcg_combo_abi_version() -> u32 {
    ABI_VERSION
}

/// 返回本引擎库版本号（如 "2.1.0"），返回字符串需 jpcg_combo_free_string 释放
/// 供宿主 UI 展示各 dll 版本。
#[unsafe(no_mangle)]
pub extern "C" fn jpcg_combo_version() -> *mut c_char {
    cstring_out(COMBO_VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(method: &str, request: &str) -> String {
        let handle = unsafe { jpcg_combo_handle_create(std::ptr::null()) };
        let method_c = CString::new(method).unwrap();
        let req_c = CString::new(request).unwrap();
        let out = unsafe { jpcg_combo_call(handle, method_c.as_ptr(), req_c.as_ptr()) };
        let s = unsafe {
            if out.is_null() {
                "".to_string()
            } else {
                CStr::from_ptr(out).to_string_lossy().into_owned()
            }
        };
        unsafe {
            jpcg_combo_free_string(out);
            jpcg_combo_handle_free(handle);
        }
        s
    }

    #[test]
    fn abi_version_ok() {
        assert_eq!(jpcg_combo_abi_version(), 1);
    }

    #[test]
    fn unknown_method_reports_error() {
        let out = call("nope", "{}");
        assert!(out.is_empty());
        let err = unsafe { jpcg_combo_last_error() };
        if !err.is_null() {
            let e = unsafe { CStr::from_ptr(err) }
                .to_string_lossy()
                .into_owned();
            unsafe { jpcg_combo_free_string(err) };
            assert!(e.contains("未知方法"));
        }
    }

    #[test]
    fn calculate_combo_roundtrip() {
        let req = r#"{
            "steps": [{"skill": {"skill_name": "宫", "skill_id": 1, "sub_id": 1, "base_damage1": 160, "base_damage2": 200, "atk_xishu": 2.609375, "has_critical_strike": true}}],
            "player": {"jcsx": "gengu", "jichu_shuxing": 21371, "jichu_gongji": 64329, "huixin_dengji": 61877, "huixin_xiaoguo": 2925, "pofang_dengji": 109160, "wuqi_shanghai": 0},
            "hostile": {"waigong_fangyu": 15176, "neigong_fangyu": 21388, "yujin_dengji": 5047, "huajin_dengji": 59402, "jianshang_bili": 0, "target_hp": 2000000, "max_hp": 0, "current_hp": 0},
            "xinfa": {"profession": "mowen", "xinfa_name": "莫问", "xinfa_nom": "gengu", "atk_up": 1.96, "pofang_up": 2.0, "huixin_up": 0.0},
            "buff": {},
            "coefficient": {"pofang_xishu": 225957.6, "huixin_xishu": 197703.0, "huixiao_xishu": 72844.2, "huajin_xishu": 30115.8, "fangyu_xishu": 126007.2, "pvp_global_jianshang": 0.9}
        }"#;
        let out = call("calculate_combo", req);
        assert!(out.contains("final_kill_prob"), "应返回击杀率: {}", out);
        assert!(out.contains("kill_prob_curve"));
        assert!(out.contains("steps"));
    }
}
