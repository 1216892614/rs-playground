//! 主界面：ASCII 标题 INTERSTICE + 带 Nerd 图标的按钮（开始游戏、设置、制作人员、离开游戏），支持 hover/click 变色。
//! 通过 CellHoverEvent / CellPressEvent / CellReleaseEvent 实现悬浮与点击。

use bevy::prelude::*;
use crate::canvas::{
    CellHoverEvent, CellPressEvent, CellReleaseEvent, Canvas, TextAlign, CANVAS_HEIGHT, CANVAS_WIDTH,
};
use crate::status_bar::StatusBarExternalHoverText;
use crate::AppSet;

// ==================== 常量 ====================

/// ASCII 艺术字：每字母 5 行 x 5 列，用实心方块绘制（# 在绘制时替换为 █）
const LETTER_W: usize = 5;
const LETTER_SPACING: usize = 1; // 字母间距
/// 标题用实心方块字符绘制
const TITLE_BLOCK: char = '\u{2588}'; // FULL BLOCK █

/// 标题 "INTERSTICE" 用实心方块拼接的艺术字
const TITLE_ART_ROW: usize = 15;

/// 按钮区域：每行 y，x 范围 [BUTTON_X_START, BUTTON_X_END)，再下移 5 格
const BUTTON_ROW_START: usize = 30; // 原 25 + 5
const BUTTON_X_START: usize = 36;
const BUTTON_X_END: usize = 62;

// 5x5 字母图案（# 表示实心，绘制时替换为 █）
const ART_I: [&str; 5] = ["#####", "  #  ", "  #  ", "  #  ", "#####"];
const ART_N: [&str; 5] = ["#   #", "##  #", "# # #", "#  ##", "#   #"];
const ART_T: [&str; 5] = ["#####", "  #  ", "  #  ", "  #  ", "  #  "];
const ART_E: [&str; 5] = ["#####", "#    ", "#####", "#    ", "#####"];
const ART_R: [&str; 5] = ["#### ", "#   #", "#### ", "#  # ", "#   #"];
const ART_S: [&str; 5] = [" ####", "#    ", " ### ", "    #", "#### "];
const ART_C: [&str; 5] = [" ####", "#    ", "#    ", "#    ", " ####"];

fn title_art_letter(c: char) -> &'static [&'static str; 5] {
    match c {
        'I' => &ART_I,
        'N' => &ART_N,
        'T' => &ART_T,
        'E' => &ART_E,
        'R' => &ART_R,
        'S' => &ART_S,
        'C' => &ART_C,
        _ => &ART_I,
    }
}

/// Nerd Font 图标 (Private Use Area)
const ICON_PLAY: char = '\u{f04b}';      // play
const ICON_COG: char = '\u{f013}';      // cog / 设置
const ICON_USERS: char = '\u{f0c0}';    // users / 制作人员
const ICON_EXIT: char = '\u{f08b}';     // sign-out / 离开

// ==================== 主菜单按钮 ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MainMenuButton {
    Start,    // 开始游戏
    Settings, // 设置
    Credits,  // 制作人员
    Quit,     // 离开游戏
}

impl MainMenuButton {
    fn row(&self) -> usize {
        match self {
            MainMenuButton::Start => BUTTON_ROW_START,
            MainMenuButton::Settings => BUTTON_ROW_START + 2,
            MainMenuButton::Credits => BUTTON_ROW_START + 4,
            MainMenuButton::Quit => BUTTON_ROW_START + 6,
        }
    }

    fn icon(&self) -> char {
        match self {
            MainMenuButton::Start => ICON_PLAY,
            MainMenuButton::Settings => ICON_COG,
            MainMenuButton::Credits => ICON_USERS,
            MainMenuButton::Quit => ICON_EXIT,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            MainMenuButton::Start => "开始游戏",
            MainMenuButton::Settings => "设置",
            MainMenuButton::Credits => "制作人员",
            MainMenuButton::Quit => "离开游戏",
        }
    }
}

// ==================== 主菜单状态 ====================

#[derive(Resource)]
struct MainMenuState {
    hovered: Option<MainMenuButton>,
    pressed: Option<MainMenuButton>,
    prev_hovered: Option<MainMenuButton>,
    prev_pressed: Option<MainMenuButton>,
}

impl Default for MainMenuState {
    fn default() -> Self {
        Self {
            hovered: None,
            pressed: None,
            prev_hovered: Some(MainMenuButton::Start), // 与 None 不同，确保首帧重绘
            prev_pressed: None,
        }
    }
}

