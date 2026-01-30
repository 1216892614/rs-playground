# Canvas Plugin 使用示例

## 完整示例代码

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CanvasPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            update_timer,
            animate_progress_bar,
            handle_keyboard,
        ))
        .run();
}

// 计时器资源
#[derive(Resource)]
struct GameTimer {
    elapsed: f32,
    progress: f32,
}

impl Default for GameTimer {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            progress: 0.0,
        }
    }
}

// 初始化
fn setup(mut commands: Commands, mut canvas: ResMut<Canvas>) {
    commands.spawn(Camera2d::default());
    commands.init_resource::<GameTimer>();
    
    // 绘制静态 UI
    draw_ui(&mut canvas);
}

// 绘制 UI 框架
fn draw_ui(canvas: &mut Canvas) {
    // 标题
    canvas.set_string(
        30, 2, 
        "Canvas Demo 演示", 
        Color::srgb(0.0, 1.0, 1.0)
    );
    
    // 主边框
    for x in 0..96 {
        canvas.set_char(x, 0, '═', Color::srgb(0.5, 0.5, 0.5));
        canvas.set_char(x, 53, '═', Color::srgb(0.5, 0.5, 0.5));
    }
    for y in 1..53 {
        canvas.set_char(0, y, '║', Color::srgb(0.5, 0.5, 0.5));
        canvas.set_char(95, y, '║', Color::srgb(0.5, 0.5, 0.5));
    }
    canvas.set_char(0, 0, '╔', Color::srgb(0.5, 0.5, 0.5));
    canvas.set_char(95, 0, '╗', Color::srgb(0.5, 0.5, 0.5));
    canvas.set_char(0, 53, '╚', Color::srgb(0.5, 0.5, 0.5));
    canvas.set_char(95, 53, '╝', Color::srgb(0.5, 0.5, 0.5));
    
    // 分隔线
    canvas.set_char(0, 5, '╠', Color::srgb(0.5, 0.5, 0.5));
    canvas.set_char(95, 5, '╣', Color::srgb(0.5, 0.5, 0.5));
    for x in 1..95 {
        canvas.set_char(x, 5, '═', Color::srgb(0.5, 0.5, 0.5));
    }
    
    // 功能区标签
    canvas.set_string(3, 7, "功能演示区域:", Color::srgb(1.0, 1.0, 0.0));
    
    // 帮助文本
    canvas.set_string(3, 50, "按 Space 重置 | ESC 退出", Color::srgb(0.7, 0.7, 0.7));
}

// 更新计时器
fn update_timer(
    mut timer: ResMut<GameTimer>,
    time: Res<Time>,
) {
    timer.elapsed += time.delta_secs();
    timer.progress = (timer.elapsed * 0.2).sin() * 0.5 + 0.5; // 0.0 to 1.0
}

// 动画进度条
fn animate_progress_bar(
    timer: Res<GameTimer>,
    mut canvas: ResMut<Canvas>,
) {
    let bar_y = 10;
    let bar_x = 5;
    let bar_width = 80;
    
    // 标签
    canvas.set_string(bar_x, bar_y - 1, "进度条:", Color::WHITE);
    
    // 清除进度条区域
    canvas.clear_rect(bar_x, bar_y, bar_width + 2, 1);
    
    // 边框
    canvas.set_char(bar_x, bar_y, '[', Color::WHITE);
    canvas.set_char(bar_x + bar_width + 1, bar_y, ']', Color::WHITE);
    
    // 填充
    let filled = (bar_width as f32 * timer.progress) as usize;
    for i in 0..filled {
        canvas.set_char(bar_x + 1 + i, bar_y, '█', Color::srgb(0.0, 1.0, 0.0));
    }
    for i in filled..bar_width {
        canvas.set_char(bar_x + 1 + i, bar_y, '░', Color::srgb(0.3, 0.3, 0.3));
    }
    
    // 百分比文本
    let percentage = (timer.progress * 100.0) as usize;
    canvas.set_string(
        bar_x + bar_width / 2 - 2,
        bar_y + 2,
        &format!("{}%", percentage),
        Color::srgb(1.0, 1.0, 0.0)
    );
    
    // 动态彩色块
    draw_animated_blocks(&mut canvas, &timer);
    
    // 文字跑马灯
    draw_marquee(&mut canvas, &timer);
}

// 绘制动画色块
fn draw_animated_blocks(canvas: &mut Canvas, timer: &GameTimer) {
    let blocks_y = 15;
    canvas.set_string(5, blocks_y - 1, "彩色动画块:", Color::WHITE);
    
    for i in 0..10 {
        let offset = ((timer.elapsed * 2.0 + i as f32) % 10.0) as usize;
        let x = 5 + offset * 8;
        let colors = [
            Color::srgb(1.0, 0.0, 0.0), // 红
            Color::srgb(1.0, 0.5, 0.0), // 橙
            Color::srgb(1.0, 1.0, 0.0), // 黄
            Color::srgb(0.0, 1.0, 0.0), // 绿
            Color::srgb(0.0, 1.0, 1.0), // 青
            Color::srgb(0.0, 0.0, 1.0), // 蓝
            Color::srgb(0.5, 0.0, 1.0), // 紫
        ];
        let color = colors[i % colors.len()];
        
        if x < 90 {
            canvas.set_string(x, blocks_y, "████", color);
        }
    }
}

