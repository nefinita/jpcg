# 配装模板与有界优化（#90）

独立 Rust crate，提供 12 槽位模板校验、属性合成、约束搜索、JSON API 和 `gear-opt` CLI。现有 `jpcg_api`、core/combo 引擎、FFI、应用 UI 和工作流均不修改。供 #91 集成；这不是 #90 全部验收完成的声明。

`data/mowen/gear/` 当前提供莫问 50 级 `live-50-20261106` 目录，`sample=false`，来源与各文件 `provenance` 一致；早期 `synthetic-v1` 才是全人为构造样例。`sample-loadout.toml` / `sample-request.toml` 的“sample”指调用示例，不代表目录数值为合成数据。莫问主属性与技能计算使用仓库现有 `data/mowen/shuxing.toml`；真实人工配装基准、其他职业数据及未标明的规则仍待维护者验证。

## 运行

仓库根目录执行，无需 GUI：

```sh
cargo run -p jpcg_gear --bin gear-opt -- validate
cargo run -p jpcg_gear --bin gear-opt -- compose --loadout data/mowen/gear/sample-loadout.toml
JPCG_DATA_DIR="$PWD/data" cargo run -p jpcg_gear --bin gear-opt -- optimize --request data/mowen/gear/sample-request.toml
printf '%s\n' '{"method":"validate"}' | cargo run -p jpcg_gear --bin gear-opt -- api
cargo test -p jpcg_gear --locked
```

`--catalog <目录>` 在子命令之前指定，默认 `data/mowen/gear`。目录必须包含 `templates.toml`、`sets.toml`、`schools.toml`、`overrides.toml`。校验/合成只读取该目录；优化的技能和值集路径沿用 core 的 `JPCG_DATA_DIR` / 安装目录规则。优化要求明确的可用数据值集，禁止静默回退 builtin。

compose/optimize 成功输出 JSON；api 输出 `{"status":"ok","result":...}` 或 `{"status":"error","message":...}`。输入错误退出码 1；命令行 optimize 无可行结果也退出 1，仍输出结果。完整遍历且无解为 `complete=true,best=null`；预算耗尽且尚无解为 `complete=false,best=null`，二者含义不同。库 API 不可行时仍返回 `Ok(OptimizeResult)`，由调用方检查 `best`。

## 文件契约（schema=1）

四文件的 `[version]` 必须完全一致，包含 `schema/season/revision/value_set/sample/provenance`；未知字段、无符号点数溢出、重复键、非法来源/档位/套装、缺少部位、饰品属性套不完整都报错。数据通道现有 manifest-gen 已递归收集所有文件，无需新增部署逻辑。

- `schools`：`id` 是现有心法 key；`zhushuxing=liliang/shenfa/gengu/yuanqi`，`gongji_type/pofang_type=waigong/neigong`。主属性点由模板的 `zhushuxing` 携带，profile 指明其类型。攻击和破防类型必须与主属性一致。没有按门派臆造点数或御劲攻击公式。
- `templates`：`school`、`choice={slot,source,tier,variant}`、`green`、`stats`、可选 `set/zhuangfen`。唯一键是门派+完整 choice；非饰品 variant=0，饰品 variant=1/2。同一键的基础/升级档绿字固定。装分只展示，完全不参与搜索。
- `sets`：每个来源最多一套防具与一套饰品，避免同源 2+2。每套按 tier 定义 `bonus2`、可选 `bonus3`、`cooldown4=[{skill_id,seconds}]`。防具2件属性与4件CD必须显式给出且非零，不能有 bonus3；饰品 bonus2 仅正化劲、bonus3 仅正御劲，没有 cooldown4。宝物堆不能定义套装。
- `overrides`：通过 `school+choice` 指向已有唯一模板；可替换 green 和/或**整份** stats，不做增量加法，不创建新模板；应用前后均校验。空覆盖与重复覆盖报错。

