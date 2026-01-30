# 字体文件说明

Canvas Plugin 需要字体文件来支持中文和特殊符号。

## 推荐字体

### 1. Sarasa Gothic (更纱黑体) - 推荐
- 下载地址: https://github.com/be5invis/Sarasa-Gothic/releases
- 支持中文、日文、韩文和各种符号
- 下载 `sarasa-gothic-ttf-<version>.7z`
- 解压后将 `sarasa-mono-sc-regular.ttf` 复制到此目录

### 2. Microsoft YaHei (微软雅黑) - Windows 自带
- 路径: `C:\Windows\Fonts\msyh.ttc`
- 复制到此目录并重命名为 `msyh.ttf`

### 3. 使用系统字体
复制以下任一字体到此目录：
- Windows: `C:\Windows\Fonts\msyh.ttc` (微软雅黑)
- Linux: `/usr/share/fonts/` 下的任何中文字体
- macOS: `/System/Library/Fonts/` 下的字体

## 快速设置 (Windows)

在 PowerShell 中运行：
```powershell
Copy-Item "C:\Windows\Fonts\msyh.ttc" "assets\fonts\font.ttf"
```

## 文件名要求

将字体文件命名为以下之一：
- `font.ttf` - 默认字体
- `msyh.ttf` - 微软雅黑
- `sarasa-mono-sc-regular.ttf` - 更纱黑体

程序会自动检测并加载这些文件。
