//! 计算后端：列举本机可用平台，并在其上批量跑 50k MC 击杀率。
//!
//! - `cpu`：复用 `jpcg_combo::kill_mc`（f32 + 计数哈希；x86 自动 AVX-512）。
//! - `wgpu:<n>`：wgpu 适配器（Metal / Vulkan / DX12），同一套 WGSL kernel。
//!
//! 两边的随机序列一致（同一 seed、同一计数器哈希），CPU 与 wgpu 的击杀计数应逐候选相同
//! （f32 运算顺序一致；由 `run_batch_*` 对拍测试保证）。

use jpcg_api::ComputeBackendDTO;
use jpcg_combo::kill_mc::{self, McEvent};

/// 历史推荐事件步长；run 使用实际事件数，不将它作为上限。
pub const MAX_EVENTS: usize = 64;
/// 固定 MC 样本数（与计算器显示一致）。
pub const SAMPLES: u32 = 50_000;

fn cpu_model() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(text) = std::fs::read_to_string("/proc/cpuinfo") {
            if let Some(line) = text.lines().find(|l| l.starts_with("model name")) {
                if let Some(v) = line.split(':').nth(1) {
                    return v.trim().to_string();
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = std::process::Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
        {
            if let Ok(text) = String::from_utf8(out.stdout) {
                let t = text.trim();
                if !t.is_empty() {
                    return t.to_string();
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(v) = std::env::var("PROCESSOR_IDENTIFIER") {
            return v;
        }
    }
    "CPU".to_string()
}

fn cpu_isa() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    {
        if kill_mc::avx512_available() {
            return "AVX-512";
        }
        return "标量 (x86-64)";
    }
    #[cfg(target_arch = "aarch64")]
    {
        return "标量 (aarch64)";
    }
    #[allow(unreachable_code)]
    "标量"
}

fn wgpu_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle())
}

fn wgpu_adapters() -> Vec<wgpu::Adapter> {
    pollster::block_on(wgpu_instance().enumerate_adapters(wgpu::Backends::all()))
}

/// 列举本机可用的计算后端：CPU 在前，其后是 wgpu 适配器。
pub fn list_backends() -> Vec<ComputeBackendDTO> {
    let threads = std::thread::available_parallelism()
        .map(|v| v.get())
        .unwrap_or(1);
    let mut out = vec![ComputeBackendDTO {
        id: "cpu".into(),
        kind: "cpu".into(),
        label: format!("{} · {} ({} 线程)", cpu_isa(), cpu_model(), threads),
        detail: "f32 + 计数哈希，x86 自动 AVX-512；无 GPU 依赖".into(),
        available: true,
    }];
    for (i, adapter) in wgpu_adapters().into_iter().enumerate() {
        let info = adapter.get_info();
        let kind = format!("{:?}", info.device_type);
        out.push(ComputeBackendDTO {
            id: format!("wgpu:{i}"),
            kind: "wgpu".into(),
            label: info.name.clone(),
            detail: format!(
                "wgpu · {:?} · {} · vendor {}",
                info.backend, kind, info.vendor
            ),
            available: true,
        });
    }
    out
}

/// 每候选（事件 + 初始血量）的 MC 输入。
pub struct CandidateMc {
    pub events: Vec<McEvent>,
    pub hp0: f64,
}

/// CPU 后端：并行逐候选跑 50k MC，返回击杀数（总样本中的命中数）。
pub fn run_cpu(candidates: &[CandidateMc], samples: u32, seed: u64) -> Vec<u64> {
    use rayon::prelude::*;
    candidates
        .par_iter()
        .map(|c| {
            kill_mc::kill_counts(&c.events, c.hp0, samples, seed)
                .iter()
                .sum::<u64>()
        })
        .collect()
}

/// 批量 MC：按后端 id 选择 CPU 或 wgpu 适配器。
pub fn run_batch(
    candidates: &[CandidateMc],
    samples: u32,
    seed: u64,
    backend: &str,
) -> Result<Vec<u64>, String> {
    if let Some(idx) = backend.strip_prefix("wgpu:") {
        let index: usize = idx.parse().map_err(|_| format!("非法后端 id: {backend}"))?;
        let gpu = WgpuCompute::new(index)?;
        return gpu.run(candidates, samples, seed);
    }
    if backend != "cpu" {
        return Err(format!("非法后端 id: {backend}"));
    }
    Ok(run_cpu(candidates, samples, seed))
}

const WGSL: &str = r#"
struct Dice { g: u32, h: u32, rate: f32, lost: f32 };
struct Params { samples: u32, seed: u32, events: u32, max_events: u32 };
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> dice: array<Dice>;
@group(0) @binding(2) var<storage, read> counts: array<u32>;
@group(0) @binding(3) var<storage, read> hp0s: array<f32>;
@group(0) @binding(4) var<storage, read_write> out: array<u32>;
var<workgroup> partial: array<u32, 256>;

fn mix32(x0: u32) -> u32 {
  var x = x0;
  x = x ^ (x >> 16u); x = x * 0x7feb352du;
  x = x ^ (x >> 15u); x = x * 0x846ca68bu;
  x = x ^ (x >> 16u);
  return x;
}

@compute @workgroup_size(256)
fn main(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>) {
  let cand = wid.x;
  let n = counts[cand];
  let hp0 = hp0s[cand];
  let base_dice = cand * params.max_events;
  var kills = 0u;
  var s = lid.x;
  loop {
    if (s >= params.samples) { break; }
    var hp = hp0;
    let base = params.seed ^ (s * 0x85ebca6bu);
    for (var e = 0u; e < n; e = e + 1u) {
      let r = mix32(base ^ (e * 0xc2b2ae35u));
      let u = f32(r >> 8u) * (1.0 / 16777216.0);
      let d = dice[base_dice + e];
      let dmg = select(f32(d.g), f32(d.h), u < d.rate);
      hp = hp - dmg;
      if (hp < 0.0) { hp = 0.0; }
      if (d.lost > 0.0 && hp > 0.0) {
        hp = hp - (hp0 - hp) * d.lost;
        if (hp < 0.0) { hp = 0.0; }
      }
      if (hp <= 0.0) { kills = kills + 1u; break; }
    }
    s = s + 256u;
  }
  partial[lid.x] = kills;
  workgroupBarrier();
  var stride = 128u;
  loop {
    if (lid.x < stride) { partial[lid.x] = partial[lid.x] + partial[lid.x + stride]; }
    workgroupBarrier();
    if (stride == 1u) { break; }
    stride = stride / 2u;
  }
  if (lid.x == 0u) { out[cand] = partial[0]; }
}
"#;

/// 常驻的 wgpu 计算上下文（适配器/设备/管线只建一次，批量复用）。
pub struct WgpuCompute {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
    scratch: std::sync::Mutex<Option<GpuScratch>>,
}

struct GpuScratch {
    dice: wgpu::Buffer,
    counts: wgpu::Buffer,
    hp: wgpu::Buffer,
    params: wgpu::Buffer,
    output: wgpu::Buffer,
    staging: wgpu::Buffer,
    dice_capacity: u64,
    candidate_capacity: u64,
}

impl WgpuCompute {
    pub fn new(adapter_index: usize) -> Result<Self, String> {
        let instance = wgpu_instance();
        let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
        let adapter = adapters
            .into_iter()
            .nth(adapter_index)
            .ok_or_else(|| format!("wgpu 适配器 {adapter_index} 不存在"))?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .map_err(|e| e.to_string())?;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("jpcg-mc"),
            source: wgpu::ShaderSource::Wgsl(WGSL.into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("jpcg-mc"),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let bgl = pipeline.get_bind_group_layout(0);
        Ok(Self {
            device,
            queue,
            pipeline,
            bgl,
            scratch: std::sync::Mutex::new(None),
        })
    }

    /// 按设备缓冲区/dispatch 限制切片，每片独立绑定，候选索引不会重复归零覆盖。
    pub fn run(
        &self,
        candidates: &[CandidateMc],
        samples: u32,
        seed: u64,
    ) -> Result<Vec<u64>, String> {
        let mut result = Vec::with_capacity(candidates.len());
        let limits = self.device.limits();
        let binding_limit = limits
            .max_storage_buffer_binding_size
            .min(limits.max_buffer_size);
        let mut offset = 0;
        while offset < candidates.len() {
            let end = (offset + limits.max_compute_workgroups_per_dimension as usize)
                .min(candidates.len());
            let events = candidates[offset..end]
                .iter()
                .map(|c| c.events.len())
                .max()
                .unwrap_or(0)
                .max(1);
            let stride = u64::try_from(events)
                .map_err(|_| "事件数溢出")?
                .checked_mul(16)
                .ok_or("事件缓冲区大小溢出")?;
            let capacity = (binding_limit / stride).min(binding_limit / 4) as usize;
            if capacity == 0 {
                return Err("单个候选事件数超过GPU缓冲区限制".into());
            }
            let len = (end - offset).min(capacity);
            result.extend(self.run_slice(
                &candidates[offset..offset + len],
                samples,
                seed,
                events,
            )?);
            offset += len;
        }
        Ok(result)
    }

    fn run_slice(
        &self,
        candidates: &[CandidateMc],
        samples: u32,
        seed: u64,
        max_events: usize,
    ) -> Result<Vec<u64>, String> {
        let n = candidates.len();
        let event_stride = u32::try_from(max_events).map_err(|_| "事件数超过u32")?;
        let mut dice = Vec::with_capacity(n * max_events * 16);
        let mut counts = Vec::with_capacity(n);
        let mut hp = Vec::with_capacity(n);
        for c in candidates {
            counts.push(c.events.len() as u32);
            hp.push(c.hp0 as f32);
            for e in &c.events {
                dice.extend_from_slice(&e.g.to_le_bytes());
                dice.extend_from_slice(&e.h.to_le_bytes());
                dice.extend_from_slice(&e.rate.to_le_bytes());
                dice.extend_from_slice(&e.lost.to_le_bytes());
            }
            dice.resize(dice.len() + (max_events - c.events.len()) * 16, 0);
        }
        let mut scratch = self.scratch.lock().map_err(|_| "GPU缓冲区锁污染")?;
        let dice_size = dice.len() as u64;
        let candidate_size = (n * 4) as u64;
        if scratch
            .as_ref()
            .is_none_or(|s| s.dice_capacity < dice_size || s.candidate_capacity < candidate_size)
        {
            let make = |label: &'static str, size: u64, usage| {
                self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(label),
                    size,
                    usage,
                    mapped_at_creation: false,
                })
            };
            *scratch = Some(GpuScratch {
                dice: make(
                    "dice",
                    dice_size,
                    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                ),
                counts: make(
                    "counts",
                    candidate_size,
                    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                ),
                hp: make(
                    "hp",
                    candidate_size,
                    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                ),
                params: make(
                    "params",
                    16,
                    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                ),
                output: make(
                    "out",
                    candidate_size,
                    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                ),
                staging: make(
                    "staging",
                    candidate_size,
                    wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                ),
                dice_capacity: dice_size,
                candidate_capacity: candidate_size,
            });
        }
        let buffers = scratch.as_ref().ok_or("GPU缓冲区未初始化")?;
        self.queue.write_buffer(&buffers.dice, 0, &dice);
        self.queue
            .write_buffer(&buffers.counts, 0, &bytemuck_u32(&counts));
        self.queue.write_buffer(&buffers.hp, 0, &bytemuck_f32(&hp));
        let params = [
            samples,
            (seed as u32) ^ ((seed >> 32) as u32),
            n as u32,
            event_stride,
        ];
        self.queue
            .write_buffer(&buffers.params, 0, &bytemuck_u32(&params));
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.dice.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.counts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers.hp.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: buffers.output.as_entire_binding(),
                },
            ],
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(n as u32, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&buffers.output, 0, &buffers.staging, 0, candidate_size);
        self.queue.submit(Some(encoder.finish()));
        let slice = buffers.staging.slice(0..candidate_size);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| format!("{e:?}"))?;
        rx.recv()
            .map_err(|e| e.to_string())?
            .map_err(|e| format!("{e:?}"))?;
        let data = slice.get_mapped_range().map_err(|e| format!("{e:?}"))?;
        let result = data
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]) as u64)
            .collect();
        drop(data);
        buffers.staging.unmap();
        Ok(result)
    }
}

