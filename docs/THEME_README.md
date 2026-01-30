# 主题系统快速上手

## 概述

已成功集成 **Epic Desert Relic** 主题系统，提供完整的配色方案管理。

## 文件结构

```
assets/
  └─ themes/
     └─ epic_desert_relic.toml  # 主题配置文件

src/
  └─ main.rs
     └─ ThemePlugin            # 主题插件
     └─ Theme (Resource)       # 主题资源
```

## 快速使用

### 1. 在系统中注入主题

```rust
fn my_system(theme: Res<Theme>) {
    // 直接使用主题颜色
    let text_color = theme.text.primary;
    let bg_color = theme.bg.void_ink;
}
```

### 2. 颜色类别

| 类别 | 访问方式 | 说明 |
|------|---------|------|
| 背景色 | `theme.bg.*` | void_ink, basalt_blue, obsidian_moss, ashen_slate |
| 地形色 | `theme.earth.*` | ruined_umber, desert_bronze, silt_gold, sunbaked_clay, bone_parchment |
| 文字色 | `theme.text.*` | primary, secondary, muted, disabled |
| 语义色 | `theme.semantic.*` | success, info, warning, danger, fire, toxic, psi, rare, tech_neon |
| 稀有度 | `theme.rarity.*` | common, uncommon, rare, epic, legendary, artifact |
| 灰度 | `theme.grayscale.*` | g00_black ~ g100_white (13级) |

### 3. 实战示例

```rust
// 绘制物品
canvas.set_string(x, y, "[传说] 神器", theme.rarity.legendary);

// 绘制状态
canvas.set_string(x, y, "HP: 85/100", theme.semantic.success);

// 绘制面板
canvas.fill_rect_with_bg(
    x, y, w, h,
    ' ',
    theme.text.primary,
    theme.bg.basalt_blue,
);

// FOV 雾效
let fog_color = theme.grayscale.g60_fog;
```

## 修改主题

编辑 `assets/themes/epic_desert_relic.toml`:

```toml
[colors.semantic]
success = "#31D07A"  # 修改成功色
danger = "#E0493E"   # 修改危险色
```

修改后重新运行程序即可生效。

## 创建新主题

1. 复制 `epic_desert_relic.toml` 为新文件
2. 修改颜色值和元数据
3. 在 `load_theme` 函数中更新文件路径

## 主题演示

运行程序可查看完整的主题演示，包括：
- ✓ 所有颜色分类展示
- ✓ 实战 UI 场景示例
- ✓ 文字层次对比
- ✓ 灰度梯度演示

## 特性

✅ 完整的配色体系 (背景/地形/文字/语义/稀有度/灰度)  
✅ TOML 配置，易于修改  
✅ 类型安全的颜色访问  
✅ 沙漠遗迹风格 (Qud-like)  
✅ 支持自定义主题  
✅ 背景色自动应用  

## 技术细节

- **插件**: `ThemePlugin` - 在 Startup 阶段加载主题
- **资源**: `Theme` - 全局主题资源，可在任何系统中注入
- **配置**: TOML 格式，支持注释
- **解析**: 使用 `toml` crate 解析配置文件
- **颜色**: 所有颜色都是 Bevy 的 `Color` 类型

## 相关文档

- [详细使用指南](./THEME_USAGE.md) - 完整的 API 文档和示例
- [主题配置](../assets/themes/epic_desert_relic.toml) - 主题配置文件

## 问题排查

**主题未加载？**
- 检查 `assets/themes/epic_desert_relic.toml` 是否存在
- 查看控制台日志

**颜色不对？**
- 确认 hex 颜色格式: `"#RRGGBB"`
- 检查 TOML 语法

**性能问题？**
- 主题只在启动时加载一次
- 运行时访问是零开销的

---

**主题风格**: Epic Desert Relic (沙漠遗迹史诗)  
**灵感来源**: Caves of Qud  
**配色哲学**: 大地色基调 + 古代遗迹 + 霓虹异常