| 字段 | 枚举 / 含义 |
| --- | --- |
| source | `mingjian` 名剑币、`weiwang` 威望、`baowu` 宝物堆 |
| tier | `base` 基础、`upgraded` 升级；宝物堆仅基础 |
| 非饰品 slot | `tou/yi/yao/shou/ku/xie/wuqi/anqi` |
| 饰品 slot | `xianglian/yaozhui/jie1/jie2`，仅威望，分别自由选属性套 |
| green | `pofang/huixin/jiasu/yujin`；名剑无御劲，对应属性点必须为正 |
| stats | `zhushuxing/gongji/qixue/huixin/huixiao/pofang/jiasu/yujin/huajin/wuqi_shanghai`，省略为0 |

已提供某饰品部位/档位时必须同时有 1/2 两套模板；允许数据只覆盖部分来源和基础档，方便正式数据分批补齐。每个已声明门派仍须覆盖全部 12 槽位。

## 合成与下游 API

```rust,ignore
let catalog = jpcg_gear::Catalog::load(gear_directory)?;
let composed = catalog.compose(&loadout)?;
let player_dto = jpcg_gear::player(&composed);
let response = jpcg_gear::call(&catalog, json_request);
```

`Loadout` 的 `school/revision/value_set/base/choices` 全部随 TOML 保存；`base` 必须是**裸装基础值**，合成时不读取计算配置已有的 player 点数，避免装备重复相加。导出用 `toml::to_string_pretty(&composed.loadout)`，重新解析即可复现。`Composed` 返回 profile、完整 stats、各套装阈值及档位、CD 元数据与 warnings（时间轴目标应用 CD）。player 转换传递伤害属性与加速等级；气血/御劲/化劲保留在 Composed 中供驾驶台展示或约束。启用 #98 历史规则时还返回派生 panel，并传递已转换的心法额外攻击，详见下方说明。

防具仅头/衣/腰/手/鞋计数，裤子/武器/暗器不能声明防具套装。2 件奖励一次、4 件另附 CD，两者同时生效；跨来源 2+2 可叠；饰品 2/3 件奖励同时生效。

**待维护者确认的混档约定**：每个阈值独立判断升级件数，升级件数达到该阈值用升级奖励，否则用基础奖励。升级奖励替代该阈值基础奖励；例如 2 升级+2 基础防具给升级 2 件奖励和基础 4 件 CD。这个约定是实现策略，不能当作已经确认的游戏规则。

JSON 输入为以下之一，拒绝多余字段：

```json
{"method":"validate"}
{"method":"compose","loadout":{"school":"mowen","revision":"live-50-20261106","value_set":"live-50","choices":[]}}
{"method":"optimize","request":{}}
```

上面 compose/optimize 是结构示意，choices 必须完整，request 形状见 `model::OptimizeRequest` 和样例 TOML。宿主直接传 Catalog，JSON 请求不能选择任意文件路径。不新增 DLL、Tauri command 或共享 DTO；#91 可先用 Rust 直接调用，动态桥接由后续集成补充。

## 优化语义和边界

`OptimizeRequest` 包含 loadout 基线、现有 CalculateRequest、objective、constraints 与 `max_combinations`（1..1000000，必填）。心法、主属性、系数、值集须与已加载技能和模板匹配。来源、禁用/锁定 choice、按槽位必选/禁用来源、是否允许升级、所有合成属性 min/max、是否允许跨来源防具 2+2 均为硬约束。禁止 2+2 是禁止该选择组合，不是隐藏已选套装的加成；禁止整个槽位会导致无法完成12槽位，返回无候选错误。

- `expected`：明确的非零正技能权重，评分是指定技能 q 的加权和。调用现有 `JpcgConfig::q_cal`，一次加载技能并缓存相同属性/CD评估；重名技能报错，不把技能池隐式当作连招。配置中 player 被合成属性替换，其余沿用当前计算链。
- `kill_probability`：完整 steps 快照、samples（1..50000）、固定 seed，直接调用现有 combo host；不修改时序/加速/采样算法。出现 4 件 CD 时默认报错；只有显式 `allow_unapplied_cd=true` 才允许做“仅属性、固定技能序列”的击杀率计算，并返回 warning。
- 时间轴目标将合成加速和套装 CD 应用于时序；纯技能权重目标不重排时序。旧连招目标仍需显式确认未应用 CD。套装规则以目录中的 `skill_id/seconds` 为准。
- 固定槽位与候选排序的**有界穷举**，非 DP/全规模高效求解器。每个完整组合都算预算，违反属性约束的组合不调用目标函数。预算内可行基线预先加入，保证有限搜索不会劣于一个符合新约束的基线。
- `complete=true` 表示全部候选组合遍历完，才能称为全局最优；`complete=false` 明确表示预算耗尽或取消。Pareto 仅针对已评估方案，以目标值和气血/御劲/化劲三个独立数值比较，不编造统一生存权重；击杀目标时第一维是击杀率。
- `optimize_with` 可注入评分器做精确小空间对照，`optimize` 绑定现有引擎。两者的进度回调收到 visited/evaluated/combinations；返回 false 在组合间取消，无法中断一次同步 combo 采样。

