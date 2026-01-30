# 架构设计文档

## 总体架构

### 技术选型
- **引擎**：Bevy 0.17.3
- **架构模式**：ECS (Entity-Component-System)
- **编程语言**：Rust Edition 2024

## 模块划分

### 核心模块

```
rs-playground
├── i18n (国际化)
│   ├── 多语言支持
│   ├── 动态字体加载
│   └── 资源状态管理
│
├── menu (菜单系统)
│   ├── 主菜单 UI
│   ├── 游戏状态管理
│   └── 按钮交互
│
├── save_system (存档系统)
│   ├── 存档数据结构
│   ├── Git 风格分支
│   └── 存档管理逻辑
│
├── save_ui_animated (存档界面)
│   ├── 时间轴 UI
│   ├── 动画系统
│   └── 交互控制
│
└── settings_ui_simple (设置界面)
    ├── 标签页系统
    ├── 全屏切换
    └── 语言选择
```

## ECS 架构详解

### 实体生命周期

```
创建 → 添加组件 → 系统处理 → 清理/销毁
```

#### 实体创建流程
1. `commands.spawn()` 创建实体
2. 添加必要的组件（Component）
3. 使用 `children![]` 宏或 `.with_children()` 添加子实体
4. 系统自动处理实体的更新和渲染

#### 实体销毁流程
1. 状态切换时触发 `OnExit` 事件
2. `DespawnOnExit` 组件标记的实体被清理
3. `I18nCleanup` 组件标记的 UI 在语言切换时清理
4. `commands.entity(entity).despawn()` 手动销毁

### 组件设计原则

#### 标记组件 (Marker Components)
```rust
#[derive(Component)]
struct MenuUI;              // 标记主菜单 UI

#[derive(Component)]
struct TimelineContainer;   // 标记时间轴容器
```

用途：
- 查询过滤
- 批量操作
- 生命周期管理

#### 数据组件 (Data Components)
```rust
#[derive(Component)]
struct TimelineNode {
    index: usize,
    save_id: SaveId,
}

#[derive(Component)]
struct NodeDot {
    current_size: f32,
    target_size: f32,
}
```

用途：
- 存储实体状态
- 驱动系统逻辑
- 动画插值数据

#### 类型组件 (Type Components)
```rust
#[derive(Component)]
enum MenuButton {
    StartGame,
    Settings,
    ReturnToDesktop,
    Quit,
}

#[derive(Component)]
enum SaveButton {
    PrevSave,
    NextSave,
    // ...
}
```

用途：
- 区分实体类型
- 分支处理逻辑
- 行为多态

### 系统设计模式

#### 初始化系统 (Setup Systems)
```rust
fn setup_main_menu(
    mut commands: Commands,
    font: Res<GameFont>,
    translations: Res<Translations>,
) {
    // 创建 UI 实体树
}
```

特点：
- 只运行一次
- 使用 `run_if` 条件
- 创建实体和组件

#### 更新系统 (Update Systems)
```rust
fn animate_display_position(
    time: Res<Time>,
    mut timeline_state: ResMut<TimelineState>,
) {
    // 每帧更新状态
}
```

特点：
- 每帧运行
- 更新组件数据
- 处理动画逻辑

#### 交互系统 (Interaction Systems)
```rust
fn button_system(
    mut interaction_query: Query<(&Interaction, &MenuButton, &Children), Changed<Interaction>>,
    mut text_query: Query<&mut TextColor, With<ButtonText>>,
) {
    // 处理用户输入
}
```

特点：
- 监听事件变化
- `Changed<T>` 过滤器
- 更新 UI 反馈

#### 条件系统 (Conditional Systems)
```rust
.add_systems(
    Update,
    setup_main_menu.run_if(
        in_state(GameState::MainMenu)
            .and(in_state(LoadingState::Ready))
            .and(resource_exists::<GameFont>)
            .and(not(any_with_component::<MenuUI>))
    ),
)
```

