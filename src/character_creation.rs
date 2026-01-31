//! 新建人物界面：分步角色创建（基础属性 → 出身 → 能力 → 物品），横向 Stepper 指示，完成一步可进入下一步。
//! 逻辑：character_creation_cell_events；渲染：character_creation_draw。

use bevy::input::keyboard::{KeyCode, KeyboardInput};
use bevy::prelude::*;

use crate::canvas::{CellHoverEvent, CellReleaseEvent, Canvas, TextAlign};
use crate::router::NavigatePop;
use crate::save::NewGameInProgress;
use crate::status_bar::StatusBarExternalHoverText;
use crate::AppState;

// ==================== 常量 ====================

/// 状态栏占用第 52–53 行，内容与按钮均不占用该区域
const STATUS_BAR_TOP: usize = 52;

/// 页面标题位置
const TITLE_X: usize = 2;
const TITLE_Y: usize = 2;

/// 返回按钮：界面右上角（不给状态栏留位时也避免与标题重叠）
const BACK_TOP_Y: usize = 2;
const BACK_TOP_X_START: usize = 90;
const BACK_TOP_X_END: usize = 96;

/// Stepper 区域：中间偏上，当前 step 名称 + 横向图标条
const STEPPER_ROW_Y: usize = 4;
/// 当前 step 名称显示在 stepper 左侧
const STEP_NAME_X_START: usize = 2;
const STEP_NAME_X_END: usize = 24;
/// Stepper 图标条：水平居中，4 个 step 图标
const STEPPER_ICONS_X_START: usize = 28;
const STEPPER_ICON_SPACING: usize = 4;
const NUM_STEPS: usize = 4;

/// 名字输入行（名字 + 输入框区域）
const NAME_ROW_Y: usize = 6;
const NAME_LABEL_X_START: usize = 4;
const NAME_LABEL_X_END: usize = 8;
const NAME_INPUT_X_START: usize = 8;
const NAME_INPUT_X_END: usize = 50;
const MAX_NAME_LEN: usize = 20;

/// 内容区域（当前 step 的表单/说明），均在 STATUS_BAR_TOP 之上
const CONTENT_ROW_START: usize = 8;
const CONTENT_X_START: usize = 4;
const CONTENT_X_END: usize = 92;

// ---------- 基础属性步骤 UI ----------
/// 总点数 30，单属性创建时最高 15，上限 20
const BASE_ATTR_POINTS_TOTAL: u8 = 30;
const BASE_ATTR_MAX_AT_CREATION: u8 = 15;
const BASE_ATTR_MAX: u8 = 20;

/// 说明行、三行属性（执行/反应/习性）
const BASE_ATTR_ROW_DESC: usize = 8;
const BASE_ATTR_ROW_EXEC: usize = 10;
const BASE_ATTR_ROW_REACT: usize = 12;
const BASE_ATTR_ROW_HABIT: usize = 14;

const BASE_ATTR_NAME_X: usize = 4;
const BASE_ATTR_MINUS_X_START: usize = 10;
const BASE_ATTR_MINUS_X_END: usize = 12;
const BASE_ATTR_VAL_X: usize = 14;
const BASE_ATTR_MOD_X_START: usize = 18;
const BASE_ATTR_MOD_X_END: usize = 28;
const BASE_ATTR_PLUS_X_START: usize = 30;
const BASE_ATTR_PLUS_X_END: usize = 32;

/// 底部按钮行（上一步/下一步），位于状态栏之上
const BUTTON_ROW_Y: usize = 46;
const BTN_PREV_X_START: usize = 2;
const BTN_PREV_X_END: usize = 18;
const BTN_NEXT_X_START: usize = 22;
const BTN_NEXT_X_END: usize = 46;

/// 图层
const LAYER_CONTENT: i32 = 0;
const LAYER_HOVER: i32 = 1;

// Step 图标（Nerd Font）
const ICON_ATTR: char = '\u{f007}';   // user → 基础属性
const ICON_ORIGIN: char = '\u{f015}'; // home → 出身
const ICON_ABILITY: char = '\u{f0e7}'; // bolt → 能力
const ICON_ITEM: char = '\u{f466}';   // box → 物品

// ==================== 基础属性 ====================

