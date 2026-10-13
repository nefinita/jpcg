
请使用 VSCode 或以 UTF-8 编码打开 TOML 文件，然后按下述格式填写。

说明：本目录下提供两种示例：`_template.toml`（字段注释版）和 `mowen.toml`（真实数据示例）。
数据文件由 `server_tools/json-to-toml` 从数据源 JSON 生成，**推荐自动化生成而非手填**（见下文）。

## 文件结构

```toml
[xinfa]              # 心法元数据
xinfa_name = "莫问"  # 心法名称
xinfa_nom = "根骨"   # 主属性（根骨/元气/力道/身法/体质）
atk_up = 1.96        # 攻击倍率
pofang_up = 2.0      # 破防倍率
huixin_up = 0        # 会心倍率

[version]            # 数据版本
level = 130          # 游戏等级
season = 3           # 赛季号
modified = 20260814  # 数据日期 YYYYMMDD

[[skill]]            # 每个技能（或技能形态）一条
skill_name = "宫"    # 技能名（同名多形态带「·形态」后缀，如 引窍·50点任脉）
skill_id = 10447     # 技能 ID（数据源池内 id）
sub_id = 14474       # 子技能 ID（同 skill_id 不同形态的区分键）
base_damage1 = 160   # 基础伤害
base_damage2 = 200   # 基础伤害2
atk_xishu = 501      # 伤害系数
watk_xishu = 0       # 武器伤害系数
hit_up = 0           # 增伤乘区
huixin_up = 0        # 额外会心
huixiao_up = 0       # 额外会效
wushifangyu = 0      # 无视防御
wushihuajin = 0      # 无视化劲
has_critical_strike = false  # 无质标签：true=伤害固定为期望 Q（无随机浮动）
zhenshishanghai = 0  # 真实伤害（>0 表示无视防御减免）
lost_hp_zhenshishanghai = 0  # 追加真伤（目标已损失生命值 × 系数，无视防御；连招中动态结算）
dot_flag = 0         # dot 标签（1=持续伤害）
dot_interval = 1.5   # dot 每跳间隔（秒，浮点，仅 dot_flag=1）
dot_duration = 3.0   # dot 总时长（秒，浮点，仅 dot_flag=1）
dot_up = 0           # dot 每跳递增比例（等比，可选）
```

### 说明
- **形态全量**：同一技能的等级/层数/点数形态全部独立成条（如段氏引窍 0~100 点任脉
  共 101 条，命名 `引窍·0点任脉`~`引窍·100点任脉`），前端技能池按基础名分组折叠
- 数值字段全部按**数据源原值**写入，不做除法（引擎侧已按 `X_MULT` 归一化）
- `dot_interval`/`dot_duration` 为浮点秒（16 帧=1 秒）；跳数由引擎按
  `dot_duration/dot_interval` 推导

## 数据源获取

数据源仓库：[IcyTide/Generator](https://github.com/IcyTide/Generator)（`assets/json/`，master 分支）。
三个 JSON 文件：

| 文件 | 内容 |
|------|------|
| `skills.json` | 技能伤害数据（池 → skill_id → sub_id → lv） |
| `dots.json` | 持续伤害（DOT）数据 |
| `belongs.json` | 技能归属/描述（心法池、招式 desc、奇穴归属） |

一键拉取（下载到 `data/raw-src/`，已 gitignore）：

```sh
server_tools/json-to-toml/fetch-json.sh
```

## 自动化生成（推荐）

数据源 JSON（IcyTide/Generator）→ `server_tools/json-to-toml`：

```sh
cargo run -p json-to-toml -- --skills <skills.json> --dots <dots.json> \
    --overrides server_tools/json-to-toml/overrides.json \
    --out data/shuxing --xinfa <技能池id>
```

全职业导出（27 个伤害心法；治疗心法/通用池在数据源中无伤害技能，跳过）：

```sh
for p in 10002 10003 10014 10015 10021 10026 10062 10081 10144 10175 \
         10224 10225 10242 10243 10268 10389 10390 10447 10464 10533 \
         10585 10615 10627 10698 10756 10786 10821; do
  cargo run -q -p json-to-toml -- --skills $SKILLS --dots $DOTS \
      --overrides server_tools/json-to-toml/overrides.json \
      --out data/shuxing --xinfa $p
done
```

`overrides.json` 每个池配置：`file`（输出文件名，10447→mowen.toml、10786→zhoutian.toml 沿用历史名）、
`xinfa_name`（belongs.json 心法名）、`xinfa_nom`（主属性，从 belongs.json `*_to_*_attack_power` 推断：
spirit→根骨、spunk→元气、strength→力道、agility→身法、vitality→体质）、
`atk_up`/`pofang_up`（**数据源无此数据**：莫问/周天功为手写校准值，其余职业暂用 1.0 占位，待逐职业校准）、
`wuzhi`（无质技能名单，按 name 精确匹配，数据源漏标时兜底）、
`lost_hp_zhenshi`（追加真伤配置：`{name, per_layer, max_layers}`，命中技能展开为破绽 0..N 层形态，
每层追加真伤 = 目标已损失生命值 × per_layer × 层，如刀宗「怒锋倾涛」每层 6% 最多 3 层）。

转换器规则：
- **全量输出**：所有 lv 形态全部保留（等级/层数/点数等均为独立可调形态），不压缩
- **命名语义化**：同名多形态优先用数据源 comment 生成 `名·形态` 后缀
  （如 `引窍·50点任脉`），无 comment 退回 `名(lv{N})`；追加真伤形态再叠 `·破绽N层`
- `has_critical_strike`（无质）：数据源 `critical_strike`/`critical_power` 任一为 `"0"/"O"` 占位
  （新数据源已批量标注无质技能）→ 无质；overrides `wuzhi` 名单命中 → 无质
- `zhenshishanghai`：数据源 `custom_damage_base` 存在（真实伤害标签，当前仅装备特效池 0）
- `lost_hp_zhenshishanghai`：overrides `lost_hp_zhenshi` 名单驱动（数据源无数值，desc 描述驱动），
  连招中引擎按目标已损失生命值动态结算（确定性伤害，只加期望不加方差）
- dot：`dot_interval`/`dot_duration` 按数据源帧换算为秒；`dot_up` 从公式等比系数提取

完成后推荐用 VSCode 检查 TOML 的编码为 UTF-8 并保存。
