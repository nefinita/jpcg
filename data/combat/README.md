# data/combat — 战斗时序手工数据（#92）

这里放**手工填写**的战斗时序数据：形态、GCD 通道、读条/CD/充能、dot、定时 buff、秘籍、奇穴。

与 `data/shuxing/{门派}.toml` 的分工：

| 目录 | 内容 | 来源 | 谁来改 |
|------|------|------|--------|
| `data/shuxing/{门派}.toml` | 技能伤害基础数据（`base_damage` / `atk_xishu` / dot 标记等） | `server_tools/json-to-toml` 从游戏数据生成 | **不要手改**（重新生成会覆盖） |
| `data/combat/{门派}.toml` | 时序 overlay（本目录） | 手工填写 | 维护者 |

`data/combat/{门派}.toml` 按 `skill_id` + `sub_id`（技能唯一键）合并到 `data/shuxing/{门派}.toml` 的技能上；
只覆盖时序字段，伤害数值仍以 shuxing 为准。**未写在 overlay 里的技能完全保持原样**（默认按「1.5s GCD、瞬发、无 CD」处理）。

## 文件位置与加载顺序

加载器（`crates/jpcg_core/src/store/paths.rs::combat_dirs`）按以下顺序找 `<门派>/combat` 目录：

1. 设置了 `JPCG_DATA_DIR`：取 `<JPCG_DATA_DIR>/combat`（若 `JPCG_DATA_DIR` 指向 `shuxing/` 则取其父目录下的 `combat/`）；
2. 否则：可执行文件同目录、同目录上一级等候选根目录 → 各自的 `combat/`。

同名 `{门派}.toml` 取**第一个存在**的文件；解析或校验失败时，`load_combat` 会记日志并返回空数据（不 panic），
`load_skills_with_combat` 会返回带文件路径的错误，方便定位手填错误。

门派文件名与 `data/shuxing/` 一致：如 `mowen.toml`（莫问）、`daozong.toml`（刀宗）。

## 顶层结构

```toml
[meta]          # 可选
[[form]]        # 形态（0~N 条；不写 = 单一默认形态）
[[skill_combat]]# 技能时序 overlay（可选覆盖 + on_cast 副作用）
[[buff]]        # 定时 buff（技能通过 buff_id 引用）
[[miji]]        # 秘籍（激活后修正技能）
[[qixue]]       # 奇穴（激活后替换技能或施加修正）
```

所有结构体都 `deny_unknown_fields`：**写错字段名会解析失败**（错误信息会指出文件与字段）。

## `[meta]`

| 字段 | 类型 | 说明 |
|------|------|------|
| `profession` | String | 门派 id（与文件名一致，用于校验/展示） |
| `version` | String | 数据版本（自由文本，便于排查） |
| `note` | String | 备注 |

## `[[form]]`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | String | 稳定 id（引擎与前端都引用，如 `single` / `dual`），**必填且唯一** |
| `name` | String | 显示名 |
| `gcd_base` | f32 | 该形态的基础公共 CD（秒；`0` = 用门派默认 `xinfa.gcd_base`，缺省 1.5） |
| `initial` | bool | 初始形态（**至多一个**；都不标则以第一条为准） |
| `skills` | `[[u32, u32]]` | 该形态可用技能键（空 = 不限） |

双形态门派（如刀宗单持/双持、蓬莱上天）就靠这里：不同形态 `gcd_base` 不同，且**各自走独立的 GCD 通道**，
互不阻塞。形态切换通过技能 `on_cast.switch_form` 或手动 `switch_form` 动作。

## `[[skill_combat]]`

| 字段 | 类型 | 说明 |
|------|------|------|
| `skill_id` | u32 | 技能 id（对应 shuxing 数据） |
| `sub_id` | u32 | 子技能 id（`0` = 主技能） |
| `form` | String? | 所属形态 id（不写 = 任意形态可用） |
| `gcd_lane` | String | 占用的 GCD 通道（不写 = `default`） |
| `on_cast` | table | 施放副作用，见下 |
| `cast_time` | f32? | 读条时长（秒；`0` = 瞬发） |
| `gcd_base` | f32? | 该技能的基础 GCD 覆盖（秒；`0` = 用形态/门派默认） |
| `gcd_kind` | u8? | GCD 行为：`0` 普通 / `1` 不占 GCD / `2` 固定（不吃加速） |
| `cooldown` | f32? | 调息时间（秒） |
| `charges` | u8? | 充能层数（`0/1` = 无充能） |
| `recharge` | f32? | 每层充能恢复间隔（秒） |
| `cd_haste` | bool? | 调息是否吃加速（`true` = `cd / (1 + 加速率)`） |
| `channel` | bool? | 是否引导技能（时间轴上按读条处理） |
| `buff_id` | String? | 施放后施加的定时 buff id（见 `[[buff]]`） |

`on_cast`：

| 字段 | 类型 | 说明 |
|------|------|------|
| `switch_form` | String? | 施放后切换到的形态 id |
| `gcd_lanes` | `[String]` | 本次施放**额外占用**的 GCD 通道（如断云同时顶住双持通道） |
| `lock_skills` | `[[u32, u32]]` | 施放后把指定技能顶进 GCD（不可用至本次 GCD 结束） |

> 可选覆盖字段只在**写了**的时候生效，没写的字段沿用 shuxing 数据。