/// 调整值 = (属性值 - 10) / 2（整数除法）
fn base_attr_modifier(value: u8) -> i32 {
    (i32::from(value) - 10) / 2
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BaseAttrKind {
    Execution, // 执行
    Reaction,  // 反应
    Habit,     // 习性
}

impl BaseAttrKind {
    fn name(self) -> &'static str {
        match self {
            BaseAttrKind::Execution => "执行",
            BaseAttrKind::Reaction => "反应",
            BaseAttrKind::Habit => "习性",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BaseAttrButton {
    Minus(BaseAttrKind),
    Plus(BaseAttrKind),
}

fn base_attr_row(btn: BaseAttrButton) -> usize {
    match btn {
        BaseAttrButton::Minus(k) | BaseAttrButton::Plus(k) => match k {
            BaseAttrKind::Execution => BASE_ATTR_ROW_EXEC,
            BaseAttrKind::Reaction => BASE_ATTR_ROW_REACT,
            BaseAttrKind::Habit => BASE_ATTR_ROW_HABIT,
        },
    }
}

fn cell_to_base_attr_button(x: usize, y: usize) -> Option<BaseAttrButton> {
    if y == BASE_ATTR_ROW_EXEC {
        if x >= BASE_ATTR_MINUS_X_START && x < BASE_ATTR_MINUS_X_END {
            return Some(BaseAttrButton::Minus(BaseAttrKind::Execution));
        }
        if x >= BASE_ATTR_PLUS_X_START && x < BASE_ATTR_PLUS_X_END {
            return Some(BaseAttrButton::Plus(BaseAttrKind::Execution));
        }
    }
    if y == BASE_ATTR_ROW_REACT {
        if x >= BASE_ATTR_MINUS_X_START && x < BASE_ATTR_MINUS_X_END {
            return Some(BaseAttrButton::Minus(BaseAttrKind::Reaction));
        }
        if x >= BASE_ATTR_PLUS_X_START && x < BASE_ATTR_PLUS_X_END {
            return Some(BaseAttrButton::Plus(BaseAttrKind::Reaction));
        }
    }
    if y == BASE_ATTR_ROW_HABIT {
        if x >= BASE_ATTR_MINUS_X_START && x < BASE_ATTR_MINUS_X_END {
            return Some(BaseAttrButton::Minus(BaseAttrKind::Habit));
        }
        if x >= BASE_ATTR_PLUS_X_START && x < BASE_ATTR_PLUS_X_END {
            return Some(BaseAttrButton::Plus(BaseAttrKind::Habit));
        }
    }
    None
}

// ==================== Step 定义 ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CreationStep {
    BaseAttr,  // 基础属性
    Origin,    // 出身
    Ability,   // 能力
    Item,      // 物品
}

impl CreationStep {
    fn index(self) -> usize {
        match self {
            CreationStep::BaseAttr => 0,
            CreationStep::Origin => 1,
            CreationStep::Ability => 2,
            CreationStep::Item => 3,
        }
    }
    fn from_index(i: usize) -> Self {
        match i {
            0 => CreationStep::BaseAttr,
            1 => CreationStep::Origin,
            2 => CreationStep::Ability,
            _ => CreationStep::Item,
        }
    }
    fn name(self) -> &'static str {
        match self {
            CreationStep::BaseAttr => "基础属性",
            CreationStep::Origin => "出身",
            CreationStep::Ability => "能力",
            CreationStep::Item => "物品",
        }
    }
    fn icon(self) -> char {
        match self {
            CreationStep::BaseAttr => ICON_ATTR,
            CreationStep::Origin => ICON_ORIGIN,
            CreationStep::Ability => ICON_ABILITY,
            CreationStep::Item => ICON_ITEM,
        }
    }
    fn next(self) -> Option<Self> {
        match self {
            CreationStep::BaseAttr => Some(CreationStep::Origin),
            CreationStep::Origin => Some(CreationStep::Ability),
            CreationStep::Ability => Some(CreationStep::Item),
            CreationStep::Item => None,
        }
    }
    fn prev(self) -> Option<Self> {
        match self {
            CreationStep::BaseAttr => None,
            CreationStep::Origin => Some(CreationStep::BaseAttr),
            CreationStep::Ability => Some(CreationStep::Origin),
            CreationStep::Item => Some(CreationStep::Ability),
        }
    }
}

// ==================== 状态 ====================

#[derive(Resource)]
struct CharacterCreationState {
    current_step: usize,
    /// 基础属性：执行、反应、习性（创建时各 0..=15，总和 30）
    attr_execution: u8,
    attr_reaction: u8,
    attr_habit: u8,
    /// 名字输入缓冲；与 NewGameInProgress.characters[0].name 同步
    name_edit_buffer: String,
    /// 是否正在编辑名字（键盘输入写入 name_edit_buffer）
    editing_name: bool,
    prev_app_state: Option<AppState>,
    prev_step: usize,
    prev_attr_execution: u8,
    prev_attr_reaction: u8,
    prev_attr_habit: u8,
    prev_name_edit_buffer: String,
    prev_editing_name: bool,
    prev_hover_prev: bool,
    prev_hover_next: bool,
    prev_hover_back: bool,
    prev_hover_name: bool,
    prev_hover_base_attr: Option<BaseAttrButton>,
    hover_prev: bool,
    hover_next: bool,
    hover_back: bool,
    hover_name: bool,
    hover_base_attr: Option<BaseAttrButton>,
}

impl CharacterCreationState {
    fn base_attr_used(&self) -> u8 {
        self.attr_execution + self.attr_reaction + self.attr_habit
    }
    fn base_attr_remaining(&self) -> u8 {
        BASE_ATTR_POINTS_TOTAL.saturating_sub(self.base_attr_used())
    }
    fn attr_for_kind_mut(&mut self, k: BaseAttrKind) -> &mut u8 {
        match k {
            BaseAttrKind::Execution => &mut self.attr_execution,
            BaseAttrKind::Reaction => &mut self.attr_reaction,
            BaseAttrKind::Habit => &mut self.attr_habit,
        }
    }
}

impl Default for CharacterCreationState {
    fn default() -> Self {
        Self {
            current_step: 0,
            attr_execution: 10,
            attr_reaction: 10,
            attr_habit: 10,
            name_edit_buffer: "HAJIMEHERO".to_string(),
            editing_name: false,
            prev_app_state: None,
            prev_step: 0,
            prev_attr_execution: 10,
            prev_attr_reaction: 10,
            prev_attr_habit: 10,
            prev_name_edit_buffer: String::new(),
            prev_editing_name: false,
            prev_hover_prev: false,
            prev_hover_next: false,
            prev_hover_back: false,
            prev_hover_name: false,
            prev_hover_base_attr: None,
            hover_prev: false,
            hover_next: false,
            hover_back: false,
            hover_name: false,
            hover_base_attr: None,
        }
    }
}

// ==================== 交互区域判断 ====================

fn cell_in_prev_button(x: usize, y: usize) -> bool {
    y == BUTTON_ROW_Y && x >= BTN_PREV_X_START && x < BTN_PREV_X_END
}

fn cell_in_next_button(x: usize, y: usize) -> bool {
    y == BUTTON_ROW_Y && x >= BTN_NEXT_X_START && x < BTN_NEXT_X_END
}

fn cell_in_back_button(x: usize, y: usize) -> bool {
    y == BACK_TOP_Y && x >= BACK_TOP_X_START && x < BACK_TOP_X_END
}

fn cell_in_name_input(x: usize, y: usize) -> bool {
    y == NAME_ROW_Y && x >= NAME_INPUT_X_START && x < NAME_INPUT_X_END
}

// ==================== Plugin ====================

pub struct CharacterCreationPlugin;

fn character_creation_enter(
    mut state: ResMut<CharacterCreationState>,
    new_game: Res<NewGameInProgress>,
) {
    state.current_step = 0;
    state.editing_name = false;
    if let Some(ref save) = (*new_game).0 {
        if let Some(c) = save.characters.first() {
            state.attr_execution = c.attr_execution;
            state.attr_reaction = c.attr_reaction;
            state.attr_habit = c.attr_habit;
            state.name_edit_buffer = c.name.clone();
            state.prev_app_state = None;
            state.prev_step = 0;
            state.prev_attr_execution = c.attr_execution;
            state.prev_attr_reaction = c.attr_reaction;
            state.prev_attr_habit = c.attr_habit;
            state.prev_name_edit_buffer = c.name.clone();
            return;
        }
    }
    state.attr_execution = 10;
    state.attr_reaction = 10;
    state.attr_habit = 10;
    state.name_edit_buffer = "HAJIMEHERO".to_string();
    state.prev_app_state = None;
    state.prev_step = 0;
    state.prev_attr_execution = 10;
    state.prev_attr_reaction = 10;
    state.prev_attr_habit = 10;
}

impl Plugin for CharacterCreationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CharacterCreationState::default())
            .add_systems(OnEnter(AppState::CharacterCreation), character_creation_enter)
            .add_systems(
                Update,
                (
                    character_creation_keyboard.run_if(in_state(AppState::CharacterCreation)),
                    character_creation_cell_events.run_if(in_state(AppState::CharacterCreation)),
                ),
            )
            .add_systems(
                Update,
                character_creation_draw.run_if(in_state(AppState::CharacterCreation)),
            );
    }
}

