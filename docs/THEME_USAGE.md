# 主题系统使用指南

## 概述

主题系统提供了一套完整的配色方案，灵感来自 Caves of Qud 的沙漠遗迹美学。主题配置文件位于 `assets/themes/` 目录。

## 主题配置文件

主题使用 TOML 格式配置，包含以下部分：

### 1. 主题元数据
```toml
[theme]
name = "epic_desert_relic"
style = "qud_like"
description = "Earthy base, ancient relics, neon anomalies"
```

### 2. 颜色类别

#### 背景色 (`colors.background`)
- `void_ink` - 主背景色
- `basalt_blue` - 面板底色 / UI 阴影
- `obsidian_moss` - 中性暗绿灰
- `ashen_slate` - 边框 / 禁用底

#### 地形色 (`colors.earth`)
- `ruined_umber` - 岩壁 / 皮革
- `desert_bronze` - 沙砾 / 金属土色
- `silt_gold` - 沙丘高光
- `sunbaked_clay` - 浅沙 / 石板
- `bone_parchment` - 亮地表 / 骨质

#### 文字色 (`colors.text`)
- `primary` - 主文字
- `secondary` - 次文字
- `muted` - 非重点 / 描述
- `disabled` - 禁用文字

#### 语义色 (`colors.semantic`)
- `success` - 治疗 / 成功
- `info` - 信息 / 可交互
- `warning` - 警告 / 任务
- `danger` - 敌对 / 致命
- `fire` - 火焰 / 热量
- `toxic` - 毒素 / 铜绿
- `psi` - 灵能
- `rare` - 稀有 / 传送
- `tech_neon` - 高能科技 / 选中

#### 稀有度 (`colors.rarity`)
- `common` - 普通
- `uncommon` - 罕见
- `rare` - 稀有
- `epic` - 史诗
- `legendary` - 传说
- `artifact` - 神器

#### 灰度梯度 (`colors.grayscale`)
- `g00_black` 到 `g100_white` - 13级灰度渐变
- 用途: FOV 衰减 / 雾效 / 扫描线 / 禁用态 / 距离显示

## 代码中使用主题

### 1. 注入主题资源

在系统函数中，使用 `Res<Theme>` 注入主题资源：

```rust
fn my_system(theme: Res<Theme>) {
    // 使用主题颜色
}
```

### 2. 访问颜色

主题提供了分类的颜色访问：

```rust
// 背景色
let bg_color = theme.bg.void_ink;
let panel_color = theme.bg.basalt_blue;

// 地形色
let wall_color = theme.earth.ruined_umber;
let sand_color = theme.earth.sunbaked_clay;

// 文字色
let text_color = theme.text.primary;
let hint_color = theme.text.secondary;

// 语义色
let success_color = theme.semantic.success;
let danger_color = theme.semantic.danger;

// 稀有度
let epic_color = theme.rarity.epic;

// 灰度
let fog_color = theme.grayscale.g60_fog;
```

### 3. 实例: 绘制 UI 面板

```rust
fn draw_inventory_panel(
    mut canvas: ResMut<Canvas>,
    theme: Res<Theme>,
) {
    // 面板背景
    canvas.fill_rect_with_bg(
        10, 10, 30, 15,
        ' ',
        theme.text.primary,
        theme.bg.basalt_blue,
    );
    
    // 标题栏
    canvas.fill_rect_with_bg(
        11, 11, 28, 1,
        ' ',
        theme.text.primary,
        theme.bg.obsidian_moss,
    );
    canvas.set_string(12, 11, "物品栏", theme.text.primary);
    
    // 物品列表
    canvas.set_string(12, 13, "[普通] 铁剑", theme.rarity.common);
    canvas.set_string(12, 14, "[稀有] 魔法戒指", theme.rarity.rare);
    canvas.set_string(12, 15, "[传说] 神器", theme.rarity.legendary);
    
    // 边框
    for x in 10..40 {
        canvas.set_char(x, 10, '─', theme.bg.ashen_slate);
        canvas.set_char(x, 24, '─', theme.bg.ashen_slate);
    }
}
```

### 4. 实例: 绘制状态栏

```rust
fn draw_status_bar(
    mut canvas: ResMut<Canvas>,
    theme: Res<Theme>,
    health: i32,
    max_health: i32,
) {
    // 背景
    canvas.fill_background_rect(0, 0, CANVAS_WIDTH, 1, theme.bg.basalt_blue);
    
    // HP 标签
    canvas.set_string(2, 0, "HP:", theme.text.secondary);
    
    // HP 数值 - 根据百分比选择颜色
    let hp_percent = health as f32 / max_health as f32;
    let hp_color = if hp_percent > 0.5 {
        theme.semantic.success
    } else if hp_percent > 0.25 {
        theme.semantic.warning
    } else {
        theme.semantic.danger
    };
    
    canvas.set_string(
        6, 0,
        &format!("{}/{}", health, max_health),
        hp_color,
    );
}
```

### 5. 实例: FOV 渐变效果

```rust
fn apply_fog_of_war(
    mut canvas: ResMut<Canvas>,
    theme: Res<Theme>,
    distance: f32,
) {
    // 根据距离选择灰度
    let fog_color = match distance as i32 {
        0..=1 => theme.grayscale.g100_white,
        2..=3 => theme.grayscale.g90_pale,
        4..=5 => theme.grayscale.g80_silver,
        6..=7 => theme.grayscale.g70_mist,
        8..=9 => theme.grayscale.g60_fog,
        10..=11 => theme.grayscale.g50_mid,
        _ => theme.grayscale.g30_slate,
    };
    
    // 应用雾效颜色
    // ...
}
```

## 创建自定义主题

1. 复制 `assets/themes/epic_desert_relic.toml` 为新文件
2. 修改主题元数据和颜色值
3. 在 `load_theme` 函数中更新主题文件路径

```rust
let theme_path = "themes/my_custom_theme.toml";
```

## 主题结构参考

```rust
Theme {
    name: String,           // 主题名称
    style: String,          // 风格标签
    description: String,    // 描述
    
    bg: BackgroundColors,   // 背景色
    earth: EarthColors,     // 地形色
    text: TextColors,       // 文字色
    semantic: SemanticColors, // 语义色
    rarity: RarityColors,   // 稀有度
    grayscale: GrayscaleColors, // 灰度
}
```

## 最佳实践

1. **使用语义化命名**: 优先使用 `semantic` 类别的颜色表达意图
2. **保持一致性**: UI 元素使用相同类别的颜色
3. **注意对比度**: 确保文字和背景有足够的对比度
4. **灰度用于层次**: 使用灰度梯度表达深度和距离
5. **稀有度要醒目**: 高稀有度物品应该使用更亮的颜色

## 颜色预览

运行程序可以看到完整的主题演示，包括：
- 所有颜色类别的展示
- 实战场景示例
- 文字层次对比
- UI 组件样式

## 问题排查

如果主题未正确加载：

1. 检查 `assets/themes/` 目录是否存在
2. 检查 TOML 文件格式是否正确
3. 查看控制台日志中的主题加载信息
4. 确保颜色值格式正确 (格式: `"#RRGGBB"`)

## 参考

- TOML 文档: https://toml.io/
- Bevy Color 文档: https://docs.rs/bevy/latest/bevy/color/
- Caves of Qud 配色参考: https://freeholdgames.itch.io/cavesofqud
