use jpcg_gear::{Catalog, Composition, Loadout};
use serde::Serialize;
use serde::de::DeserializeOwned;

#[cfg(feature = "static")]
fn call<T: Serialize, R: DeserializeOwned>(method: &str, input: &T) -> Result<R, String> {
    let input = serde_json::to_string(input).map_err(|e| e.to_string())?;
    serde_json::from_value(jpcg_gear::host::call(method, &input)?).map_err(|e| e.to_string())
}
#[cfg(feature = "dynamic")]
fn call<T: Serialize, R: DeserializeOwned>(method: &str, input: &T) -> Result<R, String> {
    super::gear_bridge::call(method, input)
}

#[tauri::command]
pub async fn gear_catalog() -> Result<Catalog, String> {
    tauri::async_runtime::spawn_blocking(|| call("catalog", &serde_json::json!({})))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn compose_gear(loadout: Loadout) -> Result<Composition, String> {
    tauri::async_runtime::spawn_blocking(move || call("compose", &loadout))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn export_gear(loadout: Loadout) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || call("export", &loadout))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn import_gear(toml: String) -> Result<Composition, String> {
    tauri::async_runtime::spawn_blocking(move || call("import", &serde_json::json!({"toml":toml})))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn optimize_gear(
    app: tauri::AppHandle,
    request: jpcg_gear::OptimizeRequest,
) -> Result<jpcg_gear::OptimizeResult, String> {
    use std::sync::{Arc, atomic::Ordering};
    use tauri::{Emitter, Manager};
    let progress = Arc::new(jpcg_gear::OptimizeProgress::new());
    {
        let state = app.state::<crate::OptimizeState>();
        let mut guard = state.progress.lock().map_err(|_| "调优状态锁污染")?;
        if guard.is_some() {
            return Err("已有调优任务正在运行".into());
        }
        *guard = Some(Arc::clone(&progress));
    }
    let monitor = {
        let progress = Arc::clone(&progress);
        let app = app.clone();
        std::thread::spawn(move || {
            let mut last = u64::MAX;
            loop {
                let pct = progress.percent();
                if pct != last {
                    last = pct;
                    let _ = app.emit("gear-optimize-progress", pct);
                }
                if progress.done.load(Ordering::Relaxed) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        })
    };
    let p = Arc::clone(&progress);
    let result = tauri::async_runtime::spawn_blocking(move || optimize_impl(request, p))
        .await
        .map_err(|e| e.to_string());
    progress.done.store(true, Ordering::Relaxed);
    let _ = monitor.join();
    {
        let state = app.state::<crate::OptimizeState>();
        if let Ok(mut guard) = state.progress.lock() {
            *guard = None;
        }
    }
    result?
}

#[cfg(feature = "static")]
fn optimize_impl(
    request: jpcg_gear::OptimizeRequest,
    progress: std::sync::Arc<jpcg_gear::OptimizeProgress>,
) -> Result<jpcg_gear::OptimizeResult, String> {
    let catalog = jpcg_gear::host::catalog()?;
    jpcg_gear::optimize_joint(&catalog, &request, &progress)
}

#[cfg(feature = "dynamic")]
fn optimize_impl(
    request: jpcg_gear::OptimizeRequest,
    progress: std::sync::Arc<jpcg_gear::OptimizeProgress>,
) -> Result<jpcg_gear::OptimizeResult, String> {
    use std::sync::atomic::Ordering;
    let id: jpcg_gear::OptimizeJobId = call("optimize_joint_start", &request)?;
    struct ReleaseJob(jpcg_gear::OptimizeJobId);
    impl Drop for ReleaseJob {
        fn drop(&mut self) {
            let _ = call::<_, serde_json::Value>("optimize_joint_release", &self.0);
        }
    }
    let release = ReleaseJob(id);
    let mut cancellation_sent = false;
    loop {
        if progress.cancel.load(Ordering::Relaxed) && !cancellation_sent {
            let _: serde_json::Value = call("optimize_joint_cancel", &release.0)?;
            cancellation_sent = true;
        }
        let status: jpcg_gear::OptimizeJobStatus = call("optimize_joint_poll", &release.0)?;
        progress
            .total
            .store(status.progress.combinations, Ordering::Relaxed);
        progress
            .visited
            .store(status.progress.visited, Ordering::Relaxed);
        progress
            .evaluated
            .store(status.progress.evaluated, Ordering::Relaxed);
        if status.done {
            if let Some(error) = status.error {
                return Err(error);
            }
            return status.result.ok_or("联合搜索未返回结果".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

#[tauri::command]
pub fn cancel_gear_optimize(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let state = app.state::<crate::OptimizeState>();
    let guard = state.progress.lock().map_err(|_| "调优状态锁污染")?;
    if let Some(progress) = guard.as_ref() {
        progress.cancel();
    }
    Ok(())
}
