pub mod drug;
pub mod food;

/// 本 const 模块库版本（等级.赛季.日期，如 130.3.20260602）
pub const CONST_VERSION: &str = env!("CARGO_PKG_VERSION");

// ============================================================================
// FFI — 跨语言 C ABI 接口（动态模式被 jpcg_core 或外部程序 dlopen）
// 当前常量为编译期数值，直接以数字返回。
// ============================================================================

use std::ffi::CString;
use std::os::raw::c_char;

static LAST_ERROR: std::sync::Mutex<Option<CString>> = std::sync::Mutex::new(None);

fn set_last_error(msg: &str) {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = Some(CString::new(msg).unwrap_or_default());
    }
}

/// 返回当前 FFI 协议版本
#[unsafe(no_mangle)]
pub extern "C" fn jpcg_const_abi_version() -> u32 {
    1
}

/// 返回本 const 模块库版本号（如 "130.3.20260602"，等级.赛季.日期）
/// 返回字符串需 jpcg_const_free_string 释放。
///
/// # Safety
/// - 返回指针需由 jpcg_const_free_string 释放
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_const_version() -> *mut c_char {
    CString::new(CONST_VERSION)
        .ok()
        .map(|c| c.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// 返回最近一次错误信息（如无错误返回空字符串）
#[unsafe(no_mangle)]
pub extern "C" fn jpcg_const_last_error() -> *mut c_char {
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
///
/// # Safety
/// - `ptr` 必须为本模块返回的指针或 null
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_const_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

/// 获取指定药物常量的数值
/// `key`: "nei_po_high" / "nei_po_low" / "nei_hui_high" / "nei_hui_low" / "nei_gong_high" / "nei_gong_low"
/// 返回常量值；未知 key 返回 0
///
/// # Safety
/// `key` 必须为 null 结尾的有效 UTF-8 C 字符串，或为 null。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_const_get_u32(key: *const c_char) -> u32 {
    use std::ffi::CStr;
    if key.is_null() {
        return 0;
    }
    let key = match unsafe { CStr::from_ptr(key) }.to_str() {
        Ok(k) => k,
        Err(_) => {
            set_last_error("key 为无效 UTF-8");
            return 0;
        }
    };
    match key {
        "nei_po_high" => drug::YAO_NEI_PO_HIGH,
        "nei_po_low" => drug::YAO_NEI_PO_LOW,
        "nei_hui_high" => drug::YAO_NEI_HUI_HIGH,
        "nei_hui_low" => drug::YAO_NEI_HUI_LOW,
        "nei_gong_high" => drug::YAO_NEI_GONG_HIGH,
        "nei_gong_low" => drug::YAO_NEI_GONG_LOW,
        _ => {
            set_last_error(&format!("未知常量: {}", key));
            0
        }
    }
}