fn bytemuck_u32(v: &[u32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}
fn bytemuck_f32(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<CandidateMc> {
        (0..8)
            .map(|i| CandidateMc {
                events: (0..6)
                    .map(|e| McEvent {
                        g: 800 + i * 100 + e * 50,
                        h: 2200 + i * 120 + e * 80,
                        rate: 0.25 + (e as f32) * 0.05,
                        lost: if (i + e) % 4 == 0 { 0.02 } else { 0.0 },
                    })
                    .collect(),
                hp0: 9000.0,
            })
            .collect()
    }

    #[test]
    fn cpu_runs() {
        let out = run_cpu(&sample(), 4000, 12345);
        assert_eq!(out.len(), 8);
        assert!(out.iter().all(|&k| k <= 4000));
    }

    #[test]
    fn cpu_matches_wgpu_when_available() {
        if std::env::var_os("JPCG_TEST_CPU_ONLY").is_some() {
            return;
        }
        let selected = std::env::var("JPCG_TEST_WGPU_INDEX")
            .ok()
            .and_then(|s| s.parse::<usize>().ok());
        let cands = sample();
        let samples = 50_000;
        let seed = 0x1234_5678_9abc_def0;
        let cpu = run_cpu(&cands, samples, seed);
        assert!(cpu.iter().any(|&count| count > 0 && count < samples as u64));
        // 找一个非 CPU 适配器（无 GPU 时自动跳过）
        let adapters = wgpu_adapters();
        let mut tested = 0;
        for (idx, adapter) in adapters.iter().enumerate() {
            let info = adapter.get_info();
            if selected.is_some_and(|selected| selected != idx) {
                continue;
            }
            if info.device_type == wgpu::DeviceType::Cpu {
                continue;
            }
            let gpu = WgpuCompute::new(idx).expect("wgpu context");
            let got = gpu.run(&cands, samples, seed).expect("wgpu run");
            assert_eq!(cpu, got, "wgpu:{idx}: {} ({:?})", info.name, info.backend);
            eprintln!(
                "CPU/wgpu:{idx} parity passed: {} ({:?})",
                info.name, info.backend
            );
            tested += 1;
        }
        eprintln!("GPU adapters tested: {tested}");
    }
}

#[cfg(test)]
mod batch_boundary_tests {
    use super::*;
    fn gpu() -> Option<WgpuCompute> {
        if std::env::var_os("JPCG_TEST_CPU_ONLY").is_some() {
            return None;
        }
        if let Ok(index) = std::env::var("JPCG_TEST_WGPU_INDEX") {
            return Some(WgpuCompute::new(index.parse().expect("GPU index")).expect("GPU context"));
        }
        let adapters = wgpu_adapters();
        let index = adapters
            .iter()
            .position(|a| a.get_info().device_type != wgpu::DeviceType::Cpu)?;
        Some(WgpuCompute::new(index).expect("GPU context"))
    }
    #[test]
    fn gpu_handles_dispatch_tail_and_more_than_64_events() {
        let Some(gpu) = gpu() else {
            return;
        };
        let candidates: Vec<_> = (0..65_536)
            .map(|i| CandidateMc {
                events: vec![McEvent {
                    g: 100,
                    h: 100,
                    rate: 1.0,
                    lost: 0.0,
                }],
                hp0: if i == 65_535 { 50.0 } else { 200.0 },
            })
            .collect();
        let got = gpu.run(&candidates, 1, 123).unwrap();
        assert_eq!(got.last(), Some(&1));
        assert!(got[..65_535].iter().all(|v| *v == 0));
        let long = vec![CandidateMc {
            events: vec![
                McEvent {
                    g: 1,
                    h: 1,
                    rate: 0.5,
                    lost: 0.0
                };
                65
            ],
            hp0: 65.0,
        }];
        assert_eq!(gpu.run(&long, 17, 456).unwrap(), run_cpu(&long, 17, 456));
        // 再次缩小输入，验证复用缓冲区不会读到上一批尾部。
        assert_eq!(gpu.run(&candidates[..1], 1, 123).unwrap(), vec![0]);
    }
}
