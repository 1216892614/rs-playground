use bevy::{prelude::*, sprite::Anchor, text::Justify, window::WindowResized};
use unicode_width::UnicodeWidthChar;

// ==================== 常量（内部） ====================

const CANVAS_WIDTH: usize = 96;
const CANVAS_HEIGHT: usize = 54;
const BASE_CELL_SIZE: f32 = 16.0; // 基础单元格尺寸（像素）
const SCALE_SAFETY_MARGIN: f32 = 0.98; // 安全边距，确保画布不会被遮挡
const CONTAINER_PADDING_PX: f32 = 2.0; // 单行容器左右内边距（像素）

// Web 字体 CDN 链接（GitHub Raw）
const NOTO_SANS_URL: &str = "https://github.com/notofonts/noto-cjk/raw/main/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf";
const NERD_FONT_URL: &str = "https://github.com/ryanoasis/nerd-fonts/raw/master/patched-fonts/Inconsolata/InconsolataNerdFont-Regular.ttf";

// ==================== Canvas Plugin ====================

pub struct CanvasPlugin;

impl Plugin for CanvasPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Canvas::new())
            .insert_resource(CanvasFonts::default())
            .add_systems(Startup, (load_fonts, initial_resize))
            .add_systems(
                Update,
                (
                    check_font_loading,
                    handle_window_resize,
                    process_canvas_commands,
                    render_canvas,
                )
                    .chain(),
            );
    }
}

// ==================== 公共类型 ====================

/// 文本对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

// ==================== 内部数据结构 ====================

#[derive(Debug, Clone)]
enum CellContent {
    Empty,
    Char(char),
    // 占位符：表示这个格子被前一个字符占用（全角字符的第二个格子）
    Continuation,
    // SVG 路径或标识符
    Svg(String),
}

#[derive(Debug, Clone)]
struct Cell {
    content: CellContent,
    // 连写范围：>0 表示从当前格开始的连写字符串占用的格子数
    span: usize,
    color: Color,
    background_color: Option<Color>,
    /// 仅用于 string 起始格：容器右边界（不包含），文本在此范围内按 align 放置
    container_end: Option<usize>,
    /// 仅用于 string 起始格：容器内文本对齐方式
    container_align: Option<TextAlign>,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            content: CellContent::Empty,
            span: 1,
            color: Color::WHITE,
            background_color: None,
            container_end: None,
            container_align: None,
        }
    }
}

// ==================== 公共 Canvas Resource ====================

#[derive(Resource)]
pub struct Canvas {
    cells: Vec<Vec<Cell>>,
    cell_size: f32,
    scale: f32,
    dirty: bool,
}

