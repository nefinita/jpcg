# data 数据维护指南（Agent）

本文件适用于整个 `data/`。这里同时存放运行期数据与数据生产材料；修改前先确认来源、等级、赛季、值集和下游口径，不要把示例值或截图识别值写成已核实的游戏事实。正式数据根是本目录，不要在下面再嵌套一套 `data/data/`。

## 目录职责

正式运行数据按职业 ID 分目录：

```text
data/
  <profession>/
    shuxing.toml       # 技能、心法元数据与属性换算
    combat.toml        # 手工战斗时序 overlay
    gear/              # 本职业装备目录与调用样例
  _template/
    shuxing.toml
    combat.toml
  values/              # 全职业共享的值集清单与快照
  skill-shots/         # 数据生产证据，不属于客户端运行资源
  raw-src/             # 上游 JSON 缓存，已被 gitignore
```

| 路径 | 内容 | 修改方式 |
| --- | --- | --- |
| `<profession>/shuxing.toml` | 心法元数据、技能伤害、DOT 基础参数、职业属性换算 | 由 JSON 转换器或截图工具生成后人工校准；不要盲目批量覆盖 |
| `<profession>/combat.toml` | 器形、形态、GCD/读条/CD、时序 DOT、buff、秘籍、奇穴与界面规则 | 手工维护的 overlay，不写技能基础伤害真源 |
| `<profession>/gear/` | 装备、套装、门派映射、覆盖、附魔、五彩石与样例请求 | 按统一 revision/value_set 维护；真实值与合成样例明确区分 |
| `values/` | 换算常数快照及 `index.toml` 清单 | 新值集新增快照并登记，不原地混用不同等级 |
| `skill-shots/` | 技能截图、OCR 中间结果和汇总 JSON | 数据生产证据，保留来源与可复核性 |
| `raw-src/` | 上游 JSON 下载缓存 | 已被 gitignore，不作为入库真源 |

新增职业时新增完整的 `data/<profession>/`，职业 ID 只允许字母、数字、下划线和连字符。技能、时序和配装都按同一个 ID 关联；不要只复制莫问文件改个名字就宣称支持新职业，未核实的系数和装备必须明确标记或保持缺失并报错。

`_template/` 只解释结构，可能保留历史等级示例，不参与正常职业加载。当前真实数据应参考对应职业文件，而不是把模板等级当作项目现状。当前仓库只有 `mowen/` 一份真实职业数据。

旧 `data/shuxing/<profession>.toml`、`data/combat/<profession>.toml` 与 `data/gear/` 仅作读取兼容，不保留仓库副本。每个数据根内都是新布局优先、旧布局其次；技能编辑只保存到 `<profession>/shuxing.toml`。新增文件不要写回旧目录。

## 通用约束

- 全部文本使用 UTF-8。多数运行期结构采用 `deny_unknown_fields`，字段拼错应解析失败，不能靠忽略未知字段“兼容”。
- 修改前保留可回溯来源：上游 commit/JSON、游戏截图、OCR 输出、校准公式或人工验证记录。无法核实的字段标为待核验，不编造精确值。
- `[version].level`、`[attributes].level`、`attributes.value_set`、gear 的 `value_set` 与所选值集等级必须一致。禁止把 50 级职业规则与 130 级技能/常数混算，也不要只改等级标签冒充迁移。
- `sample=true` 或 `revision=synthetic-*` 是人为测试数据；`sample=false` 也需要 `provenance` 支撑。样例与真实数据不能共用含混 revision。
- 数据加载优先级为 `JPCG_DATA_DIR` → 安装版用户目录 → 随包资源 → exe 同目录 `data/`。高优先级文件存在但损坏时必须报错，不能静默退回旧版本。
- 职业目录选择、默认配装、旧配装修复和属性规则判断均由 Rust 后端完成；前端只传职业 ID／用户输入并展示结果，不在 TypeScript 里另写一套规则。
- 数据修改也要新增根目录 `changes/YYYY-MM-DD-HHMMSS.md`，注明来源、影响职业/值集以及是否经过游戏内验证。

## shuxing：技能与职业换算

### 技能数据

每个职业一个 `<profession>/shuxing.toml`，主要结构为 `[xinfa]`、`[version]`、`[[skill]]`，可选 `[attributes]`。技能唯一键是 `skill_id + sub_id`，名称不是可靠主键。

