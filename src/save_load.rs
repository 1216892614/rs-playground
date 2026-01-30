//! 开始游戏界面：竖式时间轴存档/章节选择，滚轮与点击切换节点，弹簧插值动画，播放器式控制按钮。

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::canvas::{
    CellHoverEvent, CellPressEvent, CellReleaseEvent, Canvas, TextAlign, CANVAS_WIDTH,
};
use crate::status_bar::StatusBarExternalHoverText;
use crate::AppSet;
use crate::AppState;

/// 时间轴与控制器拆分到不同图层，避免重绘互相覆盖 hover
/// 时间轴内容（标题、竖线、节点）
const TIMELINE_CONTENT_LAYER: i32 = 0;
/// 时间轴 hover 高亮（仅时间轴节点悬浮时更新）
const TIMELINE_HOVER_LAYER: i32 = 1;
/// 控制器内容（控制按钮行、继续按钮、章节名）
const CONTROLLER_CONTENT_LAYER: i32 = 2;
/// 控制器 hover 高亮（仅控制/继续按钮悬浮时更新）
const CONTROLLER_HOVER_LAYER: i32 = 3;

// ==================== 常量 ====================

/// 时间轴距离左边 2 格
const TIMELINE_X: usize = 2;
/// 时间轴可视区域（选中节点固定在此区域中间）
const TIMELINE_VIEW_TOP: usize = 6;
const TIMELINE_VIEW_BOTTOM: usize = 40;
/// 选中节点所在行（时间轴“东”/中部）
const TIMELINE_CENTER_Y: usize = (TIMELINE_VIEW_TOP + TIMELINE_VIEW_BOTTOM) / 2; // 23
/// 节点之间相隔 4 格
const NODE_SPACING: usize = 4;
/// 节点标签起始 x
const LABEL_X_START: usize = 6;
const LABEL_X_END: usize = 90;

/// 标题位置
const TITLE_X: usize = 2;
const TITLE_Y: usize = 2;

/// 控制按钮行（游戏开始、上一章、上一个存档、下一个存档、下一章、最新进度）
const CONTROLS_ROW_Y: usize = 44;
const CONTROLS_X_START: usize = 2;
/// 每个控制按钮占 3 格，图标在中间一格
const CONTROL_BTN_WIDTH: usize = 3;
const NUM_CONTROL_BTNS: usize = 6; // prev save, next save, prev chapter, next chapter, start, end
const CONTROLS_X_END: usize = CONTROLS_X_START + NUM_CONTROL_BTNS * CONTROL_BTN_WIDTH;

/// 继续游戏/开始游戏按钮（与控制行隔一行）
const CONTINUE_ROW_Y: usize = 46;
const CONTINUE_X_START: usize = 2;
const CONTINUE_X_END: usize = 28;

/// 当前章节+时长显示（右下角）
const CHAPTER_LABEL_ROW_Y: usize = 48;
const CHAPTER_LABEL_X_START: usize = 60;
const CHAPTER_LABEL_X_END: usize = 94;

/// 右上角返回按钮
const BACK_BTN_Y: usize = 2;
const BACK_BTN_X_START: usize = 92;
const BACK_BTN_X_END: usize = 96;

/// 状态栏上方内容区域
const STATUS_ROW_TOP: usize = 52;

/// 弹簧刚度（view 趋近 target 的速度）
const SPRING_STIFFNESS: f32 = 12.0;
const SPRING_DAMPING: f32 = 0.7;

// 控制按钮图标：游戏开始/最新进度用 step-backward/forward，上下存档用左右三角，上下章用 PUA
const ICON_START: char = '\u{f048}';         // step-backward → 游戏开始
const ICON_LATEST: char = '\u{f051}';        // step-forward → 最新进度
const ICON_PREV_SAVE: char = '\u{25c0}';    // ◀ 左三角 → 上一个存档
const ICON_NEXT_SAVE: char = '\u{25b6}';    // ▶ 右三角 → 下一个存档
const ICON_PREV_CHAPTER: char = '\u{f045f}'; // 上一章 (U+F045F，来自 \udb81\udc5f)
const ICON_NEXT_CHAPTER: char = '\u{f0211}'; // 下一章 (U+F0211，来自 \udb80\ude11)
const ICON_CONTINUE: char = '\u{f04b}';     // play

// ==================== 时间轴节点 ====================

