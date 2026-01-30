# Changelog

## [0.6.0] - 2026-01-30

### 🎉 Unicode 字符宽度正确处理

#### 核心改进
- ✅ **正确处理全角字符**：中文、日文、韩文字符自动占2格
- ✅ **使用 unicode-width crate**：标准化的字符宽度检测
- ✅ **占位符机制**：`Continuation` 占位符标记全角字符的第二个格子
- ✅ **API 语义更新**：明确区分"独立放置"和"连写"模式

#### API 变化

**旧行为（自动连写）：**
```rust
canvas.set_string(0, 0, "Hello世界", color);
// 问题：全角字符和英文混在一起，显示错乱
```

**新行为（正确处理宽度）：**
```rust
// 默认不连写，每个字符独立放置，全角字符自动占2格
canvas.set_string(0, 0, "Hello世界", color);
// ✅ 正确：'世' 占格子 [5,6]，'界' 占格子 [7,8]

// 明确连写（用于 ASCII 装饰）
canvas.set_string_continuous(0, 1, "═══════", color);
// ✅ 紧凑：所有字符合并渲染
```

#### 新增 API

```rust
// 连写模式 API
canvas.set_string_continuous(x, y, text, color);
canvas.set_string_continuous_with_bg(x, y, text, fg, bg);
canvas.fill_rect_continuous(x, y, w, h, ch, color);
canvas.fill_rect_continuous_with_bg(x, y, w, h, ch, fg, bg);
```

#### 字符宽度支持

| 字符类型 | 示例 | 宽度 | 占用格子 |
|---------|------|------|---------|
| ASCII 字母 | `A-Z`, `a-z` | 1 | 1 格 |
| ASCII 数字 | `0-9` | 1 | 1 格 |
| 框线字符 | `─│┌┐└┘` | 1 | 1 格 |
| 方块字符 | `█▓▒░` | 1 | 1 格 |
| 中文字符 | `你好世界` | 2 | 2 格 |
| 日文字符 | `こんにちは` | 2 | 2 格 |
| 韩文字符 | `안녕하세요` | 2 | 2 格 |
| Emoji | `😀🎉✨` | 2 | 2 格 |

#### 内部实现

```rust
enum CellContent {
    Empty,           // 空格子
    Char(char),     // 实际字符
    Continuation,   // 占位符（全角字符的第二格）
    Svg(String),    // SVG（预留）
}

struct Cell {
    content: CellContent,
    span: usize,     // 0=单字符, >0=连写字符数
    color: Color,
    background_color: Option<Color>,
}
```

### 🔧 字体加载优化

#### 自动重新渲染
- ✅ **字体加载完成时自动触发重新渲染**
- ✅ **解决"初始显示方块"问题**：程序启动后字体异步加载，完成后画布自动刷新

```
INFO: ✓ Noto Sans CJK loaded!
INFO: ✓ Nerd Font loaded!
INFO: === All fonts loaded successfully! ===
INFO: Triggering canvas re-render with loaded fonts...
```

#### 渐进式加载
1. 启动时使用默认字体（显示基本字符）
2. 字体下载中（10-30秒）
3. 字体加载完成 → 自动重新渲染
4. 所有字符正确显示

### 📦 依赖

```toml
[dependencies]
unicode-width = "0.2"  # 新增：字符宽度检测
```

### 📖 文档

- 📝 新增：`docs/UNICODE_WIDTH_HANDLING.md` - Unicode 字符宽度处理详解
- 📝 更新：API 文档反映新的行为

### 🔄 迁移指南

#### 如果你的代码包含中日韩字符

**之前：**
```rust
canvas.set_string(0, 0, "文件名.txt", color);
// 显示可能错乱
```

**现在：**
```rust
canvas.set_string(0, 0, "文件名.txt", color);
// ✅ 自动正确！无需修改代码
```

#### 如果你用 ASCII 字符做装饰

**之前：**
```rust
canvas.set_string(0, 0, "═══════════", color);
// 显示紧凑
```

**现在（可选优化）：**
```rust
// 选项 1：继续使用普通 API（每个字符独立）
canvas.set_string(0, 0, "═══════════", color);

// 选项 2：明确使用连写模式（更紧凑）
canvas.set_string_continuous(0, 0, "═══════════", color);
```

### ⚠️ 破坏性变化

1. **`set_string()` 默认行为改变**
   - 之前：自动连写所有字符
   - 现在：每个字符独立放置，全角字符占2格
   - **影响**：混合中英文内容现在会正确对齐，但纯 ASCII 装饰可能需要更多空间

2. **`CellContent` 枚举新增变体**
   - 新增：`Continuation` 占位符
   - **影响**：如果你直接访问内部结构，需要处理新的枚举变体

### 🎯 使用场景

#### ✅ 适合新 API
- 显示混合语言文本（中英日韩混合）
- 对齐表格数据
- 用户输入内容
- 国际化界面

#### ⚙️ 可选使用连写模式
- ASCII 艺术
- 框线装饰
- 纯英文标题
- 进度条

---

## [0.5.0] - 2026-01-30

### 重大重构 - 模块化架构 🏗️
- ✅ Canvas Plugin 完全模块化到 `src/canvas.rs`
- ✅ 所有内部实现细节封装隐藏
- ✅ 只对外暴露干净的绘制 API
- ✅ 主程序 (`main.rs`) 精简
- ✅ 关注点完全分离

### Web 字体加载系统 🌐
- ✅ Noto Sans CJK - 主字体
- ✅ Nerd Font - 图标字体
- ✅ 智能字体选择（根据字符类型）
- ✅ 渐进式加载（字体下载时临时使用默认字体）
- ✅ 多级回退机制

### 字体特性
- 完整 Unicode 支持（CJK、框线字符、各种符号）
- 8000+ Nerd Font 图标
- 自动缓存到 `.web-asset-cache`
- 首次下载 10-30 秒，后续启动 <1 秒

---

## [0.4.0] - 2026-01-30

### 背景色支持 🎨
- ✅ 每个单元格可独立设置背景色
- ✅ 两遍渲染：先背景（Z=0），后文字（Z=1）
- ✅ 新增 API：`set_char_with_bg`, `set_string_with_bg`, `fill_rect_with_bg`

---

## [0.3.0] - 2026-01-30

### 像素对齐优化 📐
- ✅ 更精确的缩放阈值（floor + 98% 安全边距）
- ✅ 画布永远不被窗口遮挡
- ✅ 支持 2x, 1.5x, 1x, 0.75x, 0.5x, 0.25x 等对齐缩放

---

## [0.2.0] - 2026-01-30

### Plugin 架构 🔌
- ✅ Canvas 系统模块化为 Bevy Plugin
- ✅ 对外暴露清除和矩形替换 API
- ✅ 使用 Web Assets 加载字体

---

## [0.1.0] - 2026-01-30

### 初始版本 🎉
- ✅ 96×54 字符画布
- ✅ 固定等距网格
- ✅ 字符渲染
- ✅ 自动缩放和像素对齐
- ✅ 黑边填充（Contain 模式）