- `[xinfa]`：`xinfa_name`、`xinfa_nom`、`atk_up`、`pofang_up`、`huixin_up`。
- `[version]`：`level`、`season`（若当前格式使用）、`modified=YYYYMMDD`，可附来源说明。
- `[[skill]]`：基础伤害、攻击/武器系数、增伤、会心/会效、无视防御/化劲、无质、真伤、追加已损失生命真伤与 DOT 基础字段。
- 多等级、层数、点数或技能形态独立成条，使用稳定的 `名·形态` 命名；不要为压缩文件丢掉可选形态。
- `has_critical_strike=true` 表示无质，伤害固定为期望 Q；`lost_hp_zhenshishanghai` 由 combo 随 HP 动态结算，不当作固定真伤。
- 上游 JSON 的数值是否保持原值由对应转换器决定；截图/OCR 数据则按当前生成工具口径写入。两套来源不可不经核对互相覆盖。

批量 JSON 来源为 IcyTide/Generator 的 `skills.json`、`dots.json`、`belongs.json`。下载与转换：

```sh
server_tools/json-to-toml/fetch-json.sh
cargo run -p json-to-toml -- \
  --skills data/raw-src/skills.json --dots data/raw-src/dots.json \
  --overrides server_tools/json-to-toml/overrides.json \
  --out data --xinfa <技能池id>
```

`--out` 是数据根；转换器实际写入 `<out>/<profession>/shuxing.toml`，不会覆盖同职业的 `combat.toml` 或 `gear/`。`overrides.json` 负责输出职业 ID、心法名/主属性、校准倍率、无质名单和追加真伤形态。上游未提供的职业倍率不能用 `1.0` 冒充已校准结果；批量再生成前先检查现有文件是否已包含截图校准或 `[attributes]`。

### `[attributes]` 原始属性换算

只有显式选择 `attribute_rules="profession-toml"` 时才走该表。规则包括 `id/level/value_set/source/main_stat/damage_kind/qixue_multiplier`、`[[attributes.conversions]]` 与需要的常量项。

- 装备和裸装 `base` 填转换前原始点数，不能填已经合成的最终面板；`zhushuxing` 是主属性增量别名，不得与同一具名属性重复填写。
- 每条转换按 `floor(原始点数 × numerator / denominator)` 独立截断，不先合并比例，也不把一条结果喂给另一条。
- 基础气血先乘显式 `qixue_multiplier` 并截断，再加额外气血；没有增益也要明确写 `1/1`。
- 缺表、缺必需字段、非法枚举、分母为零、职业/伤害类型不匹配、等级或值集不匹配都应失败；缺失不等于“已确认系数为零”。
- 合成结果保存规则 ID、等级、值集和来源。文件重读后必须重新合成，旧面板缓存不能继续驱动伤害或连招。
- 完整契约、兼容规则与证据边界见 `docs/attribute-engine.md`，适配器见 `jpcg_gear::player()`。

## combat：手工时序 overlay

`<profession>/combat.toml` 按 `skill_id + sub_id` 合并到同职业 shuxing 技能，只覆盖时序与修正。未声明的技能默认瞬发、1.5 秒 GCD、无 CD；基础伤害仍以 shuxing 为准。

顶层可使用：

```toml
qifeng = ["yangchun", "gaoshan", "pingsha"]  # 仅需要的职业声明
[meta]
[[form]]
[[skill_combat]]
[[buff]]
[[miji]]
[[qixue]]
```

关键约定：

- `[[form]]` 定义器形/武器形态：`id/name/gcd_base/initial/skills`；技能用 `requires_instrument` 限定器形，切换用 `on_cast.switch_instrument`。`name` 同时用于界面显示和技能过滤。
- 形态与器形是两个维度：技能可写 `requires_qifeng`、`qifeng_gain`、`spend_all_qifeng`，切换用 `on_cast.switch_qifeng`；`[meta].qifeng_names` 提供形态 id 到显示名的映射。界面统称「形态」，内部 qifeng 字段与点数结算暂时保持兼容，资源引擎泛化仍按 `PLAN.md` 单独实施。
- `[meta]` 必须显式声明 `miji_per_skill_cap`、`mixed_qixue_cap`、`mixed_slot_from`，前端按下发值渲染秘籍槽和混选规则。后端默认值只用于读取旧文件，新增职业不得依赖默认值。
- `[[skill_combat]]` 常用字段：`cast_time`、`channel/channel_ticks`、`gcd_base/gcd_kind/gcd_lane`、`cooldown/cd_haste`、`charges/recharge`、`buff_id`、`jihuoqixue`。
- `on_cast` 还可声明 `gcd_lanes`、`lock_skills`、`advance_dot_targets/advance_dot_ticks` 等副作用；以 `crates/jpcg_core/src/type_set/combat.rs` 的当前结构为准。
- `[[buff]]` 定义持续时间、层数、刷新/独立实例和效果；`[[miji]]` 挂到技能上修正，`[[qixue]]` 可替换技能并附加修正。奇穴替换先于秘籍/奇穴 modifiers。
- 时序 DOT 可在 overlay 写 `dot_interval/dot_duration` 与每跳伤害参数。DOT 施加时快照攻击/会心/破防/加速，持续时长随加速变化而跳数固定。
- GCD 与读条按 16 帧/秒量化，普通加速封顶 25% 后再加可突破上限的部分；CD 当前按连续比例缩放，不要自行帧量化。
- 引用的器形、形态、技能、buff、秘籍和奇穴必须存在且 ID 唯一。`_template/combat.toml` 与 `mowen/combat.toml` 分别提供结构和真实示例。

