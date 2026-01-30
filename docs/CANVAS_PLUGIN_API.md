# Canvas Plugin API 文档

## 概述

Canvas Plugin 提供了一个 96×54 的字符画布系统，支持字符、连写文本和 SVG 渲染。画布会自动缩放以适应窗口大小，并保持像素对齐。

## 功能特性

- ✅ 96×54 固定网格大小
- ✅ 等距单元格布局
- ✅ 支持单字符和连写文本
- ✅ 自动像素对齐和缩放
- ✅ 黑边填充（contain 模式）
- ✅ Web Assets 加载字体（Noto Sans + Nerd Fonts）
- ✅ 命令式 API（通过组件）

## 快速开始

### 1. 添加 Plugin

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CanvasPlugin)
        .run();
}
```

### 2. 直接访问 Canvas Resource

```rust
fn my_system(mut canvas: ResMut<Canvas>) {
    // 设置单个字符
    canvas.set_char(0, 0, 'A', Color::WHITE);
    
    // 设置连写字符串
    canvas.set_string(0, 1, "Hello, World!", Color::GREEN);
    
    // 清除整个画布
    canvas.clear();
    
    // 清除矩形区域
    canvas.clear_rect(10, 10, 20, 5);
    
    // 填充矩形区域
    canvas.fill_rect(10, 10, 20, 5, '█', Color::RED);
}
```

### 3. 使用命令组件（推荐）

```rust
fn spawn_commands(mut commands: Commands) {
    // 清除整个画布
    commands.spawn(CanvasCommand::Clear);
    
    // 清除矩形区域
    commands.spawn(CanvasCommand::ClearRect(10, 10, 20, 5));
    
    // 填充矩形区域
    commands.spawn(CanvasCommand::FillRect(
        10, 10, 20, 5, 
        '█', 
        Color::srgb(1.0, 0.0, 0.0)
    ));
    
    // 设置单个字符
    commands.spawn(CanvasCommand::SetChar(
        5, 5, 
        'X', 
        Color::WHITE
    ));
    
    // 设置字符串
    commands.spawn(CanvasCommand::SetString(
        0, 0, 
        "Hello".to_string(), 
        Color::GREEN
    ));
}
```

## API 参考

### Canvas Resource 方法

#### `set_char(x: usize, y: usize, ch: char, color: Color)`
在指定位置设置单个字符。

**参数:**
- `x`, `y`: 单元格坐标（0-based）
- `ch`: 要显示的字符
- `color`: 文本颜色

**示例:**
```rust
canvas.set_char(10, 5, '█', Color::RED);
```

#### `set_string(x: usize, y: usize, text: &str, color: Color)`
设置连写字符串。字符串会跨越多个单元格，但保持居中对齐。

**参数:**
- `x`, `y`: 起始单元格坐标
- `text`: 要显示的文本
- `color`: 文本颜色

**示例:**
```rust
canvas.set_string(2, 3, "Hello, 世界!", Color::GREEN);
```

#### `set_svg(x: usize, y: usize, svg_id: &str, color: Color)`
设置 SVG 图标（预留功能）。

**参数:**
- `x`, `y`: 单元格坐标
- `svg_id`: SVG 资源标识符
- `color`: 图标颜色

#### `clear()`
清除整个画布，将所有单元格重置为空。

**示例:**
```rust
canvas.clear();
```

#### `clear_rect(x: usize, y: usize, width: usize, height: usize)`
清除指定的矩形区域。

**参数:**
- `x`, `y`: 矩形左上角坐标
- `width`, `height`: 矩形尺寸

**示例:**
```rust
// 清除从 (10, 10) 开始的 20×5 区域
canvas.clear_rect(10, 10, 20, 5);
```

#### `fill_rect(x: usize, y: usize, width: usize, height: usize, ch: char, color: Color)`
用指定字符填充矩形区域。

**参数:**
- `x`, `y`: 矩形左上角坐标
- `width`, `height`: 矩形尺寸
- `ch`: 填充字符
- `color`: 填充颜色

**示例:**
```rust
// 绘制一个红色方块
canvas.fill_rect(10, 10, 5, 5, '█', Color::RED);
```

### CanvasCommand 组件

命令组件提供了声明式的画布操作方式。生成命令实体后，系统会自动处理并执行命令。

#### 可用命令

- `CanvasCommand::Clear` - 清除整个画布
- `CanvasCommand::ClearRect(x, y, width, height)` - 清除矩形区域
- `CanvasCommand::FillRect(x, y, width, height, char, color)` - 填充矩形区域
- `CanvasCommand::SetChar(x, y, char, color)` - 设置单个字符
- `CanvasCommand::SetString(x, y, text, color)` - 设置字符串
- `CanvasCommand::SetSvg(x, y, svg_id, color)` - 设置 SVG

**优点:**
- ECS 友好
- 支持延迟执行
- 可以与其他系统组合
- 自动清理

## 字体支持

Canvas Plugin 自动加载以下字体来支持更广泛的字符集：

### Noto Sans
- 用于 CJK（中日韩）字符
- 自动检测 Unicode 范围 > 0x4E00 的字符

### Nerd Fonts (JetBrains Mono)
- 用于图标和特殊符号
- 自动检测私有使用区字符（0xE000-0xF8FF）

### 字符检测逻辑

```rust
// 示例：各种字符类型
canvas.set_string(0, 0, "ABC", Color::WHITE);        // 默认字体
canvas.set_string(0, 1, "你好世界", Color::WHITE);   // Noto Sans
canvas.set_string(0, 2, " ", Color::WHITE);       // Nerd Font 图标
```

## 像素对齐

Canvas 会自动计算最佳缩放比例，确保：

1. **Contain 模式**: 画布完全适应窗口，不裁剪
2. **像素对齐**: 缩放比例对齐到整数或半整数倍，避免模糊
3. **黑边填充**: 多余空间填充黑色背景

### 缩放规则

- 缩放 ≥ 2.0x: 取整数倍（2x, 3x, 4x...）
- 1.0x ≤ 缩放 < 2.0x: 取 0.5 步进（1.0x, 1.5x）
- 缩放 < 1.0x: 取 0.25 步进（0.25x, 0.5x, 0.75x）

## 常见用例

### 绘制边框

```rust
fn draw_border(mut canvas: ResMut<Canvas>) {
    // 水平边
    for x in 0..96 {
        canvas.set_char(x, 0, '─', Color::GRAY);
        canvas.set_char(x, 53, '─', Color::GRAY);
    }
    // 垂直边
    for y in 0..54 {
        canvas.set_char(0, y, '│', Color::GRAY);
        canvas.set_char(95, y, '│', Color::GRAY);
    }
    // 四个角
    canvas.set_char(0, 0, '┌', Color::GRAY);
    canvas.set_char(95, 0, '┐', Color::GRAY);
    canvas.set_char(0, 53, '└', Color::GRAY);
    canvas.set_char(95, 53, '┘', Color::GRAY);
}
```

### 动画文本

```rust
fn animate_text(
    time: Res<Time>,
    mut canvas: ResMut<Canvas>,
) {
    let offset = (time.elapsed_secs() * 10.0) as usize % 80;
    canvas.clear_rect(0, 25, 96, 1);
    canvas.set_string(offset, 25, ">>> Moving Text <<<", Color::YELLOW);
}
```

### 进度条

```rust
fn draw_progress_bar(
    mut canvas: ResMut<Canvas>,
    progress: f32, // 0.0 to 1.0
) {
    let bar_width = 50;
    let x = 20;
    let y = 20;
    
    // 清除区域
    canvas.clear_rect(x, y, bar_width + 2, 3);
    
    // 绘制边框
    canvas.set_char(x, y, '[', Color::WHITE);
    canvas.set_char(x + bar_width + 1, y, ']', Color::WHITE);
    
    // 填充进度
    let filled = (bar_width as f32 * progress) as usize;
    for i in 0..filled {
        canvas.set_char(x + 1 + i, y, '█', Color::GREEN);
    }
    for i in filled..bar_width {
        canvas.set_char(x + 1 + i, y, '░', Color::DARK_GRAY);
    }
}
```

## 性能优化

### Dirty Flag
Canvas 使用 dirty flag 来避免不必要的重新渲染。只有在画布内容改变时才会重新生成实体。

### 命令批处理
使用命令组件可以批量执行多个操作，减少系统调用次数。

```rust
fn batch_commands(mut commands: Commands) {
    for y in 0..10 {
        commands.spawn(CanvasCommand::SetString(
            0, y,
            format!("Line {}", y),
            Color::WHITE
        ));
    }
}
```

## 限制和注意事项

1. **固定尺寸**: 画布始终是 96×54，不可更改
2. **连写限制**: 连写文本不会自动换行，超出边界的部分会被裁剪
3. **字体加载**: 字体从 CDN 加载，首次启动需要网络连接
4. **SVG 支持**: SVG 功能目前为占位符，需要额外实现

## 示例项目

完整示例请查看 `src/main.rs` 中的 `demo_setup` 函数。