impl Canvas {
    pub fn new() -> Self {
        let cells = vec![vec![Cell::default(); CANVAS_WIDTH]; CANVAS_HEIGHT];
        Self {
            cells,
            cell_size: BASE_CELL_SIZE,
            scale: 1.0,
            dirty: true,
        }
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// 清除范围内所有 string/char 内容，保留背景色；并清除与该范围相交的 string 整段
    fn clear_range_string_content(&mut self, y: usize, start: usize, end: usize) {
        if y >= CANVAS_HEIGHT || start >= end {
            return;
        }
        let end = end.min(CANVAS_WIDTH);
        let row = &mut self.cells[y];

        let clear_cell_content = |cell: &mut Cell| {
            let bg = cell.background_color;
            *cell = Cell {
                content: CellContent::Empty,
                span: 1,
                color: Color::WHITE,
                background_color: bg,
                container_end: None,
                container_align: None,
            };
        };

        // 清除起点在 start 之前但延伸到 [start, end) 的 string 整段
        let mut s = 0;
        while s < start {
            let span = row[s].span;
            if span > 0 && s + span > start {
                let run_end = (s + span).min(CANVAS_WIDTH);
                for i in s..run_end {
                    clear_cell_content(&mut row[i]);
                }
                s = run_end;
            } else {
                s += 1;
            }
        }

        // 清除 [start, end) 内的内容；若某格是一段 string 的起点则整段清除
        let mut i = start;
        while i < end {
            let span = row[i].span;
            clear_cell_content(&mut row[i]);
            if span > 0 {
                let run_end = (i + span).min(CANVAS_WIDTH);
                for j in (i + 1)..run_end {
                    clear_cell_content(&mut row[j]);
                }
                i = run_end;
            } else {
                i += 1;
            }
        }
    }

    // ==================== 公共绘制 API ====================

    /// 设置单个字符（自动处理全角字符占位）
    pub fn set_char(&mut self, x: usize, y: usize, ch: char, color: Color) {
        if x >= CANVAS_WIDTH || y >= CANVAS_HEIGHT {
            return;
        }

        let width = ch.width().unwrap_or(1);
        let clear_end = (x + width).min(CANVAS_WIDTH);
        self.clear_range_string_content(y, x, clear_end);

        // 设置主字符
        self.cells[y][x] = Cell {
            content: CellContent::Char(ch),
            span: 0,
            color,
            background_color: None,
            container_end: None,
            container_align: None,
        };

        if width == 2 && x + 1 < CANVAS_WIDTH {
            self.cells[y][x + 1] = Cell {
                content: CellContent::Continuation,
                span: 0,
                color,
                background_color: None,
                container_end: None,
                container_align: None,
            };
        }

        self.mark_dirty();
    }

    /// 设置单个字符（带背景色，永远不连写）
    pub fn set_char_with_bg(
        &mut self,
        x: usize,
        y: usize,
        ch: char,
        color: Color,
        bg_color: Color,
    ) {
        if x >= CANVAS_WIDTH || y >= CANVAS_HEIGHT {
            return;
        }

        let width = ch.width().unwrap_or(1);
        let clear_end = (x + width).min(CANVAS_WIDTH);
        self.clear_range_string_content(y, x, clear_end);

        // 设置主字符
        self.cells[y][x] = Cell {
            content: CellContent::Char(ch),
            span: 0,
            color,
            background_color: Some(bg_color),
            container_end: None,
            container_align: None,
        };

        if width == 2 && x + 1 < CANVAS_WIDTH {
            self.cells[y][x + 1] = Cell {
                content: CellContent::Continuation,
                span: 0,
                color,
                background_color: Some(bg_color),
                container_end: None,
                container_align: None,
            };
        }

        self.mark_dirty();
    }

    /// 设置单行文本（在指定范围内连写渲染，支持对齐和省略；后续可扩展为多行）
    pub fn set_line(
        &mut self,
        y: usize,
        x_start: usize,
        x_end: usize,
        text: &str,
        align: TextAlign,
        color: Color,
    ) {
        if y >= CANVAS_HEIGHT || x_start >= x_end || x_start >= CANVAS_WIDTH {
            return;
        }

        let x_end = x_end.min(CANVAS_WIDTH);
        let available_width = x_end - x_start;

        self.clear_range_string_content(y, x_start, x_end);

        let (final_text, text_width) = Self::fit_text_with_ellipsis(text, available_width);

        if final_text.is_empty() {
            self.mark_dirty();
            return;
        }

        let actual_x = match align {
            TextAlign::Left => x_start,
            TextAlign::Center => x_start + (available_width.saturating_sub(text_width)) / 2,
            TextAlign::Right => x_start + available_width.saturating_sub(text_width),
        };

        let char_count = final_text.chars().count();
        if char_count == 0 {
            self.mark_dirty();
            return;
        }

        self.cells[y][actual_x] = Cell {
            content: CellContent::Char(final_text.chars().next().unwrap()),
            span: text_width,
            color,
            background_color: None,
            container_end: Some(x_end),
            container_align: Some(align),
        };

        for (i, ch) in final_text.chars().enumerate().skip(1) {
            let cell_x = actual_x + i;
            if cell_x < x_end && cell_x < CANVAS_WIDTH {
                self.cells[y][cell_x] = Cell {
                    content: CellContent::Char(ch),
                    span: 0,
                    color,
                    background_color: None,
                    container_end: None,
                    container_align: None,
                };
            }
        }

        self.mark_dirty();
    }

    /// 设置单行文本（带背景色）
    pub fn set_line_with_bg(
        &mut self,
        y: usize,
        x_start: usize,
        x_end: usize,
        text: &str,
        align: TextAlign,
        color: Color,
        bg_color: Color,
    ) {
        if y >= CANVAS_HEIGHT || x_start >= x_end || x_start >= CANVAS_WIDTH {
            return;
        }

        let x_end = x_end.min(CANVAS_WIDTH);
        let available_width = x_end - x_start;

        self.clear_range_string_content(y, x_start, x_end);

        for x in x_start..x_end {
            if self.cells[y][x].background_color.is_none() {
                self.cells[y][x].background_color = Some(bg_color);
            }
        }

        let (final_text, text_width) = Self::fit_text_with_ellipsis(text, available_width);

        if final_text.is_empty() {
            self.mark_dirty();
            return;
        }

        let actual_x = match align {
            TextAlign::Left => x_start,
            TextAlign::Center => x_start + (available_width.saturating_sub(text_width)) / 2,
            TextAlign::Right => x_start + available_width.saturating_sub(text_width),
        };

        let char_count = final_text.chars().count();
        if char_count == 0 {
            self.mark_dirty();
            return;
        }

        self.cells[y][actual_x] = Cell {
            content: CellContent::Char(final_text.chars().next().unwrap()),
            span: text_width,
            color,
            background_color: Some(bg_color),
            container_end: Some(x_end),
            container_align: Some(align),
        };

        for (i, ch) in final_text.chars().enumerate().skip(1) {
            let cell_x = actual_x + i;
            if cell_x < x_end && cell_x < CANVAS_WIDTH {
                self.cells[y][cell_x] = Cell {
                    content: CellContent::Char(ch),
                    span: 0,
                    color,
                    background_color: Some(bg_color),
                    container_end: None,
                    container_align: None,
                };
            }
        }

        self.mark_dirty();
    }

    fn fit_text_with_ellipsis(text: &str, available_width: usize) -> (String, usize) {
        if available_width == 0 {
            return (String::new(), 0);
        }

        let mut current_width = 0;
        let mut chars_vec: Vec<char> = Vec::new();

        for ch in text.chars() {
            let ch_width = ch.width().unwrap_or(1);
            if current_width + ch_width <= available_width {
                chars_vec.push(ch);
                current_width += ch_width;
            } else {
                if available_width >= 3 {
                    while current_width + 3 > available_width && !chars_vec.is_empty() {
                        if let Some(last_ch) = chars_vec.pop() {
                            current_width -= last_ch.width().unwrap_or(1);
                        }
                    }
                    chars_vec.push('.');
                    chars_vec.push('.');
                    chars_vec.push('.');
                    current_width += 3;
                } else if available_width >= 1 {
                    chars_vec.clear();
                    chars_vec.push('…');
                    current_width = 1;
                }
                break;
            }
        }

        let final_text: String = chars_vec.into_iter().collect();
        (final_text, current_width)
    }

    /// 设置 SVG
    pub fn set_svg(&mut self, x: usize, y: usize, svg_id: &str, color: Color) {
        if x < CANVAS_WIDTH && y < CANVAS_HEIGHT {
            self.cells[y][x] = Cell {
                content: CellContent::Svg(svg_id.to_string()),
                span: 1,
                color,
                background_color: None,
                container_end: None,
                container_align: None,
            };
            self.mark_dirty();
        }
    }

    /// 设置单元格背景色
    #[allow(dead_code)]
    pub fn set_background(&mut self, x: usize, y: usize, bg_color: Color) {
        if x < CANVAS_WIDTH && y < CANVAS_HEIGHT {
            self.cells[y][x].background_color = Some(bg_color);
            self.mark_dirty();
        }
    }

    /// 清除单元格背景色
    #[allow(dead_code)]
    pub fn clear_background(&mut self, x: usize, y: usize) {
        if x < CANVAS_WIDTH && y < CANVAS_HEIGHT {
            self.cells[y][x].background_color = None;
            self.mark_dirty();
        }
    }

    /// 清除整个画布
    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = Cell::default();
            }
        }
        self.mark_dirty();
    }

    /// 清除矩形区域
    pub fn clear_rect(&mut self, x: usize, y: usize, width: usize, height: usize) {
        let end_x = (x + width).min(CANVAS_WIDTH);
        let end_y = (y + height).min(CANVAS_HEIGHT);

        for row in y..end_y {
            for col in x..end_x {
                self.cells[row][col] = Cell::default();
            }
        }
        self.mark_dirty();
    }

    /// 填充矩形区域
    pub fn fill_rect(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        ch: char,
        color: Color,
    ) {
        let end_x = (x + width).min(CANVAS_WIDTH);
        let end_y = (y + height).min(CANVAS_HEIGHT);

        for row in y..end_y {
            for col in x..end_x {
                self.cells[row][col] = Cell {
                    content: CellContent::Char(ch),
                    span: 1,
                    color,
                    background_color: None,
                    container_end: None,
                    container_align: None,
                };
            }
        }
        self.mark_dirty();
    }

    /// 填充矩形区域（带背景色）
    pub fn fill_rect_with_bg(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        ch: char,
        color: Color,
        bg_color: Color,
    ) {
        let end_x = (x + width).min(CANVAS_WIDTH);
        let end_y = (y + height).min(CANVAS_HEIGHT);

        for row in y..end_y {
            for col in x..end_x {
                self.cells[row][col] = Cell {
                    content: CellContent::Char(ch),
                    span: 1,
                    color,
                    background_color: Some(bg_color),
                    container_end: None,
                    container_align: None,
                };
            }
        }
        self.mark_dirty();
    }

    /// 填充矩形背景（不改变内容）
    pub fn fill_background_rect(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        bg_color: Color,
    ) {
        let end_x = (x + width).min(CANVAS_WIDTH);
        let end_y = (y + height).min(CANVAS_HEIGHT);

        for row in y..end_y {
            for col in x..end_x {
                self.cells[row][col].background_color = Some(bg_color);
            }
        }
        self.mark_dirty();
    }
}