// 跑马灯文字
fn draw_marquee(canvas: &mut Canvas, timer: &GameTimer) {
    let marquee_y = 20;
    let text = ">>> 这是一段滚动的跑马灯文字 Scrolling Text <<<  ";
    let offset = (timer.elapsed * 10.0) as usize % text.len();
    
    canvas.set_string(5, marquee_y - 1, "跑马灯:", Color::WHITE);
    canvas.clear_rect(5, marquee_y, 85, 1);
    
    // 循环文字
    let display_text: String = text.chars()
        .cycle()
        .skip(offset)
        .take(80)
        .collect();
    
    canvas.set_string(5, marquee_y, &display_text, Color::srgb(1.0, 0.5, 1.0));
}

// 键盘处理
fn handle_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut canvas: ResMut<Canvas>,
    mut timer: ResMut<GameTimer>,
    mut exit: EventWriter<AppExit>,
) {
    // ESC 退出
    if keys.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
    
    // 空格重置
    if keys.just_pressed(KeyCode::Space) {
        timer.elapsed = 0.0;
        timer.progress = 0.0;
        canvas.clear();
        draw_ui(&mut canvas);
    }
    
    // C 键清除中心区域
    if keys.just_pressed(KeyCode::KeyC) {
        commands.spawn(CanvasCommand::ClearRect(10, 25, 76, 20));
    }
    
    // F 键填充随机方块
    if keys.just_pressed(KeyCode::KeyF) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(5..85);
        let y = rng.gen_range(25..45);
        let w = rng.gen_range(3..10);
        let h = rng.gen_range(3..8);
        let chars = ['█', '▓', '▒', '░', '■', '□', '▪', '▫'];
        let ch = chars[rng.gen_range(0..chars.len())];
        let color = Color::srgb(
            rng.gen::<f32>(),
            rng.gen::<f32>(),
            rng.gen::<f32>(),
        );
        
        commands.spawn(CanvasCommand::FillRect(x, y, w, h, ch, color));
    }
}
```

## 运行示例

```bash
cargo run
```

## 交互操作

- **Space**: 重置画布
- **C**: 清除中心区域
- **F**: 填充随机彩色方块
- **ESC**: 退出程序

## 依赖添加

如果使用随机功能，需要在 `Cargo.toml` 中添加：

```toml
[dependencies]
rand = "0.8"
```

## 进阶示例

### 示例 1: 绘制图表

```rust
fn draw_bar_chart(canvas: &mut Canvas, data: &[f32]) {
    let max_height = 20;
    let bar_width = 3;
    let start_x = 10;
    let start_y = 40;
    
    for (i, &value) in data.iter().enumerate() {
        let height = (value * max_height as f32) as usize;
        let x = start_x + i * (bar_width + 1);
        
        // 绘制柱状图
        for h in 0..height {
            canvas.fill_rect(
                x, 
                start_y - h, 
                bar_width, 
                1,
                '█',
                Color::srgb(0.0, 0.8, 1.0)
            );
        }
        
        // 数值标签
        canvas.set_string(
            x,
            start_y - height - 1,
            &format!("{:.1}", value),
            Color::WHITE
        );
    }
}
```

### 示例 2: ASCII 艺术动画

```rust
fn draw_ascii_art(canvas: &mut Canvas, frame: usize) {
    let art = vec![
        "  /\\_/\\  ",
        " ( o.o ) ",
        "  > ^ <  ",
    ];
    
    let x = 40 + ((frame as f32 * 0.1).sin() * 10.0) as usize;
    let y = 25;
    
    for (i, line) in art.iter().enumerate() {
        canvas.set_string(x, y + i, line, Color::srgb(1.0, 0.8, 0.0));
    }
}
```

### 示例 3: 实时数据监控

```rust
#[derive(Resource)]
struct DataMonitor {
    values: VecDeque<f32>,
    max_size: usize,
}

fn update_monitor(
    mut monitor: ResMut<DataMonitor>,
    mut canvas: ResMut<Canvas>,
) {
    // 获取新数据（示例）
    let new_value = rand::random::<f32>();
    monitor.values.push_back(new_value);
    if monitor.values.len() > monitor.max_size {
        monitor.values.pop_front();
    }
    
    // 绘制折线图
    let y_base = 30;
    let x_start = 5;
    canvas.clear_rect(x_start, y_base - 10, 85, 12);
    
    for (i, &value) in monitor.values.iter().enumerate() {
        let x = x_start + i;
        let y = y_base - (value * 10.0) as usize;
        canvas.set_char(x, y, '●', Color::srgb(0.0, 1.0, 0.0));
    }
}
```

## 性能提示

1. **批量操作**: 使用 `fill_rect` 而不是循环调用 `set_char`
2. **命令组合**: 使用命令组件批量执行多个操作
3. **区域清除**: 只清除需要更新的区域，而不是整个画布
4. **条件渲染**: 使用 `dirty` flag 避免不必要的重绘

## 调试技巧

```rust
// 显示坐标网格
fn debug_grid(canvas: &mut Canvas) {
    // X 轴标记
    for x in (0..96).step_by(10) {
        canvas.set_string(x, 0, &format!("{}", x), Color::GRAY);
    }
    
    // Y 轴标记
    for y in (0..54).step_by(5) {
        canvas.set_string(0, y, &format!("{}", y), Color::GRAY);
    }
}
```
