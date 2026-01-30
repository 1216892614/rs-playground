//! 像素编辑器：16×16 图标编辑，调色板（theme 颜色）、导出 JSON 到剪贴板、从剪贴板导入。

use bevy::color::Srgba;
use bevy::prelude::*;
use clipboard::ClipboardProvider;
use serde::{Deserialize, Serialize};

use crate::canvas::{
    CellHoverEvent, CellPressEvent, CellReleaseEvent, Canvas, TextAlign, CANVAS_HEIGHT, CANVAS_WIDTH,
};
use crate::status_bar::StatusBarExternalHoverText;
use crate::theme::{PaletteEntry, Theme};

// ==================== 常量 ====================

const ICON_SIZE: usize = 16;
/// 每个图标像素占 2×2 = 4 个 cell，方便点击
const CELLS_PER_PIXEL: usize = 2;
/// 画布上 16×16 图标区域（按 cell 计为 32×32）左上角
const ICON_X: usize = 2;
const ICON_Y: usize = 2;
/// 图标区域占用的 cell 宽高
const ICON_CELL_W: usize = ICON_SIZE * CELLS_PER_PIXEL; // 32
const ICON_CELL_H: usize = ICON_SIZE * CELLS_PER_PIXEL;  // 32

/// 当前选中颜色块（2×2 = 4 cell），与调色板分隔
const SELECTED_BOX_X: usize = 38;
const SELECTED_BOX_Y: usize = 2;
const SELECTED_BOX_SIZE: usize = 2;

/// 调色板网格起始位置（在选中色块下方，留一行间隔）
const PALETTE_GRID_X: usize = 38;
const PALETTE_GRID_Y: usize = 6;
/// 调色板每色块 2×2 cell
const PALETTE_SLOT_SIZE: usize = 2;
/// 调色板每行色块数（按 slot 计）
const PALETTE_SLOT_COLS: usize = 8;

/// 画笔行：在调色板下方空两行后，一排 1×1～8×8，每项左数字右图标
const BRUSH_ROW_Y: usize = 18;
const BRUSH_ROW_X: usize = 38;
/// 每项占 2 cell：左数字、右图标
const BRUSH_ITEM_WIDTH: usize = 2;
const BRUSH_SIZES: usize = 8; // 1..=8

/// 导出/导入/返回 按钮区域
const BTN_EXPORT_Y: usize = 40;
const BTN_IMPORT_Y: usize = 42;
const BTN_BACK_Y: usize = 44;
const BTN_X_START: usize = 2;
const BTN_X_END: usize = 30;

// ==================== 导出 JSON 格式 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct IconExport {
    pub name: Option<String>,
    #[serde(rename = "pixels")]
    pub pixels: Vec<PixelData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PixelData {
    pub x: usize,
    pub y: usize,
    pub color: String, // "#rrggbb"
}