// ==================== 字体资源（内部） ====================

#[derive(Resource)]
struct CanvasFonts {
    noto_sans: Option<Handle<Font>>,
    nerd_font: Option<Handle<Font>>,
    load_attempted: bool,
    noto_loaded: bool,
    nerd_loaded: bool,
}

impl Default for CanvasFonts {
    fn default() -> Self {
        Self {
            noto_sans: None,
            nerd_font: None,
            load_attempted: false,
            noto_loaded: false,
            nerd_loaded: false,
        }
    }
}

fn load_fonts(mut fonts: ResMut<CanvasFonts>, asset_server: Res<AssetServer>) {
    if fonts.load_attempted {
        return;
    }

    fonts.load_attempted = true;

    info!("=== Loading fonts via Web Assets ===");

    // Noto Sans CJK - 主字体（支持中文、日文、韩文和各种符号）
    info!("Loading Noto Sans CJK from GitHub...");
    fonts.noto_sans = Some(asset_server.load(NOTO_SANS_URL));

    // Nerd Font - 图标字体（8000+ 开发者图标）
    info!("Loading Nerd Font from GitHub...");
    fonts.nerd_font = Some(asset_server.load(NERD_FONT_URL));

    info!("Fonts are downloading from CDN. Please wait 10-30 seconds...");
}

