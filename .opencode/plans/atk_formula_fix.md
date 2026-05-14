# 攻击力公式修正方案 B

## 修改文件

### 1. `crates/jpcg_core/src/type_set/player.rs` 第 66 行

```rust
// 旧
let base = (self.jichu_gongji + self.jichu_shuxing) as f32 * (1.0 + atk_up);
// 新
let base = self.jichu_gongji as f32 + self.jichu_shuxing as f32 * atk_up;
```

### 2. `crates/jpcg_core/src/cal/atkcal.rs` 第 74 行

```rust
// 旧
self.player.atk(0.0).total()
// 新
self.player.atk(self.xinfa.atk_up).total()
```

## 效果

以莫问 `atk_up=1.96`、`jichu_gongji=200`、`jichu_shuxing=100`、`wuqi_shanghai=50` 为例：

```
B = 200 + 100 × 1.96 + 50 = 446
```

`atk_up` 只作用于基础属性，不再作用于基础攻击。per-skill 的 `atk_up` 以后通过 `b_cal()` 传入即可生效。
