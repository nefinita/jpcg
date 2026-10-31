//! 无会话的配装 C ABI；调用与字符串释放必须来自同一动态库。panic 不跨 ABI。
use std::ffi::{CStr, CString, c_char};

#[unsafe(no_mangle)]
pub extern "C" fn jpcg_gear_abi_version() -> u32 {
    1
}

fn string(value: String) -> *mut c_char {
    CString::new(value)
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

/// # Safety
/// method/input 必须为有效、NUL 结尾的 UTF-8 字符串，或 null（返回错误）。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_gear_call(
    method: *const c_char,
    input: *const c_char,
) -> *mut c_char {
    let result = std::panic::catch_unwind(|| {
        if method.is_null() || input.is_null() {
            return Err("配装请求指针为空".into());
        }
        let method = unsafe { CStr::from_ptr(method) }
            .to_str()
            .map_err(|e| e.to_string())?;
        let input = unsafe { CStr::from_ptr(input) }
            .to_str()
            .map_err(|e| e.to_string())?;
        crate::host::call(method, input)
    })
    .unwrap_or_else(|_| Err("配装模块内部错误".into()));
    let response = match result {
        Ok(result) => crate::ApiResponse::Ok { result },
        Err(message) => crate::ApiResponse::Error { message },
    };
    match serde_json::to_string(&response) {
        Ok(value) => string(value),
        Err(_) => std::ptr::null_mut(),
    }
}

/// # Safety
/// ptr 必须为本库返回且尚未释放的字符串，或 null。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jpcg_gear_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(unsafe { CString::from_raw(ptr) });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn jpcg_gear_version() -> *mut c_char {
    string(env!("CARGO_PKG_VERSION").into())
}