// 检查字体加载状态
fn check_font_loading(
    mut fonts: ResMut<CanvasFonts>,
    font_assets: Res<Assets<Font>>,
    mut canvas: ResMut<Canvas>,
) {
    if fonts.noto_loaded && fonts.nerd_loaded {
        return;
    }

    let mut any_loaded = false;

    if !fonts.noto_loaded {
        if let Some(handle) = &fonts.noto_sans {
            if font_assets.get(handle).is_some() {
                info!("✓ Noto Sans CJK loaded!");
                fonts.noto_loaded = true;
                any_loaded = true;
            }
        }
    }

    if !fonts.nerd_loaded {
        if let Some(handle) = &fonts.nerd_font {
            if font_assets.get(handle).is_some() {
                info!("✓ Nerd Font loaded!");
                fonts.nerd_loaded = true;
                any_loaded = true;
            }
        }
    }

    if any_loaded {
        if fonts.noto_loaded && fonts.nerd_loaded {
            info!("=== All fonts loaded successfully! ===");
        }
        canvas.mark_dirty();
    }
}

// ==================== 组件和命令（内部） ====================

#[derive(Component)]
struct CanvasMarker;

#[derive(Component)]
struct CellEntity {
    _x: usize,
    _y: usize,
}

/// 画布命令组件 - 添加到实体上以执行画布操作
#[derive(Component)]
#[allow(dead_code)]
pub enum CanvasCommand {
    /// 清除整个画布
    Clear,
    /// 清除矩形区域 (x, y, width, height)
    ClearRect(usize, usize, usize, usize),
    /// 填充矩形区域 (x, y, width, height, char, color)
    FillRect(usize, usize, usize, usize, char, Color),
    /// 填充矩形区域带背景 (x, y, width, height, char, fg_color, bg_color)
    FillRectWithBg(usize, usize, usize, usize, char, Color, Color),
    /// 设置字符 (x, y, char, color)
    SetChar(usize, usize, char, Color),
    /// 设置字符带背景 (x, y, char, fg_color, bg_color)
    SetCharWithBg(usize, usize, char, Color, Color),
    /// 设置字符串 (y, x_start, x_end, text, align, color)
    SetLine(usize, usize, usize, String, TextAlign, Color),
    /// 设置字符串带背景 (y, x_start, x_end, text, align, fg_color, bg_color)
    SetLineWithBg(usize, usize, usize, String, TextAlign, Color, Color),
    /// 设置 SVG (x, y, svg_id, color)
    SetSvg(usize, usize, String, Color),
    /// 填充背景矩形 (x, y, width, height, bg_color)
    FillBackgroundRect(usize, usize, usize, usize, Color),
}