#[derive(Clone, Debug)]
pub struct TimelineNode {
    /// 是否为章节自动保存点（用于上一章/下一章跳转）
    pub is_chapter: bool,
    /// 所属章节名，每个存档都有（如 "序章"、"第一章"）
    pub chapter_name: String,
    /// 保存时的游戏时长显示，如 "01:23:45"
    pub duration_display: String,
}

impl TimelineNode {
    /// 时间轴上：章节保存点显示第几章，其他存档显示游戏时长
    fn label(&self) -> String {
        if self.is_chapter {
            self.chapter_name.clone()
        } else {
            self.duration_display.clone()
        }
    }
}

fn default_timeline_nodes() -> Vec<TimelineNode> {
    let mut nodes = Vec::with_capacity(32);
    nodes.push(TimelineNode {
        is_chapter: true,
        chapter_name: "序章".to_string(),
        duration_display: "00:00:00".to_string(),
    });
    for i in 1..=24 {
        let t_min = 5 + i * 3;
        let h = t_min / 60;
        let m = t_min % 60;
        let duration_display = format!("{:02}:{:02}:00", h, m);
        let ch = (i - 1) / 4 + 1;
        let chapter_name = format!("第{}章", to_chinese_num(ch));
        let is_chapter = i % 4 == 0;
        nodes.push(TimelineNode {
            is_chapter,
            chapter_name,
            duration_display,
        });
    }
    nodes
}

fn to_chinese_num(n: usize) -> String {
    const DIGITS: [&str; 10] = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    if n < 10 {
        DIGITS[n].to_string()
    } else if n < 20 {
        format!("十{}", if n > 10 { DIGITS[n - 10] } else { "" })
    } else {
        n.to_string()
    }
}

// ==================== 控制按钮 ====================

/// 控制栏从左到右：游戏开始、上一章、上一个存档、下一个存档、下一章、最新进度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SaveLoadControlButton {
    Start,       // 游戏开始
    PrevChapter, // 上一章
    PrevSave,    // 上一个存档
    NextSave,    // 下一个存档
    NextChapter, // 下一章
    Latest,      // 最新进度
}

impl SaveLoadControlButton {
    fn icon(&self) -> char {
        match self {
            SaveLoadControlButton::Start => ICON_START,
            SaveLoadControlButton::PrevChapter => ICON_PREV_CHAPTER,
            SaveLoadControlButton::PrevSave => ICON_PREV_SAVE,
            SaveLoadControlButton::NextSave => ICON_NEXT_SAVE,
            SaveLoadControlButton::NextChapter => ICON_NEXT_CHAPTER,
            SaveLoadControlButton::Latest => ICON_LATEST,
        }
    }
    fn hover_label(&self) -> &'static str {
        match self {
            SaveLoadControlButton::Start => "游戏开始",
            SaveLoadControlButton::PrevChapter => "上一章",
            SaveLoadControlButton::PrevSave => "上一个存档",
            SaveLoadControlButton::NextSave => "下一个存档",
            SaveLoadControlButton::NextChapter => "下一章",
            SaveLoadControlButton::Latest => "最新进度",
        }
    }
    fn cell_range(&self) -> (usize, usize) {
        let i = match self {
            SaveLoadControlButton::Start => 0,
            SaveLoadControlButton::PrevChapter => 1,
            SaveLoadControlButton::PrevSave => 2,
            SaveLoadControlButton::NextSave => 3,
            SaveLoadControlButton::NextChapter => 4,
            SaveLoadControlButton::Latest => 5,
        };
        let x = CONTROLS_X_START + i * CONTROL_BTN_WIDTH;
        (x, x + CONTROL_BTN_WIDTH)
    }
}

fn cell_to_control_button(x: usize, y: usize) -> Option<SaveLoadControlButton> {
    if y != CONTROLS_ROW_Y {
        return None;
    }
    if x < CONTROLS_X_START || x >= CONTROLS_X_END {
        return None;
    }
    let i = (x - CONTROLS_X_START) / CONTROL_BTN_WIDTH;
    match i {
        0 => Some(SaveLoadControlButton::Start),
        1 => Some(SaveLoadControlButton::PrevChapter),
        2 => Some(SaveLoadControlButton::PrevSave),
        3 => Some(SaveLoadControlButton::NextSave),
        4 => Some(SaveLoadControlButton::NextChapter),
        5 => Some(SaveLoadControlButton::Latest),
        _ => None,
    }
}

fn cell_to_continue_button(x: usize, y: usize) -> bool {
    y == CONTINUE_ROW_Y && x >= CONTINUE_X_START && x < CONTINUE_X_END
}

