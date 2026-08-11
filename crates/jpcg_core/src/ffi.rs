// ============================================================================
// ffi — 跨语言 C ABI 接口（句柄 + JSON）
// 供外部程序（Python 等）与动态模式 Tauri 壳（dlopen libjpcg_core.dylib）调用。
// 协议约定：
//   - 所有输入输出均为 UTF-8 JSON 字符串（经 CString 传递，调用方释放）
//   - jpcg_handle_create: 用 JSON 配置创建会话句柄
//   - jpcg_handle_free:   释放句柄
//   - jpcg_call:          调用 host 业务函数（method = "calculate" / "combo" / ...）
//   - abi_version:        返回协议版本号，用于版本协商
//   - 实现通过 catch_unwind 防护，panic 不跨 FFI 边界
// ============================================================================

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// 当前 FFI 协议版本
pub const ABI_VERSION: u32 = 1;

// ============================================================================
// 宿主事件回调表（动态模式）
// Tauri 壳等附加组件通过 jpcg_set_host_events 注册 C 回调，
// core 在 update 编排（perform_update / perform_app_update）中调用。
// 静态模式下无需注册（Tauri 壳直接以 Rust trait 注入）。
// 表类型定义在 jpcg_api（跨端 C ABI 契约单源）。
// ============================================================================

pub use jpcg_api::HostEventsTable;

static HOST_EVENTS: std::sync::Mutex<Option<HostEventsTable>> = std::sync::Mutex::new(None);

/// 读取当前回调表（无则返回空表）
fn host_events_table() -> HostEventsTable {
    HOST_EVENTS
        .lock()
        .ok()
        .and_then(|g| *g)
        .unwrap_or(HostEventsTable {
            on_progress: None,
            request_exit: None,
            updater_path: None,
        })
}

/// 注册宿主事件回调表（动态模式专用；静态模式无需调用）
///
/// # Safety
/// - `table` 必须指向有效且存活期内不变的 HostEventsTable
/// - 传入 null 视为清除回调
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_set_host_events(table: *const HostEventsTable) -> c_int {
    if table.is_null() {
        if let Ok(mut guard) = HOST_EVENTS.lock() {
            *guard = None;
        }
        return 0;
    }
    let t = unsafe { *table };
    match HOST_EVENTS.lock() {
        Ok(mut guard) => {
            *guard = Some(t);
            0
        }
        Err(_) => -1,
    }
}

/// 基于回调表的 HostEvents 实现（供 update 编排在动态模式使用）
pub(crate) struct FfiHostEvents;

impl crate::host::update::HostEvents for FfiHostEvents {
    fn on_progress(&self, event: &jpcg_update::UpdateProgressEvent) {
        let table = host_events_table();
        if let Some(cb) = table.on_progress {
            if let Ok(json) = serde_json::to_string(event) {
                if let Ok(c) = CString::new(json) {
                    unsafe {
                        cb(c.as_ptr());
                    }
                }
            }
        }
    }

    fn request_exit(&self) {
        let table = host_events_table();
        if let Some(cb) = table.request_exit {
            unsafe {
                cb();
            }
        }
    }

    fn updater_path(&self) -> Option<std::path::PathBuf> {
        let table = host_events_table();
        if let Some(cb) = table.updater_path {
            let ptr = unsafe { cb() };
            if ptr.is_null() {
                return None;
            }
            let s = unsafe { CStr::from_ptr(ptr) }.to_str().ok()?;
            Some(std::path::PathBuf::from(s))
        } else {
            None
        }
    }
}

// ============================================================================
// 句柄与 JSON 协议
// ============================================================================

static LAST_ERROR: std::sync::Mutex<Option<CString>> = std::sync::Mutex::new(None);

/// 会话句柄
pub struct JpcgHandle {
    /// 建句柄时传入的会话配置 JSON（兼容原始语义；业务输入在 jpcg_call 的 request 里）
    _session_config: String,
}

/// 构造 C 字符串（调用方需 jpcg_free_string 释放）
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
    unsafe { CStr::from_ptr(ptr) }.to_str().ok().map(|s| s.to_string())
}