条件类型：
- `in_state(S)`：状态匹配
- `resource_exists::<R>`：资源存在
- `any_with_component::<C>`：存在带组件的实体
- `not(condition)`：逻辑取反
- `.and(condition)`：逻辑与
- `.or(condition)`：逻辑或

### 资源管理

#### 全局资源 (Global Resources)
```rust
#[derive(Resource)]
struct SaveData {
    saves: HashMap<SaveId, SavePoint>,
    next_id: SaveId,
    current_save: SaveId,
}

#[derive(Resource)]
struct TimelineState {
    main_line: Vec<SaveId>,
    target_position: usize,
    display_position: f32,
}
```

使用方式：
- `Res<T>`：只读访问
- `ResMut<T>`：可写访问
- `insert_resource(T)`：添加资源
- `remove_resource::<T>()`：移除资源

#### 资源加载流程
```rust
资源请求 → 加载中状态 → 检查完成 → 就绪状态
   ↓           ↓            ↓           ↓
LoadingState::Loading  →  check  →  Ready
```

### 状态机设计

#### 游戏状态 (GameState)
```rust
#[derive(States)]
enum GameState {
    MainMenu,      // 主菜单
    InGame,        // 游戏中
    Settings,      // 设置
    LoadGame,      // 存档界面
}
```

状态转换：
```rust
game_state.set(GameState::LoadGame);  // 切换状态
```

系统响应：
```rust
.add_systems(OnEnter(GameState::MainMenu), setup_menu)
.add_systems(OnExit(GameState::MainMenu), cleanup_menu)
.add_systems(Update, update_menu.run_if(in_state(GameState::MainMenu)))
```

#### 加载状态 (LoadingState)
```rust
#[derive(States)]
enum LoadingState {
    Loading,    // 加载中
    Ready,      // 就绪
}
```

用途：
- 控制 UI 显示时机
- 避免资源未加载时访问
- 协调异步加载流程

## 动画系统架构

### 三层分离架构

```
控制层 (Control Layer)
    ↓ 修改目标值
状态层 (State Layer)
    ↓ 插值计算
动画层 (Animation Layer)
    ↓ 应用到组件
渲染层 (Rendering Layer)
```

### 插值动画原理

```rust
// 状态数据
target_position: usize     // 目标（整数，离散）
display_position: f32      // 显示（浮点，连续）

// LERP 公式
new_value = current + (target - current) * speed * delta_time

// 实现
fn animate_display_position(
    time: Res<Time>,
    mut timeline_state: ResMut<TimelineState>,
) {
    let target = timeline_state.target_position as f32;
    let current = timeline_state.display_position;
    let delta = target - current;
    
    if delta.abs() > 0.01 {
        let speed = LERP_SPEED * time.delta_secs();
        timeline_state.display_position += delta * speed;
    } else {
        timeline_state.display_position = target;
    }
}
```

### 动画组件模式

```rust
#[derive(Component)]
struct Animated {
    current: f32,
    target: f32,
}

// 系统：更新动画
fn update_animation(mut query: Query<&mut Animated>) {
    for mut anim in query.iter_mut() {
        // LERP 插值
        anim.current = lerp(anim.current, anim.target, speed);
    }
}

// 系统：应用到视觉
fn apply_animation(query: Query<(&Animated, &mut Node)>) {
    for (anim, mut node) in query.iter() {
        node.width = Val::Px(anim.current);
    }
}
```

## UI 布局系统

### Flexbox 布局

```rust
Node {
    // 尺寸
    width: Val::Px(100.0),         // 固定像素
    height: Val::Percent(50.0),    // 百分比
    
    // Flex 容器
    flex_direction: FlexDirection::Row,    // 横向/纵向
    justify_content: JustifyContent::Center, // 主轴对齐
    align_items: AlignItems::Center,       // 交叉轴对齐
    
    // 间距
    padding: UiRect::all(px(10)),          // 内边距
    margin: UiRect::left(px(20)),          // 外边距
    column_gap: px(5),                     // 列间距
    row_gap: px(5),                        // 行间距
    
    // 定位
    position_type: PositionType::Absolute, // 绝对定位
    left: Val::Px(50.0),
    top: Val::Px(100.0),
    
    // 其他
    overflow: Overflow::clip(),            // 溢出裁剪
    ..default()
}
```

