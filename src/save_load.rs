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
/// 节点标签起始 x（时间轴区域收窄，右侧留给分叉列表）
const LABEL_X_START: usize = 6;
const LABEL_X_END: usize = 36;

/// 标题位置；标题右侧可显示当前分叉信息（非主线时）
const TITLE_X: usize = 2;
const TITLE_Y: usize = 2;
const TITLE_FORK_INFO_X: usize = 38; // 分叉于[时间][章节][#序号] 起始 x

/// 右侧分叉列表（从当前选中存档分出的其他 fork），标题在上方一行
const FORK_LIST_X_START: usize = 40;
const FORK_LIST_X_END: usize = 76;
const FORK_LIST_ROW_START: usize = 6;
const FORK_LIST_ROW_END: usize = 40;
const FORK_LIST_TITLE_ROW: usize = 4;
/// 每个分叉项占 2 行
const FORK_ITEM_HEIGHT: usize = 2;

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

// ==================== 分支（树状存档） ====================

/// 从某存档分出的分支的元信息；主线无此信息
#[derive(Clone, Debug)]
struct ForkMeta {
    /// 分叉时间显示，如 "01:15:00"
    pub time: String,
    /// 分叉时章节，如 "第一章"
    pub chapter: String,
    /// 分叉序号
    pub seq: usize,
}

/// 一条分支：主线或从某节点分出的 fork
#[derive(Clone, Debug)]
struct Branch {
    pub timeline: Vec<TimelineNode>,
    /// 若为 fork，则表示从哪条分支的哪个节点分出
    pub fork_from: Option<(usize, usize)>,
    pub fork_meta: Option<ForkMeta>,
}