## gear：配装目录与搜索输入

每个 `<profession>/gear/` 的核心文件包括 `templates.toml`、`sets.toml`、`schools.toml`、`overrides.toml`、`enchants.toml`、`wucai.toml`，另有 `sample-loadout.toml`、`sample-request.toml` 作为调用样例。完整字段契约见 `crates/jpcg_gear/README.md`。

- 各目录文件的 `[version]` 必须按契约一致，至少核对 `schema/season/revision/value_set/sample/provenance`；loadout 也必须选择匹配的 school/revision/value_set。
- 装备唯一键包含 school 与完整 choice（slot/source/tier/variant）；12 槽位、饰品双属性套、套装阈值、跨门派可穿规则均由目录校验，不能在前端另造一套。
- `base` 是裸装原始值；装备、套装、附魔、五彩石点数也是属性换算前输入。不要把 `Composed.panel` 再作为 `base`，否则会重复转换。
- 装分只展示，不参与搜索。套装 CD 必须进入支持的 timeline 适配；使用不支持 CD 的旧连招目标时要显式确认，不能静默忽略。
- `complete=true` 只表示对应搜索空间已遍历；若跨门派模板、装饰或档位被排除，必须准确描述空间。取消/超时结果不能写成全量最优。
- 固定 seed 的 50k MC 用于候选间可重复比较；CPU/GPU 对拍还要核对候选顺序、HP 状态、计数和完整结果，而不只看最佳概率。

常用验证（默认 CLI 目录是 `data/mowen/gear`，其他职业显式传 `--catalog`）：

```sh
cargo run -p jpcg_gear --bin gear-opt -- validate
cargo run -p jpcg_gear --bin gear-opt -- compose --loadout data/mowen/gear/sample-loadout.toml
JPCG_DATA_DIR="$PWD/data" cargo run -p jpcg_gear --bin gear-opt -- optimize --request data/mowen/gear/sample-request.toml
cargo test -p jpcg_gear --locked
```

## values：数值集

`values/index.toml` 的 `default` 与 `[[value_sets]]` 是运行期清单；每项包含 `id/name/level/file`，对应快照保存换算常数。新增值集时新增独立快照并登记，旧快照不因当前赛季变化而被重写。

- 客户端未显式选择时使用 `default`；手动改过的系数不应被值集切换无声覆盖。
- 技能、职业 `[attributes]` 与 gear 必须明确匹配值集。配装优化要求可用值集，禁止静默回退 builtin。
- 值集热更新后需要刷新相关快照与合成结果；缓存校验通过不代表跨等级数据可混用。

## 验证与发布

按改动范围选择验证，不以“TOML 能解析”代替数值回归：

```sh
cargo test -p jpcg_core combat
cargo test -p jpcg_combo timeline
cargo test -p jpcg_core -- golden
cargo test -p jpcg_gear --locked
python3 scripts/test-attributes-dynamic.py  # 已构建动态库时验证职业换算
```

发布数据清单由工具递归生成，不手工计算哈希：

```sh
cargo run -p manifest-gen -- --version vX.Y.Z --data-dir ./data
```

生成器没有目录排除规则：`raw-src/`、截图中间产物、嵌套副本等只要在 `data/` 下就会进入清单。发布前必须整理输入目录并检查清单文件列表，不能仅凭 gitignore 假设它们不会下发。

服务器 URL、stable/beta 目录与模块布局见 `server_manifest.md`。未经明确要求，不执行上传、部署或发布。