fn color_to_hex(c: Color) -> String {
    let srgba: Srgba = c.into();
    let r = (srgba.red * 255.0).round() as u8;
    let g = (srgba.green * 255.0).round() as u8;
    let b = (srgba.blue * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn hex_to_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
    Some(Color::srgb(r, g, b))
}

// ==================== 应用状态（由 main 定义，此处引用） ====================

// AppState 在 main.rs 定义

// ==================== 像素编辑器状态 ====================

#[derive(Resource, Default)]
pub struct PixelEditorState {
    /// 16×16 像素颜色，默认透明/黑。索引 [y][x]
    pub pixels: [[Color; ICON_SIZE]; ICON_SIZE],
    /// 当前选中的调色板索引
    pub selected_palette_index: usize,
    /// 图标名称（导出用）
    pub icon_name: String,
    /// 上一帧绘制状态，用于重绘检测
    prev_selected: usize,
    prev_pixels: [[Color; ICON_SIZE]; ICON_SIZE],
    /// 当前悬浮的图标像素 (px, py)，用于 hover 预览
    pub hovered_icon_pixel: Option<(usize, usize)>,
    /// 当前悬浮的调色板索引
    pub hovered_palette_index: Option<usize>,
    prev_hovered_icon: Option<(usize, usize)>,
    prev_hovered_palette: Option<usize>,
    /// 画笔尺寸 1～8（1×1 到 8×8）
    pub brush_size: usize,
    prev_brush_size: usize,
}

const STATUS_ROW_TOP: usize = 52;

fn default_pixel() -> Color {
    Color::srgb(0.1, 0.1, 0.12)
}

impl PixelEditorState {
    pub fn new() -> Self {
        let px = default_pixel();
        Self {
            pixels: [[px; ICON_SIZE]; ICON_SIZE],
            selected_palette_index: 0,
            icon_name: String::new(),
            prev_selected: 999,
            prev_pixels: [[default_pixel(); ICON_SIZE]; ICON_SIZE],
            hovered_icon_pixel: None,
            hovered_palette_index: None,
            prev_hovered_icon: None,
            prev_hovered_palette: None,
            brush_size: 1,
            prev_brush_size: 0,
        }
    }

    fn pixels_changed(&self) -> bool {
        self.pixels != self.prev_pixels
            || self.selected_palette_index != self.prev_selected
            || self.hovered_icon_pixel != self.prev_hovered_icon
            || self.hovered_palette_index != self.prev_hovered_palette
            || self.brush_size != self.prev_brush_size
    }

    fn mark_drawn(&mut self) {
        self.prev_pixels = self.pixels;
        self.prev_selected = self.selected_palette_index;
        self.prev_hovered_icon = self.hovered_icon_pixel;
        self.prev_hovered_palette = self.hovered_palette_index;
        self.prev_brush_size = self.brush_size;
    }
}

// ==================== 区域判断 ====================

/// 将画布 cell 坐标映射到 16×16 图标像素坐标（每个像素占 2×2 cell）
fn cell_to_icon_pixel(x: usize, y: usize) -> Option<(usize, usize)> {
    if x < ICON_X || x >= ICON_X + ICON_CELL_W || y < ICON_Y || y >= ICON_Y + ICON_CELL_H {
        return None;
    }
    let px = (x - ICON_X) / CELLS_PER_PIXEL;
    let py = (y - ICON_Y) / CELLS_PER_PIXEL;
    if px < ICON_SIZE && py < ICON_SIZE {
        Some((px, py))
    } else {
        None
    }
}

/// 是否在「当前选中颜色」块内（仅显示，点击无效，需点调色板选色）
fn cell_in_selected_box(x: usize, y: usize) -> bool {
    x >= SELECTED_BOX_X
        && x < SELECTED_BOX_X + SELECTED_BOX_SIZE
        && y >= SELECTED_BOX_Y
        && y < SELECTED_BOX_Y + SELECTED_BOX_SIZE
}

/// 将画布 cell 坐标映射到调色板索引（仅调色板网格，不含选中色块）
fn cell_to_palette_index(x: usize, y: usize, entries_len: usize) -> Option<usize> {
    if x < PALETTE_GRID_X || y < PALETTE_GRID_Y {
        return None;
    }
    let cell_col = x - PALETTE_GRID_X;
    let cell_row = y - PALETTE_GRID_Y;
    let slot_col = cell_col / PALETTE_SLOT_SIZE;
    let slot_row = cell_row / PALETTE_SLOT_SIZE;
    if slot_col >= PALETTE_SLOT_COLS {
        return None;
    }
    let idx = slot_row * PALETTE_SLOT_COLS + slot_col;
    if idx < entries_len {
        Some(idx)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorButton {
    Export,
    Import,
    Back,
}

/// 将画布 cell 映射到画笔尺寸 1～8（按数字选）
fn cell_to_brush_size(x: usize, y: usize) -> Option<usize> {
    if y != BRUSH_ROW_Y || x < BRUSH_ROW_X {
        return None;
    }
    let col = x - BRUSH_ROW_X;
    if col >= BRUSH_SIZES * BRUSH_ITEM_WIDTH {
        return None;
    }
    let size = col / BRUSH_ITEM_WIDTH + 1; // 1..=8
    Some(size)
}

fn cell_to_editor_button(x: usize, y: usize) -> Option<EditorButton> {
    if x < BTN_X_START || x >= BTN_X_END {
        return None;
    }
    match y {
        BTN_EXPORT_Y => Some(EditorButton::Export),
        BTN_IMPORT_Y => Some(EditorButton::Import),
        BTN_BACK_Y => Some(EditorButton::Back),
        _ => None,
    }
}

// ==================== Plugin ====================

pub struct PixelEditorPlugin;

impl Plugin for PixelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PixelEditorState::new())
            .add_systems(
                Update,
                (
                    pixel_editor_cell_events,
                    pixel_editor_draw,
                )
                    .chain(),
            );
    }
}

// ==================== 系统：事件（点击像素、调色板、按钮；悬浮调色板显示名称） ====================