/// 处理画布命令
fn process_canvas_commands(
    mut commands: Commands,
    mut canvas: ResMut<Canvas>,
    query: Query<(Entity, &CanvasCommand)>,
) {
    for (entity, command) in query.iter() {
        match command {
            CanvasCommand::Clear => {
                canvas.clear();
            }
            CanvasCommand::ClearRect(x, y, w, h) => {
                canvas.clear_rect(*x, *y, *w, *h);
            }
            CanvasCommand::FillRect(x, y, w, h, ch, color) => {
                canvas.fill_rect(*x, *y, *w, *h, *ch, *color);
            }
            CanvasCommand::FillRectWithBg(x, y, w, h, ch, fg, bg) => {
                canvas.fill_rect_with_bg(*x, *y, *w, *h, *ch, *fg, *bg);
            }
            CanvasCommand::SetChar(x, y, ch, color) => {
                canvas.set_char(*x, *y, *ch, *color);
            }
            CanvasCommand::SetCharWithBg(x, y, ch, fg, bg) => {
                canvas.set_char_with_bg(*x, *y, *ch, *fg, *bg);
            }
            CanvasCommand::SetLine(y, x_start, x_end, text, align, color) => {
                canvas.set_line(*y, *x_start, *x_end, text, *align, *color);
            }
            CanvasCommand::SetLineWithBg(y, x_start, x_end, text, align, fg, bg) => {
                canvas.set_line_with_bg(*y, *x_start, *x_end, text, *align, *fg, *bg);
            }
            CanvasCommand::SetSvg(x, y, svg_id, color) => {
                canvas.set_svg(*x, *y, svg_id, *color);
            }
            CanvasCommand::FillBackgroundRect(x, y, w, h, bg) => {
                canvas.fill_background_rect(*x, *y, *w, *h, *bg);
            }
        }
        // 命令执行后移除该组件
        commands.entity(entity).despawn();
    }
}