### 坐标系统

#### 相对定位 (Relative)
- 默认模式
- 基于父容器
- 受 Flexbox 影响

#### 绝对定位 (Absolute)
- 脱离文档流
- 基于父容器坐标
- 不受 Flexbox 影响
- 用于精确控制位置

### 层级管理 (Z-Index)

```rust
ZIndex(0)     // 默认层
ZIndex(10)    // 时间轴（高于控制栏）
ZIndex(100)   // 弹窗
ZIndex(1000)  // 最顶层
```

原则：
- 背景：0-9
- 内容：10-99
- 浮层：100-999
- 模态：1000+

## 事件系统

### Bevy 内置事件

```rust
// 鼠标滚轮
fn scroll_timeline(
    mut scroll_events: EventReader<MouseWheel>,
) {
    for event in scroll_events.read() {
        // 处理滚动
    }
}

// 应用退出
fn quit_game(mut app_exit: EventWriter<AppExit>) {
    app_exit.send(AppExit::Success);
}
```

### 交互状态

```rust
enum Interaction {
    Pressed,    // 按下
    Hovered,    // 悬浮
    None,       // 无交互
}

// 自动检测变化
Query<&Interaction, Changed<Interaction>>
```

## 插件系统

### 插件定义
```rust
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<MyState>()
            .insert_resource(MyResource::default())
            .add_systems(Startup, setup)
            .add_systems(Update, update)
            .add_systems(OnEnter(MyState::Active), on_enter)
            .add_systems(OnExit(MyState::Active), on_exit);
    }
}
```

### 插件组合
```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)  // Bevy 默认插件
        .add_plugins(I18nPlugin)      // 自定义插件
        .add_plugins(MenuPlugin)
        .run();
}
```

优点：
- 模块化
- 可复用
- 解耦合
- 易测试

## 性能优化

### 查询优化

#### 使用过滤器
```rust
// 好：只查询改变的
Query<&Interaction, Changed<Interaction>>

// 差：每帧都查询全部
Query<&Interaction>
```

#### 拆分查询
```rust
// 好：分开查询
Query<&Transform, With<Player>>
Query<&Transform, With<Enemy>>

// 差：单一大查询
Query<(&Transform, Option<&Player>, Option<&Enemy>)>
```

### 系统优化

#### 条件运行
```rust
// 只在需要时运行
.add_systems(Update, expensive_system.run_if(in_state(Active)))
```

#### 系统集合
```rust
// 批量配置
.add_systems(Update, (
    system_a,
    system_b,
    system_c,
).chain())  // 顺序执行
```

### 资源优化

#### 懒加载
- 按需加载字体
- 延迟加载资源
- 缓存网络资源

#### 实体池
- 复用实体而非创建/销毁
- 减少内存分配
- 提高性能

## 调试技巧

### 日志输出
```rust
println!("Debug: {:?}", value);
info!("Info message");
warn!("Warning message");
error!("Error message");
```

### 查询诊断
```rust
// 检查实体数量
let count = query.iter().count();
println!("Entity count: {}", count);

// 检查组件存在
if let Ok(component) = query.get(entity) {
    println!("Component: {:?}", component);
}
```

### 状态监控
```rust
fn debug_state(state: Res<State<GameState>>) {
    println!("Current state: {:?}", state.get());
}
```

## 最佳实践

1. **组件设计**
   - 保持组件小而专注
   - 使用标记组件分类实体
   - 避免组件间依赖

2. **系统设计**
   - 单一职责原则
   - 使用条件运行
   - 合理拆分系统

3. **状态管理**
   - 清晰的状态转换
   - 避免状态污染
   - 使用 OnEnter/OnExit 清理

4. **资源管理**
   - 及时清理未使用资源
   - 使用资源而非全局变量
   - 避免资源循环依赖

5. **性能考虑**
   - 使用 Changed 过滤器
   - 避免不必要的查询
   - 批量处理操作

---

*本文档持续更新中*
