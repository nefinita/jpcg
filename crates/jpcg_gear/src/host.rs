//! 桌面宿主入口。前端只能传职业 ID、配装或 TOML 内容，不能指定文件路径。
//! 目录选择、默认配装与旧配装修复均在此完成，前端不复制配装规则。
use crate::{Catalog, Composition, Loadout, Slot, Stats};
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;

pub fn catalog_dir(profession: &str) -> Result<PathBuf, String> {
    jpcg_core::data_paths::gear_dir(profession)
}

pub fn catalog(profession: &str) -> Result<Catalog, String> {
    Catalog::load(&catalog_dir(profession)?)
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
struct CatalogRequest {
    profession: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrepareLoadout {
    profession: String,
    loadout: Option<Loadout>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Import {
    toml: String,
}

/// 生成或修复当前职业配装。只有这个显式准备入口会迁移旧默认状态；
/// compose 仍拒绝旧模式静默忽略五维，不把兼容迁移扩散到计算底层。
fn prepare_loadout(
    catalog: &Catalog,
    profession: &str,
    existing: Option<Loadout>,
) -> Result<Loadout, String> {
    if !catalog.schools.iter().any(|school| school.id == profession) {
        return Err(format!("职业 {profession} 不在其配装目录中"));
    }
    if let Some(mut loadout) = existing {
        if loadout.school != profession {
            return Err("配装职业与请求职业不一致".into());
        }
        if !catalog.version.sample && loadout.attribute_rules.is_none() {
            loadout.attribute_rules = Some(crate::attributes::PROFESSION_TOML.into());
        }
        return Ok(loadout);
    }
    let mut choices = Vec::with_capacity(Slot::ALL.len());
    for slot in Slot::ALL {
        let picked = catalog
            .templates
            .iter()
            .find(|template| template.school == profession && template.choice.slot == slot)
            .or_else(|| {
                catalog
                    .templates
                    .iter()
                    .find(|template| template.cross_school && template.choice.slot == slot)
            })
            .ok_or_else(|| format!("职业 {profession} 缺少槽位 {slot:?}"))?;
        let mut choice = picked.choice.clone();
        choice.school = (picked.school != profession).then(|| picked.school.clone());
        choices.push(choice);
    }
    let mut enchants = std::collections::BTreeMap::new();
    for slot in Slot::ALL {
        if let Some(enchant) = catalog.enchants.iter().find(|item| item.slot == slot) {
            enchants.insert(slot, enchant.id.clone());
        }
    }
    Ok(Loadout {
        school: profession.into(),
        revision: catalog.version.revision.clone(),
        value_set: catalog.version.value_set.clone(),
        attribute_rules: Some(crate::attributes::PROFESSION_TOML.into()),
        base: Stats {
            tizhi: 18,
            gengu: 18,
            shenfa: 18,
            yuanqi: 17,
            liliang: 17,
            ..Stats::default()
        },
        choices,
        enchants,
        wucai: catalog.wucai.first().map(|item| item.id.clone()),
    })
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
            let _: CatalogRequest = serde_json::from_str(input).map_err(|e| e.to_string())?;
            catalog.validate()?;
            serde_json::to_value(catalog)
        }
        "prepare_loadout" => {
            let request: PrepareLoadout = serde_json::from_str(input).map_err(|e| e.to_string())?;
            serde_json::to_value(prepare_loadout(
                catalog,
                &request.profession,
                request.loadout,
            )?)
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

fn catalog_for_call(method: &str, input: &str) -> Result<Catalog, String> {
    let profession = match method {
        "catalog" => {
            serde_json::from_str::<CatalogRequest>(input)
                .map_err(|e| e.to_string())?
                .profession
        }
        "prepare_loadout" => {
            serde_json::from_str::<PrepareLoadout>(input)
                .map_err(|e| e.to_string())?
                .profession
        }
        "import" => {
            let request: Import = serde_json::from_str(input).map_err(|e| e.to_string())?;
            toml::from_str::<Loadout>(&request.toml)
                .map_err(|e| e.to_string())?
                .school
        }
        "compose" | "export" => {
            serde_json::from_str::<Loadout>(input)
                .map_err(|e| e.to_string())?
                .school
        }
        "optimize" | "optimize_joint_start" => {
            serde_json::from_str::<crate::OptimizeRequest>(input)
                .map_err(|e| e.to_string())?
                .loadout
                .school
        }
        _ => return Err(format!("未知配装方法: {method}")),
    };
    catalog(&profession)
}

pub fn call(method: &str, input: &str) -> Result<Value, String> {
    if let Some(result) = job_dispatch(method, input) {
        return result;
    }
    dispatch(&catalog_for_call(method, input)?, method, input)
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
mod prepare_loadout_tests {
    use super::*;

    fn catalog_at(relative: &str) -> Catalog {
        Catalog::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)).unwrap()
    }

    fn loadout_at(relative: &str) -> Loadout {
        toml::from_str(
            &std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative),
            )
            .unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn creates_complete_default_from_the_requested_profession_catalog() {
        let catalog = catalog_at("../../data/mowen/gear");
        let loadout = prepare_loadout(&catalog, "mowen", None).unwrap();
        assert_eq!(loadout.school, "mowen");
        assert_eq!(loadout.revision, catalog.version.revision);
        assert_eq!(loadout.value_set, catalog.version.value_set);
        assert_eq!(loadout.choices.len(), Slot::ALL.len());
        assert_eq!(
            loadout.attribute_rules.as_deref(),
            Some(crate::attributes::PROFESSION_TOML)
        );
        assert_eq!(
            (loadout.base.tizhi, loadout.base.gengu, loadout.base.yuanqi),
            (18, 18, 17)
        );
        assert_eq!(loadout.enchants.len(), Slot::ALL.len());
        assert!(loadout.wucai.is_some());
        // 完整合成需要进程数据根，由集成/动态桥接测试在隔离环境中覆盖。
    }

    #[test]
    fn migrates_only_real_catalog_legacy_loadouts() {
        let real = catalog_at("../../data/mowen/gear");
        let mut legacy = loadout_at("../../data/mowen/gear/sample-loadout.toml");
        legacy.attribute_rules = None;
        let migrated = prepare_loadout(&real, "mowen", Some(legacy.clone())).unwrap();
        assert_eq!(
            migrated.attribute_rules.as_deref(),
            Some(crate::attributes::PROFESSION_TOML)
        );
        assert_eq!(migrated.choices, legacy.choices);

        let sample = catalog_at("tests/fixtures/synthetic/gear");
        let untouched = prepare_loadout(&sample, "mowen", Some(legacy)).unwrap();
        assert_eq!(untouched.attribute_rules, None);
    }

    #[test]
    fn rejects_a_loadout_from_another_profession() {
        let catalog = catalog_at("../../data/mowen/gear");
        let mut other = loadout_at("../../data/mowen/gear/sample-loadout.toml");
        other.school = "badao".into();
        let error = prepare_loadout(&catalog, "mowen", Some(other)).unwrap_err();
        assert!(error.contains("职业不一致"), "{error}");
        assert!(prepare_loadout(&catalog, "badao", None).is_err());
    }
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
