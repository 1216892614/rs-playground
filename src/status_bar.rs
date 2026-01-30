//! 状态栏插件：画布最下方两行固定区域，包含网格开关、悬停文本滚动、设置按钮，按钮支持 hover/click 变色。
//! 通过 CellHoverEvent / CellPressEvent / CellReleaseEvent 实现悬浮与点击。

use bevy::prelude::*;
use crate::canvas::{
    CellHoverEvent, CellPressEvent, CellReleaseEvent, Canvas, TextAlign, CANVAS_WIDTH,
};
use crate::AppSet;
use crate::AppState;

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

// ==================== 外部悬浮文本（主菜单等设置，状态栏只读，不绑定 string 事件） ====================

/// 其它插件（如主菜单）设置的悬浮描述；状态栏在非自身按钮悬浮时显示此项，不读画布 string 的悬浮事件。
#[derive(Resource, Default)]
pub struct StatusBarExternalHoverText(pub Option<String>);

// ==================== 状态栏按钮 ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatusBarButton {
    /// 主菜单时：像素编辑器入口；像素编辑页时：返回
    Left,
    Settings,
}

impl StatusBarButton {
    fn cell(&self) -> (usize, usize) {
        match self {
            StatusBarButton::Left => (GRID_TOGGLE_X, STATUS_ROW_BOTTOM),
            StatusBarButton::Settings => (SETTINGS_BUTTON_X, STATUS_ROW_BOTTOM),
        }
    }

    /// 悬浮在按钮上时状态栏显示的完整描述（不依赖画布事件的 text）
    fn hover_label(&self, app_state: &AppState) -> &'static str {
        match self {
            StatusBarButton::Left => match app_state {
                AppState::MainMenu => "像素编辑器",
                AppState::PixelEditor => "返回主菜单",
            },
            StatusBarButton::Settings => "设置",
        }
    }
}

// ==================== 状态栏状态 ====================

#[derive(Resource)]
struct StatusBarState {
    hovered: Option<StatusBarButton>,
    pressed: Option<StatusBarButton>,
    /// 释放时在 Left 上则本帧消费：主菜单进入像素编辑器，像素编辑页返回主菜单
    pending_left_click: bool,
    /// 悬停位置的完整文本（用于滚动展示）
    hover_text: String,
    scroll_offset: usize,
    scroll_timer: f32,
    /// 上一帧状态，用于仅在有变化时重绘
    prev_hovered: Option<StatusBarButton>,
    prev_pressed: Option<StatusBarButton>,
    prev_hover_text: String,
    prev_scroll_offset: usize,
    prev_grid_visible: bool,
    prev_app_state: Option<AppState>,
}

impl Default for StatusBarState {
    fn default() -> Self {
        Self {
            hovered: None,
            pressed: None,
            pending_left_click: false,
            hover_text: String::new(),
            scroll_offset: 0,
            scroll_timer: 0.0,
            prev_hovered: None,
            prev_pressed: None,
            prev_hover_text: String::new(),
            prev_scroll_offset: 0,
            prev_grid_visible: false,
            prev_app_state: None,
        }
    }
}

// ==================== Plugin ====================

pub struct StatusBarPlugin;

impl Plugin for StatusBarPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StatusBarState::default())
            .insert_resource(StatusBarExternalHoverText::default())
            .add_systems(
                Update,
                (
                    status_bar_cell_events.in_set(AppSet::StatusBarCell),
                    status_bar_scroll_tick,
                    status_bar_draw,
                )
                    .chain(),
            );
    }
}

// ==================== 格子到状态栏按钮 ====================

fn cell_to_status_bar_button(x: usize, y: usize) -> Option<StatusBarButton> {
    if y != STATUS_ROW_BOTTOM {
        return None;
    }
    if x == GRID_TOGGLE_X {
        return Some(StatusBarButton::Left);
    }
    if x == SETTINGS_BUTTON_X {
        return Some(StatusBarButton::Settings);
    }
    None
}

// ==================== 系统：响应画布格子事件（悬浮文本 + 按钮 hover/click） ====================

fn status_bar_cell_events(
    app_state: Res<State<AppState>>,
    mut state: ResMut<StatusBarState>,
    external: Res<StatusBarExternalHoverText>,
    mut ev_hover: EventReader<CellHoverEvent>,
    mut ev_press: EventReader<CellPressEvent>,
    mut ev_release: EventReader<CellReleaseEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for ev in ev_hover.read() {
        state.hovered = ev.cell.and_then(|(x, y)| cell_to_status_bar_button(x, y));
        state.hover_text = match state.hovered {
            Some(btn) => btn.hover_label(app_state.get()).to_string(),
            None => external.0.clone().unwrap_or_default(),
        };
    }
    for ev in ev_press.read() {
        state.pressed = cell_to_status_bar_button(ev.x, ev.y);
    }
    for ev in ev_release.read() {
        if let Some(btn) = state.pressed {
            if cell_to_status_bar_button(ev.x, ev.y) == Some(btn) {
                match btn {
                    StatusBarButton::Left => state.pending_left_click = true,
                    StatusBarButton::Settings => {
                        // TODO: 打开设置页
                    }
                }
            }
            state.pressed = None;
        }
    }
    if state.pending_left_click {
        state.pending_left_click = false;
        match app_state.get() {
            AppState::MainMenu => next_state.set(AppState::PixelEditor),
            AppState::PixelEditor => next_state.set(AppState::MainMenu),
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
    app_state: Res<State<AppState>>,
    mut canvas: ResMut<Canvas>,
    mut state: ResMut<StatusBarState>,
    theme: Res<crate::theme::Theme>,
) {
    let grid_visible = canvas.grid_visible();
    let current_state = app_state.get().clone();
    let changed = state.hovered != state.prev_hovered
        || state.pressed != state.prev_pressed
        || state.hover_text != state.prev_hover_text
        || state.scroll_offset != state.prev_scroll_offset
        || grid_visible != state.prev_grid_visible
        || state.prev_app_state.as_ref() != Some(&current_state);
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

    // 第 53 行：左 1 格 = 像素编辑器/返回，1..95 = 滚动文本，95 = 设置
    let (gx, gy) = StatusBarButton::Left.cell();
    let left_label = match app_state.get() {
        AppState::MainMenu => "像",
        AppState::PixelEditor => "返",
    };
    canvas.set_line_with_bg(
        gy,
        gx,
        gx + 1,
        left_label,
        TextAlign::Center,
        theme.text.primary,
        button_bg(StatusBarButton::Left),
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
    state.prev_hover_text = state.hover_text.clone();
    state.prev_scroll_offset = state.scroll_offset;
    state.prev_grid_visible = grid_visible;
    state.prev_app_state = Some(current_state);
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
    if len <= take {
        // 内容能完整显示：只显示一次，不循环重复
        for &c in &chars {
            out.push(c);
        }
        // 右侧用空格填满
        for _ in len..take {
            out.push(PAD);
        }
    } else {
        // 内容超出宽度：按 offset 滚动，循环显示
        for i in 0..take {
            let idx = (offset + i) % len;
            out.push(chars[idx]);
        }
    }
    out.push_str(&pad_str);
    out
}