// ==================== 系统：键盘（名字输入） ====================

fn character_creation_keyboard(
    app_state: Res<State<AppState>>,
    mut state: ResMut<CharacterCreationState>,
    mut new_game: ResMut<NewGameInProgress>,
    mut ev_keyboard: MessageReader<KeyboardInput>,
) {
    if *app_state.get() != AppState::CharacterCreation || !state.editing_name {
        return;
    }
    for ev in ev_keyboard.read() {
        use bevy::input::ButtonState;
        if ev.state != ButtonState::Pressed {
            continue;
        }
        if ev.key_code == KeyCode::Backspace {
            state.name_edit_buffer.pop();
            continue;
        }
        if ev.key_code == KeyCode::Enter || ev.key_code == KeyCode::NumpadEnter {
            state.editing_name = false;
            commit_name_to_character(&state, &mut *new_game);
            continue;
        }
        if ev.key_code == KeyCode::Escape {
            state.editing_name = false;
            if let Some(ref save) = (*new_game).0 {
                if let Some(c) = save.characters.first() {
                    state.name_edit_buffer = c.name.clone();
                }
            }
            continue;
        }
        if let Some(ref text) = ev.text {
            for c in text.chars() {
                if state.name_edit_buffer.chars().count() < MAX_NAME_LEN && !c.is_control() {
                    state.name_edit_buffer.push(c);
                }
            }
        }
    }
}

