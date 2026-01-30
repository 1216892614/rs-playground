# 背景色 API 文档

## 概述

Canvas Plugin 现在支持为每个单元格设置独立的背景色。背景色是可选的，不设置时单元格背景为透明（显示黑色画布背景）。

## 新增 API

### Canvas Resource 方法

#### `set_char_with_bg(x, y, ch, fg_color, bg_color)`
设置单个字符并指定背景色。

**参数:**
- `x`, `y`: 单元格坐标
- `ch`: 字符
- `fg_color`: 前景色（文字颜色）
- `bg_color`: 背景色

**示例:**
```rust
canvas.set_char_with_bg(
    10, 10, 
    'X', 
    Color::WHITE,           // 白色文字
    Color::srgb(1.0, 0.0, 0.0)  // 红色背景
);
```

#### `set_string_with_bg(x, y, text, fg_color, bg_color)`
设置连写字符串并指定背景色。所有字符将共享相同的背景色。

**参数:**
- `x`, `y`: 起始坐标
- `text`: 文本内容
- `fg_color`: 前景色
- `bg_color`: 背景色

**示例:**
```rust
canvas.set_string_with_bg(
    5, 5,
    "高亮文本",
    Color::WHITE,           // 白色文字
    Color::srgb(0.0, 0.5, 1.0)  // 蓝色背景
);
```

#### `set_background(x, y, bg_color)`
为现有单元格设置背景色（不改变内容）。

**参数:**
- `x`, `y`: 单元格坐标
- `bg_color`: 背景色

**示例:**
```rust
// 先设置文字
canvas.set_char(10, 10, 'A', Color::WHITE);
// 后添加背景
canvas.set_background(10, 10, Color::srgb(1.0, 0.0, 0.0));
```

#### `clear_background(x, y)`
清除单元格的背景色。

**参数:**
- `x`, `y`: 单元格坐标

**示例:**
```rust
canvas.clear_background(10, 10);
```

#### `fill_rect_with_bg(x, y, width, height, ch, fg_color, bg_color)`
填充矩形区域，同时设置字符、前景色和背景色。

**参数:**
- `x`, `y`: 矩形左上角坐标
- `width`, `height`: 矩形尺寸
- `ch`: 填充字符
- `fg_color`: 前景色
- `bg_color`: 背景色

**示例:**
```rust
// 创建一个红色方块
canvas.fill_rect_with_bg(
    10, 10, 5, 3,
    ' ',                    // 空格
    Color::WHITE,
    Color::srgb(1.0, 0.0, 0.0)
);
```

#### `fill_background_rect(x, y, width, height, bg_color)`
只填充矩形区域的背景色，不改变单元格内容。

**参数:**
- `x`, `y`: 矩形左上角坐标
- `width`, `height`: 矩形尺寸
- `bg_color`: 背景色

**示例:**
```rust
// 先绘制文字
canvas.set_string(10, 10, "按钮", Color::WHITE);
// 添加背景
canvas.fill_background_rect(9, 9, 6, 3, Color::srgb(0.3, 0.3, 0.3));
```

### CanvasCommand 组件

#### `SetCharWithBg(x, y, char, fg_color, bg_color)`
通过命令设置带背景的字符。

**示例:**
```rust
commands.spawn(CanvasCommand::SetCharWithBg(
    10, 10,
    'X',
    Color::WHITE,
    Color::RED
));
```

#### `SetStringWithBg(x, y, text, fg_color, bg_color)`
通过命令设置带背景的字符串。

**示例:**
```rust
commands.spawn(CanvasCommand::SetStringWithBg(
    5, 5,
    "高亮".to_string(),
    Color::BLACK,
    Color::YELLOW
));
```

#### `FillRectWithBg(x, y, width, height, char, fg_color, bg_color)`
通过命令填充矩形区域（带背景）。

**示例:**
```rust
commands.spawn(CanvasCommand::FillRectWithBg(
    10, 10, 20, 5,
    ' ',
    Color::WHITE,
    Color::srgb(0.2, 0.2, 0.8)
));
```

