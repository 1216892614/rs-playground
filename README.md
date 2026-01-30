# Canvas Plugin - 96×54 字符画布系统

一个基于 Bevy 的高性能字符画布渲染引擎，支持完整 Unicode 字符集、连写文本和背景色。

## ✨ 特性

- **固定网格**: 96×54 单元格，等距布局
- **Unicode 字符宽度**: 正确处理全角字符（中日韩、Emoji 占2格）
- **字符渲染**: 单字符、独立文本、连写模式、SVG（预留）
- **背景色支持**: 每个单元格可独立设置背景色
- **像素对齐**: 自动缩放和像素完美对齐
- **黑边填充**: Contain 模式，自适应窗口
- **Web 字体**: 自动从 CDN 加载 Noto Sans CJK 和 Nerd Font
- **模块化**: 完全封装的 Plugin 架构

## 🚀 快速开始

### 安装

```bash
git clone <repo>
cd rs-playground
cargo run --release
```

### 最简示例

```rust
use bevy::prelude::*;
mod canvas;
use canvas::Canvas;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(canvas::CanvasPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut canvas: ResMut<Canvas>) {
    canvas.set_string(10, 10, "Hello, 世界!", Color::WHITE);
}
```

## 📖 API 文档

### 基础绘制

```rust
// 设置单个字符（自动处理全角字符）
canvas.set_char(x, y, '█', Color::RED);
canvas.set_char(x, y, '你', Color::WHITE);  // 全角字符自动占2格

// 设置字符串（默认不连写，正确处理全角字符）
canvas.set_string(x, y, "Hello世界", Color::WHITE);
// 结果：'世' 占2格，'界' 占2格，英文各占1格

// 设置连写字符串（明确连写，用于紧凑的 ASCII 装饰）
canvas.set_string_continuous(x, y, "═══════", Color::GRAY);

// 清除画布
canvas.clear();

// 清除矩形区域
canvas.clear_rect(x, y, width, height);

// 填充矩形（每个字符独立）
canvas.fill_rect(x, y, width, height, '█', Color::BLUE);

// 填充矩形（连写模式，用于 ASCII 图案）
canvas.fill_rect_continuous(x, y, width, height, '═', Color::GRAY);
```

### 背景色支持

```rust
// 设置字符+背景
canvas.set_char_with_bg(x, y, 'X', Color::WHITE, Color::RED);

// 设置字符串+背景
canvas.set_string_with_bg(
    x, y,
    "高亮文本",
    Color::BLACK,
    Color::YELLOW
);

// 填充矩形+背景
canvas.fill_rect_with_bg(
    x, y, width, height,
    ' ',
    Color::WHITE,
    Color::srgb(0.3, 0.3, 0.3)
);

// 只填充背景（不改变内容）
canvas.fill_background_rect(x, y, width, height, Color::BLUE);
```

## 🎨 字体系统

### 自动加载

Canvas Plugin 自动从 CDN 加载两个字体：

1. **Noto Sans CJK** (主字体)
   - 支持中文、日文、韩文
   - 支持所有 Unicode 框线和符号字符
   - 约 16 MB

2. **Nerd Font** (图标字体)
   - 支持 8000+ 开发者图标
   - 约 3-5 MB

### 加载时间

- **首次启动**: 10-30 秒（下载字体）
- **后续启动**: <1 秒（从缓存加载）
- **缓存位置**: `.web-asset-cache/`

### 字符支持

```rust
// 框线字符（半角，各占1格）
canvas.set_string(0, 0, "┌──┬──┐", Color::WHITE);

// 中文（全角，各占2格）
canvas.set_string(0, 1, "你好世界", Color::GREEN);
// '你' 占 [0,1]，'好' 占 [2,3]，'世' 占 [4,5]，'界' 占 [6,7]

// 方块字符（半角，各占1格）
canvas.set_string(0, 2, "█▓▒░", Color::RED);

// Nerd Font 图标（半角，各占1格）
canvas.set_string(0, 3, "   ", Color::CYAN);

// 混合文本（自动处理宽度）
canvas.set_string(0, 4, "文件: file.txt", Color::WHITE);
// '文' 占2格，'件' 占2格，': file.txt' 各占1格
```

## 📁 项目结构