`optimize_joint` 是独立的全量联合搜索入口，不使用上述预算。它优先遍历套装防具，在完整面板合成前按套装剩余可达件数、跨来源 2+2 和加速上下界剪枝；装备、镶嵌、裸装、附魔、五彩石及混档套装奖励均计入加速范围。其余属性约束仍由完整合成器检查。可行候选流式分批交给 CPU（x86 自动 AVX-512）或 wgpu 做精确 MC，不预扫描或保存全空间。`visited` 包含整支剪掉的叶子数，`evaluated` 记录实际评分数；剪枝不会消耗有界搜索的预算或改变其行为。

候选准备的合成样例基准可运行 `cargo test -p jpcg_gear --release --lib joint_prefilter_benchmark -- --ignored --nocapture`。它对拍旧的逐叶合成筛选和新剪枝，不包含时间轴/MC，不能代替实际配装的端到端性能验收。全规模搜索仍可能有大量可行候选，GPU/AVX-512 不保证任意空间在 1 秒内完成。

使用已确认的真实配置完整跑 CPU / 单卡 GPU，见[全量测速方法](../../docs/gear-full-benchmark.md)。脚本逐轮保存输入、进度、结果与端到端耗时，不含编译时间。

## 验证

集成测试覆盖模板唯一性、来源/升级限制、门派映射、套装2+2/2+4/饰品2+3、裤子排除、混档、饰品混搭、显式御劲高攻击覆盖、版本不匹配、未知字段、属性溢出、硬约束、无解/预算/取消、Pareto、TOML往返、CLI JSON及退出码，并与 core/combo 现有入口做数值一致性与固定种子回归。运行期数据测试用独立子进程设置目录，CI 无需全局修改环境。

### 合成样例 release 测量（2026-10-30）

Linux x86_64，rustc 1.99.1，`cargo build -p jpcg_gear --release --locked` 后执行 `python3 scripts/bench-gear.py`，每目标5次独立进程，计入CLI启动/解析。两目标均完整遍历16个组合、同输入结果确定；击杀目标为1步人为技能、50k采样、seed=42，显式忽略未接入的套装CD。

| 目标 | 基线 | 最佳 | CLI耗时中位数 |
| --- | ---: | ---: | ---: |
| 徵期望伤害 | 40506 | 40544 | 2.430 ms |
| 合成单步击杀率 | 0.09724 | 0.09724 | 4.948 ms |

该表只证明小样例可运行和推荐不劣于合法基线，不能替代真实装备场景、长连招或约1亿组合的性能验收。


## #91 桌面接入（后续叠放分支）