fn cell_to_back_button(x: usize, y: usize) -> bool {
    y == BACK_BTN_Y && x >= BACK_BTN_X_START && x < BACK_BTN_X_END
}

/// 时间轴上节点对应的 cell：选中节点固定在 TIMELINE_CENTER_Y，y 对应节点 idx = target + (y - CENTER) / NODE_SPACING
fn cell_to_timeline_node_index(
    x: usize,
    y: usize,
    nodes: &[TimelineNode],
    target: usize,
) -> Option<usize> {
    if x > 4 {
        return None;
    }
    if y < TIMELINE_VIEW_TOP || y > TIMELINE_VIEW_BOTTOM {
        return None;
    }
    let dy = y as i32 - TIMELINE_CENTER_Y as i32;
    if dy % NODE_SPACING as i32 != 0 {
        return None;
    }
    let offset = dy / NODE_SPACING as i32;
    let idx = target as i32 + offset;
    if idx >= 0 && idx < nodes.len() as i32 {
        Some(idx as usize)
    } else {
        None
    }
}

// ==================== 状态 ====================

#[derive(Resource)]
struct SaveLoadState {
    nodes: Vec<TimelineNode>,
    target: usize,
    view: f32,
    hovered_control: Option<SaveLoadControlButton>,
    hovered_continue: bool,
    hovered_back: bool,
    hovered_node: Option<usize>,
    prev_app_state: Option<AppState>,
    prev_target: usize,
    prev_hovered_control: Option<SaveLoadControlButton>,
    prev_hovered_continue: bool,
    prev_hovered_back: bool,
    prev_hovered_node: Option<usize>,
}

impl Default for SaveLoadState {
    fn default() -> Self {
        let nodes = default_timeline_nodes();
        let last = nodes.len().saturating_sub(1);
        Self {
            nodes,
            target: last,
            view: last as f32,
            hovered_control: None,
            hovered_continue: false,
            hovered_back: false,
            hovered_node: None,
            prev_app_state: None,
            prev_target: last,
            prev_hovered_control: None,
            prev_hovered_continue: false,
            prev_hovered_back: false,
            prev_hovered_node: None,
        }
    }
}

impl SaveLoadState {
    /// 右下角：当前选中存档的「第几章 + 时长」
    fn current_chapter_and_duration(&self) -> String {
        self.nodes
            .get(self.target)
            .map(|n| format!("{} {}", n.chapter_name, n.duration_display))
            .unwrap_or_else(|| "—".to_string())
    }
    /// 是否为最新进度（只有最新进度显示「继续游戏」）
    fn is_latest_progress(&self) -> bool {
        self.nodes.len() > 0 && self.target == self.nodes.len() - 1
    }
}

// ==================== Plugin ====================

pub struct SaveLoadPlugin;

fn save_load_enter_latest(mut state: ResMut<SaveLoadState>) {
    let last = state.nodes.len().saturating_sub(1);
    state.target = last;
    state.view = last as f32;
    state.prev_target = last;
}

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SaveLoadState::default())
            .add_systems(OnEnter(AppState::SaveLoad), save_load_enter_latest)
            .add_systems(
                Update,
                save_load_cell_events
                    .in_set(AppSet::SaveLoadCell)
                    .run_if(in_state(AppState::SaveLoad)),
            )
            .add_systems(
                Update,
                (
                    save_load_spring,
                    save_load_scroll,
                    save_load_draw,
                )
                    .run_if(in_state(AppState::SaveLoad)),
            );
    }
}

// ==================== 系统：弹簧插值 ====================

fn save_load_spring(time: Res<Time>, mut state: ResMut<SaveLoadState>) {
    let dt = time.delta_secs();
    let diff = state.target as f32 - state.view;
    state.view += diff * SPRING_STIFFNESS * dt * SPRING_DAMPING;
    if diff.abs() < 0.005 {
        state.view = state.target as f32;
    }
}

// ==================== 系统：滚轮切换节点 ====================

fn save_load_scroll(
    mut scroll_reader: EventReader<MouseWheel>,
    mut state: ResMut<SaveLoadState>,
) {
    let len = state.nodes.len().max(1);
    for ev in scroll_reader.read() {
        if ev.y > 0.0 {
            state.target = state.target.saturating_sub(1);
        } else if ev.y < 0.0 {
            state.target = (state.target + 1).min(len - 1);
        }
    }
}

// ==================== 系统：格子事件（悬浮、点击） ====================

