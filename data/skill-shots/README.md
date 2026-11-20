# 技能截图（人工维护）

这个目录放游戏内截图，由 `server_tools/skill-shots` 批量 OCR 成 JSON。

```
data/skill-shots/
  莫问/
    攻击力.png          # 属性面板：最终阳性/混元/阴性/毒性内功攻击 / 外功攻击 / 武器伤害
    技能/
      <技能名>/
        <截图>.png      # 技能 tooltip（伤害区间 + 攻击力贡献 + 释放/调息）
        秘籍/<截图>.png  # 「秘籍选择」整页截图（一页列出该技能全部秘籍）
    奇穴/
      第一重/<奇穴名>.png   # 该重里的一个可选奇穴；每个奇穴一张 tooltip 截图
      第二重/<奇穴名>.png
      第三重/<奇穴名>.png
      第四重/<奇穴名>.png
```

截图要求：**技能 tooltip 与奇穴 tooltip 只截面板本身**（Cmd+Shift+4 拖拽），
秘籍选择页才能整页截（工具会按 4×2 切卡）。换版本或技改时：替换图片 →
`cargo run -p skill-shots -- data/skill-shots`。

> 截图体积大，默认不提交仓库（见根 `.gitignore`）。需要长期留档时自行决定是否提交。