- `host::catalog(profession)` 按 core 的数据根优先级读取 `data/{职业ID}/gear/` 的六个核心文件（templates / sets / schools / overrides / enchants / wucai）并校验；旧 `data/gear/` 仅作读取回退。支持 `JPCG_DATA_DIR` 指向根或旧 `shuxing/`。前端只传职业 ID，不能传任意文件路径；损坏数据报错，不伪装成空目录或静默回退样例。
- Tauri 命令 `gear_catalog / prepare_gear_loadout / compose_gear / export_gear / import_gear` 在后台阻塞任务中调用同一 host。`prepare_gear_loadout` 统一生成默认配装并重置、迁移真实目录的旧配装；前端不再复制这套业务逻辑。合成返回 `Composition { composed, player }`，裸装 base 与手动总属性分开保存。
- 静态模式开启 `engine`；动态壳仅依赖 `default-features=false` 的模型，通过 `commands/gear_bridge.rs` dlopen gear。独立 ABI 1：`jpcg_gear_call(method,input_json)` 返回原有 status/result/message 信封，`jpcg_gear_free_string` 释放本库字符串，另有 ABI/版本 getter；无可变会话，panic 不跨 ABI。方法为 `catalog/prepare_loadout/compose/export/import`，不是时序模拟接口。
- 动态库搜索：显式 `JPCG_GEAR_LIB` → exe 的 `modules/` → exe 同目录 → CWD `target/debug` / `target/release`。缺库、缺符号或 ABI 不匹配直接报错。
- `make build-modules` / `modules-dir` 包含第五个 gear 模块；发布资产与 deploy-gen 模块清单同步加入 gear。Tauri bundle 资源按职业目录包含完整配装文件，首次安装即可读取。
- 核查动态壳不链接引擎：`cargo tree -p jpcg_app --no-default-features --features dynamic -e normal`；验证实际 bridge：先 `cargo build -p jpcg_gear`，再 `cargo test -p jpcg_gear --no-default-features --test dynamic_bridge`。`python3 scripts/test-gear-dynamic.py` 验证 C ABI、目录、错误及 TOML 往返。
- 前端新安装默认配装模式；#95 的 v1 存储迁移后仍保留手动模式。配装按心法保存、选项只来自校验后的目录；数值集不匹配时不计算。CD/加速仅展示已有元数据，连招仍调用现有接口，不新增 #92 规则。

## #98 原始属性换算

`Loadout.attribute_rules="profession-toml"` 显式从当前职业的 `data/{职业ID}/shuxing.toml` 读取 `[attributes]`。当前版本按50级对齐，但不自动填入未验证系数；职业版本、规则和所选值集等级必须一致。所有转换比例由文件提供，无四心法或130级常数白名单。`Stats` 新增五维及额外气血，输入必须为转换前原始点数；`Composed.panel` 返回派生面板与规则版本，旧手填继续保留原语义。历史130级规则ID需重新选择，不能使用缓存结果混算。录入示例、阶段、兼容与验证边界见 [属性引擎说明](../../docs/attribute-engine.md)。


## 联合搜索修复后的接口

- `gear-opt joint --request request.json --seconds 10`：短轮执行后正常取消并输出部分结果；`complete=false`、`stop_reason=cancelled`，不能作为全量最优证明。省略 `--seconds` 才执行完整空间；SIGINT/SIGTERM（Unix）或控制台 Ctrl-C（Windows）触发正常取消。
- 可穿的 `cross_school` 模板纳入候选，外门派的 `Choice.school` 显式保留；外门派装备不计本门派套装。
- `Candidate.score` 始终为目标本身，击杀率为 0..=1；同分时比较独立的 `tie_break_damage`。结果的 `mc_evaluations` 是实际批量 MC 数，不含基线。
- 伤害专用联合路径固定不影响伤害且不受硬约束的纯生存附魔/五彩石，优先使用用户当前选项；其余候选全部独立评分，不按进攻面板缓存去重。`pareto` 仅保留伤害最佳项。两个无缓存准备 worker 与评分重叠，消费端按批次序号归并；模板在每个候选前恢复 CD，再应用本次装备调整。
- 动态壳使用 JSON 方法 `optimize_joint_start`（返回 `job_id`）、`optimize_joint_poll`（进度及一次性结果/错误）、`optimize_joint_cancel`、`optimize_joint_release`，与静态壳共用联合引擎。`compute_backends` 返回实际平台清单。调用方退出时须 release，完整领取结果后会话自动移除；取消和错误不会冒充100%完成。原 `optimize` 方法保留预算语义。
- GPU批次按设备限制切片，每片独立绑定；事件缓冲区使用实际步长，支持超过64个事件，单个候选仍受设备存储限制。工作组归约保持相同采样序列与 HP 状态，设备缓冲区按容量复用。

本节实现已在ws通过相关Rust回归、静态/动态构建检查、model-only动态库会话、单张R9700 Vulkan数值/边界对拍和串行十秒测速；Mac待测。旧全量筛选计数属于修复前的本门派空间，新空间当前仅完成缩小真实配置空间的CPU/GPU结果一致性验证和短轮测速，不宣称已经完整搜索或证明全局最优。
