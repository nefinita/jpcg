
请使用 VSCode 或以 UTF-8 编码打开 TOML 文件，然后按下述格式填写。

说明：本目录下同时提供两种常见格式的示例：`template.toml`（字段注释版）和 `mowen.toml`（真实数据示例）。

字段说明：

- `xinfa_name`：心法名称（String）
- `xinfa_nom`：心法基础属性（String），例如 `根骨`、`力道` 等
- `atk_up`：攻击倍率（浮点数）
- `pofang_up`：破防倍率（浮点数）

每个技能为一个 `[[skill]]` 条目，常用字段：

- `skill_name`：技能名（String）
- `skill_id`：技能 ID（u16，可选；不影响计算可置 0）
- `sub_id`：子 ID（可选）
- `group`：套路组编号（u8）
- `weapon_request`：需要武器编号（u8）
- `design_effect`：技能生效方式（u8，0=直接伤害，1=Dot）
- `kind_type`：技能类型（u8，0=外功，1=阴性，2=混元，3=毒性，4=阳性）
- `cast_mode`：释放方式（u8，0=单体，1=群体）
- `guaranteed_hit`：必定命中（bool，true/false）
- `has_critical_strike`：可暴击（bool）
- `effect_type`：技能效果（u8，0=有害，1=有益）
- `jihuoqixue`：激活奇穴（String，可空）
- `base_damage1` / `base_damage2`：基础伤害（u32，两个值用于不同情况）
- `atk_xishu`：伤害系数（浮点或数值）
- `watk_xishu`：武器伤害系数（数值）
- `hit_up`：增伤（u32 或百分比数值）
- `huixin_up`：额外会心（u32）
- `huixiao_up`：额外会效（u32）
- `wushifangyu`：无视防御（0-100，写作百分数数值，例如 40 表示 40%）
- `wushihuajin`：无视化劲（0-100）
- `wushijianshang`：无视减伤（0-100）
- `zhenshishanghai`：真实伤害（0-100）
- `dot_flag`：是否为 DOT（0/1 或 bool，根据示例）
- `dot_up`：DOT 增益（可选，浮点）

注意：写入时优先参考 `template.toml` 的字段名，请确保列含义与上面说明一致。

示例（取自 `template.toml`）：

```toml
xinfa_name = "莫问"
xinfa_nom = "根骨"
atk_up = 1.96
pofang_up = 2.0

[[skill]]
skill_name = "宫"
skill_id = 10447
group = 1
weapon_request = 0
design_effect = 0
kind_type = 0
cast_mode = 0
guaranteed_hit = false
has_critical_strike = true
effect_type = 0
jihuoqixue = ""
base_damage1 = 160
base_damage2 = 200
atk_xishu = 501
watk_xishu = 0
hit_up = 0
huixin_up = 0
huixiao_up = 0
wushifangyu = 0
wushihuajin = 0
wushijianshang = 0
zhenshishanghai = 0
dot_flag = 0
```

所有字段应完整填写；若某些字段对计算结果无影响，可先填 `0` 或空字符串以占位。完成后推荐用 VSCode 检查 TOML 的编码为 UTF-8 并保存。