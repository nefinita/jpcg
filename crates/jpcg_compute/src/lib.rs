//! 计算后端：列举本机可用平台，并在其上批量跑 50k MC 击杀率。
//!
//! - `cpu`：复用 `jpcg_combo::kill_mc`（f32 + 计数哈希；x86 自动 AVX-512）。
//! - `wgpu:<n>`：wgpu 适配器（Metal / Vulkan / DX12），同一套 WGSL kernel。
//!
//! 两边的随机序列一致（同一 seed、同一计数器哈希），CPU 与 wgpu 的击杀计数应逐候选相同
//! （f32 运算顺序一致；由 `run_batch_*` 对拍测试保证）。

use jpcg_api::ComputeBackendDTO;
use jpcg_combo::kill_mc::{self, McEvent};

/// 每候选最多伤害事件数（GPU 缓冲区步长）。
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
        if std::is_x86_feature_detected!("avx512f") {
            return "AVX-512";
        }
        if std::is_x86_feature_detected!("avx2") {
            return "AVX2";
        }
        return "SSE";
    }
    #[cfg(target_arch = "aarch64")]
    {
        return "NEON";
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

/// CPU 后端：逐候选跑 50k MC，返回击杀数（总样本中的命中数）。
pub fn run_cpu(candidates: &[CandidateMc], samples: u32, seed: u64) -> Vec<u64> {
    candidates
        .iter()
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
    Ok(run_cpu(candidates, samples, seed))
}

const WGSL: &str = r#"
struct Dice { g: u32, h: u32, rate: f32, lost: f32 };
struct Params { samples: u32, seed: u32, events: u32, max_events: u32 };
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> dice: array<Dice>;
@group(0) @binding(2) var<storage, read> counts: array<u32>;
@group(0) @binding(3) var<storage, read> hp0s: array<f32>;
@group(0) @binding(4) var<storage, read_write> out: array<atomic<u32>>;

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
  if (lid.x == 0u) { atomicStore(&out[cand], 0u); }
  workgroupBarrier();
  atomicAdd(&out[cand], kills);
  workgroupBarrier();
}
"#;

/// 常驻的 wgpu 计算上下文（适配器/设备/管线只建一次，批量复用）。
pub struct WgpuCompute {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
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
        })
    }

    /// 批量跑 MC；返回每候选击杀数（总样本中的命中数）。
    pub fn run(
        &self,
        candidates: &[CandidateMc],
        samples: u32,
        seed: u64,
    ) -> Result<Vec<u64>, String> {
        use wgpu::util::DeviceExt;
        let n = candidates.len();
        if n == 0 {
            return Ok(Vec::new());
        }
        #[repr(C)]
        #[derive(Clone, Copy)]
        struct Dice {
            g: u32,
            h: u32,
            rate: f32,
            lost: f32,
        }
        let mut dice_bytes: Vec<u8> = Vec::with_capacity(n * MAX_EVENTS * 16);
        let mut counts: Vec<u32> = Vec::with_capacity(n);
        let mut hp0s: Vec<f32> = Vec::with_capacity(n);
        for c in candidates {
            if c.events.len() > MAX_EVENTS {
                return Err(format!(
                    "候选事件数 {} 超过上限 {MAX_EVENTS}",
                    c.events.len()
                ));
            }
            let mut d = vec![
                Dice {
                    g: 0,
                    h: 0,
                    rate: 2.0,
                    lost: 0.0
                };
                MAX_EVENTS
            ];
            for (i, e) in c.events.iter().enumerate() {
                d[i] = Dice {
                    g: e.g,
                    h: e.h,
                    rate: e.rate,
                    lost: e.lost,
                };
            }
            for x in &d {
                dice_bytes.extend_from_slice(&x.g.to_le_bytes());
                dice_bytes.extend_from_slice(&x.h.to_le_bytes());
                dice_bytes.extend_from_slice(&x.rate.to_le_bytes());
                dice_bytes.extend_from_slice(&x.lost.to_le_bytes());
            }
            counts.push(c.events.len() as u32);
            hp0s.push(c.hp0 as f32);
        }
        let seed32 = (seed as u32) ^ ((seed >> 32) as u32);
        let params: [u32; 4] = [samples, seed32, 0, MAX_EVENTS as u32];
        let params_bytes: Vec<u8> = params.iter().flat_map(|v| v.to_le_bytes()).collect();
        let dice_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("dice"),
                contents: &dice_bytes,
                usage: wgpu::BufferUsages::STORAGE,
            });
        let counts_bytes = bytemuck_u32(&counts);
        let hp0_bytes = bytemuck_f32(&hp0s);
        let counts_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("counts"),
                contents: &counts_bytes,
                usage: wgpu::BufferUsages::STORAGE,
            });
        let hp0_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("hp0s"),
                contents: &hp0_bytes,
                usage: wgpu::BufferUsages::STORAGE,
            });
        let params_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("params"),
                contents: &params_bytes,
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let out_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("out"),
            size: (n as u64) * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let stage = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("stage"),
            size: (n as u64) * 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: dice_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: counts_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: hp0_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: out_buf.as_entire_binding(),
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
            // 单次 dispatch 的 workgroup 上限 65535，按批切片。
            let mut start: u32 = 0;
            while (start as usize) < n {
                let len = ((n - start as usize).min(65_535)) as u32;
                pass.dispatch_workgroups(len, 1, 1);
                start += len;
            }
        }
        encoder.copy_buffer_to_buffer(&out_buf, 0, &stage, 0, (n as u64) * 4);
        self.queue.submit(Some(encoder.finish()));
        let slice = stage.slice(..);
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
        let out: Vec<u64> = data
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]) as u64)
            .collect();
        drop(data);
        stage.unmap();
        Ok(out)
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
        let cands = sample();
        let cpu = run_cpu(&cands, 4000, 12345);
        // 找一个非 CPU 适配器（无 GPU 时自动跳过）
        let adapters = wgpu_adapters();
        let idx = adapters
            .iter()
            .position(|a| a.get_info().device_type != wgpu::DeviceType::Cpu);
        let Some(idx) = idx else { return };
        let gpu = WgpuCompute::new(idx).expect("wgpu context");
        let got = gpu.run(&cands, 4000, 12345).expect("wgpu run");
        assert_eq!(cpu, got);
    }
}