// ==================== 系统函数（内部） ====================

// 初始化时计算正确的缩放
fn initial_resize(mut canvas: ResMut<Canvas>, window_query: Query<&Window>) {
    if let Ok(window) = window_query.single() {
        // 计算像素对齐的缩放比例
        let width_scale = window.resolution.width() / (CANVAS_WIDTH as f32 * BASE_CELL_SIZE);
        let height_scale = window.resolution.height() / (CANVAS_HEIGHT as f32 * BASE_CELL_SIZE);

        // 使用较小的缩放比例并应用安全边距以确保 contain
        let scale = width_scale.min(height_scale) * SCALE_SAFETY_MARGIN;

        // 像素对齐：向下取整以确保画布不会超出窗口
        let aligned_scale = if scale >= 2.0 {
            scale.floor()
        } else if scale >= 1.0 {
            (scale * 2.0).floor() / 2.0
        } else {
            (scale * 4.0).floor() / 4.0
        };

        canvas.scale = aligned_scale.max(0.25);
        canvas.cell_size = BASE_CELL_SIZE * canvas.scale;
        canvas.mark_dirty();

        info!(
            "Initial canvas scale: {}, cell_size: {}",
            canvas.scale, canvas.cell_size
        );
        info!(
            "Window size: {}x{}",
            window.resolution.width(),
            window.resolution.height()
        );
        info!(
            "Canvas size: {}x{}",
            CANVAS_WIDTH as f32 * canvas.cell_size,
            CANVAS_HEIGHT as f32 * canvas.cell_size
        );
    }
}

fn handle_window_resize(
    mut canvas: ResMut<Canvas>,
    window_query: Query<&Window>,
    mut resize_events: MessageReader<WindowResized>,
) {
    for event in resize_events.read() {
        if let Ok(_window) = window_query.single() {
            // 计算像素对齐的缩放比例
            let width_scale = event.width / (CANVAS_WIDTH as f32 * BASE_CELL_SIZE);
            let height_scale = event.height / (CANVAS_HEIGHT as f32 * BASE_CELL_SIZE);

            // 使用较小的缩放比例并应用安全边距以确保 contain
            let scale = width_scale.min(height_scale) * SCALE_SAFETY_MARGIN;

            // 像素对齐：向下取整以确保画布不会超出窗口
            let aligned_scale = if scale >= 2.0 {
                scale.floor()
            } else if scale >= 1.0 {
                (scale * 2.0).floor() / 2.0
            } else {
                (scale * 4.0).floor() / 4.0
            };

            canvas.scale = aligned_scale.max(0.25);
            canvas.cell_size = BASE_CELL_SIZE * canvas.scale;
            canvas.mark_dirty();
        }
    }
}