/// 业务方法分发：method + request_json → response_json
fn dispatch(method: &str, request: &str) -> Result<String, String> {
    match method {
        "calculate" => {
            let req: jpcg_api::CalculateRequest =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            let out = crate::host::calc::calculate(req)?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "compute_derivatives" => {
            let req: jpcg_api::CalculateRequest =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            let out = crate::host::calc::compute_derivatives(req)?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
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
            let out = crate::host::combo::calculate_combo(
                req.steps,
                req.player,
                req.hostile,
                req.xinfa,
                req.buff,
                req.coefficient,
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
            let out = crate::host::combo::save_combo_preset(req.name, req.steps)?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "list_combo_presets" => {
            let out = crate::host::combo::list_combo_presets();
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "load_combo_preset" => {
            #[derive(serde::Deserialize)]
            struct ComboPresetReq {
                name: String,
            }
            let req: ComboPresetReq =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            let out = crate::host::combo::load_combo_preset(req.name)?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "delete_combo_preset" => {
            #[derive(serde::Deserialize)]
            struct ComboPresetReq {
                name: String,
            }
            let req: ComboPresetReq =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            let out = crate::host::combo::delete_combo_preset(req.name)?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "export_config" => {
            let out = crate::host::combo::export_config()?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "import_config" => {
            #[derive(serde::Deserialize)]
            struct ImportConfigReq {
                toml_str: String,
            }
            let req: ImportConfigReq =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            let out = crate::host::combo::import_config(req.toml_str)?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "save_config" => {
            #[derive(serde::Deserialize)]
            struct SaveConfigReq {
                player: jpcg_api::PlayerConfigDTO,
                hostile: jpcg_api::HostileConfigDTO,
                xinfa: jpcg_api::XinfaConfigDTO,
            }
            let req: SaveConfigReq =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            crate::host::config::save_config(req.player, req.hostile, req.xinfa);
            serde_json::to_string(&serde_json::Value::Null)
                .map_err(|e| format!("响应序列化失败: {}", e))
        }
        "load_config" => {
            let out = crate::host::config::load_config();
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "list_professions" => {
            let out = crate::host::config::list_professions();
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "load_skill_data" => {
            #[derive(serde::Deserialize)]
            struct ProfessionRequest {
                profession: String,
            }
            let req: ProfessionRequest =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            let out = crate::host::skill::load_skill_data(req.profession)?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "save_skill_data" => {
            #[derive(serde::Deserialize)]
            struct SkillDataRequest {
                profession: String,
                data: jpcg_api::SkillEditorDataDTO,
            }
            let req: SkillDataRequest =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            let out = crate::host::skill::save_skill_data(req.profession, req.data)?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        "load_skill_pool" => {
            #[derive(serde::Deserialize)]
            struct ProfessionRequest {
                profession: String,
            }
            let req: ProfessionRequest =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            let out = crate::host::skill::load_skill_pool(req.profession);
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        #[cfg(feature = "net")]
        "update_check" => {
            #[derive(serde::Deserialize)]
            struct UpdateCheckRequest {
                beta: Option<bool>,
                force: Option<bool>,
            }
            let req: UpdateCheckRequest =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            let out = crate::host::update::check_update(
                req.beta.unwrap_or(false),
                req.force.unwrap_or(false),
            )?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        #[cfg(feature = "net")]
        "update_perform" => {
            #[derive(serde::Deserialize)]
            struct UpdatePerformRequest {
                beta: Option<bool>,
                has_data_update: Option<bool>,
                latest_data_version: Option<String>,
                data_files_to_update: Option<Vec<String>>,
            }
            let req: UpdatePerformRequest =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            let out = crate::host::update::perform_update(
                &FfiHostEvents,
                req.beta.unwrap_or(false),
                req.has_data_update.unwrap_or(false),
                req.latest_data_version,
                req.data_files_to_update.unwrap_or_default(),
            )?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        #[cfg(feature = "net")]
        "update_app" => {
            #[derive(serde::Deserialize)]
            struct UpdateAppRequest {
                beta: Option<bool>,
            }
            let req: UpdateAppRequest =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            let out =
                crate::host::update::perform_app_update(&FfiHostEvents, req.beta.unwrap_or(false))?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        #[cfg(feature = "net")]
        "update_modules" => {
            #[derive(serde::Deserialize)]
            struct UpdateModulesRequest {
                beta: Option<bool>,
                modules_version: Option<String>,
                modules_files_to_update: Vec<jpcg_update::modules::ModulesFileEntry>,
            }
            let req: UpdateModulesRequest =
                serde_json::from_str(request).map_err(|e| format!("请求解析失败: {}", e))?;
            let out = crate::host::update::perform_modules_update(
                &FfiHostEvents,
                req.beta.unwrap_or(false),
                req.modules_version,
                req.modules_files_to_update,
            )?;
            serde_json::to_string(&out).map_err(|e| format!("响应序列化失败: {}", e))
        }
        _ => Err(format!("未知方法: {}", method)),
    }
}

/// 调用业务方法（协议核心入口）
///
/// # Safety
/// - `handle` 必须来自 jpcg_handle_create 且未被释放
/// - `method` / `request` 必须为 null 结尾的 UTF-8 字符串
/// - 返回的 CString 必须由调用方通过 jpcg_free_string 释放
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_call(
    handle: *mut JpcgHandle,
    method: *const c_char,
    request: *const c_char,
) -> *mut c_char {
    let result = std::panic::catch_unwind(|| {
        if handle.is_null() {
            return Err(String::from("句柄为空"));
        }
        let _h = unsafe { &*handle };
        let method_s = unsafe { cstr_to_string(method) }.unwrap_or_default();
        let request_s = unsafe { cstr_to_string(request) }.unwrap_or_default();
        dispatch(&method_s, &request_s)
    });
    match result {
        Ok(Ok(resp)) => cstring_out(&resp),
        Ok(Err(e)) => {
            set_last_error(&e);
            std::ptr::null_mut()
        }
        Err(_) => {
            set_last_error("内部 panic（已通过 catch_unwind 拦截）");
            std::ptr::null_mut()
        }
    }
}

/// 创建会话句柄（session_config 当前仅存储，业务输入在调用时传入）
///
/// # Safety
/// - `session_config` 必须为 null 结尾的 UTF-8 字符串
/// - 返回的句柄必须由 jpcg_handle_free 释放
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_handle_create(session_config: *const c_char) -> *mut JpcgHandle {
    let config = unsafe { cstr_to_string(session_config) }.unwrap_or_default();
    Box::into_raw(Box::new(JpcgHandle {
        _session_config: config,
    }))
}

/// 释放会话句柄（null 安全）
///
/// # Safety
/// - `handle` 必须来自 jpcg_handle_create 或为 null
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_handle_free(handle: *mut JpcgHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

/// 释放 core 返回的字符串（null 安全）
///
/// # Safety
/// - `s` 必须为 jpcg_call 返回的指针或为 null
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

/// 读取上次错误信息（jpcg_call 返回 null 后调用），返回字符串需 jpcg_free_string 释放
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_last_error() -> *mut c_char {
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
pub extern "C" fn jpcg_abi_version() -> u32 {
    ABI_VERSION
}

// ============================================================================
// FFI 协议冒烟测试（无需数据文件）
// 数值一致性由 engine 层金标准锁定；此处验证句柄生命周期、JSON 往返、
// 错误路径与版本协商。
// ============================================================================

#[cfg(test)]
mod ffi_tests {
    use super::*;
    use serde_json::Value;

    unsafe fn call_owned(method: &str, req: &str) -> Result<String, String> {
        let handle = unsafe { jpcg_handle_create(cstring("{}").as_ptr()) };
        assert!(!handle.is_null());
        let method_c = cstring(method);
        let req_c = cstring(req);
        let resp = unsafe { jpcg_call(handle, method_c.as_ptr(), req_c.as_ptr()) };
        unsafe { jpcg_handle_free(handle) };
        if resp.is_null() {
            let err_c = unsafe { jpcg_last_error() };
            let err = unsafe { cstr_to_string(err_c) }.unwrap_or_default();
            unsafe { jpcg_free_string(err_c) };
            return Err(err);
        }
        let out = unsafe { cstr_to_string(resp) }.unwrap_or_default();
        unsafe { jpcg_free_string(resp) };
        Ok(out)
    }

    fn cstring(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn abi_version_ok() {
        assert_eq!(super::jpcg_abi_version(), 1);
    }

    #[test]
    fn handle_lifecycle_ok() {
        let c = cstring("{}");
        let h = unsafe { jpcg_handle_create(c.as_ptr()) };
        assert!(!h.is_null());
        unsafe { jpcg_handle_free(h) };
        unsafe { jpcg_handle_free(std::ptr::null_mut()) };
    }

    #[test]
    fn list_professions_roundtrip() {
        let out = unsafe { call_owned("list_professions", "{}") }.expect("调用失败");
        let v: Value = serde_json::from_str(&out).expect("响应应为合法 JSON");
        assert!(v.is_array());
    }

    #[test]
    fn unknown_method_sets_error() {
        let err = unsafe { call_owned("no_such_method", "{}") }.expect_err("应返回错误");
        assert!(err.contains("未知方法"), "错误信息: {}", err);
    }

    #[test]
    fn invalid_request_json_sets_error() {
        let err = unsafe { call_owned("calculate", "not json") }.expect_err("应返回错误");
        assert!(err.contains("解析失败"), "错误信息: {}", err);
    }

    #[test]
    fn save_config_roundtrip() {
        // 不落盘验证 JSON 契约，仅确认方法可达（会写 saved_config.toml 到 CWD）
        let req = r#"{
            "player": {"jcsx":"gengu","jichu_shuxing":18888,"jichu_gongji":4666,"huixin_dengji":33000,"huixin_xiaoguo":22000,"pofang_dengji":25000,"wuqi_shanghai":2800},
            "hostile": {"waigong_fangyu":21000,"neigong_fangyu":21000,"yujin_dengji":8500,"huajin_dengji":35000,"jianshang_bili":35,"target_hp":200},
            "xinfa": {"profession":"mowen","xinfa_name":"莫问","xinfa_nom":"根骨","atk_up":1.96,"pofang_up":2.0,"huixin_up":0.0}
        }"#;
        let out = unsafe { call_owned("save_config", req) }.expect("调用失败");
        assert_eq!(out, "null");
        let _ = std::fs::remove_file("saved_config.toml");
    }
}
