# Unicode 字符宽度处理

## 概述

Canvas Plugin 现在正确处理 Unicode 字符宽度，使用 `unicode-width` crate 来确定每个字符占用的格子数。

## 字符宽度分类

### 半角字符（宽度 = 1）
- 英文字母：`a-z`, `A-Z`
- 数字：`0-9`
- 标点符号：`,.;:!?`
- ASCII 符号：`+-*/=<>`
- 框线字符：`─│┌┐└┘├┤┬┴┼`
- 方块字符：`█▓▒░`

### 全角字符（宽度 = 2）
- 中文：`你好世界`
- 日文：`こんにちは`
- 韩文：`안녕하세요`
- 全角标点：`，。！？`
- 全角数字：`０１２３`
- Emoji：`😀🎉✨`

## API 行为变化

### 1. `set_char()` - 自动处理宽度

```rust
// 半角字符：占1格
canvas.set_char(0, 0, 'A', Color::WHITE);
// 格子 [0] = 'A'

// 全角字符：占2格
canvas.set_char(0, 1, '你', Color::WHITE);
// 格子 [0] = '你'
// 格子 [1] = Continuation（占位符）
```

### 2. `set_string()` - 默认不连写，正确处理宽度

```rust
// 混合字符串：每个字符独立放置，正确占位
canvas.set_string(0, 0, "Hello世界", Color::WHITE);

// 渲染结果：
// 格子 0: 'H'
// 格子 1: 'e'
// 格子 2: 'l'
// 格子 3: 'l'
// 格子 4: 'o'
// 格子 5: '世' (全角)
// 格子 6: Continuation (占位符)
// 格子 7: '界' (全角)
// 格子 8: Continuation (占位符)
```

### 3. `set_string_continuous()` - 明确连写（不考虑宽度）

```rust
// 强制连写：所有字符合并为一个文本实体
canvas.set_string_continuous(0, 0, "ABCDE", Color::WHITE);

// 渲染结果：
// 整个字符串 "ABCDE" 居中渲染在 5 个格子的范围内
// 用于创建紧凑的文本效果
```

## 内部实现

### Cell 结构

```rust
enum CellContent {
    Empty,                  // 空格子
    Char(char),            // 实际字符
    Continuation,          // 占位符（全角字符的第二格）
    Svg(String),          // SVG（预留）
}

struct Cell {
    content: CellContent,
    span: usize,           // 0=单字符, >0=连写字符数
    color: Color,
    background_color: Option<Color>,
}
```

### 宽度检测

```rust
use unicode_width::UnicodeWidthChar;

let width = ch.width().unwrap_or(1);

match width {
    1 => {
        // 半角字符：只占1格
        self.cells[y][x] = Cell { content: Char(ch), .. };
    }
    2 => {
        // 全角字符：占2格
        self.cells[y][x] = Cell { content: Char(ch), .. };
        self.cells[y][x+1] = Cell { content: Continuation, .. };
    }
    _ => { /* 处理其他宽度 */ }
}
```

### 渲染处理

```rust
// 渲染时跳过占位符
match cell.content {
    CellContent::Empty => { /* 跳过 */ }
    CellContent::Continuation => { /* 跳过占位符 */ }
    CellContent::Char(ch) => {
        // 渲染实际字符
        if cell.span > 0 {
            // 连写模式：收集后续字符
        } else {
            // 单字符模式：直接渲染
        }
    }
}
```

## 使用示例

### 示例 1：混合文本

```rust
fn setup(mut canvas: ResMut<Canvas>) {
    // 每个字符独立放置，全角字符自动占2格
    canvas.set_string(0, 0, "文件: file.txt", Color::WHITE);
    
    // 预期格子占用：
    // [0,1] 文 (全角)
    // [2,3] 件 (全角)
    // [4]   :  (半角)
    // [5]   空格 (半角)
    // [6-13] file.txt (各半角)
}
```

### 示例 2：对齐表格

```rust
fn draw_table(canvas: &mut Canvas) {
    // 表头（正确对齐）
    canvas.set_string(0, 0, "名称", Color::WHITE);     // 占4格
    canvas.set_string(5, 0, "数量", Color::WHITE);     // 占4格
    canvas.set_string(10, 0, "状态", Color::WHITE);    // 占4格
    
    // 数据行
    canvas.set_string(0, 1, "道具", Color::GRAY);
    canvas.set_string(5, 1, "99", Color::GRAY);
    canvas.set_string(10, 1, "正常", Color::GREEN);
}
```

### 示例 3：框线装饰（连写模式）

```rust
fn draw_box_with_text(canvas: &mut Canvas) {
    // 边框使用连写（半角字符，更紧凑）
    canvas.set_string_continuous(0, 0, "═══════════", Color::GRAY);
    
    // 内容使用普通模式（正确处理全角）
    canvas.set_string(1, 1, "标题文本", Color::WHITE);
    
    // 底部边框
    canvas.set_string_continuous(0, 2, "═══════════", Color::GRAY);
}
```

## API 对比表

| API | 连写行为 | 宽度处理 | 适用场景 |
|-----|---------|---------|---------|
| `set_char()` | 单字符 | ✅ 自动 | 单个字符放置 |
| `set_string()` | ❌ 不连写 | ✅ 自动 | 文本、混合内容 |
| `set_string_continuous()` | ✅ 连写 | ❌ 忽略 | ASCII 装饰、标题 |
| `fill_rect()` | 单字符 | ✅ 自动 | 填充区域 |
| `fill_rect_continuous()` | ✅ 连写 | ❌ 忽略 | ASCII 图案 |

## 迁移指南

### 旧代码
```rust
// 旧 API（自动连写，不考虑宽度）
canvas.set_string(0, 0, "Hello世界", Color::WHITE);
// 问题：全角字符会和后续字符重叠
```

### 新代码
```rust
// 新 API（默认不连写，正确处理宽度）
canvas.set_string(0, 0, "Hello世界", Color::WHITE);
// ✅ 正确：全角字符占2格，不会重叠

// 如果需要紧凑的ASCII装饰，使用连写模式
canvas.set_string_continuous(0, 1, "─────", Color::GRAY);
```

## 常见问题

### Q: 为什么我的中文字符旁边有空白？
A: 这是正确的行为！中文字符占2个格子宽度，第二个格子是占位符，确保不会和后续字符重叠。

### Q: 如何让文本更紧凑？
A: 如果你确定只使用半角字符（ASCII），可以使用 `set_string_continuous()` 来连写。但不要对混合内容使用连写，否则会导致显示错误。

### Q: Emoji 会正确显示吗？
A: 是的！Emoji 也是全角字符（宽度=2），会自动占2格并正确渲染。

### Q: 旧代码需要改动吗？
A: 视情况而定：
- 如果你的文本包含中日韩字符：新 API 会自动修复显示问题 ✅
- 如果你只用 ASCII 做装饰：可能需要改用 `set_string_continuous()` 以保持紧凑效果

## 技术细节

### 依赖
```toml
[dependencies]
unicode-width = "0.2"
```

### Unicode East Asian Width
- W (Wide): 宽度 = 2
- F (Fullwidth): 宽度 = 2
- N (Neutral): 宽度 = 1
- Na (Narrow): 宽度 = 1
- H (Halfwidth): 宽度 = 1
- A (Ambiguous): 宽度 = 1（默认）

参考：https://www.unicode.org/reports/tr11/
