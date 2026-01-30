//! 状态栏插件：画布最下方两行固定区域，包含网格开关、悬停文本滚动、设置按钮，按钮支持 hover/click 变色。

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::canvas::{Canvas, TextAlign, CANVAS_HEIGHT, CANVAS_WIDTH};

// ==================== 常量 ====================

/// 状态栏占用的最下方两行（0-based）
const STATUS_ROW_TOP: usize = 52;
const STATUS_ROW_BOTTOM: usize = 53;

/// 最底行布局
const GRID_TOGGLE_X: usize = 0;
const TEXT_AREA_X_START: usize = 1;
const TEXT_AREA_X_END: usize = 95; // 不包含 95，即 1..95 共 94 格
const SETTINGS_BUTTON_X: usize = 95;

/// 滚动区域可显示的大致字符数（留左右全角空格）
const SCROLL_PADDING_FULLWIDTH: usize = 2;
const SCROLL_SPEED_TICK: f32 = 0.05;

// ==================== 状态栏按钮 ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatusBarButton {
    GridToggle,
    Settings,
}

impl StatusBarButton {
    fn cell(&self) -> (usize, usize) {
        match self {
            StatusBarButton::GridToggle => (GRID_TOGGLE_X, STATUS_ROW_BOTTOM),
            StatusBarButton::Settings => (SETTINGS_BUTTON_X, STATUS_ROW_BOTTOM),
        }
    }
}

// ==================== 状态栏状态 ====================

#[derive(Resource)]
struct StatusBarState {
    hovered: Option<StatusBarButton>,
    pressed: Option<StatusBarButton>,
    /// 释放时在 GridToggle 上则本帧消费并切换网格
    pending_grid_toggle: bool,
    /// 悬停位置的完整文本（用于滚动展示）
    hover_text: String,
    scroll_offset: usize,
    scroll_timer: f32,
    /// 上一帧状态，用于仅在有变化时重绘
    prev_hovered: Option<StatusBarButton>,
    prev_pressed: Option<StatusBarButton>,
    prev_scroll_offset: usize,
    prev_grid_visible: bool,
}

impl Default for StatusBarState {
    fn default() -> Self {
        Self {
            hovered: None,
            pressed: None,
            pending_grid_toggle: false,
            hover_text: String::new(),
            scroll_offset: 0,
            scroll_timer: 0.0,
            prev_hovered: None,
            prev_pressed: None,
            prev_scroll_offset: 0,
            prev_grid_visible: false, // 与 canvas 初始 true 不同，确保首帧会重绘
        }
    }
}

// ==================== Plugin ====================

pub struct StatusBarPlugin;

impl Plugin for StatusBarPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StatusBarState::default())
            .add_systems(
                Update,
                (
                    status_bar_cursor_and_click,
                    status_bar_scroll_tick,
                    status_bar_draw,
                )
                    .chain(),
            );
    }
}

// ==================== 光标转画布格子 ====================

fn cursor_to_canvas_cell(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    canvas: &Canvas,
) -> Option<(usize, usize)> {
    let cursor = window.cursor_position()?;
    let world = camera.viewport_to_world_2d(camera_transform, cursor).ok()?;
    let cell_size = canvas.cell_size();
    let cw = CANVAS_WIDTH as f32 * cell_size;
    let ch = CANVAS_HEIGHT as f32 * cell_size;
    let origin_x = -cw / 2.0;
    let origin_y = ch / 2.0;

    let cell_x = ((world.x - origin_x) / cell_size).floor() as i32;
    let cell_y = ((origin_y - world.y) / cell_size).floor() as i32;

    if cell_x >= 0 && cell_x < CANVAS_WIDTH as i32 && cell_y >= 0 && cell_y < CANVAS_HEIGHT as i32 {
        Some((cell_x as usize, cell_y as usize))
    } else {
        None
    }
}

fn cell_to_button(x: usize, y: usize) -> Option<StatusBarButton> {
    if y != STATUS_ROW_BOTTOM {
        return None;
    }
    if x == GRID_TOGGLE_X {
        return Some(StatusBarButton::GridToggle);
    }
    if x == SETTINGS_BUTTON_X {
        return Some(StatusBarButton::Settings);
    }
    None
}

// ==================== 系统：光标与点击 ====================

