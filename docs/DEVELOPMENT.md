# 开发指南

## 开发环境设置

### 前置要求

1. **安装 Rust**
```bash
# Windows (使用 rustup-init.exe)
https://rustup.rs/

# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **更新到最新版本**
```bash
rustup update
```

3. **验证安装**
```bash
rustc --version
cargo --version
```

### IDE 推荐

#### Visual Studio Code
- 安装 `rust-analyzer` 插件
- 安装 `CodeLLDB` 插件（调试）
- 安装 `Even Better TOML` 插件

#### RustRover (JetBrains)
- 原生 Rust 支持
- 强大的调试功能
- 优秀的重构工具

### 项目设置

1. **克隆项目**
```bash
git clone <repository-url>
cd rs-playground
```

2. **首次构建**
```bash
cargo build
```

3. **运行项目**
```bash
cargo run
```

## 开发工作流

### 日常开发

```bash
# 快速检查（不生成可执行文件）
cargo check

# 运行项目
cargo run

# 运行发布版本
cargo run --release

# 构建（不运行）
cargo build

# 清理构建产物
cargo clean
```

### 代码质量

```bash
# 格式化代码
cargo fmt

# 检查格式（不修改）
cargo fmt -- --check

# Lint 检查
cargo clippy

# 严格 Lint（推荐）
cargo clippy -- -D warnings

# 运行测试
cargo test
```

### 依赖管理

```bash
# 更新依赖
cargo update

# 检查过时的依赖
cargo outdated

# 审计安全漏洞
cargo audit

# 查看依赖树
cargo tree
```

## 代码规范

### 文件组织

```
src/
├── main.rs              # 程序入口（简洁）
├── lib.rs               # 库入口（如需要）
├── module_name.rs       # 单文件模块
└── module_name/         # 多文件模块
    ├── mod.rs           # 模块入口
    ├── submodule.rs
    └── types.rs
```

### 命名约定

```rust
// 模块/文件名：snake_case
mod save_system;
mod save_ui_animated;

// 类型名：PascalCase
struct SavePoint { }
enum GameState { }
trait Saveable { }

// 函数/变量：snake_case
fn setup_main_menu() { }
let current_save = 0;

// 常量：SCREAMING_SNAKE_CASE
const NODE_SPACING: f32 = 120.0;
const MAX_SAVES: usize = 100;

// 生命周期：单个小写字母
fn foo<'a>(s: &'a str) { }

// 类型参数：单个大写字母或 PascalCase
fn bar<T>(item: T) { }
fn baz<Item>(item: Item) { }
```

### 代码风格

#### 导入顺序
```rust
// 1. 标准库
use std::collections::HashMap;

// 2. 外部 crate
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// 3. 当前 crate
use crate::save_system::SavePoint;
use crate::i18n::Language;
```

#### 结构体定义
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePoint {
    pub id: SaveId,
    pub name: String,
    pub timestamp: String,
    // ... 其他字段
}

impl SavePoint {
    /// 创建新的存档点
    pub fn new(id: SaveId, name: String) -> Self {
        Self {
            id,
            name,
            timestamp: Utc::now().to_string(),
        }
    }
    
    /// 检查是否为章节入口
    pub fn is_chapter_entry(&self) -> bool {
        self.is_chapter_entry
    }
}
```

#### 错误处理
```rust
// 使用 Result
fn load_save(id: SaveId) -> Result<SavePoint, String> {
    // ...
}

// 使用 Option
fn find_save(&self, id: SaveId) -> Option<&SavePoint> {
    self.saves.get(&id)
}

// 使用 ? 运算符
fn process() -> Result<(), Error> {
    let data = load_data()?;
    let parsed = parse_data(data)?;
    save_result(parsed)?;
    Ok(())
}
```

### 注释规范

```rust
/// 文档注释：描述公共 API
/// 
/// # Examples
/// 
/// ```
/// let save = SavePoint::new(0, "Chapter 1".to_string());
/// ```
/// 
/// # Panics
/// 
/// 当 id 重复时会 panic
pub fn create_save(&mut self, id: SaveId) { }