// ==================== 系统：格子事件 ====================

fn sync_state_to_initial_character(
    state: &CharacterCreationState,
    new_game: &mut NewGameInProgress,
) {
    if let Some(ref mut save) = (*new_game).0 {
        if let Some(c) = save.characters.get_mut(0) {
            c.attr_execution = state.attr_execution;
            c.attr_reaction = state.attr_reaction;
            c.attr_habit = state.attr_habit;
            c.name = state.name_edit_buffer.clone();
        }
    }
}

fn commit_name_to_character(state: &CharacterCreationState, new_game: &mut NewGameInProgress) {
    if let Some(ref mut save) = (*new_game).0 {
        if let Some(c) = save.characters.get_mut(0) {
            c.name = state.name_edit_buffer.clone();
        }
    }
}

fn character_creation_cell_events(
    app_state: Res<State<AppState>>,
    mut state: ResMut<CharacterCreationState>,
    mut new_game: ResMut<NewGameInProgress>,
    mut external: ResMut<StatusBarExternalHoverText>,
    mut ev_hover: MessageReader<CellHoverEvent>,
    mut ev_release: MessageReader<CellReleaseEvent>,
    mut ev_pop: MessageWriter<NavigatePop>,
) {
    if *app_state.get() != AppState::CharacterCreation {
        return;
    }
    for ev in ev_hover.read() {
        match ev.cell {
            None => {
                state.hover_prev = false;
                state.hover_next = false;
                state.hover_back = false;
                state.hover_name = false;
                state.hover_base_attr = None;
                external.0 = None;
            }
            Some((x, y)) => {
                state.hover_prev = cell_in_prev_button(x, y);
                state.hover_next = cell_in_next_button(x, y);
                state.hover_back = cell_in_back_button(x, y);
                state.hover_name = cell_in_name_input(x, y);
                state.hover_base_attr = if state.current_step == 0 {
                    cell_to_base_attr_button(x, y)
                } else {
                    None
                };
                external.0 = if state.hover_prev {
                    Some("上一步".to_string())
                } else if state.hover_next {
                    Some(
                        if state.current_step + 1 >= NUM_STEPS {
                            "完成".to_string()
                        } else {
                            "下一步".to_string()
                        },
                    )
                } else if state.hover_back {
                    Some("返回".to_string())
                } else if state.hover_name {
                    Some("点击修改名字".to_string())
                } else if let Some(btn) = state.hover_base_attr {
                    match btn {
                        BaseAttrButton::Minus(k) => {
                            Some(format!("{} -1", k.name()))
                        }
                        BaseAttrButton::Plus(k) => {
                            Some(format!("{} +1", k.name()))
                        }
                    }
                } else {
                    None
                };
            }
        }
    }
    for ev in ev_release.read() {
        let (x, y) = (ev.x, ev.y);
        if cell_in_name_input(x, y) {
            state.editing_name = true;
        } else if state.editing_name {
            state.editing_name = false;
            commit_name_to_character(&state, &mut *new_game);
        }
        if state.current_step == 0 {
            if let Some(btn) = cell_to_base_attr_button(x, y) {
                match btn {
                    BaseAttrButton::Minus(k) => {
                        let v = state.attr_for_kind_mut(k);
                        if *v > 0 {
                            *v = v.saturating_sub(1);
                            sync_state_to_initial_character(&state, &mut *new_game);
                        }
                    }
                    BaseAttrButton::Plus(k) => {
                        let remaining = state.base_attr_remaining();
                        let v = state.attr_for_kind_mut(k);
                        if remaining > 0 && *v < BASE_ATTR_MAX_AT_CREATION {
                            *v += 1;
                            sync_state_to_initial_character(&state, &mut *new_game);
                        }
                    }
                }
            }
        }
        if cell_in_prev_button(x, y) {
            if state.current_step > 0 {
                state.current_step -= 1;
            }
        } else if cell_in_next_button(x, y) {
            if state.current_step + 1 < NUM_STEPS {
                state.current_step += 1;
            } else {
                // TODO: 完成创建，进入游戏或返回存档页（此时 new_game.0 仍为当前存档）
            }
        } else if cell_in_back_button(x, y) {
            if state.editing_name {
                state.editing_name = false;
                commit_name_to_character(&state, &mut *new_game);
            }
            (*new_game).0 = None;
            ev_pop.write(NavigatePop);
        }
    }
}

