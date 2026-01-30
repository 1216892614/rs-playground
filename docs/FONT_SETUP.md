# 字体设置完全指南

## 快速设置（推荐）

### Windows 用户

在项目根目录运行：

```powershell
Copy-Item "C:\Windows\Fonts\msyh.ttc" "assets\fonts\font.ttf"
```

这会复制微软雅黑字体，支持中文、日文、韩文和各种符号。

### Linux 用户

```bash
cp /usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc assets/fonts/font.ttf
```

或者：

```bash
cp /usr/share/fonts/truetype/dejavu/DejaVuSans.ttf assets/fonts/font.ttf
```

### macOS 用户

```bash
cp "/System/Library/Fonts/PingFang.ttc" assets/fonts/font.ttf
```

## 支持的字体

Canvas Plugin 已测试并支持以下字体：

### 1. 微软雅黑 (Windows 自带) ✓ 推荐
- **文件**: `C:\Windows\Fonts\msyh.ttc`
- **大小**: ~20 MB
- **支持**: 中文、日文、韩文、拉丁字母、常用符号
- **优点**: Windows 系统自带，无需下载

### 2. Noto Sans CJK
- **下载**: https://github.com/notofonts/noto-cjk/releases
- **大小**: ~15-20 MB per variant
- **支持**: 优秀的 CJK 字符支持
- **变体**:
  - `NotoSansCJKsc-Regular.otf` - 简体中文
  - `NotoSansCJKtc-Regular.otf` - 繁体中文
  - `NotoSansCJKjp-Regular.otf` - 日文
  - `NotoSansCJKkr-Regular.otf` - 韩文

### 3. 更纱黑体 (Sarasa Gothic)
- **下载**: https://github.com/be5invis/Sarasa-Gothic/releases
- **大小**: ~10-15 MB
- **支持**: CJK + 编程字体特性
- **推荐文件**: `SarasaMonoSC-Regular.ttf`

### 4. Nerd Fonts（可选，用于图标）
- **下载**: https://www.nerdfonts.com/font-downloads
- **推荐**: JetBrains Mono Nerd Font
- **大小**: ~3-5 MB
- **支持**: 8000+ 开发者图标
- **安装位置**: `assets/fonts/nerd-font.ttf`

## 手动下载步骤

### 方法 1: 从 GitHub Releases 下载

1. 访问 https://github.com/notofonts/noto-cjk/releases
2. 下载 `Sans.zip` 或对应语言的 OTF/TTF 文件
3. 解压并找到 `-Regular.otf` 或 `-Regular.ttf` 文件
4. 复制到 `assets/fonts/font.ttf`

### 方法 2: 使用系统字体

查找系统字体：

**Windows:**
```powershell
Get-ChildItem "C:\Windows\Fonts" | Where-Object { $_.Name -like "*CJK*" -or $_.Name -like "*yahei*" }
```

**Linux:**
```bash
find /usr/share/fonts -name "*CJK*" -o -name "*Sans*"
```

**macOS:**
```bash
find /System/Library/Fonts -name "*.ttc" -o -name "*.ttf"
```

## 验证字体安装

运行程序后，检查日志输出：

### 成功示例
```
INFO: === Loading fonts ===
INFO: Loading main font from: fonts/font.ttf
INFO: ✓ Main font loaded successfully!
```

### 失败示例
```
ERROR: Path not found: .../assets/fonts/font.ttf
WARN: 字体未找到！请手动下载字体文件
```

## 支持的字符

安装正确的字体后，Canvas 支持：

### 基本字符
- ✓ 拉丁字母 (A-Z, a-z)
- ✓ 数字 (0-9)
- ✓ 基本标点

### CJK 字符
- ✓ 简体中文 (你好世界)
- ✓ 繁体中文 (你好世界)
- ✓ 日文平假名 (ひらがな)
- ✓ 日文片假名 (カタカナ)
- ✓ 韩文 (한글)