fn status_bar_cursor_and_click(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    canvas: Res<Canvas>,
    mut state: ResMut<StatusBarState>,
    mouse_btn: Res<ButtonInput<MouseButton>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let (x, y) = match cursor_to_canvas_cell(window, camera, camera_transform, &canvas) {
        Some(c) => c,
        None => {
            state.hovered = None;
            if !mouse_btn.pressed(MouseButton::Left) {
                state.pressed = None;
            }
            return;
        }
    };

    state.hovered = cell_to_button(x, y);

    if mouse_btn.just_pressed(MouseButton::Left) {
        state.pressed = state.hovered;
    }
    if mouse_btn.just_released(MouseButton::Left) {
        if let Some(btn) = state.pressed {
            if state.hovered == Some(btn) {
                match btn {
                    StatusBarButton::GridToggle => state.pending_grid_toggle = true,
                    StatusBarButton::Settings => {
                        // TODO: 打开设置页
                    }
                }
            }
            state.pressed = None;
        }
    }
}

// ==================== 系统：滚动计时 ====================

fn status_bar_scroll_tick(time: Res<Time>, mut state: ResMut<StatusBarState>) {
    state.scroll_timer += time.delta_secs();
    if state.scroll_timer >= SCROLL_SPEED_TICK {
        state.scroll_timer = 0.0;
        let len = state.hover_text.chars().count();
        if len > 0 {
            state.scroll_offset = (state.scroll_offset + 1) % len.max(1);
        }
    }
}


// ==================== 系统：绘制状态栏 ====================

fn status_bar_draw(
    mut canvas: ResMut<Canvas>,
    mut state: ResMut<StatusBarState>,
    theme: Res<crate::theme::Theme>,
) {
    if state.pending_grid_toggle {
        let visible = canvas.grid_visible();
        canvas.set_grid_visible(!visible);
        state.pending_grid_toggle = false;
    }

    let grid_visible = canvas.grid_visible();
    let changed = state.hovered != state.prev_hovered
        || state.pressed != state.prev_pressed
        || state.scroll_offset != state.prev_scroll_offset
        || grid_visible != state.prev_grid_visible;
    if !changed {
        return;
    }

    let _cell_size = canvas.cell_size();
    let normal_bg = theme.bg.basalt_blue;
    let hover_bg = theme.semantic.info.with_alpha(0.4);
    let press_bg = theme.semantic.info.with_alpha(0.6);

    let button_bg = |btn: StatusBarButton| -> Color {
        if state.pressed == Some(btn) {
            press_bg
        } else if state.hovered == Some(btn) {
            hover_bg
        } else {
            normal_bg
        }
    };

    // 清空最底两行再重绘
    canvas.clear_rect(0, STATUS_ROW_TOP, CANVAS_WIDTH, 2);

    // 第 52 行：预留操作说明/状态（可留空或占位）
    canvas.set_line(
        STATUS_ROW_TOP,
        0,
        CANVAS_WIDTH,
        "",
        TextAlign::Left,
        theme.text.muted,
    );

    // 第 53 行：左 1 格 = 网格开关，1..95 = 滚动文本，95 = 设置
    let (gx, gy) = StatusBarButton::GridToggle.cell();
    let grid_label = if canvas.grid_visible() { "线" } else { "关" };
    canvas.set_line_with_bg(
        gy,
        gx,
        gx + 1,
        grid_label,
        TextAlign::Center,
        theme.text.primary,
        button_bg(StatusBarButton::GridToggle),
    );

    let scroll_content = scroll_text_for_display(
        &state.hover_text,
        state.scroll_offset,
        TEXT_AREA_X_END - TEXT_AREA_X_START,
    );
    canvas.set_line_with_bg(
        STATUS_ROW_BOTTOM,
        TEXT_AREA_X_START,
        TEXT_AREA_X_END,
        &scroll_content,
        TextAlign::Left,
        theme.text.secondary,
        theme.bg.obsidian_moss,
    );

    let (sx, sy) = StatusBarButton::Settings.cell();
    canvas.set_line_with_bg(
        sy,
        sx,
        sx + 1,
        "设",
        TextAlign::Center,
        theme.text.primary,
        button_bg(StatusBarButton::Settings),
    );

    state.prev_hovered = state.hovered;
    state.prev_pressed = state.pressed;
    state.prev_scroll_offset = state.scroll_offset;
    state.prev_grid_visible = canvas.grid_visible();
}

fn scroll_text_for_display(content: &str, offset: usize, max_cells: usize) -> String {
    const PAD: char = '\u{3000}'; // 全角空格
    let pad_str = PAD.to_string().repeat(SCROLL_PADDING_FULLWIDTH);
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    if len == 0 {
        return format!("{}{}", pad_str, pad_str);
    }
    let take = max_cells.saturating_sub(SCROLL_PADDING_FULLWIDTH * 2);
    let mut out = String::with_capacity(max_cells + 4);
    out.push_str(&pad_str);
    for i in 0..take {
        let idx = (offset + i) % len;
        out.push(chars[idx]);
    }
    out.push_str(&pad_str);
    out
}