// ==================== 系统：绘制 ====================

fn character_creation_draw(
    app_state: Res<State<AppState>>,
    mut canvas: ResMut<Canvas>,
    mut state: ResMut<CharacterCreationState>,
    theme: Res<crate::theme::Theme>,
) {
    if *app_state.get() != AppState::CharacterCreation {
        return;
    }
    let current = app_state.get().clone();
    if current != AppState::CharacterCreation {
        state.prev_app_state = Some(current);
        return;
    }
    let just_entered = state.prev_app_state != Some(AppState::CharacterCreation);
    state.prev_app_state = Some(AppState::CharacterCreation);

    let base_attr_changed = state.current_step == 0
        && (state.attr_execution != state.prev_attr_execution
            || state.attr_reaction != state.prev_attr_reaction
            || state.attr_habit != state.prev_attr_habit);
    let name_changed = state.name_edit_buffer != state.prev_name_edit_buffer
        || state.editing_name != state.prev_editing_name;
    let content_changed = just_entered
        || state.current_step != state.prev_step
        || base_attr_changed
        || name_changed;
    let hover_changed = state.hover_prev != state.prev_hover_prev
        || state.hover_next != state.prev_hover_next
        || state.hover_back != state.prev_hover_back
        || state.hover_name != state.prev_hover_name
        || state.hover_base_attr != state.prev_hover_base_attr;

    if !content_changed && !hover_changed {
        state.prev_step = state.current_step;
        state.prev_attr_execution = state.attr_execution;
        state.prev_attr_reaction = state.attr_reaction;
        state.prev_attr_habit = state.attr_habit;
        state.prev_name_edit_buffer = state.name_edit_buffer.clone();
        state.prev_editing_name = state.editing_name;
        state.prev_hover_prev = state.hover_prev;
        state.prev_hover_next = state.hover_next;
        state.prev_hover_back = state.hover_back;
        state.prev_hover_name = state.hover_name;
        state.prev_hover_base_attr = state.hover_base_attr;
        return;
    }

    let text_color = theme.text.primary;
    let dim_color = theme.text.secondary;
    let accent = theme.semantic.info;
    let hover_bg = theme.semantic.info.with_alpha(0.4);

    if content_changed {
        canvas.clear_layer(LAYER_CONTENT);
        canvas.clear_layer(LAYER_HOVER);

        // 标题（左上角）
        canvas.set_line_layer(
            LAYER_CONTENT,
            TITLE_Y,
            TITLE_X,
            TITLE_X + 8,
            "新建人物",
            TextAlign::Left,
            text_color,
        );

        // 返回：界面右上角
        canvas.set_line_layer(
            LAYER_CONTENT,
            BACK_TOP_Y,
            BACK_TOP_X_START,
            BACK_TOP_X_END,
            "返回",
            TextAlign::Right,
            text_color,
        );

        // 名字输入行
        canvas.set_line_layer(
            LAYER_CONTENT,
            NAME_ROW_Y,
            NAME_LABEL_X_START,
            NAME_LABEL_X_END,
            "名字",
            TextAlign::Left,
            text_color,
        );
        let name_trunc: String = state
            .name_edit_buffer
            .chars()
            .take(NAME_INPUT_X_END - NAME_INPUT_X_START)
            .collect();
        let name_display = if name_trunc.is_empty() && !state.editing_name {
            "—".to_string()
        } else if state.editing_name {
            format!("{}_", name_trunc)
        } else {
            name_trunc.clone()
        };
        canvas.set_line_layer(
            LAYER_CONTENT,
            NAME_ROW_Y,
            NAME_INPUT_X_START,
            NAME_INPUT_X_END,
            &name_display,
            TextAlign::Left,
            if state.editing_name { accent } else { text_color },
        );

        // 当前 step 名称
        let step = CreationStep::from_index(state.current_step);
        canvas.set_line_layer(
            LAYER_CONTENT,
            STEPPER_ROW_Y,
            STEP_NAME_X_START,
            STEP_NAME_X_END,
            step.name(),
            TextAlign::Left,
            text_color,
        );

        // Stepper 图标条
        let mut x = STEPPER_ICONS_X_START;
        for i in 0..NUM_STEPS {
            let s = CreationStep::from_index(i);
            let is_current = i == state.current_step;
            let color = if is_current { accent } else { dim_color };
            canvas.set_char_layer(LAYER_CONTENT, x, STEPPER_ROW_Y, s.icon(), color);
            x += 1;
            x += STEPPER_ICON_SPACING;
        }

        // 内容区：基础属性步骤显示三属性与 +/- 按钮，其余步骤占位
        if state.current_step == 0 {
            let used = state.base_attr_used();
            let remaining = state.base_attr_remaining();
            canvas.set_line_layer(
                LAYER_CONTENT,
                BASE_ATTR_ROW_DESC,
                CONTENT_X_START,
                CONTENT_X_START + 24,
                &format!("剩余 {} 点未分配", remaining),
                TextAlign::Left,
                accent,
            );
            canvas.set_line_layer(
                LAYER_CONTENT,
                BASE_ATTR_ROW_DESC,
                CONTENT_X_START + 26,
                CONTENT_X_END,
                &format!(
                    "已用{}/{} 单属性最高{} 调整值=(属性-10)/2",
                    used,
                    BASE_ATTR_POINTS_TOTAL,
                    BASE_ATTR_MAX_AT_CREATION
                ),
                TextAlign::Left,
                dim_color,
            );
            for (row, kind) in [
                (BASE_ATTR_ROW_EXEC, BaseAttrKind::Execution),
                (BASE_ATTR_ROW_REACT, BaseAttrKind::Reaction),
                (BASE_ATTR_ROW_HABIT, BaseAttrKind::Habit),
            ] {
                let val = match kind {
                    BaseAttrKind::Execution => state.attr_execution,
                    BaseAttrKind::Reaction => state.attr_reaction,
                    BaseAttrKind::Habit => state.attr_habit,
                };
                let mod_str = base_attr_modifier(val);
                let mod_display = if mod_str >= 0 {
                    format!("+{}", mod_str)
                } else {
                    mod_str.to_string()
                };
                canvas.set_line_layer(
                    LAYER_CONTENT,
                    row,
                    BASE_ATTR_NAME_X,
                    BASE_ATTR_NAME_X + 4,
                    kind.name(),
                    TextAlign::Left,
                    text_color,
                );
                canvas.set_char_layer(LAYER_CONTENT, BASE_ATTR_MINUS_X_START, row, '−', text_color);
                canvas.set_line_layer(
                    LAYER_CONTENT,
                    row,
                    BASE_ATTR_VAL_X,
                    BASE_ATTR_VAL_X + 2,
                    &format!("{}", val),
                    TextAlign::Left,
                    text_color,
                );
                canvas.set_line_layer(
                    LAYER_CONTENT,
                    row,
                    BASE_ATTR_MOD_X_START,
                    BASE_ATTR_MOD_X_END,
                    &format!("调整{}", mod_display),
                    TextAlign::Left,
                    dim_color,
                );
                canvas.set_char_layer(LAYER_CONTENT, BASE_ATTR_PLUS_X_START, row, '+', text_color);
            }
        } else {
            let placeholder = format!("请在此配置「{}」……", step.name());
            canvas.set_line_layer(
                LAYER_CONTENT,
                CONTENT_ROW_START,
                CONTENT_X_START,
                CONTENT_X_END,
                &placeholder,
                TextAlign::Left,
                dim_color,
            );
        }

        // 底部按钮行（上一步/下一步），不占用状态栏区域
        canvas.set_line_layer(
            LAYER_CONTENT,
            BUTTON_ROW_Y,
            BTN_PREV_X_START,
            BTN_PREV_X_END,
            "上一步",
            TextAlign::Left,
            if state.current_step > 0 {
                text_color
            } else {
                dim_color
            },
        );
        let next_label = if state.current_step + 1 >= NUM_STEPS {
            "完成"
        } else {
            "下一步"
        };
        canvas.set_line_layer(
            LAYER_CONTENT,
            BUTTON_ROW_Y,
            BTN_NEXT_X_START,
            BTN_NEXT_X_END,
            next_label,
            TextAlign::Left,
            text_color,
        );
    }

    // Hover 高亮
    canvas.clear_layer(LAYER_HOVER);
    if state.hover_prev && state.current_step > 0 {
        canvas.set_line_with_bg_layer(
            LAYER_HOVER,
            BUTTON_ROW_Y,
            BTN_PREV_X_START,
            BTN_PREV_X_END,
            "上一步",
            TextAlign::Left,
            text_color,
            hover_bg,
        );
    }
    if state.hover_next {
        let next_label = if state.current_step + 1 >= NUM_STEPS {
            "完成"
        } else {
            "下一步"
        };
        canvas.set_line_with_bg_layer(
            LAYER_HOVER,
            BUTTON_ROW_Y,
            BTN_NEXT_X_START,
            BTN_NEXT_X_END,
            next_label,
            TextAlign::Left,
            text_color,
            hover_bg,
        );
    }
    if state.hover_back {
        canvas.set_line_with_bg_layer(
            LAYER_HOVER,
            BACK_TOP_Y,
            BACK_TOP_X_START,
            BACK_TOP_X_END,
            "返回",
            TextAlign::Right,
            text_color,
            hover_bg,
        );
    }
    if state.hover_name {
        let mut hover_name_str: String = state
            .name_edit_buffer
            .chars()
            .take(NAME_INPUT_X_END - NAME_INPUT_X_START)
            .collect();
        if state.editing_name {
            hover_name_str.push('_');
        }
        if hover_name_str.is_empty() {
            hover_name_str = "—".to_string();
        }
        canvas.set_line_with_bg_layer(
            LAYER_HOVER,
            NAME_ROW_Y,
            NAME_INPUT_X_START,
            NAME_INPUT_X_END,
            &hover_name_str,
            TextAlign::Left,
            if state.editing_name { accent } else { text_color },
            hover_bg,
        );
    }
    if state.current_step == 0 {
        if let Some(btn) = state.hover_base_attr {
            let row = base_attr_row(btn);
            match btn {
                BaseAttrButton::Minus(_) => {
                    canvas.set_char_with_bg_layer(
                        LAYER_HOVER,
                        BASE_ATTR_MINUS_X_START,
                        row,
                        '−',
                        text_color,
                        hover_bg,
                    );
                }
                BaseAttrButton::Plus(_) => {
                    canvas.set_char_with_bg_layer(
                        LAYER_HOVER,
                        BASE_ATTR_PLUS_X_START,
                        row,
                        '+',
                        text_color,
                        hover_bg,
                    );
                }
            }
        }
    }

    state.prev_step = state.current_step;
    state.prev_attr_execution = state.attr_execution;
    state.prev_attr_reaction = state.attr_reaction;
    state.prev_attr_habit = state.attr_habit;
    state.prev_name_edit_buffer = state.name_edit_buffer.clone();
    state.prev_editing_name = state.editing_name;
    state.prev_hover_prev = state.hover_prev;
    state.prev_hover_next = state.hover_next;
    state.prev_hover_back = state.hover_back;
    state.prev_hover_name = state.hover_name;
    state.prev_hover_base_attr = state.hover_base_attr;
}