## `[[buff]]`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | String | 唯一 id（技能 `buff_id` 引用），必填 |
| `name` | String | 显示名 |
| `duration` | f32 | 持续秒数（`0` = 常驻至手动移除） |
| `max_stacks` | u8 | 最大层数（`0/1` = 不可叠加） |
| `refresh` | bool | 重复施加是否刷新持续时间（`false` = 独立实例各自计时） |
| `ignore_cap` | bool | 本 buff 的加速是否**可突破 25% 上限** |
| `effects` | array | 效果列表，见下 |

`effects` 里每一项是单键表（`snake_case`）：

| 键 | 含义 |
|----|------|
| `base_atk_pct` | 基础攻击 +% |
| `huixin_pct` | 会心率 +% |
| `huixiao_pct` | 会心效果 +% |
| `pofang_pct` | 破防 +% |
| `wushi_fangyu_pct` | 无视防御 +% |
| `shanghai_pct` | 伤害 +% |
| `jiasu_pct` | 加速率 +%（`2.0` = +2% 加速率） |

加速合并规则：**不可突破上限的加速先求和 → 截到 25% → 再加上可突破上限的加速**（`min(capped, 0.25) + uncapped`）。
GCD 与读条按 16 帧/秒量化：`floor(基础帧数 / (1 + 加速率))`，不足 1 帧按 1 帧。

## `[[miji]]`（秘籍）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | String | 唯一 id |
| `name` | String | 秘籍名称 |
| `skill_id` | u32 | 所属技能 id |
| `sub_id` | u32 | 所属子技能 id（`0` = 不限） |
| `modifiers` | array | 修正项列表，见下 |

秘籍是**挂在技能上的修正项**：激活后直接参与该技能伤害/时间计算，不会新增同名技能条目。

## `[[qixue]]`（奇穴）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | String | 唯一 id |
| `name` | String | 奇穴名称 |
| `slot` | u8 | 槽位（1~N，仅展示/约束） |
| `replaces_skill_id` | u32 | 被替换的默认技能 id |
| `replaces_sub_id` | u32 | 被替换的子技能 id（`0` = 不限） |
| `replacement` | table? | 替换后的独立技能条目（不写 = 只加修正、不替换） |
| `modifiers` | array | 附加修正项（作用于替换后技能；无 replacement 时作用于默认技能） |

`replacement` 的结构与 shuxing 的 `[[skill]]` 相同（只写需要的字段即可，其余默认 `0`/空）。
奇穴替换**优先于**秘籍修正：先用奇穴替换技能，再把秘籍/奇穴 modifiers 依次套上。

`modifiers` 每一项是单键表：

| 键 | 含义 |
|----|------|
| `base_damage_add` | 基础伤害 +N |
| `base_damage_mul` | 基础伤害 ×(1 + v/100) |
| `atk_xishu_add` | 攻击系数 +v |
| `hit_up_add` | 增伤 +v 个百分点 |
| `huixin_up_add` | 会心率 +v 个百分点 |
| `huixiao_up_add` | 会心效果 +v 个百分点 |
| `wushifangyu_add` | 无视防御 +v（1024 制） |
| `cast_time_mul` | 读条时长 ×(1 + v/100) |
| `cooldown_mul` | 调息时间 ×(1 + v/100) |
| `dot_duration_mul` | dot 持续时长 ×(1 + v/100) |
| `dot_interval_mul` | dot 每跳间隔 ×(1 + v/100) |

## 校验规则

加载时依次执行：

1. `validate()`：`form.id` 非空且唯一、初始形态至多一个、`skill_combat.form` 引用的形态存在、
   `buff.id` 唯一、数值非负/有限（读条、CD、gcd 等）。
2. `validate_against_skills()`：`skill_combat` / `miji` / `qixue` 引用的技能必须存在于 shuxing 技能表，
   `on_cast.lock_skills` 同理；`buff_id` 必须在该文件的 `[[buff]]` 中定义。
3. 失败时：`load_combat` 记日志返回空（前端退化为默认 1.5s GCD 预览）；
   `load_skills_with_combat` 返回 `data/combat/xxx.toml: ...` 形式的错误。

## 怎么验证 / 预览

- Rust 单测：`cargo test -p jpcg_core combat`、`cargo test -p jpcg_combo timeline`
- 前端：计算器页先点「计算」同步面板 → 排轴器页排好连招 → 点「时间轴预览」，
  会显示窗口伤害 / 窗口 DPS / 未执行动作 / 事件流（施放、dot 跳、形态切换、buff）。
- FFI（动态模式）：`jpcg_combo_call("load_combat_bundle", {"profession":"mowen"})`、
  `jpcg_combo_call("calculate_timeline", <TimelineRequestDTO>)`。

## 已知待定（写数据时注意）

- CD 是否帧量化：当前按 `cd / (1 + 加速率)` 连续缩放（不量化）。
- dot：施加时快照破防/会心/攻击/加速，总时长随加速缩短，跳数固定，逐跳均匀分布。
- 加速等级暂由 `PlayerConfigDTO.jiasu_dengji` 显式传入（前端暂无输入框），
  加速率 = 等级 / `jiasu_xishu`（值集 TOML，50 级 `101.079`）/ 100。
- 秘籍 / 奇穴数据暂不区分「是否已学习」，由前端勾选后传入。

参考模板：`_template.toml`（复制为 `{门派}.toml` 后填写；以 `_` 开头的文件不会被加载）。