fn pixel_editor_cell_events(
    state: Res<State<crate::AppState>>,
    mut editor: ResMut<PixelEditorState>,
    mut external: ResMut<StatusBarExternalHoverText>,
    theme: Res<Theme>,
    mut ev_hover: EventReader<CellHoverEvent>,
    mut ev_press: EventReader<CellPressEvent>,
    mut ev_release: EventReader<CellReleaseEvent>,
    mut next_state: ResMut<NextState<crate::AppState>>,
) {
    if *state.get() != crate::AppState::PixelEditor {
        return;
    }
    let palette = theme.palette_entries();

    for ev in ev_hover.read() {
        if let Some((x, y)) = ev.cell {
            if let Some(idx) = cell_to_palette_index(x, y, palette.len()) {
                editor.hovered_palette_index = Some(idx);
                editor.hovered_icon_pixel = None;
                external.0 = Some(palette[idx].0.to_string());
            } else if cell_in_selected_box(x, y) {
                editor.hovered_palette_index = None;
                editor.hovered_icon_pixel = None;
                let idx = editor.selected_palette_index;
                external.0 = Some(
                    palette
                        .get(idx)
                        .map(|e| format!("当前: {}", e.0))
                        .unwrap_or_else(|| "当前颜色".to_string()),
                );
            } else if let Some((px, py)) = cell_to_icon_pixel(x, y) {
                editor.hovered_icon_pixel = Some((px, py));
                editor.hovered_palette_index = None;
                external.0 = Some("像素".to_string());
            } else if let Some(size) = cell_to_brush_size(x, y) {
                editor.hovered_icon_pixel = None;
                editor.hovered_palette_index = None;
                external.0 = Some(format!("画笔 {}×{}", size, size));
            } else if let Some(btn) = cell_to_editor_button(x, y) {
                editor.hovered_icon_pixel = None;
                editor.hovered_palette_index = None;
                external.0 = Some(match btn {
                    EditorButton::Export => "导出到剪贴板".to_string(),
                    EditorButton::Import => "从剪贴板导入".to_string(),
                    EditorButton::Back => "返回主菜单".to_string(),
                });
            } else {
                editor.hovered_icon_pixel = None;
                editor.hovered_palette_index = None;
                external.0 = None;
            }
        } else {
            editor.hovered_icon_pixel = None;
            editor.hovered_palette_index = None;
            external.0 = None;
        }
    }

    for _ev in ev_press.read() {
        // 按下时记录，在 release 时处理
    }

    for ev in ev_release.read() {
        let (x, y) = (ev.x, ev.y);
        if let Some((px, py)) = cell_to_icon_pixel(x, y) {
            let c = palette
                .get(editor.selected_palette_index)
                .map(|e| e.1)
                .unwrap_or(default_pixel());
            let sz = editor.brush_size.clamp(1, 8);
            // 以 (px, py) 为中心填 sz×sz 方块，边界夹到 0..16
            let half = sz / 2;
            let mut x0 = px.saturating_sub(half);
            let mut y0 = py.saturating_sub(half);
            x0 = x0.min(ICON_SIZE.saturating_sub(sz));
            y0 = y0.min(ICON_SIZE.saturating_sub(sz));
            for dy in 0..sz {
                for dx in 0..sz {
                    let ix = x0 + dx;
                    let iy = y0 + dy;
                    if ix < ICON_SIZE && iy < ICON_SIZE {
                        editor.pixels[iy][ix] = c;
                    }
                }
            }
        } else if let Some(size) = cell_to_brush_size(x, y) {
            editor.brush_size = size.clamp(1, 8);
        } else if let Some(idx) = cell_to_palette_index(x, y, palette.len()) {
            editor.selected_palette_index = idx;
        } else if let Some(btn) = cell_to_editor_button(x, y) {
            match btn {
                EditorButton::Export => {
                    let data = export_icon(&editor);
                    if let Ok(json) = serde_json::to_string_pretty(&data) {
                        if let Ok(mut ctx) = clipboard::ClipboardContext::new() {
                            let _ = ctx.set_contents(json);
                        }
                    }
                }
                EditorButton::Import => {
                    if let Ok(mut ctx) = clipboard::ClipboardContext::new() {
                        let contents: Result<String, _> = ctx.get_contents();
                        if let Ok(s) = contents {
                            if let Ok(data) = serde_json::from_str::<IconExport>(&s) {
                                import_icon(&mut editor, &data);
                            }
                        }
                    }
                }
                EditorButton::Back => {
                    next_state.set(crate::AppState::MainMenu);
                }
            }
        }
    }
}