fn save_load_cell_events(
    app_state: Res<State<AppState>>,
    mut state: ResMut<SaveLoadState>,
    mut external: ResMut<StatusBarExternalHoverText>,
    mut ev_hover: EventReader<CellHoverEvent>,
    _ev_press: EventReader<CellPressEvent>,
    mut ev_release: EventReader<CellReleaseEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if *app_state.get() != AppState::SaveLoad {
        return;
    }
    // 与主菜单一致：仅根据事件更新 hover，不在一开始清空；否则本帧未收到事件时 hover 会错误消失
    for ev in ev_hover.read() {
        match ev.cell {
            None => {
                state.hovered_control = None;
                state.hovered_continue = false;
                state.hovered_back = false;
                state.hovered_node = None;
                external.0 = None;
            }
            Some((x, y)) => {
                if let Some(btn) = cell_to_control_button(x, y) {
                    state.hovered_control = Some(btn);
                    state.hovered_continue = false;
                    state.hovered_back = false;
                    state.hovered_node = None;
                    external.0 = Some(btn.hover_label().to_string());
                } else if cell_to_continue_button(x, y) {
                    state.hovered_control = None;
                    state.hovered_continue = true;
                    state.hovered_back = false;
                    state.hovered_node = None;
                    external.0 = Some(if state.is_latest_progress() {
                        "继续游戏".to_string()
                    } else {
                        "开始游戏".to_string()
                    });
                } else if cell_to_back_button(x, y) {
                    state.hovered_control = None;
                    state.hovered_continue = false;
                    state.hovered_back = true;
                    state.hovered_node = None;
                    external.0 = Some("返回主菜单".to_string());
                } else if let Some(idx) = cell_to_timeline_node_index(x, y, &state.nodes, state.target)
                {
                    state.hovered_control = None;
                    state.hovered_continue = false;
                    state.hovered_back = false;
                    state.hovered_node = Some(idx);
                    external.0 = Some(state.nodes[idx].label());
                } else {
                    state.hovered_control = None;
                    state.hovered_continue = false;
                    state.hovered_back = false;
                    state.hovered_node = None;
                    external.0 = None;
                }
            }
        }
    }

    for ev in ev_release.read() {
        let (x, y) = (ev.x, ev.y);
        if let Some(btn) = cell_to_control_button(x, y) {
            let len = state.nodes.len().max(1);
            match btn {
                SaveLoadControlButton::Start => {
                    state.target = 0;
                }
                SaveLoadControlButton::PrevChapter => {
                    for i in (0..state.target).rev() {
                        if state.nodes[i].is_chapter {
                            state.target = i;
                            break;
                        }
                    }
                }
                SaveLoadControlButton::PrevSave => {
                    state.target = state.target.saturating_sub(1);
                }
                SaveLoadControlButton::NextSave => {
                    state.target = (state.target + 1).min(len - 1);
                }
                SaveLoadControlButton::NextChapter => {
                    for i in (state.target + 1)..len {
                        if state.nodes[i].is_chapter {
                            state.target = i;
                            break;
                        }
                    }
                }
                SaveLoadControlButton::Latest => {
                    state.target = len - 1;
                }
            }
        } else if cell_to_continue_button(x, y) {
            // TODO: 实际开始/继续游戏
        } else if cell_to_back_button(x, y) {
            next_state.set(AppState::MainMenu);
        } else if let Some(idx) = cell_to_timeline_node_index(x, y, &state.nodes, state.target) {
            state.target = idx;
        }
    }
}

// ==================== 系统：绘制 ====================