fn render_canvas(
    mut commands: Commands,
    mut canvas: ResMut<Canvas>,
    query: Query<Entity, With<CanvasMarker>>,
    fonts: Res<CanvasFonts>,
) {
    // 只在画布改变时重新渲染
    if !canvas.dirty {
        return;
    }

    canvas.dirty = false;

    // 清除旧的渲染
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    let cell_size = canvas.cell_size;
    let canvas_width = CANVAS_WIDTH as f32 * cell_size;
    let canvas_height = CANVAS_HEIGHT as f32 * cell_size;

    // 画布左上角位置（居中）
    let origin_x = -canvas_width / 2.0;
    let origin_y = canvas_height / 2.0;

    // 第一遍：渲染背景（string 的容器整块绘制，其余按格绘制）
    for (y, row) in canvas.cells.iter().enumerate() {
        let mut x = 0;
        while x < row.len() {
            let cell = &row[x];
            if let Some(bg_color) = cell.background_color {
                if let Some(container_end) = cell.container_end {
                    // 整块容器 [x, container_end)，屏幕上一块矩形
                    let end = container_end.min(CANVAS_WIDTH);
                    let w = (end - x) as f32 * cell_size;
                    let pos_x = origin_x + (x as f32 + end as f32) / 2.0 * cell_size;
                    let pos_y = origin_y - (y as f32 + 0.5) * cell_size;
                    commands.spawn((
                        Sprite {
                            color: bg_color,
                            custom_size: Some(Vec2::new(w, cell_size)),
                            ..default()
                        },
                        Transform::from_xyz(pos_x, pos_y, 0.0),
                        CanvasMarker,
                    ));
                    x = end;
                    continue;
                }
                let pos_x = origin_x + (x as f32 + 0.5) * cell_size;
                let pos_y = origin_y - (y as f32 + 0.5) * cell_size;
                commands.spawn((
                    Sprite {
                        color: bg_color,
                        custom_size: Some(Vec2::splat(cell_size)),
                        ..default()
                    },
                    Transform::from_xyz(pos_x, pos_y, 0.0),
                    CanvasMarker,
                ));
            }
            x += 1;
        }
    }

    // 第二遍：渲染文字内容
    for (y, row) in canvas.cells.iter().enumerate() {
        let mut x = 0;
        while x < row.len() {
            let cell = &row[x];

            match &cell.content {
                CellContent::Empty => {
                    x += 1;
                }
                CellContent::Continuation => {
                    // 跳过占位符（全角字符的第二个格子）
                    x += 1;
                }
                CellContent::Char(ch) => {
                    if cell.span > 0 {
                        // 连写起始格：在容器 [x, container_end) 内按 align 放置文本
                        let mut text = String::new();
                        text.push(*ch);
                        for i in 1..cell.span {
                            if x + i < row.len() {
                                if let CellContent::Char(next_ch) = row[x + i].content {
                                    text.push(next_ch);
                                }
                            }
                        }

                        let container_end = cell.container_end.unwrap_or(x + cell.span);
                        let align = cell.container_align.unwrap_or(TextAlign::Left);
                        let container_width = (container_end - x) as f32;

                        // 容器 [x, container_end)：左右各 CONTAINER_PADDING_PX 内边距，按 align 放置文本
                        let (pos_x, anchor, justify) = match align {
                            TextAlign::Left => (
                                origin_x + x as f32 * cell_size + CONTAINER_PADDING_PX,
                                Anchor::CENTER_LEFT,
                                Justify::Left,
                            ),
                            TextAlign::Center => (
                                origin_x + (x as f32 + container_width / 2.0) * cell_size,
                                Anchor::CENTER,
                                Justify::Center,
                            ),
                            TextAlign::Right => (
                                origin_x + container_end as f32 * cell_size - CONTAINER_PADDING_PX,
                                Anchor::CENTER_RIGHT,
                                Justify::Right,
                            ),
                        };
                        let pos_y = origin_y - (y as f32 + 0.5) * cell_size;

                        let font_size = (cell_size * 0.8).max(8.0);

                        let has_nerd_icons = text.chars().any(|c| {
                            let code = c as u32;
                            (code >= 0xE000 && code <= 0xF8FF) || (code >= 0xF0000)
                        });

                        let font = if has_nerd_icons && fonts.nerd_loaded {
                            fonts.nerd_font.clone()
                        } else if fonts.noto_loaded {
                            fonts.noto_sans.clone()
                        } else {
                            None
                        };

                        let text_layout = TextLayout::new_with_justify(justify);

                        // 与 Bevy 官方 text2d 示例一致：显式构造 TextFont，确保字体 handle 被正确使用
                        let text_font = match &font {
                            Some(handle) => TextFont {
                                font: handle.clone(),
                                font_size,
                                ..default()
                            },
                            None => TextFont {
                                font_size,
                                ..default()
                            },
                        };
                        commands.spawn((
                            Text2d::new(text),
                            text_font,
                            text_layout,
                            TextColor(cell.color),
                            anchor,
                            Transform::from_xyz(pos_x, pos_y, 1.0),
                            CanvasMarker,
                            CellEntity { _x: x, _y: y },
                        ));

                        x += cell.span;
                    } else {
                        // 单字符（set_char），单独渲染
                        let pos_x = origin_x + (x as f32 + 0.5) * cell_size;
                        let pos_y = origin_y - (y as f32 + 0.5) * cell_size;
                        let font_size = (cell_size * 0.8).max(8.0);
                        let single_text = ch.to_string();
                        let has_nerd = (*ch as u32 >= 0xE000 && *ch as u32 <= 0xF8FF)
                            || (*ch as u32 >= 0xF0000);
                        let font = if has_nerd && fonts.nerd_loaded {
                            fonts.nerd_font.clone()
                        } else if fonts.noto_loaded {
                            fonts.noto_sans.clone()
                        } else {
                            None
                        };
                        let text_font = match &font {
                            Some(handle) => TextFont {
                                font: handle.clone(),
                                font_size,
                                ..default()
                            },
                            None => TextFont {
                                font_size,
                                ..default()
                            },
                        };
                        commands.spawn((
                            Text2d::new(single_text),
                            text_font,
                            TextColor(cell.color),
                            Transform::from_xyz(pos_x, pos_y, 1.0),
                            CanvasMarker,
                            CellEntity { _x: x, _y: y },
                        ));
                        x += 1;
                    }
                }
                CellContent::Svg(_svg_id) => {
                    let pos_x = origin_x + (x as f32 + 0.5) * cell_size;
                    let pos_y = origin_y - (y as f32 + 0.5) * cell_size;

                    // 这里可以加载实际的 SVG 资源
                    // 暂时用方块代替
                    commands.spawn((
                        Sprite {
                            color: cell.color,
                            custom_size: Some(Vec2::splat(cell_size * 0.8)),
                            ..default()
                        },
                        Transform::from_xyz(pos_x, pos_y, 1.0),
                        CanvasMarker,
                        CellEntity { _x: x, _y: y },
                    ));

                    x += 1;
                }
            }
        }
    }

    // 在 cell 交界处渲染黑色细网格线
    const GRID_LINE_COLOR: Color = Color::BLACK;
    let line_thickness = 0.5;

    // 垂直线：x = origin_x + i * cell_size, i = 0..=96
    for i in 0..=CANVAS_WIDTH {
        let x = origin_x + i as f32 * cell_size;
        let center_y = origin_y - canvas_height / 2.0;
        commands.spawn((
            Sprite {
                color: GRID_LINE_COLOR,
                custom_size: Some(Vec2::new(line_thickness, canvas_height + line_thickness)),
                ..default()
            },
            Transform::from_xyz(x, center_y, 2.0), // Z=2 置于最上以便调试
            CanvasMarker,
        ));
    }

    // 水平线：y = origin_y - j * cell_size, j = 0..=54
    for j in 0..=CANVAS_HEIGHT {
        let y = origin_y - j as f32 * cell_size;
        let center_x = origin_x + canvas_width / 2.0;
        commands.spawn((
            Sprite {
                color: GRID_LINE_COLOR,
                custom_size: Some(Vec2::new(canvas_width + line_thickness, line_thickness)),
                ..default()
            },
            Transform::from_xyz(center_x, y, 2.0),
            CanvasMarker,
        ));
    }
}