### 特殊符号
- ✓ 框线字符 (─│┌┐└┘├┤┬┴┼)
- ✓ 方块字符 (█▓▒░■□▪▫)
- ✓ 箭头 (← ↑ → ↓ ↔ ↕)
- ✓ 数学符号 (≠ ≈ ≤ ≥ × ÷)

### Nerd Font 图标（需要安装 Nerd Font）
- ✓ 文件/文件夹图标
- ✓ Git 状态图标
- ✓ 开发工具图标
- ✓ 8000+ 其他图标

## 故障排查

### 问题 1: 字体文件不存在

**错误信息:**
```
ERROR: Path not found: .../assets/fonts/font.ttf
```

**解决方案:**
1. 确认 `assets/fonts/` 目录存在
2. 按照上面的步骤复制字体文件
3. 确保文件名为 `font.ttf`（小写）

### 问题 2: 中文显示为方块

**原因:** 字体不支持 CJK 字符

**解决方案:**
使用支持 CJK 的字体：
- 微软雅黑 (推荐)
- Noto Sans CJK
- 更纱黑体

### 问题 3: 符号显示不正确

**原因:** 字体缺少某些 Unicode 范围

**解决方案:**
1. 使用 Noto Sans CJK（覆盖最广）
2. 或使用更纱黑体（包含编程符号）

### 问题 4: Nerd Font 图标不显示

**原因:** 未安装 Nerd Font

**解决方案:**
1. 这是可选的，不影响基本使用
2. 如需图标，下载并安装 Nerd Font 到 `assets/fonts/nerd-font.ttf`

## 推荐配置

### 最小配置（仅中文支持）
```
assets/fonts/font.ttf  <- 微软雅黑/Noto Sans CJK
```

### 完整配置（中文 + 图标）
```
assets/fonts/font.ttf       <- 主字体
assets/fonts/nerd-font.ttf  <- 图标字体
```

## 文件大小参考

| 字体 | 大小 | 下载时间 (10Mbps) |
|------|------|------------------|
| 微软雅黑 | ~20 MB | 即时（系统自带） |
| Noto Sans CJK | ~15 MB | ~12 秒 |
| 更纱黑体 | ~10 MB | ~8 秒 |
| Nerd Font | ~3 MB | ~2 秒 |

## 常见问题

### Q: 为什么不使用 Web 字体（CDN）？
A: 大多数字体 CDN 有访问限制或跨域问题。本地字体更可靠、更快速。

### Q: 可以使用其他字体吗？
A: 可以！任何 TTF/OTF 字体都可以。只需复制到 `assets/fonts/font.ttf`。

### Q: 字体文件可以提交到 Git 吗？
A: 不建议。字体文件较大（15-20 MB），已在 `.gitignore` 中排除。

### Q: 如何在 CI/CD 中使用？
A: 在 CI 脚本中添加字体下载步骤，或使用 Docker 镜像预装字体。

### Q: 支持 web assembly (WASM) 吗？
A: 支持！Bevy 的 WebAssetPlugin 可以在 WASM 中加载本地字体。

## 进阶：自定义字体选择

修改 `src/main.rs` 中的字体路径：

```rust
const MAIN_FONT_PATH: &str = "fonts/my-custom-font.ttf";
const NERD_FONT_PATH: &str = "fonts/my-nerd-font.ttf";
```

## 许可证注意事项

确保你使用的字体许可证允许你的使用场景：

- **微软雅黑**: 仅限 Windows 系统使用
- **Noto Sans CJK**: SIL Open Font License (自由使用)
- **更纱黑体**: SIL Open Font License (自由使用)
- **Nerd Fonts**: MIT License (自由使用)

## 获取帮助

如果字体设置遇到问题：

1. 检查 `assets/fonts/` 目录是否存在
2. 确认字体文件大小 >1 MB
3. 查看程序启动日志
4. 参考本文档的故障排查部分
