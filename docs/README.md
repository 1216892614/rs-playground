# 文档目录

欢迎来到 RS-Playground 项目文档！本目录包含项目的详细技术文档和开发指南。

## 📚 文档列表

### [架构设计文档 (ARCHITECTURE.md)](./ARCHITECTURE.md)
**内容概览**：
- 总体架构设计
- ECS 架构详解
- 模块划分说明
- 组件和系统设计模式
- 动画系统架构
- UI 布局系统
- 性能优化策略

**适合人群**：
- 想要深入了解项目架构的开发者
- 需要扩展或修改核心系统的贡献者
- 学习 Bevy ECS 架构的开发者

**重点章节**：
- ECS 架构详解
- 动画系统架构
- 最佳实践

---

### [开发指南 (DEVELOPMENT.md)](./DEVELOPMENT.md)
**内容概览**：
- 开发环境设置
- 开发工作流
- 代码规范
- Bevy 开发模式
- 常见任务教程
- 调试技巧
- 性能分析
- 测试方法
- 发布流程

**适合人群**：
- 新加入项目的开发者
- 需要设置开发环境的贡献者
- 想要学习项目开发流程的人

**重点章节**：
- 快速开始
- 常见任务（如何添加新功能）
- 调试技巧
- 常见问题

---

### [开发任务列表 (TODO.md)](./TODO.md)
**内容概览**：
- 已完成功能清单
- 进行中的任务
- 计划中的功能
- 已知问题
- 想法池
- 版本规划
- 里程碑

**适合人群**：
- 想要了解项目进度的所有人
- 寻找贡献点的开发者
- 项目管理者

**重点章节**：
- 进行中功能（可以参与）
- 已知问题（可以修复）
- 想法池（可以提出新想法）

---

## 📖 其他文档

### 项目根目录文档

#### [README.md](../README.md)
**主要内容**：
- 项目概述
- 功能特性
- 技术栈
- 游戏系统详解
- 项目结构
- 快速开始

**适合人群**：所有人（项目入口文档）

---

#### [CHANGELOG.md](../CHANGELOG.md)
**主要内容**：
- 版本历史
- 功能变更记录
- Bug 修复记录
- 版本发布日期

**适合人群**：
- 想要了解版本变化的用户
- 需要追踪功能更新的开发者

---

## 🗂️ 文档结构

```
rs-playground/
├── README.md              # 项目主文档（入口）
├── CHANGELOG.md           # 版本更新历史
├── LICENSE                # 项目许可证（待添加）
│
└── docs/                  # 详细文档目录
    ├── README.md          # 本文件（文档导航）
    ├── ARCHITECTURE.md    # 架构设计文档
    ├── DEVELOPMENT.md     # 开发指南
    └── TODO.md            # 任务列表与规划
```

## 🎯 文档使用指南

### 🆕 新手入门路径
1. 阅读 [README.md](../README.md) 了解项目
2. 按照 [DEVELOPMENT.md](./DEVELOPMENT.md) 设置开发环境
3. 查看 [TODO.md](./TODO.md) 寻找可以参与的任务
4. 需要时参考 [ARCHITECTURE.md](./ARCHITECTURE.md) 了解实现细节

### 👨‍💻 开发者路径
1. 快速浏览 [README.md](../README.md) 了解整体
2. 深入阅读 [ARCHITECTURE.md](./ARCHITECTURE.md) 理解架构
3. 参考 [DEVELOPMENT.md](./DEVELOPMENT.md) 进行开发
4. 查看 [TODO.md](./TODO.md) 了解开发计划

### 🔧 贡献者路径
1. 阅读 [README.md](../README.md) 的贡献指南
2. 在 [TODO.md](./TODO.md) 中找到感兴趣的任务
3. 参考 [DEVELOPMENT.md](./DEVELOPMENT.md) 编写代码
4. 遵循 [ARCHITECTURE.md](./ARCHITECTURE.md) 的设计模式

### 📊 管理者路径
1. 定期更新 [TODO.md](./TODO.md) 的任务状态
2. 在发布时更新 [CHANGELOG.md](../CHANGELOG.md)
3. 审查 PR 时参考 [DEVELOPMENT.md](./DEVELOPMENT.md) 的代码规范
4. 规划新功能时考虑 [ARCHITECTURE.md](./ARCHITECTURE.md) 的架构设计

## 📝 文档维护

### 更新频率
- **README.md**：功能变化时更新
- **CHANGELOG.md**：每次版本发布时更新
- **ARCHITECTURE.md**：架构变更时更新
- **DEVELOPMENT.md**：工作流变化时更新
- **TODO.md**：每周更新一次

### 维护原则
1. **保持同步**：代码变化时同步更新文档
2. **清晰简洁**：使用简单明了的语言
3. **示例丰富**：提供代码示例和截图
4. **持续改进**：根据反馈不断完善

### 贡献文档
欢迎改进文档！提交文档 PR 时请：
- 检查拼写和语法
- 确保代码示例可运行
- 保持格式一致
- 添加必要的链接

## 🔗 快速链接

### 内部链接
- [项目主页](../README.md)
- [架构文档](./ARCHITECTURE.md)
- [开发指南](./DEVELOPMENT.md)
- [任务列表](./TODO.md)
- [更新日志](../CHANGELOG.md)

### 外部资源
- [Bevy 官方文档](https://bevyengine.org/learn/)
- [Bevy Cheatbook](https://bevy-cheatbook.github.io/)
- [Rust 官方文档](https://doc.rust-lang.org/)
- [Rust Book 中文版](https://kaisery.github.io/trpl-zh-cn/)

## 💡 文档建议

如果您对文档有任何建议或发现问题，请：
1. 提交 Issue 描述问题
2. 提交 PR 直接改进
3. 在讨论区分享想法

---

**文档版本**：v0.1.0  
**最后更新**：2026-01-30  
**维护者**：项目团队

*感谢您阅读文档！祝开发顺利！* 🎉