fn save_load_draw(
    app_state: Res<State<AppState>>,
    mut canvas: ResMut<Canvas>,
    mut state: ResMut<SaveLoadState>,
    theme: Res<crate::theme::Theme>,
) {
    if *app_state.get() != AppState::SaveLoad {
        return;
    }
    let current = app_state.get().clone();
    if current != AppState::SaveLoad {
        state.prev_app_state = Some(current);
        return;
    }
    let just_entered = state.prev_app_state != Some(AppState::SaveLoad);
    state.prev_app_state = Some(AppState::SaveLoad);

    let content_changed = just_entered || state.target != state.prev_target;
    let timeline_hover_changed = state.hovered_node != state.prev_hovered_node;
    let controller_hover_changed = state.hovered_control != state.prev_hovered_control
        || state.hovered_continue != state.prev_hovered_continue
        || state.hovered_back != state.prev_hovered_back;

    if !content_changed && !timeline_hover_changed && !controller_hover_changed {
        state.prev_target = state.target;
        state.prev_hovered_control = state.hovered_control;
        state.prev_hovered_continue = state.hovered_continue;
        state.prev_hovered_back = state.hovered_back;
        state.prev_hovered_node = state.hovered_node;
        return;
    }

    let text_color = theme.text.primary;
    let dim_color = theme.text.secondary;
    let accent = theme.semantic.info;
    let bg = theme.bg.basalt_blue;
    let hover_bg = theme.semantic.info.with_alpha(0.4);

    if content_changed {
        for layer in [
            TIMELINE_CONTENT_LAYER,
            TIMELINE_HOVER_LAYER,
            CONTROLLER_CONTENT_LAYER,
            CONTROLLER_HOVER_LAYER,
        ] {
            canvas.clear_rect_layer(layer, 0, 0, CANVAS_WIDTH, STATUS_ROW_TOP);
        }
        draw_timeline_content(&mut canvas, &state, &theme, text_color, dim_color, accent);
        draw_timeline_hover(&mut canvas, &state, hover_bg);
        draw_controller_content(&mut canvas, &state, &theme, text_color, dim_color, bg);
        draw_controller_hover(&mut canvas, &state, hover_bg);
    } else {
        if timeline_hover_changed {
            canvas.clear_rect_layer(TIMELINE_HOVER_LAYER, 0, 0, CANVAS_WIDTH, STATUS_ROW_TOP);
            draw_timeline_hover(&mut canvas, &state, hover_bg);
        }
        if controller_hover_changed {
            canvas.clear_rect_layer(CONTROLLER_HOVER_LAYER, 0, 0, CANVAS_WIDTH, STATUS_ROW_TOP);
            draw_controller_hover(&mut canvas, &state, hover_bg);
        }
    }

    state.prev_target = state.target;
    state.prev_hovered_control = state.hovered_control;
    state.prev_hovered_continue = state.hovered_continue;
    state.prev_hovered_back = state.hovered_back;
    state.prev_hovered_node = state.hovered_node;
}

/// 时间轴图层：标题、竖线、节点（无 hover）
fn draw_timeline_content(
    canvas: &mut Canvas,
    state: &SaveLoadState,
    _theme: &crate::theme::Theme,
    text_color: bevy::prelude::Color,
    dim_color: bevy::prelude::Color,
    accent: bevy::prelude::Color,
) {
    canvas.set_line_layer(
        TIMELINE_CONTENT_LAYER,
        TITLE_Y,
        TITLE_X,
        TITLE_X + 12,
        "[游戏存档]",
        TextAlign::Left,
        text_color,
    );
    for y in TIMELINE_VIEW_TOP..=TIMELINE_VIEW_BOTTOM {
        canvas.set_char_layer(TIMELINE_CONTENT_LAYER, TIMELINE_X, y, '│', dim_color);
    }
    for (idx, node) in state.nodes.iter().enumerate() {
        let y = (TIMELINE_CENTER_Y as i32
            + (idx as i32 - state.target as i32) * NODE_SPACING as i32) as usize;
        if y < TIMELINE_VIEW_TOP || y > TIMELINE_VIEW_BOTTOM {
            continue;
        }
        let is_selected = idx == state.target;
        let dot_ch = if is_selected { '●' } else { '◦' };
        let dot_color = if is_selected { accent } else { dim_color };
        canvas.set_char_layer(TIMELINE_CONTENT_LAYER, TIMELINE_X, y, dot_ch, dot_color);
        let label = node.label();
        let label_color = if is_selected { text_color } else { dim_color };
        canvas.set_line_layer(
            TIMELINE_CONTENT_LAYER,
            y,
            LABEL_X_START,
            LABEL_X_END,
            &label,
            TextAlign::Left,
            label_color,
        );
    }
}