// 普通注释：解释实现细节
// 计算节点的 x 位置
let x = node_index as f32 * NODE_SPACING;

/* 块注释：
 * 用于较长的说明
 * 或暂时禁用代码
 */
```

## Bevy 开发模式

### 插件开发

```rust
// 1. 定义插件结构
pub struct MyPlugin;

// 2. 实现 Plugin trait
impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app
            // 添加状态
            .init_state::<MyState>()
            
            // 添加资源
            .insert_resource(MyResource::default())
            
            // 添加系统
            .add_systems(Startup, setup)
            .add_systems(Update, update)
            .add_systems(OnEnter(MyState::Active), on_enter)
            .add_systems(OnExit(MyState::Active), on_exit);
    }
}

// 3. 在 main.rs 中注册
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MyPlugin)
        .run();
}
```

### 系统开发

```rust
// 查询系统
fn my_system(
    // 资源
    time: Res<Time>,
    mut my_resource: ResMut<MyResource>,
    
    // 查询
    mut query: Query<(&mut Transform, &MyComponent)>,
    button_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    
    // 命令
    mut commands: Commands,
) {
    // 遍历查询结果
    for (mut transform, component) in query.iter_mut() {
        // 修改组件
        transform.translation.x += 1.0;
    }
    
    // 生成实体
    commands.spawn((
        MyComponent,
        Transform::default(),
    ));
}
```

### 组件开发

```rust
// 标记组件
#[derive(Component)]
struct Player;

// 数据组件
#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

// 实现方法
impl Health {
    fn new(max: f32) -> Self {
        Self { current: max, max }
    }
    
    fn is_alive(&self) -> bool {
        self.current > 0.0
    }
}
```

### 资源开发

```rust
// 定义资源
#[derive(Resource, Default)]
struct GameSettings {
    volume: f32,
    fullscreen: bool,
}

// 初始化
app.insert_resource(GameSettings {
    volume: 0.8,
    fullscreen: false,
});

// 使用
fn use_settings(settings: Res<GameSettings>) {
    if settings.fullscreen {
        // ...
    }
}
```

## 常见任务

### 添加新的菜单按钮

1. **定义按钮类型**
```rust
// src/menu.rs
#[derive(Component)]
enum MenuButton {
    // ... 现有按钮
    NewButton,  // 添加新按钮
}
```

2. **添加翻译**
```json
// assets/messages/zh-CN/menu.json
{
  "new_button": "新按钮"
}
```

3. **创建按钮实体**
```rust
// src/menu.rs - setup_main_menu()
create_menu_button(
    &translations,
    &font_handle,
    "new_button",
    MenuButton::NewButton,
),
```

4. **处理按钮点击**
```rust
// src/menu.rs - button_system()
match button {
    MenuButton::NewButton => {
        println!("新按钮被点击");
        // 处理逻辑
    }
    // ...
}
```

### 添加新的游戏状态

1. **定义状态**
```rust
// src/menu.rs
#[derive(States)]
enum GameState {
    // ... 现有状态
    NewState,  // 添加新状态
}
```

2. **添加系统**
```rust
// src/new_module.rs
pub struct NewStatePlugin;

impl Plugin for NewStatePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::NewState), setup_new_state)
            .add_systems(Update, update_new_state.run_if(in_state(GameState::NewState)))
            .add_systems(OnExit(GameState::NewState), cleanup_new_state);
    }
}
```

3. **注册插件**
```rust
// src/main.rs
use new_module::NewStatePlugin;

fn main() {
    App::new()
        // ...
        .add_plugins(NewStatePlugin)
        .run();
}
```

### 添加动画效果

1. **定义动画组件**
```rust
#[derive(Component)]
struct Animated {
    current: f32,
    target: f32,
}
```

2. **创建动画系统**
```rust
fn animate(
    time: Res<Time>,
    mut query: Query<&mut Animated>,
) {
    for mut anim in query.iter_mut() {
        let delta = anim.target - anim.current;
        if delta.abs() > 0.01 {
            let speed = 5.0 * time.delta_secs();
            anim.current += delta * speed;
        }
    }
}
```

3. **应用动画**
```rust
fn apply_animation(
    query: Query<(&Animated, &mut Transform)>,
) {
    for (anim, mut transform) in query.iter() {
        transform.scale = Vec3::splat(anim.current);
    }
}
```

## 调试技巧

### 打印调试

```rust
// 基础打印
println!("Value: {:?}", value);