fn export_icon(editor: &PixelEditorState) -> IconExport {
    let mut pixels = Vec::new();
    for (y, row) in editor.pixels.iter().enumerate() {
        for (x, &c) in row.iter().enumerate() {
            pixels.push(PixelData {
                x,
                y,
                color: color_to_hex(c),
            });
        }
    }
    IconExport {
        name: if editor.icon_name.is_empty() {
            None
        } else {
            Some(editor.icon_name.clone())
        },
        pixels,
    }
}

fn import_icon(editor: &mut PixelEditorState, data: &IconExport) {
    if let Some(ref name) = data.name {
        editor.icon_name = name.clone();
    }
    for pd in &data.pixels {
        if pd.x < ICON_SIZE && pd.y < ICON_SIZE {
            if let Some(c) = hex_to_color(&pd.color) {
                editor.pixels[pd.y][pd.x] = c;
            }
        }
    }
    editor.mark_drawn();
}

// ==================== 系统：绘制 ====================

fn pixel_editor_draw(
    state: Res<State<crate::AppState>>,
    mut canvas: ResMut<Canvas>,
    mut editor: ResMut<PixelEditorState>,
    theme: Res<Theme>,
) {
    if *state.get() != crate::AppState::PixelEditor {
        return;
    }
    if !editor.pixels_changed() {
        return;
    }

    // 只清除状态栏上方区域，保留状态栏两行由 status_bar 绘制
    canvas.clear_rect(0, 0, CANVAS_WIDTH, STATUS_ROW_TOP);
    let bg = theme.bg.basalt_blue;
    let border = theme.text.muted;
    let palette = theme.palette_entries();

    // 标题
    canvas.set_line(
        0,
        0,
        CANVAS_WIDTH,
        " 像素编辑器 16×16 ",
        TextAlign::Center,
        theme.text.primary,
    );

    // 16×16 图标区域：每个像素 2×2 cell
    for py in 0..ICON_SIZE {
        for px in 0..ICON_SIZE {
            let cx = ICON_X + px * CELLS_PER_PIXEL;
            let cy = ICON_Y + py * CELLS_PER_PIXEL;
            let c = editor.pixels[py][px];
            canvas.fill_rect_with_bg(cx, cy, CELLS_PER_PIXEL, CELLS_PER_PIXEL, ' ', theme.text.primary, c);
        }
    }
    // 图标区边框画在区域外一圈，不覆盖边缘像素（避免边/角只有 1×2 或 1×1）
    let outer_left = ICON_X.saturating_sub(1);
    let outer_top = ICON_Y.saturating_sub(1);
    let outer_right = ICON_X + ICON_CELL_W;
    let outer_bottom = ICON_Y + ICON_CELL_H;
    for x in outer_left..=outer_right.min(CANVAS_WIDTH.saturating_sub(1)) {
        canvas.set_char_with_bg(x, outer_top, '─', border, bg);
        if outer_bottom < CANVAS_HEIGHT {
            canvas.set_char_with_bg(x, outer_bottom, '─', border, bg);
        }
    }
    for y in outer_top..=outer_bottom.min(CANVAS_HEIGHT.saturating_sub(1)) {
        canvas.set_char_with_bg(outer_left, y, '│', border, bg);
        if outer_right < CANVAS_WIDTH {
            canvas.set_char_with_bg(outer_right, y, '│', border, bg);
        }
    }

    // 当前选中颜色（悬浮边框与选中块共用）
    let selected_color = palette
        .get(editor.selected_palette_index)
        .map(|e| e.1)
        .unwrap_or(default_pixel());

    // Hover 效果：按当前画笔尺寸显示绘制范围，边框图标采用选择颜色
    if let Some((hpx, hpy)) = editor.hovered_icon_pixel {
        let sz = editor.brush_size.clamp(1, 8);
        let half = sz / 2;
        let mut x0 = hpx.saturating_sub(half);
        let mut y0 = hpy.saturating_sub(half);
        x0 = x0.min(ICON_SIZE.saturating_sub(sz));
        y0 = y0.min(ICON_SIZE.saturating_sub(sz));
        let left = ICON_X + x0 * CELLS_PER_PIXEL;
        let top = ICON_Y + y0 * CELLS_PER_PIXEL;
        let w_cells = sz * CELLS_PER_PIXEL;
        let h_cells = sz * CELLS_PER_PIXEL;
        // 边框线符用选择颜色
        let (ch_top, ch_side) = ('─', '│');
        for cx in left..(left + w_cells) {
            if cx < CANVAS_WIDTH {
                if top < CANVAS_HEIGHT {
                    canvas.set_char_with_bg(cx, top, ch_top, selected_color, bg);
                }
                let by = top + h_cells.saturating_sub(1);
                if by < CANVAS_HEIGHT {
                    canvas.set_char_with_bg(cx, by, ch_top, selected_color, bg);
                }
            }
        }
        for cy in top..(top + h_cells) {
            if cy < CANVAS_HEIGHT {
                if left < CANVAS_WIDTH {
                    canvas.set_char_with_bg(left, cy, ch_side, selected_color, bg);
                }
                let bx = left + w_cells.saturating_sub(1);
                if bx < CANVAS_WIDTH {
                    canvas.set_char_with_bg(bx, cy, ch_side, selected_color, bg);
                }
            }
        }
    }

    // 当前选中颜色块（2×2 = 4 cell），与调色板分隔
    canvas.fill_rect_with_bg(
        SELECTED_BOX_X,
        SELECTED_BOX_Y,
        SELECTED_BOX_SIZE,
        SELECTED_BOX_SIZE,
        ' ',
        theme.text.primary,
        selected_color,
    );

    // 调色板：每色块 2×2 cell；悬浮时高亮边框
    let palette_highlight = theme.semantic.info;
    for (idx, PaletteEntry(_name, color)) in palette.iter().enumerate() {
        let slot_row = idx / PALETTE_SLOT_COLS;
        let slot_col = idx % PALETTE_SLOT_COLS;
        let x = PALETTE_GRID_X + slot_col * PALETTE_SLOT_SIZE;
        let y = PALETTE_GRID_Y + slot_row * PALETTE_SLOT_SIZE;
        let (ch, fg) = if editor.selected_palette_index == idx {
            ('●', theme.text.primary)
        } else {
            (' ', theme.text.muted)
        };
        canvas.fill_rect_with_bg(x, y, PALETTE_SLOT_SIZE, PALETTE_SLOT_SIZE, ch, fg, *color);
        if editor.hovered_palette_index == Some(idx) {
            for i in 0..PALETTE_SLOT_SIZE {
                canvas.set_char_with_bg(x + i, y, '─', palette_highlight, *color);
                canvas.set_char_with_bg(x + i, y + PALETTE_SLOT_SIZE - 1, '─', palette_highlight, *color);
            }
            for i in 0..PALETTE_SLOT_SIZE {
                canvas.set_char_with_bg(x, y + i, '│', palette_highlight, *color);
                canvas.set_char_with_bg(x + PALETTE_SLOT_SIZE - 1, y + i, '│', palette_highlight, *color);
            }
        }
    }

    // 画笔行：调色板下方空两行，一排 1×1～8×8，左数字右图标（选中 ✓ / 未选 ○）
    let brush_row_bg = theme.bg.obsidian_moss;
    for size in 1..=BRUSH_SIZES {
        let col = (size - 1) * BRUSH_ITEM_WIDTH;
        let x_num = BRUSH_ROW_X + col;
        let x_icon = BRUSH_ROW_X + col + 1;
        let selected = editor.brush_size == size;
        let (icon_ch, icon_fg) = if selected {
            ('✓', theme.semantic.success) // U+2713
        } else {
            ('○', theme.text.muted) // U+25CB
        };
        let num_str = size.to_string();
        canvas.set_char_with_bg(x_num, BRUSH_ROW_Y, num_str.chars().next().unwrap(), theme.text.primary, brush_row_bg);
        canvas.set_char_with_bg(x_icon, BRUSH_ROW_Y, icon_ch, icon_fg, brush_row_bg);
    }

    // 按钮
    let btn_bg = theme.bg.obsidian_moss;
    canvas.set_line_with_bg(
        BTN_EXPORT_Y,
        BTN_X_START,
        BTN_X_END,
        " 导出 ",
        TextAlign::Left,
        theme.text.primary,
        btn_bg,
    );
    canvas.set_line_with_bg(
        BTN_IMPORT_Y,
        BTN_X_START,
        BTN_X_END,
        " 导入 ",
        TextAlign::Left,
        theme.text.primary,
        btn_bg,
    );
    canvas.set_line_with_bg(
        BTN_BACK_Y,
        BTN_X_START,
        BTN_X_END,
        " 返回 ",
        TextAlign::Left,
        theme.text.primary,
        btn_bg,
    );

    editor.mark_drawn();
}
