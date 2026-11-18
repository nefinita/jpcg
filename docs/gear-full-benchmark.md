# 完整联合穷举测速

在仓库根目录操作。ws 和 Mac 使用相同请求快照；CPU、GPU 分开跑，任一时刻只运行一个测速进程。GPU 只选择一张卡。开始前结束其他计算任务；通过 SSH 跑长任务建议在 tmux 会话中运行。

```sh
git switch fix/dongtian-nerf-20261112
git pull --ff-only
cargo build --release --locked -p jpcg_gear --bin gear-opt
./target/release/gear-opt backends
```

`scripts/benchmarks/gear-full-20261117.json` 是用户已确认的配置快照：莫问、live-50、加速至少928、竞技场防具四件套、全来源且只选升级档、50,000采样、seed=12345。沿用原连招、目标属性、附魔/五彩石选择和其他限制。脚本只覆盖 `compute`，不缩小搜索空间。

先完整跑 CPU（ws 自动使用 AVX-512；Mac 的 CPU 路径为标量）：

```sh
python3 scripts/bench-gear-joint.py --backend cpu
```

等上一条结束后再跑单卡 GPU。从 `backends` 输出选择对应的 R9700 / Apple M4 Max ID；不要假定重启后的设备编号不变。上次 ws 的一张 R9700 是 `wgpu:1`，示例：

```sh
python3 scripts/bench-gear-joint.py --backend wgpu:1
```

Mac 同样先列出 `backends`，将参数换成 Metal 的实际 ID。指定其他 JSON 请求或自定义产物路径可用 `--request path.json`、`--binary path/to/gear-opt`；完整参数见 `--help`。

默认输出目录是 `.scratch/gear-full/<时间-后端-pid>/`：

- `request.json`：该轮实际输入。
- `progress.log`：每10秒的 visited / evaluated / MC 进度，以及诊断日志。
- `result.json`：完整引擎输出，包括基线、最佳方案、Pareto、warnings。
- `summary.json`：端到端总秒数、实际评分数/秒、MC评分数、完整性与最佳击杀率。

没有 `--seconds` 或候选预算限制；联合搜索会跑完整空间，`max_combinations` 是兼容有界接口的字段，不限制此入口。正常完成必须同时满足 `complete=true`、`stop_reason="exhausted"`。Ctrl-C 会请求引擎取消并尽量保存部分结果，脚本以非零状态退出；取消结果不能作为全量最优结论。不要 kill -9，否则无法保存结果。

耗时从启动 CLI 计到退出，包含解析、剪枝、属性合成、缓存、MC、Pareto 与结果序列化，不含编译。`evaluated_per_second` 是实际评分候选的端到端吞吐；剪枝跳过数不能当作 MC 吞吐，`mc_evaluations` 不含基线且缓存命中不重复计数。比较 CPU/GPU 时使用同一份请求和数据，核对完整性、进度计数、最佳击杀率及结果，不能只比较百分比或短轮前缀。