// 漂亮打印
println!("{:#?}", complex_struct);

// 条件打印
#[cfg(debug_assertions)]
println!("Debug: {:?}", value);
```

### 使用 dbg! 宏

```rust
// 打印并返回值
let x = dbg!(some_calculation());

// 打印表达式
dbg!(player.position);

// 链式调用
player.position
    .then(dbg!(calculate_target()))
    .map(dbg!(transform));
```

### Bevy 日志

```rust
use bevy::log::*;

info!("Info message");
warn!("Warning message");
error!("Error message");
debug!("Debug message");
trace!("Trace message");
```

### 可视化调试

```rust
// 显示实体数量
fn count_entities(query: Query<Entity>) {
    info!("Total entities: {}", query.iter().count());
}

// 显示组件信息
fn debug_components(query: Query<(Entity, &Transform)>) {
    for (entity, transform) in query.iter() {
        info!("Entity {:?}: {:?}", entity, transform);
    }
}
```

## 性能分析

### 基准测试

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // 被测试的代码
            my_function(black_box(100))
        })
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

### 性能监控

```rust
// 添加帧率显示插件
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(FrameTimeDiagnosticsPlugin)
    .add_plugins(LogDiagnosticsPlugin::default())
    .run();
```

## 测试

### 单元测试

```rust
// src/save_system.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_save() {
        let mut data = SaveData::default();
        let id = data.create_save(
            "Test Save".to_string(),
            "Chapter 1".to_string(),
            false,
            SaveType::Manual,
            None,
        );
        
        assert!(data.saves.contains_key(&id));
    }
}
```

### 集成测试

```rust
// tests/integration_test.rs
use rs_playground::*;

#[test]
fn test_integration() {
    // 集成测试
}
```

## 常见问题

### 编译错误

**问题**：`error[E0425]: cannot find value`
**解决**：检查变量名拼写，确保变量已声明

**问题**：`error[E0308]: mismatched types`
**解决**：检查类型匹配，可能需要类型转换

**问题**：`error[E0382]: borrow of moved value`
**解决**：使用引用或 `.clone()`

### 运行时错误

**问题**：资源未找到 panic
**解决**：使用 `run_if(resource_exists::<T>())` 条件

**问题**：实体未找到
**解决**：使用 `get()` 而非 `single()`，检查 `Option` 返回值

**问题**：字体未加载
**解决**：检查网络连接，确保 `LoadingState` 正确管理

### 性能问题

**问题**：帧率下降
**解决**：
- 使用 `Changed<T>` 过滤器
- 拆分大系统为小系统
- 减少不必要的查询

**问题**：编译慢
**解决**：
- 使用 `dynamic_linking` 特性
- 增量编译
- 使用 `cargo check` 替代 `cargo build`

## 发布流程

### 准备发布

1. **更新版本号**
```toml
# Cargo.toml
[package]
version = "0.2.0"
```

2. **更新 CHANGELOG**
```markdown
## [0.2.0] - 2026-01-30
### Added
- 新功能
### Changed
- 修改的功能
### Fixed
- 修复的 bug
```

3. **清理代码**
```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

### 构建发布版本

```bash
# 构建
cargo build --release

# 可执行文件位置
# Windows: target/release/rs-playground.exe
# Linux/macOS: target/release/rs-playground
```

### 优化发布大小

```toml
# Cargo.toml
[profile.release]
codegen-units = 1
lto = true
opt-level = 'z'     # 优化大小
strip = true        # 移除符号
```

## 资源链接

- [Rust Book](https://doc.rust-lang.org/book/)
- [Bevy 官方文档](https://bevyengine.org/learn/)
- [Bevy Cheatbook](https://bevy-cheatbook.github.io/)
- [Rust API 文档](https://doc.rust-lang.org/std/)

---

*持续更新中*
