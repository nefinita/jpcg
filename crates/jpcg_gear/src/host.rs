//! 桌面宿主入口。前端只能传配装或 TOML 内容，不能指定文件路径。
use crate::{Catalog, Composition, Loadout};
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;

pub fn catalog_dir() -> Result<PathBuf, String> {
    jpcg_core::data_paths::gear_dir()
        .ok_or("找不到配装模板，请安装 gear 数据或设置 JPCG_DATA_DIR".into())
}

pub fn catalog() -> Result<Catalog, String> {
    Catalog::load(&catalog_dir()?)
}
pub fn compose(catalog: &Catalog, loadout: &Loadout) -> Result<Composition, String> {
    let composed = catalog.compose(loadout)?;
    Ok(Composition {
        player: crate::player(&composed),
        composed,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Import {
    toml: String,
}

pub fn dispatch(catalog: &Catalog, method: &str, input: &str) -> Result<Value, String> {
    if let Some(result) = job_dispatch(method, input) {
        return result;
    }
    if method == "optimize_joint_start" {
        return start_joint(catalog, input);
    }
    let value = match method {
        "catalog" => {
            let _: Empty = serde_json::from_str(input).map_err(|e| e.to_string())?;
            catalog.validate()?;
            serde_json::to_value(catalog)
        }
        "compose" => {
            let loadout: Loadout = serde_json::from_str(input).map_err(|e| e.to_string())?;
            serde_json::to_value(compose(catalog, &loadout)?)
        }
        "export" => {
            let loadout: Loadout = serde_json::from_str(input).map_err(|e| e.to_string())?;
            catalog.compose(&loadout)?;
            serde_json::to_value(toml::to_string_pretty(&loadout).map_err(|e| e.to_string())?)
        }
        "import" => {
            let request: Import = serde_json::from_str(input).map_err(|e| e.to_string())?;
            let loadout: Loadout = toml::from_str(&request.toml).map_err(|e| e.to_string())?;
            serde_json::to_value(compose(catalog, &loadout)?)
        }
        "optimize" => {
            let request: crate::OptimizeRequest =
                serde_json::from_str(input).map_err(|e| e.to_string())?;
            serde_json::to_value(crate::optimize(catalog, &request, |_| true)?)
        }
        _ => return Err(format!("未知配装方法: {method}")),
    };
    value.map_err(|e| e.to_string())
}

pub fn call(method: &str, input: &str) -> Result<Value, String> {
    if let Some(result) = job_dispatch(method, input) {
        return result;
    }
    dispatch(&catalog()?, method, input)
}

struct JointJob {
    progress: std::sync::Arc<crate::OptimizeProgress>,
    result: std::sync::Mutex<Option<Result<crate::OptimizeResult, String>>>,
    released: std::sync::atomic::AtomicBool,
}
static JOINT_JOBS: std::sync::Mutex<std::collections::BTreeMap<u64, std::sync::Arc<JointJob>>> =
    std::sync::Mutex::new(std::collections::BTreeMap::new());
static NEXT_JOB: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn start_joint(catalog: &Catalog, input: &str) -> Result<Value, String> {
    let request: crate::OptimizeRequest = serde_json::from_str(input).map_err(|e| e.to_string())?;
    let mut jobs = JOINT_JOBS.lock().map_err(|_| "联合搜索会话锁污染")?;
    if !jobs.is_empty() {
        return Err("已有联合搜索任务，请先取消或读取结果".into());
    }
    let job_id = NEXT_JOB.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let job = std::sync::Arc::new(JointJob {
        progress: std::sync::Arc::new(crate::OptimizeProgress::new()),
        result: std::sync::Mutex::new(None),
        released: std::sync::atomic::AtomicBool::new(false),
    });
    jobs.insert(job_id, std::sync::Arc::clone(&job));
    let catalog = catalog.clone();
    let spawn = std::thread::Builder::new()
        .name("gear-joint".into())
        .spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                crate::optimize_joint(&catalog, &request, &job.progress)
            }))
            .unwrap_or_else(|_| Err("联合搜索内部错误".into()));
            if let Ok(mut slot) = job.result.lock() {
                *slot = Some(result);
            }
            job.progress
                .done
                .store(true, std::sync::atomic::Ordering::Relaxed);
            if job.released.load(std::sync::atomic::Ordering::Relaxed)
                && let Ok(mut jobs) = JOINT_JOBS.lock()
            {
                jobs.remove(&job_id);
            }
        });
    if let Err(error) = spawn {
        jobs.remove(&job_id);
        return Err(error.to_string());
    }
    serde_json::to_value(crate::OptimizeJobId { job_id }).map_err(|e| e.to_string())
}

fn job_dispatch(method: &str, input: &str) -> Option<Result<Value, String>> {
    if method == "compute_backends" {
        return Some((|| {
            let _: Empty = serde_json::from_str(input).map_err(|e| e.to_string())?;
            serde_json::to_value(jpcg_compute::list_backends()).map_err(|e| e.to_string())
        })());
    }
    if !matches!(
        method,
        "optimize_joint_poll" | "optimize_joint_cancel" | "optimize_joint_release"
    ) {
        return None;
    }
    Some((|| {
        use std::sync::atomic::Ordering;
        let request: crate::OptimizeJobId =
            serde_json::from_str(input).map_err(|e| e.to_string())?;
        let mut jobs = JOINT_JOBS.lock().map_err(|_| "联合搜索会话锁污染")?;
        let job = jobs
            .get(&request.job_id)
            .cloned()
            .ok_or("联合搜索任务不存在或结果已领取")?;
        if method != "optimize_joint_poll" {
            job.progress.cancel();
            if method == "optimize_joint_release" {
                job.released.store(true, Ordering::Relaxed);
                if job
                    .result
                    .lock()
                    .map_err(|_| "联合搜索结果锁污染")?
                    .is_some()
                {
                    jobs.remove(&request.job_id);
                }
            }
            return Ok(serde_json::json!({}));
        }
        let mut slot = job.result.lock().map_err(|_| "联合搜索结果锁污染")?;
        let done = slot.is_some();
        let (result, error) = match slot.take() {
            Some(Ok(result)) => (Some(result), None),
            Some(Err(error)) => (None, Some(error)),
            None => (None, None),
        };
        let progress = crate::Progress {
            visited: job.progress.visited.load(Ordering::Relaxed),
            evaluated: job.progress.evaluated.load(Ordering::Relaxed),
            combinations: job.progress.total.load(Ordering::Relaxed),
        };
        if done {
            jobs.remove(&request.job_id);
        }
        serde_json::to_value(crate::OptimizeJobStatus {
            progress,
            done,
            result,
            error,
        })
        .map_err(|e| e.to_string())
    })())
}

#[cfg(test)]
mod joint_job_tests {
    use super::*;
    #[test]
    fn polling_returns_worker_error_and_releases_the_job() {
        let job_id = NEXT_JOB.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let job = std::sync::Arc::new(JointJob {
            progress: std::sync::Arc::new(crate::OptimizeProgress::new()),
            result: std::sync::Mutex::new(Some(Err("synthetic worker error".into()))),
            released: std::sync::atomic::AtomicBool::new(false),
        });
        JOINT_JOBS.lock().unwrap().insert(job_id, job);
        let input = serde_json::json!({"job_id":job_id}).to_string();
        let result = job_dispatch("optimize_joint_poll", &input)
            .unwrap()
            .unwrap();
        assert_eq!(result["done"], true);
        assert_eq!(result["error"], "synthetic worker error");
        assert!(
            job_dispatch("optimize_joint_poll", &input)
                .unwrap()
                .is_err()
        );
    }
}
