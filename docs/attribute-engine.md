# 装备属性换算引擎（#98）

本实现先补齐「体质 → 气血」与「基础属性 → 会心」的可验证链路。它是**显式选择的历史 130 级莫问规则**，不是全部心法、当前版本或所有装备系统的完全复刻。`data/gear` 的 `synthetic-v1` 仍是人为样例；没有新增真实装备库，也没有修改现有手填金标准。

## 规则与来源

规则 ID：`jx3-130-mowen-20260814`。只接受 `school=mowen`、根骨、内功攻击/破防、`value_set=live-130`；输出在该规则内明确为**阴性**面板。其他心法、体验服、未知 ID 均报错，不套用占位系数。没有自动选择“最新”版本。

独立实现依据为 IcyTide/Generator 的固定提交 [`21f472e211581ac1c1a7fa5ea2ff9199df6151cb`](https://github.com/IcyTide/Generator/commit/21f472e211581ac1c1a7fa5ea2ff9199df6151cb)（2026-08-14），与仓库莫问技能数据的修改日期相符：

- [`base/constant.py`](https://github.com/IcyTide/Generator/blob/21f472e211581ac1c1a7fa5ea2ff9199df6151cb/base/constant.py)：LEVEL=130；体质转气血10、根骨转内功会心0.9、元气转内功攻击0.181及破防0.3。会心/破防/会效分母与 `live-130` 相符；这不等于证明整个赛季内容一致。
- [`assets/json/belongs.json` 的 10447/10447](https://github.com/IcyTide/Generator/blob/21f472e211581ac1c1a7fa5ea2ff9199df6151cb/assets/json/belongs.json#L7184)：莫问根骨转阴性攻击2007/1024、转阴性会心410/1024、气血加成307/1024；无额外破防转换。
- [`base/attribute.py`](https://github.com/IcyTide/Generator/blob/21f472e211581ac1c1a7fa5ea2ff9199df6151cb/base/attribute.py)：基础与额外转换分开截断，攻击增益先作用于基础攻击，之后加心法额外攻击；气血基础阶段先计体质再乘心法增益。

该来源是可执行的社区计算器模型，**不是官方游戏面板实测**。未发现该提交的许可证；本仓库不复制其代码、不批量再分发其数据，仅独立表达核对过的公式、必要常数及来源链接。上游当前 master 的等级和数值已变化，禁止混用。

## 输入、阶段与取整

`Loadout.attribute_rules` 可选；缺失保持旧语义，旧存档不自动迁移为新计算口径。驾驶台的规则选择与配装一起保存/导入/导出，默认仍是旧模式。

`Stats` 新增 `tizhi/liliang/shenfa/gengu/yuanqi`，缺失为0。旧 `zhushuxing` 是主属性**增量别名**；莫问 `G = gengu + zhushuxing`，各来源只加一次。不要把同一份根骨同时填到两栏。力道/身法保留供以后扩展，但不影响莫问的阴性输出。

`base`、模板和套装属性都必须是**基础属性及心法转换前的平铺点数**。脱装后的最终游戏面板通常已含转换，不能直接用来填这些原始值。系统不自动加44点基础属性、心法固有攻击6518、固有会心3962等裸装数值，避免与输入双算。需要完整裸装值时必须由可核实的数据显式提供。

合成顺序：裸装原始值 + 12件模板 + 已激活套装的原始点数 → 一次换算 → 玩家DTO。

设 `Y=yuanqi`、`T=tizhi`，`floor` 为非负整数向下取整：

```text
基础攻击 = gongji + floor(Y × 181 / 1000)
心法额外攻击 = floor(G × 2007 / 1024)
面板攻击 = 基础攻击 + 心法额外攻击
会心等级 = huixin + floor(G × 9 / 10) + floor(G × 410 / 1024)
破防等级 = pofang + floor(Y × 3 / 10)
气血 = floor((qixue + T × 10) × 1331 / 1024)
```

每个转换项单独截断，不合并系数。中间乘法使用u64，输入求和与最终u32越界返回错误，不静默饱和。气血、攻击、会心、破防都给出零输入与临界值测试；例如根骨1时会心是0+0=0，合并系数会错误产生1。

未选规则时若输入非零体质或具名基础属性，拒绝计算，防止数据被旧适配器静默丢弃。旧模式仍保留主属性→攻击的原有浮点口径，并显示未实现体质/基础属性会心转换的说明。

## 接入与兼容

- `Composed.stats` 始终保留原始合计，新增 `panel` 保存派生面板及规则、来源、`verified_in_game=false`。不要将 `panel` 再作为 `base` 输入。
- `jpcg_gear::player()` 是唯一玩家适配器：会心/破防使用面板等级，加速完整传递；`jichu_gongji` 为基础攻击，新增可选 `zhuanhua_gongji` 为已转换的心法额外攻击。
- `zhuanhua_gongji=None` 保持旧 core 公式与取整；`Some(0)` 也明确禁止再次做主属性转换。新路径为 `floor(基础攻击 × (1+基础攻击增益/100)) + 心法额外攻击`。武器伤害仍独立参与技能公式。
- core 单技能、combo、时间轴、配置保存/加载均使用同一 DTO 转换，防止加速或转换标记丢失。前端请求归一化也保留这些字段。后端仍可读取已转换的配置快照；手动表单拒绝把这种缺少原始五维数据的快照载入编辑，提示改为导入原始配装 TOML，避免编辑主属性后使用过期转换结果。
- 玩家气血属于自身面板，**不会覆盖敌人的 target_hp/max_hp/current_hp**。配装约束在启用规则时使用派生点数（攻击、会心、破防、气血、根骨及主属性），其余属性仍直接传递；Pareto 生存比较也使用派生气血。旧模式的约束行为不变。
- 旧属性求导只会改变 DTO 主属性，不能重新计算已经转换的会心/额外攻击。因此携带转换标记的请求明确报错、要求重新合成配装；未伪造主属性零收益。
- C ABI仍是 JSON 协议；新增字段向旧存档兼容。运行新配装规则时必须配套更新 app/gear/core/combo 模块；旧动态库会忽略未知字段，**混用旧 core/combo 不受支持**。

## 核验与剩余范围

`crates/jpcg_gear/tests/fixtures/attributes-130-mowen.json` 是独立手算的固定样例，不是游戏截图。测试覆盖分段取整、溢出、版本错误、原始来源合成一次、气血约束、优化目标与 core 一致、宿主/TOML 往返，以及前端模式与持久化。

可运行 `python3 scripts/test-attributes-dynamic.py` 测试实际 gear 动态库；附加 `--reference-dir /path/to/trusted/Generator --samples 10000` 时要求该目录为上述固定提交的干净 checkout，直接对比 Rust 合成面板与实际上游执行结果。脚本不下载或再分发上游代码。

开发时在临时目录实际执行固定提交的上游 `BaseAttribute`，与独立公式对照：4组典型输入及10,000组固定种子随机输入（每项0..1,000,000）零差异。并对0..1,000,000范围比较0.181/0.163/0.9/0.3浮点截断与整数分数，以及元气u32边界。该实验不等于全u32域或真实游戏一致性的证明。

仍需维护者提供/确认：真实游戏版本与面板前后对照、其他心法和伤害类型、真实裸装/装备/套装数值、强化/镶嵌/附魔/特效、混档套装语义，以及体质/气血/基础属性战斗增益的完整叠加规则。本实现不改 #92 的战斗调度或套装CD，不能将体质增加解释成对外输出伤害的无条件增加。issue #98 应保持开放，直到这些范围有独立基准支撑。

### 2026-10-31 云环境验证记录

- `cargo test --locked --workspace --exclude jpcg_app`：140项通过（子进程重复执行不重复计数），含9项既有金标准。
- `cargo clippy --locked --workspace --exclude jpcg_app --all-targets`：通过，保留仓库既有告警。
- 前端 `npm test`：79项通过；`npm run build`：TypeScript + Vite通过。
- `cargo build --locked -p jpcg_core -p jpcg_combo -p jpcg_gear`、gear的 `--no-default-features --test dynamic_bridge`：通过。
- `scripts/test-gear-dynamic.py`（含新属性动态回归）、`scripts/test-values-dynamic.py`：通过。
- `scripts/test-attributes-dynamic.py --reference-dir <固定提交checkout> --samples 10000`：实际 Rust动态库与上游10,000组随机样例零差异，seed=98。
- `cargo fmt --all -- --check`、`git diff --check`：通过。
- `make check-all` 在桌面依赖检查处失败：环境缺少 `glib-2.0.pc`（要求GLib≥2.70）。未验证原生桌面构建/GUI，不能将上述引擎与前端结果当作完整桌面检查通过。