#### `FillBackgroundRect(x, y, width, height, bg_color)`
通过命令只填充背景。

**示例:**
```rust
commands.spawn(CanvasCommand::FillBackgroundRect(
    10, 10, 20, 5,
    Color::srgb(0.3, 0.3, 0.3)
));
```

## 使用示例

### 示例 1: 创建按钮

```rust
fn create_button(canvas: &mut Canvas, x: usize, y: usize, text: &str) {
    let width = text.len() + 4;
    let height = 3;
    
    // 背景
    canvas.fill_background_rect(
        x, y, width, height,
        Color::srgb(0.3, 0.3, 0.3)
    );
    
    // 边框
    for i in 0..width {
        canvas.set_char_with_bg(
            x + i, y,
            '─',
            Color::WHITE,
            Color::srgb(0.3, 0.3, 0.3)
        );
        canvas.set_char_with_bg(
            x + i, y + height - 1,
            '─',
            Color::WHITE,
            Color::srgb(0.3, 0.3, 0.3)
        );
    }
    
    // 文字
    canvas.set_string_with_bg(
        x + 2, y + 1,
        text,
        Color::WHITE,
        Color::srgb(0.3, 0.3, 0.3)
    );
}

// 使用
create_button(&mut canvas, 10, 10, "确定");
```

### 示例 2: 高亮文本

```rust
fn highlight_text(canvas: &mut Canvas, x: usize, y: usize, text: &str) {
    canvas.set_string_with_bg(
        x, y,
        &format!(" {} ", text), // 左右添加空格
        Color::BLACK,
        Color::srgb(1.0, 1.0, 0.0) // 黄色高亮
    );
}

// 使用
highlight_text(&mut canvas, 10, 5, "重要");
```

### 示例 3: 进度条

```rust
fn draw_progress_bar_with_bg(
    canvas: &mut Canvas,
    x: usize,
    y: usize,
    width: usize,
    progress: f32, // 0.0 to 1.0
) {
    let filled = (width as f32 * progress) as usize;
    
    // 已完成部分（绿色背景）
    canvas.fill_rect_with_bg(
        x, y, filled, 1,
        ' ',
        Color::WHITE,
        Color::srgb(0.0, 1.0, 0.0)
    );
    
    // 未完成部分（灰色背景）
    canvas.fill_rect_with_bg(
        x + filled, y, width - filled, 1,
        ' ',
        Color::WHITE,
        Color::srgb(0.3, 0.3, 0.3)
    );
}

// 使用
draw_progress_bar_with_bg(&mut canvas, 10, 10, 50, 0.65);
```

### 示例 4: 彩色标签

```rust
fn create_tag(canvas: &mut Canvas, x: usize, y: usize, text: &str, color: Color) {
    canvas.set_string_with_bg(
        x, y,
        &format!(" {} ", text),
        Color::WHITE,
        color
    );
}

// 使用
create_tag(&mut canvas, 5, 5, "新", Color::srgb(1.0, 0.0, 0.0));
create_tag(&mut canvas, 12, 5, "热门", Color::srgb(1.0, 0.5, 0.0));
create_tag(&mut canvas, 22, 5, "推荐", Color::srgb(0.0, 0.8, 0.0));
```

### 示例 5: 菜单选项

```rust
fn draw_menu_item(
    canvas: &mut Canvas,
    x: usize,
    y: usize,
    text: &str,
    selected: bool,
) {
    let (fg, bg) = if selected {
        (Color::BLACK, Color::WHITE)
    } else {
        (Color::WHITE, Color::srgb(0.2, 0.2, 0.2))
    };
    
    canvas.fill_rect_with_bg(
        x, y, text.len() + 2, 1,
        ' ',
        fg,
        bg
    );
    
    canvas.set_string_with_bg(
        x + 1, y,
        text,
        fg,
        bg
    );
}

// 使用
draw_menu_item(&mut canvas, 5, 10, "选项 1", true);   // 选中
draw_menu_item(&mut canvas, 5, 11, "选项 2", false);  // 未选中
draw_menu_item(&mut canvas, 5, 12, "选项 3", false);
```