```
src/
├── main.rs      # 主程序（113 行）
└── canvas.rs    # Canvas Plugin 模块（完全封装）

docs/
├── CANVAS_PLUGIN_API.md      # API 详细文档
├── BACKGROUND_COLOR_API.md   # 背景色 API
├── MODULE_STRUCTURE.md       # 模块化架构
└── FINAL_FONT_SETUP.md       # 字体系统说明

assets/
└── fonts/
    ├── README.md             # 字体说明
    └── font.ttf              # 本地字体（可选）
```

## 🎯 使用场景

### 游戏开发
- Roguelike 游戏
- 终端风格 RPG
- ASCII 艺术游戏

### 数据可视化
- 实时图表
- 系统监控面板
- 日志查看器

### 开发工具
- 终端仿真器
- 代码编辑器
- 调试界面

## 🔧 技术细节

### 像素对齐算法

```rust
缩放 >= 2.0x  → floor(scale)           // 2x, 3x, 4x...
1.0x ≤ 缩放 < 2.0x → floor(scale * 2) / 2  // 1.0x, 1.5x
缩放 < 1.0x   → floor(scale * 4) / 4   // 0.25x, 0.5x, 0.75x
```

应用 98% 安全边距确保画布不被遮挡。

### 渲染优化

- **Dirty Flag**: 只在画布改变时重新渲染
- **两遍渲染**: 先渲染背景（Z=0），再渲染文字（Z=1）
- **批量操作**: 提供区域操作方法减少调用次数

## 📊 性能指标

- **初始化**: ~5ms
- **全画布刷新**: ~15ms (5184 单元格)
- **局部更新**: <1ms
- **内存占用**: ~20 MB (含字体)

## 🛠️ 构建配置

### Cargo.toml

```toml
[dependencies]
bevy = { version = "0.17.3", features = [
    "dynamic_linking",  # 开发时快速编译
    "https",            # Web Assets 支持
    "web_asset_cache",  # 字体缓存
] }
```

### 编译

```bash
# 开发模式（快速迭代）
cargo run

# 发布模式（最佳性能）
cargo run --release
```

## 📝 完整示例

查看 `src/main.rs` 中的 `demo_setup` 函数，包含：
- 边框绘制
- 装饰图案
- 连写文本
- 背景色演示
- 按钮样式

## ⚙️ 配置

### 修改画布尺寸

在 `src/canvas.rs` 中修改常量：

```rust
const CANVAS_WIDTH: usize = 96;   // 修改宽度
const CANVAS_HEIGHT: usize = 54;  // 修改高度
```

### 修改字体

在 `src/canvas.rs` 中修改 URL：

```rust
const NOTO_SANS_URL: &str = "YOUR_FONT_URL";
const NERD_FONT_URL: &str = "YOUR_ICON_FONT_URL";
```

## 📚 文档

- [API 文档](docs/CANVAS_PLUGIN_API.md) - 完整 API 参考
- [Unicode 字符宽度](docs/UNICODE_WIDTH_HANDLING.md) - 字符宽度处理详解 ⭐ 重要
- [背景色 API](docs/BACKGROUND_COLOR_API.md) - 背景色使用指南
- [模块化架构](docs/MODULE_STRUCTURE.md) - 代码组织说明
- [字体系统](docs/FINAL_FONT_SETUP.md) - 字体加载详情
- [使用示例](docs/USAGE_EXAMPLE.md) - 更多示例代码

## 🐛 已知问题

- [ ] SVG 支持（占位符，未实现）
- [ ] 首次启动需要网络连接（字体下载）

## ⚠️ 重要变更（v0.6.0）

### API 行为变化

`set_string()` 和 `fill_rect()` 的行为已改变：

**之前（v0.5.0 及更早）：**
- 自动连写所有字符，不考虑字符宽度
- 中日韩字符可能显示重叠或错位

**现在（v0.6.0+）：**
- 默认不连写，每个字符独立放置
- 全角字符（中日韩、Emoji）自动占2格
- 需要连写时使用 `set_string_continuous()` 或 `fill_rect_continuous()`

### 迁移建议

```rust
// ✅ 混合文本：无需修改，自动修复
canvas.set_string(0, 0, "文件: file.txt", color);

// ⚙️ ASCII 装饰：可选使用连写模式保持紧凑
canvas.set_string_continuous(0, 1, "═══════", color);
```

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License

---

**Made with ❤️ using Bevy 0.17 and Rust**
