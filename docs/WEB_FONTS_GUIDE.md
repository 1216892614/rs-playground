# Web Fonts 加载指南

## 概述

Canvas Plugin 现在支持从 CDN 加载字体，无需手动下载和管理字体文件。

## 使用的字体

### 1. **Noto Sans CJK** - 主字体
- **用途**: 中文、日文、韩文和基本拉丁字符
- **CDN**: jsDelivr (https://cdn.jsdelivr.net/)
- **来源**: Google Noto Fonts
- **支持字符**: 
  - CJK 统一表意文字 (U+4E00 - U+9FFF)
  - CJK 扩展 A (U+3400 - U+4DBF)
  - 中文标点 (U+3000 - U+303F)
  - 拉丁字母、数字、基本符号

### 2. **JetBrains Mono Nerd Font** - 图标字体
- **用途**: 开发者图标和特殊符号
- **CDN**: jsDelivr (https://cdn.jsdelivr.net/)
- **来源**: Nerd Fonts Project
- **支持字符**:
  - Nerd Font 私有使用区 (U+E000 - U+F8FF)
  - 扩展私有使用区 (U+F0000 - U+10FFFF)
  - 包含 8000+ 个图标

## 字体加载流程

1. **启动时加载**
   ```
   [Startup] -> load_fonts() -> 从 CDN 请求字体
   ```

2. **异步下载**
   - Bevy 的 AssetServer 在后台异步下载字体
   - 不会阻塞程序启动

3. **加载检测**
   - `check_font_loading()` 系统持续检查加载状态
   - 加载成功后在日志中显示确认信息

4. **渲染使用**
   - 渲染系统根据字符类型自动选择合适的字体
   - 如果字体未加载完成，临时使用 Bevy 默认字体

## 字符自动分类

```rust
// CJK 字符 -> Noto Sans CJK
if (code >= 0x4E00 && code <= 0x9FFF) ||
   (code >= 0x3400 && code <= 0x4DBF) ||
   (code >= 0x3000 && code <= 0x303F) {
    use main_font
}

// Nerd Font 图标 -> Nerd Font
else if (code >= 0xE000 && code <= 0xF8FF) ||
        (code >= 0xF0000 && code <= 0x10FFFF) {
    use nerd_font
}

// 其他字符 -> Noto Sans CJK (保持一致性)
else {
    use main_font
}
```

## Nerd Font 图标示例

### 常用图标

#### 文件和文件夹
```
  - 文件夹
  - 文件
  - 代码文件
```

#### Git 相关
```
  - Git 分支
  - Git 提交
  - Git 合并
```

#### 开发工具
```
  - 终端
  - 齿轮/设置
  - 搜索
```

#### 状态图标
```
  - 成功/对勾
  - 警告
  - 错误
```

#### 编程语言
```
  - Python
  - JavaScript
  - Rust
```

### 在代码中使用

```rust
// 文件图标
canvas.set_string(0, 0, " file.txt", Color::WHITE);

// Git 分支
canvas.set_string(0, 1, "  main", Color::GREEN);

// 终端提示符
canvas.set_string(0, 2, " $ ", Color::CYAN);

// 状态指示
canvas.set_string(0, 3, "  Success", Color::GREEN);
canvas.set_string(0, 4, "  Warning", Color::YELLOW);
canvas.set_string(0, 5, "  Error", Color::RED);
```

## 网络要求

### CDN 访问
- **需要互联网连接**在首次运行时下载字体
- 字体文件会被 Bevy 缓存，后续启动速度更快

### CDN 提供商
- **jsDelivr**: 全球 CDN，中国大陆可访问
- **备用方案**: 如果 CDN 失败，会回退到 Bevy 默认字体

### 字体文件大小
- Noto Sans CJK: ~15-20 MB
- Nerd Font: ~3-5 MB
- **总计**: ~20-25 MB (仅首次下载)

## 故障排查

### 字体未显示

1. **检查网络连接**
   ```bash
   # 测试 CDN 可访问性
   curl https://cdn.jsdelivr.net/
   ```

2. **查看日志输出**
   ```
   INFO: Loading fonts from CDN...
   INFO: ✓ Main font (Noto Sans CJK) loaded successfully!
   INFO: ✓ Nerd Font loaded successfully!
   ```

3. **清除缓存**
   - Bevy 的字体缓存位于系统临时目录
   - 删除后重新运行程序

### 只显示英文字符

- **原因**: 字体尚未下载完成
- **解决**: 等待 10-30 秒，字体下载完成后会自动显示

### 图标显示为方块

- **原因**: Nerd Font 未正确加载
- **解决**: 
  1. 确认使用的是 Nerd Font 字符范围 (U+E000-U+F8FF)
  2. 检查日志确认 Nerd Font 加载状态

## 性能优化

### 首次启动
- 字体下载可能需要 10-30 秒
- 在背景进行，不阻塞程序

### 后续启动
- 字体被缓存，启动快速
- 无需重新下载

### 内存使用
- 两个字体文件 ~20-25 MB 内存
- 在现代系统上影响可忽略

## 自定义字体 URL

如果需要使用其他字体或 CDN，修改 `src/main.rs`：

```rust
const NOTO_SANS_CDN: &str = "YOUR_FONT_URL_HERE";
const NERD_FONT_CDN: &str = "YOUR_NERD_FONT_URL_HERE";
```

### 推荐的字体 CDN

1. **jsDelivr** (当前使用)
   - https://cdn.jsdelivr.net/
   - 全球 CDN，中国可访问

2. **unpkg**
   - https://unpkg.com/
   - npm 包 CDN

3. **GitHub Raw** (备用)
   - https://raw.githubusercontent.com/
   - 直接从仓库加载

## 离线使用

如需离线使用，可以：

1. 下载字体文件到 `assets/fonts/`
2. 修改代码使用本地路径：
   ```rust
   fonts.main_font = Some(asset_server.load("fonts/NotoSansCJKsc-Regular.otf"));
   ```

## 许可证

- **Noto Sans CJK**: SIL Open Font License 1.1
- **Nerd Fonts**: MIT License

两者均可自由用于商业和非商业项目。