/// 控制器图层：控制按钮行、继续按钮、章节名（无 hover）
fn draw_controller_content(
    canvas: &mut Canvas,
    state: &SaveLoadState,
    _theme: &crate::theme::Theme,
    text_color: bevy::prelude::Color,
    dim_color: bevy::prelude::Color,
    bg: bevy::prelude::Color,
) {
    for (i, btn) in [
        SaveLoadControlButton::Start,
        SaveLoadControlButton::PrevChapter,
        SaveLoadControlButton::PrevSave,
        SaveLoadControlButton::NextSave,
        SaveLoadControlButton::NextChapter,
        SaveLoadControlButton::Latest,
    ]
    .iter()
    .enumerate()
    {
        let x = CONTROLS_X_START + i * CONTROL_BTN_WIDTH;
        canvas.set_char_with_bg_layer(
            CONTROLLER_CONTENT_LAYER,
            x,
            CONTROLS_ROW_Y,
            ' ',
            text_color,
            bg,
        );
        canvas.set_char_with_bg_layer(
            CONTROLLER_CONTENT_LAYER,
            x + 1,
            CONTROLS_ROW_Y,
            btn.icon(),
            text_color,
            bg,
        );
        canvas.set_char_with_bg_layer(
            CONTROLLER_CONTENT_LAYER,
            x + 2,
            CONTROLS_ROW_Y,
            ' ',
            text_color,
            bg,
        );
    }
    let continue_text = if state.is_latest_progress() {
        "继续游戏"
    } else {
        "开始游戏"
    };
    canvas.set_char_with_bg_layer(
        CONTROLLER_CONTENT_LAYER,
        CONTINUE_X_START,
        CONTINUE_ROW_Y,
        ICON_CONTINUE,
        text_color,
        bg,
    );
    canvas.set_line_with_bg_layer(
        CONTROLLER_CONTENT_LAYER,
        CONTINUE_ROW_Y,
        CONTINUE_X_START + 1,
        CONTINUE_X_END,
        continue_text,
        TextAlign::Left,
        text_color,
        bg,
    );
    let chapter_and_duration = state.current_chapter_and_duration();
    canvas.set_line_layer(
        CONTROLLER_CONTENT_LAYER,
        CHAPTER_LABEL_ROW_Y,
        CHAPTER_LABEL_X_START,
        CHAPTER_LABEL_X_END,
        &chapter_and_duration,
        TextAlign::Right,
        dim_color,
    );
    canvas.set_line_with_bg_layer(
        CONTROLLER_CONTENT_LAYER,
        BACK_BTN_Y,
        BACK_BTN_X_START,
        BACK_BTN_X_END,
        "返回",
        TextAlign::Right,
        text_color,
        bg,
    );
}

/// 时间轴 hover 层：仅节点行高亮
fn draw_timeline_hover(
    canvas: &mut Canvas,
    state: &SaveLoadState,
    hover_bg: bevy::prelude::Color,
) {
    if let Some(idx) = state.hovered_node {
        let y = (TIMELINE_CENTER_Y as i32
            + (idx as i32 - state.target as i32) * NODE_SPACING as i32) as usize;
        if y >= TIMELINE_VIEW_TOP && y <= TIMELINE_VIEW_BOTTOM {
            canvas.fill_background_rect_layer(
                TIMELINE_HOVER_LAYER,
                LABEL_X_START,
                y,
                LABEL_X_END.saturating_sub(LABEL_X_START),
                1,
                hover_bg,
            );
        }
    }
}

/// 控制器 hover 层：仅控制按钮与继续按钮高亮
fn draw_controller_hover(
    canvas: &mut Canvas,
    state: &SaveLoadState,
    hover_bg: bevy::prelude::Color,
) {
    if let Some(btn) = state.hovered_control {
        let i = match btn {
            SaveLoadControlButton::Start => 0,
            SaveLoadControlButton::PrevChapter => 1,
            SaveLoadControlButton::PrevSave => 2,
            SaveLoadControlButton::NextSave => 3,
            SaveLoadControlButton::NextChapter => 4,
            SaveLoadControlButton::Latest => 5,
        };
        let x = CONTROLS_X_START + i * CONTROL_BTN_WIDTH;
        canvas.fill_background_rect_layer(
            CONTROLLER_HOVER_LAYER,
            x,
            CONTROLS_ROW_Y,
            CONTROL_BTN_WIDTH,
            1,
            hover_bg,
        );
    }
    if state.hovered_continue {
        canvas.fill_background_rect_layer(
            CONTROLLER_HOVER_LAYER,
            CONTINUE_X_START,
            CONTINUE_ROW_Y,
            CONTINUE_X_END.saturating_sub(CONTINUE_X_START),
            1,
            hover_bg,
        );
    }
    if state.hovered_back {
        canvas.fill_background_rect_layer(
            CONTROLLER_HOVER_LAYER,
            BACK_BTN_X_START,
            BACK_BTN_Y,
            BACK_BTN_X_END.saturating_sub(BACK_BTN_X_START),
            1,
            hover_bg,
        );
    }
}
