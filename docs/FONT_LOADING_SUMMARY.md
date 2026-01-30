# Canvas Plugin 字体加载总结

## 当前配置

### 字体来源

Canvas Plugin 从 CDN 加载三个字体：

1. **更纱黑体 (Sarasa Gothic)** - 主字体
   - URL: GitHub Raw
   - 支持: 中文、日文、韩文、拉丁字母、各种符号
   - 优先级: 最高（除图标外的所有字符）

2. **Noto Sans CJK** - 备用字体
   - URL: GitHub Raw
   - 支持: CJK 字符和基本符号
   - 优先级: 中等（Sarasa 失败时使用）

3. **Nerd Font** - 图标字体
   - URL: GitHub Raw
   - 支持: 8000+ 开发者图标
   - 优先级: 检测到图标字符时使用

### 字体选择逻辑

```rust
// 检测 Nerd Font 图标字符 (U+E000-U+F8FF, U+F0000+)
if has_nerd_icons && nerd_font_loaded {
    use nerd_font
}
// 优先使用 Sarasa Gothic（最全面）
else if sarasa_loaded {
    use sarasa
}
// 备用 Noto Sans
else if noto_sans_loaded {
    use noto_sans
}
// 最终回退到 Bevy 默认字体
else {
    use bevy_default (仅 ASCII)
}
```

## 字体加载流程

### 1. 启动时异步加载
```
App Start → load_fonts() → 
  - asset_server.load(SARASA_GOTHIC_URL)
  - asset_server.load(NOTO_SANS_URL)  
  - asset_server.load(NERD_FONT_URL)
```

### 2. 持续检查加载状态
```
Update Loop → check_font_loading() →
  检查每个字体是否加载完成 →
  设置 loaded 标记 →
  全部完成后打印确认信息
```

### 3. 渲染时使用字体
```
render_canvas() →
  根据字符类型选择已加载的字体 →
  如未加载完成，临时使用默认字体
```

## CDN URLs

### Sarasa Gothic
```
https://raw.githubusercontent.com/be5invis/Sarasa-Gothic/refs/heads/main/out/SarasaMonoSC-Regular.ttf
```

### Noto Sans CJK
```
https://github.com/notofonts/noto-cjk/raw/main/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf
```

### Nerd Font (Inconsolata)
```
https://github.com/ryanoasis/nerd-fonts/raw/master/patched-fonts/Inconsolata/InconsolataNerdFont-Regular.ttf
```

## 特性

### 优点
- ✅ 无需手动下载字体
- ✅ 自动缓存（Bevy Web Asset Cache）
- ✅ 渐进式增强（字体加载前使用默认字体）
- ✅ 多字体回退机制

### 注意事项
- ⏱️ 首次启动需要下载字体（10-30秒）
- 🌐 需要网络连接（首次）
- 💾 字体会被缓存到 `.web-asset-cache`
- 📦 总大小约 20-30 MB

## 验证字体加载

### 检查日志

成功加载时会看到：
```
INFO: === Loading fonts from CDN ===
INFO: Loading Sarasa Gothic...
INFO: Loading Noto Sans CJK...
INFO: Loading Nerd Font...
INFO: ✓ Sarasa Gothic loaded!
INFO: ✓ Noto Sans CJK loaded!
INFO: ✓ Nerd Font loaded!
INFO: === All fonts loaded successfully! ===
```

### 失败处理

如果看到 HTTP 错误：
```
ERROR: Encountered HTTP status 403/404 when loading asset
```

说明 CDN 链接失败，程序会回退到默认字体。

## 字符支持范围

### Sarasa Gothic（主字体）
- ✅ 完整 CJK 字符集
- ✅ 拉丁字母、数字
- ✅ Unicode 框线字符 (U+2500-U+257F)
- ✅ 方块字符 (U+2580-U+259F)
- ✅ 各种标点和符号
- ✅ 数学符号
- ✅ 希腊字母、西里尔字母

### Nerd Font（图标）
- ✅ 8000+ 开发者图标
- ✅ 文件类型图标
- ✅ Git 图标
- ✅ 品牌 logo
- ✅ 编程语言图标

## 性能影响

### 首次启动
- 字体下载: 10-30秒
- 内存占用: +20-30 MB
- 不阻塞程序运行

### 后续启动
- 从缓存加载: <1秒
- 内存占用: +20-30 MB
- 启动速度快

## 常见问题

### Q: 为什么有些字符显示为方块？
A: 字体还在下载中，等待 10-30 秒后会自动显示

### Q: 如何清除字体缓存？
A: 删除 `.web-asset-cache` 文件夹

### Q: 可以使用本地字体吗？
A: 可以，修改 `canvas.rs` 中的 URL 为本地路径：
```rust
const SARASA_GOTHIC_URL: &str = "fonts/sarasa.ttf";
```

### Q: 如何禁用某个字体？
A: 将对应的字体加载代码注释掉即可