fn default_branches() -> Vec<Branch> {
    let main_nodes = default_timeline_nodes();
    let mut branches = vec![Branch {
        timeline: main_nodes.clone(),
        fork_from: None,
        fork_meta: None,
    }];
    // 示例：从主线第 5 个节点分出的 fork，沿用前 6 个节点后接一段新进度
    let mut fork_nodes: Vec<TimelineNode> = main_nodes.iter().take(6).cloned().collect();
    for i in 6..=16usize {
        let t_min = 25 + i * 4;
        let h = t_min / 60;
        let m = t_min % 60;
        let duration_display = format!("{:02}:{:02}:00", h, m);
        let ch = (i.saturating_sub(1)) / 4 + 1;
        let chapter_name = format!("第{}章", to_chinese_num(ch));
        let is_chapter = i % 4 == 0;
        fork_nodes.push(TimelineNode {
            is_chapter,
            chapter_name,
            duration_display,
        });
    }
    branches.push(Branch {
        timeline: fork_nodes,
        fork_from: Some((0, 5)),
        fork_meta: Some(ForkMeta {
            time: "01:15:00".to_string(),
            chapter: "第一章".to_string(),
            seq: 1,
        }),
    });
    branches
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

/// 时间轴上节点对应的 cell：左侧点/竖线 (x<=4) 或右侧标题 (LABEL_X_START..LABEL_X_END) 均可触发；y 对应节点 idx = target + (y - CENTER) / NODE_SPACING
fn cell_to_timeline_node_index(
    x: usize,
    y: usize,
    nodes: &[TimelineNode],
    target: usize,
) -> Option<usize> {
    let in_left = x <= 4;
    let in_label = x >= LABEL_X_START && x < LABEL_X_END;
    if !in_left && !in_label {
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

/// 当前选中节点分出的 fork 的 branch_id 列表（不含父分支）
fn fork_branch_ids_from(branches: &[Branch], from_branch_id: usize, from_node: usize) -> Vec<usize> {
    branches
        .iter()
        .enumerate()
        .filter(|(_, b)| b.fork_from == Some((from_branch_id, from_node)))
        .map(|(id, _)| id)
        .collect()
}

/// 右侧列表应显示的 branch_id：在分叉点包含父分支（主线），其余为从该节点分出的 fork；不包含当前分支自己
fn list_branch_ids(state: &SaveLoadState) -> Vec<usize> {
    let branches = &state.branches;
    let cur = state.current_branch_id;
    let target = state.target;
    let ids: Vec<usize> = if let Some((parent_id, from_node)) = state.current_branch().fork_from {
        if target == from_node {
            let mut list = vec![parent_id];
            list.extend(fork_branch_ids_from(branches, parent_id, from_node));
            list
        } else {
            fork_branch_ids_from(branches, cur, target)
        }
    } else {
        fork_branch_ids_from(branches, cur, target)
    };
    ids.into_iter().filter(|&id| id != cur).collect()
}

/// 右侧分叉列表：点击区域 (x,y) 对应列表中的第几项（考虑滚动）；不在列表内返回 None
fn cell_to_fork_list_index(
    x: usize,
    y: usize,
    fork_count: usize,
    fork_list_offset: usize,
) -> Option<usize> {
    if x < FORK_LIST_X_START || x >= FORK_LIST_X_END {
        return None;
    }
    if y < FORK_LIST_ROW_START || y > FORK_LIST_ROW_END {
        return None;
    }
    let row = y - FORK_LIST_ROW_START;
    let item = row / FORK_ITEM_HEIGHT;
    let idx = fork_list_offset + item;
    if idx < fork_count {
        Some(idx)
    } else {
        None
    }
}

// ==================== 状态 ====================

#[derive(Resource)]
struct SaveLoadState {
    /// 所有分支（主线 + 各 fork）
    branches: Vec<Branch>,
    /// 当前查看的分支 id
    current_branch_id: usize,
    /// 当前分支的时间轴（= branches[current_branch_id].timeline）
    nodes: Vec<TimelineNode>,
    target: usize,
    view: f32,
    /// 右侧分叉列表滚动偏移（第几项开始显示）
    fork_list_offset: usize,
    hovered_control: Option<SaveLoadControlButton>,
    hovered_continue: bool,
    hovered_back: bool,
    hovered_node: Option<usize>,
    /// 右侧分叉列表中悬浮的项（列表内索引，非 branch_id）
    hovered_fork_index: Option<usize>,
    prev_app_state: Option<AppState>,
    prev_target: usize,
    prev_branch_id: usize,
    prev_fork_list_offset: usize,
    prev_hovered_control: Option<SaveLoadControlButton>,
    prev_hovered_continue: bool,
    prev_hovered_back: bool,
    prev_hovered_node: Option<usize>,
    prev_hovered_fork_index: Option<usize>,
}

impl Default for SaveLoadState {
    fn default() -> Self {
        let branches = default_branches();
        let current_branch_id = 0;
        let nodes = branches[0].timeline.clone();
        let last = nodes.len().saturating_sub(1);
        Self {
            branches,
            current_branch_id,
            nodes,
            target: last,
            view: last as f32,
            fork_list_offset: 0,
            hovered_control: None,
            hovered_continue: false,
            hovered_back: false,
            hovered_node: None,
            hovered_fork_index: None,
            prev_app_state: None,
            prev_target: last,
            prev_branch_id: 0,
            prev_fork_list_offset: 0,
            prev_hovered_control: None,
            prev_hovered_continue: false,
            prev_hovered_back: false,
            prev_hovered_node: None,
            prev_hovered_fork_index: None,
        }
    }
}

impl SaveLoadState {
    /// 当前分支引用
    fn current_branch(&self) -> &Branch {
        &self.branches[self.current_branch_id]
    }
    /// 是否为主线（第一个分支）
    fn is_main_branch(&self) -> bool {
        self.current_branch_id == 0
    }
    /// 右侧列表应显示的 branch_id（分叉点含父分支，否则为该节点分出的 fork）
    fn list_branch_ids(&self) -> Vec<usize> {
        list_branch_ids(self)
    }
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
    /// 切换到指定分支，并定位到该分支最新进度（不更新 prev_*，由绘制帧末更新，以便本帧触发重绘）
    fn switch_to_branch(&mut self, branch_id: usize) {
        if branch_id >= self.branches.len() || branch_id == self.current_branch_id {
            return;
        }
        self.current_branch_id = branch_id;
        self.nodes = self.branches[branch_id].timeline.clone();
        let last = self.nodes.len().saturating_sub(1);
        self.target = last;
        self.view = last as f32;
    }
}

// ==================== Plugin ====================

pub struct SaveLoadPlugin;

fn save_load_enter_latest(mut state: ResMut<SaveLoadState>) {
    let last = state.nodes.len().saturating_sub(1);
    state.target = last;
    state.view = last as f32;
    state.prev_target = last;
    state.prev_branch_id = state.current_branch_id;
    // 使本帧绘制认为“刚进入”，触发完整重绘（否则从主菜单返回时 prev_app_state 仍为 SaveLoad 会跳过重绘）
    state.prev_app_state = None;
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
                state.hovered_fork_index = None;
                external.0 = None;
            }
            Some((x, y)) => {
                let fork_ids = state.list_branch_ids();
                let fork_count = fork_ids.len();
                let fork_idx = cell_to_fork_list_index(x, y, fork_count, state.fork_list_offset);
                if let Some(btn) = cell_to_control_button(x, y) {
                    state.hovered_control = Some(btn);
                    state.hovered_continue = false;
                    state.hovered_back = false;
                    state.hovered_node = None;
                    state.hovered_fork_index = None;
                    external.0 = Some(btn.hover_label().to_string());
                } else if cell_to_continue_button(x, y) {
                    state.hovered_control = None;
                    state.hovered_continue = true;
                    state.hovered_back = false;
                    state.hovered_node = None;
                    state.hovered_fork_index = None;
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
                    state.hovered_fork_index = None;
                    external.0 = Some("返回主菜单".to_string());
                } else if let Some(idx) = cell_to_timeline_node_index(x, y, &state.nodes, state.target)
                {
                    state.hovered_control = None;
                    state.hovered_continue = false;
                    state.hovered_back = false;
                    state.hovered_node = Some(idx);
                    state.hovered_fork_index = None;
                    external.0 = Some(state.nodes[idx].label());
                } else if let Some(fi) = fork_idx {
                    state.hovered_control = None;
                    state.hovered_continue = false;
                    state.hovered_back = false;
                    state.hovered_node = None;
                    state.hovered_fork_index = Some(fi);
                    let bid = fork_ids[fi];
                    let b = &state.branches[bid];
                    external.0 = Some(
                        b.fork_meta
                            .as_ref()
                            .map(|m| format!("切换到 [{}] #{}", m.time, m.seq))
                            .unwrap_or_else(|| "切换到 [00:00:00] 序章 #0".to_string()),
                    );
                } else {
                    state.hovered_control = None;
                    state.hovered_continue = false;
                    state.hovered_back = false;
                    state.hovered_node = None;
                    state.hovered_fork_index = None;
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
        } else {
            let fork_ids = state.list_branch_ids();
            let fork_count = fork_ids.len();
            if let Some(fi) = cell_to_fork_list_index(x, y, fork_count, state.fork_list_offset) {
                if fi < fork_count {
                    state.switch_to_branch(fork_ids[fi]);
                }
            }
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

    let content_changed = just_entered
        || state.target != state.prev_target
        || state.current_branch_id != state.prev_branch_id
        || state.fork_list_offset != state.prev_fork_list_offset;
    let timeline_hover_changed = state.hovered_node != state.prev_hovered_node;
    let controller_hover_changed = state.hovered_control != state.prev_hovered_control
        || state.hovered_continue != state.prev_hovered_continue
        || state.hovered_back != state.prev_hovered_back
        || state.hovered_fork_index != state.prev_hovered_fork_index;

    if !content_changed && !timeline_hover_changed && !controller_hover_changed {
        state.prev_target = state.target;
        state.prev_branch_id = state.current_branch_id;
        state.prev_fork_list_offset = state.fork_list_offset;
        state.prev_hovered_control = state.hovered_control;
        state.prev_hovered_continue = state.hovered_continue;
        state.prev_hovered_back = state.hovered_back;
        state.prev_hovered_node = state.hovered_node;
        state.prev_hovered_fork_index = state.hovered_fork_index;
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
    state.prev_branch_id = state.current_branch_id;
    state.prev_fork_list_offset = state.fork_list_offset;
    state.prev_hovered_control = state.hovered_control;
    state.prev_hovered_continue = state.hovered_continue;
    state.prev_hovered_back = state.hovered_back;
    state.prev_hovered_node = state.hovered_node;
    state.prev_hovered_fork_index = state.hovered_fork_index;
}

/// 时间轴图层：标题（含分叉信息）、竖线、节点；右侧分叉列表
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
        TITLE_X + 8,
        "存档页面",
        TextAlign::Left,
        text_color,
    );
    let fork_info = state.current_branch().fork_meta.as_ref().map(|m| {
        format!("分叉与 [{}] {} [#{}]", m.time, m.chapter, m.seq)
    }).unwrap_or_else(|| "分叉与 [00:00:00] 序章 [#0]".to_string());
    canvas.set_line_layer(
        TIMELINE_CONTENT_LAYER,
        TITLE_Y,
        TITLE_FORK_INFO_X,
        FORK_LIST_X_END,
        &fork_info,
        TextAlign::Left,
        dim_color,
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
        let has_forks = !fork_branch_ids_from(&state.branches, state.current_branch_id, idx).is_empty()
            || state.current_branch().fork_from.map(|(_, from_node)| from_node == idx).unwrap_or(false);
        if has_forks {
            canvas.set_char_layer(TIMELINE_CONTENT_LAYER, TIMELINE_X.saturating_sub(1), y, '◇', dim_color);
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
    let fork_ids = state.list_branch_ids();
    if !fork_ids.is_empty() {
        canvas.set_line_layer(
            TIMELINE_CONTENT_LAYER,
            FORK_LIST_TITLE_ROW,
            FORK_LIST_X_START,
            FORK_LIST_X_END,
            "分叉",
            TextAlign::Left,
            dim_color,
        );
    }
    for (list_idx, &branch_id) in fork_ids.iter().skip(state.fork_list_offset).enumerate() {
        let row0 = FORK_LIST_ROW_START + list_idx * FORK_ITEM_HEIGHT;
        if row0 > FORK_LIST_ROW_END {
            break;
        }
        let branch = &state.branches[branch_id];
        let (line1, line2) = match &branch.fork_meta {
            Some(m) => (format!("[{}] {}", m.time, m.chapter), format!("#{}", m.seq)),
            None => ("[00:00:00] 序章".to_string(), "#0".to_string()),
        };
        canvas.set_line_layer(
            TIMELINE_CONTENT_LAYER,
            row0,
            FORK_LIST_X_START,
            FORK_LIST_X_END,
            &line1,
            TextAlign::Left,
            dim_color,
        );
        if row0 + 1 <= FORK_LIST_ROW_END {
            canvas.set_line_layer(
                TIMELINE_CONTENT_LAYER,
                row0 + 1,
                FORK_LIST_X_START,
                FORK_LIST_X_END,
                &line2,
                TextAlign::Left,
                dim_color,
            );
        }
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

/// 控制器 hover 层：控制按钮、继续按钮、分叉列表项高亮
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
    if let Some(fork_idx) = state.hovered_fork_index {
        let row0 = FORK_LIST_ROW_START
            .saturating_add(
                fork_idx.saturating_sub(state.fork_list_offset) * FORK_ITEM_HEIGHT,
            );
        if row0 <= FORK_LIST_ROW_END {
            canvas.fill_background_rect_layer(
                CONTROLLER_HOVER_LAYER,
                FORK_LIST_X_START,
                row0,
                FORK_LIST_X_END.saturating_sub(FORK_LIST_X_START),
                FORK_ITEM_HEIGHT.min(FORK_LIST_ROW_END - row0 + 1),
                hover_bg,
            );
        }
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