// ==================== Plugin ====================

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MainMenuState::default())
            .add_systems(
                Update,
                (main_menu_cell_events.in_set(AppSet::MainMenuCell), main_menu_draw),
            );
    }
}

// ==================== 格子到主菜单按钮 ====================

fn cell_to_main_menu_button(x: usize, y: usize) -> Option<MainMenuButton> {
    if x < BUTTON_X_START || x >= BUTTON_X_END {
        return None;
    }
    match y {
        30 => Some(MainMenuButton::Start),
        32 => Some(MainMenuButton::Settings),
        34 => Some(MainMenuButton::Credits),
        36 => Some(MainMenuButton::Quit),
        _ => None,
    }
}

// ==================== 系统：响应画布格子事件（hover/click） ====================

fn main_menu_cell_events(
    mut state: ResMut<MainMenuState>,
    mut external: ResMut<StatusBarExternalHoverText>,
    mut ev_hover: EventReader<CellHoverEvent>,
    mut ev_press: EventReader<CellPressEvent>,
    mut ev_release: EventReader<CellReleaseEvent>,
) {
    for ev in ev_hover.read() {
        state.hovered = ev.cell.and_then(|(x, y)| cell_to_main_menu_button(x, y));
        // 主菜单按钮（图标或文字任意格）悬浮时设置状态栏显示完整按钮文本
        external.0 = state.hovered.map(|btn| btn.label().to_string());
    }
    for ev in ev_press.read() {
        state.pressed = cell_to_main_menu_button(ev.x, ev.y);
    }
    for ev in ev_release.read() {
        if let Some(btn) = state.pressed {
            if cell_to_main_menu_button(ev.x, ev.y) == Some(btn) {
                match btn {
                    MainMenuButton::Start => {
                        // TODO: 开始游戏
                    }
                    MainMenuButton::Settings => {
                        // TODO: 打开设置
                    }
                    MainMenuButton::Credits => {
                        // TODO: 制作人员
                    }
                    MainMenuButton::Quit => {
                        // TODO: 离开游戏
                    }
                }
            }
            state.pressed = None;
        }
    }
}

// ==================== 系统：绘制主菜单 ====================

fn main_menu_draw(
    mut canvas: ResMut<Canvas>,
    mut state: ResMut<MainMenuState>,
    theme: Res<crate::theme::Theme>,
) {
    let changed = state.hovered != state.prev_hovered || state.pressed != state.prev_pressed;
    if !changed {
        return;
    }

    let title_color = theme.text.primary;
    let normal_bg = theme.bg.basalt_blue;
    let hover_bg = theme.semantic.info.with_alpha(0.4);
    let press_bg = theme.semantic.info.with_alpha(0.6);

    let button_bg = |btn: MainMenuButton| -> Color {
        if state.pressed == Some(btn) {
            press_bg
        } else if state.hovered == Some(btn) {
            hover_bg
        } else {
            normal_bg
        }
    };

    // 标题 INTERSTICE：线条拼接的 ASCII 艺术字，每字母 5x5，# 为线条
    const TITLE_WORD: &str = "INTERSTICE";
    let letter_step = LETTER_W + LETTER_SPACING;
    let total_w = TITLE_WORD.len() * letter_step - LETTER_SPACING;
    let title_x_start = (CANVAS_WIDTH - total_w) / 2;

    for (row, _) in ART_I.iter().enumerate() {
        let y = TITLE_ART_ROW + row;
        if y >= CANVAS_HEIGHT {
            break;
        }
        let mut x = title_x_start;
        for c in TITLE_WORD.chars() {
            let art = title_art_letter(c);
            let line = art.get(row).copied().unwrap_or("     ");
            for (col, ch) in line.chars().take(LETTER_W).enumerate() {
                let cx = x + col;
                if cx < CANVAS_WIDTH {
                    let draw_ch = if ch == '#' { TITLE_BLOCK } else { ch };
                    canvas.set_char(cx, y, draw_ch, title_color);
                }
            }
            x += letter_step;
        }
    }

    // 四个按钮：Nerd icon + 右侧文本，带 hover/click 背景
    for &btn in &[
        MainMenuButton::Start,
        MainMenuButton::Settings,
        MainMenuButton::Credits,
        MainMenuButton::Quit,
    ] {
        let y = btn.row();
        let text = format!("{}  {}", btn.icon(), btn.label());
        canvas.set_line_with_bg(
            y,
            BUTTON_X_START,
            BUTTON_X_END,
            &text,
            TextAlign::Left,
            theme.text.primary,
            button_bg(btn),
        );
    }

    state.prev_hovered = state.hovered;
    state.prev_pressed = state.pressed;
}
