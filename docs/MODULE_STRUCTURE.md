# Canvas Plugin 模块化结构

## 架构概览

Canvas Plugin 现在是一个完全独立的模块，所有内部实现都被封装，只对外暴露必要的绘制 API。

## 文件结构

```
src/
├── main.rs          # 主程序，只包含演示代码
└── canvas.rs        # Canvas Plugin 模块（完全封装）
```

## 模块分离

### `src/canvas.rs` - Canvas Plugin（内部）

**封装的内容：**
- ✅ 所有常量（CANVAS_WIDTH, CANVAS_HEIGHT, BASE_CELL_SIZE 等）
- ✅ 内部数据结构（CellContent, Cell）
- ✅ 字体管理（CanvasFonts）
- ✅ 渲染系统（render_canvas, handle_window_resize）
- ✅ 命令处理（process_canvas_commands）
- ✅ 像素对齐算法
- ✅ 字体加载和检测

**对外暴露：**
- ✅ `CanvasPlugin` - Plugin 结构体
- ✅ `Canvas` - Resource，提供所有绘制 API
- ✅ `CanvasCommand` - 命令组件（可选，已标记为内部使用）

### `src/main.rs` - 主程序

**简洁的主程序：**
```rust
use bevy::prelude::*;

mod canvas;
use canvas::Canvas;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(canvas::CanvasPlugin)
        .add_systems(Startup, demo_setup)
        .run();
}
```

**只需关注：**
- 应用程序配置
- 使用 Canvas API 绘制内容
- 无需了解内部实现细节

## 公共 API

### Canvas Resource 方法

所有这些方法都是公开的，可以在任何系统中使用：

```rust
fn my_system(mut canvas: ResMut<Canvas>) {
    // 基础绘制
    canvas.set_char(x, y, ch, color);
    canvas.set_string(x, y, text, color);
    
    // 带背景色
    canvas.set_char_with_bg(x, y, ch, fg_color, bg_color);
    canvas.set_string_with_bg(x, y, text, fg_color, bg_color);
    
    // 区域操作
    canvas.clear();
    canvas.clear_rect(x, y, width, height);
    canvas.fill_rect(x, y, width, height, ch, color);
    canvas.fill_rect_with_bg(x, y, width, height, ch, fg_color, bg_color);
    canvas.fill_background_rect(x, y, width, height, bg_color);
    
    // 单独设置背景
    canvas.set_background(x, y, bg_color);
    canvas.clear_background(x, y);
}
```

## 使用示例

### 最简单的使用

```rust
use bevy::prelude::*;

mod canvas;
use canvas::{Canvas, CanvasPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CanvasPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut canvas: ResMut<Canvas>) {
    canvas.set_string(0, 0, "Hello!", Color::WHITE);
}
```

### 完整的游戏示例

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CanvasPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            update_game_state,
            render_game_ui,
            handle_input,
        ))
        .run();
}

fn render_game_ui(
    mut canvas: ResMut<Canvas>,
    game_state: Res<GameState>,
) {
    // 清除上一帧
    canvas.clear_rect(0, 0, 96, 54);
    
    // 渲染游戏界面
    canvas.set_string(2, 2, &game_state.title, Color::YELLOW);
    
    // 渲染角色状态
    canvas.set_string(2, 5, &format!("HP: {}/{}",
        game_state.hp, game_state.max_hp), Color::GREEN);
    
    // 绘制地图
    for (y, row) in game_state.map.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            canvas.set_char(x + 5, y + 10, tile.char, tile.color);
        }
    }
}
```

## 内部实现隐藏

用户代码**不需要**知道：
- ❌ 单元格如何存储（Cell 结构）
- ❌ 字体如何加载（CanvasFonts）
- ❌ 渲染如何工作（render_canvas 系统）
- ❌ 缩放算法细节（像素对齐逻辑）
- ❌ Dirty flag 优化机制

## 优点

### 1. **关注点分离**
- 主程序只关心"绘制什么"
- Canvas 模块负责"如何渲染"

### 2. **易于维护**
- Canvas 内部实现可以自由优化
- 不影响使用 API 的代码

### 3. **清晰的接口**
- 所有公共方法都有文档
- API 简单直观

### 4. **可扩展性**
- 可以轻松添加新的绘制方法
- 可以替换内部渲染实现

## 运行日志

从日志可以看到模块化成功：

```
INFO rs_playground::canvas: === Loading font ===
INFO rs_playground::canvas: Initial canvas scale: 0.75, cell_size: 12
INFO rs_playground::canvas: Loading main font from: fonts/font.ttf
INFO rs_playground::canvas: Window size: 1280x720
INFO rs_playground::canvas: Canvas size: 1152x648
INFO rs_playground::canvas: ✓ Font loaded successfully!
```

- ✅ 所有日志来自 `canvas` 模块
- ✅ 字体加载成功
- ✅ 画布正确缩放
- ✅ 主程序完全不涉及内部细节

## 下一步扩展

### 可能的改进

1. **添加更多绘制 API**
```rust
// 绘制线条
canvas.draw_line(x1, y1, x2, y2, ch, color);

// 绘制圆
canvas.draw_circle(cx, cy, radius, ch, color);

// 绘制文本框
canvas.draw_box(x, y, width, height, title, style);
```

2. **动画支持**
```rust
canvas.animate_char(x, y, frames, duration);
```

3. **层系统**
```rust
canvas.push_layer();
canvas.pop_layer();
canvas.merge_layers();
```

4. **事件系统**
```rust
// 点击检测
if canvas.is_clicked(x, y) {
    // ...
}
```

## 总结

重构后的代码结构：
- 📦 **模块化**: Canvas 完全独立
- 🔒 **封装**: 内部实现隐藏
- 🎯 **简洁**: API 清晰易用
- ⚡ **性能**: 优化对用户透明
- 📝 **可维护**: 易于扩展和修改
