use crate::commands::types::*;

#[tauri::command]
pub async fn compute_derivatives(req: CalculateRequest) -> Result<DerivativesOutputDTO, String> {
    derivatives_impl(req)
}

#[cfg(feature = "static")]
fn derivatives_impl(req: CalculateRequest) -> Result<DerivativesOutputDTO, String> {
    jpcg_core::host::calc::compute_derivatives(req)
}

#[cfg(feature = "dynamic")]
fn derivatives_impl(req: CalculateRequest) -> Result<DerivativesOutputDTO, String> {
    crate::commands::ffi_bridge::call("compute_derivatives", &req)
}