### 示例 6: 代码高亮

```rust
fn draw_code_line(canvas: &mut Canvas, y: usize, line_num: usize, code: &str) {
    // 行号背景
    canvas.fill_rect_with_bg(
        0, y, 5, 1,
        ' ',
        Color::srgb(0.6, 0.6, 0.6),
        Color::srgb(0.2, 0.2, 0.2)
    );
    
    // 行号
    canvas.set_string_with_bg(
        1, y,
        &format!("{:3}", line_num),
        Color::srgb(0.6, 0.6, 0.6),
        Color::srgb(0.2, 0.2, 0.2)
    );
    
    // 代码
    canvas.set_string(7, y, code, Color::srgb(0.8, 0.8, 1.0));
}

// 使用
draw_code_line(&mut canvas, 10, 1, "fn main() {");
draw_code_line(&mut canvas, 11, 2, "    println!(\"Hello\");");
draw_code_line(&mut canvas, 12, 3, "}");
```

## 渲染细节

### 渲染顺序

Canvas 使用两遍渲染：

1. **第一遍**: 渲染所有背景（Z=0）
2. **第二遍**: 渲染所有文字内容（Z=1）

这确保文字始终在背景之上。

### 性能考虑

- 背景使用 `Sprite` 组件渲染，每个有背景的单元格会创建一个额外的实体
- 如果大量使用背景色，可能会增加实体数量
- Dirty flag 优化仍然有效，只在画布改变时重新渲染

### 透明度支持

背景色支持完整的 RGBA 颜色，可以设置透明度：

```rust
canvas.set_char_with_bg(
    10, 10,
    'X',
    Color::WHITE,
    Color::srgba(1.0, 0.0, 0.0, 0.5) // 半透明红色
);
```

## 最佳实践

1. **一致性**: 为相同类型的元素使用相同的背景色
2. **对比度**: 确保前景色和背景色有足够对比度，便于阅读
3. **性能**: 避免在每帧都改变大量单元格的背景色
4. **语义化**: 使用背景色传达语义（如：红色=错误，绿色=成功）

## 常用配色方案

### 终端风格
```rust
const BG_BLACK: Color = Color::srgb(0.0, 0.0, 0.0);
const BG_RED: Color = Color::srgb(0.8, 0.0, 0.0);
const BG_GREEN: Color = Color::srgb(0.0, 0.8, 0.0);
const BG_YELLOW: Color = Color::srgb(0.8, 0.8, 0.0);
const BG_BLUE: Color = Color::srgb(0.0, 0.0, 0.8);
const BG_MAGENTA: Color = Color::srgb(0.8, 0.0, 0.8);
const BG_CYAN: Color = Color::srgb(0.0, 0.8, 0.8);
const BG_WHITE: Color = Color::srgb(0.8, 0.8, 0.8);
```

### 状态颜色
```rust
const STATUS_SUCCESS: Color = Color::srgb(0.2, 0.8, 0.2);
const STATUS_WARNING: Color = Color::srgb(1.0, 0.8, 0.0);
const STATUS_ERROR: Color = Color::srgb(0.9, 0.2, 0.2);
const STATUS_INFO: Color = Color::srgb(0.2, 0.6, 1.0);
```

### UI 元素
```rust
const UI_BUTTON: Color = Color::srgb(0.3, 0.3, 0.3);
const UI_BUTTON_HOVER: Color = Color::srgb(0.4, 0.4, 0.4);
const UI_BUTTON_ACTIVE: Color = Color::srgb(0.5, 0.5, 0.5);
const UI_SELECTED: Color = Color::srgb(0.2, 0.4, 0.8);
const UI_DISABLED: Color = Color::srgb(0.15, 0.15, 0.15);
```
